//! `gfx/ui/coop menu.anm2`: the head crops in the `coop menu.png` sheet.
//! The anm2 is XML: `Main` animation, `Main` layer (id 0), one `<Frame>` per cell.

use crate::diagnostics::{Diagnostic, Source};
use crate::ids::CharacterId;
use crate::sprite::Rect;
use crate::strings::children_named;
use crate::xml::{elements, Element};

pub const SHEET: &str = "gfx/ui/coop menu.png";

/// The frames of layer 0 of the `Main` animation, in order; `None` where the crop is missing.
pub fn parse(bytes: &[u8], diagnostics: &mut Vec<Diagnostic>) -> Vec<Option<Rect>> {
    let els = match elements(bytes) {
        Ok(els) => els,
        Err(_) => {
            diagnostics.push(Diagnostic::SourceUnreadable {
                source: Source::CoopMenuAnm2,
            });
            return Vec::new();
        }
    };
    let Some(main) = els
        .iter()
        .position(|e| e.name == "Animation" && e.attr("Name") == Some("Main"))
    else {
        return Vec::new();
    };
    let main_depth = els[main].depth;
    // The first <LayerAnimation LayerId="0"> inside the Main animation.
    let Some(layer) = els[main + 1..]
        .iter()
        .take_while(|e| e.depth > main_depth)
        .position(|e| e.name == "LayerAnimation" && e.attr("LayerId") == Some("0"))
        .map(|p| main + 1 + p)
    else {
        return Vec::new();
    };
    children_named(&els, layer, "Frame")
        .into_iter()
        .map(rect_of)
        .collect()
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

/// The frame that portrays the character.
///
/// Verified by eye on 2026-09-03 by cropping `coop menu.png` into `samples/sprites/heads/`
/// and looking at the 38 frames one by one: frame 0 is the menu's "?" placeholder, then
/// the sequence follows the order of `players.xml`. Three cells repeat, and it's exactly
/// where the game reuses the base form's face: Lazarus II (id 11) on Lazarus's frame,
/// Black Judas (id 12) on Judas's, The Soul (id 17) on The Forgotten's. Esau (id 20) has
/// no cell — in the menu you pick "Jacob & Esau" as a single choice — and the sequence
/// ends at Tainted Jacob (id 37): Tainted Lazarus Risen (38), Dark Esau (39) and Tainted
/// Soul (40) have no head.
pub fn frame_for(id: CharacterId) -> Option<usize> {
    match id.0 {
        0 => Some(1),   // Isaac
        1 => Some(2),   // Magdalene
        2 => Some(3),   // Cain
        3 => Some(4),   // Judas
        4 => Some(5),   // ??? (Blue Baby)
        5 => Some(6),   // Eve
        6 => Some(7),   // Samson
        7 => Some(8),   // Azazel
        8 => Some(9),   // Lazarus
        9 => Some(10),  // Eden
        10 => Some(11), // The Lost
        11 => Some(12), // Lazarus Risen: Lazarus's cell
        12 => Some(13), // Black Judas: Judas's cell
        13 => Some(14), // Lilith
        14 => Some(15), // Keeper
        15 => Some(16), // Apollyon
        16 => Some(17), // The Forgotten
        17 => Some(18), // The Soul: The Forgotten's cell
        18 => Some(19), // Bethany
        19 => Some(20), // Jacob
        21 => Some(21), // Tainted Isaac
        22 => Some(22), // Tainted Magdalene
        23 => Some(23), // Tainted Cain
        24 => Some(24), // Tainted Judas
        25 => Some(25), // Tainted ???
        26 => Some(26), // Tainted Eve
        27 => Some(27), // Tainted Samson
        28 => Some(28), // Tainted Azazel
        29 => Some(29), // Tainted Lazarus
        30 => Some(30), // Tainted Eden
        31 => Some(31), // Tainted Lost
        32 => Some(32), // Tainted Lilith
        33 => Some(33), // Tainted Keeper
        34 => Some(34), // Tainted Apollyon
        35 => Some(35), // Tainted Forgotten
        36 => Some(36), // Tainted Bethany
        37 => Some(37), // Tainted Jacob
        // Allowed: ids are open numbers, not an enum. This is where Esau (20), the forms
        // without a cell (38, 39, 40), and any character added by a future patch fall.
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ANM2: &[u8] = br#"<AnimatedActor>
<Content><Spritesheets><Spritesheet Path="coop menu.png" Id="0"/></Spritesheets>
<Layers><Layer Name="Main" Id="0" SpritesheetId="0"/><Layer Name="Babies" Id="1" SpritesheetId="1"/></Layers></Content>
<Animations DefaultAnimation="Main">
<Animation Name="Main" FrameNum="3" Loop="false">
 <RootAnimation><Frame XPosition="0" YPosition="0" Delay="3" Visible="true"/></RootAnimation>
 <LayerAnimations>
  <LayerAnimation LayerId="0" Visible="true">
   <Frame XPosition="0" YPosition="0" Delay="1" Visible="true"/>
   <Frame XPosition="0" YPosition="0" XCrop="0" YCrop="0" Width="32" Height="32" Delay="1" Visible="true"/>
   <Frame XPosition="0" YPosition="0" XCrop="32" YCrop="0" Width="32" Height="32" Delay="1" Visible="true"/>
  </LayerAnimation>
  <LayerAnimation LayerId="1" Visible="true">
   <Frame XCrop="99" YCrop="99" Width="1" Height="1" Delay="1" Visible="true"/>
  </LayerAnimation>
 </LayerAnimations>
</Animation>
<Animation Name="Arrows" FrameNum="1"><LayerAnimations><LayerAnimation LayerId="0"><Frame XCrop="7" YCrop="7" Width="7" Height="7" Delay="1"/></LayerAnimation></LayerAnimations></Animation>
</Animations></AnimatedActor>"#;

    #[test]
    fn only_main_animation_main_layer_frames_are_taken_in_order() {
        let mut d = Vec::new();
        let frames = parse(ANM2, &mut d);
        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0], None, "the first frame has no crop");
        assert_eq!(
            frames[1],
            Some(Rect {
                x: 0,
                y: 0,
                w: 32,
                h: 32
            })
        );
        assert_eq!(
            frames[2],
            Some(Rect {
                x: 32,
                y: 0,
                w: 32,
                h: 32
            })
        );
        assert!(d.is_empty());
    }

    #[test]
    fn the_map_skips_esau_and_the_forms_without_a_cell() {
        // Up to Jacob the sequence is offset by one from the placeholder at the head...
        assert_eq!(frame_for(CharacterId(0)), Some(1), "Isaac");
        assert_eq!(frame_for(CharacterId(19)), Some(20), "Jacob");
        // ...and after Esau, who has no cell, it goes back to matching the id.
        assert_eq!(frame_for(CharacterId(21)), Some(21), "Tainted Isaac");
        assert_eq!(frame_for(CharacterId(37)), Some(37), "Tainted Jacob");
        // Esau, the forms without a cell, and an id that does not exist in the game.
        assert_eq!(frame_for(CharacterId(20)), None);
        assert_eq!(frame_for(CharacterId(38)), None);
        assert_eq!(frame_for(CharacterId(39)), None);
        assert_eq!(frame_for(CharacterId(40)), None);
        assert_eq!(frame_for(CharacterId(999)), None);
        // Frame 0 is the "?" placeholder: it belongs to no one.
        assert!((0..=40).all(|i| frame_for(CharacterId(i)) != Some(0)));
    }

    #[test]
    fn junk_yields_no_frames_and_a_diagnostic() {
        let mut d = Vec::new();
        assert!(parse(b"<AnimatedActor><Animation", &mut d).is_empty());
        assert_eq!(
            d,
            vec![Diagnostic::SourceUnreadable {
                source: Source::CoopMenuAnm2
            }]
        );
    }
}
