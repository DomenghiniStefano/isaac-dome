//! Which sheet the game lays an achievement's drawing on, and where (B33). The anm2 here is
//! synthetic, shaped like `gfx/ui/achievement/achievements.anm2`; `achievement_art_real.rs`
//! runs the same reading on the installed game.

use catalog::anm2_frames;
use ipc::{achievement_backing, decode_rgba, on_paper, ACHIEVEMENT_ANM2};

/// A frame at `at`, pivoting on the paper's own centre the way the game's do — so the origin is
/// negative and the drawing's place on the paper is the difference of two origins, never a raw
/// position.
fn frame(x: u32, at: (i32, i32)) -> String {
    format!(
        r#"<Frame XPosition="{}" YPosition="{}" XPivot="130" YPivot="88" XCrop="{x}" YCrop="0" Width="272" Height="176" Visible="true"/>"#,
        at.0, at.1
    )
}

/// The popup: a `Paper` and an `Achievement` layer, both on `Paper.png` — the game swaps the
/// second one's sheet for the achievement's own drawing at runtime. `Appear` slides both in
/// from the right; `Idle` is where they rest.
fn popup(layers: &[&str], idle_drawing_at: (i32, i32)) -> Vec<u8> {
    let declared: String = layers
        .iter()
        .enumerate()
        .map(|(i, name)| format!(r#"<Layer Id="{i}" Name="{name}" SpritesheetId="0"/>"#))
        .collect();
    let animation = |name: &str, paper_at: (i32, i32), drawing_at: (i32, i32)| {
        let tracks: String = layers
            .iter()
            .enumerate()
            .map(|(i, layer)| {
                let at = if *layer == "Paper" {
                    paper_at
                } else {
                    drawing_at
                };
                // The crop names the layer, so a test can tell which layer it was handed.
                format!(
                    r#"<LayerAnimation LayerId="{i}">{}</LayerAnimation>"#,
                    frame(i as u32, at)
                )
            })
            .collect();
        format!(
            r#"<Animation Name="{name}"><LayerAnimations>{tracks}</LayerAnimations></Animation>"#
        )
    };
    format!(
        r#"<AnimatedActor><Content><Spritesheets><Spritesheet Id="0" Path="Paper.png"/></Spritesheets><Layers>{declared}</Layers></Content><Animations>{}{}</Animations></AnimatedActor>"#,
        animation("Appear", (500, 0), (500, 0)),
        animation("Idle", (0, 0), idle_drawing_at)
    )
    .into_bytes()
}

#[test]
fn the_backing_is_the_paper_layer_at_rest_in_the_anm2s_folder() {
    let frames = anm2_frames(&popup(&["Paper", "Achievement"], (0, 0))).expect("valid XML");
    let backing = achievement_backing(&frames).expect("a paper");
    assert_eq!(backing.paper.path, "gfx/ui/achievement/Paper.png");
    // Layer 0's crop, not the drawing's: x names the layer in the fixture.
    assert_eq!(
        backing.paper.rect.map(|r| (r.x, r.w, r.h)),
        Some((0, 272, 176))
    );
    assert!(ACHIEVEMENT_ANM2.starts_with("gfx/ui/achievement/"));
}

#[test]
fn the_drawing_sits_where_idle_places_it_on_the_paper() {
    // `Appear` puts both layers 500 pixels away; only `Idle` is the picture the player reads.
    let frames = anm2_frames(&popup(&["Paper", "Achievement"], (6, -3))).expect("valid XML");
    let backing = achievement_backing(&frames).expect("a paper");
    assert_eq!(backing.drawing_at, (6, -3));
}

#[test]
fn a_popup_missing_either_layer_has_no_backing() {
    // Without the paper there is nothing to lay the drawing on; without the drawing's layer
    // there is no place to lay it, and (0, 0) would be a guess that reads like a measurement.
    for layers in [&["Achievement"][..], &["Paper"][..]] {
        let frames = anm2_frames(&popup(layers, (0, 0))).expect("valid XML");
        assert!(achievement_backing(&frames).is_none(), "{layers:?}");
    }
    assert!(achievement_backing(&[]).is_none());
}

/// A `w`×`h` picture with one opaque pixel at `at`, so two pictures can be told apart.
fn dot(w: u32, h: u32, at: (i32, i32)) -> Vec<u8> {
    let rgba: Vec<u8> = (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .flat_map(|(x, y)| {
            if (x as i32, y as i32) == at {
                [200, 10, 10, 255]
            } else {
                [0, 0, 0, 0]
            }
        })
        .collect();
    let mut out = Vec::new();
    let mut enc = png::Encoder::new(&mut out, w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header()
        .and_then(|mut writer| writer.write_image_data(&rgba))
        .expect("encode");
    out
}

#[test]
fn the_drawing_is_laid_on_the_paper_at_its_place() {
    let paper = dot(8, 4, (0, 0));
    let drawing = dot(2, 2, (0, 0));
    let composed = on_paper(Some(&paper), drawing, (5, 2));
    let (w, h, rgba) = decode_rgba(&composed).expect("a PNG");
    assert_eq!((w, h), (8, 4), "the paper's size, not the drawing's");
    let alpha = |x: u32, y: u32| rgba[((y * w + x) * 4 + 3) as usize];
    assert_eq!(alpha(0, 0), 255, "the paper's own mark is still there");
    assert_eq!(alpha(5, 2), 255, "the drawing landed where the anm2 put it");
    assert_eq!(alpha(4, 2), 0);
}

#[test]
fn without_a_paper_the_drawing_is_served_alone() {
    // Degrade, never fail: a paper that is missing or unreadable costs the backing, never the
    // drawing, which is the thing the row is about.
    let drawing = dot(2, 2, (1, 1));
    assert_eq!(on_paper(None, drawing.clone(), (0, 0)), drawing);
    assert_eq!(
        on_paper(Some(b"not a png"), drawing.clone(), (0, 0)),
        drawing
    );
}
