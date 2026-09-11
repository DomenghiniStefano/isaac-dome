//! The marks map against the installed game: both anm2 files read, every column resolves
//! to two tiers, and each crop is a real 16×16 piece of a sheet that exists in the archives.
//! The synthetic `mark_art.rs` pins the rules; this says whether the game still agrees.

use ipc::{
    crop_png, decode_rgba, mark_source, MarkFrames, MarkTier, BOSSES, LOBBY_ANM2, WIDGET_ANM2,
};

#[test]
fn every_column_crops_two_real_symbols() {
    let Some(dir) = test_support::packed_dir() else {
        return;
    };
    let rs = unpack::ResourceSet::open(&dir);
    let read = |p: &str| {
        rs.read(p)
            .and_then(|b| catalog::anm2_frames(&b))
            .unwrap_or_default()
    };
    let frames = MarkFrames {
        widget: read(WIDGET_ANM2),
        lobby: read(LOBBY_ANM2),
    };
    for (column, boss) in BOSSES.iter().enumerate() {
        for tier in [MarkTier::Normal, MarkTier::Hard] {
            let sprite = mark_source(column, tier, &frames)
                .unwrap_or_else(|| panic!("{boss} {tier:?}: no frame"));
            let sheet = rs
                .read(&sprite.path)
                .unwrap_or_else(|| panic!("{boss}: sheet {} not in the archives", sprite.path));
            let r = sprite.rect.expect("a mark is a piece of a sheet");
            let png = crop_png(&sheet, r.x, r.y, r.w, r.h)
                .unwrap_or_else(|| panic!("{boss} {tier:?}: the rectangle is outside the sheet"));
            let (w, h, _) = decode_rgba(&png).expect("the crop is a PNG");
            assert_eq!((w, h), (16, 16), "{boss} {tier:?}");
        }
    }
}
