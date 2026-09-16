//! Every bestiary key that moved between two snapshots, per tally and per entity.
//!
//! `matched_window` reports a tally as "N records / sum S", which answers *whether* a tally
//! moved and never *which entity* moved it. The open question about tally 4 is exactly the
//! second one — **it has to gain 1 on the killer's key**, and a sum going up by one somewhere
//! is not that claim.
//!
//! The key is the game's `(EntityType, variant, subtype)`. Names are not looked up here:
//! `core-save` knows nothing about the catalog and must not start. Pipe the output through the
//! two-line node script in the entry, which reads `dataset/raw/cargo/entity.json`.
//!
//! `cargo run -p core-save --example bestiary_diff -- <before.dat> <after.dat>`

use std::collections::BTreeMap;

use core_save::Save;

fn load(p: &str) -> Save {
    let bytes = std::fs::read(p).expect("readable");
    Save::parse(&bytes).expect("parses")
}

fn keys(save: &Save) -> BTreeMap<(u32, (u32, u32, u32)), u32> {
    let mut out = BTreeMap::new();
    let Some(b) = save.bestiary_tallies() else {
        return out;
    };
    for t in &b.tallies {
        for r in &t.records {
            out.insert(
                (t.id, (r.entity.kind, r.entity.variant, r.entity.subtype)),
                r.count,
            );
        }
    }
    out
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (pa, pb) = (
        args.next().expect("give me the before snapshot"),
        args.next().expect("give me the after snapshot"),
    );
    let (a, b) = (keys(&load(&pa)), keys(&load(&pb)));

    let mut moved = 0usize;
    for (k, after) in &b {
        let before = a.get(k).copied().unwrap_or(0);
        if *after == before {
            continue;
        }
        moved += 1;
        let (tally, (kind, variant, subtype)) = k;
        let new = if a.contains_key(k) { "" } else { "  (new key)" };
        println!(
            "tally {tally}  {kind}.{variant}.{subtype}  {before} -> {after}  ({:+}){new}",
            i64::from(*after) - i64::from(before)
        );
    }
    // A key that disappears would be the game rewriting the list rather than adding to it,
    // which nothing has ever seen — so it is reported rather than assumed impossible.
    for k in a.keys() {
        if !b.contains_key(k) {
            let (tally, (kind, variant, subtype)) = k;
            println!("tally {tally}  {kind}.{variant}.{subtype}  GONE");
            moved += 1;
        }
    }
    if moved == 0 {
        println!("no bestiary key moved");
    }
}
