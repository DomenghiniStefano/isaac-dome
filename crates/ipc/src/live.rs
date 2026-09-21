//! What the run being watched would open — M4 sub-project 2b.
//!
//! The archive says what is being played; the graph says what a cell of the completion matrix
//! is worth. This module is the join, and it is pure: the command hands it both, from one
//! reading of the profile, because two commands cannot promise that two answers describe the
//! same one (N8).
//!
//! **The rule is narrow on purpose.** A run would open an achievement when *every* requirement
//! still missing from it is a mark for the character being played. One other missing
//! requirement — another character, an item, a counter — and finishing this run does not open
//! it, so it is not offered. The screen would rather say less than promise what the graph does
//! not.

use serde::Serialize;

use crate::graph::{AchievementRef, MarkColumnView, MarkLevelView, RequirementView, UnlockNode};
use crate::runs::RunView;

/// One achievement this run could open, and how much it opens in turn: the graph already
/// counts that for Unlock, and a run is worth more when what it gives unlocks more.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct LiveAchievement {
    pub achievement: AchievementRef,
    pub fan_out: u32,
}

/// One cell of the matrix, and what beating it with this character would open.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct LiveOpen {
    pub character: u32,
    pub character_name: String,
    pub column: MarkColumnView,
    pub level: MarkLevelView,
    pub achievements: Vec<LiveAchievement>,
}

/// Everything that stops this screen from answering, said out loud. None of them may be
/// drawn as "this run opens nothing": that is an answer, and these are the absence of one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum LiveDiagnostic {
    /// The app is watching, and nothing is being played.
    NoRun,
    /// A run is open and no item line has named its character yet: the seed line does not.
    CharacterNotNamed,
    /// The name the log printed reaches no character we know.
    UnknownCharacter { name: String },
    /// It reaches **more than one**, because the game gives a Tainted character the base
    /// form's name. The screen says both rather than choosing: picking one from the starting
    /// items would be an inference, and this repo pays for those.
    AmbiguousCharacter { name: String, forms: u32 },
    /// No graph — no catalog, or rules that did not parse. The run still draws.
    NoGraph,
    /// No profile chosen, so nothing is known to be missing. Not the same as no graph, and
    /// saying so is the difference between "we cannot read your progress" and "the game is
    /// not installed": the run still draws either way.
    NoProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct LiveView {
    pub run: Option<RunView>,
    /// The completion row of the character being played — two rows when the name reaches two
    /// forms. `None` without a profile or without the sections that hold the marks.
    pub marks: Option<LiveMarks>,
    /// Grouped by the cell it needs: "beat Mom's Heart with Cain" once, with everything it
    /// opens under it, instead of the same boss read five times.
    pub opens: Vec<LiveOpen>,
    pub diagnostics: Vec<LiveDiagnostic>,
}

/// The achievement as this screen offers it: with what it opens in turn, which is the graph's
/// own count and not a second reading of it.
fn offered(node: &UnlockNode) -> LiveAchievement {
    LiveAchievement {
        achievement: node.achievement.clone(),
        fan_out: match node.graph {
            crate::graph::GraphInfo::Computed { fan_out, .. }
            | crate::graph::GraphInfo::Partial { fan_out, .. } => fan_out,
        },
    }
}

/// The mark a requirement is, when it is one.
fn mark_of(r: &RequirementView) -> Option<(u32, &str, MarkColumnView, MarkLevelView)> {
    match r {
        RequirementView::Mark {
            character,
            character_name,
            column,
            level,
        } => Some((*character, character_name, *column, *level)),
        RequirementView::Item { .. }
        | RequirementView::Character { .. }
        | RequirementView::Challenge { .. }
        | RequirementView::Boss { .. }
        | RequirementView::Gate { .. }
        | RequirementView::Counter { .. }
        | RequirementView::Threshold { .. }
        | RequirementView::Unknown { .. } => None,
    }
}

/// What the command could gather of the graph. Three cases and not an `Option`, because
/// "no profile" and "no graph" are different sentences to the person reading the screen.
pub enum LiveGraph<'a> {
    Nodes(&'a Vec<UnlockNode>),
    NoProfile,
    NoGraph,
}

/// The join. `characters` answers who the catalog says this is: given the id the log stated it
/// answers exactly one, and given only a name it answers one — or two, when the base and the
/// Tainted form share it, which is the ambiguity the id exists to end.
pub fn live_view(
    run: Option<RunView>,
    nodes: LiveGraph<'_>,
    marks: Option<LiveMarks>,
    characters: impl Fn(&str, Option<u32>) -> Vec<(u32, String)>,
) -> LiveView {
    let mut diagnostics = Vec::new();
    let Some(run) = run else {
        return LiveView {
            run: None,
            marks: None,
            opens: Vec::new(),
            diagnostics: vec![LiveDiagnostic::NoRun],
        };
    };
    let nodes = match nodes {
        LiveGraph::Nodes(nodes) => nodes,
        LiveGraph::NoProfile => {
            return LiveView {
                run: Some(run),
                marks,
                opens: Vec::new(),
                diagnostics: vec![LiveDiagnostic::NoProfile],
            }
        }
        LiveGraph::NoGraph => {
            return LiveView {
                run: Some(run),
                marks,
                opens: Vec::new(),
                diagnostics: vec![LiveDiagnostic::NoGraph],
            }
        }
    };
    let Some(name) = run.character.clone() else {
        return LiveView {
            run: Some(run),
            marks,
            opens: Vec::new(),
            diagnostics: vec![LiveDiagnostic::CharacterNotNamed],
        };
    };
    // The log states the character by id when it says `Initialized player …`, and that is the
    // only source that tells a Tainted form from its base. When it is there nothing is
    // ambiguous; the name is the fallback for a run folded before that line was read.
    let forms = characters(&name, run.character_id);
    if forms.is_empty() {
        return LiveView {
            run: Some(run),
            marks,
            opens: Vec::new(),
            diagnostics: vec![LiveDiagnostic::UnknownCharacter { name }],
        };
    }
    if forms.len() > 1 {
        diagnostics.push(LiveDiagnostic::AmbiguousCharacter {
            name: name.clone(),
            forms: forms.len() as u32,
        });
    }

    // In the order the cells appear on the nodes: the graph's order is the achievements' own,
    // and inventing one here would put a boss first for a reason nobody could read.
    let mut opens: Vec<LiveOpen> = Vec::new();
    for node in nodes.iter().filter(|n| !n.done) {
        let mut marks = node.missing.iter().filter_map(mark_of);
        let Some((character, character_name, column, level)) = marks.next() else {
            continue;
        };
        // Everything missing has to be *this* mark: a second requirement of any kind — even a
        // second mark of the same character — is something this run cannot give.
        if node.missing.len() != 1 || !forms.iter().any(|(id, _)| *id == character) {
            continue;
        }
        let name = character_name.to_string();
        match opens
            .iter_mut()
            .find(|o| o.character == character && o.column == column && o.level == level)
        {
            Some(open) => open.achievements.push(offered(node)),
            None => opens.push(LiveOpen {
                character,
                character_name: name,
                column,
                level,
                achievements: vec![offered(node)],
            }),
        }
    }
    LiveView {
        run: Some(run),
        marks,
        opens,
        diagnostics,
    }
}

/// One character's row of the completion matrix, as Live draws it: what this character has
/// taken, and what it still has to. The screen puts it beside what the run could open, which
/// is the only place in the app where the two questions meet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct LiveMarkRow {
    pub character: String,
    pub head_url: Option<String>,
    pub cells: Vec<crate::marks::Cell>,
    /// Cells with nothing taken yet. Not a percentage: an unread cell is not a zero, and the
    /// matrix already refuses to average the two (B22/B23).
    pub missing: u32,
}

/// The rows Live shows, with the columns they are read against.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct LiveMarks {
    pub bosses: Vec<String>,
    pub art: Vec<crate::marks::MarkArtView>,
    pub rows: Vec<LiveMarkRow>,
}

/// The chosen rows of the matrix, copied rather than rebuilt: the matrix is the one place
/// that knows how a cell is read, and a second reading of the same counters would be a second
/// chance to disagree with the screen that draws them all.
pub fn live_marks(matrix: &crate::marks::MarksMatrix, rows: &[usize]) -> LiveMarks {
    LiveMarks {
        bosses: matrix.bosses.clone(),
        art: matrix.art.clone(),
        rows: rows
            .iter()
            .filter_map(|r| matrix.characters.get(*r))
            .map(|row| {
                LiveMarkRow {
                character: row.character.clone(),
                head_url: row.head_url.clone(),
                missing: row
                    .cells
                    .iter()
                    // A cell that reached no level, read off `level` and not off a bare 0:
                    // the online bit alone is not a mark, so a 4 is still a boss to beat.
                    .filter(|c| {
                        matches!(c, crate::marks::Cell::Known { level, .. } if !level.reached())
                    })
                    .count() as u32,
                cells: row.cells.clone(),
            }
            })
            .collect(),
    }
}
