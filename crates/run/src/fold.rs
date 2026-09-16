use crate::event::{Event, SeedKind};

/// What an item does when it is picked up. The three the fold has to tell apart, and no more:
/// passives and familiars accumulate, an active replaces the active before it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    Passive,
    Active,
    Familiar,
}

/// Where the fold gets an item's kind. A trait rather than a dependency: `run` must not know
/// about `catalog`, and a test must be able to answer with a table.
pub trait ItemKinds {
    fn kind_of(&self, id: u32) -> ItemKind;
}

/// How a run ended. Three of the four are stated by the log; `Abandoned` is inferred, and
/// `Open` is a run whose stream simply stopped — the one being played right now, or a log that
/// ends mid-run. **`Open` is not a failure state.**
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Outcome {
    Won { ending: String },
    Died { killer: String },
    Abandoned,
    Open,
}

/// One generation pass: what it built and how long it took to build it. The loop count is kept
/// because the line states it — dropping half of a measured line is a decision, and this crate
/// makes none about what the numbers mean.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Pass {
    pub rooms: u32,
    pub loops: u32,
}

/// What the log said about how a floor was built.
///
/// Three states and not an `Option<u32>`, for the reason `graph` has `Partial` and `floor` has
/// `Unmodelled`: **a floor nobody described must never read as a floor of no rooms.**
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Generated {
    /// The log described no pass for this floor. Greed mode never does — seven floors and not
    /// one `generate...` line, measured 2026-09-16 on
    /// `samples/logs/20260912-greed-online-coop.log.txt`, and it is not the online that
    /// silences it: the other `[Net]` log describes all eleven of its floors. A floor whose
    /// `Level::Init` was read in an earlier pass is the other way to land here.
    NotSaid,
    /// One pass, which is every floor of the normal path in everything we hold.
    Once { rooms: u32, loops: u32 },
    /// Several passes under one `Level::Init`, in the order the log wrote them.
    ///
    /// **Which of them is the floor that was walked is not established**, and this crate does
    /// not pick: the only such floor in the corpus is `m_Stage 4, m_StageType 4` — Mines II,
    /// the one floor in it that has an area of its own — and its two passes report the *same*
    /// 19 rooms, so nothing measured here can tell "first" from "last". A reader that needs
    /// one number has to say which it took.
    Several { passes: Vec<Pass> },
}

/// One floor, as the game announced it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Floor {
    pub stage: u32,
    pub stage_type: u32,
    pub seed: u32,
    pub generated: Generated,
}

/// One run: what was played, with what, and how it ended.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Run {
    pub seed_words: String,
    pub seed_numeric: u32,
    pub seed_kind: SeedKind,
    /// Not in the seed line: it arrives with the first item. A run with no item line keeps it
    /// `None` rather than guessing.
    pub character: Option<String>,
    /// The character's own id, from `Initialized player with Variant 0 and Subtype N`. The
    /// **only** line that tells a Tainted form from its base: the item line writes the name,
    /// and the game gives both the same one.
    pub character_id: Option<u32>,
    /// What the character began with. Not a find — see the starting window below.
    pub starting_items: Vec<u32>,
    /// Everything picked up after the first room transition, in order, including actives that
    /// were later replaced.
    pub collected: Vec<u32>,
    pub passives: Vec<u32>,
    pub familiars: Vec<u32>,
    /// The active actually being carried at the end. Actives replace one another, so summing
    /// the lines gives a player holding five books.
    pub held_active: Option<u32>,
    pub floors: Vec<Floor>,
    pub achievements: Vec<u32>,
    pub outcome: Outcome,
}

impl Run {
    fn open(seed_words: String, seed_numeric: u32, seed_kind: SeedKind) -> Self {
        Self {
            seed_words,
            seed_numeric,
            seed_kind,
            character: None,
            character_id: None,
            starting_items: Vec::new(),
            collected: Vec::new(),
            passives: Vec::new(),
            familiars: Vec::new(),
            held_active: None,
            floors: Vec::new(),
            achievements: Vec::new(),
            outcome: Outcome::Open,
        }
    }

    /// Every run in a stream of events, in the order they were played.
    ///
    /// This is where every judgment lives. The rules file says what a line *is*; what a line
    /// *means* is decided here, where it can be tested without a game.
    pub fn fold(events: impl Iterator<Item = Event>, kinds: &dyn ItemKinds) -> Vec<Run> {
        let mut done: Vec<Run> = Vec::new();
        let mut current: Option<Run> = None;
        // The starting window: open from the seed line to the first room transition. Inside it
        // an `ItemAdded` is the character's own gift, whatever pool the line claims.
        let mut starting = false;
        // A solo run initializes its player **before** the seed line and an online one after
        // it, measured on this machine's logs (2026-09-15). An init with no run yet is held
        // for the run about to start, and taken by it — a run that states its own wins.
        let mut pending_character: Option<u32> = None;

        for event in events {
            match event {
                Event::RunStarted {
                    seed_words,
                    seed_numeric,
                    kind,
                } => {
                    // The same seed on a run that is **still open** is that run resumed — the
                    // game logs `[Continue, 1]` with the seed it already had. The seed decides
                    // and not the label, because a label can be a word we have never met.
                    // `Open` is load-bearing: a seed can be replayed deliberately, and a run
                    // that already ended is closed, so the same number arriving again starts a
                    // second run rather than reopening the first.
                    if current.as_ref().is_some_and(|run| {
                        run.seed_numeric == seed_numeric && run.outcome == Outcome::Open
                    }) {
                        continue;
                    }
                    if let Some(mut previous) = current.take() {
                        if previous.outcome == Outcome::Open {
                            previous.outcome = Outcome::Abandoned;
                        }
                        done.push(previous);
                    }
                    let mut run = Run::open(seed_words, seed_numeric, kind);
                    run.character_id = pending_character.take();
                    current = Some(run);
                    starting = true;
                }
                other => {
                    // Events arriving before the first seed line belong to no run: a log
                    // begins mid-session, with menu lines and an intro cutscene. The one
                    // exception is the player being initialized, which a solo run logs just
                    // before its seed.
                    let Some(run) = current.as_mut() else {
                        if let Event::PlayerInitialized { subtype, .. } = other {
                            pending_character.get_or_insert(subtype);
                        }
                        continue;
                    };
                    apply(run, other, kinds, &mut starting);
                }
            }
        }

        if let Some(run) = current.take() {
            done.push(run);
        }
        done
    }
}

fn apply(run: &mut Run, event: Event, kinds: &dyn ItemKinds, starting: &mut bool) {
    match event {
        // Handled by the caller: a run cannot start inside itself.
        Event::RunStarted { .. } => {}
        Event::FloorEntered {
            stage,
            stage_type,
            seed,
        } => run.floors.push(Floor {
            stage,
            stage_type,
            seed,
            generated: Generated::NotSaid,
        }),
        // The summary belongs to the floor the game last announced. With no floor to attach it
        // to it is dropped rather than made into one: a read begins wherever the last one
        // stopped, so a pass whose `Level::Init` was taken by an earlier read is the ordinary
        // case, and inventing a floor for it would put a stage nobody played in the archive.
        Event::RoomsGenerated { rooms, loops } => {
            if let Some(floor) = run.floors.last_mut() {
                floor.generated = match std::mem::replace(&mut floor.generated, Generated::NotSaid)
                {
                    Generated::NotSaid => Generated::Once { rooms, loops },
                    Generated::Once {
                        rooms: first,
                        loops: first_loops,
                    } => Generated::Several {
                        passes: vec![
                            Pass {
                                rooms: first,
                                loops: first_loops,
                            },
                            Pass { rooms, loops },
                        ],
                    },
                    Generated::Several { mut passes } => {
                        passes.push(Pass { rooms, loops });
                        Generated::Several { passes }
                    }
                };
            }
        }
        // Kept for the rules to be complete; nothing is read from it yet.
        Event::RoomEntered { .. } => {}
        Event::RoomTransition => *starting = false,
        Event::ItemAdded { id, character, .. } => {
            if run.character.is_none() {
                run.character = Some(character);
            }
            if *starting {
                run.starting_items.push(id);
            } else {
                run.collected.push(id);
            }
            match kinds.kind_of(id) {
                ItemKind::Passive => run.passives.push(id),
                ItemKind::Familiar => run.familiars.push(id),
                ItemKind::Active => run.held_active = Some(id),
            }
        }
        Event::Died { killer, .. } => run.outcome = Outcome::Died { killer },
        Event::Ended { name, .. } => run.outcome = Outcome::Won { ending: name },
        Event::AchievementUnlocked { id } => run.achievements.push(id),
        // The watcher's trigger, and it tells the fold nothing. A field for it here would be a
        // promise this crate does not keep.
        Event::SaveWritten { .. } => {}
        // The first one is the run's: in co-op the line repeats for every player at the
        // table, and this app speaks about the profile it reads.
        Event::PlayerInitialized { subtype, .. } => {
            run.character_id.get_or_insert(subtype);
        }
    }
}
