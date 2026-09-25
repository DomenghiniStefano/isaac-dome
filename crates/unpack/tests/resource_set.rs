//! The resolver: given a logical path ("gfx/items/..."), finds the resource across
//! multiple archives, trying the known roots and respecting DLC precedence.

use unpack::ResourceSet;

fn open_or_skip() -> Option<ResourceSet> {
    Some(ResourceSet::open(&test_support::packed_dir()?))
}

#[test]
fn opens_every_archive_it_finds_and_reports_them() {
    let Some(rs) = open_or_skip() else { return };
    let names: Vec<&str> = rs.archives().iter().map(|a| a.name.as_str()).collect();
    for expected in [
        "repentance.a",
        "afterbirthp.a",
        "afterbirth.a",
        "graphics.a",
        "config.a",
    ] {
        assert!(
            names.contains(&expected),
            "missing {expected} among {names:?}"
        );
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
    assert!(rs.read("gfx/does/not/exist.png").is_none());
    assert!(rs.read("").is_none());
}

/// An archive that is not there is an edition that does not ship it, and is skipped. One that
/// is there and does not open is a broken install, and is said (card #80, R6): both used to
/// be the same silence.
#[test]
fn a_broken_archive_is_reported_and_a_missing_one_is_not() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("config.a"), b"not an archive at all").unwrap();
    std::fs::write(dir.path().join("fonts.a"), b"short").unwrap();

    let rs = ResourceSet::open(dir.path());

    assert!(rs.archives().is_empty());
    let broken: Vec<(&str, &unpack::ArchiveFault)> = rs
        .broken()
        .iter()
        .map(|b| (b.name.as_str(), &b.fault))
        .collect();
    assert_eq!(
        broken,
        vec![
            ("fonts.a", &unpack::ArchiveFault::TooShort),
            ("config.a", &unpack::ArchiveFault::BadMagic),
        ]
    );
}

#[test]
fn a_folder_with_no_archives_has_nothing_broken() {
    let dir = tempfile::tempdir().unwrap();
    let rs = ResourceSet::open(dir.path());
    assert!(rs.archives().is_empty());
    assert!(rs.broken().is_empty());
}

/// The archives that open are listed in precedence order, each with the mode and the entry
/// count its own header declares, beside the ones that did not open.
#[test]
fn the_archives_that_open_are_listed_in_precedence_order_with_their_header() {
    fn archive(mode: u8, records: u16) -> Vec<u8> {
        let mut v = b"ARCH000".to_vec();
        v.push(mode);
        v.extend_from_slice(&14u32.to_le_bytes());
        v.extend_from_slice(&records.to_le_bytes());
        for i in 0..u32::from(records) {
            for w in [i, i, 0, 1, 0] {
                v.extend_from_slice(&w.to_le_bytes());
            }
        }
        v
    }
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("config.a"), archive(1, 2)).unwrap();
    std::fs::write(dir.path().join("fonts.a"), archive(1, 1)).unwrap();
    std::fs::write(dir.path().join("music.a"), b"short").unwrap();

    let rs = ResourceSet::open(dir.path());

    let listed: Vec<(&str, unpack::CompressionMode, usize)> = rs
        .archives()
        .iter()
        .map(|a| (a.name.as_str(), a.mode, a.entries))
        .collect();
    assert_eq!(
        listed,
        vec![
            ("fonts.a", unpack::CompressionMode::Lzw, 1),
            ("config.a", unpack::CompressionMode::Lzw, 2),
        ]
    );
    assert_eq!(rs.broken().len(), 1);
    assert_eq!(rs.broken()[0].name, "music.a");
}
