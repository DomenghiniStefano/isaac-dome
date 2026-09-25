//! The painted grid's ranking, and the pictures the grid draws with.
//!
//! Wiring only: the grid arrives from the screen, the rules are embedded in `floor`, and the
//! icons are names the pure crate decides. Nothing here judges anything.

use ipc::{FloorView, IpcError, RoomIconView, RoomKindView};

use crate::icons::icon_url;
use crate::state::ResourcesState;

#[tauri::command]
pub(crate) fn floor_candidates(cells: Vec<Option<RoomKindView>>) -> Result<FloorView, IpcError> {
    Ok(ipc::floor_view(cells))
}

/// The fourteen room kinds with the game's own minimap icon for each.
///
/// Its own command rather than a field on `FloorView`: the icons do not depend on what is
/// painted, and `floor_candidates` runs again for every cell a dragged pointer crosses.
///
/// **No game, no URL.** The screen falls back to its own symbols either way, but a link that
/// is certain to 404 makes it fall back a frame *later* — every cell drawn once empty and then
/// again with the symbol, on a machine that never had the picture to begin with.
#[tauri::command]
pub(crate) fn room_icons(
    app: tauri::AppHandle,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<Vec<RoomIconView>, IpcError> {
    let installed = resources.get(&app).is_some();
    Ok(ipc::room_icons(
        |r| {
            if installed {
                icon_url(r)
            } else {
                None
            }
        },
    ))
}
