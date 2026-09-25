//! Bytes arrive in chunks that do not respect line endings.

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
fn wherever_a_read_ends_the_lines_that_come_out_are_the_same() {
    // A property over every split point, not a pinned value: a read ends wherever 256 KiB ends,
    // and the lines must not depend on where that was.
    let log = b"[INFO] - first\r\nsecond\n\n[INFO] - fourth\n";
    let whole: Vec<String> = Tail::default().advance(log);
    assert_eq!(whole.len(), 4, "the property below needs lines to compare");
    for split in 0..=log.len() {
        let mut tail = Tail::default();
        let mut lines = tail.advance(&log[..split]);
        lines.extend(tail.advance(&log[split..]));
        assert_eq!(lines, whole, "split at byte {split}");
        assert_eq!(tail.pending(), 0, "split at byte {split}");
    }
}

#[test]
fn an_empty_line_is_a_line_and_not_skipped() {
    // Dropping a blank line would be a judgment about what it means, and every
    // judgment belongs to the fold.
    let mut tail = Tail::default();
    assert_eq!(tail.advance(b"a\n\nb\n"), vec!["a", "", "b"]);
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
