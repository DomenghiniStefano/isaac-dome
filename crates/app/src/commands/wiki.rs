//! The wiki and the search over it.

use tauri::AppHandle;

use core_save::Kind;
use ipc::IpcError;

use crate::icons::icon_url;
use crate::state::{active_save, discovery_now, CatalogState, ResourcesState, SearchState};
/// The wiki page for a target, if the dataset knows it. The embedded dataset failing to
/// load is an expected case, diagnosed elsewhere (`ExtractionReport.wiki`): here it's
/// enough to say the command can't answer.
#[tauri::command]
pub fn wiki_entry(target: ipc::Target) -> Result<Option<ipc::Entry>, IpcError> {
    let ds = wiki::Dataset::embedded().map_err(|_| IpcError::WikiUnavailable)?;
    Ok(ds.entry(&target).cloned())
}

/// Every page the dataset has, once per window: the category lists, the tab labels and
/// the icon of every reference on a page read from it (spec 3.5, Decision 2). A dataset
/// that didn't load is an empty index that says so, not an `Err`: the landing shows it.
#[tauri::command]
pub fn wiki_index(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<ipc::WikiIndex, IpcError> {
    let d = discovery_now(&app);
    let game_updated_unix = d.game.as_ref().and_then(|g| g.updated_unix);
    // No game is expected: the index goes out with no icon links, and the screen says so.
    let catalog = resources.get(&app).and_then(|rs| state.get_or_build(rs));
    Ok(ipc::wiki_index(
        wiki::Dataset::embedded(),
        catalog,
        game_updated_unix,
        icon_url,
    ))
}

/// One query over the wiki's text and the catalog's names. No profile is a **diagnostic**, not
/// an error: search answers before a save is chosen, and says the marks are unknown.
#[tauri::command]
pub fn search(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    index: tauri::State<'_, SearchState>,
    query: String,
    limit: usize,
) -> Result<ipc::SearchView, IpcError> {
    // Game not installed is expected: the answer goes out with wiki titles alone.
    let catalog = resources.get(&app).and_then(|rs| state.get_or_build(rs));
    let sections = active_save(&app)
        .ok()
        .map(|(_, s)| (s.flags(Kind::Achievements), s.flags(Kind::Items)));
    let flags = sections.as_ref().map(|(a, i)| ipc::SaveFlags {
        achievements: a.as_deref(),
        items: i.as_deref(),
    });
    Ok(ipc::search(
        index.get(),
        catalog,
        flags,
        &query,
        limit,
        icon_url,
    ))
}
