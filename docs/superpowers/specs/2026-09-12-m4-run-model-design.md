# M4, sub-project 1 — the run model and the log watcher

**Status:** design agreed in conversation on 2026-09-12 (path, backfill, storage model,
boundaries). Sections 2 to 4 were written after the owner said "mi fido, scrivi un doc":
they follow from the three decisions below, but they were not walked through one by one, so
they are the part of this document most worth disagreeing with.

**Split into two plans on 2026-09-13**, when the TDD plan was written and the size was visible.
**1a — the run model** (`docs/superpowers/plans/archive/2026-09-13-run-model.md`): the `run` crate
alone — `Tail`, the rules file, the events, the fold. Pure, and finished software on its own.
**1b — it goes live**: `log-watch`, the store's migration, run identity and the backfill of
`online_logs\`, the agreement with the save's own counters, and the view-models. This document
is the spec for both; §3's identity and §3's *Agreeing with the game* belong entirely to 1b,
because a source is a file and `run` never sees one.

**Corrected in place on 2026-09-13, before 1b was planned**, by four decisions taken in
conversation and one measurement that contradicted the document. §1 gains the game's data folder,
which `discovery` does not expose today unless a save happens to sit in it; §1 moves the resume
decision into `run`, where a return value worth checking belongs; §3's identity rests on an
anchor inside the run content, because the 4 KiB prefix turned out to be the machine describing
itself and one millisecond timing; §3 names the single window that can answer the counters
without asserting a vacuous truth; §5 says how far the app side goes. The measurement also cost
the sample set a file: the two logs of 2026-09-08 are one launch, not two.

**Scope.** The data: what a run *is*, and the machinery that builds one from `log.txt`.
**Not** in scope: the `Live` and `Runs` screens. Both routes already exist in the shell as
placeholders and both are views of this model; designing them first would let a layout
shape the IPC contract, which is the failure this repo has a rule against.

---

## 0. What is already decided, and by what

Three things come from the project, not from this design, and the design has to fit them.

- **`docs/PROJECT.md` fixes what the two screens answer.** `Runs`: win rate by character,
  most common ending and killer, streaks. `Live`: what I have collected in this run, and
  the current plan. The second half of `Live` is the interesting one — it ties a run in
  progress to `plan` and `graph`, so this model has to be joinable against a `TargetKey`,
  not only readable on its own.
- **The B8 spike report (2026-09-08)** answered what a real log contains, and three of its
  findings change this design rather than inform it. They are used below and cited where
  they bite.
- **The log is rewritten on every launch**, so the archive only ever contains what the app
  was running to see — plus whatever backfill can recover, which the owner chose to make as
  much as possible.

### The three decisions taken in conversation

1. **Backfill everything.** On start the app reads the runs already in `log.txt` *and* the
   logs of past online sessions in `online_logs\`. The archive is born full — 22 sessions
   are on the disk today — instead of empty. The cost is the whole of section 3: two ways
   to produce a run, and a need for run identity that does not depend on a clock the log
   does not have.
2. **Events are the archive; a run is a fold over them.** Not the run as the stored entity.
   Chosen because backfill and live then become the same function over a stream that is
   finished in one case and growing in the other, and because the rules file is already
   required to be updatable without recompiling — so re-derivation is a consequence of a
   decision already taken, not a speculative feature.
3. **Abandoned runs are kept and marked, and whether they count is measured, not chosen.**
   "Come il gioco": the save keeps `STREAK_COUNTER [22]`, `BEST_STREAK [23]`,
   `NEGATIVE_STREAK_COUNTER [113]` and `DEATHS [10]`. Our numbers have to agree with the
   game's, which makes this a property to test rather than a definition to invent.

---

## 1. Boundaries

The rule that decides the split is the repo's own: *if a return value is worth checking, it
lives in a pure crate*. In a file watcher the parts worth checking are not the ones that
touch the disk.

### `run` — new, pure, no I/O

Owns the typed events, the fold, and the rules file.

```rust
Rules::parse(text: &str) -> Result<Rules, RulesError>
Rules::event(&self, line: &str) -> Option<Event>
Run::fold(events: impl Iterator<Item = Event>, kinds: &dyn ItemKinds) -> Vec<Run>
```

*(Written `-> Run` on 2026-09-12; one stream contains every run in a log, so 1a returns them
all.)*

It also owns the part of tailing that is not I/O, because that is where the mistakes are:
the last line read may be half-written, one event can span several physical lines (B8: the
`Framebuffer Width:` block is one event over six), and a file that got *shorter* is a new
game launch rather than a file that grew.

```rust
Tail::advance(&mut self, chunk: &[u8]) -> Vec<String>  // complete lines; keeps the remainder
Tail::restarted(&self, len: u64) -> bool               // shorter than we had: a new launch
```

`run` does **not** depend on `catalog`. The fold needs item kinds — B8 finding (e): actives
replace one another, so summing `Adding collectible` lines gives a player holding five books
— and it takes them as a trait object the caller supplies. `ipc` wires the real catalog in.

**1b adds one thing to this crate, and the reason it lands here rather than in `log-watch` is
the same rule.** *Is this the same launch, or a new one?* is a return value worth checking, so
it is decided by a pure function and not by the code that holds the file handle:

```rust
SourceKey { prefix: u64, anchor: u64, offset: u64 }
Resume { Continue { offset: u64 }, Fresh }
run::resume(stored: &SourceKey, prefix: &[u8], len: u64, at_offset: &[u8]) -> Resume
```

`log-watch` supplies the bytes — 4 KiB from the head, and the 64 that end at the stored offset
— and asks. §3 says why the second window exists.

### `discovery` — the game's data folder stops being a side effect

*(Added 2026-09-13, while planning 1b. It is not a refinement: without it the watcher has
nowhere to watch.)*

`discovery` reaches `Documents\My Games\Binding of Isaac Repentance[+]\` only to look for
`.dat` files, and the folder surfaces **only inside `SaveSource::Documents { folder }` — that
is, only when a save happens to live there.** With Steam Cloud on, which is the ordinary case
and this machine's, the save is under `userdata\` and no candidate is `Documents`: nothing in
`Discovery` names the folder that holds `log.txt` and `online_logs\`.

So `Discovery` gains `game_data: Option<GameDataFolder>`, probed always and independently of
the saves:

```rust
GameDataFolder { dir: PathBuf, log: Option<PathBuf>, online_logs: Option<PathBuf>, save_backups: Option<PathBuf> }
```

Each of the three is an `Option` because each can be absent on a real machine. Like `Discovery`
itself it is **not `Serialize`**: these are paths, and a path never crosses the IPC. Its absence
is a `Diagnostic`, never an `Err` — the app starts without the game's logs the way it starts
without the game.

### `log-watch` — new, deliberately thin

`notify`, an offset, and a read. The trigger is B8 finding (a): the game logs
`Saving PersistentGameData to Steam Cloud: rep+persistentgamedata1.dat.` 132 times in one
run, which is a better signal than watching the directory because it also names the file.
Nothing in this crate is worth a test; everything that is has been moved into `run`.

**Never the whole file.** The log B8 measured is 33,222 lines, 87% of them animation
warnings. Offset tailing is the repo's rule about not loading a game file into memory, not
an optimisation.

### `store` — one migration, the fourth

*(Written as "the third" on 2026-09-12 and corrected on 2026-09-13: `SCHEMA_VERSION` is
already 3, since the window session landed with `feature/background-and-tray`.)*

`events` as rows, and `runs` as a **derived cache** carrying the rules version that produced
it. A newer rules file invalidates the cache and the runs are folded again. This is decision
2 made structural instead of promised.

**Three tables, not one** (1b): `sources` — one row per log file read, holding the key of §3 and
the offset reached; `events`, keyed `(source_id, seq)`, which *are* the archive; `runs`, keyed
`(source_id, ordinal)` and carrying the `rules_version` that produced the row. Only the third is
disposable, and it says so by carrying the number that makes it stale. `Event` and `SeedKind`
need `Serialize`/`Deserialize`, which 1a did not give them: an event is stored, not sent, so this
is a storage format and not an IPC one — it is under no obligation to be `camelCase`.

### `ipc` — the view-models, as always the only contract with Vue

---

## 2. Events, and where judgment lives

**The rules file maps text to events and does nothing else. Every judgment lives in the
fold.** This matters because the rules file is data the user can update: a rule that could
decide *meaning* would let a bad file change what a run is, and would put untestable logic
outside the crate that is tested.

**The lines below are quoted without the prefix they actually carry**, and that was found on
2026-09-13 while planning 1a rather than while reading them. A real line is
`[INFO] - RNG Start Seed: …`, and some carry a frame marker too —
`[INFO] - [Frame 74] Starting room transition (type 0)`. About 1% carry no `[INFO] -` at all:
the `Framebuffer Width:` block and the library banners. **No pattern may be anchored at the
start of a line**, or it matches nothing at all. `CLAUDE.md` quoted them the same way and is
corrected with this.

The events, from lines verified in M0 and re-verified by B8:

| Event | Line |
|---|---|
| `RunStarted { seed_words, seed_numeric, kind }` | `RNG Start Seed: FYQ8 QQ8G (586324166) [New, 1]` |
| `FloorEntered { stage, stage_type, seed }` | `Level::Init m_Stage 2, m_StageType 1 Seed …` |
| `RoomEntered { id, name }` | `Room 1.2(Start Room)` — three forms, 264 lines |
| `RoomTransition` | `Starting room transition (type 0)` |
| `ItemAdded { id, name, player, character, pool }` | `Adding collectible 225 (Gimpy) to player 0 (Cain) from pool treasure` |
| `Died { killer, spawner }` | `Game Over. Killed by (9.0) spawned by (84.0) …` |
| `Ended { cutscene, name }` | `playing cutscene 21 (Greed Mode).` |
| `AchievementUnlocked { id }` | `unlock steam achievement '19'` |
| `SaveWritten { file }` | `Saving PersistentGameData to Steam Cloud: …` |

**The `kind` on `RunStarted` has three values and they are not decoration**, which was measured
on 2026-09-13 and is not in the row above. The logs contain `New`, `Continue` and `Net`. A
**`Continue` is a run resumed** from an earlier launch, and the game logs it with the seed the
run already had — so the abandonment rule of §3 below, read literally, marks a run that is still
being played as abandoned and then counts it twice. `20260912-solo-judas.log.txt` opens with
one. **The fold decides by seed, not by label**: the same seed on a run still open is that run
resumed. `Net` is an online run, and it is the only free discriminator we have for the co-op
question §3 leaves open.

Three judgments the fold owns — the third is the one above; the first two are B8's:

- **The starting item.** `from pool X` lies: Judas starts with Book of Belial and the line
  claims `pool treasure`, identical in shape to a real pickup. The only discriminator is
  position — an `ItemAdded` between `RunStarted` and the first `RoomTransition` is the
  character's starting item, whatever pool it claims. This is why `RoomTransition` is an
  event at all. Eden and the Tainted emit several lines in that window and the rule holds
  for them unchanged. Getting this wrong over-counts treasure-room finds by one in every
  run, with perfectly well-formed data and no test noticing.
- **The inventory.** Passives and familiars accumulate, actives replace. The run that B8
  watched picked up five actives and ended holding one.

**The character** is not in `RunStarted`: it arrives on the first `ItemAdded`. The fold
carries the run as unnamed until then, and a run with no item line at all keeps the
character `Unknown` rather than guessing.

---

## 3. Identity, outcome, and the game's own numbers

### Identity — the hard part backfill created

**The log has no wall clock.** It has frame numbers, not timestamps. So identity cannot be
"seed plus time", and the seed alone is not unique either: a seed can be replayed
deliberately.

A **source** is one log file as read once: an `online_logs\<session>\` folder, or one launch
of the game. A run is `(source, ordinal)`.

- An `online_logs` session is identified by its folder path. Re-reading it finds the same
  source and imports nothing twice.
- A launch of `log.txt` is identified by **its first 4 KiB and its length**. When the app
  restarts while the game is still running, a matching prefix means the same launch: resume
  from the stored offset instead of importing the runs again. `Tail::restarted` catches the
  other direction — the file got shorter, so the game relaunched and a new source begins.

This is the weakest part of the design and it is stated here so it gets attacked rather than
discovered: a prefix hash is a heuristic, not an identity.

**It was attacked on 2026-09-13, before 1b was planned, and it does not survive as written.**
The first 4 KiB of a log is the machine speaking about itself — OpenGL version, driver, the
OpenAL banner, the path of the game's own DLL — and none of it changes between two launches on
one machine. Measured on the four logs in `samples/logs/`: the only byte that separates the two
of 2026-09-12 inside that window is `load archives: 2633 milliseconds` against `2549`, at byte
1633. **The discriminating power of the prefix is a millisecond timing**, which is an accident
and can collide; everything around it is constant by construction.

The same measurement corrected the sample set: `20260908-run-megasatan-judas.log.txt` is a
**strict prefix** of `20260908-s1.log.txt` (`cmp` reports EOF on the shorter with no differing
byte). They are not two launches, they are one launch copied twice at different moments — so
the folder holds three distinct launches, not four, and it contains **no pair of different
launches of the same machine at all**. The direction that matters most cannot be tested on real
data today, which is itself a reason not to rest the design on the prefix.

The two mistakes do not cost the same. A false *same launch* resumes at an offset into a new
file and loses the runs before it; a false *different launch* **re-imports runs that are already
in the archive**, and a duplicated run is wrong in the win rate and the streak forever. The
second is the one to make impossible.

**So the prefix stays as a cheap filter and the proof is an anchor.** A source stores
`(prefix, anchor, offset)`, where `anchor` hashes the 64 bytes that *end* at the offset already
consumed — bytes inside the run content, not inside the banner. The watcher resumes only when
the prefix matches, `len >= offset`, and the anchor still hashes the same; anything else is a
new source read from zero. The anchor costs one 64-byte read and removes the constant-banner
weakness entirely, because two different launches never write the same bytes at the same offset
past the header. It is still a heuristic — but it is one whose failure mode is re-reading a log,
not merging two.

### Outcome — four values, one of them inferred

`Won { ending }` · `Died { killer }` · `Abandoned` · `Open`

Only the first two are stated by the log. **`Abandoned` is inferred**: a `RunStarted` arrives
while the previous run has neither a death nor an ending. `Open` is a run whose stream simply
stopped — the run being played right now, or a session whose log ends mid-run. `Open` is not
a failure state and must never be shown as one.

### Agreeing with the game

The archive's streak has to equal `STREAK_COUNTER [22]`; its deaths have to move with
`DEATHS [10]`. That turns "do abandoned runs count?" from a product opinion into a
measurement, and it is the property this sub-project is tested by.

**One observation already contradicts the naive version of that rule, and it is today's.**
The online co-op Greed win of 2026-09-12 did **not** move counter 22. Either co-op does not
count toward the streak — consistent with everything else measured about co-op the same day
— or Greed mode does not. Until that is separated, the property is asserted for solo,
non-Greed runs only, and the archive does not claim to mirror 22 for the rest.

**Which window can actually answer, named so the test is not written against nothing** (1b).
Of the three sources on disk, two cannot: the co-op Greed session is excluded by the paragraph
above, and `20260912-solo-judas.log.txt` is an `Open` run with neither a death nor an ending, so
every counter correctly stays put and a test that checks it passes while saying nothing — the
vacuity the repo has a rule about. The one live case is **2026-09-08**: the Mega Satan win with
Judas, with `20260907` and `20260908` in `samples/` making the window around the log of that day.

The task is a measurement with an open outcome, not an assertion waiting for a green tick. If
the archive's numbers move with the save's, the property is written **with its non-vacuity
guard** — it asserts that the window contains the win it is about. If they do not move together,
that is a measured fact for the report and an `Unknown` in the documents, and no rule is
invented to cover the gap. Either way the archive stops claiming, in code, an agreement nobody
has checked.

---

## 4. Degrading, and what the tests can actually run on

- **An unknown line is a diagnostic, never an error.** 87% of the log is animation
  warnings, and third-party mods write into the same file (`Lua Debug:` lines). The 2024
  log had mod noise and the 2026 one had none, so the rules file has to handle its absence
  as cheerfully as its presence.
- **A rules file that will not parse falls back to the embedded one** and reports it. The
  app starts. This is "degrade, never fail" applied to the one file a user can edit.
- **A half-read line is not an error either.** `Tail` keeps the remainder and waits.

### Tests

The fold is tested on hand-written event sequences, with no log and no game: that is the
point of putting it in a pure crate. The rules are tested against real logs, which now
exist in the repo's ignored `samples/logs/`:

```
20260908-run-megasatan-judas.log.txt    solo, hard, won, an unlock mid-run
20260912-greed-online-coop.log.txt      online co-op, Greed, won
20260912-solo-judas.log.txt             solo, no ending, no death — an Open run
```

Those three cover, between them, every outcome except `Died`, which needs one more log.

**Checked on 2026-09-13, and it is worse than "one more log" implies:** `grep -c "Game Over"`
returns **0 on all four** files in `samples/logs/` — the fourth, `20260908-s1.log.txt`, included.
So `Died` has no real-data coverage at all, only the line shape recorded in M0. The plan for 1a
asserts that absence in a test that fails the day a log with a death arrives, rather than
letting a green suite imply the event is covered.

**And the four files are three sources** (measured 2026-09-13, see §3): the two of 2026-09-08 are
one launch copied at two moments, the shorter being a byte-for-byte prefix of the longer. That is
a gift for one test and a hole for another — the pair is exactly the *same launch, grown* case
the resume rule is about, and there is no *different launch* pair anywhere in the folder.

**`test-support` needs one new accessor.** Its functions today reach saves
(`sample`, `dated_series`); the logs need the same treatment, declaring `sample: …` or
`skip: …` on stderr, because a test on real data has to say which slice of the domain it
ran on. No test opens `samples/logs/` by hand.

---

## 5. What this sub-project does not do

**Where 1b stops, said as a boundary rather than as a list of absences** (decided 2026-09-13):
it goes as far as the watcher running in the real Tauri process — managed state, started at
launch, the backfill once — the commands and the view-models, and the runs shown on the
development-only `#verify` page. That is the furthest one can go and still verify the thing by
playing a game, without a layout getting a vote on the contract. The page is not a screen and
does not become one: `Live` and `Runs` are designed later, on a model that has been used.

- Neither screen. `Live` and `Runs` stay placeholders.
- No secret-room finder: the backlog puts it in a different product and nothing here moves
  it.
- No reading of `sharedsave_*.dat`. Online co-op's shared profile is a second progression
  and a second format; B21 killed the branch that needed it and this design does not revive
  it.
