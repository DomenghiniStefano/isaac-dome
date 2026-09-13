//! The one decision `log-watch` is not allowed to make: is this the launch we were reading,
//! or a new one? Pure, so it is tested without a file.

use run::{fingerprint, resume, Resume, SourceKey, ANCHOR_BYTES, PREFIX_BYTES};

/// The banner a real log opens with is the machine describing itself: measured on 2026-09-13,
/// two different launches on this machine share their first 1,632 bytes exactly.
fn banner() -> Vec<u8> {
    let mut b = Vec::new();
    while b.len() < PREFIX_BYTES {
        b.extend_from_slice(b"[INFO] - OpenGL version 4.6.0 NVIDIA 610.88\n");
    }
    b.truncate(PREFIX_BYTES);
    b
}

fn window(text: &str) -> Vec<u8> {
    let mut w = text.as_bytes().to_vec();
    w.resize(ANCHOR_BYTES, b' ');
    w
}

#[test]
fn the_fingerprint_is_fnv1a_and_not_the_hasher_of_the_day() {
    // The published FNV-1a 64 vectors, not values read off our own output: the empty string is
    // the offset basis, and the other two are in the reference implementation.
    assert_eq!(fingerprint(b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(fingerprint(b"a"), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(fingerprint(b"foobar"), 0x8594_4171_f739_67e8);
}

#[test]
fn a_file_that_grew_from_where_we_stopped_is_the_same_launch() {
    let consumed = window("Level::Init m_Stage 2");
    let stored = SourceKey::new(&banner(), &consumed, 40_000);
    assert_eq!(
        resume(&stored, &banner(), 90_000, &consumed),
        Resume::Continue { offset: 40_000 }
    );
}

#[test]
fn the_same_banner_over_different_bytes_at_the_offset_is_a_new_launch() {
    // The case the 4 KiB prefix cannot see, and the reason the anchor exists: two launches on
    // one machine open with the same bytes.
    let stored = SourceKey::new(&banner(), &window("Level::Init m_Stage 2"), 40_000);
    assert_eq!(
        resume(&stored, &banner(), 90_000, &window("Level::Init m_Stage 7")),
        Resume::Fresh
    );
}

#[test]
fn a_file_shorter_than_what_we_read_is_a_new_launch() {
    // `Tail::restarted`'s reasoning, applied before a single byte is read: the game rewrites
    // log.txt from scratch on every launch.
    let consumed = window("Level::Init m_Stage 2");
    let stored = SourceKey::new(&banner(), &consumed, 40_000);
    assert_eq!(resume(&stored, &banner(), 900, &consumed), Resume::Fresh);
}

#[test]
fn another_machines_banner_is_a_new_launch() {
    let consumed = window("Level::Init m_Stage 2");
    let stored = SourceKey::new(&banner(), &consumed, 40_000);
    let mut other = banner();
    other[0] = b'x';
    assert_eq!(resume(&stored, &other, 90_000, &consumed), Resume::Fresh);
}

#[test]
fn a_file_exactly_as_long_as_what_we_read_is_the_same_launch_with_nothing_new() {
    // The ordinary case between two writes: not a relaunch, and not a reason to re-import.
    let consumed = window("Level::Init m_Stage 2");
    let stored = SourceKey::new(&banner(), &consumed, 40_000);
    assert_eq!(
        resume(&stored, &banner(), 40_000, &consumed),
        Resume::Continue { offset: 40_000 }
    );
}

#[test]
fn a_source_read_from_zero_has_an_empty_anchor_and_still_resumes() {
    // A log seen but not yet read: offset 0, so the anchor window is empty. It must not be a
    // special case at the call site.
    let stored = SourceKey::new(&banner(), &[], 0);
    assert_eq!(
        resume(&stored, &banner(), 5_000, &[]),
        Resume::Continue { offset: 0 }
    );
}

#[test]
fn a_log_still_shorter_than_the_prefix_window_is_the_same_launch_when_it_grows() {
    // Found by a test on 2026-09-13, and it is the failure that duplicates runs. A log is a few
    // hundred bytes for its first instants, so "the first 4 KiB" of it is the whole file: read
    // that window again once the game has written more and the hash is a different number, the
    // source reads as new, and everything already in the archive is imported a second time.
    // The window is stored with its length, so it stays the window it was.
    let early = b"[INFO] - OpenGL version 4.6.0\n[INFO] - RNG Start Seed: FYQ8 QQ8G (1) [New, 1]\n";
    let stored = SourceKey::new(early, &window("Seed 408474304"), 80);

    // Later the same file is far longer, but the first 80 bytes are the bytes they always were.
    assert_eq!(
        resume(&stored, early, 900_000, &window("Seed 408474304")),
        Resume::Continue { offset: 80 }
    );
}

#[test]
fn a_prefix_read_over_a_different_number_of_bytes_is_not_the_same_prefix() {
    // The guard that makes the rule above hold: a hash over 80 bytes and a hash over 4,096 are
    // not comparable, and comparing them is how the bug got in.
    let early = b"[INFO] - OpenGL version 4.6.0\n";
    let stored = SourceKey::new(early, &[], 0);
    let mut longer = early.to_vec();
    longer.extend_from_slice(b"[INFO] - and more of it\n");
    assert_eq!(resume(&stored, &longer, 900_000, &[]), Resume::Fresh);
}
