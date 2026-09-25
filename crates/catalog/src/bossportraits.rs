//! `bossportraits.xml`: 103 bosses with literal name and portrait. Two declared
//! portraits (*The Beast*, *Cadavra*) don't exist in the archives: the catalog reports
//! them as-is from the file, and it's up to whoever resolves the sprite to find out.

use crate::diagnostics::{Diagnostic, SkipReason, Source};
use crate::ids::{AchievementId, BossId};
use crate::items::normalize_root;
use crate::sprite::SpriteRef;
use crate::versusscreen::PortraitCrops;
use crate::xml::{elements, Element};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Boss {
    pub id: BossId,
    pub name: String,
    pub portrait: SpriteRef,
    pub unlocked_by: Option<AchievementId>,
}

pub fn parse(bytes: &[u8], crops: &PortraitCrops, diagnostics: &mut Vec<Diagnostic>) -> Vec<Boss> {
    let els = match elements(bytes) {
        Ok(els) => els,
        Err(_) => {
            diagnostics.push(Diagnostic::SourceUnreadable {
                source: Source::BossPortraits,
            });
            return Vec::new();
        }
    };
    let root = els
        .iter()
        .find(|e| e.name == "bosses")
        .and_then(|e| e.attr("root"))
        .map(normalize_root)
        .filter(|r| !r.is_empty())
        .unwrap_or_else(|| "gfx/ui/boss".to_string());

    els.iter()
        .filter(|e| e.name == "boss")
        .filter_map(|e| boss_from(e, &root, crops, diagnostics))
        .collect()
}

fn boss_from(
    e: &Element,
    root: &str,
    crops: &PortraitCrops,
    d: &mut Vec<Diagnostic>,
) -> Option<Boss> {
    let skip = |id: Option<u32>, reason: SkipReason, d: &mut Vec<Diagnostic>| {
        d.push(Diagnostic::ElementSkipped {
            source: Source::BossPortraits,
            id,
            reason,
        });
        None
    };
    let Some(raw_id) = e.attr("id") else {
        return skip(None, SkipReason::MissingId, d);
    };
    let Ok(id) = raw_id.parse::<u32>() else {
        return skip(None, SkipReason::MalformedId, d);
    };
    let Some(portrait) = e.attr("portrait") else {
        return skip(Some(id), SkipReason::MissingSprite, d);
    };
    let Some(name) = e.attr("name") else {
        return skip(Some(id), SkipReason::MissingName, d);
    };
    Some(Boss {
        id: BossId(id),
        name: name.to_string(),
        portrait: {
            // Not the whole file: six portraits hold the boss and the rubble it climbs out of
            // side by side, and Mother holds her hands under her (B70). The rectangle is the
            // one the game's own versus screen cuts, never a size of ours.
            let path = format!("{root}/{portrait}");
            let rect = crops.rect_for(&path);
            SpriteRef { path, rect }
        },
        unlocked_by: e
            .attr("achievement")
            .and_then(|a| a.parse().ok())
            .map(AchievementId),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const B: &[u8] = b"<bosses root=\"gfx/ui/boss/\" anm2=\"versusscreen.anm2\">
\t<boss id=\"1\" name=\"Monstro\" nameimage=\"BossName_20.0_Monstro.png\" portrait=\"Portrait_20.0_Monstro.png\" pivotX=\"96\" pivotY=\"132\" />
\t<boss id=\"9\" name=\"Famine\" portrait=\"Portrait_Famine.png\" achievement=\"5\" />
\t<boss id=\"100\" name=\"The Beast\" portrait=\"Portrait_The Beast.png\" />
\t<boss name=\"NoId\" portrait=\"x.png\" />
\t<boss id=\"7\" name=\"NoPortrait\" />
</bosses>";

    /// The scene the game ships, reduced to the one layer this module's crop comes from.
    const SCENE: &[u8] = b"<AnimatedActor><Content>
\t<Spritesheets><Spritesheet Id=\"4\" Path=\"Portrait_20.0_Monstro.png\" /></Spritesheets>
\t<Layers><Layer Id=\"4\" Name=\"BossPortrait\" SpritesheetId=\"4\" /></Layers>
\t</Content><Animations><Animation Name=\"Scene\"><LayerAnimations>
\t<LayerAnimation LayerId=\"4\" Visible=\"true\">
\t\t<Frame XCrop=\"0\" YCrop=\"0\" Width=\"192\" Height=\"192\" Visible=\"true\"/>
\t</LayerAnimation>
\t</LayerAnimations></Animation></Animations></AnimatedActor>";

    fn scene_crops() -> PortraitCrops {
        crate::versusscreen::crops(Some(SCENE), &[])
    }

    #[test]
    fn a_row_carries_the_crop_its_scene_declares() {
        let mut d = Vec::new();
        let b = parse(B, &scene_crops(), &mut d);
        assert_eq!(
            b[0].portrait.rect,
            Some(crate::sprite::Rect {
                x: 0,
                y: 0,
                w: 192,
                h: 192
            }),
            "the portrait is the square the versus screen draws, not the file"
        );
    }

    #[test]
    fn without_a_scene_a_row_carries_the_whole_file_as_before() {
        let mut d = Vec::new();
        let b = parse(B, &PortraitCrops::default(), &mut d);
        assert_eq!(b[0].portrait.rect, None);
    }

    #[test]
    fn portrait_uses_the_declared_root_and_keeps_spaces_in_names() {
        let mut d = Vec::new();
        let b = parse(B, &scene_crops(), &mut d);
        assert_eq!(b[0].portrait.path, "gfx/ui/boss/Portrait_20.0_Monstro.png");
        assert_eq!(
            b[2].portrait.path, "gfx/ui/boss/Portrait_The Beast.png",
            "the space in the file name stays: that's how the game has it"
        );
        assert_eq!(b[0].name, "Monstro");
        assert_eq!(b[1].unlocked_by, Some(AchievementId(5)));
        assert_eq!(b[0].unlocked_by, None);
    }

    #[test]
    fn malformed_rows_are_skipped() {
        let mut d = Vec::new();
        let b = parse(B, &scene_crops(), &mut d);
        assert_eq!(b.len(), 3);
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::BossPortraits,
            id: None,
            reason: SkipReason::MissingId
        }));
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::BossPortraits,
            id: Some(7),
            reason: SkipReason::MissingSprite
        }));
    }

    #[test]
    fn the_skips_come_in_file_order_with_the_portrait_checked_before_the_name() {
        let mut d = Vec::new();
        let b = parse(
            b"<bosses>
<boss id=\"x\" name=\"A\" portrait=\"a.png\" />
<boss name=\"B\" portrait=\"b.png\" />
<boss id=\"3\" />
<boss id=\"4\" portrait=\"d.png\" />
</bosses>",
            &PortraitCrops::default(),
            &mut d,
        );
        assert!(b.is_empty());
        let skipped = |id, reason| Diagnostic::ElementSkipped {
            source: Source::BossPortraits,
            id,
            reason,
        };
        assert_eq!(
            d,
            vec![
                skipped(None, SkipReason::MalformedId),
                skipped(None, SkipReason::MissingId),
                skipped(Some(3), SkipReason::MissingSprite),
                skipped(Some(4), SkipReason::MissingName),
            ]
        );
    }

    #[test]
    fn without_a_root_the_portrait_takes_the_game_folder() {
        let mut d = Vec::new();
        let b = parse(
            b"<bosses><boss id=\"1\" name=\"M\" portrait=\"m.png\" achievement=\"x\" /></bosses>",
            &PortraitCrops::default(),
            &mut d,
        );
        assert_eq!(b[0].portrait.path, "gfx/ui/boss/m.png");
        assert_eq!(b[0].unlocked_by, None, "a malformed link is no link");
    }

    #[test]
    fn junk_is_empty_with_one_diagnostic() {
        let mut d = Vec::new();
        assert!(parse(b"<bosses><boss", &PortraitCrops::default(), &mut d).is_empty());
        assert_eq!(
            d,
            vec![Diagnostic::SourceUnreadable {
                source: Source::BossPortraits
            }]
        );
    }
}
