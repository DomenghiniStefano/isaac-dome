use std::cmp::Reverse;
use std::collections::VecDeque;

use serde::Serialize;

use crate::grid::{neighbours, Grid, CELLS};
use crate::room::Cell;
use crate::rules::{is_special, Constraint, Rules, Target};

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

/// A fallback count rule, remembered while the pass runs because its condition cannot be
/// judged until the narrowing rules have had their say. See `resolve_fallbacks`.
struct Fallback {
    rule: String,
    superseded_by_at_least: u8,
}

pub fn solve(grid: &Grid, rules: &Rules, target: Target) -> Solution {
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut unresolved: Vec<Unresolved> = Vec::new();
    let mut fallbacks: Vec<Fallback> = Vec::new();

    let has_start = distance_from_start(grid).iter().any(Option::is_some);

    for rule in rules.for_target(target) {
        match &rule.constraint {
            Constraint::Unmodelled { note } => unresolved.push(Unresolved {
                rule: rule.id.clone(),
                note: note.clone(),
                quote: rule.quote.clone(),
                url: rule.url.clone(),
            }),
            Constraint::DeadEndDistanceRank { rank: _ } if !has_start => {
                unresolved.push(Unresolved {
                    rule: rule.id.clone(),
                    note: "no start room is painted".to_string(),
                    quote: rule.quote.clone(),
                    url: rule.url.clone(),
                })
            }
            Constraint::NeighbourCount { allowed, rank } => {
                propose(&mut candidates, grid, allowed, *rank, &rule.id)
            }
            // Proposed here like any count rule, so that the narrowing rules below apply to its
            // cells too; whether it survives is decided after the pass, by `resolve_fallbacks`.
            Constraint::NeighbourCountFallback {
                allowed,
                rank,
                superseded_by_at_least,
            } => {
                propose(&mut candidates, grid, allowed, *rank, &rule.id);
                fallbacks.push(Fallback {
                    rule: rule.id.clone(),
                    superseded_by_at_least: *superseded_by_at_least,
                });
            }
            // The three below narrow what the count rules proposed: they never add a cell.
            Constraint::ForbiddenNeighbour { kinds } => {
                candidates.retain(|c| {
                    !grid
                        .neighbour_kinds(c.cell)
                        .into_iter()
                        .any(|k| kinds.contains(&k))
                });
                note_applied(&mut candidates, &rule.id);
            }
            Constraint::NeighbourNotSpecial => {
                candidates.retain(|c| !grid.neighbour_kinds(c.cell).into_iter().any(is_special));
                note_applied(&mut candidates, &rule.id);
            }
            Constraint::DeadEndDistanceRank { rank } => {
                let distances = distance_from_start(grid);
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
                let chosen = reach.get(*rank as usize).map(|(cell, _)| *cell);
                for c in candidates.iter_mut() {
                    if Some(c.cell) == chosen {
                        c.rank = 0;
                        c.applied.push(rule.id.clone());
                    } else {
                        c.rank = c.rank.saturating_add(1);
                    }
                }
            }
        }
    }

    resolve_fallbacks(&mut candidates, &fallbacks);
    candidates.sort_by(|a, b| a.rank.cmp(&b.rank).then(a.cell.cmp(&b.cell)));
    Solution {
        target,
        candidates,
        unresolved,
    }
}

/// Every empty cell whose painted neighbours number one of `allowed`.
fn propose(candidates: &mut Vec<Candidate>, grid: &Grid, allowed: &[u8], rank: u8, id: &str) {
    for cell in 0..CELLS as u16 {
        if !matches!(grid.at(cell), Cell::Empty) {
            continue;
        }
        let count = grid.neighbour_kinds(cell).len() as u8;
        if allowed.contains(&count) {
            candidates.push(Candidate {
                cell,
                neighbours: count,
                rank,
                applied: vec![id.to_string()],
            });
        }
    }
}

/// A fallback's cells stand only while nothing better does. The wiki says a 1-neighbour secret
/// room happens "only … if there are no **valid** 3+ neighbor locations", and *valid* is what
/// makes this a phase of its own: a cell with four neighbours that the Boss Room rules out was
/// never a valid location, so it must not switch the fallback off. That is only knowable once
/// the narrowing rules have run, which is why this is not a match arm above.
fn resolve_fallbacks(candidates: &mut Vec<Candidate>, fallbacks: &[Fallback]) {
    for f in fallbacks {
        let superseded = candidates
            .iter()
            .any(|c| !c.applied.contains(&f.rule) && c.neighbours >= f.superseded_by_at_least);
        if superseded {
            candidates.retain(|c| !c.applied.contains(&f.rule));
        }
    }
}

/// A narrowing rule applies to everything that survived it, and saying so is how a candidate
/// can name every rule behind it rather than only the one that proposed it.
fn note_applied(candidates: &mut [Candidate], id: &str) {
    for c in candidates.iter_mut() {
        c.applied.push(id.to_string());
    }
}
