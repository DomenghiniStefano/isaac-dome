//! Steam's achievement definitions against the game's own XML (`docs/BACKLOG.md` B56).
//!
//! `Steam\appcache\stats\UserGameStatsSchema_250900.bin` carries, per achievement, the English
//! name, the description and the icon hashes. `catalog` reads the same two strings out of the
//! game's XML — `steam_description` is literally that attribute. **Two independent copies of the
//! same sentences, and nobody had compared them**, which is what B56 calls its cheap half and
//! rates above the timeline it was logged for.
//!
//! **The KeyValues reader lives here and not in a crate, on purpose.** B56 says the format is a
//! parser this repo does not have and would have to justify; a probe is an instrument, not
//! something shipped, so it justifies nothing. The day a timeline becomes a feature, the parser
//! gets that argument on its own.
//!
//! `cargo run -p catalog --example probe_steam_schema [path to the .bin]`

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Valve's binary KeyValues: a type byte, a NUL-terminated key, then the value. `0` opens a
/// nested object and `8` closes one. Only the types this file actually uses are read; anything
/// else stops the walk rather than guessing at a length and reading rubbish afterwards.
fn read_kv(buf: &[u8]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let mut i = 0usize;
    let mut path: Vec<String> = Vec::new();
    while i < buf.len() {
        let t = buf[i];
        i += 1;
        if t == 8 {
            if path.pop().is_none() {
                break;
            }
            continue;
        }
        let start = i;
        while i < buf.len() && buf[i] != 0 {
            i += 1;
        }
        let key = String::from_utf8_lossy(&buf[start..i]).into_owned();
        i += 1;
        match t {
            0 => path.push(key),
            1 => {
                let s = i;
                while i < buf.len() && buf[i] != 0 {
                    i += 1;
                }
                let value = String::from_utf8_lossy(&buf[s..i]).into_owned();
                i += 1;
                let mut full = path.join("/");
                full.push('/');
                full.push_str(&key);
                out.insert(full, value);
            }
            2 | 3 | 6 => i += 4,
            7 => i += 8,
            _ => {
                println!("unknown KeyValues type {t} at {i}; stopping there");
                return out;
            }
        }
    }
    out
}

fn default_schema() -> PathBuf {
    PathBuf::from(r"C:\Program Files (x86)\Steam\appcache\stats\UserGameStatsSchema_250900.bin")
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_schema);
    let Ok(bytes) = std::fs::read(&path) else {
        println!("schema not readable at {}", path.display());
        println!("pass the path as the first argument; it is under Steam\\appcache\\stats\\");
        return;
    };
    let kv = read_kv(&bytes);

    // `250900/stats/<n>/bits/<k>/name` is the achievement's **id as a string**, and `bit` is its
    // 0-based position. The two differ by one on the first block — bit 0 is id 1 — so the id is
    // read from the file rather than derived from the index.
    let mut steam: BTreeMap<u32, (String, String)> = BTreeMap::new();
    for (key, value) in &kv {
        let Some(prefix) = key.strip_suffix("/name") else {
            continue;
        };
        if !prefix.contains("/bits/") {
            continue;
        }
        let Ok(id) = value.parse::<u32>() else {
            continue;
        };
        let name = kv
            .get(&format!("{prefix}/display/name/english"))
            .cloned()
            .unwrap_or_default();
        let desc = kv
            .get(&format!("{prefix}/display/desc/english"))
            .cloned()
            .unwrap_or_default();
        steam.insert(id, (name, desc));
    }
    println!("{} achievements in Steam's schema", steam.len());

    let packed = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../samples/packed");
    let rs = unpack::ResourceSet::open(&packed);
    let cat = catalog::Catalog::build(|p| rs.read(p));
    let mine: Vec<_> = cat.achievements().collect();
    println!("{} achievements in the game's XML", mine.len());
    if mine.is_empty() {
        println!("no catalog: samples/packed is missing, so there is nothing to compare");
        return;
    }

    let (mut both, mut desc_same, mut desc_differ, mut desc_absent) = (0, 0, 0, 0);
    let (mut name_in_text, mut name_missing) = (0, 0);
    let mut differences = Vec::new();
    for a in &mine {
        let Some((name, desc)) = steam.get(&a.id.0) else {
            continue;
        };
        both += 1;
        // **An empty `steam_description` is an absence, not a disagreement**, and counting it as
        // one was this probe's first answer: 232 "differ" that were almost all the XML having
        // the attribute with nothing in it.
        match a.steam_description.as_deref().filter(|d| !d.is_empty()) {
            None => desc_absent += 1,
            Some(d) if d == desc => desc_same += 1,
            Some(d) => {
                desc_differ += 1;
                differences.push(format!(
                    "  {} desc\n    xml:   {d}\n    steam: {desc}",
                    a.id.0
                ));
            }
        }
        // The XML's text is the in-game message, `You unlocked "Magdalene"`, so the agreement to
        // look for is containment and not equality.
        if !name.is_empty() && a.text.contains(name.as_str()) {
            name_in_text += 1;
        } else {
            name_missing += 1;
            differences.push(format!(
                "  {} name\n    xml:   {}\n    steam: {name}",
                a.id.0, a.text
            ));
        }
    }

    println!("\n{both} ids in both");
    println!(
        "  description: {desc_same} agree, {desc_differ} differ, {desc_absent} absent from the XML"
    );
    println!("  name: {name_in_text} found in the XML's text, {name_missing} not");

    // Three sources, three counts: Steam's schema, the game's XML, and the save, which declares
    // 642. Whoever is missing from whom is worth naming rather than subtracting.
    let ids: std::collections::BTreeSet<u32> = mine.iter().map(|a| a.id.0).collect();
    let only_steam: Vec<u32> = steam.keys().copied().filter(|i| !ids.contains(i)).collect();
    let only_xml: Vec<u32> = ids
        .iter()
        .copied()
        .filter(|i| !steam.contains_key(i))
        .collect();
    println!("  only in Steam: {only_steam:?}");
    println!("  only in the XML: {only_xml:?}");
    if !differences.is_empty() {
        println!("\n=== every disagreement ===");
        for d in &differences {
            println!("{d}");
        }
    }
}
