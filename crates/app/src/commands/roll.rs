//! The draw: read, draw again, change the preset. Three commands, each answering with the
//! whole `RollView` (N8) — one screen, one round trip, whatever the user just did.

use std::time::{SystemTime, UNIX_EPOCH};

use tauri::AppHandle;

use ipc::IpcError;

use crate::events::{announce, ROLL_CHANGED};
use crate::icons::icon_url;
use crate::state::*;

/// Everything the view needs, read once. The catalog may be absent — the game is not
/// installed — and that is a diagnostic, never an error.
fn view_now(
    app: &AppHandle,
    catalog: &CatalogState,
    resources: &ResourcesState,
    store: &StoreState,
) -> Result<ipc::RollView, IpcError> {
    let (flags, counters) = progress_sections(app)?;
    let rs = resources.get();
    let cat = rs.and_then(|rs| catalog.get_or_build(rs));
    let (document, store_reason) = read_document(app, store);
    Ok(ipc::roll_view(
        ipc::RollInputs {
            counters: counters.as_deref(),
            flags: flags.as_deref(),
            catalog: cat,
            document: document.as_ref(),
            store_reason,
        },
        icon_url,
    ))
}

/// The saved document, or the default one with the reason the database could not give it.
/// Extracted so the three commands read it the same way, and so the `if` that decides it is
/// not inlined into a call.
fn read_document(
    app: &AppHandle,
    store: &StoreState,
) -> (
    Result<roll::Document, roll::DocumentError>,
    Option<ipc::StoreReason>,
) {
    match store.lock(app) {
        Ok(guard) => match guard.roll() {
            Ok(inner) => (inner, None),
            Err(e) => (Ok(roll::Document::default()), Some((&e).into())),
        },
        Err(reason) => (Ok(roll::Document::default()), Some(reason)),
    }
}

/// Nanoseconds since the epoch. The clock lives here and nowhere else: `roll::draw` is a
/// function of `(deck, seed)`, which is what makes it testable at all.
fn seed_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        // A clock before 1970 is not a reason to refuse a draw.
        .unwrap_or(0)
}

#[tauri::command]
pub fn roll(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    store: tauri::State<'_, StoreState>,
) -> Result<ipc::RollView, IpcError> {
    view_now(&app, &state, &resources, &store)
}

#[tauri::command]
pub fn roll_draw(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    store: tauri::State<'_, StoreState>,
) -> Result<ipc::RollView, IpcError> {
    let (flags, counters) = progress_sections(&app)?;
    let rs = resources.get();
    let cat = rs.and_then(|rs| state.get_or_build(rs));
    let (document, _) = read_document(&app, &store);
    let mut doc = document.unwrap_or_default();
    let (space, playability_known) = ipc::roll_space(counters.as_deref(), flags.as_deref(), cat);
    // The preset the deck is actually built with: "only playable" cannot be applied when
    // nothing says which characters are unlocked, and `ipc` reports that on the view.
    let effective = roll::Preset {
        only_playable: doc.preset.only_playable && playability_known,
        ..doc.preset.clone()
    };
    let deck = roll::deck(&space, &effective);
    doc.current = roll::draw(&deck, seed_now()).map(|target| roll::Drawn {
        target,
        deck_size: deck.targets.len(),
        drawn_unix: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
    });
    write_document(&app, &store, &doc);
    view_now(&app, &state, &resources, &store)
}

#[tauri::command]
pub fn set_roll_preset(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    store: tauri::State<'_, StoreState>,
    preset: ipc::PresetView,
) -> Result<ipc::RollView, IpcError> {
    let (document, _) = read_document(&app, &store);
    // An unreadable document is **not** overwritten by a read. It is overwritten here, and
    // only here, because the user just changed something: that is the one moment where
    // replacing it is what they asked for (spec §7).
    let mut doc = document.unwrap_or_default();
    doc.preset = ipc::preset_from_view(&preset);
    write_document(&app, &store, &doc);
    view_now(&app, &state, &resources, &store)
}

/// Writes and tells the other windows. A write that cannot land is not an error the screen can
/// act on — the view that follows is read from the same store and will show what is really
/// there.
fn write_document(app: &AppHandle, store: &StoreState, doc: &roll::Document) {
    if let Ok(guard) = store.lock(app) {
        let _ = guard.set_roll(doc);
    }
    announce(app, ROLL_CHANGED);
}
