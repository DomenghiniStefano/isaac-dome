//! The walk against a profile. Pure arithmetic over the edges built in `build.rs`: the
//! save reaches this file only as a slice of booleans.

use std::collections::{BTreeMap, BTreeSet};

use catalog::CharacterId;

use crate::build::{Graph, GraphDiagnostic};
use crate::model::Requirement;
use crate::rules::{CounterName, MarkColumn, MarkLevel};

/// What the graph is allowed to ask a save. Three questions, all already resolved by the
/// caller: this crate names a column, a level and a tally, and never an index.
///
/// The double `Option` on `mark` is the point, not clumsiness. One `None` cannot mean both
/// "this cell isn't located" and "this cell is at zero": the first has to make the node
/// `Partial`, the second has to leave it computed and unmet. `counter` has the same shape
/// for free — `None` unread, `Some(0)` read and zero.
pub trait Profile {
    fn done(&self) -> Option<&[bool]>;
    /// `None` — cannot say: the cell isn't located, or section 2 wasn't read.
    /// `Some(None)` — located and read, nothing reached yet.
    /// `Some(Some(level))` — the highest level reached.
    fn mark(&self, character: CharacterId, column: MarkColumn) -> Option<Option<MarkLevel>>;
    /// `None` when section 2 wasn't read. Not `Some(0)`: an unread tally is not a zero one.
    fn counter(&self, name: CounterName) -> Option<u32>;
}

/// A caller that has the achievement flags and nothing else. It answers "I can't say" to
/// the other two questions, so every node holding a mark or a tally stays `Partial` — the
/// honest outcome, and what stops a half-wired caller from claiming progress it never read.
pub struct FlagsOnly<'a>(pub Option<&'a [bool]>);

impl Profile for FlagsOnly<'_> {
    fn done(&self) -> Option<&[bool]> {
        self.0
    }
    fn mark(&self, _: CharacterId, _: MarkColumn) -> Option<Option<MarkLevel>> {
        None
    }
    fn counter(&self, _: CounterName) -> Option<u32> {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeInfo {
    Computed {
        available_now: bool,
        blocked_by: u32,
        fan_out: u32,
        steps_missing: u32,
    },
    /// Requirements only partly interpreted, or a node caught in a cycle. It carries no
    /// `steps_missing`: with something uninterpreted the transitive count isn't knowable,
    /// and a zero there would be the exact lie this variant exists to prevent.
    Partial {
        blocked_by: u32,
        fan_out: u32,
        unknown: u32,
    },
}

pub struct Eval {
    infos: BTreeMap<u32, NodeInfo>,
    diagnostics: Vec<GraphDiagnostic>,
}

impl Eval {
    pub fn node(&self, achievement: u32) -> Option<&NodeInfo> {
        self.infos.get(&achievement)
    }

    pub fn diagnostics(&self) -> &[GraphDiagnostic] {
        &self.diagnostics
    }
}

impl Graph {
    pub fn evaluate(&self, profile: &dyn Profile) -> Eval {
        let Some(flags) = profile.done() else {
            // Section 1 wasn't read. No nodes: "unread" must not become "not done".
            return Eval {
                infos: BTreeMap::new(),
                diagnostics: Vec::new(),
            };
        };
        // A slot the file doesn't reach is not done. The other reading would claim
        // progress the save doesn't contain.
        let done = |id: u32| flags.get(id as usize).copied().unwrap_or(false);

        // A gate the graph can't express is passed when the profile has already earned an
        // achievement that carries it: if "beat Delirium with Isaac" is done, Delirium is
        // reachable for this player, whatever gates it. That is evidence read from the
        // save, not an optimistic guess, and it is what keeps the late game countable —
        // on a real profile it takes the uninterpreted requirements from 210 to 6.
        let mut evidence: BTreeMap<&str, u32> = BTreeMap::new();
        for n in self.nodes() {
            if done(n.achievement) {
                for label in &n.unknown {
                    *evidence.entry(label.as_str()).or_insert(0) += 1;
                }
            }
        }

        let mut fan_out: BTreeMap<u32, u32> = BTreeMap::new();
        for n in self.nodes() {
            for &p in &n.prerequisites {
                *fan_out.entry(p).or_insert(0) += 1;
            }
        }

        let mut missing: BTreeMap<u32, Option<BTreeSet<u32>>> = BTreeMap::new();
        let mut cycles: Vec<Vec<u32>> = Vec::new();
        for n in self.nodes() {
            let mut stack = Vec::new();
            transitive(
                self,
                n.achievement,
                &done,
                &mut missing,
                &mut stack,
                &mut cycles,
            );
        }

        let mut infos = BTreeMap::new();
        for n in self.nodes() {
            let id = n.achievement;
            let blocked_by = n.prerequisites.iter().filter(|&&p| !done(p)).count() as u32;
            let fan = fan_out.get(&id).copied().unwrap_or(0);
            let transitive_known = missing.get(&id).and_then(|m| m.as_ref());
            let unproven = n
                .unknown
                .iter()
                .filter(|l| !evidence.contains_key(l.as_str()))
                .count() as u32;
            // A requirement this profile cannot answer joins the uninterpreted ones. An
            // unread section 2 and one of the 40 unlocated cells are both "we can't say",
            // and neither is allowed to read as satisfied — which is what would happen if
            // an unanswerable mark simply fell out of the count.
            let unanswerable = n
                .requirements
                .iter()
                .filter(|r| match r {
                    Requirement::Mark {
                        character, column, ..
                    } => profile.mark(*character, *column).is_none(),
                    Requirement::Counter { name, .. } => profile.counter(*name).is_none(),
                    Requirement::Character { .. }
                    | Requirement::Boss { .. }
                    | Requirement::Challenge { .. }
                    | Requirement::Item { .. }
                    | Requirement::Gate { .. }
                    | Requirement::Unknown { .. }
                    | Requirement::None => false,
                })
                .count() as u32;
            let info = match (unproven + unanswerable, transitive_known) {
                (0, Some(set)) => NodeInfo::Computed {
                    available_now: blocked_by == 0 && !done(id),
                    blocked_by,
                    fan_out: fan,
                    steps_missing: set.len() as u32,
                },
                // Either a requirement wasn't interpreted, or the node sits in a cycle and
                // the transitive count can't be taken. Both are "we can't say".
                (unknown, _) => NodeInfo::Partial {
                    blocked_by,
                    fan_out: fan,
                    unknown: unknown.max(1),
                },
            };
            infos.insert(id, info);
        }

        let mut diagnostics = self.diagnostics().to_vec();
        for nodes in cycles {
            diagnostics.push(GraphDiagnostic::Cycle { nodes });
        }
        // The inference is declared, never silent: whoever reads the screen can see that
        // the app decided a gate was passed, and on what evidence.
        for (label, done) in evidence {
            diagnostics.push(GraphDiagnostic::GateSatisfiedByEvidence {
                label: label.to_string(),
                done,
            });
        }
        Eval { infos, diagnostics }
    }

    /// The not-done achievements standing between the profile and this node, transitively,
    /// in ascending id order.
    ///
    /// Empty when the node is done, when it is available now, when the graph can't say,
    /// and when the node isn't in the graph at all — four different situations that the
    /// caller tells apart from `NodeInfo`, not from this list. It answers one question:
    /// what would still have to be earned.
    pub fn missing_chain(&self, achievement: u32, profile: &dyn Profile) -> Vec<u32> {
        let Some(flags) = profile.done() else {
            return Vec::new();
        };
        let done = |id: u32| flags.get(id as usize).copied().unwrap_or(false);
        let mut memo = BTreeMap::new();
        let mut stack = Vec::new();
        let mut cycles = Vec::new();
        transitive(self, achievement, &done, &mut memo, &mut stack, &mut cycles)
            .map(|set| set.into_iter().collect())
            .unwrap_or_default()
    }
}

/// The set of not-done achievements standing between the profile and this node. `None`
/// means "not knowable": the node sits in a cycle. Memoized, so each node is computed once
/// even when many nodes share an ancestor.
fn transitive(
    g: &Graph,
    id: u32,
    done: &impl Fn(u32) -> bool,
    memo: &mut BTreeMap<u32, Option<BTreeSet<u32>>>,
    stack: &mut Vec<u32>,
    cycles: &mut Vec<Vec<u32>>,
) -> Option<BTreeSet<u32>> {
    if let Some(hit) = memo.get(&id) {
        return hit.clone();
    }
    // Done owes nothing. The same rule as for a done prerequisite, applied to the node
    // itself: an achievement already earned cannot be N runs away from you.
    if done(id) {
        memo.insert(id, Some(BTreeSet::new()));
        return Some(BTreeSet::new());
    }
    if stack.contains(&id) {
        let mut nodes = stack.clone();
        nodes.push(id);
        cycles.push(nodes);
        memo.insert(id, None);
        return None;
    }
    stack.push(id);
    let mut set = BTreeSet::new();
    let mut knowable = true;
    if let Some(node) = g.node(id) {
        for &p in &node.prerequisites {
            // A done prerequisite ends the walk. Whatever the graph thinks it needed is
            // irrelevant — you already have it — and recursing anyway would count
            // achievements the profile can never owe again. The graph is inferred, the
            // save is fact: where they disagree, the save wins.
            if done(p) {
                continue;
            }
            set.insert(p);
            match transitive(g, p, done, memo, stack, cycles) {
                Some(inner) => set.extend(inner),
                None => knowable = false,
            }
        }
    }
    stack.pop();
    let out = if knowable { Some(set) } else { None };
    memo.insert(id, out.clone());
    out
}
