//! Which piece of which game sheet draws a column's mark.
//!
//! Domain knowledge, so it sits beside `BOSSES` and follows its order (DESIGN-BRIEF.md §5.6,
//! backlog B13). It lived in `crates/design-export`'s `marks.json` while only the design pack
//! needed it; the app serves the symbols now, and one map in two places drifts.
//!
//! Eleven columns are layers of `completion_widget.anm2`. The twelfth, Delirium, is named
//! only by Repentance+'s online lobby, on its own sheet — and the lobby draws it twice, on
//! the player card and on the background, so there its animation is part of the key.

use catalog::{Anm2Frame, SpriteRef};

use crate::icon::MarkTier;

pub const WIDGET_ANM2: &str = "gfx/ui/completion_widget.anm2";
pub const LOBBY_ANM2: &str = "gfx/ui/main menu/onlinelobby.anm2";

#[derive(Debug, Clone, Copy)]
enum Source {
    Widget,
    Lobby,
}

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

/// One row per column of `BOSSES`, in its order. Every row but The Lamb is read from a layer
/// name; The Lamb is `Cross` by elimination, the one symbol and the one column left once every
/// other pairing is settled.
const MARK_LAYERS: [MarkLayer; 12] = [
    widget("Heart"),           // Mom's Heart
    widget("Polaroid"),        // Isaac
    widget("UpsideDownCross"), // Satan
    widget("Star"),            // Boss Rush
    widget("Negative"),        // Blue Baby
    widget("Cross"),           // The Lamb, by elimination
    widget("MegaSatan"),       // Mega Satan
    widget("Greed"),           // Greed
    widget("Hush"),            // Hush
    // Delirium: the online lobby's background, not its player card.
    MarkLayer {
        source: Source::Lobby,
        animation: Some("Background"),
        layer: "Completion_Delirium",
    },
    widget("Knife"),    // Mother
    widget("DadsNote"), // The Beast
];

/// The frames of the two anm2 files, as `catalog::anm2_frames` reads them.
#[derive(Debug, Clone, Default)]
pub struct MarkFrames {
    pub widget: Vec<Anm2Frame>,
    pub lobby: Vec<Anm2Frame>,
}

/// A tier is a frame of its layer: the rectangles cut into `heart_00.png` and
/// `heart_02.png`, drawn as the two levels on the Kit page. Frame 0 is declared hidden (the
/// "not taken" state); whether frame 1 repeats its rectangle waits for a machine with the
/// game, and if it doesn't, `Normal` becomes frame 1 here and nowhere else.
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

/// The piece that draws `column`'s mark at `tier`. `None` when the layer, the animation or the
/// frame isn't there: never a neighbour's picture.
pub fn mark_source(column: usize, tier: MarkTier, frames: &MarkFrames) -> Option<SpriteRef> {
    let m = MARK_LAYERS.get(column)?;
    let (anm2, list) = match m.source {
        Source::Widget => (WIDGET_ANM2, &frames.widget),
        Source::Lobby => (LOBBY_ANM2, &frames.lobby),
    };
    let wanted = frame_index(tier);
    let f = list.iter().find(|f| {
        f.layer == m.layer && m.animation.is_none_or(|a| f.animation == a) && f.index == wanted
    })?;
    Some(SpriteRef {
        path: sheet_path(anm2, &f.sheet),
        rect: Some(f.rect),
    })
}
