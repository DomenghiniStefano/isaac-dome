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
