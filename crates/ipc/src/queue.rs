//! The plan queue as the UI sees it: rows already resolved to nodes, and every reason a
//! row is missing said out loud.

use catalog::Catalog;
use serde::Serialize;
use wiki::Dataset;

use crate::graph::{unlock_view, AchievementRef, UnlockNode};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct QueueView {
    pub rows: Vec<QueueRow>,
    pub diagnostics: Vec<QueueDiagnostic>,
    /// `false` when `store` won't open: the queue can't be seen or changed, and the UI
    /// says so instead of showing an empty list.
    pub store_available: bool,
}

/// A queue row is an **achievement**, not a target: wanting Tainted Lost and wanting the
/// achievement that unlocks it are the same wish seen from two sides.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct QueueRow {
    /// The same node the Unlock screen draws, so the two can never disagree.
    pub node: UnlockNode,
    /// You asked for this one, for itself.
    pub wanted: bool,
    /// The wanted achievements whose chain passes through this row.
    pub origins: Vec<u32>,
    /// Prerequisites this row still needs that are **not** in the queue.
    pub steps_not_queued: u32,
}

/// Every way a row can be absent, said out loud. `Unreadable` and an empty queue are
/// different things, and so are `Completed` and a row that just vanished.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum QueueDiagnostic {
    /// The database failed to open, and which case it is: a variant, never a sentence,
    /// and never SQLite's own message.
    StoreUnavailable { reason: crate::StoreReason },
    /// The saved document didn't parse: the queue is empty because it couldn't be read,
    /// which is not the same as being empty.
    Unreadable,
    /// Rows left out of this read because the profile has completed them. A row never
    /// vanishes without a word.
    Completed { count: u32, wanted: Vec<u32> },
    /// A row whose achievement this catalog no longer knows — an older edition, or a patch
    /// that removed it. It stays in the file and is named by id.
    Unresolved { achievement: u32 },
    /// Goals saved before the queue existed, not yet imported. One action moves them in;
    /// nothing happens on its own, because a read never writes.
    GoalsPending { count: u32 },
    /// No catalog: the game isn't installed, so no row resolves to anything.
    NoCatalog,
}

/// Every achievement whose `unlocks` names this target, in the catalog's order.
///
/// The inverse of `Catalog::unlocks`. A list and not an `Option`: a challenge's
/// `unlocked_by` is a list in the game's own file, so two ways in is a shape the data has,
/// and picking one silently is a wrong answer wearing a right one's clothes.
pub fn achievements_unlocking(c: &Catalog, key: &crate::goals::TargetKey) -> Vec<u32> {
    use crate::catalog_view::item_kind;
    use crate::goals::TargetKey;
    use catalog::Unlock;
    c.achievements()
        .filter(|a| {
            c.unlocks(a.id).iter().any(|u| match (u, key) {
                (
                    Unlock::Item { kind, id },
                    TargetKey::Item {
                        item_kind: k,
                        id: want,
                    },
                ) => item_kind(*k) == *kind && id.0 == *want,
                (Unlock::Character { id }, TargetKey::Character { id: want }) => id.0 == *want,
                (Unlock::Boss { id }, TargetKey::Boss { id: want }) => id.0 == *want,
                (Unlock::Challenge { id }, TargetKey::Challenge { id: want }) => id.0 == *want,
                // A pair of enums has sixteen combinations of which four mean anything.
                // The exhaustiveness rule bans a catch-all that hides a new variant of one
                // closed enum; this one hides nothing — the four are written out above it.
                _ => false,
            })
        })
        .map(|a| a.id.0)
        .collect()
}

/// The first way in, which is all the goals import needs: it resolves a goal to one queue
/// row. Anything that has to *show* the ways uses `achievements_unlocking`.
pub fn achievement_unlocking(c: &Catalog, key: &crate::goals::TargetKey) -> Option<u32> {
    achievements_unlocking(c, key).first().copied()
}

/// Everything `queue_view` needs. A struct rather than eight parameters: past seven
/// `clippy::too_many_arguments` objects, and a list that long is hard to call correctly
/// anyway.
pub struct QueueInputs<'a> {
    pub catalog: Option<&'a Catalog>,
    /// The embedded wiki dataset, for the page a requirement links to. `None` links nothing.
    pub dataset: Option<&'a Dataset>,
    pub flags: Option<&'a [bool]>,
    pub graph: Option<&'a graph::Graph>,
    pub eval: Option<&'a graph::evaluate::Eval>,
    /// What the save says about marks and tallies, for the requirements the graph now
    /// answers from the profile. `None` draws none of them — which is right only when
    /// `eval` was built without one too, or the queue and Unlock would disagree.
    pub progress: Option<&'a dyn graph::Profile>,
    pub queue: Result<&'a plan::Queue, &'a plan::QueueError>,
    pub goals_pending: u32,
    /// `Some` when the database itself failed, as the case it is: never a sentence, and
    /// never SQLite's own message.
    pub store_reason: Option<crate::StoreReason>,
}

pub fn queue_view(
    inputs: QueueInputs<'_>,
    icon: impl FnMut(&crate::IconRef) -> Option<String>,
) -> QueueView {
    let QueueInputs {
        catalog,
        dataset,
        flags,
        graph,
        eval,
        progress,
        queue,
        goals_pending,
        store_reason,
    } = inputs;
    let mut diagnostics = Vec::new();
    let store_available = store_reason.is_none();
    if let Some(reason) = store_reason {
        diagnostics.push(QueueDiagnostic::StoreUnavailable { reason });
    }
    if goals_pending > 0 {
        diagnostics.push(QueueDiagnostic::GoalsPending {
            count: goals_pending,
        });
    }
    let empty = |diagnostics| QueueView {
        rows: Vec::new(),
        diagnostics,
        store_available,
    };
    let queue = match queue {
        Ok(q) => q,
        Err(_) => {
            // Unreadable is not empty, and saying which is the whole point.
            diagnostics.push(QueueDiagnostic::Unreadable);
            return empty(diagnostics);
        }
    };
    let Some(c) = catalog else {
        diagnostics.push(QueueDiagnostic::NoCatalog);
        return empty(diagnostics);
    };

    // One `unlock_view`, indexed by achievement: a queue row shows **the same node** the
    // Unlock screen shows, so the two can never drift apart.
    let view = unlock_view(Some(c), dataset, flags, graph, eval, progress, icon);
    let by_id: std::collections::BTreeMap<u32, &UnlockNode> = view
        .nodes
        .iter()
        .filter_map(|n| match &n.achievement {
            AchievementRef::Known { id, .. } => Some((*id, n)),
            AchievementRef::Unknown { .. } => None,
        })
        .collect();
    let queued: std::collections::BTreeSet<u32> =
        queue.rows().iter().map(|r| r.achievement).collect();

    let mut rows = Vec::new();
    let mut completed = 0u32;
    let mut completed_wanted = Vec::new();
    for r in queue.rows() {
        let Some(node) = by_id.get(&r.achievement) else {
            diagnostics.push(QueueDiagnostic::Unresolved {
                achievement: r.achievement,
            });
            continue;
        };
        if node.done {
            completed += 1;
            if r.wanted {
                completed_wanted.push(r.achievement);
            }
            continue;
        }
        let steps_not_queued = graph
            .zip(flags)
            .map(|(g, f)| {
                g.missing_chain(r.achievement, &graph::FlagsOnly(Some(f)))
                    .iter()
                    .filter(|id| !queued.contains(id))
                    .count() as u32
            })
            .unwrap_or(0);
        rows.push(QueueRow {
            node: (*node).clone(),
            wanted: r.wanted,
            origins: r.origins.clone(),
            steps_not_queued,
        });
    }
    if completed > 0 {
        diagnostics.push(QueueDiagnostic::Completed {
            count: completed,
            wanted: completed_wanted,
        });
    }
    QueueView {
        rows,
        diagnostics,
        store_available,
    }
}

/// What the queue's ordering rule asks the graph, answered from a table instead of a walk.
///
/// The chains are computed **once, for the rows involved**, and not per question: a move
/// asks `requires` twice per row, and each answer would otherwise be a fresh transitive
/// walk over the whole graph.
///
/// A node the graph can't compute has an empty chain, so it is never dragged and never
/// walls — the spec's "rows the graph can't compute carry no constraints", expressed once,
/// here. It lives in this crate rather than in the Tauri one because it is a rule with a
/// return value worth checking, and the Tauri crate is wiring and isn't tested.
pub struct GraphDeps {
    chains: std::collections::BTreeMap<u32, std::collections::BTreeSet<u32>>,
}

impl GraphDeps {
    /// The chains as the graph gives them, for the rows a move involves.
    pub fn new(g: &graph::Graph, flags: Option<&[bool]>, rows: &[u32]) -> GraphDeps {
        GraphDeps::from_chains(
            rows.iter()
                .map(|a| (*a, g.missing_chain(*a, &graph::FlagsOnly(flags)))),
        )
    }

    /// The same table without a graph: the part worth checking, and what the tests use.
    pub fn from_chains(chains: impl IntoIterator<Item = (u32, Vec<u32>)>) -> GraphDeps {
        GraphDeps {
            chains: chains
                .into_iter()
                .map(|(a, c)| (a, c.into_iter().collect()))
                .collect(),
        }
    }
}

impl plan::Dependencies for GraphDeps {
    fn requires(&self, a: u32, b: u32) -> bool {
        self.chains.get(&a).is_some_and(|c| c.contains(&b))
    }
}
