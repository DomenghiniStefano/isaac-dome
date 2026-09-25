//! Throwaway (B58): every *located mark cell* that moves between two consecutive
//! snapshots, beside the four located tallies and index 188.
//!
//! Run: `cargo run -q -p core-save --example era_series -- <a.dat> <b.dat> ...`
//!
//! The point is to re-derive the Mother and The Beast bases **in the 638 era**, the same
//! way they were derived in the 642 one: a mark cell rising on the same day as that boss's
//! kill counter. The bases in `marks.rs` came from the 642 series only, and an era in
//! between can only be trusted if it is bracketed by two that were measured.

use std::collections::BTreeMap;

use core_save::{cell_index, counter_index_of, Column, CounterKey};
use core_save::{Kind, Save};

const TALLIES: [CounterKey; 4] = [
    CounterKey::HushKills,
    CounterKey::DeliriumKills,
    CounterKey::MotherKills,
    CounterKey::BeastKills,
];

/// Every located cell, keyed by its index, so a moved index can name itself. The rows are
/// named by `ipc`'s roster, the one list of them.
fn located() -> BTreeMap<usize, String> {
    let cells = ipc::ROSTER.iter().enumerate().flat_map(|(row, character)| {
        Column::ALL.into_iter().filter_map(move |column| {
            let i = cell_index(row, column)?;
            Some((i, format!("{} x {column:?}", character.name)))
        })
    });
    let tallies = TALLIES
        .into_iter()
        .map(|key| (counter_index_of(key), format!("tally {key:?}")));
    cells
        .chain(tallies)
        .chain([(188, "winners mask".to_string())])
        .collect()
}

fn load(p: &str) -> Save {
    let bytes = std::fs::read(p).expect("readable");
    Save::parse(&bytes).expect("parses")
}

fn main() {
    let known = located();
    let files: Vec<String> = std::env::args().skip(1).collect();
    for pair in files.windows(2) {
        let (a, b) = (load(&pair[0]), load(&pair[1]));
        let (ca, cb) = (
            a.u32s(Kind::Counters).expect("counters"),
            b.u32s(Kind::Counters).expect("counters"),
        );
        let moved: Vec<(usize, u32, u32)> = ca
            .iter()
            .zip(cb.iter())
            .enumerate()
            .filter(|(i, (x, y))| x != y && known.contains_key(i))
            .map(|(i, (x, y))| (i, *x, *y))
            .collect();
        if moved.is_empty() {
            continue;
        }
        println!("\n#### {} -> {}", pair[0], pair[1]);
        for (i, x, y) in &moved {
            println!("   [{i}] {x} -> {y}   {}", known[i]);
        }
    }
}
