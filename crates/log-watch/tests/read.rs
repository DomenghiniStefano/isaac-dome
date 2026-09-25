//! Positional reads. The log is the user's file and its size is their choice, so nothing here
//! ever asks for all of it.

use std::fs;

#[test]
fn the_head_stops_at_what_was_asked_for() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("log.txt");
    fs::write(&p, b"0123456789").unwrap();
    assert_eq!(log_watch::head(&p, 4).unwrap(), b"0123");
}

#[test]
fn the_head_of_a_file_shorter_than_the_window_is_the_whole_file() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("log.txt");
    fs::write(&p, b"abc").unwrap();
    assert_eq!(log_watch::head(&p, 4096).unwrap(), b"abc");
}

#[test]
fn the_window_ends_at_the_offset_it_was_given() {
    // The anchor of `run::resume`: the bytes just before where we stopped reading.
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("log.txt");
    fs::write(&p, b"0123456789").unwrap();
    assert_eq!(log_watch::window_ending_at(&p, 7, 3).unwrap(), b"456");
}

#[test]
fn a_window_wider_than_the_offset_starts_at_the_beginning() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("log.txt");
    fs::write(&p, b"0123456789").unwrap();
    assert_eq!(log_watch::window_ending_at(&p, 3, 64).unwrap(), b"012");
}

#[test]
fn a_chunk_reads_from_the_offset_and_no_further_than_asked() {
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("log.txt");
    fs::write(&p, b"0123456789").unwrap();
    assert_eq!(log_watch::chunk(&p, 4, 3).unwrap(), b"456");
    assert_eq!(log_watch::chunk(&p, 8, 100).unwrap(), b"89");
    assert!(log_watch::chunk(&p, 10, 100).unwrap().is_empty());
}

#[test]
fn a_file_that_is_not_there_is_an_error_with_a_kind_and_not_a_message() {
    // The rule the whole repo follows: `io::ErrorKind`, never the OS's sentence, which is not
    // translatable and repeats the path on some platforms.
    let tmp = tempfile::tempdir().unwrap();
    match log_watch::len(&tmp.path().join("nothing.txt")) {
        Err(log_watch::WatchError::Io { kind }) => {
            assert_eq!(kind, std::io::ErrorKind::NotFound);
        }
        other => panic!("expected Io, got {other:?}"),
    }
}

#[test]
fn a_chunk_larger_than_one_read_of_the_os_comes_back_whole() {
    // One `read` call may return fewer bytes than asked; a chunk is every byte up to `max`,
    // however many calls that takes. `CHUNK` is the size the ingest asks for.
    let tmp = tempfile::tempdir().unwrap();
    let p = tmp.path().join("log.txt");
    let body: Vec<u8> = (0..(log_watch::CHUNK * 2 + 17))
        .map(|i| (i % 251) as u8)
        .collect();
    fs::write(&p, &body).unwrap();
    let from = 11;
    assert_eq!(
        log_watch::chunk(&p, from as u64, log_watch::CHUNK).unwrap(),
        &body[from..from + log_watch::CHUNK]
    );
    assert_eq!(
        log_watch::chunk(&p, (body.len() - 5) as u64, log_watch::CHUNK).unwrap(),
        &body[body.len() - 5..]
    );
}
