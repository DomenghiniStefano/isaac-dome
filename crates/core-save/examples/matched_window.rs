//! Throwaway: the whole diff between two snapshots of one profile, nothing elided.
//!
//! The instrument the open measurements keep asking for — play a run, compare the live
//! save against the snapshot taken before it, read which cells moved. `live_probe` gives
//! the running commentary but names only the first eight counters and no bestiary at all;
//! `SaveDiff` covers the sections we have names for and skips 3, 5, 8 and 9, which are
//! exactly the ones still called `Unknown`. This prints all of it.
//!
//! Used on 2026-09-12 to establish that an online co-op run moves section 2, the marks and
//! bestiary tallies 1 and 2, while 3, 5, 8 and 9 do not move a byte even when the log
//! announces a cutscene.

use core_save::{diff, Kind, Save, Tally};

fn load(p: &str) -> Save {
    let bytes = std::fs::read(p).expect("readable");
    Save::parse(&bytes).expect("parses")
}

/// One cell per entry, at the section's own on-disk width. Section 10 describes itself and
/// its header does not describe this layout, so it is read through `bestiary_tallies()`
/// below and skipped here.
fn cells(s: &core_save::Section) -> Vec<u32> {
    let width = (s.bytes.len() / s.count.max(1) as usize).max(1);
    s.bytes
        .chunks(width)
        .map(|c| match c.len() {
            1 => c[0] as u32,
            4 => u32::from_le_bytes([c[0], c[1], c[2], c[3]]),
            _ => 0,
        })
        .collect()
}

fn sum(t: &Tally) -> u64 {
    t.records.iter().map(|r| r.count as u64).sum()
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (pa, pb) = (
        args.next().expect("give me the before snapshot"),
        args.next().expect("give me the after snapshot"),
    );
    let (a, b) = (load(&pa), load(&pb));

    println!("--- every section, cell by cell ---");
    for (sa, sb) in a.sections.iter().zip(b.sections.iter()) {
        if sa.kind == Kind::Bestiary {
            continue;
        }
        let (ca, cb) = (cells(sa), cells(sb));
        let moved: Vec<String> = ca
            .iter()
            .zip(cb.iter())
            .enumerate()
            .filter(|(_, (x, y))| x != y)
            .map(|(i, (x, y))| format!("[{i}] {x}->{y}"))
            .collect();
        println!(
            "  kind {:?} (n={}): {}",
            sa.kind,
            sa.kind.number(),
            if moved.is_empty() {
                "unchanged".to_string()
            } else {
                moved.join(" ")
            }
        );
    }

    let d = diff(&a, &b);
    println!("\n--- what `diff` names ---");
    println!("  achievements: {:?}", d.achievements);
    println!("  items:        {:?}", d.items);
    println!("  challenges:   {:?}", d.challenges);
    println!("  bosses:       {:?}", d.bosses);

    println!("\n--- bestiary ---");
    match (a.bestiary_tallies(), b.bestiary_tallies()) {
        (Some(ba), Some(bb)) => {
            for (ta, tb) in ba.tallies.iter().zip(bb.tallies.iter()) {
                println!(
                    "  tally id {}: {} records / sum {}  ->  {} records / sum {} ({:+})",
                    ta.id,
                    ta.records.len(),
                    sum(ta),
                    tb.records.len(),
                    sum(tb),
                    sum(tb) as i64 - sum(ta) as i64
                );
            }
            println!("  trailing: {:?} -> {:?}", ba.trailing, bb.trailing);
        }
        _ => println!("  (unreadable in one of the two)"),
    }
}
