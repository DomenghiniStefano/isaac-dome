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
//!   **eleven** of the twelve marks — and each layer's frames are its states. Delirium's
//!   mark is missing from it entirely, and is named only in Repentance+'s online lobby,
//!   `main menu/onlinelobby.anm2`, as `Completion_Delirium`. A sheet can hold a drawing no
//!   layer of *its own* file points at, which is exactly what a cutter driven by layers
//!   cannot see.
//!
//! Hence a single structure: a frame, with the name of the animation and layer that
//! contain it. It's up to the caller to decide what to do with it.

use crate::sprite::{Point, Rect};
use crate::xml::{children_named, elements, Element};

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
    /// Where the layer puts the crop, as `position - pivot` in the actor's own space.
    ///
    /// The rectangle says *what* to cut; this says *where it goes*, and only a picture made
    /// of several layers needs it — one icon is served on its own and lands wherever the
    /// interface puts it. The completion widget is the case that needs it: one paper with
    /// eleven marks laid on it, each at its own place, and the mark's offset inside the
    /// paper is the difference of the two origins.
    ///
    /// A frame that declares no position sits at `0,0`: absent is not a refusal.
    pub origin: Point,
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
    Some(
        layer_animations(&els)
            .into_iter()
            .flat_map(crops_of)
            .collect(),
    )
}

/// A `<LayerAnimation>` as the file declares it: the animation that holds it, the layer and
/// sheet it draws, and **every** `<Frame>` it has, crop or not, in order.
///
/// Shared by [`frames`], which keeps the crops, and `heads`, which needs the frames without
/// one too: its map indexes a layer by position.
pub(crate) struct LayerAnimation<'a> {
    pub animation: String,
    /// The `LayerId` attribute as written, `None` when there is none.
    pub layer_id: Option<&'a str>,
    pub layer: String,
    pub sheet: String,
    pub frames: Vec<&'a Element>,
}

/// Every `<LayerAnimation>` of `els`, in document order.
pub(crate) fn layer_animations(els: &[Element]) -> Vec<LayerAnimation<'_>> {
    let layers = layer_names(els);
    let sheets = sheet_paths(els);
    els.iter()
        .enumerate()
        .filter(|(_, e)| e.name == "LayerAnimation")
        .map(|(i, e)| {
            let layer_id = e.attr("LayerId");
            let declared = layer_id.and_then(|id| layers.iter().find(|(k, _, _)| k == id));
            LayerAnimation {
                animation: animation_of(els, i).unwrap_or_default(),
                layer_id,
                layer: declared.map(|(_, n, _)| n.clone()).unwrap_or_default(),
                sheet: sheet_of(declared.map(|(_, _, s)| s.as_str()), &sheets),
                frames: children_named(els, i, "Frame"),
            }
        })
        .collect()
}

/// The layer's sheet; if the layer doesn't declare one, the file's first — the only case
/// where "the first one" is the right answer.
fn sheet_of(declared: Option<&str>, sheets: &[(String, String)]) -> String {
    declared
        .and_then(|s| sheets.iter().find(|(k, _)| k == s))
        .or_else(|| sheets.first())
        .map(|(_, p)| p.clone())
        .unwrap_or_default()
}

/// The crops of one layer animation. `index` counts every frame, the ones left out
/// included: it is the frame's position in its layer.
fn crops_of(la: LayerAnimation<'_>) -> impl Iterator<Item = Anm2Frame> + '_ {
    let LayerAnimation {
        animation,
        layer,
        sheet,
        frames,
        ..
    } = la;
    frames
        .into_iter()
        .enumerate()
        .filter_map(move |(index, f)| {
            Some(Anm2Frame {
                animation: animation.clone(),
                layer: layer.clone(),
                sheet: sheet.clone(),
                index,
                visible: f.attr("Visible") != Some("false"),
                rect: rect_of(f)?,
                origin: origin_of(f),
            })
        })
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

/// The frame's top-left, `position - pivot`. Every attribute is optional and a missing one
/// reads as zero: a file that places nothing places everything at the origin, which is what
/// `minimap_icons.anm2` does and what the crop-only callers have always assumed.
fn origin_of(e: &Element) -> Point {
    let n = |name: &str| {
        e.attr(name)
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(0)
    };
    Point {
        x: n("XPosition") - n("XPivot"),
        y: n("YPosition") - n("YPivot"),
    }
}

/// The piece of the sheet a frame draws: all four of `XCrop`, `YCrop`, `Width`, `Height`,
/// or none — a frame missing one is not a crop.
pub(crate) fn rect_of(e: &Element) -> Option<Rect> {
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

    /// Mirrors the real `completion_widget.anm2`, which places its layers: the paper at
    /// `0,0` and Mom's Heart at `22,7`, both pivoting on `16,16`. Measured on the installed
    /// game on 2026-09-21.
    const PLACED: &[u8] = br#"<AnimatedActor>
<Content><Spritesheets><Spritesheet Id="0" Path="completion_widget.png"/></Spritesheets>
<Layers><Layer Id="0" Name="Paper"/><Layer Id="1" Name="Heart"/></Layers></Content>
<Animations><Animation Name="Idle"><LayerAnimations>
<LayerAnimation LayerId="0"><Frame XPosition="0" YPosition="0" XPivot="16" YPivot="16" XCrop="0" YCrop="0" Width="96" Height="96" Visible="true"/></LayerAnimation>
<LayerAnimation LayerId="1"><Frame XPosition="22" YPosition="7" XPivot="16" YPivot="16" XCrop="64" YCrop="112" Width="16" Height="16" Visible="true"/></LayerAnimation>
</LayerAnimations></Animation></Animations></AnimatedActor>"#;

    #[test]
    fn a_frame_carries_where_the_layer_draws_it_not_only_what_to_cut() {
        // The game draws a frame at `position - pivot` in the actor's own space. The
        // subtraction is done here so no caller has to remember to do it — and it goes
        // negative, which is why the field is signed.
        let f = frames(PLACED).expect("valid XML");
        assert_eq!(f[0].origin, Point { x: -16, y: -16 }, "the paper");
        assert_eq!(f[1].origin, Point { x: 6, y: -9 }, "Mom's Heart");
    }

    #[test]
    fn two_layers_of_one_actor_are_placed_by_the_difference_of_their_origins() {
        // What a composition needs: the mark's top-left inside the paper. The game's own
        // numbers say 22,7 — the anm2's `XPosition`/`YPosition` for that layer, which only
        // come out right because both origins carry the pivot.
        let f = frames(PLACED).expect("valid XML");
        let (paper, heart) = (f[0].origin, f[1].origin);
        assert_eq!(heart.x - paper.x, 22);
        assert_eq!(heart.y - paper.y, 7);
    }

    #[test]
    fn a_frame_that_declares_no_placement_sits_at_the_origin() {
        // `minimap_icons.anm2` and the synthetic fixtures above carry no position at all.
        // Absent is `0,0` and not a refusal: the crop is still a crop.
        let f = frames(PER_LAYER).expect("valid XML");
        assert!(f.iter().all(|f| f.origin == Point { x: 0, y: 0 }));
    }

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

    /// A layer animation naming a layer nobody declared, one outside any `<Animation>`, a
    /// frame without `Visible`, and a frame nested one level too deep to be the layer's.
    const LOOSE: &[u8] = br#"<AnimatedActor>
<Content><Spritesheets><Spritesheet Id="5" Path="first.png"/><Spritesheet Id="6" Path="second.png"/></Spritesheets>
<Layers><Layer Id="0" Name="Known" SpritesheetId="9"/><Layer Name="NoId"/></Layers></Content>
<LayerAnimation LayerId="0"><Frame XCrop="1" YCrop="1" Width="1" Height="1"/></LayerAnimation>
<Animations><Animation Name="A"><LayerAnimations>
<LayerAnimation LayerId="42"><Frame XCrop="2" YCrop="2" Width="2" Height="2" Visible="False"/><Group><Frame XCrop="9" YCrop="9" Width="9" Height="9"/></Group></LayerAnimation>
<LayerAnimation><Frame XCrop="3" YCrop="3" Width="3" Height="3" XPosition="x"/></LayerAnimation>
</LayerAnimations></Animation></Animations></AnimatedActor>"#;

    #[test]
    fn what_a_file_leaves_undeclared_reads_as_empty_names_and_the_first_sheet() {
        let f = frames(LOOSE).expect("valid XML");
        let seen: Vec<(&str, &str, &str, usize, bool, u32)> = f
            .iter()
            .map(|f| {
                (
                    f.animation.as_str(),
                    f.layer.as_str(),
                    f.sheet.as_str(),
                    f.index,
                    f.visible,
                    f.rect.x,
                )
            })
            .collect();
        assert_eq!(
            seen,
            vec![
                // Outside any animation; its layer's sheet id names no sheet.
                ("", "Known", "first.png", 0, true, 1),
                // A layer nobody declared; `Visible` is only `false` when it says so.
                ("A", "", "first.png", 0, true, 2),
                // No `LayerId` at all.
                ("A", "", "first.png", 0, true, 3),
            ],
            "the nested frame is not the layer's"
        );
        assert_eq!(
            f[2].origin,
            Point { x: 0, y: 0 },
            "a malformed position is 0"
        );
    }

    #[test]
    fn a_crop_with_a_malformed_coordinate_is_not_a_crop() {
        let bad: &[u8] = br#"<AnimatedActor><Animations><Animation Name="A"><LayerAnimations>
<LayerAnimation LayerId="0"><Frame XCrop="-1" YCrop="0" Width="4" Height="4"/><Frame XCrop="0" YCrop="0" Width="4" Height="4"/></LayerAnimation>
</LayerAnimations></Animation></Animations></AnimatedActor>"#;
        let f = frames(bad).expect("valid XML");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].index, 1, "the index still counts the frame left out");
        assert_eq!(f[0].sheet, "", "no sheet declared at all");
    }

    #[test]
    fn an_unreadable_file_has_no_spritesheets() {
        assert!(spritesheets(b"not xml <<<").is_empty());
        assert_eq!(
            spritesheets(LOOSE),
            vec!["first.png", "second.png"],
            "declaration order"
        );
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
