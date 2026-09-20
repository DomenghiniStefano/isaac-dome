//! `gfx/ui/minimap_icons.anm2`: the icons the game draws on its own minimap.
//!
//! Forty-one one-frame animations on one sheet, and **the animation's name is the icon's
//! name** — `IconShop`, `IconBoss`, `IconSecretRoom`. That is the whole index: there is no
//! numbering to get wrong and no order to depend on, so a patch that adds an icon adds a name
//! and nothing here has to move.
//!
//! What this module does not do is decide which room *kind* wears which name. That mapping is
//! a reading of the game, it belongs to the view layer (`ipc::icon`), and it is the part that
//! can be wrong.

use std::collections::BTreeMap;

use crate::anm2;
use crate::diagnostics::{Diagnostic, Source};
use crate::sprite::SpriteRef;

pub const SHEET: &str = "gfx/ui/minimap_icons.png";

/// Every icon the file names, by that name.
///
/// The **first** frame of each animation wins. They have one each today; taking the first
/// rather than the last means a file that grows a second frame keeps answering what it
/// answered before, instead of silently changing which piece of the sheet an icon is.
pub fn parse(bytes: &[u8], diagnostics: &mut Vec<Diagnostic>) -> BTreeMap<String, SpriteRef> {
    let Some(frames) = anm2::frames(bytes) else {
        diagnostics.push(Diagnostic::SourceUnreadable {
            source: Source::MinimapIcons,
        });
        return BTreeMap::new();
    };
    let mut out: BTreeMap<String, SpriteRef> = BTreeMap::new();
    for frame in frames {
        if frame.animation.is_empty() {
            continue;
        }
        out.entry(frame.animation).or_insert(SpriteRef {
            path: SHEET.to_string(),
            rect: Some(frame.rect),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // The shape of the real file, cut down to three icons: one sheet, one layer, one frame
    // per animation, and the name on the animation.
    const SAMPLE: &[u8] = br#"<?xml version="1.0"?>
<AnimatedActor><Content><Spritesheets><Spritesheet Id="0" Path="minimap_icons.png"/></Spritesheets>
<Layers><Layer Id="0" Name="icon" SpritesheetId="0"/></Layers></Content>
<Animations>
<Animation Name="IconShop" FrameNum="1"><LayerAnimations><LayerAnimation LayerId="0">
<Frame XCrop="16" YCrop="0" Width="16" Height="16" Visible="true"/>
</LayerAnimation></LayerAnimations></Animation>
<Animation Name="IconBoss" FrameNum="1"><LayerAnimations><LayerAnimation LayerId="0">
<Frame XCrop="32" YCrop="16" Width="16" Height="16" Visible="true"/>
</LayerAnimation></LayerAnimations></Animation>
</Animations></AnimatedActor>"#;

    #[test]
    fn indexes_every_icon_by_the_name_the_game_gave_it() {
        let mut d = Vec::new();
        let icons = parse(SAMPLE, &mut d);
        assert_eq!(icons.len(), 2);
        assert!(icons.contains_key("IconShop"));
        assert!(icons.contains_key("IconBoss"));
        assert!(d.is_empty());
    }

    #[test]
    fn carries_the_crop_and_the_sheet_it_crops_from() {
        let mut d = Vec::new();
        let icons = parse(SAMPLE, &mut d);
        let shop = &icons["IconShop"];
        assert_eq!(shop.path, SHEET);
        let rect = shop.rect.expect("an icon is a crop, never a whole sheet");
        assert_eq!((rect.x, rect.y, rect.w, rect.h), (16, 0, 16, 16));
    }

    #[test]
    fn says_so_when_the_file_is_not_readable_instead_of_answering_nothing() {
        let mut d = Vec::new();
        assert!(parse(b"not xml at all <<<", &mut d).is_empty());
        assert_eq!(
            d,
            vec![Diagnostic::SourceUnreadable {
                source: Source::MinimapIcons
            }]
        );
    }
}
