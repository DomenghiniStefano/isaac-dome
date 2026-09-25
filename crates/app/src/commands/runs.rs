//! The run archive, read out of the store. Wiring: the fold happened when the log was read.

use std::sync::Arc;

use tauri::AppHandle;

use catalog::Catalog;
use ipc::{IpcError, RunSource, RunView, RunsDiagnostic, RunsInputs, RunsView};

use crate::commands::graph::unlock_of;
use crate::state::{
    active_save, catalog_now, progress_sections, ArchiveState, CatalogState, GraphState,
    LiveUnlockState, ResourcesState, StoreState,
};

#[tauri::command]
pub(crate) fn runs(
    app: AppHandle,
    store: tauri::State<'_, StoreState>,
    catalog: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    archive: tauri::State<'_, ArchiveState>,
) -> Result<RunsView, IpcError> {
    let catalog = catalog_now(&app, &resources, &catalog);
    let (sources, read) = archived_sources(&app, &store, &archive);
    // What the archive's own reading met comes first: it is why the list may be short.
    let diagnostics = archive
        .health()
        .diagnostics()
        .into_iter()
        .chain(read)
        .collect();
    Ok(ipc::runs_view(
        RunsInputs {
            sources,
            catalog,
            diagnostics,
        },
        crate::icons::icon_url,
    ))
}

/// Every source's cached runs, and what reading them met. A database that will not open or
/// will not answer is no runs and one diagnostic, never an `Err`: the screen still draws.
fn archived_sources(
    app: &AppHandle,
    store: &StoreState,
    archive: &ArchiveState,
) -> (Vec<(RunSource, Vec<run::Run>)>, Vec<RunsDiagnostic>) {
    let read = store.lock(app).and_then(|guard| {
        guard
            .archived_runs(archive.rules().version())
            .map_err(|e| (&e).into())
    });
    match read {
        Ok(archived) => {
            let diagnostics = archived.diagnostics();
            (archived.sources, diagnostics)
        }
        Err(reason) => (
            Vec::new(),
            vec![RunsDiagnostic::StoreUnavailable { reason }],
        ),
    }
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
    let cat = catalog_now(&app, &resources, &catalog);
    let open = open_run(&app, &store, &archive, cat);
    let unlocked = live_unlock_view(
        &app,
        &live_unlock,
        cat,
        catalog.bosses(cat),
        cat.and_then(|c| graph.get(c)),
    );
    let nodes = ipc::live_graph(unlocked.as_deref());
    let marks = cat.and_then(|c| live_marks(&app, open.as_ref(), c));
    Ok(ipc::live_view(open, nodes, marks, |name, id| {
        characters_named(cat, name, id)
    }))
}

/// The characters a run's name and id reach, or none without the catalog to look them up in.
fn characters_named(catalog: Option<&Catalog>, name: &str, id: Option<u32>) -> Vec<(u32, String)> {
    catalog
        .map(|c| ipc::characters_named(c, name, id))
        .unwrap_or_default()
}

/// The row of the completion matrix for whoever is being played — two rows when the name
/// reaches two forms. Built from the same counters the Completion screen reads, through the
/// same function: a second reading would be a second chance to disagree with it. `None` with no
/// open run, no character named yet, no counters, or no row that matches.
fn live_marks(app: &AppHandle, open: Option<&RunView>, c: &Catalog) -> Option<ipc::LiveMarks> {
    let run = open?;
    let name = run.character.as_deref()?;
    let (_, counters) = progress_sections(app).ok()?;
    let matrix = ipc::marks_matrix(&counters?, Some(c), crate::icons::icon_url);
    let wanted: Vec<u32> = ipc::characters_named(c, name, run.character_id)
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    let rows = ipc::live_mark_rows(c, &wanted);
    (!rows.is_empty()).then(|| ipc::live_marks(&matrix, &rows))
}

/// The open run of the launch being followed, from that launch's own cached fold and not the
/// whole archive, which is every session ever played and would be read on every line the
/// watcher reports. At most one: the fold never leaves two runs open on the launch it is
/// following. A store that cannot be read is no run to show, as for the `runs` command.
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
    bosses: &ipc::BossKeys,
    g: Option<&graph::build::Graph>,
) -> Result<Arc<ipc::UnlockView>, IpcError> {
    let (_, save) = active_save(app)?;
    if let (Some(c), Some(g)) = (catalog, g) {
        return state
            .0
            .get(&save, || Ok(unlock_of(&save, Some(c), bosses, Some(g))));
    }
    Ok(Arc::new(unlock_of(&save, catalog, bosses, g)))
}
