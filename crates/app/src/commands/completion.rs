//! The save's own numbers: the summary, the completion matrix, and what we got out of the archives.

use tauri::AppHandle;

use core_save::Kind;
use discovery::{discover, Options};
use ipc::{IpcError, MarksMatrix, SaveSummary};

use crate::icons::icon_url;

use crate::state::{active_save, CatalogState, ResourcesState};

#[tauri::command]
pub fn save_summary(app: AppHandle) -> Result<SaveSummary, IpcError> {
    let (id, save) = active_save(&app)?;
    Ok(ipc::save_summary(&id, &save))
}

#[tauri::command]
pub fn completion(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<MarksMatrix, IpcError> {
    let (_, save) = active_save(&app)?;
    let counters = save.u32s(Kind::Counters).unwrap_or_default();
    // Game not installed is expected: the matrix goes out without art, and the screen draws
    // the fallback outfit.
    let catalog = resources.get().and_then(|rs| state.get_or_build(rs));
    Ok(ipc::marks_matrix(&counters, catalog, icon_url))
}

/// How many icons to extract for the verification screen: a sample, not the whole catalog.
const SAMPLE_ICONS: usize = 60;

/// What we managed to extract from the game's archives. The catalog (names, sprites)
/// is the real one, built once and kept in `CatalogState`.
#[tauri::command]
pub fn extraction_report(
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<ipc::ExtractionReport, IpcError> {
    let d = discover(&Options::default());
    // The wiki dataset's status is independent of the game's archives: it's computed
    // regardless, even when the game isn't installed.
    let game_updated_unix = d.game.as_ref().and_then(|g| g.updated_unix);
    let wiki = ipc::wiki_info(wiki::Dataset::embedded(), game_updated_unix);
    // Game not installed is an expected case, not an error: the command still answers
    // and the report says there's nothing to extract.
    let Some(resources) = resources.get() else {
        return Ok(ipc::extraction_report(Vec::new(), None, Vec::new(), wiki));
    };
    let catalog = state.get_or_build(resources);

    let view = catalog.map(ipc::catalog_view);
    // The icons are extracted here, where I/O is allowed, and go out already resolved.
    let sprites = catalog
        .map(|c| {
            ipc::item_views(c, |p| resources.read(p), SAMPLE_ICONS)
                .into_iter()
                .filter_map(|i| i.data_url.map(|u| (i.id, i.name, u)))
                .collect()
        })
        .unwrap_or_default();

    Ok(ipc::extraction_report(
        ipc::archive_views(resources.archives()),
        view,
        sprites,
        wiki,
    ))
}
