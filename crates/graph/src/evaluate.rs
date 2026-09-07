//! The walk against a profile. Pure arithmetic over the edges built in `build.rs`: the
//! save reaches this file only as a slice of booleans.

use std::collections::{BTreeMap, BTreeSet};

use crate::build::{Graph, GraphDiagnostic};

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
    pub fn evaluate(&self, flags: Option<&[bool]>) -> Eval {
        let Some(flags) = flags else {
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
            let info = match (unproven, transitive_known) {
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
