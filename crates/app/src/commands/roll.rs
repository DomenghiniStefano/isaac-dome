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
///
/// `write_failed` is the one thing a read of the store cannot discover on its own: the lock
/// can still succeed and `guard.roll()` can still return the old document, so a failed write
/// would otherwise be indistinguishable from "nothing changed" and no diagnostic would fire at
/// all. When it is `true` the reason is forced to `Unreadable`, whatever the read itself found.
fn view_now(
    app: &AppHandle,
    catalog: &CatalogState,
    resources: &ResourcesState,
    store: &StoreState,
    write_failed: bool,
) -> Result<ipc::RollView, IpcError> {
    let (flags, counters) = progress_sections(app)?;
    let rs = resources.get();
    let cat = rs.and_then(|rs| catalog.get_or_build(rs));
    let (document, read_reason) = read_document(app, store);
    let store_reason = if write_failed {
        Some(ipc::StoreReason::Unreadable)
    } else {
        read_reason
    };
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
    view_now(&app, &state, &resources, &store, false)
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
    // An unreadable document is replaced here too, exactly as `set_roll_preset` replaces one
    // below: pressing Pesca is asking for something new just as much as changing the preset
    // is, so there is nothing for the old, unparseable document to contribute.
    let mut doc = document.unwrap_or_default();
    let (space, playability_known) = ipc::roll_space(counters.as_deref(), flags.as_deref(), cat);
    // The same judgment `roll_view` makes for the count it reports, from the one place it is
    // written (`ipc::deck_preset`): the deck the draw picks from and the deck size the card
    // later reports must never be two independently maintained rules.
    let effective = ipc::deck_preset(&doc.preset, playability_known);
    let deck = roll::deck(&space, &effective);
    doc.current = roll::draw(&deck, seed_now()).map(|target| roll::Drawn {
        target,
        deck_size: deck.targets.len(),
        drawn_unix: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
    });
    let wrote = write_document(&app, &store, &doc);
    view_now(&app, &state, &resources, &store, !wrote)
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
    // An unreadable document is **not** overwritten by a read. It is overwritten by a preset
    // change or by a draw (`roll_draw`, above) — never by anything else — because both are the
    // user asking for something new, and replacing it is what they asked for (spec §7).
    // `queue_mutate` takes the opposite stance on the same question because there a parse
    // failure would destroy a plan a newer version wrote and this one cannot read; a roll
    // document holds nothing a freshly built default can't stand in for just as well.
    let mut doc = document.unwrap_or_default();
    doc.preset = ipc::preset_from_view(&preset);
    let wrote = write_document(&app, &store, &doc);
    view_now(&app, &state, &resources, &store, !wrote)
}

/// Writes and tells the other windows — but only once the write has actually landed: a failed
/// write changed nothing, so announcing it would tell every other window to re-read for no
/// reason. Reports whether it landed, so the caller's own `view_now` can say so when it didn't
/// — a lock that succeeds and a `set_roll` that fails would otherwise read back the old
/// document with no diagnostic at all, since nothing on disk actually changed.
fn write_document(app: &AppHandle, store: &StoreState, doc: &roll::Document) -> bool {
    let wrote = matches!(store.lock(app), Ok(guard) if guard.set_roll(doc).is_ok());
    if wrote {
        announce(app, ROLL_CHANGED);
    }
    wrote
}
