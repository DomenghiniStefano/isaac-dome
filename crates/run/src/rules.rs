use crate::event::{Event, SeedKind};
use regex::{Captures, Regex};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt;

/// The rules file as it is written: a version and one pattern per event kind.
#[derive(Debug, Deserialize)]
struct File {
    version: u32,
    patterns: BTreeMap<String, String>,
}

/// Why a rules file could not be used. Typed rather than a `String` because the caller reports
/// it to the user and then falls back to the embedded file: "which pattern" is the only part
/// worth saying, and a sentence built with `format!` is not translatable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulesError {
    /// The file is not the JSON this expects.
    Json(String),
    /// A pattern is not a regex.
    Pattern { kind: String, message: String },
}

impl fmt::Display for RulesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(message) => write!(f, "rules file is not valid JSON: {message}"),
            Self::Pattern { kind, message } => {
                write!(f, "pattern for {kind} is not a regex: {message}")
            }
        }
    }
}

impl std::error::Error for RulesError {}

/// Which event a pattern produces. Closed on purpose: a rules file naming something else is a
/// file we do not understand, not a new capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    RunStarted,
    FloorEntered,
    RoomTransition,
    RoomEntered,
    ItemAdded,
    Died,
    Ended,
    AchievementUnlocked,
    SaveWritten,
}

impl Kind {
    fn parse(name: &str) -> Option<Self> {
        match name {
            "runStarted" => Some(Self::RunStarted),
            "floorEntered" => Some(Self::FloorEntered),
            "roomTransition" => Some(Self::RoomTransition),
            "roomEntered" => Some(Self::RoomEntered),
            "itemAdded" => Some(Self::ItemAdded),
            "died" => Some(Self::Died),
            "ended" => Some(Self::Ended),
            "achievementUnlocked" => Some(Self::AchievementUnlocked),
            "saveWritten" => Some(Self::SaveWritten),
            _ => None,
        }
    }
}

/// The patterns, compiled. Turning a line into an event is all this does.
#[derive(Debug)]
pub struct Rules {
    version: u32,
    patterns: Vec<(Kind, Regex)>,
}

impl Rules {
    /// The file that ships, embedded at build time the way `graph` embeds its own rules. The
    /// app never reads it from disk.
    pub fn embedded() -> Self {
        // The embedded file is ours and is covered by a test: a failure here is a broken build,
        // not a user's bad file.
        Self::parse(include_str!("../rules/events.json"))
            .expect("the embedded rules file is part of the build")
    }

    /// A rules file supplied by the user. Never panics: the caller reports the error and keeps
    /// the embedded rules.
    pub fn parse(text: &str) -> Result<Self, RulesError> {
        let file: File = serde_json::from_str(text).map_err(|e| RulesError::Json(e.to_string()))?;
        let mut patterns = Vec::new();
        for (name, pattern) in &file.patterns {
            // A name we do not know is skipped rather than refused: a newer rules file may
            // carry an event an older binary has no variant for, and refusing the whole file
            // would cost every rule in it.
            let Some(kind) = Kind::parse(name) else {
                continue;
            };
            let regex = Regex::new(pattern).map_err(|e| RulesError::Pattern {
                kind: name.clone(),
                message: e.to_string(),
            })?;
            patterns.push((kind, regex));
        }
        // The order is the enum's, not the file's: a `BTreeMap` sorts by name, and the one
        // ordering that matters — a room transition is not a room — must not depend on how
        // somebody spelled the keys.
        patterns.sort_by_key(|(kind, _)| *kind);
        Ok(Self {
            version: file.version,
            patterns,
        })
    }

    /// The version the file declares. `store` keeps it beside a derived run, so a newer file
    /// invalidates the cache instead of leaving two readings side by side.
    pub fn version(&self) -> u32 {
        self.version
    }

    /// The event this line is, if it is one. An unknown line is `None`, never an error: 87% of
    /// a real log is animation warnings and mods write into the same file.
    pub fn event(&self, line: &str) -> Option<Event> {
        self.patterns
            .iter()
            .find_map(|(kind, regex)| regex.captures(line).and_then(|c| build(*kind, &c)))
    }
}

/// A named capture as text. Absent means the pattern does not have that group, which is a
/// rules file that does not match this binary: the line yields nothing rather than half an
/// event.
fn text(captures: &Captures<'_>, name: &str) -> Option<String> {
    captures.name(name).map(|m| m.as_str().to_string())
}

/// A named capture as a number. A value that does not fit is `None` for the whole event: the
/// line was not what the rule thought it was.
fn number(captures: &Captures<'_>, name: &str) -> Option<u32> {
    captures.name(name)?.as_str().parse().ok()
}

fn build(kind: Kind, c: &Captures<'_>) -> Option<Event> {
    Some(match kind {
        Kind::RunStarted => Event::RunStarted {
            seed_words: text(c, "words")?,
            seed_numeric: number(c, "numeric")?,
            kind: SeedKind::parse(&text(c, "kind")?),
        },
        Kind::FloorEntered => Event::FloorEntered {
            stage: number(c, "stage")?,
            stage_type: number(c, "stage_type")?,
            seed: number(c, "seed")?,
        },
        Kind::RoomTransition => Event::RoomTransition,
        Kind::RoomEntered => Event::RoomEntered {
            id: text(c, "id")?,
            name: text(c, "name")?,
        },
        Kind::ItemAdded => Event::ItemAdded {
            id: number(c, "id")?,
            name: text(c, "name")?,
            player: number(c, "player")?,
            character: text(c, "character")?,
            pool: text(c, "pool")?,
        },
        Kind::Died => Event::Died {
            killer: text(c, "killer")?,
            spawner: text(c, "spawner")?,
        },
        Kind::Ended => Event::Ended {
            cutscene: number(c, "cutscene")?,
            name: text(c, "name")?,
        },
        Kind::AchievementUnlocked => Event::AchievementUnlocked {
            id: number(c, "id")?,
        },
        Kind::SaveWritten => Event::SaveWritten {
            file: text(c, "file")?,
        },
    })
}
