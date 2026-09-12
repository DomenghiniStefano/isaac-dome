#![allow(dead_code)] // one test binary today; the module is shaped for the next one too

//! Shared setup for the real-data tests. Every function says on stderr which slice of the
//! real domain it ran on, or why it skipped: a green suite that skipped everything is the
//! failure mode this discipline exists to prevent.

use catalog::Catalog;
use core_save::{Kind, Save};
use graph::Graph;

/// "a requires b" over the graph's transitive prerequisites, with the chains computed once
/// for the rows involved. The same shape the app uses; three lines rather than a dependency
/// on the Tauri crate, which nothing depends on.
pub struct GraphDeps {
    chains: std::collections::BTreeMap<u32, std::collections::BTreeSet<u32>>,
}

impl GraphDeps {
    pub fn new(g: &Graph, flags: Option<&[bool]>, rows: &[u32]) -> GraphDeps {
        GraphDeps {
            chains: rows
                .iter()
                .map(|a| {
                    (
                        *a,
                        g.missing_chain(*a, &graph::FlagsOnly(flags))
                            .into_iter()
                            .collect(),
                    )
                })
                .collect(),
        }
    }
}

impl plan::Dependencies for GraphDeps {
    fn requires(&self, a: u32, b: u32) -> bool {
        self.chains.get(&a).is_some_and(|c| c.contains(&b))
    }
}

pub fn real_graph_and_flags() -> Option<(Graph, Vec<bool>)> {
    let Some(packed) = test_support::packed_dir() else {
        test_support::skip("samples/packed missing: no game to build the catalog from");
        return None;
    };
    let rs = unpack::ResourceSet::open(&packed);
    let catalog = Catalog::build(|p| rs.read(p));
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
    let rules = match graph::rules::embedded() {
        Ok(r) => r,
        // Not a skip: the rules are compiled in, so this can only be our own broken file.
        Err(e) => panic!("the embedded rules must parse: {e}"),
    };
    eprintln!("sample: {}", save.display());
    Some((Graph::build(&catalog, rules), flags))
}
