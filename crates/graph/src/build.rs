//! The graph itself: one node per achievement the catalog knows, with the achievements it
//! sits behind. Edges come from the game's own `unlocked_by` links, never from the wiki —
//! the wiki only says *what* is needed.

use std::collections::BTreeSet;

use catalog::{AchievementId, Catalog, ChallengeId, CharacterId};

use crate::model::Requirement;
use crate::resolve::{requirement_with, NameIndex};
use crate::rules::{RefRow, Rules, Verdict};
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
    /// Sorted by achievement id, which lets `node` search it by halves. Every
    /// constructor goes through `Graph::new`, which makes the order.
    pub(crate) nodes: Vec<Node>,
    pub(crate) diagnostics: Vec<GraphDiagnostic>,
}

/// What one requirement adds to its node besides itself: an edge, an unknown, a diagnostic,
/// or nothing at all.
enum EdgeOutcome {
    /// No edge and nothing unknown: content the game does not gate, a ref judged as gating
    /// nothing, and every requirement the profile answers.
    Nothing,
    Edge(AchievementId),
    Unknown(String),
    /// A challenge unlocked by several achievements.
    Disjunction {
        challenge: ChallengeId,
        count: u32,
    },
    /// A gate naming an achievement this catalog doesn't have.
    OutsideCatalog(AchievementId),
}

impl EdgeOutcome {
    fn edge(&self) -> Option<AchievementId> {
        match self {
            EdgeOutcome::Edge(by) => Some(*by),
            EdgeOutcome::Nothing
            | EdgeOutcome::Unknown(_)
            | EdgeOutcome::Disjunction { .. }
            | EdgeOutcome::OutsideCatalog(_) => None,
        }
    }

    fn unknown(&self) -> Option<String> {
        match self {
            EdgeOutcome::Unknown(label) => Some(label.clone()),
            // Labelled by the challenge it came from, so evidence for one challenge never
            // speaks for another.
            EdgeOutcome::Disjunction { challenge, .. } => {
                Some(format!("challenge:{}", challenge.0))
            }
            EdgeOutcome::Nothing | EdgeOutcome::Edge(_) | EdgeOutcome::OutsideCatalog(_) => None,
        }
    }

    fn diagnostic(&self, node: AchievementId) -> Option<GraphDiagnostic> {
        match self {
            EdgeOutcome::Disjunction { count, .. } => Some(GraphDiagnostic::Disjunction {
                node,
                count: *count,
            }),
            EdgeOutcome::OutsideCatalog(achievement) => Some(GraphDiagnostic::EdgeOutsideCatalog {
                node,
                achievement: *achievement,
            }),
            EdgeOutcome::Nothing | EdgeOutcome::Edge(_) | EdgeOutcome::Unknown(_) => None,
        }
    }
}

impl Graph {
    pub fn build(c: &Catalog, rules: &Rules) -> Graph {
        let index = NameIndex::new(c);
        let (nodes, diagnostics): (Vec<Node>, Vec<Vec<GraphDiagnostic>>) = c
            .achievements()
            .map(|a| node_for(c, rules, &index, a.id))
            .unzip();
        Graph::new(nodes, diagnostics.into_iter().flatten().collect())
    }

    /// The one constructor, and where the order `node` relies on is made. The catalog hands
    /// its achievements over by id already, so for `build` the sort moves nothing; the test
    /// constructors take whatever order a test writes.
    fn new(mut nodes: Vec<Node>, diagnostics: Vec<GraphDiagnostic>) -> Graph {
        nodes.sort_by_key(|n| n.achievement);
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
        Graph::new(nodes, Vec::new())
    }

    /// Reachable only through `crate::for_tests`, which is where the reason lives.
    #[cfg(feature = "test-api")]
    pub(crate) fn from_requirements(rows: &[(u32, Vec<Requirement>)]) -> Graph {
        let nodes = rows
            .iter()
            .map(|(achievement, requirements)| Node {
                achievement: AchievementId(*achievement),
                requirements: requirements.clone(),
                prerequisites: Vec::new(),
                unknown: Vec::new(),
            })
            .collect();
        Graph::new(nodes, Vec::new())
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    pub fn node(&self, achievement: AchievementId) -> Option<&Node> {
        self.nodes
            .binary_search_by_key(&achievement, |n| n.achievement)
            .ok()
            .map(|i| &self.nodes[i])
    }

    pub fn diagnostics(&self) -> &[GraphDiagnostic] {
        &self.diagnostics
    }
}

/// One achievement's node, and the diagnostics building it raised, in the order its refs
/// were read.
fn node_for(
    c: &Catalog,
    rules: &Rules,
    index: &NameIndex,
    id: AchievementId,
) -> (Node, Vec<GraphDiagnostic>) {
    let refs = rules.refs(id);
    let character = sentence_character(c, rules, index, refs);
    let requirements: Vec<Requirement> = refs
        .iter()
        .map(|row| requirement_with(c, rules, index, row, character))
        .collect();
    let outcomes: Vec<EdgeOutcome> = requirements.iter().map(|r| edge_of(c, rules, r)).collect();
    let (prerequisites, names_itself) = prerequisites_of(id, &outcomes);
    let diagnostics = outcomes
        .iter()
        .filter_map(|o| o.diagnostic(id))
        .chain(names_itself.then_some(GraphDiagnostic::SelfPrerequisite { node: id }))
        .collect();
    let node = Node {
        achievement: id,
        requirements,
        prerequisites,
        unknown: unknown_of(&outcomes),
    };
    (node, diagnostics)
}

/// The character an achievement names applies to its whole sentence: "defeat Mother as
/// Magdalene" is one requirement spread over two references, and the boss reference is the
/// one that needs to know. Found once per achievement, because it is a property of the
/// sentence and not of any one reference.
///
/// **By the wiki's id first, and only then by name.** The game gives a Tainted character the
/// base form's name and tells them apart by a flag, so the name index holds one entry for the
/// two and neither form can be named reliably: "Tainted Isaac" is not a key it has at all,
/// and plain "Isaac" can come back as whichever of the two won the insert. The id is the only
/// thing that tells them apart, and here the answer is a row of the completion matrix — a
/// wrong one is a different character's cell, read with full confidence.
///
/// Both halves of that were measured: by name alone, 141 of the 396 character references
/// resolved to nothing and fell through to the tally; name-first, Ultra Greedier as Keeper
/// picked row 29, which is T. Keeper.
fn sentence_character(
    c: &Catalog,
    rules: &Rules,
    index: &NameIndex,
    refs: &[RefRow],
) -> Option<CharacterId> {
    refs.iter().find_map(|r| match &r.target {
        Target::Character { id } => {
            crate::resolve::character_of(c, index, *id, rules.alias(&r.label))
        }
        Target::Item { .. }
        | Target::Trinket { .. }
        | Target::Achievement { .. }
        | Target::Challenge { .. }
        | Target::Entity { .. }
        | Target::Transformation { .. }
        | Target::Stage { .. }
        | Target::Room { .. }
        | Target::Concept { .. } => None,
    })
}

/// The edge a requirement draws, read from the game's own `unlocked_by` links.
///
/// A mark, a tally and a threshold draw none and are not unknown either: they travel in
/// `requirements` and evaluation asks the profile about them. Counted as unknown, the node
/// would stay `Partial` with the answer sitting right there. A threshold has one more reason
/// to draw no edge: the prerequisites of *any three of these eight* are a disjunction of
/// subsets, which this model cannot say — the same shape as a challenge unlocked by several
/// achievements, except that the profile can answer this one at evaluation.
fn edge_of(c: &Catalog, rules: &Rules, r: &Requirement) -> EdgeOutcome {
    let game_gate = |by: Option<AchievementId>| by.map_or(EdgeOutcome::Nothing, EdgeOutcome::Edge);
    match r {
        Requirement::Unknown { label } => EdgeOutcome::Unknown(label.clone()),
        Requirement::None
        | Requirement::Mark { .. }
        | Requirement::Counter { .. }
        | Requirement::Threshold { .. } => EdgeOutcome::Nothing,
        Requirement::Character { id } => game_gate(c.character(*id).and_then(|ch| ch.unlocked_by)),
        Requirement::Boss { id } => game_gate(c.boss(*id).and_then(|b| b.unlocked_by)),
        Requirement::Item { kind, id } => game_gate(c.item(*kind, *id).and_then(|i| i.unlocked_by)),
        Requirement::Challenge { id } => challenge_edge(c, *id),
        Requirement::Gate { gate } => gate_edge(c, rules, gate),
    }
}

/// "Either of these" is a disjunction, and the model has no way to say it. Unknown is
/// wrong-free; picking one of the achievements would not be.
fn challenge_edge(c: &Catalog, id: ChallengeId) -> EdgeOutcome {
    let by = c
        .challenge(id)
        .map(|ch| ch.unlocked_by.as_slice())
        .unwrap_or_default();
    match by {
        [] => EdgeOutcome::Nothing,
        [one] => EdgeOutcome::Edge(*one),
        several => EdgeOutcome::Disjunction {
            challenge: id,
            count: several.len() as u32,
        },
    }
}

fn gate_edge(c: &Catalog, rules: &Rules, gate: &str) -> EdgeOutcome {
    match gate_target(rules, gate) {
        Some(target) if c.achievement(target).is_some() => EdgeOutcome::Edge(target),
        Some(target) => EdgeOutcome::OutsideCatalog(target),
        None => EdgeOutcome::Nothing,
    }
}

/// The achievement on the other side of a gate. A ref straight to another achievement is the
/// id itself, with no verdict involved; any other gate has one only when it is judged
/// `behind` one.
fn gate_target(rules: &Rules, gate: &str) -> Option<AchievementId> {
    let direct = gate
        .strip_prefix("achievement:")
        .and_then(|n| n.parse::<u32>().ok())
        .map(AchievementId);
    direct.or_else(|| match rules.verdict(gate) {
        Some(Verdict::Behind { achievement }) => Some(*achievement),
        Some(Verdict::AlwaysAvailable(_))
        | Some(Verdict::NotAPrerequisite(_))
        | Some(Verdict::Unknown { .. })
        // A gate answered by the profile is not an edge to another achievement: there is no
        // achievement on the other side.
        | Some(Verdict::Progress { .. })
        | None => None,
    })
}

/// The node's edges, sorted and deduplicated, without the node itself — and whether it was
/// among them.
fn prerequisites_of(id: AchievementId, outcomes: &[EdgeOutcome]) -> (Vec<AchievementId>, bool) {
    let edges: BTreeSet<AchievementId> = outcomes.iter().filter_map(EdgeOutcome::edge).collect();
    let names_itself = edges.contains(&id);
    let prerequisites = edges.into_iter().filter(|p| *p != id).collect();
    (prerequisites, names_itself)
}

fn unknown_of(outcomes: &[EdgeOutcome]) -> Vec<String> {
    let labels: BTreeSet<String> = outcomes.iter().filter_map(EdgeOutcome::unknown).collect();
    labels.into_iter().collect()
}
