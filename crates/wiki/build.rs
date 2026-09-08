//! With the `embedded` feature it compresses `dataset/wiki.json` into `$OUT_DIR/wiki.json.deflate`, the
//! blob `Dataset::embedded` bakes into the binary: the readable JSON weighs tens of
//! megabytes, the deflate a fraction of that. A missing file is a build error, with the expected
//! path in the message. Without the feature (the `wiki-snapshot` tool, which is what produces the file)
//! it does nothing.

use std::path::Path;

/// `miniz_oxide`'s maximum level: it compresses once per build, and is read at every startup.
const LEVEL: u8 = 10;

fn main() {
    println!("cargo:rerun-if-changed=../../dataset/wiki.json");
    if std::env::var_os("CARGO_FEATURE_EMBEDDED").is_none() {
        return;
    }
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dataset/wiki.json");
    let bytes = std::fs::read(&source).unwrap_or_else(|e| {
        panic!(
            "dataset/wiki.json not found at {} ({e}): the `embedded` feature of the `wiki` crate \
             bakes in the derived dataset, which lives in the repo; to regenerate it run `pnpm wiki:build` \
             (the tool compiles without the feature)",
            source.display()
        )
    });
    let deflated = miniz_oxide::deflate::compress_to_vec(&bytes, LEVEL);
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR is set by cargo");
    let target = Path::new(&out_dir).join("wiki.json.deflate");
    std::fs::write(&target, deflated)
        .unwrap_or_else(|e| panic!("writing {}: {e}", target.display()));
}
