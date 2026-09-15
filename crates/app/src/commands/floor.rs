//! The painted grid's ranking. Wiring only: the grid arrives from the screen, the rules are
//! embedded in `floor`, and nothing is read from disk — so there is no state to hold and no
//! failure that is not already a diagnostic.

use ipc::{FloorView, IpcError, RoomKindView};

#[tauri::command]
pub(crate) fn floor_candidates(cells: Vec<Option<RoomKindView>>) -> Result<FloorView, IpcError> {
    Ok(ipc::floor_view(cells))
}
