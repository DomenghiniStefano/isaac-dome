//! Writes `ui/src/lib/ipc/types.ts` from the Rust types. Offline, run by hand through
//! `pnpm ipc:types`; the app never runs this.
//!
//! Thin on purpose, like `crates/graph/src/bin/graph-rules.rs`: paths, I/O and exit codes.
//! Everything with a return value worth checking is in `ipc::contract`.

use std::path::{Path, PathBuf};

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out = match std::env::var("ISAACDOME_TYPES_OUT") {
        Ok(p) => PathBuf::from(p),
        Err(_) => root.join("ui/src/lib/ipc/types.ts"),
    };
    if let Some(dir) = out.parent() {
        if let Err(e) = std::fs::create_dir_all(dir) {
            eprintln!("cannot create {}: {e}", dir.display());
            std::process::exit(1);
        }
    }
    if let Err(e) = std::fs::write(&out, ipc::contract::render()) {
        eprintln!("cannot write {}: {e}", out.display());
        std::process::exit(1);
    }
    println!("wrote {}", out.display());
}
