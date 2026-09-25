//! What to play tonight, as the interface sees it: a preset, a deck's accounting, and the
//! card currently drawn — or the diagnostics that explain why any of that is thin.
//!
//! This is where the game's measured counts (`ipc::marks`) meet the pure `roll` crate: the
//! matrix's cells become a `roll::Space`, a preset becomes a `roll::Deck`, and everything the
//! deck excluded travels out named, never silently. `catalog: None` is the ordinary path here,
//! not an edge case — without the game's archives there is no catalog, so `NoCatalog` and
//! `PlayabilityUnknown` fire on every load, and the screen still has to answer: a target is an
//! index pair, not a sprite.

use crate::flags::playable_unless_locked;
use crate::graph::MarkColumnView;
use crate::icon::{IconRef, MarkTier};
use crate::marks::{cell_at, character_for, BOSSES, ROSTER};
use crate::StoreReason;
use catalog::Catalog;
use core_save::marks::Column;
use serde::{Deserialize, Serialize};

/// One target as the card names it: a mark names its column, a Greedier says so it is one —
/// there is no level number, because the second level only exists in the Greed column
/// (`roll::Target`'s own reasoning, carried into the view).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum DrawnTargetView {
    Mark { column: MarkColumnView },
    Greedier,
}

/// Recomputed from the save on every read, never stored: the card closes itself, and nothing
/// can be ticked off wrongly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum StatusView {
    Missing,
    Taken,
    Unreadable,
}

/// The card on screen, if a draw is on file and the matrix still holds its target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DrawnView {
    pub target: DrawnTargetView,
    pub character: String,
    /// The co-op menu head. `None` without a catalog, or when the game has none for this row.
    pub head_url: Option<String>,
    /// The mark's own symbol, at the tier the game draws for this cell. `None` without a
    /// catalog: a URL nothing can serve draws a broken image where the fallback belongs.
    pub art_url: Option<String>,
    pub status: StatusView,
    /// The deck at the moment of the draw. Stored, not recomputed: the deck as it stood then
    /// is gone once the save moves.
    pub deck_size: usize,
    pub drawn_unix: i64,
}

/// What a preset leaves to draw, and the size of everything it took out. `size + taken +
/// unreadable + locked + filtered` accounts for every target of the matrix — a property
/// checked in `crates/ipc/tests/roll.rs` and `roll_real.rs`, not a fact this type enforces on
/// its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct DeckView {
    pub size: usize,
    pub taken: usize,
    pub unreadable: usize,
    pub locked: usize,
    pub filtered: usize,
}

/// One row of either axis: a character or a column, with what ticking it would be worth. `id`
/// is the row's position — index *is* id, the way `roll::Contributions` already keys it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct RollRowView {
    pub id: u8,
    pub name: String,
    pub selected: bool,
    pub targets: usize,
}

/// Every value of an axis, or the ones named — the view's mirror of `roll::Selection`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SelectionView {
    All,
    Only { ids: Vec<u8> },
}

/// The view's mirror of `roll::Preset`. Also `Deserialize`: `set_roll_preset` takes
/// one inbound, and it is the only type on this screen that travels inward.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct PresetView {
    pub characters: SelectionView,
    pub columns: SelectionView,
    pub include_taken: bool,
    pub only_playable: bool,
}

/// Everything that can make the screen answer with less than the whole picture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RollDiagnostic {
    /// Section 2 of the save didn't read: every cell is `Unreadable`.
    NoCounterSection,
    /// The saved document didn't parse. The default preset was used instead.
    DocumentUnreadable,
    /// The document is a version newer than this binary reads. The default preset was used
    /// instead.
    DocumentFromTheFuture { version: u32, supported: u32 },
    /// The game isn't installed: no names, no art, no way to tell who is unlocked.
    NoCatalog,
    /// `only_playable` could not be applied — the catalog or the achievement flags are
    /// missing — so it was forced off for the deck build. The document's own choice still
    /// shows in `preset`: a filter that silently kept everything would be worse than one
    /// that admits it is off.
    PlayabilityUnknown,
    /// The database is unavailable, and which case it is.
    StoreUnavailable { reason: StoreReason },
    /// Nothing is left to draw, whether or not any reason above is why.
    EmptyDeck,
}

/// Everything `roll_view` needs. A struct rather than positional arguments, the same
/// reasoning `QueueInputs` carries.
pub struct RollInputs<'a> {
    pub counters: Option<&'a [u32]>,
    pub flags: Option<&'a [bool]>,
    pub catalog: Option<&'a Catalog>,
    pub document: Result<&'a roll::Document, &'a roll::DocumentError>,
    pub store_reason: Option<StoreReason>,
}

/// The whole screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct RollView {
    pub drawn: Option<DrawnView>,
    pub deck: DeckView,
    pub characters: Vec<RollRowView>,
    pub columns: Vec<RollRowView>,
    pub preset: PresetView,
    pub diagnostics: Vec<RollDiagnostic>,
}

/// A row the catalog does not name is playable: unknown must never hide a row. A character
/// unlocked by nothing (Isaac) is playable by definition; otherwise the flag decides, and an
/// achievement id past the end of the flags reads the same as "unknown" — playable.
fn row_playable(row: usize, catalog: &Catalog, flags: &[bool]) -> bool {
    let Some(character) = character_for(row, catalog) else {
        return true;
    };
    match character.unlocked_by {
        None => true,
        Some(id) => playable_unless_locked(flags, id.0),
    }
}

/// Whether each row of the matrix is a run you could actually start. `None` when either the
/// catalog or the achievement flags are missing — that is exactly what raises
/// `PlayabilityUnknown`, rather than a filter that silently keeps everything.
fn playable_rows(catalog: Option<&Catalog>, flags: Option<&[bool]>) -> Option<Vec<bool>> {
    let (catalog, flags) = (catalog?, flags?);
    Some(
        (0..ROSTER.len())
            .map(|row| row_playable(row, catalog, flags))
            .collect(),
    )
}

/// One cell of the matrix, read from the counters when they exist. `Cell::Known` stays;
/// `Cell::Unknown` and `Cell::Unexpected` both become `Unreadable` — a draw does not need to
/// tell "not located" from "suspicious value" apart, only `ipc::Cell`'s diagnostics screen
/// does.
fn cell_value(counters: Option<&[u32]>, character: usize, boss: usize) -> roll::CellValue {
    match counters.map(|c| cell_at(c, character, boss)) {
        Some(crate::Cell::Known { bits, .. }) => roll::CellValue::Known { bits },
        Some(crate::Cell::Unknown) | Some(crate::Cell::Unexpected { .. }) | None => {
            roll::CellValue::Unreadable
        }
    }
}

/// The all-`Unreadable` space of the same shape `roll_space` tried to build. Exists for the
/// error it is documented never to reach, rather than a panic standing in its place. Falls back
/// to `roll::Space::empty()` — infallible, no recursion — for the one impossibility further
/// than that: this constructor being refused too.
fn unreadable_space(rows: usize, columns: usize, greed: usize) -> roll::Space {
    roll::Space::new(
        rows,
        columns,
        greed,
        vec![roll::CellValue::Unreadable; rows * columns],
        vec![true; rows],
    )
    .unwrap_or_else(|_| roll::Space::empty())
}

/// The one judgment for which preset the deck is actually built with, shared by `roll_view`
/// (the count it reports) and `roll_draw` (the deck it draws from): "only playable" cannot be
/// applied when nothing says which characters are unlocked, so it is forced off there, while
/// the document's own choice always still shows on `preset`. Written once and called from both
/// sites — `crates/app` is untested by design, so a rule with two independently written copies
/// would let the two decks drift apart with nothing here to notice.
pub fn deck_preset(preset: &roll::Preset, playability_known: bool) -> roll::Preset {
    if playability_known {
        preset.clone()
    } else {
        roll::Preset {
            only_playable: false,
            ..preset.clone()
        }
    }
}

/// The matrix as a `roll::Space`, and whether playability could be determined at all.
///
/// `ROSTER.len()` x `BOSSES.len()`, with Greed at its own `Column::position` — the column
/// itself, not a name looked up in a table of names, which is what it was until card #82 and
/// fell back to Mom's Heart on a miss. The lengths come from the same tables `ipc::marks`
/// measured, so `Space::new` cannot fail here — the fallback exists for
/// the case that never happens, not for a panic to stand in its place.
pub fn roll_space(
    counters: Option<&[u32]>,
    flags: Option<&[bool]>,
    catalog: Option<&Catalog>,
) -> (roll::Space, bool) {
    let rows = ROSTER.len();
    let columns = BOSSES.len();
    let greed = Column::Greed.position();
    let cells: Vec<roll::CellValue> = (0..rows)
        .flat_map(|r| (0..columns).map(move |c| (r, c)))
        .map(|(r, c)| cell_value(counters, r, c))
        .collect();
    let known = playable_rows(catalog, flags);
    let playability_known = known.is_some();
    let playable = known.unwrap_or_else(|| vec![true; rows]);
    let space = roll::Space::new(rows, columns, greed, cells, playable)
        .unwrap_or_else(|_| unreadable_space(rows, columns, greed));
    (space, playability_known)
}

fn selection_view(selection: &roll::Selection) -> SelectionView {
    match selection {
        roll::Selection::All => SelectionView::All,
        roll::Selection::Only { ids } => SelectionView::Only { ids: ids.clone() },
    }
}

/// The view's mirror of a `roll::Preset`, named so its inbound twin `preset_from_view`
/// can sit beside it — the two halves of one mapping.
pub fn preset_view(preset: &roll::Preset) -> PresetView {
    PresetView {
        characters: selection_view(&preset.characters),
        columns: selection_view(&preset.columns),
        include_taken: preset.include_taken,
        only_playable: preset.only_playable,
    }
}

fn selection_from_view(view: &SelectionView) -> roll::Selection {
    match view {
        SelectionView::All => roll::Selection::All,
        SelectionView::Only { ids } => roll::Selection::Only { ids: ids.clone() },
    }
}

/// The inbound half of the mapping `preset_view` makes outbound: what `set_roll_preset`
/// receives from the client, turned into the type `roll` actually works with.
pub fn preset_from_view(view: &PresetView) -> roll::Preset {
    roll::Preset {
        characters: selection_from_view(&view.characters),
        columns: selection_from_view(&view.columns),
        include_taken: view.include_taken,
        only_playable: view.only_playable,
    }
}

fn status_view(status: roll::Status) -> StatusView {
    match status {
        roll::Status::Missing => StatusView::Missing,
        roll::Status::Taken => StatusView::Taken,
        roll::Status::Unreadable => StatusView::Unreadable,
    }
}

/// `None` for a column the matrix does not have. `drawn_view` has already dropped such a draw
/// through `space.status`, but the index comes from a document on disk, so it is read with
/// `get` rather than trusted.
fn target_view(target: roll::Target) -> Option<DrawnTargetView> {
    match target {
        roll::Target::Mark { column, .. } => Column::ALL
            .get(column as usize)
            .map(|&column| DrawnTargetView::Mark { column }),
        roll::Target::Greedier { .. } => Some(DrawnTargetView::Greedier),
    }
}

fn target_character(target: roll::Target) -> u8 {
    match target {
        roll::Target::Mark { character, .. } => character,
        roll::Target::Greedier { character } => character,
    }
}

/// The card's art reference: a mark's own symbol, at the tier the game draws for that cell —
/// normal for a `Mark`, hard for a `Greedier`, since the second level *is* the tier the game
/// draws.
fn target_icon(target: roll::Target, greed: usize) -> IconRef {
    match target {
        roll::Target::Mark { column, .. } => IconRef::Mark {
            column: column as usize,
            tier: MarkTier::Normal,
        },
        roll::Target::Greedier { .. } => IconRef::Mark {
            column: greed,
            tier: MarkTier::Hard,
        },
    }
}

/// The current card, if the document has one and the matrix still holds its target — a stored
/// draw the matrix no longer describes (a patch that shrank it) simply stops being drawn:
/// not an error, and not a card with an invented state.
fn drawn_view(
    current: Option<&roll::Drawn>,
    space: &roll::Space,
    catalog: Option<&Catalog>,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> Option<DrawnView> {
    let drawn = current?;
    let status = space.status(&drawn.target)?;
    let character = target_character(drawn.target);
    let head_url = catalog
        .and_then(|cat| character_for(character as usize, cat))
        .and_then(|ch| ch.head.as_ref())
        .and_then(|_| {
            icon(&IconRef::Head {
                row: character as usize,
            })
        });
    // Every URL is `None` without a catalog: a URL nothing can serve draws a broken image
    // where the fallback belongs.
    let art_url = catalog.and_then(|_| icon(&target_icon(drawn.target, space.greed_column())));
    Some(DrawnView {
        target: target_view(drawn.target)?,
        character: ROSTER[character as usize].name.to_string(),
        head_url,
        art_url,
        status: status_view(status),
        deck_size: drawn.deck_size,
        drawn_unix: drawn.drawn_unix,
    })
}

fn document_diagnostic(err: &roll::DocumentError) -> RollDiagnostic {
    match err {
        // A `Debug`-ish string on the IPC is untranslatable and is what the boundary rule
        // forbids: the reason stays in `roll`, and the two document failures are two
        // variants, with no payload beyond that.
        roll::DocumentError::Unreadable { .. } => RollDiagnostic::DocumentUnreadable,
        roll::DocumentError::FromTheFuture { version, supported } => {
            RollDiagnostic::DocumentFromTheFuture {
                version: *version,
                supported: *supported,
            }
        }
    }
}

fn row_view(id: usize, name: &str, selected: bool, targets: usize) -> RollRowView {
    RollRowView {
        id: id as u8,
        name: name.to_string(),
        selected,
        targets,
    }
}

/// The Roll screen, built from the save, the catalog and the saved document — every one of
/// them optional, because "the game isn't installed" and "the counters didn't read" are the
/// ordinary paths this screen has to answer on, not edge cases.
pub fn roll_view(
    inputs: RollInputs<'_>,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> RollView {
    let RollInputs {
        counters,
        flags,
        catalog,
        document,
        store_reason,
    } = inputs;

    let mut diagnostics = Vec::new();
    let document = match document {
        Ok(d) => d.clone(),
        Err(e) => {
            diagnostics.push(document_diagnostic(e));
            roll::Document::default()
        }
    };
    if counters.is_none() {
        diagnostics.push(RollDiagnostic::NoCounterSection);
    }
    if catalog.is_none() {
        diagnostics.push(RollDiagnostic::NoCatalog);
    }

    let (space, playability_known) = roll_space(counters, flags, catalog);
    if !playability_known {
        diagnostics.push(RollDiagnostic::PlayabilityUnknown);
    }
    if let Some(reason) = store_reason {
        diagnostics.push(RollDiagnostic::StoreUnavailable { reason });
    }

    // The document's own choice always shows in `preset`; only the deck build is forced off
    // when playability cannot be determined.
    let effective = deck_preset(&document.preset, playability_known);
    let deck = roll::deck(&space, &effective);
    let contributions = roll::contributions(&space, &effective);

    let characters = ROSTER
        .iter()
        .enumerate()
        .map(|(i, row)| {
            row_view(
                i,
                row.name,
                document.preset.characters.has(i as u8),
                contributions.characters.get(i).copied().unwrap_or(0),
            )
        })
        .collect();
    let columns = BOSSES
        .iter()
        .enumerate()
        .map(|(i, &name)| {
            row_view(
                i,
                name,
                document.preset.columns.has(i as u8),
                contributions.columns.get(i).copied().unwrap_or(0),
            )
        })
        .collect();

    let drawn = drawn_view(document.current.as_ref(), &space, catalog, &mut icon);

    // Last, and only when the deck actually is empty: whether or not any diagnostic above is
    // why, an empty deck is its own thing to say.
    if deck.targets.is_empty() {
        diagnostics.push(RollDiagnostic::EmptyDeck);
    }

    RollView {
        drawn,
        deck: DeckView {
            size: deck.targets.len(),
            taken: deck.excluded.taken,
            unreadable: deck.excluded.unreadable,
            locked: deck.excluded.locked,
            filtered: deck.excluded.filtered,
        },
        characters,
        columns,
        preset: preset_view(&document.preset),
        diagnostics,
    }
}

/// The document after a draw (card #81, V1: this was the body of the `roll_draw` command). The
/// deck is the preset's, made effective by what the save can tell — the same judgment
/// `roll_view` makes for the count it reports, from the one place it is written — and the draw
/// records that deck's size and the time it was handed. The seed and the time come from `app`:
/// `roll::draw` is a function of `(deck, seed)`, and so is this.
pub fn drawn_document(
    mut doc: roll::Document,
    counters: Option<&[u32]>,
    flags: Option<&[bool]>,
    catalog: Option<&Catalog>,
    seed: u64,
    drawn_unix: i64,
) -> roll::Document {
    let (space, playability_known) = roll_space(counters, flags, catalog);
    let effective = deck_preset(&doc.preset, playability_known);
    let deck = roll::deck(&space, &effective);
    doc.current = roll::draw(&deck, seed).map(|target| roll::Drawn {
        target,
        deck_size: deck.targets.len(),
        drawn_unix,
    });
    doc
}
