//! Throwaway: the distribution of mark values, by column.
//!
//! Run: `cargo run -q -p core-save --example mark_values -- <save.dat>`

use std::collections::BTreeMap;

use core_save::{cell_index, Column};
use core_save::{Kind, Save};

fn main() {
    let path = std::env::args().nth(1).expect("a save");
    let bytes = std::fs::read(&path).expect("readable");
    let cells = Save::parse(&bytes)
        .expect("parses")
        .u32s(Kind::Counters)
        .expect("counters");
    println!(
        "{:<12} {:>4} {:>4} {:>4} {:>4} {:>4} {:>4}",
        "column", 0, 1, 2, 3, 5, 7
    );
    let mut total: BTreeMap<u32, u32> = BTreeMap::new();
    for column in Column::ALL {
        let mut seen: BTreeMap<u32, u32> = BTreeMap::new();
        for row in 0..34 {
            if let Some(i) = cell_index(row, column) {
                *seen.entry(cells[i]).or_default() += 1;
                *total.entry(cells[i]).or_default() += 1;
            }
        }
        let g = |v: u32| seen.get(&v).copied().unwrap_or(0);
        println!(
            "{:<12} {:>4} {:>4} {:>4} {:>4} {:>4} {:>4}",
            format!("{column:?}"),
            g(0),
            g(1),
            g(2),
            g(3),
            g(5),
            g(7)
        );
    }
    println!(
        "other values: {:?}",
        total
            .keys()
            .filter(|v| ![0, 1, 2, 3, 5, 7].contains(v))
            .collect::<Vec<_>>()
    );
}
