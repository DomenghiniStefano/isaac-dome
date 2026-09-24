//! The graph itself: one node per achievement the catalog knows, with the achievements it
//! sits behind. Edges come from the game's own `unlocked_by` links, never from the wiki —
//! the wiki only says *what* is needed.

use catalog::{AchievementId, Catalog, CharacterId};

use crate::model::Requirement;
use crate::resolve::{requirement_with, NameIndex};
use crate::rules::{Rules, Verdict};
use wiki::Target;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub achievement: AchievementId,
    pub requirements: Vec<Requirement>,
    /// Achievement ids this node sits behind. Sorted and deduplicated: two refs naming the
    /// same prerequisite are one run, not two.
    pub prerequisites: Vec<AchievementId>,
    /// The requirements that couldn't be interpreted, by label. Labels rather than a
    /// count because evaluation needs to know **which** gate is missing: one the profile
    /// has already passed stops blocking, and that is decided per gate, per profile.
    pub unknown: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphDiagnostic {
    /// A curated `behind` points at an achievement this catalog doesn't have.
    EdgeOutsideCatalog {
        node: AchievementId,
        achievement: AchievementId,
    },
    /// A challenge unlocked by several achievements: "either of these", which the model
    /// has no way to say. The requirement becomes unknown rather than wrong.
    Disjunction { node: AchievementId, count: u32 },
    /// Nodes that form a cycle. Filled in during evaluation.
    Cycle { nodes: Vec<AchievementId> },
    /// A node listed among its own prerequisites. The achievement that unlocks Tainted
    /// Isaac names Tainted Isaac in its requirements — true of the wiki's sentence, and
    /// meaningless as an edge. Dropped rather than treated as a cycle, because a cycle
    /// makes every node downstream unknowable and this is just a self-reference.
    SelfPrerequisite { node: AchievementId },
    /// A transformation whose set the profile has not reached: `current` of `at_least` of
    /// its items are unlocked. The node is `Partial`, and this says why — otherwise "we
    /// cannot say" and "you are one item short" would look the same from outside.
    ThresholdUnmet {
        node: AchievementId,
        label: String,
        current: u32,
        at_least: u32,
    },
    /// A requirement the graph can't express, treated as passed because the profile has
    /// already earned `done` achievements that carry it. Evidence read from the save, not
    /// an optimistic guess — and declared, so the inference is visible rather than magic.
    GateSatisfiedByEvidence { label: String, done: u32 },
}

pub struct Graph {
    pub(crate) nodes: Vec<Node>,
    pub(crate) diagnostics: Vec<GraphDiagnostic>,
}

impl Graph {
    pub fn build(c: &Catalog, rules: &Rules) -> Graph {
        let index = NameIndex::new(c);
        let mut nodes = Vec::new();
        let mut diagnostics = Vec::new();
        for a in c.achievements() {
            let id = a.id;
            let mut requirements = Vec::new();
            let mut prerequisites = Vec::new();
            let mut unknown: Vec<String> = Vec::new();
            // The character an achievement names applies to its whole sentence: "defeat
            // Mother as Magdalene" is one requirement spread over two references, and the
            // boss reference is the one that needs to know. Found once per achievement,
            // because it is a property of the sentence and not of any one reference.
            //
            // **By the wiki's id first, and only then by name.** The game gives a Tainted
            // character the base form's name and tells them apart by a flag, so the name
            // index holds one entry for the two and neither form can be named reliably:
            // "Tainted Isaac" is not a key it has at all, and plain "Isaac" can come back
            // as whichever of the two won the insert. The id is the only thing that tells
            // them apart, and here the answer is a row of the completion matrix — a wrong
            // one is a different character's cell, read with full confidence.
            //
            // Both halves of that were measured: by name alone, 141 of the 396 character
            // references resolved to nothing and fell through to the tally; name-first,
            // Ultra Greedier as Keeper picked row 29, which is T. Keeper.
            let character = rules.refs(id).iter().find_map(|r| match &r.target {
                Target::Character { id: cid } => c
                    .character(CharacterId(*cid))
                    .map(|ch| ch.id)
                    .or_else(|| index.character(rules.alias(&r.label))),
                Target::Item { .. }
                | Target::Trinket { .. }
                | Target::Achievement { .. }
                | Target::Challenge { .. }
                | Target::Entity { .. }
                | Target::Transformation { .. }
                | Target::Stage { .. }
                | Target::Room { .. }
                | Target::Concept { .. } => None,
            });
            for row in rules.refs(id) {
                let r = requirement_with(c, rules, &index, row, character);
                match &r {
                    Requirement::Unknown { label } => unknown.push(label.clone()),
                    Requirement::None => {}
                    // Answered by the profile, not by another achievement: no edge, and
                    // not unknown either. It travels in `requirements` and evaluation asks
                    // the profile about it — which is why it must not join `unknown`, or
                    // the node would stay `Partial` with the answer sitting right there.
                    // A threshold joins them for a sharper reason: the prerequisites of
                    // *any three of these eight* are a disjunction of subsets, and this
                    // model has no way to say one. The repo has met that shape before — a
                    // challenge unlocked by several achievements — and answered it with an
                    // unknown rather than an invented conjunction. Here the answer comes at
                    // evaluation, where the profile is, so it is not unknown either: it is
                    // simply not an edge.
                    Requirement::Mark { .. }
                    | Requirement::Counter { .. }
                    | Requirement::Threshold { .. } => {}
                    Requirement::Character { id: cid } => {
                        if let Some(by) = c.character(*cid).and_then(|ch| ch.unlocked_by) {
                            prerequisites.push(by);
                        }
                    }
                    Requirement::Boss { id: bid } => {
                        if let Some(by) = c.boss(*bid).and_then(|b| b.unlocked_by) {
                            prerequisites.push(by);
                        }
                    }
                    Requirement::Item { kind, id: iid } => {
                        if let Some(by) = c.item(*kind, *iid).and_then(|i| i.unlocked_by) {
                            prerequisites.push(by);
                        }
                    }
                    Requirement::Challenge { id: chid } => {
                        let by = c
                            .challenge(*chid)
                            .map(|ch| ch.unlocked_by.clone())
                            .unwrap_or_default();
                        match by.len() {
                            0 => {}
                            1 => prerequisites.push(by[0]),
                            n => {
                                // "Either of these" is a disjunction, and the model has no
                                // way to say it. Unknown is wrong-free; picking one would
                                // not be. It is labelled by the challenge it came from, so
                                // evidence for one challenge never speaks for another.
                                unknown.push(format!("challenge:{}", chid.0));
                                diagnostics.push(GraphDiagnostic::Disjunction {
                                    node: id,
                                    count: n as u32,
                                });
                            }
                        }
                    }
                    Requirement::Gate { gate } => {
                        // A ref straight to another achievement: the edge is the id itself,
                        // no verdict involved.
                        let direct = gate
                            .strip_prefix("achievement:")
                            .and_then(|n| n.parse::<u32>().ok())
                            .map(AchievementId);
                        let edge = match (direct, rules.verdict(gate)) {
                            (Some(id), _) => Some(id),
                            (None, Some(Verdict::Behind { achievement })) => Some(*achievement),
                            (None, Some(Verdict::AlwaysAvailable(_)))
                            | (None, Some(Verdict::NotAPrerequisite(_)))
                            | (None, Some(Verdict::Unknown { .. }))
                            // A gate answered by the profile is not an edge to another
                            // achievement: there is no achievement on the other side.
                            | (None, Some(Verdict::Progress { .. }))
                            | (None, None) => None,
                        };
                        if let Some(target) = edge {
                            if c.achievement(target).is_some() {
                                prerequisites.push(target);
                            } else {
                                diagnostics.push(GraphDiagnostic::EdgeOutsideCatalog {
                                    node: id,
                                    achievement: target,
                                });
                            }
                        }
                    }
                }
                requirements.push(r);
            }
            prerequisites.sort_unstable();
            prerequisites.dedup();
            if prerequisites.contains(&id) {
                prerequisites.retain(|p| *p != id);
                diagnostics.push(GraphDiagnostic::SelfPrerequisite { node: id });
            }
            unknown.sort();
            unknown.dedup();
            nodes.push(Node {
                achievement: id,
                requirements,
                prerequisites,
                unknown,
            });
        }
        Graph { nodes, diagnostics }
    }

    /// Reachable only through `crate::for_tests`, which is where the reason lives.
    #[cfg(feature = "test-api")]
    pub(crate) fn from_edges(edges: &[(u32, &[u32])], unknown: &[(u32, &[&str])]) -> Graph {
        let nodes = edges
            .iter()
            .map(|(id, prerequisites)| Node {
                achievement: AchievementId(*id),
                requirements: Vec::new(),
                prerequisites: prerequisites.iter().copied().map(AchievementId).collect(),
                unknown: unknown
                    .iter()
                    .find(|(n, _)| n == id)
                    .map(|(_, labels)| labels.iter().map(|l| l.to_string()).collect())
                    .unwrap_or_default(),
            })
            .collect();
        Graph {
            nodes,
            diagnostics: Vec::new(),
        }
    }

    /// Reachable only through `crate::for_tests`, which is where the reason lives.
    #[cfg(feature = "test-api")]
    pub(crate) fn from_requirements(rows: &[(u32, Vec<Requirement>)]) -> Graph {
        Graph {
            nodes: rows
                .iter()
                .map(|(achievement, requirements)| Node {
                    achievement: AchievementId(*achievement),
                    requirements: requirements.clone(),
                    prerequisites: Vec::new(),
                    unknown: Vec::new(),
                })
                .collect(),
            diagnostics: Vec::new(),
        }
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn node(&self, achievement: AchievementId) -> Option<&Node> {
        self.nodes.iter().find(|n| n.achievement == achievement)
    }

    pub fn diagnostics(&self) -> &[GraphDiagnostic] {
        &self.diagnostics
    }
}
