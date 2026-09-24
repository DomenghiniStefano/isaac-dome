//! Bytes arrive in chunks that do not respect line endings, and the game relaunches.

use run::Tail;

#[test]
fn a_whole_line_comes_out_whole() {
    let mut tail = Tail::default();
    assert_eq!(tail.advance(b"first\nsecond\n"), vec!["first", "second"]);
}

#[test]
fn a_half_written_line_waits_for_its_rest() {
    let mut tail = Tail::default();
    assert_eq!(tail.advance(b"complete\npar"), vec!["complete"]);
    assert_eq!(tail.advance(b"tial\n"), vec!["partial"]);
}

#[test]
fn a_windows_line_ending_is_not_part_of_the_line() {
    // The game writes CRLF; a `\r` left on the end makes every regex anchored at `$` fail.
    let mut tail = Tail::default();
    assert_eq!(tail.advance(b"line\r\n"), vec!["line"]);
}

#[test]
fn a_chunk_with_no_newline_yields_nothing_and_loses_nothing() {
    let mut tail = Tail::default();
    assert!(tail.advance(b"still writing").is_empty());
    assert_eq!(tail.advance(b"\n"), vec!["still writing"]);
}

#[test]
fn a_shorter_file_is_a_new_launch() {
    let mut tail = Tail::default();
    tail.advance(b"aaaa\nbbbb\n");
    assert!(
        tail.restarted(4),
        "10 bytes read, file is 4: the game relaunched"
    );
    assert!(!tail.restarted(10), "same length: the same launch");
    assert!(!tail.restarted(99), "longer: it just grew");
}

#[test]
fn a_relaunch_drops_the_remainder_of_the_old_file() {
    // Otherwise the first line of the new log arrives glued to half a line of the old one.
    let mut tail = Tail::default();
    tail.advance(b"old half");
    tail.restart();
    assert_eq!(tail.advance(b"[INFO] - new\n"), vec!["[INFO] - new"]);
}

#[test]
fn a_half_written_line_is_counted_as_not_yet_read() {
    // What the watcher stores as its offset is the end of the last **complete** line: a read
    // that stopped mid-line must not leave that line behind for ever.
    let mut tail = run::Tail::default();
    let lines = tail.advance(b"first\nsecond\nthi");
    assert_eq!(lines, vec!["first", "second"]);
    assert_eq!(tail.pending(), 3);
    tail.advance(b"rd\n");
    assert_eq!(tail.pending(), 0);
}

/// Card #80, P7: a read ends wherever 256 KiB ends, and that can be inside a character. Each
/// chunk used to be decoded on its own, so an "é" split across two reads became two U+FFFD;
/// and the bytes held back were counted after decoding, so the offset the watcher stores moved
/// by the difference.
#[test]
fn a_character_split_across_two_reads_comes_out_whole() {
    let mut t = run::Tail::default();
    assert!(t.advance(b"Adding collectible 1 (Caf\xC3").is_empty());
    let lines = t.advance(b"\xA9)\n");
    assert_eq!(lines, vec!["Adding collectible 1 (Café)".to_string()]);
}

#[test]
fn the_bytes_held_back_are_bytes_of_the_file() {
    let mut t = run::Tail::default();
    // A finished line, then three raw bytes of an unfinished one ending mid-character.
    t.advance(b"done\nab\xC3");
    assert_eq!(
        t.pending(),
        3,
        "what was read and not yet returned, as the file counts it"
    );
}

#[test]
fn a_byte_that_is_not_utf8_still_costs_only_its_own_line() {
    let mut t = run::Tail::default();
    let lines = t.advance(b"bad \xFF byte\ngood line\n");
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[1], "good line");
}
