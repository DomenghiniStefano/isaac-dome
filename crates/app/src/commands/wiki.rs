//! The wiki and the search over it.

use tauri::AppHandle;

use core_save::Kind;
use ipc::IpcError;

use crate::icons::icon_url;
use crate::state::{
    active_save, catalog_now, discovery_now, CatalogState, ResourcesState, SearchState,
};
/// The wiki page for a target, if the dataset knows it. The embedded dataset failing to
/// load is an expected case, diagnosed elsewhere (`ExtractionReport.wiki`): here it's
/// enough to say the command can't answer.
#[tauri::command]
pub fn wiki_entry(target: ipc::Target) -> Result<Option<ipc::Entry>, IpcError> {
    let ds = wiki::Dataset::embedded().map_err(|_| IpcError::WikiUnavailable)?;
    Ok(ds.entry(&target).cloned())
}

/// The pools the installed game lists an item in (design decision 4 of
/// `2026-09-26-wiki-complete-design.md`): `None` without the game, or for a target that
/// isn't a collectible — a trinket included, since `itempools.xml` has no pools for those at
/// all (`ipc::item_pools`'s doc comment has the measurement). Not cached: the game turning up
/// after launch has to be seen the next time a page asks, the same as everything else behind
/// `catalog_now`.
#[tauri::command]
pub fn wiki_item_pools(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    target: ipc::Target,
) -> Result<Option<Vec<ipc::PoolMembershipView>>, IpcError> {
    let catalog = catalog_now(&app, &resources, &state);
    let ds = wiki::Dataset::embedded().ok();
    Ok(catalog.and_then(|c| ipc::item_pools(c, ds, &target)))
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
    let catalog = catalog_now(&app, &resources, &state);
    Ok(ipc::wiki_index(
        wiki::Dataset::embedded(),
        catalog,
        state.bosses(catalog),
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
    let catalog = catalog_now(&app, &resources, &state);
    let sections = active_save(&app)
        .ok()
        .map(|(_, s)| (s.flags(Kind::Achievements), s.flags(Kind::Items)));
    let flags = sections.as_ref().map(|(a, i)| ipc::SaveFlags {
        achievements: a.as_deref(),
        items: i.as_deref(),
    });
    let game = ipc::SearchCatalog {
        catalog,
        bosses: state.bosses(catalog),
        dataset: wiki::Dataset::embedded().ok(),
    };
    Ok(ipc::search(
        index.get(),
        &game,
        flags,
        &query,
        limit,
        icon_url,
    ))
}

/// The save's state for every wiki page that has one (design decision 6,
/// `2026-09-27-wiki-restyle-design.md`). `None` when no save is chosen — never cached, the
/// same rule every other command reads `active_save` under: a profile picked while the wiki
/// is open has to show up the next time this is asked, not after a restart.
#[tauri::command]
pub fn wiki_progress(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<Option<ipc::WikiProgress>, IpcError> {
    let catalog = catalog_now(&app, &resources, &state);
    let Ok((_, save)) = active_save(&app) else {
        return Ok(None);
    };
    let achievements = save.flags(Kind::Achievements);
    let items = save.flags(Kind::Items);
    let challenges = save.flags(Kind::Challenges);
    let counters = save.u32s(Kind::Counters);
    let bestiary = save.bestiary_tallies();
    Ok(Some(ipc::wiki_progress(ipc::WikiProgressInputs {
        dataset: wiki::Dataset::embedded(),
        catalog,
        achievements: achievements.as_deref(),
        items: items.as_deref(),
        challenges: challenges.as_deref(),
        counters: counters.as_deref(),
        bestiary: bestiary.as_ref(),
    })))
}
