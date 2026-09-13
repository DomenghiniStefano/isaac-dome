//! What do the lower bits of the Cargo `dlc` integer mean?
//!
//! Run: `cargo run -q -p ipc --example dlc_mask`
//!
//! Reads `dataset/raw/cargo/collectible.json` and `catalog::origin_of`, nothing else: no
//! game install, no network. **It asserts nothing on purpose.** The encoding is already
//! settled and load-bearing — `wiki::resolver` documents it (1 Rebirth, 2 Afterbirth,
//! 4 Afterbirth+, 8 Repentance, 16 Repentance+) and `in_current_edition` filters on bit 16
//! to keep Tonsil's trinket and drop its Afterbirth+ collectible. What is open is whether
//! the four lower bits mean "exists in", the way `in_current_edition`'s comment reads them.
//!
//! Blue Cap (342) is the reason to doubt it: the first Afterbirth collectible, it does not
//! exist in vanilla Rebirth, yet its mask sets bit 1. A test that asserted the naive
//! reading would be a test that is wrong, so this is an example instead — its output is a
//! measurement, and the answer belongs in the spec, written by a person.

use std::collections::BTreeMap;
use std::path::PathBuf;

use catalog::{origin_of, ItemId, ItemKind, Origin};

const BITS: [(u32, &str); 5] = [
    (1, "Rebirth"),
    (2, "Afterbirth"),
    (4, "Afterbirth+"),
    (8, "Repentance"),
    (16, "Repentance+"),
];

fn mask_name(mask: u32) -> String {
    let named: Vec<&str> = BITS
        .iter()
        .filter(|(bit, _)| mask & bit != 0)
        .map(|(_, name)| *name)
        .collect();
    let leftover = mask & !BITS.iter().map(|(b, _)| b).sum::<u32>();
    match leftover {
        0 => format!("{mask:>2} = {}", named.join("+")),
        x => format!("{mask:>2} = {} + unknown bits {x:#b}", named.join("+")),
    }
}

/// The lowest bit set, read as "the edition this row first appears in".
fn lowest_edition(mask: u32) -> Option<&'static str> {
    BITS.iter()
        .find(|(bit, _)| mask & bit != 0)
        .map(|(_, name)| *name)
}

fn origin_name(o: Origin) -> &'static str {
    match o {
        Origin::Rebirth => "Rebirth",
        Origin::Afterbirth => "Afterbirth",
        Origin::AfterbirthPlus => "Afterbirth+",
        Origin::Repentance => "Repentance",
    }
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset/raw");
    let raw = match wiki::Raw::load(&root) {
        Ok(raw) => raw,
        Err(e) => {
            eprintln!("dataset/raw/ does not load: {e:?}");
            return;
        }
    };

    // (origin from the game's id boundaries, mask from the wiki) -> how many items.
    let mut table: BTreeMap<(&str, u32), u32> = BTreeMap::new();
    let mut masks: BTreeMap<u32, u32> = BTreeMap::new();
    let mut agrees = 0u32;
    let mut disagrees: Vec<String> = Vec::new();
    let mut rows = 0u32;

    for row in &raw.tables.collectible {
        let Some(id) = row.get("id").and_then(|s| s.trim().parse::<u32>().ok()) else {
            continue;
        };
        let Some(mask) = row.get("dlc").and_then(|s| s.trim().parse::<u32>().ok()) else {
            continue;
        };
        // The game's own answer, from id boundaries verified against `items.xml`.
        let Some(origin) = origin_of(ItemKind::Passive, ItemId(id)) else {
            continue;
        };
        rows += 1;
        let o = origin_name(origin);
        *table.entry((o, mask)).or_insert(0) += 1;
        *masks.entry(mask).or_insert(0) += 1;

        match lowest_edition(mask) {
            Some(first) if first == o => agrees += 1,
            other => {
                if disagrees.len() < 12 {
                    disagrees.push(format!(
                        "  {id:>3} {:<28} game says {o:<12} mask's lowest bit says {}",
                        row.get("_pageName").map(String::as_str).unwrap_or(""),
                        other.unwrap_or("nothing")
                    ));
                }
            }
        }
    }

    println!("collectible rows with both an id and a dlc: {rows}\n");

    println!("masks seen:");
    for (mask, n) in &masks {
        println!("  {:<44} {n:>4} items", mask_name(*mask));
    }

    println!("\nreading the lowest set bit as \"the edition it first appears in\":");
    println!(
        "  agrees with the game {agrees} of {rows} ({:.1}%)",
        f64::from(agrees) / f64::from(rows.max(1)) * 100.0
    );
    println!("  first disagreements:");
    for d in &disagrees {
        println!("{d}");
    }

    println!("\norigin (game) x mask (wiki):");
    for ((o, mask), n) in &table {
        println!("  {o:<12} {:<44} {n:>4}", mask_name(*mask));
    }
}
