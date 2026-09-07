//! `requirements.json` is exactly `generate(dataset/wiki.json)`. The same guarantee
//! `crates/wiki`'s `derived` test gives the dataset: regenerate and forget to commit, and
//! this goes red instead of the two files drifting apart in silence.

use std::path::{Path, PathBuf};

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(rel)
}

#[test]
fn requirements_json_is_what_the_generator_produces() {
    let wiki_json = std::fs::read_to_string(repo("dataset/wiki.json")).expect("dataset/wiki.json");
    let d = wiki::Dataset::from_json(&wiki_json).expect("the dataset parses");
    let expected =
        serde_json::to_string_pretty(&graph::generate::generate(&d)).expect("serializes");
    let committed = std::fs::read_to_string(repo("crates/graph/rules/requirements.json"))
        .expect("crates/graph/rules/requirements.json");
    assert_eq!(
        committed.replace("\r\n", "\n"),
        format!("{expected}\n"),
        "requirements.json differs from generate(dataset/wiki.json): run `pnpm graph:rules`"
    );
}
