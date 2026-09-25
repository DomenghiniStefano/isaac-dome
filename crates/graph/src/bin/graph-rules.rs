//! Writes `crates/graph/rules/requirements.json` from `dataset/wiki.json`. Offline, one
//! pass per snapshot: the app never runs this.

use std::path::Path;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let src = root.join("dataset/wiki.json");
    let json = match std::fs::read_to_string(&src) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("cannot read {}: {e}", src.display());
            std::process::exit(1);
        }
    };
    let dataset = match wiki::Dataset::from_json(&json) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("invalid dataset: {e}");
            std::process::exit(1);
        }
    };
    let requirements = graph::generate::generate(&dataset);
    let body = match serde_json::to_string_pretty(&requirements) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("cannot serialize: {e}");
            std::process::exit(1);
        }
    };
    let out = root.join("crates/graph/rules/requirements.json");
    if let Some(dir) = out.parent() {
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!("cannot create {}: {e}", dir.display());
            std::process::exit(1);
        }
    }
    if let Err(e) = std::fs::write(&out, format!("{body}\n")) {
        eprintln!("cannot write {}: {e}", out.display());
        std::process::exit(1);
    }
    eprintln!(
        "{} achievements, {} targets, each needing a verdict -> {}",
        requirements.achievements.len(),
        requirements.targets.len(),
        out.display()
    );
}
