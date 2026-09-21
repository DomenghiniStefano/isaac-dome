//! The versus screen: the rectangle the game cuts a boss portrait to.
//!
//! A portrait file is not always one picture. Six of them are 384x192 and hold the boss on
//! the left and, on the right, the rubble it climbs out of — *Pin*, *Polycephalus* twice,
//! *Mega Fred*, *The Stain*, *Big Horn* — and `Portrait_Mother.png` is 480x440, her body
//! above and her two hands below. Drawing the file whole therefore draws a creature **and a
//! second thing beside it**, which is what this app did until 2026-09-22 (B70).
//!
//! The game does not guess where to cut, and neither do we: `bossportraits.xml` names its
//! scene in the root element (`anm2="versusscreen.anm2"`), and that scene declares a layer
//! `BossPortrait` whose frames carry the crop — `XCrop="0" YCrop="0" Width="192"
//! Height="192"`. The second drawing belongs to sibling layers, `BossPortraitGround` at
//! `XCrop="192"` and, in Mother's scene, `BossPortraitExtra` at `YCrop="220"`: they are
//! drawn *behind* the boss at the same spot, not beside it.
//!
//! **Two bosses have a scene of their own**, which the game picks in code and no file
//! declares: `versusscreen_mother.anm2` (480x220, her top half) and
//! `versusscreen_dogma.anm2` (208x192). They are read here as sources like any other, and a
//! row takes one only when that scene's `BossPortrait` layer names the row's own portrait as
//! its sheet — so the match is the game's data agreeing with itself, never a name we
//! inferred. Measured on 2026-09-22: Dogma's file is 192x192 and his scene asks for 208, so
//! his crop changes nothing; he is here because the rule is one rule, not a rule and an
//! exception for Mother.
//!
//! No scene, no crop: the portrait stays the whole file, which is what every build before
//! this one served.

use std::collections::BTreeMap;

use crate::anm2;
use crate::sprite::Rect;

/// The layer the scene draws the boss itself in.
const PORTRAIT_LAYER: &str = "BossPortrait";

/// What to cut a portrait down to: the scene every row shares, and the scenes that name one
/// sheet each.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PortraitCrops {
    /// From the scene `bossportraits.xml` itself points at. `None` when it could not be read,
    /// and then nothing is cropped at all.
    default: Option<Rect>,
    /// By the sheet the scene names, lowercased, file name only.
    by_sheet: BTreeMap<String, Rect>,
}

/// Reads the crop out of the scenes' bytes. Every argument is optional because every one of
/// them is a file that may not be in the archives, and a missing scene costs a crop, never
/// the catalog.
pub fn crops(default_scene: Option<&[u8]>, own_scenes: &[&[u8]]) -> PortraitCrops {
    PortraitCrops {
        default: default_scene.and_then(portrait_frame).map(|(_, rect)| rect),
        by_sheet: own_scenes
            .iter()
            .filter_map(|bytes| portrait_frame(bytes))
            .map(|(sheet, rect)| (file_key(&sheet), rect))
            .collect(),
    }
}

/// The sheet and the crop of a scene's `BossPortrait` layer: its **first** frame, because
/// every frame of that layer declares the same rectangle and only the position moves — the
/// portrait slides in from the side of the screen (measured on all three scenes, 2026-09-22).
fn portrait_frame(scene: &[u8]) -> Option<(String, Rect)> {
    anm2::frames(scene)?
        .into_iter()
        .find(|f| f.layer == PORTRAIT_LAYER)
        .map(|f| (f.sheet, f.rect))
}

/// What two files call the same picture: the file name alone, lowercased.
/// `bossportraits.xml` writes `Portrait_Mother.png` under a declared root, its scene writes
/// `portrait_mother.png` beside itself, and Windows reads the two as one file.
fn file_key(path: &str) -> String {
    path.rsplit(['/', '\\'])
        .next()
        .unwrap_or(path)
        .to_lowercase()
}

impl PortraitCrops {
    /// The rectangle for a portrait, by the path `bossportraits.xml` gave it.
    pub fn rect_for(&self, portrait_path: &str) -> Option<Rect> {
        self.by_sheet
            .get(&file_key(portrait_path))
            .copied()
            .or(self.default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scene with the two layers that matter, written the way the game writes them.
    fn scene(sheet: &str, portrait: &str, ground: &str) -> String {
        format!(
            "<AnimatedActor>
	<Content>
		<Spritesheets>
			<Spritesheet Id=\"0\" Path=\"ground.png\" />
			<Spritesheet Id=\"4\" Path=\"{sheet}\" />
		</Spritesheets>
		<Layers>
			<Layer Id=\"0\" Name=\"Background\" SpritesheetId=\"0\" />
			<Layer Id=\"4\" Name=\"BossPortrait\" SpritesheetId=\"4\" />
			<Layer Id=\"13\" Name=\"BossPortraitGround\" SpritesheetId=\"4\" />
		</Layers>
	</Content>
	<Animations>
		<Animation Name=\"Scene\" FrameNum=\"2\" Loop=\"false\">
			<LayerAnimations>
				<LayerAnimation LayerId=\"4\" Visible=\"true\">
					<Frame XPosition=\"400\" YPosition=\"70\" {portrait} Visible=\"false\"/>
					<Frame XPosition=\"120\" YPosition=\"70\" {portrait} Visible=\"true\"/>
				</LayerAnimation>
				<LayerAnimation LayerId=\"13\" Visible=\"true\">
					<Frame XPosition=\"120\" YPosition=\"70\" {ground} Visible=\"true\"/>
				</LayerAnimation>
			</LayerAnimations>
		</Animation>
	</Animations>
</AnimatedActor>"
        )
    }

    const SQUARE: &str = "XCrop=\"0\" YCrop=\"0\" Width=\"192\" Height=\"192\"";
    const RIGHT_HALF: &str = "XCrop=\"192\" YCrop=\"0\" Width=\"192\" Height=\"192\"";
    const TOP_HALF: &str = "XCrop=\"0\" YCrop=\"0\" Width=\"480\" Height=\"220\"";
    const BOTTOM_HALF: &str = "XCrop=\"0\" YCrop=\"220\" Width=\"480\" Height=\"220\"";

    fn default_scene() -> String {
        scene("Portrait_20.0_Monstro.png", SQUARE, RIGHT_HALF)
    }

    fn mother_scene() -> String {
        scene("portrait_mother.png", TOP_HALF, BOTTOM_HALF)
    }

    #[test]
    fn the_scene_the_file_points_at_gives_the_crop_every_row_shares() {
        let c = crops(Some(default_scene().as_bytes()), &[]);
        assert_eq!(
            c.rect_for("gfx/ui/boss/Portrait_62.0_Pin.png"),
            Some(Rect {
                x: 0,
                y: 0,
                w: 192,
                h: 192
            }),
            "the boss is the square the BossPortrait layer draws, not the whole sheet"
        );
    }

    #[test]
    fn the_layer_that_draws_the_ground_is_not_the_one_that_draws_the_boss() {
        // `BossPortraitGround` sits at XCrop=192 and is the very thing the bug showed. Reading
        // any layer but `BossPortrait` would cut the rubble and call it the boss.
        let c = crops(Some(default_scene().as_bytes()), &[]);
        assert_eq!(c.rect_for("Portrait_62.0_Pin.png").map(|r| r.x), Some(0));
    }

    #[test]
    fn a_scene_of_its_own_wins_for_the_sheet_it_names() {
        let c = crops(
            Some(default_scene().as_bytes()),
            &[mother_scene().as_bytes()],
        );
        assert_eq!(
            c.rect_for("gfx/ui/boss/Portrait_Mother.png"),
            Some(Rect {
                x: 0,
                y: 0,
                w: 480,
                h: 220
            }),
            "Mother's own scene cuts 480x220; the shared square would cut her face in four"
        );
    }

    #[test]
    fn the_sheet_is_matched_by_name_whatever_its_case_and_folder() {
        // `bossportraits.xml` writes `Portrait_Mother.png` under a root, the scene writes
        // `portrait_mother.png` beside itself. Same file, two spellings, and the game reads
        // both.
        let c = crops(
            Some(default_scene().as_bytes()),
            &[mother_scene().as_bytes()],
        );
        assert_eq!(
            c.rect_for("GFX/UI/BOSS/PORTRAIT_MOTHER.PNG").map(|r| r.h),
            Some(220)
        );
    }

    #[test]
    fn a_sheet_no_scene_of_its_own_names_takes_the_shared_crop() {
        let c = crops(
            Some(default_scene().as_bytes()),
            &[mother_scene().as_bytes()],
        );
        assert_eq!(c.rect_for("Portrait_BigHorn.png").map(|r| r.w), Some(192));
    }

    #[test]
    fn no_scene_at_all_leaves_the_portrait_whole() {
        // Degrade, never fail: without the scene the app serves the file it has always
        // served, and nothing about the catalog stops working.
        let c = crops(None, &[]);
        assert_eq!(c.rect_for("Portrait_62.0_Pin.png"), None);
    }

    #[test]
    fn a_scene_that_is_not_an_anm2_at_all_is_no_crop_rather_than_a_wrong_one() {
        let c = crops(Some(b"not xml <<<"), &[b"neither is this"]);
        assert_eq!(c.rect_for("Portrait_62.0_Pin.png"), None);
    }
}
