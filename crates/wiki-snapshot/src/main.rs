//! `wiki-snapshot`: a developer tool, never shipped. `fetch` downloads the pages and
//! Cargo tables from bindingofisaacrebirth.wiki.gg into `dataset/raw/`, the only place
//! in the repo that talks to the network; `build` turns that snapshot into
//! `dataset/wiki.json`, the file the `wiki` crate embeds into the binary.

mod api;
mod http;
mod store;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use wiki::{build, page_file_name, Corrections, Dataset, IndexEntry, PageKind, Raw, Row};

use crate::api::{
    cargo_url, is_translation_subpage, pages_url, parse_cargo, parse_pages, sort_rows, Pending,
    ROWS_PER_REQUEST, TABLES,
};
use crate::store::{prune, write_if_changed};

const USAGE: &str =
    "usage:\n  wiki-snapshot fetch [--out <dir>]\n  wiki-snapshot build [--raw <dir>] [--out <file>]";
/// How many entries of each diagnostic map `build` shows.
const DIAGNOSTIC_ROWS: usize = 20;

/// How the program ends when things don't go well: a usage error (exit 2) or a
/// network or disk one (exit 1).
#[derive(Debug)]
enum Failure {
    Usage(String),
    Error(String),
}

type Outcome = Result<(), Failure>;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(Failure::Usage(msg)) => {
            eprintln!("{msg}\n{USAGE}");
            ExitCode::from(2)
        }
        Err(Failure::Error(msg)) => {
            eprintln!("error: {msg}");
            ExitCode::from(1)
        }
    }
}

/// The workspace root: two levels above the crate. Path defaults start from
/// here, so the tool works from whatever the current directory is.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

/// Reads `--name value` options among the allowed ones. An unknown, repeated, or
/// valueless option is a usage error.
fn parse_options(args: &[String], allowed: &[&str]) -> Result<BTreeMap<String, String>, Failure> {
    let mut opts = BTreeMap::new();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let name = arg
            .strip_prefix("--")
            .filter(|n| allowed.contains(n))
            .ok_or_else(|| Failure::Usage(format!("unknown argument: {arg}")))?;
        let value = it
            .next()
            .ok_or_else(|| Failure::Usage(format!("{arg} requires a value")))?;
        if opts.insert(name.to_string(), value.clone()).is_some() {
            return Err(Failure::Usage(format!("{arg} repeated")));
        }
    }
    Ok(opts)
}

fn run(args: &[String]) -> Outcome {
    let root = workspace_root();
    match args.first().map(String::as_str) {
        Some("fetch") => {
            let opts = parse_options(&args[1..], &["out"])?;
            let out = opts
                .get("out")
                .map(PathBuf::from)
                .unwrap_or_else(|| root.join("dataset").join("raw"));
            fetch(&out)
        }
        Some("build") => {
            let opts = parse_options(&args[1..], &["raw", "out"])?;
            let raw = opts
                .get("raw")
                .map(PathBuf::from)
                .unwrap_or_else(|| root.join("dataset").join("raw"));
            let out = opts
                .get("out")
                .map(PathBuf::from)
                .unwrap_or_else(|| root.join("dataset").join("wiki.json"));
            build_dataset(&raw, &out, &root.join("dataset").join("corrections.json"))
        }
        Some(other) => Err(Failure::Usage(format!("unknown command: {other}"))),
        None => Err(Failure::Usage("missing command".into())),
    }
}

fn io_error(what: &Path, e: std::io::Error) -> Failure {
    Failure::Error(format!("{}: {e}", what.display()))
}

/// A page's text as it goes to disk: LF line endings, because the repo forces LF in the
/// working copy and a downloaded CRLF would get rewritten on every `fetch`.
fn page_bytes(text: &str) -> Vec<u8> {
    text.replace("\r\n", "\n").into_bytes()
}

/// Downloads all pages of one kind, writes them and updates the index; returns the
/// file names written or confirmed, for the directory's `prune`.
fn fetch_kind(
    kind: PageKind,
    dir: &Path,
    index: &mut BTreeMap<String, IndexEntry>,
) -> Result<(usize, usize, BTreeSet<String>), Failure> {
    let mut keep = BTreeSet::new();
    // `page_file_name` is injective, but Windows's filesystem is case-insensitive:
    // two titles that collide would keep the first one and warn about it.
    let mut seen_lower: BTreeMap<String, String> = BTreeMap::new();
    let mut pages = 0;
    let mut written = 0;
    // Pages listed without text: the server relists in every batch the ones already
    // delivered too, and in a truncated batch it lists ahead of time the ones that will
    // arrive later. Only at the end of the kind do we know if any are still missing.
    let mut pending = Pending::default();
    let mut cont = BTreeMap::new();
    loop {
        let body = http::get(&pages_url(kind.template(), &cont)).map_err(Failure::Error)?;
        let batch = parse_pages(&body).map_err(Failure::Error)?;
        for (pageid, title) in &batch.without_revision {
            pending.seen_without(*pageid, title);
        }
        for page in batch.pages {
            pending.delivered(page.pageid);
            // Translations (`Steven/de`) transclude the same infobox: they aren't pages of ours.
            if is_translation_subpage(&page.title) {
                continue;
            }
            // A page that reappears with text within the same kind silently replaces the
            // entry; the same page under a different kind stays with the first one and warns.
            if let Some(prev) = index.get(&page.title).filter(|prev| prev.kind != kind) {
                eprintln!(
                    "  warning: «{}» is already of kind {}; ignored as {}",
                    page.title,
                    prev.kind.dir(),
                    kind.dir()
                );
                continue;
            }
            let name = format!("{}.wikitext", page_file_name(&page.title));
            if let Some(first) = seen_lower
                .get(&name.to_lowercase())
                .filter(|first| **first != page.title)
            {
                eprintln!(
                    "  warning: «{}» and «{}» have the same file name on a case-insensitive \
                     filesystem; keeping the first one",
                    first, page.title
                );
                continue;
            }
            seen_lower.insert(name.to_lowercase(), page.title.clone());
            let path = dir.join(&name);
            if write_if_changed(&path, &page_bytes(&page.text)).map_err(|e| io_error(&path, e))? {
                written += 1;
            }
            if keep.insert(name) {
                pages += 1;
            }
            index.insert(
                page.title,
                IndexEntry {
                    kind,
                    pageid: page.pageid,
                    revid: page.revid,
                    timestamp: page.timestamp,
                },
            );
        }
        match batch.cont {
            Some(c) => cont = c,
            None => break,
        }
    }
    let unresolved = pending.unresolved();
    if !unresolved.is_empty() {
        return Err(Failure::Error(format!(
            "{}: {} pages listed but never arrived with text: {}",
            kind.dir(),
            unresolved.len(),
            unresolved.join(", ")
        )));
    }
    Ok((pages, written, keep))
}

/// Downloads a whole Cargo table, in pages of `ROWS_PER_REQUEST` rows.
fn fetch_table(table: &str, fields: &str) -> Result<Vec<Row>, Failure> {
    let mut rows = Vec::new();
    let mut offset = 0;
    loop {
        let body = http::get(&cargo_url(table, fields, offset)).map_err(Failure::Error)?;
        let batch = parse_cargo(&body).map_err(Failure::Error)?;
        let n = batch.len();
        rows.extend(batch);
        if n < ROWS_PER_REQUEST {
            break;
        }
        offset += ROWS_PER_REQUEST;
    }
    sort_rows(&mut rows);
    Ok(rows)
}

fn pretty_json<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, Failure> {
    let mut s = serde_json::to_string_pretty(value)
        .map_err(|e| Failure::Error(format!("serialization: {e}")))?;
    s.push('\n');
    Ok(s.into_bytes())
}

fn fetch(out: &Path) -> Outcome {
    println!("snapshot at {}", out.display());
    let mut index = BTreeMap::new();
    for kind in PageKind::ALL {
        let dir = out.join("pages").join(kind.dir());
        let (pages, written, keep) = fetch_kind(kind, &dir, &mut index)?;
        let removed = prune(&dir, &keep).map_err(|e| io_error(&dir, e))?;
        for name in &removed {
            println!("  deleted {}", name);
        }
        println!(
            "{}: {pages} pages, {written} written, {} deleted",
            kind.dir(),
            removed.len()
        );
    }
    for (table, fields) in TABLES {
        let rows = fetch_table(table, fields)?;
        let path = out.join("cargo").join(format!("{table}.json"));
        write_if_changed(&path, &pretty_json(&rows)?).map_err(|e| io_error(&path, e))?;
        println!("{table}: {} rows", rows.len());
    }
    let path = out.join("index.json");
    write_if_changed(&path, &pretty_json(&index)?).map_err(|e| io_error(&path, e))?;
    println!("index.json: {} pages", index.len());
    Ok(())
}

/// `corrections.json` if present; a missing file counts as an empty map, a malformed one is an error.
fn load_corrections(path: &Path) -> Result<Corrections, Failure> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text)
            .map_err(|e| Failure::Error(format!("{}: {e}", path.display()))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Corrections::default()),
        Err(e) => Err(io_error(path, e)),
    }
}

/// A diagnostic map sorted by descending value, the top `DIAGNOSTIC_ROWS`.
fn print_diagnostic(title: &str, map: &BTreeMap<String, u32>) {
    let total: u32 = map.values().sum();
    println!("{title}: {} entries, {total} occurrences", map.len());
    let mut rows: Vec<(&String, &u32)> = map.iter().collect();
    rows.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    for (name, n) in rows.into_iter().take(DIAGNOSTIC_ROWS) {
        println!("  {n:>6}  {name}");
    }
}

fn print_meta(ds: &Dataset) {
    let m = &ds.meta;
    let c = &m.counts;
    println!(
        "entries: {} items, {} trinkets, {} achievements, {} bosses, {} challenges, {} characters, {} transformations",
        c.items,
        c.trinkets,
        c.achievements,
        c.bosses,
        c.challenges,
        c.characters,
        c.transformations
    );
    println!("snapshotAt: {} (max revid {})", m.snapshot_at, m.max_revid);
    match &m.last_known_patch {
        Some(p) => println!("last known patch: {} ({})", p.number, p.date),
        None => println!("last known patch: none"),
    }
    println!("pages without id: {}", m.diagnostics.pages_without_id);
    println!(
        "transformations whose two item lists disagree: {}",
        m.diagnostics.transformation_sources_disagree
    );
    print_diagnostic("unresolved references", &m.diagnostics.unresolved);
    print_diagnostic("unknown templates", &m.diagnostics.unknown_templates);
    print_diagnostic("discarded sections", &m.diagnostics.discarded_sections);
}

fn build_dataset(raw_dir: &Path, out: &Path, corrections_path: &Path) -> Outcome {
    let raw = Raw::load(raw_dir).map_err(|e| Failure::Error(e.to_string()))?;
    let corrections = load_corrections(corrections_path)?;
    let ds = build(&raw, &corrections);
    let changed = write_if_changed(out, ds.to_json().as_bytes()).map_err(|e| io_error(out, e))?;
    println!(
        "{}: {}",
        out.display(),
        if changed { "written" } else { "unchanged" }
    );
    print_meta(&ds);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn options_are_validated() {
        let args = |s: &[&str]| s.iter().map(|a| a.to_string()).collect::<Vec<_>>();
        let opts = parse_options(&args(&["--out", "x"]), &["out"]).unwrap();
        assert_eq!(opts.get("out").map(String::as_str), Some("x"));
        assert!(parse_options(&args(&["--nope", "x"]), &["out"]).is_err());
        assert!(parse_options(&args(&["--out"]), &["out"]).is_err());
        assert!(parse_options(&args(&["--out", "a", "--out", "b"]), &["out"]).is_err());
        assert!(matches!(
            run(&args(&["frobnicate"])),
            Err(Failure::Usage(_))
        ));
        assert!(matches!(run(&[]), Err(Failure::Usage(_))));
    }

    #[test]
    fn page_bytes_normalise_line_endings() {
        assert_eq!(page_bytes("a\r\nb\n"), b"a\nb\n");
    }
}
