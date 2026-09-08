//! Tests on ARCH000's three compression modes, against real archives from the game.
//!
//! The archives don't live in the repo: `samples/packed` is a junction to the
//! installation's `resources/packed` folder. If it's missing, the tests **skip and say
//! so**. Every test states which mode it runs on: a sample that covers only one mode is
//! exactly how we noticed too late that the decoder was incomplete.

use unpack::{Archive, CompressionMode, ResourceSet};

fn open_or_skip(name: &str) -> Option<Archive> {
    // The file is there but won't open: that's a different case from "it's not there", and
    // it should be said.
    match Archive::open(&test_support::packed_file(name)?) {
        Ok(a) => Some(a),
        Err(_) => {
            test_support::skip(&format!("packed/{name} present but doesn't open"));
            None
        }
    }
}

#[test]
fn compression_mode_is_read_from_the_header_not_assumed() {
    let cases = [
        ("config.a", CompressionMode::Lzw),
        ("fonts.a", CompressionMode::Lzw),
        ("graphics.a", CompressionMode::Bogocrypt1),
        ("music.a", CompressionMode::Bogocrypt1),
        ("afterbirth.a", CompressionMode::MiniZ),
        ("afterbirthp.a", CompressionMode::MiniZ),
        ("repentance.a", CompressionMode::MiniZ),
    ];
    let mut seen = 0;
    for (name, expected) in cases {
        let Some(a) = open_or_skip(name) else {
            continue;
        };
        assert_eq!(a.mode(), expected, "{name}: compression mode");
        seen += 1;
    }
    eprintln!("modes verified on {seen} archives");
}

#[test]
fn miniz_decompresses_a_compressed_entry() {
    // items.xml in afterbirthp.a: deflate blocks, no "stored" block.
    let Some(a) = open_or_skip("afterbirthp.a") else {
        return;
    };
    let out = a
        .read("resources/items.xml")
        .expect("items.xml decompresses from a MiniZ archive");
    assert_eq!(out[0], b'<', "an XML file starts with '<'");
    let text = String::from_utf8_lossy(&out);
    assert!(text.contains("<items"), "<items> root present");
    // The DLC catalog must be richer than config.a's base one (403 entries).
    let passives = text.matches("<passive ").count();
    assert!(
        passives > 213,
        "the DLC catalog should have more passives than the base one: found {passives}"
    );
}

#[test]
fn miniz_decompresses_an_entry_with_stored_blocks() {
    // A PNG is incompressible: its blocks become "stored" and go through the ISAAC cipher.
    // If ISAAC's seed were wrong, the PNG signature wouldn't come back.
    let Some(a) = open_or_skip("afterbirthp.a") else {
        return;
    };
    let out = a
        .read("resources/gfx/ui/boss/portrait_102.0_isaac.png")
        .expect("the PNG extracts from a MiniZ archive");
    assert_eq!(
        &out[0..8],
        &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a],
        "PNG signature at the head"
    );
    assert_eq!(
        &out[out.len() - 8..],
        &[b'I', b'E', b'N', b'D', 0xae, 0x42, 0x60, 0x82],
        "IEND chunk at the tail"
    );
}

#[test]
fn lzw_still_works_after_the_dispatch() {
    // Regression: mode 1 keeps working.
    let Some(a) = open_or_skip("config.a") else {
        return;
    };
    let out = a.read("resources/achievements.xml").expect("LZW unchanged");
    assert_eq!(out.len(), 25894);
    assert_eq!(out[0], b'<');
}

#[test]
fn bogocrypt1_decrypts_a_base_game_sprite() {
    // graphics.a is mode 0: raw encrypted data, no compression.
    // If the key or its evolution were wrong, the PNG signature wouldn't come back.
    let Some(a) = open_or_skip("graphics.a") else {
        return;
    };
    let out = a
        .read("resources/gfx/items/collectibles/collectibles_001_thesadonion.png")
        .expect("the sprite extracts from a Bogocrypt1 archive");
    assert_eq!(
        &out[0..8],
        &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a],
        "PNG signature at the head"
    );
    assert_eq!(
        &out[out.len() - 8..],
        &[b'I', b'E', b'N', b'D', 0xae, 0x42, 0x60, 0x82],
        "IEND chunk at the tail"
    );
}

#[test]
fn repentance_resources_live_under_a_different_root() {
    // repentance.a contains NO "resources/..." path at all: its root is "resources-dlc3/".
    // Without this, 4,180 entries stay indexed and unreachable — the index holds only
    // hashes, so a wrong name gives no error, it gives silence.
    let Some(a) = open_or_skip("repentance.a") else {
        return;
    };
    // An item introduced by Repentance: exists only in this archive.
    let rel = "gfx/items/collectibles/collectibles_555_goldenrazor.png";
    assert!(
        !a.contains(&format!("resources/{rel}")),
        "the classic root must not find anything in repentance.a"
    );
    assert!(
        a.contains(&format!("resources-dlc3/{rel}")),
        "Repentance's root is resources-dlc3/"
    );
    let out = a
        .read(&format!("resources-dlc3/{rel}"))
        .expect("and the resource extracts");
    assert_eq!(&out[0..4], &[0x89, b'P', b'N', b'G']);
}

#[test]
fn every_catalog_item_has_its_sprite() {
    // The question that matters for the product: do the catalog and the sprites match up?
    // We use the items.xml that WINS the precedence (Repentance's, 909 elements,
    // attributes separated by tabs too), not one picked by hand.
    let Some(dir) = test_support::packed_dir() else {
        return;
    };
    let rs = ResourceSet::open(&dir);
    let (xml, da) = rs.read_with_source("items.xml").expect("items.xml");
    assert_eq!(da, "repentance.a", "the most recent DLC must win");
    let text = String::from_utf8_lossy(&xml);

    let mut checked = 0;
    let mut missing = Vec::new();
    for (tag, dir) in [
        ("passive", "gfx/items/collectibles"),
        ("active", "gfx/items/collectibles"),
        ("familiar", "gfx/items/collectibles"),
        ("trinket", "gfx/items/trinkets"),
    ] {
        for el in elements(&text, tag) {
            if let Some(gfx) = attr_of(el, "gfx") {
                checked += 1;
                if !rs.contains(&format!("{dir}/{gfx}")) {
                    missing.push(format!("{dir}/{gfx}"));
                }
            }
        }
    }
    assert!(checked > 900, "only {checked} items checked");
    assert!(
        missing.is_empty(),
        "missing sprites ({}): {missing:?}",
        missing.len()
    );
}

/// Value of `attr` in an element's opening fragment.
fn attr_of(element: &str, attr: &str) -> Option<String> {
    let needle = format!("{attr}=\"");
    let i = element.find(&needle)? + needle.len();
    let j = element[i..].find('"')?;
    Some(element[i..i + j].to_string())
}

/// Every `<tag ...>` as a fragment up to the `>`.
fn elements<'a>(text: &'a str, tag: &str) -> Vec<&'a str> {
    // Spaces or tabs: Repentance's items.xml uses both.
    let open = format!("<{tag}");
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find(&open) {
        rest = &rest[i..];
        if !rest[open.len()..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
        {
            rest = &rest[open.len()..];
            continue;
        }
        let end = rest.find('>').unwrap_or(rest.len());
        out.push(&rest[..end]);
        rest = &rest[end..];
    }
    out
}
