# Cycle 3.2 — Completion: Implementation Plan

> **For agentic workers:** executed inline with superpowers:executing-plans (the owner
> delegated the whole sub-project: "comincia a fare in autonomia"). Steps use checkbox
> (`- [ ]`) syntax.

**Goal:** the Completion screen — the character × mark matrix with honest totals — on real
data, with the mark symbols and character heads served as crops of the user's own game sheets.

**Architecture:** `ipc` learns the marks map (column → anm2 layer → frame), two icon references
(`Mark`, `Head`) and a PNG crop; `MarksMatrix` gains `tainted`, `headUrl` and a parallel `art`
array, all additive. The Tauri crate only wires: the `completion` command passes the catalog and
the icon URL builder, the protocol handler crops. The frontend reads the matrix through a Pinia
store; every number on screen comes from pure functions in `lib/completion/`, tested first on the
reference profile's real matrix, which the fixtures carry.

**Tech Stack:** Rust 2021 (`png` 0.17, `catalog::anm2_frames`), Tauri 2, Vue 3.5, TypeScript,
Pinia 4.0.3, Tailwind v4.3, Reka UI 2.10.4, vue-i18n 11.4, Vitest.

**Spec:** `docs/superpowers/specs/2026-09-11-screens-completion-design.md`

## Global Constraints

- The IPC change is additive and handed on: `tainted`, `headUrl`, `art`; `totals.started` counts `bits & 3 != 0`.
- No game asset in the package; the design pack is read only under `import.meta.env.DEV` (fixtures, Kit).
- Degrade, never fail: no catalog → every URL `null`; a symbol that fails to load → fallback outfit; no counters → all `unknown` plus an `Alert`.
- No percentage anywhere on the screen; "unreadable" outside every denominator.
- Rust: no `unwrap`/`panic` outside tests, no `_ =>` on closed enums, `rename_all = "camelCase"` on every struct that crosses the IPC.
- Frontend: the five rules, `assertNever` on closed switches, `as const` objects, every visible string through `useMessages()`, character and boss names are data.
- This machine has no game installed: archive-backed tests declare their skip; the real sprites are looked at on the first launch on a machine with the game.
- Commits: Conventional Commits, English, **no `Co-Authored-By` or any Claude reference** (the owner's rule overrides any tool default), at logical boundaries.

---

### Task 1: The sheet crop moves to `ipc`

**Files:**
- Create: `crates/ipc/src/sprite_png.rs`, `crates/ipc/tests/sprite_png.rs`
- Modify: `crates/ipc/Cargo.toml` (`png = "0.17"`), `crates/ipc/src/lib.rs`
- Modify: `crates/design-export/src/atlas.rs` (loses `crop` and `mod crops`; `decode` wraps `ipc::decode_rgba`), `sheets.rs`, `images.rs`

**Interfaces:**
- Produces: `ipc::decode_rgba(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)>`; `ipc::crop_png(sheet: &[u8], x: u32, y: u32, w: u32, h: u32) -> Option<Vec<u8>>`.

- [ ] **Step 1: Write the failing test** — `crates/ipc/tests/sprite_png.rs`

```rust
//! Cutting a piece out of a game sheet. Moved from `design-export` with B13: the app serves
//! the mark symbols and the character heads as crops, and a crop that takes the wrong cell is
//! a picture that looks plausible and is wrong.

use ipc::{crop_png, decode_rgba};

fn encode(w: u32, h: u32, pixel: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().unwrap();
        writer.write_image_data(pixel).unwrap();
    }
    out
}

/// A `w`×`h` sheet whose left half is `left` and whose right half is `right`.
fn two_halves(w: u32, h: u32, left: [u8; 4], right: [u8; 4]) -> Vec<u8> {
    let pixel: Vec<u8> = (0..h)
        .flat_map(|_| (0..w).flat_map(move |x| if x < w / 2 { left } else { right }))
        .collect();
    encode(w, h, &pixel)
}

#[test]
fn the_crop_takes_exactly_the_requested_rectangle() {
    let sheet = two_halves(8, 4, [255, 0, 0, 255], [0, 255, 0, 255]);
    let green = crop_png(&sheet, 4, 0, 4, 4).expect("the right half");
    let (w, h, pixel) = decode_rgba(&green).expect("the crop is a PNG");
    assert_eq!((w, h), (4, 4));
    assert!(
        pixel.chunks_exact(4).all(|p| p == [0, 255, 0, 255]),
        "the crop took the wrong cell"
    );
}

#[test]
fn a_rect_past_the_edge_is_clipped_not_dropped() {
    let sheet = two_halves(4, 4, [1, 2, 3, 255], [1, 2, 3, 255]);
    let corner = crop_png(&sheet, 2, 2, 8, 8).expect("the bottom-right corner remains");
    let (w, h, _) = decode_rgba(&corner).unwrap();
    assert_eq!((w, h), (2, 2));
}

#[test]
fn past_the_sheet_edge_no_empty_png_is_invented() {
    let sheet = two_halves(4, 4, [1, 2, 3, 255], [1, 2, 3, 255]);
    assert!(crop_png(&sheet, 4, 0, 4, 4).is_none(), "x right on the edge");
    assert!(crop_png(&sheet, 0, 9, 4, 4).is_none(), "y past the sheet");
    assert!(crop_png(&sheet, 0, 0, 0, 4).is_none(), "zero width");
}

#[test]
fn something_that_is_not_a_png_is_not_cropped() {
    assert!(crop_png(b"nothing", 0, 0, 1, 1).is_none());
    assert!(decode_rgba(b"nothing").is_none());
}
```

- [ ] **Step 2: Run it to see it fail** — `cargo test -p ipc --test sprite_png` → FAIL, unresolved imports `crop_png`, `decode_rgba`.

- [ ] **Step 3: `crates/ipc/src/sprite_png.rs`** — the body of `design-export`'s `decode` and `crop`, returning a tuple instead of the private `Image`:

```rust
//! A piece of a game sheet, as a PNG of its own.
//!
//! The game ships some pictures joined in one sheet — `completion_widget.png`, the co-op
//! menu's heads — and the anm2 files say where to cut. The app serves those pieces through
//! its icon protocol, so the cut lives in this pure crate: bytes in, bytes out.

/// Decodes a PNG to 8-bit RGBA, whatever its internal format (palette, grayscale, no alpha
/// channel): the game's sprites aren't all the same type. `(width, height, pixels)`.
pub fn decode_rgba(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut d = png::Decoder::new(bytes);
    d.set_transformations(png::Transformations::normalize_to_color8() | png::Transformations::ALPHA);
    let mut reader = d.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    let (w, h) = (info.width, info.height);
    let expected = (w as usize) * (h as usize);
    let pixel = match info.color_type {
        png::ColorType::Rgba => {
            buf.truncate(expected * 4);
            buf
        }
        // Grayscale with alpha: two channels, expanded to four.
        png::ColorType::GrayscaleAlpha => buf
            .chunks_exact(2)
            .take(expected)
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        png::ColorType::Grayscale | png::ColorType::Rgb | png::ColorType::Indexed => return None,
    };
    (pixel.len() >= expected * 4).then_some((w, h, pixel))
}

/// Crops a rectangle out of a sheet and turns it into a standalone PNG.
///
/// A rectangle that runs past the sheet's edge is **clipped to what's there**, not rejected:
/// the anm2 files occasionally declare cells bigger than the sheet, and losing the icon over
/// that would be worse than giving it a bit smaller. If nothing is left, `None`.
pub fn crop_png(sheet: &[u8], x: u32, y: u32, w: u32, h: u32) -> Option<Vec<u8>> {
    let (sheet_w, sheet_h, pixel) = decode_rgba(sheet)?;
    if x >= sheet_w || y >= sheet_h {
        return None;
    }
    let w = w.min(sheet_w - x);
    let h = h.min(sheet_h - y);
    if w == 0 || h == 0 {
        return None;
    }
    let cut: Vec<u8> = (0..h)
        .flat_map(|row| {
            let from = (((y + row) as usize) * (sheet_w as usize) + x as usize) * 4;
            pixel[from..from + (w as usize) * 4].iter().copied()
        })
        .collect();
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, w, h);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header().ok()?;
        writer.write_image_data(&cut).ok()?;
    }
    Some(out)
}
```

`crates/ipc/Cargo.toml`: `png = "0.17"` under `[dependencies]`. `crates/ipc/src/lib.rs`: `mod sprite_png;` and `pub use sprite_png::{crop_png, decode_rgba};`.

- [ ] **Step 4: Run it to see it pass** — `cargo test -p ipc --test sprite_png` → 4 passed.

- [ ] **Step 5: `design-export` imports the crop** — in `atlas.rs` delete `pub fn crop` with its doc comment and the whole `mod crops`, and replace `decode`'s body:

```rust
fn decode(byte: &[u8]) -> Option<Image> {
    ipc::decode_rgba(byte).map(|(w, h, pixel)| Image { w, h, pixel })
}
```

`sheets.rs`: `use crate::atlas::crop;` → `use ipc::crop_png as crop;`. `images.rs:391`: `crate::atlas::crop(` → `ipc::crop_png(`.

- [ ] **Step 6: Run the two crates** — `cargo test -p ipc -p design-export && cargo clippy -p ipc -p design-export --all-targets -- -D warnings` → green.

- [ ] **Step 7: Commit** — `refactor(ipc): the sheet crop moves to ipc, where the app can reach it`

---

### Task 2: Two icon references, `Mark` and `Head`

**Files:**
- Modify: `crates/ipc/src/icon.rs`, `crates/ipc/src/lib.rs`
- Test: `crates/ipc/tests/icon.rs`

**Interfaces:**
- Consumes: `ipc::{BOSSES, CHARACTERS, character_for}`.
- Produces: `ipc::MarkTier { Normal, Hard }`; `IconRef::Mark { column: usize, tier: MarkTier }` ↔ `mark/<column>/<normal|hard>`; `IconRef::Head { row: usize }` ↔ `head/<row>`; `icon_source` resolves `Head` through the catalog and answers `None` for `Mark`.

- [ ] **Step 1: Write the failing tests** — append to `crates/ipc/tests/icon.rs`

```rust
use ipc::MarkTier;

#[test]
fn marks_and_heads_survive_the_round_trip() {
    let all = [
        IconRef::Mark { column: 0, tier: MarkTier::Normal },
        IconRef::Mark { column: 9, tier: MarkTier::Hard },
        IconRef::Mark { column: 11, tier: MarkTier::Hard },
        IconRef::Head { row: 0 },
        IconRef::Head { row: 33 },
    ];
    for r in all {
        let path = r.to_path();
        assert_eq!(IconRef::parse(&path), Some(r), "round trip of {path:?}");
    }
    assert_eq!(IconRef::Mark { column: 9, tier: MarkTier::Hard }.to_path(), "mark/9/hard");
    assert_eq!(IconRef::Head { row: 17 }.to_path(), "head/17");
}

#[test]
fn a_mark_or_head_outside_the_matrix_parses_to_nothing() {
    // Twelve columns and 34 rows: past them the handler answers 400 before it opens an archive.
    for bad in [
        "mark", "mark/0", "mark/12/hard", "mark/0/wizard", "mark/x/hard", "mark/0/hard/extra",
        "head", "head/", "head/34", "head/x", "head/0/extra",
    ] {
        assert_eq!(IconRef::parse(bad), None, "{bad:?} must not parse");
    }
}

const PLAYERS: &[u8] = b"<players portraitroot=\"gfx/ui/stage/\">
<player id=\"0\" name=\"#ISAAC_NAME\" portrait=\"PlayerPortrait_Isaac.png\" />
<player id=\"21\" name=\"#ISAAC_NAME\" portrait=\"PlayerPortrait_Isaac_b.png\" />
</players>";

/// `coop menu.anm2` shaped like the game's: frame 0 without a crop (the "?" placeholder),
/// then one 32px cell per frame, eight to a row.
fn coop_menu_anm2() -> Vec<u8> {
    let frames: String = (1..=37)
        .map(|f| {
            format!(
                r#"<Frame XCrop="{}" YCrop="{}" Width="32" Height="32" Visible="true"/>"#,
                32 * (f % 8),
                32 * (f / 8)
            )
        })
        .collect();
    format!(
        r#"<AnimatedActor><Content><Spritesheets><Spritesheet Path="coop menu.png" Id="0"/></Spritesheets><Layers><Layer Name="Main" Id="0" SpritesheetId="0"/></Layers></Content><Animations><Animation Name="Main"><LayerAnimations><LayerAnimation LayerId="0"><Frame Delay="1" Visible="true"/>{frames}</LayerAnimation></LayerAnimations></Animation></Animations></AnimatedActor>"#
    )
    .into_bytes()
}

fn catalog_with_heads() -> Catalog {
    let anm2 = coop_menu_anm2();
    Catalog::build(|p| match p {
        "players.xml" => Some(PLAYERS.to_vec()),
        "gfx/ui/coop menu.anm2" => Some(anm2.clone()),
        _ => None,
    })
}

#[test]
fn a_head_resolves_through_the_matrix_row() {
    let c = catalog_with_heads();
    let at = |row| {
        icon_source(&c, &IconRef::Head { row })
            .map(|s| (s.path.clone(), s.rect.map(|r| (r.x, r.y, r.w, r.h))))
    };
    // Row 0 is Isaac (id 0, frame 1); row 17 is T. Isaac (id 21, frame 21 = column 5, row 2).
    assert_eq!(at(0), Some(("gfx/ui/coop menu.png".to_string(), Some((32, 0, 32, 32)))));
    assert_eq!(at(17), Some(("gfx/ui/coop menu.png".to_string(), Some((160, 64, 32, 32)))));
    assert_eq!(at(1), None, "Magdalene isn't in this catalog: no head, not a neighbour's");
}

#[test]
fn a_mark_is_not_in_the_catalog() {
    let c = catalog_with_heads();
    assert!(icon_source(&c, &IconRef::Mark { column: 0, tier: MarkTier::Hard }).is_none());
}
```

- [ ] **Step 2: Run to see them fail** — `cargo test -p ipc --test icon` → FAIL, no `MarkTier`, no variant `Mark`.

- [ ] **Step 3: Implement** — in `crates/ipc/src/icon.rs`:

```rust
use crate::marks::{character_for, BOSSES, CHARACTERS};

/// The two levels of a mark. The game draws them as two different symbols, not one tinted
/// (DESIGN-BRIEF.md §5.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkTier {
    Normal,
    Hard,
}
```

Add to `IconRef`:

```rust
    /// A column's mark symbol: a piece of `completion_widget.png` or of the online lobby's
    /// sheet, resolved by `mark_source`, never by the catalog.
    Mark { column: usize, tier: MarkTier },
    /// The co-op menu head of a matrix row, resolved through `character_for`.
    Head { row: usize },
```

Tokens and paths:

```rust
fn tier_token(t: MarkTier) -> &'static str {
    match t {
        MarkTier::Normal => "normal",
        MarkTier::Hard => "hard",
    }
}

fn tier_from_token(s: &str) -> Option<MarkTier> {
    match s {
        "normal" => Some(MarkTier::Normal),
        "hard" => Some(MarkTier::Hard),
        // A string from a webview, not a closed enum: anything else isn't ours.
        _ => None,
    }
}
```

`to_path` gains `IconRef::Mark { column, tier } => format!("mark/{column}/{}", tier_token(*tier))` and `IconRef::Head { row } => format!("head/{row}")`. `parse` gains:

```rust
            ("mark", column, Some(tier)) => IconRef::Mark {
                column: column.parse().ok().filter(|&c| c < BOSSES.len())?,
                tier: tier_from_token(tier)?,
            },
            ("head", row, None) => IconRef::Head {
                row: row.parse().ok().filter(|&r| r < CHARACTERS.len())?,
            },
```

`icon_source` gains:

```rust
        IconRef::Head { row } => character_for(row, c).and_then(|ch| ch.head.as_ref()),
        // Not the catalog's: the symbols live on the widget's sheets, see `mark_source`.
        IconRef::Mark { .. } => None,
```

`lib.rs`: `pub use icon::{icon_source, IconRef, MarkTier, ICON_SCHEME};`.

- [ ] **Step 4: Run to see them pass** — `cargo test -p ipc --test icon` → green; `cargo build -p app` still compiles (the handler's `match` doesn't exist yet; `icon_source` is the only exhaustive consumer).

- [ ] **Step 5: Commit** — `feat(ipc): icon references for a mark's symbol and a row's head`

---

### Task 3: The marks map moves into `ipc`

**Files:**
- Create: `crates/ipc/src/mark_art.rs`, `crates/ipc/tests/mark_art.rs`, `crates/ipc/tests/mark_art_real.rs`
- Modify: `crates/ipc/src/lib.rs`

**Interfaces:**
- Consumes: `catalog::{Anm2Frame, SpriteRef, anm2_frames}`, `ipc::MarkTier`.
- Produces: `ipc::WIDGET_ANM2`, `ipc::LOBBY_ANM2`; `ipc::MarkFrames { widget: Vec<Anm2Frame>, lobby: Vec<Anm2Frame> }` (`Default`); `ipc::mark_source(column: usize, tier: MarkTier, frames: &MarkFrames) -> Option<SpriteRef>`.

- [ ] **Step 1: Write the failing tests** — `crates/ipc/tests/mark_art.rs`

```rust
//! Which piece of which sheet draws a column's mark (B13). The anm2 files are synthetic,
//! shaped like the game's; `mark_art_real.rs` runs the same map on the installed game.

use catalog::{anm2_frames, SpriteRef};
use ipc::{mark_source, MarkFrames, MarkTier, BOSSES};

const WIDGET_LAYERS: [&str; 11] = [
    "Heart", "Polaroid", "UpsideDownCross", "Star", "Negative", "Cross", "MegaSatan", "Greed",
    "Hush", "Knife", "DadsNote",
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
                format!(r#"<Frame XCrop="{x}" YCrop="{}" Width="16" Height="16" Visible="true"/>"#, 32 * f)
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
    for column in 0..BOSSES.len() {
        let normal = place(mark_source(column, MarkTier::Normal, &f));
        let hard = place(mark_source(column, MarkTier::Hard, &f));
        assert!(normal.is_some() && hard.is_some(), "{} has no symbol", BOSSES[column]);
        assert_ne!(normal, hard, "{}: the two tiers are two drawings", BOSSES[column]);
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
    assert_eq!(place(mark_source(9, MarkTier::Normal, &f)), Some((lobby.clone(), 224, 0)));
    assert_eq!(place(mark_source(9, MarkTier::Hard, &f)), Some((lobby, 224, 64)));
}

#[test]
fn a_missing_layer_or_frame_is_nothing_not_a_neighbour() {
    let only_heart_frame_0 = MarkFrames {
        widget: anm2_frames(
            br#"<AnimatedActor><Content><Spritesheets><Spritesheet Id="0" Path="completion_widget.png"/></Spritesheets><Layers><Layer Id="0" Name="Heart"/></Layers></Content><Animations><Animation Name="Idle"><LayerAnimations><LayerAnimation LayerId="0"><Frame XCrop="0" YCrop="0" Width="16" Height="16" Visible="false"/></LayerAnimation></LayerAnimations></Animation></Animations></AnimatedActor>"#,
        )
        .unwrap(),
        lobby: Vec::new(),
    };
    assert!(mark_source(0, MarkTier::Normal, &only_heart_frame_0).is_some());
    assert!(mark_source(0, MarkTier::Hard, &only_heart_frame_0).is_none(), "no frame 2");
    assert!(mark_source(1, MarkTier::Normal, &only_heart_frame_0).is_none(), "no Polaroid");
    assert!(mark_source(9, MarkTier::Hard, &only_heart_frame_0).is_none(), "no lobby");
    assert!(mark_source(12, MarkTier::Hard, &frames()).is_none(), "no thirteenth column");
    assert!(mark_source(0, MarkTier::Hard, &MarkFrames::default()).is_none());
}
```

`crates/ipc/tests/mark_art_real.rs`:

```rust
//! The marks map against the installed game: both anm2 files read, every column resolves
//! to two tiers, and each crop is a real 16×16 piece of a sheet that exists.

use ipc::{crop_png, decode_rgba, mark_source, MarkFrames, MarkTier, BOSSES, LOBBY_ANM2, WIDGET_ANM2};

#[test]
fn every_column_crops_two_real_symbols() {
    let Some(dir) = test_support::packed_dir() else {
        return;
    };
    let rs = unpack::ResourceSet::open(&dir);
    let read = |p: &str| catalog::anm2_frames(&rs.read(p).unwrap_or_default()).unwrap_or_default();
    let frames = MarkFrames { widget: read(WIDGET_ANM2), lobby: read(LOBBY_ANM2) };
    for column in 0..BOSSES.len() {
        for tier in [MarkTier::Normal, MarkTier::Hard] {
            let sprite = mark_source(column, tier, &frames)
                .unwrap_or_else(|| panic!("{} {tier:?}: no frame", BOSSES[column]));
            let sheet = rs
                .read(&sprite.path)
                .unwrap_or_else(|| panic!("{}: sheet {} not in the archives", BOSSES[column], sprite.path));
            let r = sprite.rect.expect("a mark is a piece of a sheet");
            let png = crop_png(&sheet, r.x, r.y, r.w, r.h).expect("the rectangle is inside the sheet");
            let (w, h, _) = decode_rgba(&png).unwrap();
            assert_eq!((w, h), (16, 16), "{} {tier:?}", BOSSES[column]);
        }
    }
}
```

- [ ] **Step 2: Run to see them fail** — `cargo test -p ipc --test mark_art --test mark_art_real` → FAIL, unresolved `mark_source`.

- [ ] **Step 3: `crates/ipc/src/mark_art.rs`**

```rust
//! Which piece of which game sheet draws a column's mark.
//!
//! Domain knowledge, so it sits beside `BOSSES` and follows its order (DESIGN-BRIEF.md §5.6,
//! B13). It lived in `crates/design-export`'s `marks.json` while only the design pack needed
//! it; the app serves the symbols now, and a map in two places drifts.
//!
//! Eleven columns are layers of `completion_widget.anm2`. The twelfth, Delirium, is named
//! only by Repentance+'s online lobby, on its own sheet — and the lobby draws it twice, on
//! the player card and on the background, so its animation is part of the key.

use catalog::{Anm2Frame, SpriteRef};

use crate::icon::MarkTier;

pub const WIDGET_ANM2: &str = "gfx/ui/completion_widget.anm2";
pub const LOBBY_ANM2: &str = "gfx/ui/main menu/onlinelobby.anm2";

#[derive(Debug, Clone, Copy)]
enum Source {
    Widget,
    Lobby,
}

struct MarkLayer {
    source: Source,
    /// `None` where the file has one animation and the layer alone is unique.
    animation: Option<&'static str>,
    layer: &'static str,
}

const fn widget(layer: &'static str) -> MarkLayer {
    MarkLayer { source: Source::Widget, animation: None, layer }
}

/// One row per column of `BOSSES`, in its order. Every row but The Lamb is read from a layer
/// name; The Lamb is `Cross` by elimination, the one symbol and the one column left once every
/// other pairing is settled.
const MARK_LAYERS: [MarkLayer; 12] = [
    widget("Heart"),           // Mom's Heart
    widget("Polaroid"),        // Isaac
    widget("UpsideDownCross"), // Satan
    widget("Star"),            // Boss Rush
    widget("Negative"),        // Blue Baby
    widget("Cross"),           // The Lamb, by elimination
    widget("MegaSatan"),       // Mega Satan
    widget("Greed"),           // Greed
    widget("Hush"),            // Hush
    MarkLayer {
        source: Source::Lobby,
        animation: Some("Background"),
        layer: "Completion_Delirium",
    }, // Delirium
    widget("Knife"),    // Mother
    widget("DadsNote"), // The Beast
];

/// The frames of the two anm2 files, as `catalog::anm2_frames` reads them.
#[derive(Debug, Clone, Default)]
pub struct MarkFrames {
    pub widget: Vec<Anm2Frame>,
    pub lobby: Vec<Anm2Frame>,
}

/// A tier is a frame of its layer: the rectangles the design pack cut into `heart_00.png`
/// and `heart_02.png`, drawn as the two levels on the Kit page. Frame 0 is declared hidden
/// (the "not taken" state); whether frame 1 repeats its rectangle waits for a machine with
/// the game, and if it doesn't, `Normal` becomes frame 1 here and nowhere else.
fn frame_index(tier: MarkTier) -> usize {
    match tier {
        MarkTier::Normal => 0,
        MarkTier::Hard => 2,
    }
}

/// A sheet named by an anm2 sits in the anm2's folder, as `design-export` reads it.
fn sheet_path(anm2: &str, sheet: &str) -> String {
    match anm2.rsplit_once('/') {
        Some((dir, _)) => format!("{dir}/{sheet}"),
        None => sheet.to_string(),
    }
}

/// The piece that draws `column`'s mark at `tier`. `None` when the layer, the animation or
/// the frame isn't there: never a neighbour's picture.
pub fn mark_source(column: usize, tier: MarkTier, frames: &MarkFrames) -> Option<SpriteRef> {
    let m = MARK_LAYERS.get(column)?;
    let (anm2, list) = match m.source {
        Source::Widget => (WIDGET_ANM2, &frames.widget),
        Source::Lobby => (LOBBY_ANM2, &frames.lobby),
    };
    let f = list.iter().find(|f| {
        f.layer == m.layer
            && m.animation.is_none_or(|a| f.animation == a)
            && f.index == frame_index(tier)
    })?;
    Some(SpriteRef {
        path: sheet_path(anm2, &f.sheet),
        rect: Some(f.rect),
    })
}
```

`lib.rs`: `mod mark_art;` and `pub use mark_art::{mark_source, MarkFrames, LOBBY_ANM2, WIDGET_ANM2};`. (If `rustc` predates `Option::is_none_or`, 1.82, use `m.animation.map_or(true, …)` and allow nothing: check `rustc --version` first.)

- [ ] **Step 4: Run to see them pass** — `cargo test -p ipc --test mark_art --test mark_art_real -- --nocapture` → 4 passed, 1 passing with `skip: samples/packed missing` on this machine.

- [ ] **Step 5: Commit** — `feat(ipc): the marks map, from design-export to beside BOSSES`

---

### Task 4: The matrix carries its art, and "started" agrees with the grid

**Files:**
- Modify: `crates/ipc/src/marks.rs`, `crates/ipc/src/lib.rs`
- Test: `crates/ipc/tests/marks.rs`
- Modify (callers): `crates/ipc/tests/cross_check.rs:98`, `crates/design-export/src/payload.rs:230`, `crates/app/src/lib.rs:109-113`

**Interfaces:**
- Consumes: `IconRef::{Mark, Head}`, `MarkTier`, `character_for`.
- Produces: `CharacterRow.tainted: bool`, `CharacterRow.head_url: Option<String>`; `ipc::MarkArtView { normal_url, hard_url }`; `MarksMatrix.art: Vec<MarkArtView>`; `marks_matrix(counters: &[u32], catalog: Option<&catalog::Catalog>, icon: impl FnMut(&IconRef) -> Option<String>) -> MarksMatrix`.

- [ ] **Step 1: Write the failing tests** — in `crates/ipc/tests/marks.rs`, every existing `marks_matrix(x)` becomes `marks_matrix(x, None, no_icon)`, and:

```rust
use catalog::Catalog;
use ipc::{IconRef, MarkArtView};

fn no_icon(_: &IconRef) -> Option<String> {
    None
}

#[test]
fn rows_know_whether_they_are_tainted() {
    let m = marks_matrix(&counters(523, &[]), None, no_icon);
    assert!(!m.characters[0].tainted, "Isaac");
    assert!(!m.characters[14].tainted, "The Forgotten");
    assert!(!m.characters[16].tainted, "Jacob & Esau");
    assert!(m.characters[17].tainted, "T. Isaac");
    assert!(m.characters[33].tainted, "T. Jacob & Esau");
    assert_eq!(m.characters.iter().filter(|r| r.tainted).count(), 17);
}

#[test]
fn without_a_catalog_nothing_carries_a_url() {
    let m = marks_matrix(&counters(523, &[]), None, |r| Some(r.to_path()));
    assert_eq!(m.art.len(), m.bosses.len(), "art[i] draws bosses[i]");
    assert!(m.art.iter().all(|a| a.normal_url.is_none() && a.hard_url.is_none()));
    assert!(m.characters.iter().all(|r| r.head_url.is_none()));
}

const PLAYERS: &[u8] = b"<players portraitroot=\"gfx/ui/stage/\">
<player id=\"0\" name=\"#ISAAC_NAME\" portrait=\"PlayerPortrait_Isaac.png\" />
<player id=\"21\" name=\"#ISAAC_NAME\" portrait=\"PlayerPortrait_Isaac_b.png\" />
</players>";

fn coop_menu_anm2() -> Vec<u8> {
    let frames: String = (1..=37)
        .map(|f| format!(r#"<Frame XCrop="{}" YCrop="0" Width="32" Height="32" Visible="true"/>"#, 32 * f))
        .collect();
    format!(
        r#"<AnimatedActor><Content><Spritesheets><Spritesheet Path="coop menu.png" Id="0"/></Spritesheets><Layers><Layer Name="Main" Id="0" SpritesheetId="0"/></Layers></Content><Animations><Animation Name="Main"><LayerAnimations><LayerAnimation LayerId="0"><Frame Delay="1" Visible="true"/>{frames}</LayerAnimation></LayerAnimations></Animation></Animations></AnimatedActor>"#
    )
    .into_bytes()
}

#[test]
fn with_a_catalog_urls_follow_what_it_knows() {
    let anm2 = coop_menu_anm2();
    let c = Catalog::build(|p| match p {
        "players.xml" => Some(PLAYERS.to_vec()),
        "gfx/ui/coop menu.anm2" => Some(anm2.clone()),
        _ => None,
    });
    let m = marks_matrix(&counters(523, &[]), Some(&c), |r| Some(r.to_path()));
    assert_eq!(m.characters[0].head_url.as_deref(), Some("head/0"), "Isaac has a head");
    assert_eq!(m.characters[17].head_url.as_deref(), Some("head/17"), "T. Isaac has a head");
    assert_eq!(m.characters[1].head_url, None, "Magdalene isn't in this catalog");
    assert_eq!(
        m.art[9],
        MarkArtView {
            normal_url: Some("mark/9/normal".to_string()),
            hard_url: Some("mark/9/hard".to_string()),
        }
    );
}

#[test]
fn a_cell_holding_only_the_unconfirmed_bit_is_not_started() {
    // 4 has never been observed; if it appears, the grid draws it empty, and so must the total.
    let m = marks_matrix(&counters(523, &[(27, 4), (41, 5)]), None, no_icon);
    assert_eq!(m.totals.started, 1, "5 carries the normal mark, 4 carries nothing the grid draws");
}

#[test]
fn the_new_fields_are_camel_case_on_the_wire() {
    let m = marks_matrix(&counters(523, &[]), None, no_icon);
    let json = serde_json::to_value(&m).unwrap();
    let row = json["characters"][0].as_object().unwrap();
    assert!(row.contains_key("tainted") && row.contains_key("headUrl"), "{row:?}");
    let art = json["art"][0].as_object().unwrap();
    assert!(art.contains_key("normalUrl") && art.contains_key("hardUrl"), "{art:?}");
}
```

`cross_check.rs:98`: `marks_matrix(&counters, None, |_| None)`.

- [ ] **Step 2: Run to see them fail** — `cargo test -p ipc --test marks` → FAIL, wrong number of arguments.

- [ ] **Step 3: Implement** — in `crates/ipc/src/marks.rs`:

```rust
use crate::icon::{IconRef, MarkTier};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterRow {
    pub character: String,
    pub group: CharacterGroup,
    /// The Tainted form: the screen groups rows the way a player does, base and Tainted,
    /// while `group` stays the file's three blocks.
    pub tainted: bool,
    pub cells: Vec<Cell>,
    /// The co-op menu head; `None` without a catalog or for a character the menu doesn't draw.
    pub head_url: Option<String>,
}

/// The symbol URLs of one column, one per tier. Both `None` when the game's archives aren't
/// open: the screen draws the fallback outfit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkArtView {
    pub normal_url: Option<String>,
    pub hard_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarksMatrix {
    pub characters: Vec<CharacterRow>,
    pub bosses: Vec<String>,
    /// `art[i]` draws `bosses[i]`: a parallel array, so `bosses` keeps the shape the design
    /// was built on.
    pub art: Vec<MarkArtView>,
    pub totals: MarksTotals,
}

/// Builds the matrix from the counters read out of the file. It assumes no fixed length: an
/// index past the section read produces `Unknown`. With a catalog (the game's archives are
/// open) rows and columns carry the URLs `icon` builds; without one, none.
pub fn marks_matrix(
    counters: &[u32],
    catalog: Option<&catalog::Catalog>,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> MarksMatrix {
    let rows: Vec<CharacterRow> = CHARACTERS
        .iter()
        .enumerate()
        .map(|(c, &(name, group))| CharacterRow {
            character: name.to_string(),
            group,
            tainted: CHARACTER_KEYS.get(c).is_some_and(|&(_, tainted)| tainted),
            cells: (0..BOSSES.len()).map(|b| cell_at(counters, c, b)).collect(),
            head_url: catalog
                .and_then(|cat| character_for(c, cat))
                .and_then(|ch| ch.head.as_ref())
                .and_then(|_| icon(&IconRef::Head { row: c })),
        })
        .collect();
    let art = (0..BOSSES.len())
        .map(|column| match catalog {
            Some(_) => MarkArtView {
                normal_url: icon(&IconRef::Mark { column, tier: MarkTier::Normal }),
                hard_url: icon(&IconRef::Mark { column, tier: MarkTier::Hard }),
            },
            None => MarkArtView { normal_url: None, hard_url: None },
        })
        .collect();
    let totals = totals_of(&rows);
    MarksMatrix {
        characters: rows,
        bosses: BOSSES.iter().map(|b| b.to_string()).collect(),
        art,
        totals,
    }
}
```

In `totals_of`: `started: count(|c| matches!(c, Cell::Known { bits } if bits & 3 != 0)),` with the comment "bit 0 or bit 1: the unconfirmed bit alone draws nothing (markVisual, cycle 2)". `lib.rs` re-exports `MarkArtView`.

Callers: `design-export/src/payload.rs:230` → `ipc::marks_matrix(&counters, Some(c), <the same icon closure this function passes to unlock_view>)`; `crates/app/src/lib.rs` `completion` → Task 5.

- [ ] **Step 4: Run to see them pass** — `cargo test -p ipc && cargo test -p design-export` → green (`crates/app` compiles in Task 5).

- [ ] **Step 5: Commit together with Task 5** (the workspace doesn't build between them).

---

### Task 5: The app serves the crops

**Files:**
- Modify: `crates/app/src/lib.rs`

**Interfaces:**
- Consumes: `ipc::{marks_matrix, mark_source, MarkFrames, WIDGET_ANM2, LOBBY_ANM2, crop_png, icon_source, IconRef}`.
- Produces: `completion` answers `art`/`headUrl` when the game is installed; `isaac://…/mark/<c>/<tier>` and `…/head/<row>` answer PNG crops.

- [ ] **Step 1: `MarkFramesState`**, beside `CatalogState`:

```rust
/// The frames of the two anm2 files the marks are cut from, read once. Kept only when both
/// read: a failed read — the game absent, an archive missing — is tried again next time.
#[derive(Default)]
struct MarkFramesState(OnceLock<ipc::MarkFrames>);

impl MarkFramesState {
    fn get(&self, rs: &ResourceSet) -> Option<&ipc::MarkFrames> {
        if let Some(f) = self.0.get() {
            return Some(f);
        }
        let widget = catalog::anm2_frames(&rs.read(ipc::WIDGET_ANM2)?)?;
        let lobby = catalog::anm2_frames(&rs.read(ipc::LOBBY_ANM2)?)?;
        Some(self.0.get_or_init(|| ipc::MarkFrames { widget, lobby }))
    }
}
```

- [ ] **Step 2: `completion`**

```rust
#[tauri::command]
fn completion(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<MarksMatrix, IpcError> {
    let (_, save) = active_save(&app)?;
    let counters = save.u32s(Kind::Counters).unwrap_or_default();
    // Game not installed is expected: the matrix goes out without art, and the screen draws
    // the fallback outfit.
    let catalog = resources.get().and_then(|rs| state.get_or_build(rs));
    Ok(ipc::marks_matrix(&counters, catalog, icon_url))
}
```

- [ ] **Step 3: `icon_bytes` crops** — replace the resolution and read:

```rust
    let sprite = match reference {
        ipc::IconRef::Mark { column, tier } => app
            .state::<MarkFramesState>()
            .get(rs)
            .and_then(|f| ipc::mark_source(column, tier, f)),
        ipc::IconRef::Achievement { .. } | ipc::IconRef::Item { .. } | ipc::IconRef::Head { .. } => app
            .state::<CatalogState>()
            .get_or_build(rs)
            .and_then(|c| ipc::icon_source(c, &reference).cloned()),
    };
    let Some(sprite) = sprite else {
        return no_icon(404);
    };
    let Some(png) = sprite_bytes(rs, &sprite) else {
        return no_icon(404);
    };
```

and add:

```rust
/// The file a sprite names, cropped when it names a piece of a sheet: the mark symbols and
/// the co-op menu heads are cells of one picture, achievements and items whole files.
fn sprite_bytes(rs: &ResourceSet, sprite: &catalog::SpriteRef) -> Option<Vec<u8>> {
    let file = rs.read(&sprite.path)?;
    match sprite.rect {
        None => Some(file),
        Some(r) => ipc::crop_png(&file, r.x, r.y, r.w, r.h),
    }
}
```

Update `icon_bytes`'s doc comment (the rect is no longer ignored) and register `.manage(MarkFramesState::default())`.

- [ ] **Step 4: Verify** — `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` → green.

- [ ] **Step 5: Commit** — `feat(ipc): the completion matrix carries its mark art and row heads` (Tasks 4 and 5).

---

### Task 6: The mirror, the fixtures and the development art

**Files:**
- Modify: `ui/src/lib/ipc/types.ts`, `ui/src/stores/profile.ts`, `ui/src/lib/ipc/fixtures/index.ts`, `ui/src/lib/ipc/transport.test.ts`, `ui/src/lib/constants/stores.ts`
- Create: `ui/src/lib/ipc/errors.ts`, `ui/src/lib/ipc/fixtures/art.ts`, `ui/src/lib/ipc/fixtures/completion.ts`
- Modify: `ui/src/kit/markArt.ts` (reads `fixtures/art.ts`), `ui/src/components/marks/markVisual.ts` (+ test)

**Interfaces:**
- Produces: TS `CharacterRow`, `MarkArtView`, `MarksMatrix.art`; `isIpcError(e): e is IpcError`; `packMarkArt: MarkArtView[]`, `packHeadUrl(row): string | null`; `completionMatrix(withArt: boolean): MarksMatrix`; `markArtOf(view: MarkArtView | undefined): MarkArt | null`; `StoreId.Completion`.

- [ ] **Step 1: Write the failing tests** — `transport.test.ts` gains:

```ts
import type { MarksMatrix } from './types'

describe('the completion fixture', () => {
  it('answers the reference matrix with an active profile', async () => {
    const m = await answer<MarksMatrix>(
      Command.Completion,
      undefined,
      FixtureScenario.Active,
    )
    expect(m.characters).toHaveLength(34)
    expect(m.art).toHaveLength(m.bosses.length)
    expect(m.totals).toEqual({
      cells: 408,
      readable: 368,
      unknown: 40,
      unexpected: 0,
      started: 166,
    })
  })

  it('refuses the matrix without an active profile, as the backend does', async () => {
    await expect(
      answer(Command.Completion, undefined, FixtureScenario.None),
    ).rejects.toEqual({ kind: 'noActiveProfile' })
  })
})
```

`markVisual.test.ts` gains:

```ts
describe('markArtOf', () => {
  it('draws a column only when both tiers have a URL', () => {
    expect(markArtOf({ normalUrl: 'n', hardUrl: 'h' })).toEqual({
      normal: 'n',
      hard: 'h',
    })
    expect(markArtOf({ normalUrl: 'n', hardUrl: null })).toBeNull()
    expect(markArtOf(undefined)).toBeNull()
  })
})
```

- [ ] **Step 2: Run to see them fail** — `pnpm --filter ui exec vitest run src/lib/ipc src/components/marks` → FAIL.

- [ ] **Step 3: Implement**
  - `types.ts`: `export interface CharacterRow { character: string; group: string; tainted: boolean; cells: Cell[]; headUrl: string | null }`, `export interface MarkArtView { normalUrl: string | null; hardUrl: string | null }`, and `MarksMatrix { characters: CharacterRow[]; bosses: string[]; art: MarkArtView[]; totals }` with a comment: `art[i]` draws `bosses[i]`; every URL is `null` when the game's archives aren't open.
  - `errors.ts`: `export const isIpcError = (e: unknown): e is IpcError => typeof e === 'object' && e !== null && 'kind' in e`; `stores/profile.ts` imports it.
  - `markVisual.ts`: `export const markArtOf = (view: MarkArtView | undefined): MarkArt | null => view?.normalUrl && view.hardUrl ? { normal: view.normalUrl, hard: view.hardUrl } : null`.
  - `fixtures/art.ts`: the globs of `kit/markArt.ts` re-rooted (`../../../../../design-export/isaacdome-design-pack/images/sheets/…`), plus `coop_menu/main_*.png`; `packMarkArt` in `ipc::BOSSES` order (`heart, polaroid, upsidedowncross, star, negative, cross, megasatan, greed, hush` from the widget, `background_completion_delirium` from the lobby, `knife, dadsnote`); `packHeadUrl(row)` through the rows' co-op menu frames `[1…11, 14, 15, 16, 17, 19, 20, 21…37]` (catalog's head map: id 0–19 → frame id + 1, id 21–37 → frame id; the "Jacob & Esau" row is Jacob).
  - `kit/markArt.ts`: `kitMarkArt = { heart: markArtOf(packMarkArt[0]), polaroid: markArtOf(packMarkArt[1]), star: markArtOf(packMarkArt[3]), delirium: markArtOf(packMarkArt[9]), knife: markArtOf(packMarkArt[10]) }`.
  - `fixtures/completion.ts`: the 34 rows `[name, digits, group, tainted]` from the design export's `completion.json` (Isaac `773223227331` … T. Jacob & Esau `0000000000??`), cells from digits (`?` → `unknown`), totals counted from the cells (`started` on `bits & 3`), `art` and `headUrl` from `art.ts` when `withArt`.
  - `fixtures/index.ts`: `[Command.Completion]: (_args, scenario) => setupFor(scenario).active.kind === 'active' ? completionMatrix(artShown()) : Promise.reject(noActiveProfile)`, where `artShown()` is false for `?art=none`.
  - `stores.ts`: `Completion: 'completion'`.

- [ ] **Step 4: Run to see them pass** — `pnpm --filter ui exec vitest run && pnpm typecheck` → green.

- [ ] **Step 5: Commit** — `feat(ui): the completion matrix's mirror, fixture and development art`

---

### Task 7: What the screen counts

**Files:**
- Create: `ui/src/lib/completion/completionView.ts`, `ui/src/lib/completion/completionView.test.ts`

**Interfaces:**
- Consumes: `completionMatrix(false)` (tests only).
- Produces: `CellStatus { Empty, Normal, Hard, Both, Unknown, Unexpected }`; `cellReading(cell): { status: CellStatus; third: boolean }`; `Tally { started; readable; complete }`; `rowTally(row)`; `columnTallies(matrix): Tally[]`; `MatrixGroup { Base, Tainted }`; `matrixGroups(matrix): GroupView[]` with `GroupView { group; rows: { row: CharacterRow; index: number; tally: Tally }[]; first; last; started; readable; unknown }`; `completionKpis(matrix): CompletionKpis { started; readable; both; completeCharacters; characters; unknown; cells }`.

- [ ] **Step 1: Write the failing test** — expectations from DESIGN-BRIEF.md §5.4 and the Kit page's tiles (166 / 368, 120 cells, 3 / 34, 40 unreadable), and counted by hand on the export's digit strings:

```ts
import { describe, expect, it } from 'vitest'
import { completionMatrix } from '@/lib/ipc/fixtures/completion'
import {
  CellStatus,
  MatrixGroup,
  cellReading,
  columnTallies,
  completionKpis,
  matrixGroups,
  rowTally,
} from './completionView'

const reference = completionMatrix(false)
const row = (name: string) => {
  const found = reference.characters.find((r) => r.character === name)
  if (!found) throw new Error(`no row ${name}`)
  return found
}

describe('completionKpis on the reference profile', () => {
  it('states the totals §5.4 printed', () => {
    expect(completionKpis(reference)).toEqual({
      started: 166,
      readable: 368,
      both: 120,
      completeCharacters: 3,
      characters: 34,
      unknown: 40,
      cells: 408,
    })
  })
})

describe('rowTally', () => {
  it('calls a row complete when every readable cell is started', () => {
    expect(rowTally(row('Isaac'))).toEqual({ started: 12, readable: 12, complete: true })
  })
  it('keeps unreadable cells out of the denominator', () => {
    expect(rowTally(row('The Forgotten'))).toEqual({ started: 9, readable: 10, complete: false })
  })
  it('counts an untouched row as zero of its readable cells', () => {
    expect(rowTally(row('T. Magdalene'))).toEqual({ started: 0, readable: 10, complete: false })
  })
})

describe('columnTallies', () => {
  it('counts each boss over the characters whose cell is readable', () => {
    const tallies = columnTallies(reference)
    expect(tallies).toHaveLength(12)
    expect(tallies[0]).toEqual({ started: 21, readable: 34, complete: false })
    expect(tallies[11]).toEqual({ started: 5, readable: 14, complete: false })
  })
})

describe('matrixGroups', () => {
  it('splits base and Tainted, each saying what it cannot read', () => {
    const [base, tainted] = matrixGroups(reference)
    expect(base).toMatchObject({ group: MatrixGroup.Base, first: 'Isaac', last: 'Jacob & Esau', unknown: 6 })
    expect(base?.rows).toHaveLength(17)
    expect(tainted).toMatchObject({ group: MatrixGroup.Tainted, first: 'T. Isaac', last: 'T. Jacob & Esau', unknown: 34 })
    expect(tainted?.rows[0]?.index).toBe(17)
  })
})

describe('cellReading', () => {
  const known = (bits: number) => cellReading({ kind: 'known', bits })
  it('reads the two levels apart and together', () => {
    expect(known(0)).toEqual({ status: CellStatus.Empty, third: false })
    expect(known(1)).toEqual({ status: CellStatus.Normal, third: false })
    expect(known(2)).toEqual({ status: CellStatus.Hard, third: false })
    expect(known(3)).toEqual({ status: CellStatus.Both, third: false })
  })
  it('carries the unconfirmed bit beside the level, never as one', () => {
    expect(known(4)).toEqual({ status: CellStatus.Empty, third: true })
    expect(known(5)).toEqual({ status: CellStatus.Normal, third: true })
    expect(known(7)).toEqual({ status: CellStatus.Both, third: true })
  })
  it('keeps what it cannot read, and what it should not see, apart', () => {
    expect(known(8)).toEqual({ status: CellStatus.Unexpected, third: false })
    expect(cellReading({ kind: 'unknown' })).toEqual({ status: CellStatus.Unknown, third: false })
    expect(cellReading({ kind: 'unexpected', value: 49 })).toEqual({ status: CellStatus.Unexpected, third: false })
  })
})
```

- [ ] **Step 2: Run to see it fail** — `pnpm --filter ui exec vitest run src/lib/completion` → FAIL, no module.

- [ ] **Step 3: Implement** `completionView.ts` — a `switch` with `assertNever` over `cell.kind`; bits outside 0–7 are `Unexpected`; `isStarted` = `Normal | Hard | Both`; `isReadable` = not `Unknown` and not `Unexpected`; one `tallyOf(cells)` behind `rowTally`, `columnTallies` and the groups; groups built by `tainted`, dropping an empty group.

- [ ] **Step 4: Run to see it pass** — green, and `pnpm typecheck`.

- [ ] **Step 5: Commit** — `feat(ui): what the Completion screen counts, as pure functions`

---

### Task 8: The screen

**Files:**
- Create: `ui/src/stores/completion.ts`, `ui/src/screens/CompletionScreen.vue`, `ui/src/screens/completion/CompletionKpis.vue`, `MarksMatrixCard.vue`, `MatrixLegend.vue`, `ui/src/components/marks/MarksGrid.vue`, `ui/src/components/sprite/PixelSprite.vue`
- Modify: `ui/src/components/marks/MarkCell.vue` (a symbol that fails falls back), `ui/src/assets/theme/spacing.css`, `ui/src/assets/utilities.css`, `ui/src/router/routes.ts`, `ui/src/router/routeTable.ts`, `ui/src/i18n/messages/it.ts`, `en.ts`

- [ ] **Step 1: Tokens and utilities** — `spacing.css`: `--spacing-matrix-name: 176px; --spacing-matrix-total: 52px; --spacing-matrix-header: 118px;` under a comment naming `Schermate.dc.html`. `utilities.css`:

```css
/* The completion matrix: the name column, one mark cell per boss, the row total. The column
   count is data, bound from the template as --matrix-columns. */
@utility grid-cols-matrix {
  grid-template-columns:
    var(--spacing-matrix-name)
    repeat(var(--matrix-columns), var(--spacing-mark-cell))
    var(--spacing-matrix-total);
}

/* A column header read bottom to top. */
@utility writing-vertical {
  writing-mode: vertical-rl;
  transform: rotate(180deg);
}
```

- [ ] **Step 2: The store** — `useCompletionStore` (setup syntax, `StoreId.Completion`): `matrix`, `status: LoadStatus`, `error: IpcError | null`, `load()` that clears the matrix, calls `completion()`, and on failure keeps `isIpcError(e) ? e : null`.

- [ ] **Step 3: `PixelSprite` and `MarkCell`** — `PixelSprite` (props `url: string | null`, `placeholder?: boolean`): an `<img class="pixelated">` until it errors, then a `<span aria-hidden>` with `hatch-placeholder` when `placeholder`; the error resets when `url` changes; sizes come from the parent's class. `MarkCell`: a `failed` ref reset on `art` change, `symbol` is `null` when failed, `<img @error="failed = true">`.

- [ ] **Step 4: `MarksGrid`** — props `matrix`; `:style="{ '--matrix-columns': matrix.bosses.length }"`; header row (`Personaggio`, per boss a `writing-vertical` name over the hard symbol in `size-mark-symbol` via `PixelSprite`, `iniziati`), per group a band (title, first – last, `started/readable iniziati`, `N non leggibili` when any) and its rows (`PixelSprite` head `size-8` with placeholder, truncated name, a `Tooltip` per cell around a labelled `MarkCell` — content: `character · boss` and the reading's message, "valore fuori da quelli previsti: N" for unexpected, `marks.thirdLevel` appended when `third` — then the row tally), the footer "Personaggi con il marchio" with `columnTallies`; rows alternate on `bg-row-alt`, hover `bg-row-hover`; tally colour `text-state-done-foreground` complete, `text-faint-foreground` nothing readable, else `text-subtle-foreground`; the whole grid in `overflow-x-auto`.

- [ ] **Step 5: The screen parts** — `CompletionKpis` (four `KpiTile`s per the spec's table, explanations in the `explain` slot), `MatrixLegend` (five decorative `MarkCell`s: 0, 1, 3, 7, `unknown`, with column 0's art), `MarksMatrixCard` (`Card` → `CardHeader`/`CardTitle` "Matrice dei marchi", a legend strip, `CardContent` with `MarksGrid`), `CompletionScreen` (`ScreenHeader` with `Grid2x2Icon`; `ProfileError` on failure with retry; KPIs, the nothing-readable `Alert` when `readable === 0`, the card; `Skeleton`s while loading; loads on the active profile id, immediately).

- [ ] **Step 6: i18n and route** — `completion.*` in `it.ts` and `en.ts` (intro, kpi labels and explanations, `cells` unit, card title, legend, grid labels, group names, cell states, nothing-readable); `routes.ts` maps `RouteName.Completion` to `CompletionScreen`; `routeTable.ts` drops its `routeArrives` entry and both message files drop `placeholder.completion`.

- [ ] **Step 7: Verify** — `pnpm typecheck && pnpm ui:test && pnpm lint && pnpm format:check && pnpm scan` → green; then `pnpm ui:dev` and look: `?fixture=active` (art, tooltips, the unknown block bottom-right, totals 166/368, 120, 3/34, 40/408), `?fixture=active&art=none` (bars, placeholder heads), `?fixture=pick` (the gate on Completion); the Kit's MarkCell and Wiki sections unchanged.

- [ ] **Step 8: Commit** — `feat(ui): the Completion screen, the marks matrix on the active profile`

---

### Task 9: Handed on, and checked

**Files:**
- Modify: `DESIGN-BRIEF.md` (§5.6 the map lives in `ipc` now; §7 the `MarksMatrix` additions and the `started` rule), `docs/frontend-conventions.md` (`grid-cols-matrix`, `writing-vertical`, `PixelSprite`, fixtures' art and `?art=none`), `docs/BACKLOG.md` (B13 closed), `docs/STATUS.md` (3.2 ticked, session log), `CLAUDE.md` (State paragraph), the spec's "Deviations" section

- [ ] **Step 1: Documents** — as listed; the spec records every deviation met while executing.
- [ ] **Step 2: Production build** — `pnpm --filter ui build`, then check `ui/dist` holds no `fixtures`, no `isaacdome-design-pack` image and no Kit.
- [ ] **Step 3: `sh scripts/check`** → all green, skips counted and declared.
- [ ] **Step 4: Commit** — `docs: cycle 3.2 lands, the Completion screen and the marks map handed on`
