//! The painted grid's answer, as JSON. Pure: the grid arrives from the screen, the rules are
//! embedded in `floor`, and nothing here opens a file.
//!
//! Every lit cell carries the sentence that lit it. That is not decoration: the rules are
//! quotations from a CC BY-SA wiki, and showing the quotation is how the attribution reaches
//! the person reading the screen.

use serde::{Deserialize, Serialize};

use crate::icon::IconRef;

/// A room as the screen paints it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum RoomKindView {
    Start,
    Normal,
    Boss,
    Treasure,
    Shop,
    Curse,
    Challenge,
    Sacrifice,
    Arcade,
    Library,
    Miniboss,
    Secret,
    SuperSecret,
    UltraSecret,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum TargetView {
    Secret,
    SuperSecret,
    UltraSecret,
}

/// One rule behind a candidate, with the sentence it was read from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct AppliedRule {
    pub id: String,
    pub quote: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FloorCandidate {
    pub cell: u16,
    pub neighbours: u8,
    /// Its place in the preference order the cited rule states; 0 is that rule's first.
    pub rank: u8,
    pub applied: Vec<AppliedRule>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FloorUnresolved {
    pub rule: String,
    pub note: String,
    pub quote: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FloorSolutionView {
    pub target: TargetView,
    pub candidates: Vec<FloorCandidate>,
    pub unresolved: Vec<FloorUnresolved>,
}

/// Everything that stops the screen from answering, said out loud. None of them may be drawn
/// as "there is nowhere for a secret room": that is an answer, and these are the absence of one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum FloorDiagnostic {
    /// Nothing painted yet.
    GridEmpty,
    /// No Start room, so the rule about how many rooms are walked cannot be read.
    NoStartRoom,
    /// The embedded rules did not parse. A build-time mistake, reported rather than hidden.
    RulesUnreadable,
    /// The grid did not have 169 cells. The screen sent something that is not a floor.
    GridMalformed { cells: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct FloorView {
    pub solutions: Vec<FloorSolutionView>,
    /// How many cells are painted. A count, not a fraction: nothing here knows how many rooms
    /// the floor has.
    pub painted: u32,
    pub diagnostics: Vec<FloorDiagnostic>,
}

fn kind_of(view: RoomKindView) -> floor::RoomKind {
    match view {
        RoomKindView::Start => floor::RoomKind::Start,
        RoomKindView::Normal => floor::RoomKind::Normal,
        RoomKindView::Boss => floor::RoomKind::Boss,
        RoomKindView::Treasure => floor::RoomKind::Treasure,
        RoomKindView::Shop => floor::RoomKind::Shop,
        RoomKindView::Curse => floor::RoomKind::Curse,
        RoomKindView::Challenge => floor::RoomKind::Challenge,
        RoomKindView::Sacrifice => floor::RoomKind::Sacrifice,
        RoomKindView::Arcade => floor::RoomKind::Arcade,
        RoomKindView::Library => floor::RoomKind::Library,
        RoomKindView::Miniboss => floor::RoomKind::Miniboss,
        RoomKindView::Secret => floor::RoomKind::Secret,
        RoomKindView::SuperSecret => floor::RoomKind::SuperSecret,
        RoomKindView::UltraSecret => floor::RoomKind::UltraSecret,
    }
}

fn target_view(t: floor::Target) -> TargetView {
    match t {
        floor::Target::Secret => TargetView::Secret,
        floor::Target::SuperSecret => TargetView::SuperSecret,
        floor::Target::UltraSecret => TargetView::UltraSecret,
    }
}

/// The join: the painted grid in, the three rankings out.
///
/// The diagnostics come in the order the grid is checked: its shape, then whether anything is
/// painted and whether a start room is, then the rules.
pub fn floor_view(cells: Vec<Option<RoomKindView>>) -> FloorView {
    let count = cells.len() as u32;
    let painted = cells.iter().filter(|c| c.is_some()).count() as u32;
    let Some(grid) = floor::Grid::from_cells(cells.into_iter().map(cell_of).collect()) else {
        return unsolved(
            painted,
            vec![FloorDiagnostic::GridMalformed { cells: count }],
        );
    };
    let has_start = floor::distance_from_start(&grid)
        .iter()
        .any(Option::is_some);
    let grid_diagnostics = [
        (painted == 0).then_some(FloorDiagnostic::GridEmpty),
        (!has_start).then_some(FloorDiagnostic::NoStartRoom),
    ]
    .into_iter()
    .flatten();
    let Ok(rules) = floor::Rules::embedded() else {
        return unsolved(
            painted,
            grid_diagnostics
                .chain([FloorDiagnostic::RulesUnreadable])
                .collect(),
        );
    };
    let solutions = [
        floor::Target::Secret,
        floor::Target::SuperSecret,
        floor::Target::UltraSecret,
    ]
    .into_iter()
    .map(|t| solution_view(floor::solve(&grid, rules, t), rules))
    .collect();
    FloorView {
        solutions,
        painted,
        diagnostics: grid_diagnostics.collect(),
    }
}

/// A view with no rankings, and why.
fn unsolved(painted: u32, diagnostics: Vec<FloorDiagnostic>) -> FloorView {
    FloorView {
        solutions: Vec::new(),
        painted,
        diagnostics,
    }
}

fn cell_of(c: Option<RoomKindView>) -> floor::Cell {
    match c {
        None => floor::Cell::Empty,
        Some(k) => floor::Cell::Room {
            kind: kind_of(k),
            shape: floor::Shape::Single,
        },
    }
}

fn solution_view(s: floor::Solution, rules: &floor::Rules) -> FloorSolutionView {
    FloorSolutionView {
        target: target_view(s.target),
        candidates: s
            .candidates
            .into_iter()
            .map(|c| candidate_view(c, rules))
            .collect(),
        unresolved: s
            .unresolved
            .into_iter()
            .map(|u| FloorUnresolved {
                rule: u.rule,
                note: u.note,
                quote: u.quote,
                url: u.url,
            })
            .collect(),
    }
}

/// A candidate cell, with the rules that placed it quoted by id. An id the rules file no longer
/// holds is left out rather than shown without its words.
fn candidate_view(c: floor::Candidate, rules: &floor::Rules) -> FloorCandidate {
    FloorCandidate {
        cell: c.cell,
        neighbours: c.neighbours,
        rank: c.rank,
        applied: c
            .applied
            .into_iter()
            .filter_map(|id| {
                rules.all().find(|r| r.id == id).map(|r| AppliedRule {
                    id: r.id.clone(),
                    quote: r.quote.clone(),
                    url: r.url.clone(),
                })
            })
            .collect(),
    }
}

/// Every room kind, once, in the enum's order. Written out by hand, and **no compiler holds it
/// complete**: the exhaustive matches beside it force a new kind to be named there, not here.
/// What does is a test, `room_kinds_lists_every_kind_the_wire_declares_and_each_one_round_trips`,
/// which compares it with the values the wire declares for `RoomKindView`.
pub const ROOM_KINDS: [RoomKindView; 14] = [
    RoomKindView::Start,
    RoomKindView::Normal,
    RoomKindView::Boss,
    RoomKindView::Treasure,
    RoomKindView::Shop,
    RoomKindView::Curse,
    RoomKindView::Challenge,
    RoomKindView::Sacrifice,
    RoomKindView::Arcade,
    RoomKindView::Library,
    RoomKindView::Miniboss,
    RoomKindView::Secret,
    RoomKindView::SuperSecret,
    RoomKindView::UltraSecret,
];

/// The name the **game** gives a room kind's minimap icon, when it has one.
///
/// Two kinds answer `None`, and each for a reason of its own:
///
/// - **Normal** — the game draws nothing on a normal room, and neither do we.
/// - **Start** — `minimap_icons.anm2` has no icon for it. The starting room is a normal room
///   with the player standing in it, and the marker is the player, not the room.
///
/// The other twelve have one.
///
/// **Challenge is a reading, and it is the one that could be wrong.** The file has no
/// `IconChallengeRoom`: it has `IconAmbushRoom` and `IconBossAmbushRoom`, and the wiki has
/// exactly two Challenge Room icons, the ordinary one and the boss one. Two names, two
/// pictures, the same relationship — that is structure and not a resemblance, which is why it
/// is written here instead of being left blank. It is still the line to suspect first if a
/// screenshot ever shows the wrong symbol.
pub fn minimap_icon_name(kind: RoomKindView) -> Option<&'static str> {
    match kind {
        RoomKindView::Normal | RoomKindView::Start => None,
        RoomKindView::Boss => Some("IconBoss"),
        RoomKindView::Miniboss => Some("IconMiniboss"),
        RoomKindView::Treasure => Some("IconTreasureRoom"),
        RoomKindView::Shop => Some("IconShop"),
        RoomKindView::Curse => Some("IconCurseRoom"),
        RoomKindView::Challenge => Some("IconAmbushRoom"),
        RoomKindView::Sacrifice => Some("IconSacrificeRoom"),
        RoomKindView::Arcade => Some("IconArcade"),
        RoomKindView::Library => Some("IconLibrary"),
        RoomKindView::Secret => Some("IconSecretRoom"),
        RoomKindView::SuperSecret => Some("IconSuperSecretRoom"),
        RoomKindView::UltraSecret => Some("IconUltraSecretRoom"),
    }
}

/// A room kind and the game's own picture of it, when the game is there to have one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct RoomIconView {
    pub kind: RoomKindView,
    /// `None` means "draw your own symbol": the game is not installed, the icon is not in the
    /// sheet, or the kind never had one. The screen cannot tell those apart and does not need
    /// to — all three end in the same drawing.
    pub icon_url: Option<String>,
}

/// The fourteen kinds with whatever picture the game has for each.
///
/// Answered on its own and not inside `floor_view`: the icons do not depend on what is
/// painted, and folding them into an answer that is recomputed on every stroke would send the
/// same fourteen answers — twelve URLs and two `None`s — back for every cell the pointer crosses.
pub fn room_icons(mut icon: impl FnMut(&IconRef) -> Option<String>) -> Vec<RoomIconView> {
    ROOM_KINDS
        .iter()
        .map(|&kind| RoomIconView {
            kind,
            // `and_then`, never `and`: `and` would evaluate the lookup for the two kinds that
            // have no icon and then throw the answer away — a request that cannot succeed,
            // made anyway, once per screen.
            icon_url: minimap_icon_name(kind).and_then(|_| icon(&IconRef::Room { kind })),
        })
        .collect()
}
