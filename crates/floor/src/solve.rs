use std::cmp::Reverse;
use std::collections::{BTreeSet, VecDeque};

use serde::Serialize;

use crate::grid::{neighbours, Grid, CELLS};
use crate::room::Cell;
use crate::rules::{is_special, Constraint, Rule, Rules, Target};

/// One cell a rule allows, with the count that earned it and the rank of the rule that did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Candidate {
    pub cell: u16,
    pub neighbours: u8,
    pub rank: u8,
    pub applied: Vec<String>,
}

/// A rule the grid cannot evaluate, carried out loud.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Unresolved {
    pub rule: String,
    pub note: String,
    pub quote: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Solution {
    pub target: Target,
    pub candidates: Vec<Candidate>,
    pub unresolved: Vec<Unresolved>,
}

/// How many rooms are walked from the start room to each painted cell. `None` for an empty
/// cell, for a cell no path reaches, and for every cell when nothing is painted Start.
pub fn distance_from_start(grid: &Grid) -> Vec<Option<u16>> {
    let mut out = vec![None; CELLS];
    let start = (0..CELLS as u16).find(
        |c| matches!(grid.at(*c), Cell::Room { kind, shape: _ } if kind == crate::RoomKind::Start),
    );
    let Some(start) = start else { return out };
    out[start as usize] = Some(0);
    let mut queue = VecDeque::from([start]);
    while let Some(cell) = queue.pop_front() {
        let Some(d) = out[cell as usize] else {
            continue;
        };
        for n in neighbours(cell) {
            if matches!(grid.at(n), Cell::Empty) || out[n as usize].is_some() {
                continue;
            }
            out[n as usize] = Some(d + 1);
            queue.push_back(n);
        }
    }
    out
}

/// A fallback count rule, kept aside because its condition cannot be judged until the narrowing
/// rules have had their say. See `resolve_fallbacks`.
struct Fallback {
    rule: String,
    superseded_by_at_least: u8,
}

/// Where each secret room can be, in phases rather than in the order the file lists its rules:
/// the count rules **propose** cells, the narrowing rules **restrict** what was proposed, the
/// dead-end rule **ranks** what survived, and a fallback stands only if nothing better did.
/// Every rule is a sentence about the floor, and none of them says "after the one above".
pub fn solve(grid: &Grid, rules: &Rules, target: Target) -> Solution {
    let rules: Vec<&Rule> = rules.for_target(target).collect();
    let distances = distance_from_start(grid);
    let has_start = distances.iter().any(Option::is_some);

    let unresolved = rules
        .iter()
        .filter_map(|rule| unresolved(rule, has_start))
        .collect();
    let fallbacks: Vec<Fallback> = rules.iter().filter_map(|rule| fallback(rule)).collect();

    let proposed: Vec<Candidate> = rules.iter().flat_map(|rule| propose(grid, rule)).collect();
    let restricted = rules.iter().fold(proposed, |candidates, rule| {
        restrict(grid, rule, candidates)
    });
    let ranked = rules.iter().fold(restricted, |candidates, rule| {
        rank(&distances, has_start, rule, candidates)
    });

    let mut candidates = resolve_fallbacks(ranked, &fallbacks);
    candidates.sort_by(|a, b| a.rank.cmp(&b.rank).then(a.cell.cmp(&b.cell)));
    Solution {
        target,
        candidates,
        unresolved,
    }
}

/// A rule the grid cannot evaluate, said out loud: a sentence this grid cannot draw, or the
/// distance rule on a floor with no start room to measure from.
fn unresolved(rule: &Rule, has_start: bool) -> Option<Unresolved> {
    let note = match &rule.constraint {
        Constraint::Unmodelled { note } => note.clone(),
        Constraint::DeadEndDistanceRank { .. } if !has_start => {
            "no start room is painted".to_string()
        }
        Constraint::DeadEndDistanceRank { .. }
        | Constraint::NeighbourCount { .. }
        | Constraint::NeighbourCountFallback { .. }
        | Constraint::RedRoomConnections { .. }
        | Constraint::ForbiddenNeighbour { .. }
        | Constraint::NeighbourNotSpecial
        | Constraint::NoPaintedNeighbour
        | Constraint::RedRoomForbiddenNeighbour { .. } => return None,
    };
    Some(Unresolved {
        rule: rule.id.clone(),
        note,
        quote: rule.quote.clone(),
        url: rule.url.clone(),
    })
}

fn fallback(rule: &Rule) -> Option<Fallback> {
    match &rule.constraint {
        Constraint::NeighbourCountFallback {
            superseded_by_at_least,
            ..
        } => Some(Fallback {
            rule: rule.id.clone(),
            superseded_by_at_least: *superseded_by_at_least,
        }),
        Constraint::NeighbourCount { .. }
        | Constraint::RedRoomConnections { .. }
        | Constraint::ForbiddenNeighbour { .. }
        | Constraint::NeighbourNotSpecial
        | Constraint::NoPaintedNeighbour
        | Constraint::RedRoomForbiddenNeighbour { .. }
        | Constraint::DeadEndDistanceRank { .. }
        | Constraint::Unmodelled { .. } => None,
    }
}

/// The proposing phase: the cells a count rule puts forward. A fallback is proposed like any
/// count rule, so that the narrowing rules apply to its cells too; whether it survives is
/// decided at the end, by `resolve_fallbacks`.
fn propose(grid: &Grid, rule: &Rule) -> Vec<Candidate> {
    match &rule.constraint {
        Constraint::NeighbourCount { allowed, rank }
        | Constraint::NeighbourCountFallback { allowed, rank, .. } => {
            by_count(grid, allowed, *rank, &rule.id)
        }
        Constraint::RedRoomConnections {
            at_least,
            at_most,
            rank,
        } => by_reach(grid, *at_least, *at_most, *rank, &rule.id),
        Constraint::ForbiddenNeighbour { .. }
        | Constraint::NeighbourNotSpecial
        | Constraint::NoPaintedNeighbour
        | Constraint::RedRoomForbiddenNeighbour { .. }
        | Constraint::DeadEndDistanceRank { .. }
        | Constraint::Unmodelled { .. } => Vec::new(),
    }
}

/// The restricting phase: a narrowing rule keeps some of what was proposed and never adds a
/// cell. The rules of every other phase leave the candidates as they are.
fn restrict(grid: &Grid, rule: &Rule, candidates: Vec<Candidate>) -> Vec<Candidate> {
    match &rule.constraint {
        Constraint::ForbiddenNeighbour { kinds } => keep(candidates, &rule.id, |c| {
            !grid
                .neighbour_kinds(c.cell)
                .into_iter()
                .any(|k| kinds.contains(&k))
        }),
        Constraint::NeighbourNotSpecial => keep(candidates, &rule.id, |c| {
            !grid.neighbour_kinds(c.cell).into_iter().any(is_special)
        }),
        Constraint::NoPaintedNeighbour => keep(candidates, &rule.id, |c| {
            grid.neighbour_kinds(c.cell).is_empty()
        }),
        Constraint::RedRoomForbiddenNeighbour { kinds } => keep(candidates, &rule.id, |c| {
            red_rooms(grid, c.cell).into_iter().all(|red| {
                !grid
                    .neighbour_kinds(red)
                    .into_iter()
                    .any(|k| kinds.contains(&k))
            })
        }),
        Constraint::NeighbourCount { .. }
        | Constraint::NeighbourCountFallback { .. }
        | Constraint::RedRoomConnections { .. }
        | Constraint::DeadEndDistanceRank { .. }
        | Constraint::Unmodelled { .. } => candidates,
    }
}

/// The ranking phase: the dead-end rule moves the cell it chooses to the front and every other
/// one back a place. Without a start room there is nothing to measure, and `unresolved` says so.
fn rank(
    distances: &[Option<u16>],
    has_start: bool,
    rule: &Rule,
    candidates: Vec<Candidate>,
) -> Vec<Candidate> {
    match &rule.constraint {
        Constraint::DeadEndDistanceRank { rank } if has_start => {
            let chosen = farthest_dead_end(&candidates, distances, *rank);
            candidates
                .into_iter()
                .map(|mut c| {
                    if Some(c.cell) == chosen {
                        c.rank = 0;
                        c.applied.push(rule.id.clone());
                    } else {
                        c.rank = c.rank.saturating_add(1);
                    }
                    c
                })
                .collect()
        }
        Constraint::DeadEndDistanceRank { .. }
        | Constraint::NeighbourCount { .. }
        | Constraint::NeighbourCountFallback { .. }
        | Constraint::RedRoomConnections { .. }
        | Constraint::ForbiddenNeighbour { .. }
        | Constraint::NeighbourNotSpecial
        | Constraint::NoPaintedNeighbour
        | Constraint::RedRoomForbiddenNeighbour { .. }
        | Constraint::Unmodelled { .. } => candidates,
    }
}

/// The candidate `rank` places down the list of dead ends ordered by how far their room is from
/// the start, farthest first. Ties keep the candidates' own order: the sort is stable.
fn farthest_dead_end(candidates: &[Candidate], distances: &[Option<u16>], rank: u8) -> Option<u16> {
    let mut reach: Vec<(u16, u16)> = candidates
        .iter()
        .filter_map(|c| {
            let d = neighbours(c.cell)
                .into_iter()
                .filter_map(|n| distances[n as usize])
                .max()?;
            Some((c.cell, d))
        })
        .collect();
    reach.sort_by_key(|(_, d)| Reverse(*d));
    reach.get(rank as usize).map(|(cell, _)| *cell)
}

/// The cells of the grid nothing is painted on: the only places a secret room can go.
fn empty_cells(grid: &Grid) -> impl Iterator<Item = u16> + '_ {
    (0..CELLS as u16).filter(|cell| matches!(grid.at(*cell), Cell::Empty))
}

/// Every empty cell whose painted neighbours number one of `allowed`.
fn by_count(grid: &Grid, allowed: &[u8], rank: u8, id: &str) -> Vec<Candidate> {
    empty_cells(grid)
        .filter_map(|cell| {
            let count = grid.neighbour_kinds(cell).len() as u8;
            allowed.contains(&count).then(|| Candidate {
                cell,
                neighbours: count,
                rank,
                applied: vec![id.to_string()],
            })
        })
        .collect()
}

/// The sides of `cell` where a red room could open: its empty neighbours. A painted neighbour
/// is not one — the room is already there — and a side outside the grid is not one either,
/// which is the border the wiki allows rather than a side that fails.
fn red_rooms(grid: &Grid, cell: u16) -> Vec<u16> {
    neighbours(cell)
        .into_iter()
        .filter(|n| matches!(grid.at(*n), Cell::Empty))
        .collect()
}

/// How many distinct painted rooms `cell` reaches through those red rooms.
///
/// **Distinct**, because two red rooms beside the same room are one way in and not two, and
/// the wiki counts rooms reached rather than doors. The cell itself is never counted: it is
/// empty, and it is the thing being placed.
///
/// Nothing on this grid is a red room, so "non-red rooms" is every painted room it reaches.
/// The ceiling is twelve — four sides, three rooms each — which is why a `u8` is enough.
fn reached_rooms(grid: &Grid, cell: u16) -> u8 {
    let reached: BTreeSet<u16> = red_rooms(grid, cell)
        .into_iter()
        .flat_map(neighbours)
        .filter(|n| *n != cell && !matches!(grid.at(*n), Cell::Empty))
        .collect();
    reached.len() as u8
}

/// Every empty cell reaching between `at_least` and `at_most` rooms through its red rooms.
fn by_reach(grid: &Grid, at_least: u8, at_most: Option<u8>, rank: u8, id: &str) -> Vec<Candidate> {
    empty_cells(grid)
        .filter(|cell| {
            let reached = reached_rooms(grid, *cell);
            reached >= at_least && at_most.is_none_or(|m| reached <= m)
        })
        .map(|cell| Candidate {
            cell,
            // The rooms *touching* the cell, as for any other candidate — zero, once
            // `ultra-secret-not-connected` has had its say. The count that earned this one is
            // two cells out and is not the same measurement, so it does not go in this field.
            neighbours: grid.neighbour_kinds(cell).len() as u8,
            rank,
            applied: vec![id.to_string()],
        })
        .collect()
}

/// A fallback's cells stand only while nothing better does. The wiki says a 1-neighbour secret
/// room happens "only … if there are no **valid** 3+ neighbor locations", and *valid* is what
/// makes this a phase of its own: a cell with four neighbours that the Boss Room rules out was
/// never a valid location, so it must not switch the fallback off. That is only knowable once
/// the narrowing rules have run.
fn resolve_fallbacks(candidates: Vec<Candidate>, fallbacks: &[Fallback]) -> Vec<Candidate> {
    fallbacks.iter().fold(candidates, |candidates, f| {
        let superseded = candidates
            .iter()
            .any(|c| !c.applied.contains(&f.rule) && c.neighbours >= f.superseded_by_at_least);
        if superseded {
            candidates
                .into_iter()
                .filter(|c| !c.applied.contains(&f.rule))
                .collect()
        } else {
            candidates
        }
    })
}

/// The candidates a narrowing rule keeps, each of them now naming it: a candidate names every
/// rule behind it, not only the one that proposed it.
fn keep(candidates: Vec<Candidate>, id: &str, kept: impl Fn(&Candidate) -> bool) -> Vec<Candidate> {
    candidates
        .into_iter()
        .filter(|c| kept(c))
        .map(|mut c| {
            c.applied.push(id.to_string());
            c
        })
        .collect()
}
