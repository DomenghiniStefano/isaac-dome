#![allow(dead_code)] // each test binary uses part of this module; the rest is not dead

//! Shared setup for the real-data tests. Every function says on stderr which slice of the
//! real domain it ran on, or why it skipped: a green suite that skipped everything is the
//! failure mode this crate exists to prevent.

use std::collections::BTreeMap;

use catalog::Catalog;
use core_save::{Kind, Save};
use graph::evaluate::NodeInfo;
use graph::Graph;

/// The real catalog, built from the game's archives.
pub fn real_catalog() -> Option<(Catalog, unpack::ResourceSet)> {
    let Some(packed) = test_support::packed_dir() else {
        test_support::skip("samples/packed missing: no game to build the catalog from");
        return None;
    };
    let rs = unpack::ResourceSet::open(&packed);
    let c = Catalog::build(|p| rs.read(p));
    Some((c, rs))
}

fn embedded_rules() -> &'static graph::Rules {
    match graph::rules::embedded() {
        Ok(r) => r,
        // Not a skip: the rules are compiled in, so this can only be our own broken file.
        Err(e) => panic!("the embedded rules must parse: {e}"),
    }
}

pub fn real_graph() -> Option<(Catalog, Graph)> {
    let (catalog, _rs) = real_catalog()?;
    let g = Graph::build(&catalog, embedded_rules());
    Some((catalog, g))
}

pub fn real_graph_and_flags() -> Option<(Graph, Vec<bool>)> {
    let (catalog, _rs) = real_catalog()?;
    let series = test_support::dated_series("rep+persistentgamedata1.dat");
    let Some(save) = series.last() else {
        test_support::skip("no dated save in samples/: nothing to evaluate against");
        return None;
    };
    let s = match Save::open(save) {
        Ok(s) => s,
        Err(_) => {
            // A sample that is present but unreadable is declared, never a silent skip.
            test_support::skip("the most recent dated save exists but doesn't read");
            return None;
        }
    };
    let Some(flags) = s.flags(Kind::Achievements) else {
        test_support::skip("section 1 missing from the most recent dated save");
        return None;
    };
    eprintln!("sample: {}", save.display());
    Some((Graph::build(&catalog, embedded_rules()), flags))
}

/// One entry per dated save, oldest first: file name, the evaluation, and the flags.
/// Skips when fewer than two eras are present — a comparison needs two.
/// A profile at one moment: the file it came from, its evaluation, and its flags.
pub type Era = (String, BTreeMap<u32, NodeInfo>, Vec<bool>);

pub fn series_evals() -> Option<Vec<Era>> {
    let (catalog, _rs) = real_catalog()?;
    let g = Graph::build(&catalog, embedded_rules());
    let mut out = Vec::new();
    for path in test_support::dated_series("rep+persistentgamedata1.dat") {
        let Ok(s) = Save::open(&path) else {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            test_support::skip(&format!("{name} is in the series and does not open"));
            continue;
        };
        let Some(flags) = s.flags(Kind::Achievements) else {
            continue;
        };
        let e = g.evaluate(&graph::FlagsOnly(Some(&flags)));
        let infos: BTreeMap<u32, NodeInfo> = g
            .nodes()
            .iter()
            .filter_map(|n| e.node(n.achievement).map(|i| (n.achievement, i.clone())))
            .collect();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        out.push((name, infos, flags));
    }
    eprintln!("sample: {} dated saves in the series", out.len());
    if out.len() < 2 {
        test_support::skip("fewer than two dated saves: the series properties need two eras");
        return None;
    }
    Some(out)
}
