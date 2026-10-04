//! A floor's name, from the log's `m_Stage, m_StageType` through the game's own `stages.xml` and
//! `stringtable.sta`. The fixtures keep the real files' shapes: a stage's `name` is a string key,
//! and the greed stages sit inside an XML comment.

use catalog::Catalog;

const STAGES: &str = r##"<stages root="rooms/">
    <stage id="1" name="#BASEMENT_NAME" path="01.Basement.xml" />
    <stage id="2" name="#CELLAR_NAME" path="02.Cellar.xml" />
    <stage id="29" name="#MINES_NAME" path="29.Mines.xml" />
    <stage id="13" name="#BLUE_WOMB_NAME" path="13.Blue Womb.xml" />
<!--
    <stage id="19" name="#BASEMENT_NAME" path="19.Greed Basement.xml" />
-->
</stages>"##;

const STRINGS: &str = r#"<stringtable><languages>
  <language id="21" index="0" name="Key"/><language id="0" index="1" name="English"/></languages>
  <category name="Stages">
  <key name="BASEMENT_NAME"><string>Basement</string></key>
  <key name="CELLAR_NAME"><string>Cellar</string></key>
  <key name="MINES_NAME"><string>Mines</string></key>
  <key name="BLUE_WOMB_NAME"><string>Blue Womb</string></key>
  </category></stringtable>"#;

fn catalog() -> Catalog {
    Catalog::build(|path| match path {
        "stages.xml" => Some(STAGES.as_bytes().to_vec()),
        "stringtable.sta" => Some(STRINGS.as_bytes().to_vec()),
        _ => None,
    })
}

#[test]
fn a_chapter_floor_is_named_with_its_number() {
    assert_eq!(catalog().floor_name(1, 0).as_deref(), Some("Basement I"));
    assert_eq!(catalog().floor_name(2, 1).as_deref(), Some("Cellar II"));
}

// The one pair this repo has measured on a real log (`docs/log-format.md`).
#[test]
fn stage_four_of_type_four_is_mines_two() {
    assert_eq!(catalog().floor_name(4, 4).as_deref(), Some("Mines II"));
}

#[test]
fn a_floor_alone_in_its_chapter_has_no_number() {
    assert_eq!(catalog().floor_name(9, 0).as_deref(), Some("Blue Womb"));
}

// Greed's floors are commented out in the file and have no row in the table: they keep their
// numbers rather than borrow "Basement" from the normal path.
#[test]
fn a_pair_with_no_row_has_no_name() {
    assert_eq!(catalog().floor_name(1, 3), None);
    assert_eq!(catalog().floor_name(99, 0), None);
}

#[test]
fn without_the_strings_there_is_no_name() {
    let c = Catalog::build(|p| (p == "stages.xml").then(|| STAGES.as_bytes().to_vec()));
    assert_eq!(c.floor_name(1, 0), None);
}
