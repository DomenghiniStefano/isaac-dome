use catalog::CharacterId;
use ipc::{character_for, CHARACTERS};

#[test]
fn every_matrix_row_finds_exactly_one_character() {
    let Some(dir) = test_support::packed_dir() else {
        return;
    };
    let rs = unpack::ResourceSet::open(&dir);
    let c = catalog::Catalog::build(|p| rs.read(p));
    let mut seen = std::collections::HashSet::new();
    for (row, (label, _)) in CHARACTERS.iter().enumerate() {
        let ch = character_for(row, &c).unwrap_or_else(|| panic!("no character for {label}"));
        assert!(
            seen.insert(ch.id),
            "{label} and another row both claim the same character {}",
            ch.id.0
        );
    }
}

/// "34 distinct" isn't enough: if the two tables (`CHARACTERS`, `CHARACTER_KEYS`) drift
/// out of sync with each other, the test above stays green regardless (rows still
/// distinct, just shifted). These anchors pin the row -> `CharacterId` identity at known
/// points, verified against the real catalog: a silent shift breaks them.
#[test]
fn matrix_row_anchors_point_at_the_expected_character_ids() {
    let Some(dir) = test_support::packed_dir() else {
        return;
    };
    let rs = unpack::ResourceSet::open(&dir);
    let c = catalog::Catalog::build(|p| rs.read(p));
    let anchors: [(usize, &str, CharacterId); 5] = [
        (0, "Isaac", CharacterId(0)),
        (14, "The Forgotten", CharacterId(16)),
        (16, "Jacob & Esau", CharacterId(19)),
        (17, "T. Isaac", CharacterId(21)),
        (33, "T. Jacob & Esau", CharacterId(37)),
    ];
    for (row, label, expected_id) in anchors {
        assert_eq!(CHARACTERS[row].0, label, "row {row} is no longer {label}");
        let ch = character_for(row, &c).unwrap_or_else(|| panic!("no character for {label}"));
        assert_eq!(
            ch.id, expected_id,
            "{label} (row {row}) points at {:?}, not at {expected_id:?}",
            ch.id
        );
    }
}
