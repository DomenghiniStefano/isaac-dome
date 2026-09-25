//! The plan queue as the UI sees it: rows already resolved to nodes, and every reason a
//! row is missing said out loud.

use std::collections::{BTreeMap, BTreeSet};

use catalog::{AchievementId, Catalog};
use serde::Serialize;
use wiki::Dataset;

use crate::graph::{unlock_view, AchievementRef, UnlockInputs, UnlockNode};
use crate::target_sprite::BossKeys;

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
    use crate::goals::TargetKey;
    use catalog::Unlock;
    c.achievements()
        .filter(|a| {
            // Exhaustive on `Unlock`, so a new kind of unlock breaks the build here instead of
            // matching nothing; the key is then only asked whether it names the same thing.
            c.unlocks(a.id).iter().any(|u| match u {
                Unlock::Item { kind, id } => matches!(
                    key,
                    TargetKey::Item { item_kind: k, id: want }
                        if *k == *kind && id.0 == *want
                ),
                Unlock::Character { id } => {
                    matches!(key, TargetKey::Character { id: want } if id.0 == *want)
                }
                Unlock::Boss { id } => matches!(key, TargetKey::Boss { id: want } if id.0 == *want),
                Unlock::Challenge { id } => {
                    matches!(key, TargetKey::Challenge { id: want } if id.0 == *want)
                }
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
    /// The catalog's boss keys (`boss_keys`), settled once beside it.
    pub bosses: &'a BossKeys,
    /// The embedded wiki dataset, for the page a requirement links to. `None` links nothing.
    pub dataset: Option<&'a Dataset>,
    pub flags: Option<&'a [bool]>,
    pub graph: Option<&'a graph::build::Graph>,
    pub eval: Option<&'a graph::evaluate::Eval>,
    /// What the save says about marks and tallies, for the requirements the graph now
    /// answers from the profile. `None` draws none of them — which is right only when
    /// `eval` was built without one too, or the queue and Unlock would disagree.
    pub progress: Option<&'a dyn graph::evaluate::Profile>,
    pub queue: Result<&'a plan::Queue, &'a plan::QueueError>,
    pub goals_pending: u32,
    /// `Some` when the database itself failed, as the case it is: never a sentence, and
    /// never SQLite's own message.
    pub store_reason: Option<crate::StoreReason>,
}

/// The queue as the UI sees it. The diagnostics come out in a fixed order: what is known before
/// the queue is read — the store, the goals still to import — then whatever reading it met.
pub fn queue_view(
    inputs: QueueInputs<'_>,
    icon: impl FnMut(&crate::IconRef) -> Option<String>,
) -> QueueView {
    let store_available = inputs.store_reason.is_none();
    let before = inputs
        .store_reason
        .map(|reason| QueueDiagnostic::StoreUnavailable { reason })
        .into_iter()
        .chain(
            (inputs.goals_pending > 0).then_some(QueueDiagnostic::GoalsPending {
                count: inputs.goals_pending,
            }),
        );
    let (rows, read) = match (inputs.queue, inputs.catalog) {
        // Unreadable is not empty, and saying which is the whole point.
        (Err(_), _) => (Vec::new(), vec![QueueDiagnostic::Unreadable]),
        (Ok(_), None) => (Vec::new(), vec![QueueDiagnostic::NoCatalog]),
        (Ok(queue), Some(c)) => resolved_rows(&inputs, c, queue, icon),
    };
    QueueView {
        rows,
        diagnostics: before.chain(read).collect(),
        store_available,
    }
}

/// The rows of a readable queue against a catalog, and every row that did not make it: one
/// `Unresolved` per row the catalog does not know, in the queue's order, then the completed
/// ones as one count.
fn resolved_rows(
    inputs: &QueueInputs<'_>,
    c: &Catalog,
    queue: &plan::Queue,
    icon: impl FnMut(&crate::IconRef) -> Option<String>,
) -> (Vec<QueueRow>, Vec<QueueDiagnostic>) {
    // One `unlock_view`, indexed by achievement: a queue row shows **the same node** the
    // Unlock screen shows, so the two can never drift apart.
    let view = unlock_view(
        UnlockInputs {
            catalog: Some(c),
            bosses: inputs.bosses,
            dataset: inputs.dataset,
            flags: inputs.flags,
            graph: inputs.graph,
            eval: inputs.eval,
            progress: inputs.progress,
        },
        icon,
    );
    let by_id: BTreeMap<u32, &UnlockNode> = view
        .nodes
        .iter()
        .filter_map(|n| match &n.achievement {
            AchievementRef::Known { id, .. } => Some((*id, n)),
            AchievementRef::Unknown { .. } => None,
        })
        .collect();
    let queued: BTreeSet<AchievementId> = queue.rows().iter().map(|r| r.achievement).collect();
    let placed: Vec<(&plan::Row, Option<&UnlockNode>)> = queue
        .rows()
        .iter()
        .map(|r| (r, by_id.get(&r.achievement.0).copied()))
        .collect();

    let rows = placed
        .iter()
        .filter_map(|(r, node)| node.filter(|n| !n.done).map(|n| (*r, n)))
        .map(|(r, node)| QueueRow {
            node: node.clone(),
            wanted: r.wanted,
            origins: r.origins.iter().map(|a| a.0).collect(),
            steps_not_queued: steps_not_queued(inputs, r.achievement, &queued),
        })
        .collect();
    let unresolved = placed
        .iter()
        .filter(|(_, node)| node.is_none())
        .map(|(r, _)| QueueDiagnostic::Unresolved {
            achievement: r.achievement.0,
        });
    let completed: Vec<&plan::Row> = placed
        .iter()
        .filter(|(_, node)| node.is_some_and(|n| n.done))
        .map(|(r, _)| *r)
        .collect();
    let diagnostics = unresolved.chain(completed_diagnostic(&completed)).collect();
    (rows, diagnostics)
}

/// The rows left out because the profile has completed them, said once: a row never vanishes
/// without a word.
fn completed_diagnostic(completed: &[&plan::Row]) -> Option<QueueDiagnostic> {
    (!completed.is_empty()).then(|| QueueDiagnostic::Completed {
        count: completed.len() as u32,
        wanted: completed
            .iter()
            .filter(|r| r.wanted)
            .map(|r| r.achievement.0)
            .collect(),
    })
}

/// The prerequisites this row still needs that the queue does not hold. Zero without a graph
/// or without section 1, which have nothing to count from.
fn steps_not_queued(
    inputs: &QueueInputs<'_>,
    achievement: AchievementId,
    queued: &BTreeSet<AchievementId>,
) -> u32 {
    inputs
        .graph
        .zip(inputs.flags)
        .map(|(g, f)| {
            g.missing_chain(achievement, &graph::evaluate::FlagsOnly(Some(f)))
                .iter()
                .filter(|id| !queued.contains(id))
                .count() as u32
        })
        .unwrap_or(0)
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
    chains: std::collections::BTreeMap<AchievementId, std::collections::BTreeSet<AchievementId>>,
}

impl GraphDeps {
    /// The chains as the graph gives them, for the rows a move involves.
    pub fn new(
        g: &graph::build::Graph,
        flags: Option<&[bool]>,
        rows: &[AchievementId],
    ) -> GraphDeps {
        GraphDeps::from_chains(
            rows.iter()
                .map(|a| (*a, g.missing_chain(*a, &graph::evaluate::FlagsOnly(flags)))),
        )
    }

    /// The same table without a graph: the part worth checking, and what the tests use.
    pub fn from_chains(
        chains: impl IntoIterator<Item = (AchievementId, Vec<AchievementId>)>,
    ) -> GraphDeps {
        GraphDeps {
            chains: chains
                .into_iter()
                .map(|(a, c)| (a, c.into_iter().collect()))
                .collect(),
        }
    }
}

impl plan::Dependencies for GraphDeps {
    fn requires(&self, a: AchievementId, b: AchievementId) -> bool {
        self.chains.get(&a).is_some_and(|c| c.contains(&b))
    }
}

/// How many goals nothing in the queue stands for yet. A goal whose target no achievement
/// unlocks can never be stood for, so it stays pending — the queue has no row that would mean
/// it.
pub fn goals_pending(
    c: &Catalog,
    goals: &[crate::goals::Goal],
    queued: &std::collections::BTreeSet<AchievementId>,
) -> u32 {
    goals
        .iter()
        .filter(|g| {
            achievement_unlocking(c, &g.target).is_none_or(|a| !queued.contains(&AchievementId(a)))
        })
        .count() as u32
}
