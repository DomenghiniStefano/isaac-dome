//! Which piece of which sheet draws a column's mark (B13). The anm2 files here are
//! synthetic, shaped like the game's; `mark_art_real.rs` runs the same map on the installed
//! game.

use catalog::{anm2_frames, SpriteRef};
use ipc::{mark_source, paper_source, widget_source, MarkFill, MarkFrames, MarkTier, BOSSES};

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

/// The layer id the paper takes in the fixture. Last, so the mark layers keep the ids the
/// tests below read them by.
const PAPER_ID: usize = WIDGET_LAYERS.len();

/// Where the fixture places mark layer `i` inside the paper. The real file scatters them
/// (`Heart` at `22,7`, `DadsNote` at `41,54`, measured 2026-09-21); what matters to a test is
/// that every layer sits somewhere different and that the arithmetic survives the pivot.
fn placed_at(i: usize) -> (i32, i32) {
    (10 + 3 * i as i32, 5 + 2 * i as i32)
}

/// One `Idle` animation: the `Paper` layer plus one layer per mark in the order above, three
/// frames each — for a mark, 0 hidden and 1 and 2 shown — at rectangles that name their layer
/// (x = 16 × layer) and their frame (y = 16 × frame).
///
/// Every layer carries a position and the pivot the game uses, `16,16`, because the placement
/// of a layer inside another is the difference of the two and a fixture that pivots on nothing
/// would let a missing pivot pass.
fn widget_anm2() -> Vec<u8> {
    let layers: String = WIDGET_LAYERS
        .iter()
        .enumerate()
        .map(|(i, name)| format!(r#"<Layer Id="{i}" Name="{name}" SpritesheetId="0"/>"#))
        .chain(std::iter::once(format!(
            r#"<Layer Id="{PAPER_ID}" Name="Paper" SpritesheetId="0"/>"#
        )))
        .collect();
    let frame = |x: u32, y: u32, size: u32, at: (i32, i32), visible: bool| {
        format!(
            r#"<Frame XPosition="{}" YPosition="{}" XPivot="16" YPivot="16" XCrop="{x}" YCrop="{y}" Width="{size}" Height="{size}" Visible="{visible}"/>"#,
            at.0, at.1
        )
    };
    // The paper pivots on 16,16 at 0,0, exactly as the game's does, so its own origin is
    // negative and a mark's offset inside it is the difference and not the raw position.
    let paper: String = (0..3)
        .map(|f| frame(0, 128 * f, 96, (0, 0), true))
        .collect();
    let animations: String = (0..WIDGET_LAYERS.len())
        .map(|i| {
            let frames: String = (0..3)
                .map(|f| frame(16 * i as u32, 16 * f, 16, placed_at(i), f != 0))
                .collect();
            format!(r#"<LayerAnimation LayerId="{i}">{frames}</LayerAnimation>"#)
        })
        .chain(std::iter::once(format!(
            r#"<LayerAnimation LayerId="{PAPER_ID}">{paper}</LayerAnimation>"#
        )))
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

// The widget's own picture: one paper with the marks laid on it, which is how the game draws
// it. Measured on the installed game 2026-09-21 — the `Paper` layer is layer 0 of
// `completion_widget.anm2` and the eleven marks are placed inside it, `Heart` at `22,7`,
// `Greed` at `64,16`, `DadsNote` at `41,54`. There is no paper *per mark*: that is the
// premise B19 was written on, and it is not what the file says.

/// The twelve columns with nothing on them.
fn nothing() -> [MarkFill; 12] {
    [MarkFill::None; 12]
}

fn all(fill: MarkFill) -> [MarkFill; 12] {
    [fill; 12]
}

#[test]
fn the_paper_is_a_frame_of_its_own_layer_read_like_a_marks() {
    let f = frames();
    let normal = place(paper_source(MarkTier::Normal, &f));
    let hard = place(paper_source(MarkTier::Hard, &f));
    // Frame 0 and frame 2, the same index a mark uses: one rule for both layers.
    assert_eq!(normal, Some((WIDGET.to_string(), 0, 0)));
    assert_eq!(hard, Some((WIDGET.to_string(), 0, 256)));
    assert!(paper_source(MarkTier::Normal, &MarkFrames::default()).is_none());
}

#[test]
fn the_widget_draws_only_the_columns_that_have_something() {
    let f = frames();
    let mut fills = nothing();
    fills[0] = MarkFill::Hard;
    fills[3] = MarkFill::Normal;
    let art = widget_source(&fills, &f).expect("a paper and two marks");
    assert_eq!(art.marks.len(), 2, "an empty column draws nothing at all");
    // The tier decides which frame, exactly as a lone symbol does.
    assert_eq!(art.marks[0].0.rect.map(|r| (r.x, r.y)), Some((0, 32)));
    assert_eq!(art.marks[1].0.rect.map(|r| (r.x, r.y)), Some((48, 0)));
    assert_eq!(
        widget_source(&nothing(), &f)
            .expect("a bare paper")
            .marks
            .len(),
        0
    );
}

#[test]
fn a_mark_sits_on_the_paper_where_the_anm2_places_it() {
    // The one number this composition cannot guess. It is the difference of the two origins,
    // never the raw position: both layers pivot on 16,16, and a reading that forgot the pivot
    // would be off by sixteen pixels in each direction and still look almost right.
    let f = frames();
    let mut fills = nothing();
    fills[4] = MarkFill::Normal;
    let art = widget_source(&fills, &f).expect("a paper and one mark");
    assert_eq!((art.marks[0].1, art.marks[0].2), placed_at(4));
}

#[test]
fn the_paper_is_bloodied_only_when_every_column_is_hard() {
    let f = frames();
    let rect = |fills: &[MarkFill; 12]| {
        widget_source(fills, &f)
            .and_then(|a| a.paper.rect)
            .map(|r| (r.x, r.y))
    };
    assert_eq!(
        rect(&all(MarkFill::Hard)),
        Some((0, 256)),
        "everything on hard"
    );
    assert_eq!(rect(&all(MarkFill::Normal)), Some((0, 0)));
    assert_eq!(rect(&nothing()), Some((0, 0)));
    // One column short is not finished, and the paper is the only thing that says so at a
    // glance: the vacuity guard for the assertion above.
    let mut nearly = all(MarkFill::Hard);
    nearly[11] = MarkFill::Normal;
    assert_eq!(
        rect(&nearly),
        Some((0, 0)),
        "eleven of twelve is not all of them"
    );
}

#[test]
fn delirium_is_not_on_the_widgets_paper() {
    // Column 9 is the online lobby's, another actor: its position is measured in that
    // actor's space, so there is no offset that would put it on this paper. The matrix below
    // the band is where all twelve columns live.
    let f = frames();
    let mut fills = nothing();
    fills[9] = MarkFill::Hard;
    assert_eq!(widget_source(&fills, &f).expect("a paper").marks.len(), 0);
    assert!(
        mark_source(9, MarkTier::Hard, &f).is_some(),
        "and it still has a symbol of its own"
    );
}

#[test]
fn without_the_paper_there_is_no_widget() {
    // The marks without their sheet are eleven pictures floating on nothing. A composition
    // that dropped the base would answer with something, and something wrong.
    let no_paper = MarkFrames {
        widget: anm2_frames(&widget_anm2())
            .expect("valid XML")
            .into_iter()
            .filter(|f| f.layer != "Paper")
            .collect(),
        lobby: Vec::new(),
    };
    assert!(widget_source(&all(MarkFill::Hard), &no_paper).is_none());
    assert!(widget_source(&nothing(), &MarkFrames::default()).is_none());
}
