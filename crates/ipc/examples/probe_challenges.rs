//! Throwaway probe (2026-09-17): which cell of section 7 is challenge *n*?
//!
//! `challenges.xml` declares 45 challenges and the save's section 7 declares 46 cells, so the
//! mapping is either identity or off by one, and guessing it would put a tick on the wrong row.
//!
//! The instrument is the catalog's own `rewards` link: finishing a challenge grants a known set
//! of achievements, so for the right offset "the cell is set" and "every reward achievement is
//! done" agree almost everywhere, and for the wrong one they do not. Run it against the newest
//! sample:
//!
//! ```sh
//! cargo run -p ipc --example probe_challenges
//! ```

use catalog::Catalog;
use core_save::{Kind, Save};

fn main() {
    let Some(packed) = test_support::packed_dir() else {
        eprintln!("skip: samples/packed is missing, no catalog to read");
        return;
    };
    let rs = unpack::ResourceSet::open(&packed);
    let catalog = Catalog::build(|p| rs.read(p));

    let series = test_support::dated_series("rep+persistentgamedata1.dat");
    let Some(path) = series.last() else {
        eprintln!("skip: no rep+ sample to read");
        return;
    };
    eprintln!("sample: {}", path.display());

    let bytes = std::fs::read(path).expect("the sample reads");
    let save = Save::parse(&bytes).expect("the sample parses");

    let cells: Vec<u8> = save
        .section(Kind::Challenges)
        .map(|s| s.bytes.to_vec())
        .unwrap_or_default();
    let done: Vec<bool> = save
        .section(Kind::Achievements)
        .map(|s| s.bytes.iter().map(|&b| b != 0).collect())
        .unwrap_or_default();

    println!(
        "section 7: {} cells, {} set",
        cells.len(),
        cells.iter().filter(|&&b| b != 0).count()
    );
    println!("cell 0 = {}", cells.first().copied().unwrap_or(0));
    println!(
        "achievements: {} of {} done",
        done.iter().filter(|&&d| d).count(),
        done.len()
    );

    let challenges: Vec<_> = catalog.challenges().collect();
    println!(
        "catalog: {} challenges, ids {:?}..{:?}",
        challenges.len(),
        challenges.first().map(|c| c.id.0),
        challenges.last().map(|c| c.id.0)
    );

    // For each candidate offset: how often does "the cell is set" agree with "every achievement
    // this challenge rewards is done"? A challenge with no reward says nothing and is left out.
    for offset in [0i64, -1] {
        let mut agree = 0;
        let mut disagree = 0;
        let mut judged = 0;
        let mut both_true = 0;
        let mut both_false = 0;
        for ch in &challenges {
            if ch.rewards.is_empty() {
                continue;
            }
            let index = ch.id.0 as i64 + offset;
            let Ok(index) = usize::try_from(index) else {
                continue;
            };
            let Some(&cell) = cells.get(index) else {
                continue;
            };
            let rewarded = ch
                .rewards
                .iter()
                .all(|a| done.get(a.0 as usize).copied().unwrap_or(false));
            judged += 1;
            if (cell != 0) == rewarded {
                agree += 1;
                if rewarded {
                    both_true += 1
                } else {
                    both_false += 1
                }
            } else {
                disagree += 1;
            }
        }
        println!("offset {offset:+}: {agree} agree, {disagree} disagree, of {judged} judged");
        if offset == 0 {
            println!("           both true: {both_true}, both false: {both_false}");
        }
    }

    // Is `unlocked_by` "all of these achievements" or "any of them"? Nothing in the file says,
    // and the save cannot show a challenge being *offered* — but a challenge you have **finished**
    // must have been reachable, so every done challenge is a case. If one of them sits behind a
    // gate that is not done, "all of" is refuted.
    let mut gated_done = 0;
    let mut all_of_holds = 0;
    let mut any_of_only = 0;
    for ch in &challenges {
        let finished = cells.get(ch.id.0 as usize).copied().unwrap_or(0) != 0;
        if !finished || ch.unlocked_by.is_empty() {
            continue;
        }
        gated_done += 1;
        let all = ch
            .unlocked_by
            .iter()
            .all(|a| done.get(a.0 as usize).copied().unwrap_or(false));
        let any = ch
            .unlocked_by
            .iter()
            .any(|a| done.get(a.0 as usize).copied().unwrap_or(false));
        if all {
            all_of_holds += 1;
        } else if any {
            any_of_only += 1;
            println!(
                "  any-of only: {:>2} {:<28} gates={:?}",
                ch.id.0,
                ch.name,
                ch.unlocked_by.iter().map(|a| a.0).collect::<Vec<_>>()
            );
        } else {
            println!(
                "  NO gate done: {:>2} {:<28} gates={:?}",
                ch.id.0,
                ch.name,
                ch.unlocked_by.iter().map(|a| a.0).collect::<Vec<_>>()
            );
        }
    }
    println!(
        "\ndone challenges that have gates: {gated_done} — all gates done on {all_of_holds}, only some on {any_of_only}"
    );
    println!(
        "challenges with no gate at all: {}",
        challenges
            .iter()
            .filter(|c| c.unlocked_by.is_empty())
            .count()
    );

    // Does the embedded dataset have a page for every challenge? A row that shows the wiki's
    // conditions has to say when it has none, and a missing page must be loud (B45).
    let dataset = wiki::Dataset::embedded();
    let missing: Vec<u32> = challenges
        .iter()
        .filter(|ch| {
            dataset
                .as_ref()
                .ok()
                .and_then(|d| d.entry(&wiki::Target::Challenge { number: ch.id.0 }))
                .is_none()
        })
        .map(|ch| ch.id.0)
        .collect();
    println!(
        "wiki pages: {} of {} challenges, missing {:?}",
        challenges.len() - missing.len(),
        challenges.len(),
        missing
    );

    // The disagreements of the identity mapping, named, so a pattern is visible rather than
    // summarised away.
    println!("\nunder offset 0, the rows that disagree:");
    for ch in &challenges {
        if ch.rewards.is_empty() {
            continue;
        }
        let Some(&cell) = cells.get(ch.id.0 as usize) else {
            continue;
        };
        let rewarded = ch
            .rewards
            .iter()
            .all(|a| done.get(a.0 as usize).copied().unwrap_or(false));
        if (cell != 0) != rewarded {
            println!(
                "  {:>2} {:<28} cell={} rewards={:?} done={}",
                ch.id.0,
                ch.name,
                cell,
                ch.rewards.iter().map(|a| a.0).collect::<Vec<_>>(),
                rewarded
            );
        }
    }
}
