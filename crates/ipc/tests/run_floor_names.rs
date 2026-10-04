//! A floor's name on the wire. Greed writes the normal path's `m_Stage, m_StageType` pairs for
//! floors of its own, so the same pair names a different floor in each mode: `1,1` is the normal
//! path's Cellar I and Greed's Cellar, a chapter of one floor.

use ipc::{runs_view, RunSource, RunsInputs};
use run::{Floor, Generated, Outcome, Run, SeedKind};

const STAGES: &str = r##"<stages root="rooms/">
    <stage id="2" name="#CELLAR_NAME" path="02.Cellar.xml" />
</stages>"##;

const STRINGS: &str = r#"<stringtable><languages>
  <language id="21" index="0" name="Key"/><language id="0" index="1" name="English"/></languages>
  <category name="Stages"><key name="CELLAR_NAME"><string>Cellar</string></key></category>
  </stringtable>"#;

fn played(greed: bool) -> Run {
    Run {
        seed_words: "AAAA AAAA".to_string(),
        seed_numeric: 1,
        seed_kind: SeedKind::Net,
        character: None,
        character_id: None,
        starting_items: vec![],
        collected: vec![],
        passives: vec![],
        familiars: vec![],
        held_active: None,
        // `1,1` is the Greed sample's first floor, and the normal path's Cellar I.
        floors: vec![Floor {
            stage: 1,
            stage_type: 1,
            seed: 7,
            generated: Generated::NotSaid,
        }],
        achievements: vec![],
        greed,
        outcome: Outcome::Open,
    }
}

fn first_floor_name(run: Run) -> Option<String> {
    let c = catalog::Catalog::build(|path| match path {
        "stages.xml" => Some(STAGES.as_bytes().to_vec()),
        "stringtable.sta" => Some(STRINGS.as_bytes().to_vec()),
        _ => None,
    });
    let view = runs_view(
        RunsInputs {
            sources: vec![(
                RunSource::Live {
                    id: 1,
                    written_unix: None,
                },
                vec![run],
            )],
            catalog: Some(&c),
            wiki: None,
            diagnostics: vec![],
        },
        |_| None,
    );
    view.runs[0].floor_details[0].name.clone()
}

#[test]
fn the_same_numbers_name_a_floor_of_each_mode() {
    assert_eq!(first_floor_name(played(false)).as_deref(), Some("Cellar I"));
    assert_eq!(first_floor_name(played(true)).as_deref(), Some("Cellar"));
}
