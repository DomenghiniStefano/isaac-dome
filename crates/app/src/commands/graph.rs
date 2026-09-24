//! The graph's three screens: Unlock, Next steps and the Collection.

use std::collections::BTreeSet;

use tauri::AppHandle;

use core_save::Kind;
use ipc::IpcError;

use crate::icons::icon_url;

use crate::state::{
    active_save, progress_sections, CatalogState, GraphState, ResourcesState, StoreState,
};

/// The Unlock view. Not a command since N8: it is built once per screen load, inside
/// `graph_views`, and a second entry point is a second reading of the profile.
pub(crate) fn unlock(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::UnlockView, IpcError> {
    let (flags, counters) = progress_sections(&app)?;
    // Game not installed is expected: the view goes out without a catalog and says so.
    let resources = resources.get(&app);
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

/// Both graph screens in one answer (N8). They used to be two commands, which the frontend
/// called together and which rebuilt the same pipeline twice — settings, a walk of the Steam
/// libraries, the `.dat` read whole and parsed, 642 nodes, the evaluation — for one screen
/// load. One command reads the profile once, by construction rather than by a cache.
///
/// The queue is read too, because the steps leave out what it already holds. A queue that
/// can't be read leaves nothing out: every suggestion shows, which is the screen as it was
/// before the queue existed, and the queue's own card says why it is missing.
#[tauri::command]
pub fn graph_views(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
    store: tauri::State<'_, StoreState>,
) -> Result<ipc::GraphViews, IpcError> {
    let queued: BTreeSet<u32> = match store.lock(&app).map(|guard| guard.queue()) {
        Ok(Ok(Ok(q))) => q.rows().iter().map(|r| r.achievement.0).collect(),
        _ => BTreeSet::new(),
    };
    Ok(ipc::graph_views(
        unlock(app, state, resources, graph)?,
        &queued,
    ))
}

#[tauri::command]
pub fn collection(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<ipc::CollectionView, IpcError> {
    let (_, save) = active_save(&app)?;
    // Game not installed is expected: the view goes out without a catalog and says so.
    let resources = resources.get(&app);
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
    let resources = resources.get(&app);
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

/// The forty-five challenges for the active profile. Wiring only: the join is `ipc`'s, and
/// the section's own length is what the totals state — never the constant 46.
#[tauri::command]
pub fn challenges(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<ipc::ChallengesView, IpcError> {
    let (_, save) = active_save(&app)?;
    // Game not installed is expected: the view goes out without a catalog and says so.
    let resources = resources.get(&app);
    let catalog = resources.and_then(|rs| state.get_or_build(rs));
    let challenges = save.flags(Kind::Challenges);
    let achievements = save.flags(Kind::Achievements);
    Ok(ipc::challenges_view(
        catalog,
        wiki::Dataset::embedded().ok(),
        challenges.as_deref(),
        achievements.as_deref(),
        icon_url,
    ))
}
