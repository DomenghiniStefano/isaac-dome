//! `wiki-snapshot`: a developer tool, never shipped. `fetch` downloads the pages and
//! Cargo tables from bindingofisaacrebirth.wiki.gg into `dataset/raw/`, the only place
//! in the repo that talks to the network; `build` turns that snapshot into
//! `dataset/wiki/`, one file per collection, which the `wiki` crate merges and embeds
//! into the binary.

mod admit;
mod api;
mod fetch;
mod http;
mod namespace;
mod store;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use wiki::{build, dead_links, Corrections, Dataset, Raw};

const USAGE: &str =
    "usage:\n  wiki-snapshot fetch [--out <dir>]\n  wiki-snapshot build [--raw <dir>] [--out <dir>]";
/// How many entries of each diagnostic map `build` shows.
const DIAGNOSTIC_ROWS: usize = 20;

/// How the program ends when things don't go well: a usage error (exit 2) or a
/// network or disk one (exit 1).
#[derive(Debug)]
pub(crate) enum Failure {
    Usage(String),
    Error(String),
}

pub(crate) type Outcome = Result<(), Failure>;

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
            fetch::fetch(&out)
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
                .unwrap_or_else(|| root.join("dataset").join("wiki"));
            build_dataset(&raw, &out, &root.join("dataset").join("corrections.json"))
        }
        Some(other) => Err(Failure::Usage(format!("unknown command: {other}"))),
        None => Err(Failure::Usage("missing command".into())),
    }
}

pub(crate) fn io_error(what: &Path, e: std::io::Error) -> Failure {
    Failure::Error(format!("{}: {e}", what.display()))
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
        "entries: {} items, {} trinkets, {} achievements, {} bosses, {} challenges, {} characters, {} transformations, {} entities, {} articles",
        c.items,
        c.trinkets,
        c.achievements,
        c.bosses,
        c.challenges,
        c.characters,
        c.transformations,
        c.entities,
        c.articles
    );
    println!("snapshotAt: {} (max revid {})", m.snapshot_at, m.max_revid);
    match &m.last_known_patch {
        Some(p) => println!("last known patch: {} ({})", p.number, p.date),
        None => println!("last known patch: none"),
    }
    // Every counter, destructured **without `..`**, so a counter added later breaks the build
    // until it is printed: named by hand, five of the ten went unprinted.
    let wiki::Diagnostics {
        unresolved,
        unknown_templates,
        discarded_sections,
        pages_without_id,
        orphan_closers,
        unknown_dlc_codes,
        transformation_sources_disagree,
        unknown_entities,
        unknown_infoboxes,
        spans_outside_their_page,
    } = &m.diagnostics;
    println!("pages without id: {pages_without_id}");
    println!("transformations whose two item lists disagree: {transformation_sources_disagree}");
    println!("orphan closers: {orphan_closers}");
    println!("spans outside their page: {spans_outside_their_page}");
    print_diagnostic("unresolved references", unresolved);
    print_diagnostic("unknown templates", unknown_templates);
    print_diagnostic("discarded sections", discarded_sections);
    print_diagnostic("unknown dlc codes", unknown_dlc_codes);
    print_diagnostic("unknown entities", unknown_entities);
    print_diagnostic("unknown infoboxes", unknown_infoboxes);
    // Not part of `meta.diagnostics`: whether a `Ref`'s target has a page is a fact about
    // the whole dataset, not about the page one is printed from, and it never ships in
    // `dataset/wiki/` — the destinations are the parser's own maintenance concern, not the
    // app's at runtime.
    let links = dead_links(ds);
    print_diagnostic("dead concept links", &links.concept_pages);
    print_diagnostic("unopenable refs (id with no page)", &links.unopenable_refs);
}

fn build_dataset(raw_dir: &Path, out: &Path, corrections_path: &Path) -> Outcome {
    let raw = Raw::load(raw_dir).map_err(|e| Failure::Error(e.to_string()))?;
    let corrections = load_corrections(corrections_path)?;
    let ds = build(&raw, &corrections);
    let report = ds.write_dir(out).map_err(|e| io_error(out, e))?;
    for (file, written) in &report {
        println!(
            "{}: {}",
            out.join(file).display(),
            if *written { "written" } else { "unchanged" }
        );
    }
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
}
