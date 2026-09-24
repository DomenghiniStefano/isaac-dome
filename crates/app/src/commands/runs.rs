//! The run archive, read out of the store. Wiring: the fold happened when the log was read.

use tauri::AppHandle;

use ipc::{IpcError, RunSource, RunsDiagnostic, RunsInputs, RunsView};

use crate::state::{
    progress_sections, ArchiveState, CatalogState, GraphState, ResourcesState, StoreState,
};

#[tauri::command]
pub(crate) fn runs(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    archive: tauri::State<'_, ArchiveState>,
) -> Result<RunsView, IpcError> {
    let rs = resources.get(&app);
    let catalog = rs.and_then(|rs| catalog.get_or_build(rs));
    let mut diagnostics = Vec::new();
    let mut sources = Vec::new();

    match store.lock(&app) {
        Ok(guard) => match guard.archived_runs(archive.rules().version()) {
            Ok(archived) => {
                diagnostics.extend(archived.diagnostics());
                sources = archived.sources;
            }
            Err(e) => diagnostics.push(RunsDiagnostic::StoreUnavailable {
                reason: (&e).into(),
            }),
        },
        Err(reason) => diagnostics.push(RunsDiagnostic::StoreUnavailable { reason }),
    }

    Ok(ipc::runs_view(
        RunsInputs {
            sources,
            catalog,
            diagnostics,
        },
        crate::icons::icon_url,
    ))
}

/// What the run being watched would open (M4 2b). One command, because the archive's open run
/// and the graph's marks have to describe **the same profile**: two commands cannot promise
/// that, which is the lesson N8 spent itself on.
///
/// The run draws whatever the rest answers: a missing profile and a missing game are two
/// different sentences, and both leave the run on screen.
#[tauri::command]
pub(crate) fn live(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    archive: tauri::State<'_, ArchiveState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::LiveView, IpcError> {
    let app_for_marks = app.clone();
    let archive_view = runs(
        app.clone(),
        store,
        catalog.clone(),
        resources.clone(),
        archive,
    )?;
    // At most one: the fold never leaves two runs open on the launch it is following.
    let open = archive_view.runs.into_iter().find(|r| {
        matches!(r.source, RunSource::Live) && matches!(r.outcome, ipc::RunOutcomeView::Open)
    });

    let rs = resources.get(&app);
    let cat = rs.and_then(|rs| catalog.get_or_build(rs));
    let unlocked = crate::commands::graph::unlock(app, catalog.clone(), resources.clone(), graph);
    let nodes = ipc::live_graph(&unlocked);

    let by_name = |name: &str, id: Option<u32>| -> Vec<(u32, String)> {
        cat.map(|c| ipc::characters_named(c, name, id))
            .unwrap_or_default()
    };

    // The row of the completion matrix for whoever is being played — two rows when the name
    // reaches two forms. Built from the same counters the Completion screen reads, through the
    // same function: a second reading would be a second chance to disagree with it.
    let marks = match (open.as_ref().and_then(|r| r.character.as_deref()), cat) {
        (Some(name), Some(c)) => {
            progress_sections(&app_for_marks)
                .ok()
                .and_then(|(_, counters)| {
                    let counters = counters?;
                    let matrix = ipc::marks_matrix(&counters, Some(c), crate::icons::icon_url);
                    let wanted: Vec<u32> =
                        by_name(name, open.as_ref().and_then(|r| r.character_id))
                            .into_iter()
                            .map(|(id, _)| id)
                            .collect();
                    let rows = ipc::live_mark_rows(c, &wanted);
                    (!rows.is_empty()).then(|| ipc::live_marks(&matrix, &rows))
                })
        }
        _ => None,
    };

    Ok(ipc::live_view(open, nodes, marks, by_name))
}
