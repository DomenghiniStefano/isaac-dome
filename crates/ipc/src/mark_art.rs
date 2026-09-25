//! Which piece of which game sheet draws a column's mark.
//!
//! Domain knowledge, so it sits beside `BOSSES` and is keyed by the same `Column` (DESIGN-BRIEF.md §5.6,
//! backlog B13). It lived in `crates/design-export`'s `marks.json` while only the design pack
//! needed it; the app serves the symbols now, and one map in two places drifts.
//!
//! Eleven columns are layers of `completion_widget.anm2`. The twelfth, Delirium, is named
//! only by Repentance+'s online lobby, on its own sheet — and the lobby draws it twice, on
//! the player card and on the background, so there its animation is part of the key.

use catalog::{Anm2Frame, SpriteRef};

use crate::icon::{MarkFill, MarkTier};
use crate::marks::BOSSES;
use core_save::marks::Column;

pub const WIDGET_ANM2: &str = "gfx/ui/completion_widget.anm2";
pub const LOBBY_ANM2: &str = "gfx/ui/main menu/onlinelobby.anm2";

/// The sheet the widget lays every one of its marks on, layer 0 of the same file.
///
/// **There is one, not one per mark.** Measured on the installed game on 2026-09-21: the
/// eleven symbol layers are placed *inside* this one — `Heart` at `22,7`, `Greed` at
/// `64,16`, `DadsNote` at `41,54` — so the paper is the picture's ground and never a cell's
/// background. Backlog B19 was written the other way round, and the file disagrees with it.
const PAPER_LAYER: &str = "Paper";

#[derive(Debug, Clone, Copy)]
enum Source {
    Widget,
    Lobby,
}

#[derive(Debug, Clone, Copy)]
struct MarkLayer {
    source: Source,
    /// `None` where the file has a single animation and the layer alone is unique.
    animation: Option<&'static str>,
    layer: &'static str,
}

const fn widget(layer: &'static str) -> MarkLayer {
    MarkLayer {
        source: Source::Widget,
        animation: None,
        layer,
    }
}

/// The layer that draws each column. Every column but The Lamb is read from a layer name; The
/// Lamb is `Cross` by elimination, the one symbol and the one column left once every other
/// pairing is settled.
///
/// A match over [`Column`] and not an array in its order (card #82, S1): the order used to be
/// held only by a comment at the end of each line, and a column inserted in the middle would
/// have shifted every symbol after it onto its neighbour.
const fn mark_layer(column: Column) -> MarkLayer {
    match column {
        Column::MomsHeart => widget("Heart"),
        Column::Isaac => widget("Polaroid"),
        Column::Satan => widget("UpsideDownCross"),
        Column::BossRush => widget("Star"),
        Column::BlueBaby => widget("Negative"),
        Column::TheLamb => widget("Cross"),
        Column::MegaSatan => widget("MegaSatan"),
        Column::Greed => widget("Greed"),
        Column::Hush => widget("Hush"),
        // The online lobby's background, not its player card.
        Column::Delirium => MarkLayer {
            source: Source::Lobby,
            animation: Some("Background"),
            layer: "Completion_Delirium",
        },
        Column::Mother => widget("Knife"),
        Column::TheBeast => widget("DadsNote"),
    }
}

/// The frames of the two anm2 files, as `catalog::anm2_frames` reads them.
#[derive(Debug, Clone, Default)]
pub struct MarkFrames {
    pub widget: Vec<Anm2Frame>,
    pub lobby: Vec<Anm2Frame>,
}

/// A tier is a frame of its layer, and the **same** frame for the paper as for a symbol:
/// one rule, two layers, so the sheet under a mark can never say a different level from the
/// mark on it.
///
/// Frame 0 is declared hidden — the "not taken" state. Whether frame 1 repeats its
/// rectangle waited for a machine with the game: **it does**, measured 2026-09-21 on the
/// installed copy. Every symbol layer declares frame 0 and frame 1 at the identical
/// rectangle (`Heart` both at `64,112`, `DadsNote` both at `176,112`), so reading `Normal`
/// as frame 0 takes the drawing frame 1 would have taken. The doubt is closed and the
/// mapping stays.
fn frame_index(tier: MarkTier) -> usize {
    match tier {
        MarkTier::Normal => 0,
        MarkTier::Hard => 2,
    }
}

/// A sheet named by an anm2 sits in the anm2's folder, which is how the game stores it.
fn sheet_path(anm2: &str, sheet: &str) -> String {
    match anm2.rsplit_once('/') {
        Some((dir, _)) => format!("{dir}/{sheet}"),
        None => sheet.to_string(),
    }
}

/// The frame a layer declares at `index`, inside one of the two files' frame lists.
fn frame_at<'a>(
    list: &'a [Anm2Frame],
    layer: &str,
    animation: Option<&str>,
    index: usize,
) -> Option<&'a Anm2Frame> {
    list.iter().find(|f| {
        f.layer == layer && animation.is_none_or(|a| f.animation == a) && f.index == index
    })
}

fn sprite_of(anm2: &str, f: &Anm2Frame) -> SpriteRef {
    SpriteRef {
        path: sheet_path(anm2, &f.sheet),
        rect: Some(f.rect),
    }
}

/// The frames of `column`'s layer, and which file they came from.
fn column_frames(
    column: usize,
    frames: &MarkFrames,
) -> Option<(MarkLayer, &'static str, &[Anm2Frame])> {
    let m = mark_layer(*Column::ALL.get(column)?);
    Some(match m.source {
        Source::Widget => (m, WIDGET_ANM2, &frames.widget),
        Source::Lobby => (m, LOBBY_ANM2, &frames.lobby),
    })
}

/// The piece that draws `column`'s mark at `tier`. `None` when the layer, the animation or the
/// frame isn't there: never a neighbour's picture.
pub fn mark_source(column: usize, tier: MarkTier, frames: &MarkFrames) -> Option<SpriteRef> {
    let (m, anm2, list) = column_frames(column, frames)?;
    let f = frame_at(list, m.layer, m.animation, frame_index(tier))?;
    Some(sprite_of(anm2, f))
}

/// The sheet the marks are laid on, at `tier`. Same file, same frame rule as a symbol.
pub fn paper_source(tier: MarkTier, frames: &MarkFrames) -> Option<SpriteRef> {
    let f = frame_at(&frames.widget, PAPER_LAYER, None, frame_index(tier))?;
    Some(sprite_of(WIDGET_ANM2, f))
}

/// The game's completion widget, as a list of pieces to lay on one another.
pub struct WidgetArt {
    /// The paper, and the whole picture's size: a mark is placed on it, never beyond it.
    pub paper: SpriteRef,
    /// Each mark to draw and its top-left **inside the paper**, in the paper's own pixels.
    pub marks: Vec<(SpriteRef, i32, i32)>,
}

/// The widget for a profile: the paper, with a symbol on it for every column that has one.
///
/// Three things this decides, and each of them is a decision rather than a reading:
///
/// - **The paper is the bloodied frame only when all twelve columns are `Hard`.** It is a
///   picture of the whole matrix, so eleven of twelve is not finished — even though the
///   twelfth is not drawn on it (below). The sheet carries six frames; the four this never
///   asks for have a meaning in the game that nobody here has measured, and giving the torn
///   one the sense of "halfway" would invent a third level the domain does not have.
/// - **An empty column draws nothing.** That is `MarkFill::None`'s whole point.
/// - **Delirium is not on this paper.** Its symbol lives in the online lobby's actor, whose
///   positions are measured in *that* actor's space: there is no offset that would put it
///   here, and inventing one would place it by eye. The widget the game draws has eleven
///   marks; the matrix under the band is where all twelve live.
///
/// `None` only when the paper is missing — the marks without their ground are pictures
/// floating on nothing, while the paper without a mark is still the truth about that column.
pub fn widget_source(fills: &[MarkFill; BOSSES.len()], frames: &MarkFrames) -> Option<WidgetArt> {
    let complete = fills.iter().all(|&f| f == MarkFill::Hard);
    let paper_frame = frame_at(
        &frames.widget,
        PAPER_LAYER,
        None,
        frame_index(if complete {
            MarkTier::Hard
        } else {
            MarkTier::Normal
        }),
    )?;
    let marks = fills
        .iter()
        .enumerate()
        .filter_map(|(column, fill)| {
            let tier = fill.tier()?;
            let (m, _, list) = column_frames(column, frames)?;
            // Only what this actor places. `Source::Lobby` is another actor's space.
            matches!(m.source, Source::Widget).then_some(())?;
            let f = frame_at(list, m.layer, m.animation, frame_index(tier))?;
            Some((
                sprite_of(WIDGET_ANM2, f),
                f.origin.x - paper_frame.origin.x,
                f.origin.y - paper_frame.origin.y,
            ))
        })
        .collect();
    Some(WidgetArt {
        paper: sprite_of(WIDGET_ANM2, paper_frame),
        marks,
    })
}
