//! The graph's three screens: Unlock, Next steps and the Collection.

use tauri::AppHandle;

use core_save::Kind;
use ipc::IpcError;

use crate::icons::icon_url;

use crate::state::*;

#[tauri::command]
pub fn unlock(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::UnlockView, IpcError> {
    let (flags, counters) = progress_sections(&app)?;
    // Game not installed is expected: the view goes out without a catalog and says so.
    let resources = resources.get();
    let catalog = resources.and_then(|rs| state.get_or_build(rs));
    let g = catalog.and_then(|c| graph.get(c));
    let progress = ipc::SaveProgress::new(flags.as_deref(), counters.as_deref(), catalog);
    let eval = g.map(|g| g.evaluate(&progress));
    Ok(ipc::unlock_view(
        catalog,
        wiki::Dataset::embedded().ok(),
        flags.as_deref(),
        g,
        eval.as_ref(),
        Some(&progress),
        icon_url,
    ))
}

#[tauri::command]
pub fn next_steps(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::NextSteps, IpcError> {
    // The steps are a filter over the full view: same state, no extra work.
    let view = unlock(app, state, resources, graph)?;
    Ok(ipc::next_steps(&view))
}

#[tauri::command]
pub fn collection(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<ipc::CollectionView, IpcError> {
    let (_, save) = active_save(&app)?;
    // Game not installed is expected: the view goes out without a catalog and says so.
    let resources = resources.get();
    let catalog = resources.and_then(|rs| state.get_or_build(rs));
    let items = save.flags(Kind::Items);
    let achievements = save.flags(Kind::Achievements);
    Ok(ipc::collection_view(
        catalog,
        wiki::Dataset::embedded().ok(),
        items.as_deref(),
        achievements.as_deref(),
        icon_url,
    ))
}

/// "I want this — what do I have to play?". The same state `unlock` reads, asked from the
/// other end: no profile and no game are expected answers and travel in the payload.
#[tauri::command]
pub fn want(
    app: AppHandle,
    target: wiki::Target,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::WantView, IpcError> {
    let (flags, counters) = progress_sections(&app)?;
    // Game not installed is expected: the view goes out without a catalog and says so.
    let resources = resources.get();
    let catalog = resources.and_then(|rs| state.get_or_build(rs));
    let g = catalog.and_then(|c| graph.get(c));
    let progress = ipc::SaveProgress::new(flags.as_deref(), counters.as_deref(), catalog);
    let eval = g.map(|g| g.evaluate(&progress));
    let view = ipc::unlock_view(
        catalog,
        wiki::Dataset::embedded().ok(),
        flags.as_deref(),
        g,
        eval.as_ref(),
        Some(&progress),
        icon_url,
    );
    Ok(ipc::want_view(
        catalog,
        &view,
        flags.as_deref(),
        g,
        &target,
        icon_url,
    ))
}
