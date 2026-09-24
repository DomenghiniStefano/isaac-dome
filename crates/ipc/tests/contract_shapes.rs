//! The five cases the B2 spike singled out, pinned on the real types rather than on copies.

use ts_rs::{Config, TS};

fn cfg() -> Config {
    Config::new().with_large_int("number")
}

#[test]
fn the_fields_of_a_struct_variant_are_camel_case_too() {
    // `rename_all` on an enum renames the variants; the fields inside them need
    // `rename_all_fields`. Forgetting it is silent — TypeScript reads `undefined`.
    assert!(<ipc::ActiveProfile as TS>::decl(&cfg()).contains("autoSelected"));
    assert!(!<ipc::ActiveProfile as TS>::decl(&cfg()).contains("auto_selected"));
}

#[test]
fn a_serde_rename_is_read_and_not_guessed() {
    // B9's rule in its original form: `miniZ` is not a typo and no generator may tidy it.
    assert!(<ipc::ArchiveMode as TS>::decl(&cfg()).contains(r#""miniZ""#));
}

#[test]
fn a_one_field_newtype_is_its_inner_type() {
    // It used to be called `a_transparent_newtype_…`, after a `#[serde(transparent)]`
    // ts-rs never read: the declaration comes from the shape of the struct, not from
    // the attribute, which is why removing the attribute left this line untouched.
    assert_eq!(<ipc::GoalId as TS>::decl(&cfg()), "type GoalId = string;");
}

#[test]
fn a_unix_timestamp_is_a_number_and_never_a_bigint() {
    // `i64` takes the configured large-int type, whose default is `bigint`. JSON has no
    // bigint: `serde_json` writes a number, so `bigint` would be a lie about the wire.
    assert!(<ipc::Goal as TS>::decl(&cfg()).contains("createdUnix: number"));
}

#[test]
fn the_section_kind_stops_being_a_bare_string() {
    // The documented incident, from the other side: the hand-written mirror typed this
    // `string`, which is why a renamed variant changed the wire with the suite green.
    assert!(<ipc::SectionCount as TS>::decl(&cfg()).contains("kind: Kind"));
}

#[test]
fn no_fieldless_enum_crosses_as_a_tagged_object() {
    // CLAUDE.md: a fieldless enum is a bare camelCase string, and a tagged unit enum is a bug.
    // Read from the generated contract, not a list of names, so a new one fails the day it is
    // written. The recognizer's own cases are in `tests/contract.rs`.
    let contract = ipc::contract::render();

    assert_eq!(
        ipc::contract::tagged_fieldless_unions(&contract),
        Vec::<&str>::new()
    );
}
