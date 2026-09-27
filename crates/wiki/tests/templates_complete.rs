//! Decision 10: every template that occurs anywhere in `dataset/raw/` is either read into
//! structure, or listed in `corrections.json` under `excluded.templates` with the reason. The
//! scanner is the parser's own (`for_tests::all_template_names`, which walks the same
//! `template_segments` the real build reads), not a second one written by hand to agree with
//! it — a template's *name* is what this test judges, not the section it happens to sit in:
//! `entity row minimal` gets a real arm even though most of its occurrences sit inside a
//! discarded heading, because the ones that don't need it.

use std::collections::BTreeSet;
use std::path::PathBuf;

use wiki::for_tests::all_template_names;
use wiki::{is_layout_template, parses_into_entries, Corrections, Raw};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset")
}

fn raw() -> Raw {
    Raw::load(&root().join("raw")).expect("dataset/raw/ in the repo")
}

fn corrections() -> Corrections {
    let text = std::fs::read_to_string(root().join("corrections.json"))
        .expect("dataset/corrections.json in the repo");
    serde_json::from_str(&text).expect("corrections.json is the expected map")
}

/// Every template name this parser reads into structure by name, beyond what
/// `is_layout_template` and the `infobox …` prefix already cover (`page.rs` strips those from
/// the body before any of this ever runs). Kept as one flat, alphabetised list for the same
/// reason `resolver::LAYOUT` is one list: a name added to a match arm and not here is a name
/// this test cannot see got handled, which defeats the point of a completeness check.
const HANDLED: &[&str] = &[
    "!",
    "=",
    "a",
    "achievement",
    "achievement text",
    "achievement unlock",
    "anchor",
    "bc",
    "blindfolded",
    "book of belial synergy",
    "book of virtues synergy",
    "bug",
    "c",
    "chal",
    "code",
    "collectible rows",
    "collectible table",
    "column list",
    "cu",
    "curse",
    "dlc",
    "dlc+",
    "dlc-",
    "dlcalt",
    "e",
    "entity row minimal",
    "entity table",
    "floor",
    "heart",
    "hearts",
    "i",
    "ip",
    "m",
    "machine",
    "mode",
    "p",
    "plat",
    "r",
    "room",
    "s",
    "scroll box",
    "t",
    "tear delay down",
    "tf",
    "transformation contribution",
    "trinket rows",
    "trinket table",
];

fn understood(name: &str) -> bool {
    is_layout_template(name) || name.starts_with("infobox") || HANDLED.contains(&name)
}

/// The full set of template names in the pages the build reads, at every nesting depth,
/// exactly as the parser's own scanner sees them — walking every section, because a template
/// inside a discarded heading still "occurs in dataset/raw/". A page of a kind the build does
/// not read yet is left out: what is not parsed cannot be judged read or lost.
fn names_in_raw() -> BTreeSet<String> {
    raw()
        .pages
        .iter()
        .filter(|p| parses_into_entries(p.index.kind))
        .flat_map(|p| all_template_names(&p.text))
        .collect()
}

#[test]
fn every_template_is_handled_or_excluded_with_a_reason() {
    let names = names_in_raw();
    // Vacuity guard: the corpus has to actually contain templates for this to say anything.
    assert!(
        names.len() > 50,
        "only {} template names found",
        names.len()
    );

    let corr = corrections();
    let unaccounted: Vec<&String> = names
        .iter()
        .filter(|name| !understood(name) && !corr.excluded.templates.contains_key(name.as_str()))
        .collect();
    assert!(
        unaccounted.is_empty(),
        "templates neither handled nor excluded, with a reason, in corrections.json: {unaccounted:?}"
    );
}

#[test]
fn no_excluded_template_is_stale() {
    let names = names_in_raw();
    let corr = corrections();
    let stale: Vec<&String> = corr
        .excluded
        .templates
        .keys()
        .filter(|name| !names.contains(name.as_str()))
        .collect();
    assert!(
        stale.is_empty(),
        "excluded templates the corpus no longer uses: {stale:?}"
    );
}

/// The two lists have to disagree with each other for the tests above to mean anything: a
/// name understood on both counts, or a `HANDLED` entry that doesn't exist as a template
/// anywhere, would let a broken check pass by asserting nothing.
#[test]
fn the_lists_are_not_vacuous_against_each_other() {
    let names = names_in_raw();
    assert!(
        !corrections().excluded.templates.is_empty(),
        "corrections.json carries at least one exclusion"
    );
    let handled_and_present: Vec<&&str> = HANDLED
        .iter()
        .filter(|name| names.contains(**name))
        .collect();
    assert!(
        handled_and_present.len() > 10,
        "too few of HANDLED's own names occur in the corpus: {handled_and_present:?}"
    );
}
