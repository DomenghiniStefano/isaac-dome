//! The one shape `ts-rs` cannot spell the way frontend rule 5 demands.
//!
//! The input is the generator's raw output — double quotes, trailing semicolon — because the
//! rewriter runs before prettier does.

use ipc::contract::{pascal_case, to_const_enums};

#[test]
fn a_union_of_string_literals_becomes_the_const_pair() {
    let input = r#"export type MissingReason = "steamNotFound" | "gameNotFound" | "noSaves";"#;

    let expected = r#"export const MissingReason = {
  SteamNotFound: "steamNotFound",
  GameNotFound: "gameNotFound",
  NoSaves: "noSaves",
} as const;
export type MissingReason = (typeof MissingReason)[keyof typeof MissingReason];"#;

    assert_eq!(to_const_enums(input), expected);
}

#[test]
fn a_snake_case_value_still_yields_a_pascal_case_key() {
    // `SavePrefix` comes from a domain crate and keeps its own snake_case on the wire.
    let input = r#"export type SavePrefix = "rep" | "rep_plus";"#;

    assert!(to_const_enums(input).contains(r#"RepPlus: "rep_plus","#));
}

#[test]
fn a_tagged_union_is_left_alone() {
    // Not a union of string literals: every member is an object. Rule 5 is about values.
    let input = r#"export type SaveDiagnostic = { "kind": "trailingBytes" } | { "kind": "sectionOverrun", "section": number };"#;

    assert_eq!(to_const_enums(input), input);
}

#[test]
fn a_union_with_one_non_string_member_is_left_alone() {
    let input = r#"export type Weird = "a" | number;"#;

    assert_eq!(to_const_enums(input), input);
}

#[test]
fn an_interface_is_left_alone() {
    let input = "export interface Goal { id: GoalId, note: string | null, }";

    assert_eq!(to_const_enums(input), input);
}

#[test]
fn the_docs_above_a_rewritten_union_survive_it() {
    let input = r#"/**
 * Why we could not find a save.
 */
export type MissingReason = "steamNotFound" | "noSaves";"#;

    let out = to_const_enums(input);
    assert!(out.starts_with("/**\n * Why we could not find a save.\n */\n"));
    assert!(out.contains("export const MissingReason = {"));
}

#[test]
fn every_declaration_in_a_whole_file_is_visited() {
    let input = r#"export type A = "one" | "two";

export interface B { a: A, }

export type C = "three";"#;

    let out = to_const_enums(input);
    assert!(out.contains("export const A = {"));
    assert!(out.contains("export const C = {"));
    assert!(out.contains("export interface B { a: A, }"));
}

#[test]
fn pascal_case_reads_both_spellings() {
    assert_eq!(pascal_case("steamNotFound"), "SteamNotFound");
    assert_eq!(pascal_case("rep_plus"), "RepPlus");
    assert_eq!(pascal_case("rep"), "Rep");
    assert_eq!(pascal_case("championVersions"), "ChampionVersions");
}
