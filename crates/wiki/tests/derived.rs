//! `dataset/wiki.json` is the `build` of `dataset/raw/`, byte for byte: whoever touches the
//! parser or the snapshot must regenerate the derived file, or the binary embeds a stale one.

use std::path::PathBuf;

use wiki::{build, Corrections, Raw};

#[test]
fn wiki_json_is_the_build_of_raw() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset");
    let raw = Raw::load(&root.join("raw")).expect("dataset/raw/");
    let corrections: Corrections = serde_json::from_str(
        &std::fs::read_to_string(root.join("corrections.json")).expect("corrections.json"),
    )
    .expect("json");
    let expected = build(&raw, &corrections).to_json();
    let actual = std::fs::read_to_string(root.join("wiki.json")).expect("wiki.json");
    assert!(
        expected == actual,
        "dataset/wiki.json does not match build(raw/): rerun `pnpm wiki:build` and commit"
    );
}
