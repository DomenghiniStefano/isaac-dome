//! A run's face, on the installed game. The item line names a Tainted character with its base
//! form's name, so the face is found by the id the player line states, and by name only when
//! the name means one character.

use ipc::{runs_view, RunSource, RunsInputs};
use run::{Outcome, Run, SeedKind};

fn played(character: &str, id: Option<u32>) -> Run {
    Run {
        seed_words: "AAAA AAAA".to_string(),
        seed_numeric: 1,
        seed_kind: SeedKind::New,
        character: Some(character.to_string()),
        character_id: id,
        starting_items: vec![],
        collected: vec![],
        passives: vec![],
        familiars: vec![],
        held_active: None,
        floors: vec![],
        achievements: vec![],
        greed: false,
        outcome: Outcome::Open,
    }
}

/// The face each run is given, as the icon reference that would be served, so the test reads
/// which row of the co-op menu was picked without a server.
fn faces(c: &catalog::Catalog, runs: Vec<Run>) -> Vec<Option<String>> {
    runs_view(
        RunsInputs {
            sources: vec![(
                RunSource::Live {
                    id: 1,
                    written_unix: None,
                },
                runs,
            )],
            catalog: Some(c),
            wiki: None,
            diagnostics: vec![],
        },
        |icon| Some(format!("{icon:?}")),
    )
    .runs
    .into_iter()
    .map(|r| r.character_head_url)
    .collect()
}

#[test]
fn a_tainted_character_has_its_own_face_by_id_and_none_by_a_shared_name() {
    let Some(dir) = test_support::packed_dir() else {
        return;
    };
    let rs = unpack::ResourceSet::open(&dir);
    let c = catalog::Catalog::build(|p| rs.read(p));
    // Isaac is 0 and Tainted Isaac 21 in `players.xml`; both are written "Isaac" on an item line.
    let got = faces(
        &c,
        vec![
            played("Isaac", Some(0)),
            played("Isaac", Some(21)),
            played("Isaac", None),
        ],
    );
    let (base, tainted) = (got[0].clone(), got[1].clone());
    assert!(base.is_some(), "Isaac has a face: {got:?}");
    assert!(tainted.is_some(), "Tainted Isaac has a face: {got:?}");
    assert_ne!(base, tainted, "two characters, two faces: {got:?}");
    assert_eq!(got[2], None, "a name two characters share picks neither");
}
