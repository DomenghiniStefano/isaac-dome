//! Throwaway: one character's whole row of the completion matrix, before and after.
//!
//! Run: `cargo run -q -p core-save --example row_marks -- <row> <before.dat> <after.dat>`

use core_save::{cell_index, Column};
use core_save::{Kind, Save};

fn cells(p: &str) -> Vec<u32> {
    let bytes = std::fs::read(p).expect("readable");
    Save::parse(&bytes)
        .expect("parses")
        .u32s(Kind::Counters)
        .expect("counters")
}

fn main() {
    let mut a = std::env::args().skip(1);
    let row: usize = a.next().expect("row").parse().expect("number");
    let (before, after) = (
        cells(&a.next().expect("before")),
        cells(&a.next().expect("after")),
    );
    let mut bit0 = 0;
    let mut bit1 = 0;
    for column in Column::ALL {
        match cell_index(row, column) {
            Some(i) => {
                if before[i] & 1 != 0 {
                    bit0 += 1;
                }
                if before[i] & 2 != 0 {
                    bit1 += 1;
                }
                println!(
                    "  {:<16} [{i:>3}]  {:>2} -> {:>2}",
                    format!("{column:?}"),
                    before[i],
                    after[i]
                );
            }
            None => println!("  {:<16} [ - ]  not located", format!("{column:?}")),
        }
    }
    let (mut a0, mut a1) = (0, 0);
    for column in Column::ALL {
        if let Some(i) = cell_index(row, column) {
            if after[i] & 1 != 0 {
                a0 += 1;
            }
            if after[i] & 2 != 0 {
                a1 += 1;
            }
        }
    }
    println!("before: bit 0 in {bit0} columns, bit 1 in {bit1}");
    println!("after:  bit 0 in {a0} columns, bit 1 in {a1}");
}
