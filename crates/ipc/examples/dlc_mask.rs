//! What do the lower bits of the Cargo `dlc` integer mean? **Answered on 2026-09-15**, and
//! kept as an example because what it prints is still worth looking at.
//!
//! Run: `cargo run -q -p ipc --example dlc_mask`
//!
//! Reads `dataset/raw/cargo/collectible.json` and `catalog::origin_of`, nothing else: no
//! game install, no network. **It asserts nothing on purpose** — `wiki`'s
//! `the_cargo_dlc_integer_is_the_infobox_code_through_the_wikis_own_switch` is the test.
//!
//! The integer is what the wiki's own `Template:Dlcset` returns for the page's `dlc` code:
//! a five-bit set of the editions the row is valid in (1 Rebirth, 2 Afterbirth,
//! 4 Afterbirth+, 8 Repentance, 16 Repentance+), which `wiki::Editions` transcribes and
//! `in_current_edition` filters on bit 16 to keep Tonsil's trinket and drop its
//! Afterbirth+ collectible. So yes, the lower bits mean "exists in".
//!
//! **Blue Cap (342) was the reason to doubt it, and it is the reason to believe it.** The
//! first Afterbirth collectible does not exist in vanilla Rebirth, and its mask sets bit 1
//! — because its mask is **31**, every bit, which is what the switch returns for a page
//! that declares no range at all. 341 of the 720 collectible pages do. Nothing there claims
//! Blue Cap exists in Rebirth; the naive reading of "the lowest set bit is the edition it
//! first appears in" is what has no answer for a row that declares nothing.
//!
//! That accounts for five of the eight disagreements below. The other three are the id
//! ranges, not the mask: 474 is Tonsil's slot reused by Broken Glass Cannon, and 263 and
//! 441 are pages whose edition the wiki states differently from where the id falls.

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
