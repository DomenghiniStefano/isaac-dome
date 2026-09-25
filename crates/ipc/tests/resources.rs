//! The view-model of "what we managed to extract".

use ipc::{
    archive_views, broken_archive_views, data_url, extraction_report, wiki_info, ArchiveMode,
    ArchiveReason, IoReason,
};
use unpack::{ArchiveFault, ArchiveInfo, BrokenArchive, CompressionMode};

fn info(name: &str, mode: CompressionMode, entries: usize) -> ArchiveInfo {
    ArchiveInfo {
        name: name.to_string(),
        mode,
        entries,
    }
}

#[test]
fn archive_modes_become_our_own_tagged_enum() {
    // The mode comes from a domain crate and has a newtype variant, a shape forbidden
    // past the IPC boundary: it must be remapped onto an enum of our own with struct
    // variants.
    let views = archive_views(&[
        info("config.a", CompressionMode::Lzw, 24),
        info("afterbirthp.a", CompressionMode::MiniZ, 10228),
        info("strano.a", CompressionMode::Unknown(9), 3),
    ]);
    assert_eq!(views[0].mode, ArchiveMode::Lzw);
    assert_eq!(views[1].mode, ArchiveMode::MiniZ);
    assert_eq!(views[2].mode, ArchiveMode::Unknown { value: 9 });
    assert_eq!(views[1].entries, 10228);
}

#[test]
fn archive_view_json_shape_is_pinned() {
    let views = archive_views(&[info("strano.a", CompressionMode::Unknown(9), 3)]);
    let v = serde_json::to_value(&views[0]).expect("serializes");
    assert_eq!(v["name"], "strano.a");
    assert_eq!(v["entries"], 3);
    // Tagged struct variant, fields in camelCase: never a newtype.
    assert_eq!(v["mode"]["kind"], "unknown");
    assert_eq!(v["mode"]["value"], 9);

    // The other labels need pinning: the TypeScript side writes them by hand, and a
    // typo doesn't produce an error, it produces a branch that never fires.
    let etichetta = |m: CompressionMode| {
        serde_json::to_value(&archive_views(&[info("x.a", m, 0)])[0]).unwrap()["mode"]["kind"]
            .as_str()
            .unwrap()
            .to_string()
    };
    assert_eq!(etichetta(CompressionMode::Lzw), "lzw");
    assert_eq!(etichetta(CompressionMode::MiniZ), "miniZ");
    assert_eq!(etichetta(CompressionMode::Bogocrypt1), "bogocrypt1");
    assert_eq!(etichetta(CompressionMode::Bogocrypt2), "bogocrypt2");
}

#[test]
fn data_url_wraps_the_bytes_for_an_img_tag() {
    // Minimal PNG: the first bytes are enough, only the encoding matters here.
    let url = data_url(&[0x89, b'P', b'N', b'G']);
    assert!(url.starts_with("data:image/png;base64,"));
    assert_eq!(url, "data:image/png;base64,iVBORw==");
}

#[test]
fn base64_matches_the_known_vectors() {
    // The classic RFC 4648 cases: they cover the three possible paddings.
    let url = |s: &str| data_url(s.as_bytes()).replace("data:image/png;base64,", "");
    assert_eq!(url("Man"), "TWFu");
    assert_eq!(url("Ma"), "TWE=");
    assert_eq!(url("M"), "TQ==");
    assert_eq!(url(""), "");
    assert_eq!(url("any carnal pleasure."), "YW55IGNhcm5hbCBwbGVhc3VyZS4=");
    assert_eq!(url("any carnal pleasure"), "YW55IGNhcm5hbCBwbGVhc3VyZQ==");
    // High bytes: the alphabet must reach all the way to '/' and '+'.
    assert_eq!(
        data_url(&[0xfb, 0xff, 0xfe]).replace("data:image/png;base64,", ""),
        "+//+"
    );
}

fn broken(name: &str, fault: ArchiveFault) -> BrokenArchive {
    BrokenArchive {
        name: name.to_string(),
        fault,
    }
}

#[test]
fn a_broken_archive_says_which_case_it_is() {
    // Card #80, R6: an archive that is there and does not open reaches the report, with the
    // same shape a save that does not open already has (`SaveReason`).
    let views = broken_archive_views(&[
        broken("repentance.a", ArchiveFault::TooShort),
        broken("graphics.a", ArchiveFault::BadMagic),
        broken(
            "music.a",
            ArchiveFault::Io {
                kind: std::io::ErrorKind::PermissionDenied,
            },
        ),
        broken(
            "fonts.a",
            ArchiveFault::Io {
                kind: std::io::ErrorKind::UnexpectedEof,
            },
        ),
    ]);
    assert_eq!(
        views.iter().map(|v| v.name.as_str()).collect::<Vec<_>>(),
        ["repentance.a", "graphics.a", "music.a", "fonts.a"]
    );
    assert_eq!(views[0].reason, ArchiveReason::TooShort);
    assert_eq!(views[1].reason, ArchiveReason::BadMagic);
    assert_eq!(
        views[2].reason,
        ArchiveReason::Io {
            reason: IoReason::PermissionDenied
        }
    );
    assert_eq!(
        views[3].reason,
        ArchiveReason::Io {
            reason: IoReason::Other
        }
    );
}

#[test]
fn broken_archive_json_shape_is_pinned() {
    let views = broken_archive_views(&[broken(
        "music.a",
        ArchiveFault::Io {
            kind: std::io::ErrorKind::PermissionDenied,
        },
    )]);
    let v = serde_json::to_value(&views[0]).expect("serializes");
    assert_eq!(v["name"], "music.a");
    assert_eq!(v["reason"]["kind"], "io");
    assert_eq!(v["reason"]["reason"], "permissionDenied");
    let kind = |f: ArchiveFault| {
        serde_json::to_value(&broken_archive_views(&[broken("x.a", f)])[0]).unwrap()["reason"]
            ["kind"]
            .as_str()
            .unwrap()
            .to_string()
    };
    assert_eq!(kind(ArchiveFault::TooShort), "tooShort");
    assert_eq!(kind(ArchiveFault::BadMagic), "badMagic");
}

#[test]
fn the_report_carries_the_broken_archives_next_to_the_open_ones() {
    let report = extraction_report(
        archive_views(&[info("config.a", CompressionMode::Lzw, 24)]),
        broken_archive_views(&[broken("repentance.a", ArchiveFault::BadMagic)]),
        None,
        Vec::new(),
        wiki_info(wiki::Dataset::embedded(), None),
    );
    let v = serde_json::to_value(&report).expect("serializes");
    assert_eq!(v["archives"].as_array().map(Vec::len), Some(1));
    assert_eq!(v["broken"][0]["name"], "repentance.a");
    assert_eq!(v["broken"][0]["reason"]["kind"], "badMagic");
    // A broken archive indexes nothing: the total counts the open ones only.
    assert_eq!(v["totalEntries"], 24);
}
