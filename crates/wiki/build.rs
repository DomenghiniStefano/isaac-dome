//! With the `embedded` feature it reads every file of `dataset/wiki/`, merges them into one
//! compact JSON object keyed by file stem (`meta`, `items`, `trinkets`, …), and deflates that
//! single buffer into `$OUT_DIR/wiki.json.deflate`, the blob `Dataset::embedded` inflates and
//! parses at startup. A missing directory is a build error, with the expected path in the
//! message. Without the feature (the `wiki-snapshot` tool, which is what produces the
//! directory) it does nothing. Generic on purpose: a new file under `dataset/wiki/` — a new
//! collection — needs no change here, since the merge just mirrors whatever is in the
//! directory.

use std::path::{Path, PathBuf};

/// `miniz_oxide`'s maximum level: it compresses once per build, and is read at every startup.
const LEVEL: u8 = 10;

fn main() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dataset/wiki");
    println!("cargo:rerun-if-changed={}", source.display());
    if std::env::var_os("CARGO_FEATURE_EMBEDDED").is_none() {
        return;
    }
    let files = collection_files(&source);
    for file in &files {
        println!("cargo:rerun-if-changed={}", file.display());
    }
    let merged = merge(&source, &files);
    let deflated = miniz_oxide::deflate::compress_to_vec(merged.as_bytes(), LEVEL);
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is set by cargo");
    let target = Path::new(&out_dir).join("wiki.json.deflate");
    std::fs::write(&target, deflated)
        .unwrap_or_else(|e| panic!("writing {}: {e}", target.display()));
}

/// Every `*.json` file directly under `dir`, sorted for a deterministic merge order.
fn collection_files(dir: &Path) -> Vec<PathBuf> {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| {
        panic!(
            "dataset/wiki/ not found at {} ({e}): the `embedded` feature of the `wiki` crate \
             bakes in the derived dataset, which lives in the repo; to regenerate it run `pnpm wiki:build` \
             (the tool compiles without the feature)",
            dir.display()
        )
    });
    let mut files: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    files
}

/// One compact JSON object, `{"meta": …, "items": …, …}`, one key per file (its stem), each
/// value parsed and re-emitted rather than concatenated as text, so the result is valid JSON
/// regardless of the source files' own formatting.
fn merge(dir: &Path, files: &[PathBuf]) -> String {
    let mut map = serde_json::Map::new();
    for path in files {
        let key = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_else(|| panic!("{}: not a plain file name", path.display()))
            .to_string();
        let text = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
        let value: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or_else(|e| panic!("{} is not valid JSON: {e}", path.display()));
        map.insert(key, value);
    }
    serde_json::to_string(&serde_json::Value::Object(map))
        .unwrap_or_else(|e| panic!("{}: merging into one object: {e}", dir.display()))
}
