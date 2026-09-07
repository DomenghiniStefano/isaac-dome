//! The graph itself: one node per achievement the catalog knows, with the achievements it
//! sits behind. Edges come from the game's own `unlocked_by` links, never from the wiki —
//! the wiki only says *what* is needed.

use catalog::{AchievementId, Catalog};

use crate::model::Requirement;
use crate::resolve::{requirement_with, NameIndex};
use crate::rules::{Rules, Verdict};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub achievement: u32,
    pub requirements: Vec<Requirement>,
    /// Achievement ids this node sits behind. Sorted and deduplicated: two refs naming the
    /// same prerequisite are one run, not two.
    pub prerequisites: Vec<u32>,
    /// How many requirements couldn't be interpreted. Above zero the node can never claim
    /// "available now".
    pub unknown: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphDiagnostic {
    /// A curated `behind` points at an achievement this catalog doesn't have.
    EdgeOutsideCatalog { node: u32, achievement: u32 },
    /// A challenge unlocked by several achievements: "either of these", which the model
    /// has no way to say. The requirement becomes unknown rather than wrong.
    Disjunction { node: u32, count: u32 },
    /// Nodes that form a cycle. Filled in during evaluation.
    Cycle { nodes: Vec<u32> },
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
            let id = a.id.0;
            let mut requirements = Vec::new();
            let mut prerequisites = Vec::new();
            let mut unknown = 0u32;
            for row in rules.refs(id) {
                let r = requirement_with(c, rules, &index, row);
                match &r {
                    Requirement::Unknown { .. } => unknown += 1,
                    Requirement::None => {}
                    Requirement::Character { id: cid } => {
                        if let Some(by) = c.character(*cid).and_then(|ch| ch.unlocked_by) {
                            prerequisites.push(by.0);
                        }
                    }
                    Requirement::Boss { id: bid } => {
                        if let Some(by) = c.boss(*bid).and_then(|b| b.unlocked_by) {
                            prerequisites.push(by.0);
                        }
                    }
                    Requirement::Item { kind, id: iid } => {
                        if let Some(by) = c.item(*kind, *iid).and_then(|i| i.unlocked_by) {
                            prerequisites.push(by.0);
                        }
                    }
                    Requirement::Challenge { id: chid } => {
                        let by = c
                            .challenge(*chid)
                            .map(|ch| ch.unlocked_by.clone())
                            .unwrap_or_default();
                        match by.len() {
                            0 => {}
                            1 => prerequisites.push(by[0].0),
                            n => {
                                // "Either of these" is a disjunction, and the model has no
                                // way to say it. Unknown is wrong-free; picking one would
                                // not be.
                                unknown += 1;
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
                            .and_then(|n| n.parse::<u32>().ok());
                        let edge = match (direct, rules.verdict(gate)) {
                            (Some(id), _) => Some(id),
                            (None, Some(Verdict::Behind { achievement })) => Some(*achievement),
                            (None, Some(Verdict::AlwaysAvailable(_)))
                            | (None, Some(Verdict::NotAPrerequisite(_)))
                            | (None, Some(Verdict::Unknown { .. }))
                            | (None, None) => None,
                        };
                        if let Some(target) = edge {
                            if c.achievement(AchievementId(target)).is_some() {
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
            nodes.push(Node {
                achievement: id,
                requirements,
                prerequisites,
                unknown,
            });
        }
        Graph { nodes, diagnostics }
    }

    /// A graph straight from edges, for tests on the walk that don't need a catalog.
    pub fn from_edges_for_tests(edges: &[(u32, &[u32])], unknown: &[(u32, u32)]) -> Graph {
        let nodes = edges
            .iter()
            .map(|(id, prerequisites)| Node {
                achievement: *id,
                requirements: Vec::new(),
                prerequisites: prerequisites.to_vec(),
                unknown: unknown
                    .iter()
                    .find(|(n, _)| n == id)
                    .map(|(_, u)| *u)
                    .unwrap_or(0),
            })
            .collect();
        Graph {
            nodes,
            diagnostics: Vec::new(),
        }
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn node(&self, achievement: u32) -> Option<&Node> {
        self.nodes.iter().find(|n| n.achievement == achievement)
    }

    pub fn diagnostics(&self) -> &[GraphDiagnostic] {
        &self.diagnostics
    }
}
