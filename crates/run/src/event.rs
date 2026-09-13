/// How the game labelled the seed. Measured on 2026-09-13 across the four logs in
/// `samples/logs/`: `New`, `Continue` and `Net` all occur, and they are not interchangeable —
/// a `Continue` is a run resumed from an earlier launch, so reading it as a fresh start would
/// mark the run it continues as abandoned. `Net` is an online run, which is the only free
/// discriminator we have for co-op.
///
/// `Unknown` keeps a word we have not met rather than dropping the run around it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SeedKind {
    New,
    Continue,
    Net,
    Unknown,
}

impl SeedKind {
    /// The game's own word. Anything else is `Unknown` — deliberately, and named for what we
    /// know rather than for a guess at what it might be.
    pub fn parse(word: &str) -> Self {
        match word {
            "New" => Self::New,
            "Continue" => Self::Continue,
            "Net" => Self::Net,
            _ => Self::Unknown,
        }
    }
}

/// One thing the log said, with nothing read into it. Every judgment belongs to the fold: the
/// rules file is data a user can edit, and a rule that could decide meaning would put
/// untestable logic outside the crate that is tested.
///
/// Stored as a JSON row by `store`, which is why it serializes. **A storage format, not a wire
/// one**: it never reaches TypeScript, so it does not carry the IPC's `camelCase` rules.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Event {
    RunStarted {
        seed_words: String,
        seed_numeric: u32,
        kind: SeedKind,
    },
    FloorEntered {
        stage: u32,
        stage_type: u32,
        seed: u32,
    },
    RoomEntered {
        id: String,
        name: String,
    },
    RoomTransition,
    ItemAdded {
        id: u32,
        name: String,
        player: u32,
        character: String,
        pool: String,
    },
    Died {
        killer: String,
        spawner: String,
    },
    Ended {
        cutscene: u32,
        name: String,
    },
    AchievementUnlocked {
        id: u32,
    },
    SaveWritten {
        file: String,
    },
}
