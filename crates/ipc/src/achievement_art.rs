//! The sheet the game lays an achievement's drawing on.
//!
//! An achievement picture is dark strokes on transparency: the game never shows it bare. The
//! unlock popup, `achievements.anm2`, has a `Paper` layer — a torn, stained parchment — and an
//! `Achievement` layer on top whose sheet the game swaps for the drawing being unlocked. Read
//! on the installed game on 2026-10-03: both rest at the same origin in `Idle`, so the drawing
//! fills the paper edge to edge. The place is still read from the file and not written down
//! here, so a patch that moves it moves the picture with it.

use catalog::{Anm2Frame, SpriteRef};

use crate::mark_art::sprite_of;
use crate::sprite_png::overlay;

pub const ACHIEVEMENT_ANM2: &str = "gfx/ui/achievement/achievements.anm2";

/// The animation the popup rests in. `Appear` and `Dissapear` slide both layers across the
/// screen; this is the picture the player actually reads.
const RESTING: &str = "Idle";
const PAPER_LAYER: &str = "Paper";
const DRAWING_LAYER: &str = "Achievement";

/// The paper, and where the drawing's top-left sits on it, in the paper's own pixels.
#[derive(Debug, Clone)]
pub struct AchievementBacking {
    pub paper: SpriteRef,
    pub drawing_at: (i32, i32),
}

fn resting_frame<'a>(frames: &'a [Anm2Frame], layer: &str) -> Option<&'a Anm2Frame> {
    frames
        .iter()
        .find(|f| f.animation == RESTING && f.layer == layer && f.index == 0)
}

/// The popup's paper and the drawing's place on it. `None` when either layer is missing: no
/// paper is nothing to lay the drawing on, and no drawing layer is no place to lay it — a
/// default offset would be a guess that reads like a measurement.
pub fn achievement_backing(frames: &[Anm2Frame]) -> Option<AchievementBacking> {
    let paper = resting_frame(frames, PAPER_LAYER)?;
    let drawing = resting_frame(frames, DRAWING_LAYER)?;
    Some(AchievementBacking {
        paper: sprite_of(ACHIEVEMENT_ANM2, paper),
        drawing_at: (
            drawing.origin.x - paper.origin.x,
            drawing.origin.y - paper.origin.y,
        ),
    })
}

/// The picture an achievement is served as: its drawing laid on the paper at `at`.
///
/// **A paper that is missing or unreadable costs the backing, never the drawing**, which is
/// what the row is about — so the drawing comes back alone, the way it was served before
/// there was a paper. The two halves come from the same archive, so on a real install this is
/// the corrupted-file case and not the absent-game one: without the game there is no drawing
/// to serve either.
pub fn on_paper(paper: Option<&[u8]>, drawing: Vec<u8>, at: (i32, i32)) -> Vec<u8> {
    paper
        .and_then(|p| overlay(p, &[(drawing.as_slice(), at.0, at.1)]))
        .unwrap_or(drawing)
}
