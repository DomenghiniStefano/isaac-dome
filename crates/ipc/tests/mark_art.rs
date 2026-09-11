//! Which piece of which sheet draws a column's mark (B13). The anm2 files here are
//! synthetic, shaped like the game's; `mark_art_real.rs` runs the same map on the installed
//! game.

use catalog::{anm2_frames, SpriteRef};
use ipc::{mark_source, MarkFrames, MarkTier, BOSSES};

const WIDGET_LAYERS: [&str; 11] = [
    "Heart",
    "Polaroid",
    "UpsideDownCross",
    "Star",
    "Negative",
    "Cross",
    "MegaSatan",
    "Greed",
    "Hush",
    "Knife",
    "DadsNote",
];

/// One `Idle` animation, one layer per mark in the order above, three frames each — 0
/// hidden, 1 and 2 shown — at rectangles that name their layer (x = 16 × layer) and their
/// frame (y = 16 × frame).
fn widget_anm2() -> Vec<u8> {
    let layers: String = WIDGET_LAYERS
        .iter()
        .enumerate()
        .map(|(i, name)| format!(r#"<Layer Id="{i}" Name="{name}" SpritesheetId="0"/>"#))
        .collect();
    let animations: String = (0..WIDGET_LAYERS.len())
        .map(|i| {
            let frames: String = (0..3)
                .map(|f| {
                    format!(
                        r#"<Frame XCrop="{}" YCrop="{}" Width="16" Height="16" Visible="{}"/>"#,
                        16 * i,
                        16 * f,
                        f != 0
                    )
                })
                .collect();
            format!(r#"<LayerAnimation LayerId="{i}">{frames}</LayerAnimation>"#)
        })
        .collect();
    format!(
        r#"<AnimatedActor><Content><Spritesheets><Spritesheet Id="0" Path="completion_widget.png"/></Spritesheets><Layers>{layers}</Layers></Content><Animations><Animation Name="Idle"><LayerAnimations>{animations}</LayerAnimations></Animation></Animations></AnimatedActor>"#
    )
    .into_bytes()
}

/// The lobby draws Delirium's mark twice: on the player card first, on the background
/// second. A lookup that ignores the animation finds the card.
fn lobby_anm2() -> Vec<u8> {
    let animation = |name: &str, x: u32| {
        let frames: String = (0..3)
            .map(|f| {
                format!(
                    r#"<Frame XCrop="{x}" YCrop="{}" Width="16" Height="16" Visible="true"/>"#,
                    32 * f
                )
            })
            .collect();
        format!(
            r#"<Animation Name="{name}"><LayerAnimations><LayerAnimation LayerId="0">{frames}</LayerAnimation></LayerAnimations></Animation>"#
        )
    };
    format!(
        r#"<AnimatedActor><Content><Spritesheets><Spritesheet Id="0" Path="online_lobby.png"/></Spritesheets><Layers><Layer Id="0" Name="Completion_Delirium" SpritesheetId="0"/></Layers></Content><Animations>{}{}</Animations></AnimatedActor>"#,
        animation("PlayerInfo", 416),
        animation("Background", 224)
    )
    .into_bytes()
}

fn frames() -> MarkFrames {
    MarkFrames {
        widget: anm2_frames(&widget_anm2()).expect("valid XML"),
        lobby: anm2_frames(&lobby_anm2()).expect("valid XML"),
    }
}

fn place(s: Option<SpriteRef>) -> Option<(String, u32, u32)> {
    let s = s?;
    let r = s.rect?;
    Some((s.path, r.x, r.y))
}

const WIDGET: &str = "gfx/ui/completion_widget.png";

#[test]
fn every_column_resolves_to_two_different_tiers() {
    let f = frames();
    for (column, boss) in BOSSES.iter().enumerate() {
        let normal = place(mark_source(column, MarkTier::Normal, &f));
        let hard = place(mark_source(column, MarkTier::Hard, &f));
        assert!(normal.is_some() && hard.is_some(), "{boss} has no symbol");
        assert_ne!(normal, hard, "{boss}: the two tiers are two drawings");
    }
}

#[test]
fn a_tier_is_a_frame_of_its_column_layer() {
    let f = frames();
    let at = |column, tier| place(mark_source(column, tier, &f));
    // Mom's Heart is Heart (layer 0): frame 0 for normal, frame 2 for hard.
    assert_eq!(at(0, MarkTier::Normal), Some((WIDGET.to_string(), 0, 0)));
    assert_eq!(at(0, MarkTier::Hard), Some((WIDGET.to_string(), 0, 32)));
    // The Lamb is Cross (layer 5), by elimination.
    assert_eq!(at(5, MarkTier::Hard), Some((WIDGET.to_string(), 80, 32)));
    // Mother is Knife (layer 9), The Beast is DadsNote (layer 10).
    assert_eq!(at(10, MarkTier::Hard), Some((WIDGET.to_string(), 144, 32)));
    assert_eq!(at(11, MarkTier::Normal), Some((WIDGET.to_string(), 160, 0)));
}

#[test]
fn delirium_reads_the_lobby_background_not_the_player_card() {
    let f = frames();
    let lobby = "gfx/ui/main menu/online_lobby.png".to_string();
    assert_eq!(
        place(mark_source(9, MarkTier::Normal, &f)),
        Some((lobby.clone(), 224, 0))
    );
    assert_eq!(
        place(mark_source(9, MarkTier::Hard, &f)),
        Some((lobby, 224, 64))
    );
}

#[test]
fn a_missing_layer_or_frame_is_nothing_not_a_neighbour() {
    let only_heart_frame_0 = MarkFrames {
        widget: anm2_frames(
            br#"<AnimatedActor><Content><Spritesheets><Spritesheet Id="0" Path="completion_widget.png"/></Spritesheets><Layers><Layer Id="0" Name="Heart"/></Layers></Content><Animations><Animation Name="Idle"><LayerAnimations><LayerAnimation LayerId="0"><Frame XCrop="0" YCrop="0" Width="16" Height="16" Visible="false"/></LayerAnimation></LayerAnimations></Animation></Animations></AnimatedActor>"#,
        )
        .expect("valid XML"),
        lobby: Vec::new(),
    };
    assert!(mark_source(0, MarkTier::Normal, &only_heart_frame_0).is_some());
    assert!(
        mark_source(0, MarkTier::Hard, &only_heart_frame_0).is_none(),
        "no frame 2"
    );
    assert!(
        mark_source(1, MarkTier::Normal, &only_heart_frame_0).is_none(),
        "no Polaroid layer"
    );
    assert!(
        mark_source(9, MarkTier::Hard, &only_heart_frame_0).is_none(),
        "no lobby"
    );
    assert!(
        mark_source(12, MarkTier::Hard, &frames()).is_none(),
        "no thirteenth column"
    );
    assert!(mark_source(0, MarkTier::Hard, &MarkFrames::default()).is_none());
}
