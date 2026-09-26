//! `dataset/wiki/` is the `build` of `dataset/raw/`, file for file: whoever touches the
//! parser or the snapshot must regenerate the derived directory, or the binary embeds a
//! stale one.

use std::path::{Path, PathBuf};

use wiki::{build, Corrections, Dataset, Raw};

#[test]
fn wiki_dir_is_the_build_of_raw() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset");
    let raw = Raw::load(&root.join("raw")).expect("dataset/raw/");
    let corrections: Corrections = serde_json::from_str(
        &std::fs::read_to_string(root.join("corrections.json")).expect("corrections.json"),
    )
    .expect("json");
    let expected = build(&raw, &corrections);
    let committed_dir = root.join("wiki");
    let committed = Dataset::read_dir(&committed_dir).expect("dataset/wiki/ reads as a dataset");
    assert!(
        expected == committed,
        "dataset/wiki/ does not match build(raw/): rerun `pnpm wiki:build` and commit"
    );
    assert_every_file_matches(&expected, &committed_dir);
}

/// [`Dataset`]'s `PartialEq` alone would pass on a directory missing a file (a wrongly
/// deleted `challenges.json` still deserializes to the same empty map [`Dataset::empty`]
/// starts with), so the comparison above is not enough: every file the writer would have
/// produced has to actually be there, with the exact bytes it would write.
fn assert_every_file_matches(expected: &Dataset, dir: &Path) {
    let scratch = tempfile::tempdir().expect("tempdir");
    let written = expected
        .write_dir(scratch.path())
        .expect("writing the reference copy");
    for (file, _) in written {
        let committed =
            std::fs::read(dir.join(&file)).unwrap_or_else(|e| panic!("dataset/wiki/{file}: {e}"));
        let reference = std::fs::read(scratch.path().join(&file)).expect("just written");
        assert!(
            committed == reference,
            "dataset/wiki/{file} does not match build(raw/): rerun `pnpm wiki:build` and commit"
        );
    }
}
