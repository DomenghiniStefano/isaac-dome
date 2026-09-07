//! The two rules files are compiled in with `include_str!`: cargo has to rebuild when
//! either changes, not only when the Rust does.

fn main() {
    println!("cargo:rerun-if-changed=rules/requirements.json");
    println!("cargo:rerun-if-changed=rules/corrections.json");
}
