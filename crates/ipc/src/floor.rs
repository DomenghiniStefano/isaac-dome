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
    RulesUnreadable { reason: String },
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
pub fn floor_view(cells: Vec<Option<RoomKindView>>) -> FloorView {
    let mut diagnostics = Vec::new();

    let count = cells.len();
    let painted = cells.iter().filter(|c| c.is_some()).count() as u32;
    let grid = floor::Grid::from_cells(
        cells
            .into_iter()
            .map(|c| match c {
                None => floor::Cell::Empty,
                Some(k) => floor::Cell::Room {
                    kind: kind_of(k),
                    shape: floor::Shape::Single,
                },
            })
            .collect(),
    );

    let Some(grid) = grid else {
        diagnostics.push(FloorDiagnostic::GridMalformed {
            cells: count as u32,
        });
        return FloorView {
            solutions: Vec::new(),
            painted,
            diagnostics,
        };
    };
    if painted == 0 {
        diagnostics.push(FloorDiagnostic::GridEmpty);
    }
    if !floor::distance_from_start(&grid)
        .iter()
        .any(Option::is_some)
    {
        diagnostics.push(FloorDiagnostic::NoStartRoom);
    }

    let rules = match floor::Rules::embedded() {
        Ok(r) => r,
        Err(e) => {
            diagnostics.push(FloorDiagnostic::RulesUnreadable { reason: e.message });
            return FloorView {
                solutions: Vec::new(),
                painted,
                diagnostics,
            };
        }
    };

    let solutions = [
        floor::Target::Secret,
        floor::Target::SuperSecret,
        floor::Target::UltraSecret,
    ]
    .into_iter()
    .map(|t| {
        let s = floor::solve(&grid, rules, t);
        FloorSolutionView {
            target: target_view(s.target),
            candidates: s
                .candidates
                .into_iter()
                .map(|c| FloorCandidate {
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
                })
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
    })
    .collect();

    FloorView {
        solutions,
        painted,
        diagnostics,
    }
}

/// Every room kind, once. Written out rather than derived: the match below makes the compiler
/// refuse a kind that is missing, which is the only way a list like this stays complete.
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
/// Three kinds answer `None`, and each for a reason of its own:
///
/// - **Normal** — the game draws nothing on a normal room, and neither do we.
/// - **Start** — `minimap_icons.anm2` has no icon for it. The starting room is a normal room
///   with the player standing in it, and the marker is the player, not the room.
/// - none of the others.
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
/// same fourteen strings back for every cell the pointer crosses.
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
