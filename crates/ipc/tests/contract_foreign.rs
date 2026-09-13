//! The five types that cross the boundary from crates that are not `ipc`.
//!
//! `core_save::Kind` is the documented incident: it travels inside `ipc::SectionCount`, the
//! hand-written mirror typed it as `string`, and renaming a variant changed the wire with the
//! whole suite green. These assertions are what makes a rename fail the build instead.

use ts_rs::{Config, TS};

fn cfg() -> Config {
    Config::new().with_large_int("number")
}

#[test]
fn the_sections_of_the_save_are_a_closed_set_of_names() {
    assert_eq!(
        <core_save::Kind as TS>::decl(&cfg()),
        r#"type Kind = "achievements" | "counters" | "level_counters" | "items" | "unknown5" | "bosses" | "challenges" | "unknown8" | "unknown9" | "bestiary";"#
    );
}

#[test]
fn the_two_save_prefixes_keep_their_own_spelling() {
    assert_eq!(
        <discovery::SavePrefix as TS>::decl(&cfg()),
        r#"type SavePrefix = "rep" | "rep_plus";"#
    );
}
