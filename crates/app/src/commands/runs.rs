//! The run archive, read out of the store. Wiring: the fold happened when the log was read.

use tauri::AppHandle;

use ipc::{IpcError, RunSource, RunsDiagnostic, RunsInputs, RunsView};
use store::SourceKind;

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
    let rs = resources.get();
    let catalog = rs.and_then(|rs| catalog.get_or_build(rs));
    let mut diagnostics = Vec::new();
    let mut sources = Vec::new();

    match store.lock(&app) {
        Ok(guard) => {
            let version = archive.rules().version();
            match guard.sources() {
                Ok(rows) => {
                    let mut unreadable = 0;
                    for row in rows {
                        let name = match (row.kind, &row.key) {
                            (SourceKind::Session, Some(n)) => {
                                RunSource::Session { name: n.clone() }
                            }
                            // A session row with no name cannot be told from a launch, and the
                            // launch is the honest reading: it is the source with no name.
                            (SourceKind::Session, None) | (SourceKind::Log, _) => RunSource::Live,
                        };
                        match guard.cached_runs(row.id, version) {
                            // Including an empty fold, which is a source read under these rules
                            // that holds no run: it contributes no row and no total, and saying
                            // so is cheaper than a second rule about which sources may be here.
                            Ok(Some(runs)) => sources.push((name, runs)),
                            // Since migration 5 this is one state and not three: nobody has
                            // folded this source under these rules. It is folded again the next
                            // time its log is read — which for a launch with no run in it used
                            // to be a promise that could not come true, because folding it
                            // produced nothing and nothing was what it had cached.
                            Ok(None) => {}
                            Err(_) => unreadable += 1,
                        }
                    }
                    if unreadable > 0 {
                        diagnostics.push(RunsDiagnostic::UnreadableEvents { count: unreadable });
                    }
                }
                Err(e) => diagnostics.push(RunsDiagnostic::StoreUnavailable {
                    reason: (&e).into(),
                }),
            }
        }
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

    let unlocked = crate::commands::graph::unlock(app, catalog.clone(), resources.clone(), graph);
    let nodes = match &unlocked {
        Ok(view) => ipc::LiveGraph::Nodes(&view.nodes),
        Err(IpcError::NoActiveProfile) => ipc::LiveGraph::NoProfile,
        Err(_) => ipc::LiveGraph::NoGraph,
    };

    // The catalog's own names, which is where the ambiguity comes from: a Tainted character
    // answers to the base form's name, so this hands back every character that name reaches
    // and `live_view` says there were two rather than choosing one.
    let rs = resources.get();
    let cat = rs.and_then(|rs| catalog.get_or_build(rs));
    // Given the id the log stated, exactly that character; given only a name, everyone who
    // answers to it — which is two whenever a Tainted form is involved, because the game gives
    // it the base form's name.
    let by_name = |name: &str, id: Option<u32>| -> Vec<(u32, String)> {
        let Some(c) = cat else { return Vec::new() };
        c.characters()
            .filter(|ch| match id {
                Some(id) => ch.id.0 == id,
                None => c.text(&ch.name, catalog::Language::English) == name,
            })
            .map(|ch| {
                (
                    ch.id.0,
                    c.text(&ch.name, catalog::Language::English).to_string(),
                )
            })
            .collect()
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
                    let rows: Vec<usize> = (0..ipc::CHARACTERS.len())
                        .filter(|row| {
                            ipc::character_for(*row, c).is_some_and(|ch| wanted.contains(&ch.id.0))
                        })
                        .collect();
                    (!rows.is_empty()).then(|| ipc::live_marks(&matrix, &rows))
                })
        }
        _ => None,
    };

    Ok(ipc::live_view(open, nodes, marks, by_name))
}
