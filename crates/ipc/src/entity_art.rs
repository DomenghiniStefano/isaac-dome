//! Composes a non-boss entity's own picture from its `.anm2`: every layer's first visible
//! frame of the entity's default animation, laid out by the layer's own origin.
//!
//! `mark_art::widget_source` composes the completion widget the same way — several pieces of
//! one file, placed by the difference of their origins — except the widget has a "paper" to
//! give the picture its size and this doesn't: an entity is drawn on nothing, so the canvas
//! is the bounding box of what its layers actually draw. `sprite_of` (which itself reads
//! `sheet_path`) is `mark_art`'s: one reading of "which file a layer's sheet lives in", not
//! two.

use catalog::{Anm2Frame, SpriteRef};

use crate::mark_art::sprite_of;

/// A picture composed of several pieces, and the canvas they are placed on: what `overlay`
/// needs once a blank canvas of `width`x`height` is built for it (`sprite_png::blank_canvas`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposedArt {
    pub width: u32,
    pub height: u32,
    /// Each piece and its top-left **inside the canvas**, in the canvas's own pixels.
    pub layers: Vec<(SpriteRef, i32, i32)>,
}

/// Composes `anm2_path`'s `default_animation`: the first visible frame of every layer that
/// animation draws, positioned by each frame's own origin. `None` when the animation draws
/// nothing visible, or the layers' combined box has no area — a picture with nothing in it
/// is the same as no picture.
pub fn compose(
    anm2_path: &str,
    frames: &[Anm2Frame],
    default_animation: &str,
) -> Option<ComposedArt> {
    let selected = first_visible_per_layer(frames, default_animation);
    let (min_x, min_y, max_x, max_y) = bounds(&selected)?;
    let width = u32::try_from(max_x - min_x).ok().filter(|&w| w > 0)?;
    let height = u32::try_from(max_y - min_y).ok().filter(|&h| h > 0)?;
    let layers = selected
        .into_iter()
        .map(|f| {
            let x = i32::try_from(i64::from(f.origin.x) - min_x).ok()?;
            let y = i32::try_from(i64::from(f.origin.y) - min_y).ok()?;
            Some((sprite_of(anm2_path, f), x, y))
        })
        .collect::<Option<Vec<_>>>()?;
    Some(ComposedArt {
        width,
        height,
        layers,
    })
}

/// One frame per layer of `animation`: the first one written **visible**, in the file's own
/// order. A layer with no visible frame in this animation draws nothing — an overlay that
/// is "off" in the entity's resting pose (a costume piece, an alternate colour) rather than
/// a missing crop.
fn first_visible_per_layer<'a>(frames: &'a [Anm2Frame], animation: &str) -> Vec<&'a Anm2Frame> {
    let mut seen: Vec<&str> = Vec::new();
    frames
        .iter()
        .filter(|f| f.animation == animation && f.visible)
        .filter(|f| {
            if seen.contains(&f.layer.as_str()) {
                false
            } else {
                seen.push(f.layer.as_str());
                true
            }
        })
        .collect()
}

/// The box every selected frame occupies together, `origin` to `origin + rect`. `i64`, not
/// `u32`: an origin can be negative, and the box has to be measured before it is known to fit
/// a canvas at all. `origin` is bounded to `i32` and `rect` to `u32`, so the sum never comes
/// close to overflowing `i64` — the width this function's own caller checks against `u32` is
/// the bound that can fail, not this one.
fn bounds(frames: &[&Anm2Frame]) -> Option<(i64, i64, i64, i64)> {
    frames
        .iter()
        .map(|f| {
            let x0 = i64::from(f.origin.x);
            let y0 = i64::from(f.origin.y);
            (x0, y0, x0 + i64::from(f.rect.w), y0 + i64::from(f.rect.h))
        })
        .reduce(|(min_x, min_y, max_x, max_y), (x0, y0, x1, y1)| {
            (min_x.min(x0), min_y.min(y0), max_x.max(x1), max_y.max(y1))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use catalog::{Point, Rect};

    fn frame(
        animation: &str,
        layer: &str,
        index: usize,
        visible: bool,
        rect: Rect,
        origin: Point,
    ) -> Anm2Frame {
        Anm2Frame {
            animation: animation.to_string(),
            layer: layer.to_string(),
            sheet: "gaper.png".to_string(),
            index,
            visible,
            rect,
            origin,
        }
    }

    fn rect(w: u32, h: u32) -> Rect {
        Rect { x: 0, y: 0, w, h }
    }

    #[test]
    fn one_frame_per_layer_composes_a_canvas_sized_to_their_union() {
        let frames = vec![
            frame("Idle", "Body", 0, true, rect(16, 16), Point { x: 0, y: 0 }),
            frame("Idle", "Head", 0, true, rect(10, 10), Point { x: 3, y: -4 }),
        ];
        let art = compose("gfx/010_gaper.anm2", &frames, "Idle").expect("two visible layers");
        // The box runs from (0, -4) to (16, 16): width 16, height 20.
        assert_eq!((art.width, art.height), (16, 20));
        assert_eq!(art.layers.len(), 2);
        let body = art
            .layers
            .iter()
            .find(|(s, ..)| s.path.ends_with("gaper.png"))
            .unwrap();
        assert_eq!(
            (body.1, body.2),
            (0, 4),
            "the body sits 4px below the canvas top"
        );
    }

    #[test]
    fn a_layer_hidden_in_the_default_animation_draws_nothing() {
        let frames = vec![
            frame("Idle", "Body", 0, true, rect(16, 16), Point { x: 0, y: 0 }),
            frame(
                "Idle",
                "Extra",
                0,
                false,
                rect(16, 16),
                Point { x: 0, y: 0 },
            ),
        ];
        let art = compose("gfx/x.anm2", &frames, "Idle").unwrap();
        assert_eq!(art.layers.len(), 1);
    }

    #[test]
    fn only_the_first_visible_frame_of_a_layer_is_taken() {
        let frames = vec![
            frame("Idle", "Body", 0, false, rect(16, 16), Point { x: 0, y: 0 }),
            frame("Idle", "Body", 1, true, rect(8, 8), Point { x: 2, y: 2 }),
            frame("Idle", "Body", 2, true, rect(20, 20), Point { x: 0, y: 0 }),
        ];
        let art = compose("gfx/x.anm2", &frames, "Idle").unwrap();
        assert_eq!(art.layers.len(), 1);
        assert_eq!((art.width, art.height), (8, 8), "frame 1, not frame 2");
    }

    #[test]
    fn frames_of_another_animation_are_not_composed() {
        let frames = vec![
            frame(
                "Appear",
                "Body",
                0,
                true,
                rect(16, 16),
                Point { x: 0, y: 0 },
            ),
            frame("Idle", "Body", 0, true, rect(10, 10), Point { x: 0, y: 0 }),
        ];
        let art = compose("gfx/x.anm2", &frames, "Idle").unwrap();
        assert_eq!((art.width, art.height), (10, 10));
    }

    #[test]
    fn nothing_visible_in_the_default_animation_composes_to_nothing() {
        let frames = vec![frame(
            "Idle",
            "Body",
            0,
            false,
            rect(16, 16),
            Point { x: 0, y: 0 },
        )];
        assert_eq!(compose("gfx/x.anm2", &frames, "Idle"), None);
    }

    #[test]
    fn an_animation_the_file_does_not_have_composes_to_nothing() {
        let frames = vec![frame(
            "Idle",
            "Body",
            0,
            true,
            rect(16, 16),
            Point { x: 0, y: 0 },
        )];
        assert_eq!(compose("gfx/x.anm2", &frames, "Walk"), None);
    }

    #[test]
    fn the_sheet_sits_beside_the_anm2_the_same_way_the_widget_reads_it() {
        let frames = vec![frame(
            "Idle",
            "Body",
            0,
            true,
            rect(16, 16),
            Point { x: 0, y: 0 },
        )];
        let art = compose("gfx/monsters/010_gaper.anm2", &frames, "Idle").unwrap();
        assert_eq!(art.layers[0].0.path, "gfx/monsters/gaper.png");
    }
}
