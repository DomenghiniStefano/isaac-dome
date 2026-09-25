//! Throwaway: what the dated series says about a mark's third bit, and about index 188.
//!
//! This is the instrument that named bit 2 on 2026-09-12. The live run gave one positive —
//! a won online Greed run lit bit 2 on `Greed × Cain` — and one positive names nothing on
//! its own. The series is what turns it into a reading: every window is printed with the
//! characters whose marks moved, whether any of them gained bit 2, and the winner mask,
//! so the correlation with `online_logs\` can be checked by eye against a folder listing.
//!
//! It also re-derives index 188's bit -> character map instead of assuming it. The assumed
//! order fails on one row (the 2026-07-23 window lights bit 28 while T. Azazel's marks
//! move), and the map that actually fits is: the 14 originals at bits 0..13, the 19 later
//! characters from bit 19. Two bits at once is a local co-op win, which is the only way
//! that mask ever names two characters — and the reason `marks_real.rs` skips those
//! windows when it pins the bases by identity.

use core_save::{cell_index, Column};
use core_save::{Kind, Save};

/// Index 188, `CHARACTER_LAST_RUN_WIN`: which characters won the most recent run.
const WINNER_MASK: usize = 188;

/// The character whose mark sits at index `i`, read off the layout and named by `ipc`'s
/// roster — every located cell, not the first blocks this probe was written against.
fn who(i: usize) -> Option<&'static str> {
    (0..ipc::ROSTER.len())
        .find(|&row| {
            Column::ALL
                .into_iter()
                .any(|c| cell_index(row, c) == Some(i))
        })
        .map(|row| ipc::ROSTER[row].name)
}

/// The first row of the 19 later characters: where bit 19 of the mask lands.
const FIRST_LATER_ROW: usize = 15;

/// The map the series fits, not the one the matrix rows use: 0..13 then a gap then 19...
fn winner(bit: u32) -> &'static str {
    let row = match bit {
        0..=13 => Some(bit as usize),
        19..=37 => Some(bit as usize - 19 + FIRST_LATER_ROW),
        _ => None,
    };
    row.and_then(|r| ipc::ROSTER.get(r))
        .map_or("?", |character| character.name)
}

fn counters(s: &Save) -> Vec<u32> {
    s.sections
        .iter()
        .find(|x| x.kind == Kind::Counters)
        .map(|x| {
            x.bytes
                .chunks(4)
                .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect()
        })
        .unwrap_or_default()
}

fn main() {
    // The series through `test-support`, which declares every file it hands out and uses the
    // one date filter the repo has (`is_dated`): a looser one once let a same-day
    // `20260912-pre.` snapshot win the dedup.
    let files = test_support::dated_series("rep+persistentgamedata1.dat");
    if files.len() < 2 {
        eprintln!("skip: fewer than two dated rep+ samples");
        return;
    }

    let mut prev: Option<(String, Vec<u32>)> = None;
    for path in files {
        let f = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let Ok(save) = Save::parse(&bytes) else {
            continue;
        };
        let cur = counters(&save);
        if let Some((pd, pc)) = &prev {
            let mut movers: Vec<&str> = Vec::new();
            let mut bit2: Vec<String> = Vec::new();
            for (i, (a, b)) in pc.iter().zip(cur.iter()).enumerate() {
                if a == b {
                    continue;
                }
                if let Some(name) = who(i) {
                    if !movers.contains(&name) {
                        movers.push(name);
                    }
                    if a & 4 == 0 && b & 4 != 0 {
                        bit2.push(format!("{name} ({a}->{b})"));
                    }
                }
            }
            let mask = cur.get(WINNER_MASK).copied().unwrap_or(0);
            let was = pc.get(WINNER_MASK).copied().unwrap_or(0);
            if was != mask {
                println!(
                    "      188: {was} -> {mask}   (kept {:b}, added {:b}, lost {:b})",
                    was & mask,
                    mask & !was,
                    was & !mask
                );
            }
            let won: Vec<&str> = (0..32).filter(|b| mask >> b & 1 == 1).map(winner).collect();
            if movers.is_empty() && won.is_empty() {
                prev = Some((f[..8].to_string(), cur));
                continue;
            }
            println!(
                "{pd}->{}  marks: {:<44}  188: {:<26}{}",
                &f[..8],
                if movers.is_empty() {
                    "-".into()
                } else {
                    movers.join(", ")
                },
                if won.is_empty() {
                    "-".into()
                } else {
                    won.join(" + ")
                },
                if bit2.is_empty() {
                    String::new()
                } else {
                    format!("  ONLINE: {}", bit2.join(", "))
                }
            );
        }
        prev = Some((f[..8].to_string(), cur));
    }
}
