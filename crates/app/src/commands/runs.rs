//! The run archive, read out of the store. Wiring: the fold happened when the log was read.

use std::sync::Arc;

use tauri::AppHandle;

use catalog::Catalog;
use ipc::{IpcError, RunSource, RunView, RunsDiagnostic, RunsInputs, RunsView};

use crate::commands::graph::unlock_of;
use crate::state::{
    active_save, progress_sections, ArchiveState, CatalogState, GraphState, LiveUnlockState,
    ResourcesState, StoreState,
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
    // What the archive's own reading met comes first: it is why the list may be short.
    let mut diagnostics = archive.health().diagnostics();
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
    live_unlock: tauri::State<'_, LiveUnlockState>,
) -> Result<ipc::LiveView, IpcError> {
    let cat = resources.get(&app).and_then(|rs| catalog.get_or_build(rs));
    let open = open_run(&app, &store, &archive, cat);
    let unlocked = live_unlock_view(&app, &live_unlock, cat, cat.and_then(|c| graph.get(c)));
    let nodes = ipc::live_graph(unlocked.as_deref());

    let by_name = |name: &str, id: Option<u32>| -> Vec<(u32, String)> {
        cat.map(|c| ipc::characters_named(c, name, id))
            .unwrap_or_default()
    };

    // The row of the completion matrix for whoever is being played — two rows when the name
    // reaches two forms. Built from the same counters the Completion screen reads, through the
    // same function: a second reading would be a second chance to disagree with it.
    let marks = match (open.as_ref().and_then(|r| r.character.as_deref()), cat) {
        (Some(name), Some(c)) => progress_sections(&app).ok().and_then(|(_, counters)| {
            let counters = counters?;
            let matrix = ipc::marks_matrix(&counters, Some(c), crate::icons::icon_url);
            let wanted: Vec<u32> = by_name(name, open.as_ref().and_then(|r| r.character_id))
                .into_iter()
                .map(|(id, _)| id)
                .collect();
            let rows = ipc::live_mark_rows(c, &wanted);
            (!rows.is_empty()).then(|| ipc::live_marks(&matrix, &rows))
        }),
        _ => None,
    };

    Ok(ipc::live_view(open, nodes, marks, by_name))
}

/// The open run of the launch being followed, from that launch's own cached fold (card #80,
/// R10: this was the whole archive, read on every line the watcher reported). At most one: the
/// fold never leaves two runs open on the launch it is following. A store that cannot be read
/// is no run to show, the way it was when this read went through `runs`.
fn open_run(
    app: &AppHandle,
    store: &StoreState,
    archive: &ArchiveState,
    catalog: Option<&Catalog>,
) -> Option<RunView> {
    let guard = store.lock(app).ok()?;
    let runs = guard.live_runs(archive.rules().version()).ok()??;
    let inputs = RunsInputs {
        sources: vec![(RunSource::Live, runs)],
        catalog,
        diagnostics: Vec::new(),
    };
    ipc::runs_view(inputs, crate::icons::icon_url)
        .runs
        .into_iter()
        .find(|r| matches!(r.outcome, ipc::RunOutcomeView::Open))
}

/// The Unlock view Live compares the run against, evaluated once per read of the save rather
/// than once per line of the log. Without the catalog or the graph it is built every time:
/// "the game isn't installed" is never kept, so installing it mid-run is seen at the next line.
fn live_unlock_view(
    app: &AppHandle,
    state: &LiveUnlockState,
    catalog: Option<&Catalog>,
    g: Option<&graph::build::Graph>,
) -> Result<Arc<ipc::UnlockView>, IpcError> {
    let (_, save) = active_save(app)?;
    if let (Some(c), Some(g)) = (catalog, g) {
        return state
            .0
            .get(&save, || Ok(unlock_of(&save, Some(c), Some(g))));
    }
    Ok(Arc::new(unlock_of(&save, catalog, g)))
}
