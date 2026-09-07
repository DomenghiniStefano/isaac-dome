//! The resolver: given a logical path ("gfx/items/..."), finds the resource across
//! multiple archives, trying the known roots and respecting DLC precedence.

use unpack::ResourceSet;

fn open_or_skip() -> Option<ResourceSet> {
    Some(ResourceSet::open(&test_support::packed_dir()?))
}

#[test]
fn opens_every_archive_it_finds_and_reports_them() {
    let Some(rs) = open_or_skip() else { return };
    let nomi: Vec<&str> = rs.archives().iter().map(|a| a.name.as_str()).collect();
    for atteso in [
        "repentance.a",
        "afterbirthp.a",
        "afterbirth.a",
        "graphics.a",
        "config.a",
    ] {
        assert!(nomi.contains(&atteso), "missing {atteso} among {nomi:?}");
    }
    // Every archive declares how many entries it has and in what mode they're encoded.
    let tot: usize = rs.archives().iter().map(|a| a.entries).sum();
    assert!(tot > 15_000, "unexpected total entries: {tot}");
}

#[test]
fn resolves_a_path_without_the_caller_knowing_the_root() {
    let Some(rs) = open_or_skip() else { return };
    // The caller names neither the archive nor the root.
    let png = rs
        .read("gfx/items/collectibles/collectibles_001_thesadonion.png")
        .expect("a base-game item");
    assert_eq!(&png[0..4], &[0x89, b'P', b'N', b'G']);

    let png = rs
        .read("gfx/items/collectibles/collectibles_555_goldenrazor.png")
        .expect("an item introduced by Repentance");
    assert_eq!(&png[0..4], &[0x89, b'P', b'N', b'G']);
}

#[test]
fn the_newest_dlc_wins() {
    let Some(rs) = open_or_skip() else { return };
    // items.xml exists in multiple archives: the most recent DLC's copy must win,
    // which is also the largest one because it contains more items.
    let (bytes, da) = rs
        .read_with_source("items.xml")
        .expect("items.xml exists in multiple archives");
    assert!(
        bytes.len() > 100_000,
        "the wrong archive won: {} bytes from {da}",
        bytes.len()
    );
}

#[test]
fn a_missing_resource_is_none_not_a_panic() {
    let Some(rs) = open_or_skip() else { return };
    assert!(rs.read("gfx/non/esiste/proprio.png").is_none());
    assert!(rs.read("").is_none());
}
