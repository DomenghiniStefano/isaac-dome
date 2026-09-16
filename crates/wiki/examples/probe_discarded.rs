//! Which sections the build drops, grouped the way the decision is actually made, and **with
//! the page each one came from** (`docs/BACKLOG.md` B54).
//!
//! `discardedSections` in the built dataset is a count per **raw** title, and the decision is
//! taken on the **normalized** one — templates and links stripped, lowercased. So the counter
//! reports more distinct titles than there are, and it cannot say which page lost a section,
//! which is the only thing that lets a title be placed. This probe answers both.
//!
//! `cargo run -p wiki --example probe_discarded`

use std::collections::BTreeMap;
use std::path::Path;

use wiki::{normalize_title, section_kind, split_page, Raw};

struct Dropped {
    raws: BTreeMap<String, u32>,
    pages: Vec<(String, usize)>,
}

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../dataset/raw");
    let raw = match Raw::load(&dir) {
        Ok(raw) => raw,
        Err(e) => {
            println!("dataset/raw could not be read: {e}");
            return;
        }
    };

    let mut dropped: BTreeMap<String, Dropped> = BTreeMap::new();
    let mut kept = 0u32;
    for page in &raw.pages {
        let (_, sections) = split_page(&page.text);
        for s in sections {
            if section_kind(&s.title).is_some() {
                kept += 1;
                continue;
            }
            let entry = dropped
                .entry(normalize_title(&s.title))
                .or_insert_with(|| Dropped {
                    raws: BTreeMap::new(),
                    pages: Vec::new(),
                });
            *entry.raws.entry(s.title.clone()).or_default() += 1;
            entry
                .pages
                .push((page.title.clone(), s.body.lines().count()));
        }
    }

    let total: u32 = dropped.values().map(|d| d.raws.values().sum::<u32>()).sum();
    let raw_keys: usize = dropped.values().map(|d| d.raws.len()).sum();
    println!(
        "{} pages, {kept} sections kept, {total} dropped",
        raw.pages.len()
    );
    println!(
        "{} normalized titles, {raw_keys} raw spellings of them",
        dropped.len()
    );

    // The shape B54 exists for: a title seen once is not a category the parser declined, it is
    // one page's own heading, and the whole section under it is gone. Reading the total hides
    // that — so the split is printed, and the singles carry their page and their size.
    let mut many: Vec<_> = dropped
        .iter()
        .filter(|(_, d)| d.raws.values().sum::<u32>() > 1)
        .collect();
    many.sort_by_key(|(_, d)| std::cmp::Reverse(d.raws.values().sum::<u32>()));
    println!("\n=== seen more than once ===");
    for (norm, d) in many {
        let n: u32 = d.raws.values().sum();
        let spellings: Vec<&str> = d.raws.keys().map(String::as_str).collect();
        println!("{n:>5}  {norm}");
        if spellings.len() > 1 {
            println!("       spelled: {}", spellings.join(" | "));
        }
    }

    let once: Vec<_> = dropped
        .iter()
        .filter(|(_, d)| d.raws.values().sum::<u32>() == 1)
        .collect();
    println!("\n=== seen exactly once: {} ===", once.len());
    for (norm, d) in once {
        let (page, lines) = &d.pages[0];
        let raw_title = d.raws.keys().next().map(String::as_str).unwrap_or("");
        let shown = if norm.is_empty() {
            "<normalizes to nothing>"
        } else {
            norm.as_str()
        };
        println!("  {shown}  — {page} ({lines} lines)");
        if raw_title != norm {
            println!("      raw: {raw_title}");
        }
    }
}
