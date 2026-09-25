use core_save::{Diagnostic, Kind, Save};

/// Assembles a synthetic `.dat`. Each section: (kind, declared count, raw data).
/// `declared_size` is written as count*4, as in the real format.
fn build(unknown: u32, sections: &[(u32, u32, &[u8])]) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(b"ISAACNGSAVE09R  ");
    v.extend_from_slice(&unknown.to_le_bytes());
    for &(kind, count, data) in sections {
        v.extend_from_slice(&kind.to_le_bytes());
        v.extend_from_slice(&count.wrapping_mul(4).to_le_bytes());
        v.extend_from_slice(&count.to_le_bytes());
        v.extend_from_slice(data);
    }
    v.extend_from_slice(&[0u8; 4]); // dummy checksum
    v
}

#[test]
fn parses_well_formed_sections() {
    // kind 1: 3 achievements (1 byte each); kind 2: 2 counters (4 bytes each).
    let counters: Vec<u8> = [7u32, 42u32].iter().flat_map(|v| v.to_le_bytes()).collect();
    let bytes = build(0, &[(1, 3, &[1, 0, 1]), (2, 2, &counters)]);

    let save = Save::parse(&bytes).unwrap();
    assert_eq!(save.sections.len(), 2);
    assert_eq!(save.sections[0].kind, Kind::Achievements);
    assert_eq!(save.sections[0].count, 3);
    assert_eq!(save.sections[0].bytes, vec![1, 0, 1]);
    assert_eq!(save.sections[1].kind, Kind::Counters);
    assert_eq!(save.sections[1].count, 2);
    assert!(save.diagnostics.is_empty(), "expected no diagnostics");
}

#[test]
fn bestiary_extends_to_end_ignoring_header_count() {
    // kind 10 with a bogus count (80) but only 16 bytes of real data (2 records of 8).
    let data = [0u8; 16];
    let bytes = build(0, &[(10, 80, &data)]);

    let save = Save::parse(&bytes).unwrap();
    assert_eq!(save.sections.len(), 1);
    let bestiary = &save.sections[0];
    assert_eq!(bestiary.kind, Kind::Bestiary);
    assert_eq!(bestiary.count, 80); // header preserved...
    assert_eq!(bestiary.bytes.len(), 16); // ...but length comes from the bytes, not from count
    assert!(save.diagnostics.is_empty());
}

#[test]
fn flags_a_section_that_overruns_the_buffer() {
    // kind 1 declares 100 entries of 1 byte, but supplies only 3.
    let bytes = build(0, &[(1, 100, &[0, 0, 0])]);

    let save = Save::parse(&bytes).unwrap();
    // The section is present, truncated to the available bytes.
    assert_eq!(save.sections.len(), 1);
    assert_eq!(save.sections[0].bytes.len(), 3);
    assert!(
        save.diagnostics.iter().any(|d| matches!(
            d,
            Diagnostic::SectionOverrun {
                kind: 1,
                needed: 100,
                available: 3,
                ..
            }
        )),
        "expected SectionOverrun, found {:?}",
        save.diagnostics
    );
}

#[test]
fn flags_an_unexpected_kind_but_keeps_going() {
    // First section declares kind 5 instead of 1: it's noted and parsing continues with the kind read.
    let bytes = build(0, &[(5, 2, &[0, 0]), (2, 1, &1u32.to_le_bytes())]);

    let save = Save::parse(&bytes).unwrap();
    assert!(
        save.diagnostics.iter().any(|d| matches!(
            d,
            Diagnostic::UnexpectedKind {
                expected: 1,
                found: 5,
                ..
            }
        )),
        "expected UnexpectedKind, found {:?}",
        save.diagnostics
    );
    assert_eq!(save.sections[0].kind, Kind::Unknown5);
    assert_eq!(save.sections.len(), 2);
}

#[test]
fn a_missing_section_flags_only_the_one_that_took_its_place() {
    // Kind 2 is absent: 3 arrives where 2 was expected, and 4 then follows 3 as it should.
    // Counting on from the expected number flagged 4 as well, and every section after it
    // (card #80, P11a).
    let bytes = build(
        0,
        &[(1, 1, &[1]), (3, 1, &7u32.to_le_bytes()), (4, 1, &[1])],
    );

    let save = Save::parse(&bytes).unwrap();
    let unexpected: Vec<_> = save
        .diagnostics
        .iter()
        .filter_map(|d| {
            let Diagnostic::UnexpectedKind {
                expected, found, ..
            } = d
            else {
                return None;
            };
            Some((*expected, *found))
        })
        .collect();
    assert_eq!(unexpected, vec![(2, 3)]);
    assert_eq!(save.sections.len(), 3);
}

#[test]
fn flags_trailing_bytes_before_checksum() {
    // A 2-byte section, then 3 leftover bytes, then the checksum.
    let bytes = build(0, &[(1, 2, &[0, 0, 9, 9, 9])]); // count=2 but 5 bytes of data
                                                       // count*1 = 2 consumes 2 bytes; 3 bytes remain before the checksum.
    let save = Save::parse(&bytes).unwrap();
    assert_eq!(save.sections[0].bytes.len(), 2);
    assert!(
        save.diagnostics
            .iter()
            .any(|d| matches!(d, Diagnostic::TrailingBytes { len: 3, .. })),
        "expected TrailingBytes, found {:?}",
        save.diagnostics
    );
}

#[test]
fn accessors_expose_typed_views() {
    let counters: Vec<u8> = [7u32, 42u32].iter().flat_map(|v| v.to_le_bytes()).collect();
    let bytes = build(0, &[(1, 3, &[1, 0, 5]), (2, 2, &counters)]);
    let save = Save::parse(&bytes).unwrap();

    assert_eq!(save.section(Kind::Achievements).unwrap().count, 3);
    assert_eq!(
        save.flags(Kind::Achievements),
        Some(vec![true, false, true])
    );
    assert_eq!(save.u32s(Kind::Counters), Some(vec![7, 42]));

    // Wrong type for the section → None (not a panic).
    assert_eq!(save.flags(Kind::Counters), None);
    assert_eq!(save.u32s(Kind::Achievements), None);
    // Section absent → None.
    assert_eq!(save.flags(Kind::Items), None);
}

#[test]
fn bestiary_accessor_returns_raw_bytes() {
    let data = [1u8; 24];
    let bytes = build(0, &[(10, 3, &data)]);
    let save = Save::parse(&bytes).unwrap();
    assert_eq!(save.bestiary(), Some(&[1u8; 24][..]));
}

#[test]
fn diff_reports_newly_set_flags_and_changed_counters() {
    use core_save::diff;

    let ca: Vec<u8> = [1u32, 1u32].iter().flat_map(|v| v.to_le_bytes()).collect();
    let cb: Vec<u8> = [1u32, 9u32].iter().flat_map(|v| v.to_le_bytes()).collect();

    // a: achievements [1,0,0]; b: [1,0,1] → index 2 is new. Counter 1: 1→9.
    let a = Save::parse(&build(0, &[(1, 3, &[1, 0, 0]), (2, 2, &ca)])).unwrap();
    let b = Save::parse(&build(0, &[(1, 3, &[1, 0, 1]), (2, 2, &cb)])).unwrap();

    let d = diff(&a, &b);
    assert_eq!(d.achievements, vec![2]);
    assert_eq!(d.items, Vec::<usize>::new());
    assert_eq!(d.counters, vec![(1, 1, 9)]);
}

#[test]
fn a_kind_outside_the_ten_stops_the_reading_and_the_rest_is_trailing() {
    // Kind 11 has no entry size, so nothing after its header can be sized: the reading stops
    // there, names the kind, and reports everything from that header on as unread.
    let bytes = build(0, &[(1, 1, &[1]), (11, 2, &[5, 5]), (2, 1, &[0, 0, 0, 0])]);

    let save = Save::parse(&bytes).unwrap();
    assert_eq!(save.sections.len(), 1);
    let header_at = 0x14 + 12 + 1;
    let end = bytes.len() - 4;
    assert_eq!(
        save.diagnostics,
        vec![
            Diagnostic::UnexpectedKind {
                at: header_at,
                expected: 2,
                found: 11,
            },
            Diagnostic::TrailingBytes {
                at: header_at,
                len: end - header_at,
            },
        ]
    );
}

#[test]
fn a_section_carries_its_header_as_read_and_where_its_data_starts() {
    let counters: Vec<u8> = [7u32, 42u32].iter().flat_map(|v| v.to_le_bytes()).collect();
    let bytes = build(0, &[(1, 3, &[1, 0, 1]), (2, 2, &counters)]);

    let save = Save::parse(&bytes).unwrap();
    // `build` writes the second header word as count × 4, the way the game does.
    assert_eq!(save.sections[0].declared_size, 12);
    assert_eq!(save.sections[1].declared_size, 8);
    assert_eq!(save.sections[0].offset, 0x14 + 12);
    assert_eq!(save.sections[1].offset, 0x14 + 12 + 3 + 12);
}

#[test]
fn fewer_bytes_than_a_header_after_the_last_section_are_trailing() {
    // Eight bytes cannot hold a twelve-byte header: they are reported, never read as one.
    let bytes = build(0, &[(1, 2, &[0, 0, 1, 2, 3, 4, 5, 6, 7, 8])]);

    let save = Save::parse(&bytes).unwrap();
    assert_eq!(save.sections.len(), 1);
    assert_eq!(
        save.diagnostics,
        vec![Diagnostic::TrailingBytes {
            at: 0x14 + 12 + 2,
            len: 8,
        }]
    );
}
