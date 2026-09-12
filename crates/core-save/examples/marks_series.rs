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

use core_save::{Kind, Save};

const BLOCKS_14: &[usize] = &[27, 41, 55, 69, 83, 97, 116, 130, 144, 173];
const BLOCKS_19: &[usize] = &[214, 233, 252, 271, 290, 309, 328, 347, 366];
const CHARS_14: [&str; 14] = [
    "Isaac",
    "Magdalene",
    "Cain",
    "Judas",
    "Blue Baby",
    "Eve",
    "Samson",
    "Azazel",
    "Lazarus",
    "Eden",
    "The Lost",
    "Lilith",
    "Keeper",
    "Apollyon",
];
const CHARS_19: [&str; 19] = [
    "Bethany",
    "Jacob & Esau",
    "T. Isaac",
    "T. Magdalene",
    "T. Cain",
    "T. Judas",
    "T. Blue Baby",
    "T. Eve",
    "T. Samson",
    "T. Azazel",
    "T. Lazarus",
    "T. Eden",
    "T. The Lost",
    "T. Lilith",
    "T. Keeper",
    "T. Apollyon",
    "T. Forgotten",
    "T. Bethany",
    "T. Jacob",
];
/// Index 188, `CHARACTER_LAST_RUN_WIN`: which characters won the most recent run.
const WINNER_MASK: usize = 188;

fn who(i: usize) -> Option<&'static str> {
    for base in BLOCKS_14 {
        if i >= *base && i < base + 14 {
            return Some(CHARS_14[i - base]);
        }
    }
    for base in BLOCKS_19 {
        if i >= *base && i < base + 19 {
            return Some(CHARS_19[i - base]);
        }
    }
    None
}

/// The map the series fits, not the one the matrix rows use: 0..13 then a gap then 19...
fn winner(bit: u32) -> &'static str {
    match bit {
        0..=13 => CHARS_14[bit as usize],
        19..=37 => CHARS_19[bit as usize - 19],
        _ => "?",
    }
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
    let dir = std::path::Path::new("samples");
    let Ok(entries) = std::fs::read_dir(dir) else {
        eprintln!("skip: no samples/ directory");
        return;
    };
    let mut files: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| {
            n.len() == 8 + 1 + "rep+persistentgamedata1.dat".len()
                && n.as_bytes()[8] == b'.'
                && n[..8].chars().all(|c| c.is_ascii_digit())
                && n.ends_with("rep+persistentgamedata1.dat")
        })
        .collect();
    files.sort();
    files.dedup_by(|a, b| a[..8] == b[..8]);
    if files.len() < 2 {
        eprintln!("skip: fewer than two dated rep+ samples");
        return;
    }

    let mut prev: Option<(String, Vec<u32>)> = None;
    for f in files {
        let Ok(bytes) = std::fs::read(dir.join(&f)) else {
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
