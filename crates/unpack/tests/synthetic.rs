use unpack::{path_key, Archive, OpenError};

/// Builds a minimal ARCH000: header + records, dummy data.
/// records: (djb2, fnv, offset, decompressed_len, checksum).
fn build(records: &[(u32, u32, u32, u32, u32)], data: &[u8]) -> Vec<u8> {
    let index_off = 14 + data.len();
    let mut v = Vec::new();
    v.extend_from_slice(b"ARCH000");
    v.push(0x01);
    v.extend_from_slice(&(index_off as u32).to_le_bytes());
    v.extend_from_slice(&(records.len() as u16).to_le_bytes());
    v.extend_from_slice(data);
    for &(d, f, o, dl, c) in records {
        for w in [d, f, o, dl, c] {
            v.extend_from_slice(&w.to_le_bytes());
        }
    }
    v
}

#[test]
fn opens_and_reads_index() {
    // Two records; the actual data isn't needed for the index.
    let bytes = build(
        &[(111, 222, 14, 10, 999), (333, 444, 24, 5, 888)],
        &[0u8; 20],
    );
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), &bytes).unwrap();

    let a = Archive::open(tmp.path()).unwrap();
    assert_eq!(a.entries().len(), 2);
    assert_eq!(a.entries()[0].key.djb2, 111);
    assert_eq!(a.entries()[0].offset, 14);
    assert_eq!(a.entries()[1].decompressed_len, 5);
}

#[test]
fn rejects_bad_magic_and_short_files() {
    let mut bytes = build(&[(1, 2, 14, 1, 0)], &[0u8; 4]);
    bytes[0] = b'X';
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), &bytes).unwrap();
    assert!(matches!(
        Archive::open(tmp.path()),
        Err(OpenError::BadMagic { .. })
    ));

    let tmp2 = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp2.path(), b"ARCH0").unwrap();
    assert!(matches!(
        Archive::open(tmp2.path()),
        Err(OpenError::TooShort)
    ));
}

#[test]
fn skips_index_entry_pointing_out_of_bounds() {
    // A record with an offset past the end of the file is discarded, no panic.
    let bytes = build(
        &[(111, 222, 14, 10, 0), (333, 444, 9_000_000, 5, 0)],
        &[0u8; 20],
    );
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), &bytes).unwrap();
    let a = Archive::open(tmp.path()).unwrap();
    assert_eq!(
        a.entries().len(),
        1,
        "the out-of-bounds record is discarded"
    );
    assert_eq!(a.entries()[0].key.djb2, 111);
}

// ── FIX 2: no-panic tests on corrupt LZW input/archive ─────────────────────

#[test]
fn lzw_garbage_bytes_returns_none() {
    // Random/garbage byte stream: must produce no output and must not panic.
    let garbage = [0xDE, 0xAD, 0xBE, 0xEF, 0x13, 0x37, 0xCA, 0xFE, 0x00, 0xFF];
    assert!(
        unpack::for_tests::lzw_decompress(&garbage, 0, 100).is_none(),
        "garbage must not produce output"
    );
}

#[test]
fn lzw_chunk_len_overflows_slice_returns_none() {
    // Huge u32 in the chunkLen field, followed by too few bytes: out-of-bounds → None, no panic.
    let mut data = Vec::new();
    data.extend_from_slice(&u32::MAX.to_le_bytes()); // huge chunk_len
    data.extend_from_slice(&[0xAA, 0xBB, 0xCC]); // insufficient data
    assert!(
        unpack::for_tests::lzw_decompress(&data, 0, 16).is_none(),
        "a chunk_len that overruns the end of the buffer must give None"
    );
}

#[test]
fn lzw_truncated_data_returns_none() {
    // decompressed_len much larger than the available bytes: None, no panic.
    let tiny = [0x00u8; 8]; // too short to produce megabytes
    assert!(
        unpack::for_tests::lzw_decompress(&tiny, 0, 1_000_000).is_none(),
        "truncated data must give None"
    );
}

#[test]
fn archive_read_with_invalid_lzw_data_returns_none() {
    // A synthetic archive where an entry has known hashes but invalid non-LZW data.
    // Archive::read must return None, never panic.
    let test_path = "resources/prova.xml";
    let k = path_key(test_path);
    // Offset 14 (right after the header), dummy data that doesn't decompress.
    let bad_data = [0xFFu8; 32];
    let bytes = build(&[(k.djb2, k.fnv, 14, 100, 0)], &bad_data);
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), &bytes).unwrap();
    let a = Archive::open(tmp.path()).unwrap();
    // contains must find it (correct hashes), read must return None (invalid LZW).
    assert!(
        a.contains(test_path),
        "the entry must be present in the index"
    );
    assert!(
        a.read(test_path).is_none(),
        "invalid LZW data must give None, not panic"
    );
}
