//! Card #81, V1: who is being played, and which rows of the completion matrix that is. This was
//! logic inside the `live` command, which is wiring and untested; the expected values are what
//! that command did, read from it — a move, not a change.

use catalog::Catalog;

/// Isaac twice, the way `players.xml` has him: the Tainted form shares the base form's name and
/// is told apart by the `_b` portrait.
const PLAYERS: &[u8] = b"<players portraitroot=\"gfx/ui/stage/\">\
<player id=\"0\" name=\"#ISAAC_NAME\" portrait=\"isaac.png\" />\
<player id=\"1\" name=\"#MAGDALENE_NAME\" portrait=\"magdalene.png\" />\
<player id=\"21\" name=\"#ISAAC_NAME\" portrait=\"isaac_b.png\" />\
</players>";

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "players.xml" => Some(PLAYERS.to_vec()),
        _ => None,
    })
}

fn isaac_name(c: &Catalog) -> String {
    let isaac = c.characters().next().expect("the first player");
    c.text(&isaac.name, catalog::Language::English).to_string()
}

#[test]
fn a_name_two_forms_answer_to_is_both_of_them() {
    let c = catalog();
    let name = isaac_name(&c);
    let ids: Vec<u32> = ipc::characters_named(&c, &name, None)
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    assert_eq!(ids, vec![0, 21]);
}

#[test]
fn the_id_the_log_stated_is_exactly_that_character() {
    let c = catalog();
    let name = isaac_name(&c);
    let found = ipc::characters_named(&c, &name, Some(21));
    assert_eq!(found, vec![(21, name)]);
}

#[test]
fn a_name_nobody_answers_to_is_nobody() {
    assert!(ipc::characters_named(&catalog(), "Nobody", None).is_empty());
}

#[test]
fn the_matrix_rows_are_the_rows_of_the_characters_asked_for() {
    let c = catalog();
    let tainted_isaac = ipc::ROSTER
        .iter()
        .position(|r| r.key == "ISAAC" && r.tainted)
        .expect("Tainted Isaac has a row");
    assert_eq!(ipc::live_mark_rows(&c, &[0]), vec![0]);
    assert_eq!(ipc::live_mark_rows(&c, &[0, 21]), vec![0, tainted_isaac]);
    assert!(ipc::live_mark_rows(&c, &[]).is_empty());
}
