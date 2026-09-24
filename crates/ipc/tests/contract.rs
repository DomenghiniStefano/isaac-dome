//! The one shape `ts-rs` cannot spell the way frontend rule 5 demands.
//!
//! The input is the generator's raw output — double quotes, trailing semicolon — because the
//! rewriter runs before prettier does.

use ipc::contract::{pascal_case, tagged_fieldless_unions, to_const_enums};

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

#[test]
fn a_union_of_bare_tags_is_named() {
    let file = r#"export type WantDiagnostic = { "kind": "noCatalog" } | { "kind": "noProfile" };"#;

    assert_eq!(tagged_fieldless_unions(file), vec!["WantDiagnostic"]);
}

#[test]
fn one_member_with_data_makes_the_tag_legitimate() {
    let file =
        r#"export type SaveReason = { "kind": "tooShort" } | { "kind": "io", reason: IoReason, };"#;

    assert!(tagged_fieldless_unions(file).is_empty());
}

#[test]
fn an_empty_struct_variant_carries_nothing_and_counts_as_bare() {
    let file = r#"export type Drawn = { "kind": "greedier", } | { "kind": "other" };"#;

    assert_eq!(tagged_fieldless_unions(file), vec!["Drawn"]);
}

#[test]
fn a_single_bare_variant_is_named_too() {
    let file = r#"export type Only = { "kind": "one" };"#;

    assert_eq!(tagged_fieldless_unions(file), vec!["Only"]);
}

#[test]
fn a_pipe_inside_a_field_is_not_a_member_boundary_that_hides_data() {
    let file =
        r#"export type Lock = { "kind": "free" } | { "kind": "unlocked", text: string | null, };"#;

    assert!(tagged_fieldless_unions(file).is_empty());
}

#[test]
fn string_unions_and_the_const_pair_are_not_tagged() {
    let file = r#"export type MissingReason = "steamNotFound" | "noSaves";
export const X = {
  A: "a",
} as const;
export type X = (typeof X)[keyof typeof X];"#;

    assert!(tagged_fieldless_unions(file).is_empty());
}

#[test]
fn a_declaration_split_across_lines_is_not_read() {
    // The limit, stated: the recognizer reads one line per declaration, as `ts-rs` writes a
    // union whose members carry no field docs. A multi-line union has fields, so this is not a
    // hole today — the test exists so that the day it is, the reason is already written here.
    let file = "export type Split = { \"kind\": \"a\" }\n  | { \"kind\": \"b\" };";

    assert!(tagged_fieldless_unions(file).is_empty());
}
