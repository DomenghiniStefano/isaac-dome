//! The game's `.anm2` files: where to cut a sprite sheet.
//!
//! An `.anm2` is XML describing an animated actor. What we care about is not the
//! animation: it's that **every frame carries the rectangle of the sheet piece it
//! draws** (`XCrop`, `YCrop`, `Width`, `Height`). That's how the game cuts its sheets,
//! and therefore the only correct way for us to cut them too — the coordinates aren't
//! something you can guess.
//!
//! Real files come in three different shapes, and this module treats them all the same
//! way because they're the same thing seen from different angles:
//!
//! - `minimap_icons.anm2`: 41 one-frame animations — the animation name **is** the
//!   icon's name (`IconShop`, `IconDevilRoom`…);
//! - `hudstats.anm2`: two nine-frame animations — the name lives on the animation, and
//!   the individual piece is told apart by position;
//! - `completion_widget.anm2`: a single animation with twelve layers — `Paper` plus
//!   **eleven** of the twelve marks — and each layer's frames are its states. The twelfth
//!   mark, Delirium's, is a cell of the sheet that no layer claims, which is exactly the
//!   kind of thing a cutter driven by layers cannot see.
//!
//! Hence a single structure: a frame, with the name of the animation and layer that
//! contain it. It's up to the caller to decide what to do with it.

use crate::sprite::Rect;
use crate::strings::children_named;
use crate::xml::{elements, Element};

/// A frame of an `.anm2`: a crop, and where it comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anm2Frame {
    /// The name of the animation that contains it.
    pub animation: String,
    /// The layer's name. Can be empty: `hudstats2.anm2` leaves the `Name` field empty.
    pub layer: String,
    /// The sheet this frame crops from, as the anm2 names it.
    ///
    /// **It's not always the file's first sheet.** Every layer declares its own
    /// `SpritesheetId`, and `leaderboardmenu.anm2` uses four of them: taking the first
    /// one for all of them gives plausible-looking crops taken from the wrong place,
    /// which is the worst way to get it wrong because no one notices just by looking.
    pub sheet: String,
    /// The frame's position inside its layer, starting at 0.
    pub index: usize,
    /// Invisible frames are marks that have been "turned off" (not yet earned), not
    /// different pieces of the sheet: a normal cutter skips them, but knowing they
    /// exist matters.
    pub visible: bool,
    pub rect: Rect,
}

/// The sheets the file cites, in declaration order.
pub fn spritesheets(bytes: &[u8]) -> Vec<String> {
    let Ok(els) = elements(bytes) else {
        return Vec::new();
    };
    els.iter()
        .filter(|e| e.name == "Spritesheet")
        .filter_map(|e| e.attr("Path").map(str::to_string))
        .collect()
}

/// All the frames of the file, in document order.
///
/// A frame without the four coordinates is not a crop and is left out: in `.anm2`
/// files, the `<Frame>` elements inside `<RootAnimation>` are transforms of the whole
/// actor and have no `XCrop`. They're the reason the filter exists.
/// `None` when the file is not readable XML; `Some(vec![])` when it's valid but
/// contains no crops. The two cases are drawn differently, and this module has no
/// catalog source to attribute a diagnostic to: the caller knows which file it opened.
pub fn frames(bytes: &[u8]) -> Option<Vec<Anm2Frame>> {
    let els = elements(bytes).ok()?;
    let layers = layer_names(&els);
    let sheets = sheet_paths(&els);
    let mut out = Vec::new();
    for (i, e) in els.iter().enumerate() {
        if e.name != "LayerAnimation" {
            continue;
        }
        let animation = animation_of(&els, i).unwrap_or_default();
        let declared = e
            .attr("LayerId")
            .and_then(|id| layers.iter().find(|(k, _, _)| k == id));
        let layer = declared.map(|(_, n, _)| n.clone()).unwrap_or_default();
        // The layer's sheet; if the layer doesn't declare one, the file's first — the
        // only case where "the first one" is the right answer.
        let sheet = declared
            .and_then(|(_, _, s)| sheets.iter().find(|(k, _)| k == s))
            .map(|(_, p)| p.clone())
            .or_else(|| sheets.first().map(|(_, p)| p.clone()))
            .unwrap_or_default();
        for (index, f) in children_named(&els, i, "Frame").into_iter().enumerate() {
            let Some(rect) = rect_of(f) else { continue };
            out.push(Anm2Frame {
                animation: animation.clone(),
                layer: layer.clone(),
                sheet: sheet.clone(),
                index,
                visible: f.attr("Visible") != Some("false"),
                rect,
            });
        }
    }
    Some(out)
}

/// `(id, name, sheet id)` of the layers declared in `<Layers>`.
fn layer_names(els: &[Element]) -> Vec<(String, String, String)> {
    els.iter()
        .filter(|e| e.name == "Layer")
        .filter_map(|e| {
            Some((
                e.attr("Id")?.to_string(),
                e.attr("Name").unwrap_or_default().to_string(),
                e.attr("SpritesheetId").unwrap_or_default().to_string(),
            ))
        })
        .collect()
}

/// `(id, path)` of the sheets declared in `<Spritesheets>`.
fn sheet_paths(els: &[Element]) -> Vec<(String, String)> {
    els.iter()
        .filter(|e| e.name == "Spritesheet")
        .filter_map(|e| Some((e.attr("Id")?.to_string(), e.attr("Path")?.to_string())))
        .collect()
}

/// The animation that contains element `i`: the nearest enclosing `<Animation>`.
///
/// We look backward instead of scanning forward because elements arrive in document
/// order along with their depth, and the ancestor is the first one further up.
fn animation_of(els: &[Element], i: usize) -> Option<String> {
    let depth = els[i].depth;
    els[..i]
        .iter()
        .rev()
        .filter(|e| e.depth < depth)
        .find(|e| e.name == "Animation")
        .and_then(|e| e.attr("Name"))
        .map(str::to_string)
}

fn rect_of(e: &Element) -> Option<Rect> {
    let n = |name: &str| e.attr(name).and_then(|v| v.parse::<u32>().ok());
    Some(Rect {
        x: n("XCrop")?,
        y: n("YCrop")?,
        w: n("Width")?,
        h: n("Height")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mirrors `completion_widget.anm2`: a `<RootAnimation>` with no crops, two layers,
    /// and one layer with its first frame turned off.
    const PER_LAYER: &[u8] = br#"<AnimatedActor>
<Content><Spritesheets><Spritesheet Id="0" Path="completion_widget.png"/></Spritesheets>
<Layers><Layer Id="0" Name="Paper"/><Layer Id="1" Name="Heart"/></Layers></Content>
<Animations DefaultAnimation="Idle"><Animation Name="Idle">
<RootAnimation><Frame XPosition="0" YPosition="0" Visible="true"/></RootAnimation>
<LayerAnimations>
<LayerAnimation LayerId="0"><Frame XCrop="0" YCrop="0" Width="96" Height="96" Visible="true"/></LayerAnimation>
<LayerAnimation LayerId="1"><Frame XCrop="64" YCrop="112" Width="16" Height="16" Visible="false"/><Frame XCrop="64" YCrop="96" Width="16" Height="16" Visible="true"/></LayerAnimation>
</LayerAnimations></Animation></Animations></AnimatedActor>"#;

    /// Mirrors `minimap_icons.anm2`: a single layer, with the name on the animation.
    const PER_ANIMATION: &[u8] = br#"<AnimatedActor>
<Content><Spritesheets><Spritesheet Id="0" Path="minimap_icons.png"/></Spritesheets>
<Layers><Layer Id="0" Name="main"/></Layers></Content>
<Animations><Animation Name="IconShop"><LayerAnimations>
<LayerAnimation LayerId="0"><Frame XCrop="0" YCrop="0" Width="9" Height="9" Visible="true"/></LayerAnimation>
</LayerAnimations></Animation>
<Animation Name="IconDevilRoom"><LayerAnimations>
<LayerAnimation LayerId="0"><Frame XCrop="9" YCrop="0" Width="9" Height="9" Visible="true"/></LayerAnimation>
</LayerAnimations></Animation></Animations></AnimatedActor>"#;

    #[test]
    fn each_frame_carries_its_animation_and_layer_name() {
        let f = frames(PER_LAYER).expect("valid XML");
        assert_eq!(f.len(), 3, "three crops: {f:?}");
        assert_eq!(f[0].animation, "Idle");
        assert_eq!(f[0].layer, "Paper");
        assert_eq!(f[1].layer, "Heart");
        assert_eq!(f[1].index, 0);
        assert_eq!(f[2].index, 1, "the index restarts inside each layer");
    }

    #[test]
    fn the_root_animation_frame_is_not_a_crop() {
        // Has `Visible` but no `XCrop`: if it got in, every file would gain a
        // zero-sized rectangle matching nothing on the sheet.
        assert!(frames(PER_LAYER).unwrap().iter().all(|f| f.rect.w > 0));
    }

    #[test]
    fn the_hidden_frame_stays_but_is_flagged() {
        let f = frames(PER_LAYER).expect("valid XML");
        let heart: Vec<&Anm2Frame> = f.iter().filter(|f| f.layer == "Heart").collect();
        assert!(
            !heart[0].visible,
            "the first one is the not-yet-earned mark"
        );
        assert!(heart[1].visible);
    }

    #[test]
    fn with_a_single_layer_the_useful_name_is_the_animations() {
        let f = frames(PER_ANIMATION).expect("valid XML");
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].animation, "IconShop");
        assert_eq!(f[1].animation, "IconDevilRoom");
        assert_eq!(
            f[1].rect,
            Rect {
                x: 9,
                y: 0,
                w: 9,
                h: 9
            }
        );
        assert!(f.iter().all(|f| f.layer == "main"));
    }

    /// Mirrors `leaderboardmenu.anm2`: four sheets, and the layers use different ones.
    const MULTI_SHEET: &[u8] = br#"<AnimatedActor>
<Content><Spritesheets>
<Spritesheet Id="0" Path="LeaderboardMenu.png"/>
<Spritesheet Id="1" Path="LeaderboardItems.png"/>
<Spritesheet Id="3" Path="LeaderboardGoals.png"/>
</Spritesheets>
<Layers><Layer Id="0" Name="bg" SpritesheetId="0"/><Layer Id="7" Name="RightArrow" SpritesheetId="1"/><Layer Id="14" Name="Goal" SpritesheetId="3"/></Layers></Content>
<Animations><Animation Name="Idle"><LayerAnimations>
<LayerAnimation LayerId="0"><Frame XCrop="0" YCrop="0" Width="8" Height="8" Visible="true"/></LayerAnimation>
<LayerAnimation LayerId="7"><Frame XCrop="8" YCrop="0" Width="8" Height="8" Visible="true"/></LayerAnimation>
<LayerAnimation LayerId="14"><Frame XCrop="16" YCrop="0" Width="8" Height="8" Visible="true"/></LayerAnimation>
</LayerAnimations></Animation></Animations></AnimatedActor>"#;

    #[test]
    fn each_layer_crops_from_the_sheet_it_declares() {
        // The bug this test prevents: taking the first sheet for every layer gives
        // plausible-looking crops taken from the wrong place, invisible to the eye.
        let f = frames(MULTI_SHEET).expect("valid XML");
        assert_eq!(f.len(), 3);
        assert_eq!(f[0].sheet, "LeaderboardMenu.png");
        assert_eq!(f[1].sheet, "LeaderboardItems.png");
        assert_eq!(
            f[2].sheet, "LeaderboardGoals.png",
            "the Goal layer draws on the third sheet, not the first"
        );
    }

    #[test]
    fn with_a_single_sheet_every_frame_points_to_it() {
        let f = frames(PER_LAYER).expect("valid XML");
        assert!(f.iter().all(|f| f.sheet == "completion_widget.png"));
    }

    #[test]
    fn the_sheet_to_cut_is_read_from_the_file() {
        assert_eq!(spritesheets(PER_LAYER), vec!["completion_widget.png"]);
        assert_eq!(spritesheets(PER_ANIMATION), vec!["minimap_icons.png"]);
    }

    #[test]
    fn an_unreadable_file_and_one_without_crops_are_two_different_cases() {
        // A `None` and a `Some(vec![])` are drawn differently: the first is a broken
        // file, the second a sheet that genuinely declares no crops. Flattening both
        // into an empty list would make them indistinguishable to the caller.
        assert_eq!(frames(b"not xml <<<"), None);
        let vuoto: &[u8] = br#"<AnimatedActor><Content/><Animations/></AnimatedActor>"#;
        assert_eq!(frames(vuoto), Some(Vec::new()));
    }
}
