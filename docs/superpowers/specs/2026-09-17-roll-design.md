# Roll — what to play tonight, drawn from the matrix

**Status:** seven decisions taken in conversation on 2026-09-17, six of them answers to a
question and one a correction the save format forced. §2 is that correction, and it is the
part of this spec most worth disagreeing with, because it overrides what the owner first
asked for.

**Why now.** The project's thesis is *"what am I missing, and what's worth playing
tonight"*, and every ordering the app offers today — Next steps, the plan queue — is
**derived**: the graph ranks, the player reads. Nothing in it ever says *pick one for me*.
This is that screen, and it is small because the space it picks from is already modelled:
the 34 x 12 completion matrix, which `ipc::marks_matrix` builds and the Completion screen
already draws.

---

## 1. What a draw is

**One target, drawn from the completion matrix, with the rest of the deck shown underneath**
("drawn from 137"). One, not three: a series of targets to do tonight is a queue, and the
Plan already is one.

```rust
pub enum Target {
    Mark { character: u8, column: u8 },   // "Mother, with Blue Baby"      34 x 12 = 408
    Greedier { character: u8 },           // "Ultra Greedier, with Judas"        +   34
}
```

442 targets. An enum and not `{ character, column, level }` because `Greedier` is legal on
exactly one column: a struct with a level field can hold "the second mark of Isaac's Boss
Rush", which is a thing this project refuses to name (§2).

A target's **status is derived on every read, never stored**:

| status | when |
|---|---|
| `Missing` | the cell is `Known { bits: 0 }` — for `Greedier`, `bits & 2 == 0` |
| `Taken` | the cell is `Known` and non-zero — for `Greedier`, `bits & 2 != 0` |
| `Unreadable` | the cell is `Unknown` or `Unexpected` |

`Taken` is *any* non-zero value, and that is a measurement, not a shortcut:
`docs/save-format.md` records four cells going **1 -> 2** across the 638-era series, so a
cell's bits **replace** one another rather than accumulating. There is no "the cleared bit".

Deriving the status is what makes the drawn target **close itself**: the card is not a record
of a task, it is a question re-asked against the current save every time the view is built.
Nothing has to be ticked off, and nothing can be ticked off wrongly.

## 2. The correction: `hard` is not something this app can read

The owner's answer to "does the draw carry the mode?" was **yes** — "Mother with Blue Baby,
in hard" as a target distinct from "Mother with Blue Baby". It cannot be built honestly, and
`docs/save-format.md` is why:

- **Bit 1 is measured in exactly one column.** In **Greed** it is *Ultra Greedier*, measured
  2026-09-12 on three days with three characters. What it means in the other eleven is
  **unmeasured**, which is why `graph::rules::MarkLevel` is named `Base` / `Second` after the
  bits and not `Hard` after a meaning.
- **`CLAUDE.md` forbids the guess** in so many words: *don't name a section, a bit, or a
  tally from a guess*. Sections 3 and 6 carried wrong names for months on exactly that kind
  of evidence.

A "hard" target would therefore have no readable status: the app could not say whether it is
already done, and could not notice when it got done. That is a card that never closes — the
button that does nothing B14 warns about, with a sentence on it the file does not support.

**So the mode is not part of a target, and the one measured second level is.** `Greedier`
carries its name from the measurement, and it is the only second-level target in the deck.
Thirty-four of them, one per character.

*Out of scope but found on the way, and bigger than a line:* **the Completion screen already
names bit 1 `hard`, in both languages.** `ui/src/i18n/messages/en.ts:600-605` has
`hard: 'marks at hard'`, *"every hard mark is also a normal one"* and *"characters with every
one of their readable cells at hard"*; `ui/src/i18n/messages/it.ts:608` is the same sentence
in Italian. That is a meaning `docs/save-format.md` refuses to state, written on a screen
that landed on 2026-09-17 — so the app is currently saying out loud what this spec is not
allowed to say. It is a backlog entry (a column's wording, both message files, and the
explanations under it), not a fix in this sub-project.

## 3. `roll` — the pure crate (the seventeenth)

A crate, not a module of `ipc`: eligibility, the deck and the draw are domain judgments, and
`ipc`'s job is to turn other crates' answers into view-models. The precedent is `floor` —
a pure crate for one screen's reasoning — and the rule in `CLAUDE.md`: *if a return value is
worth checking, it lives in a pure crate.*

**It knows nothing about the game.** No 34, no 12, no Greed:

```rust
pub struct Space { /* rows, columns, greed_column, cells, playable */ }
impl Space {
    pub fn new(
        rows: usize, columns: usize, greed_column: usize,
        cells: Vec<CellValue>,        // rows * columns, row-major
        playable: Vec<bool>,          // rows
    ) -> Result<Space, SpaceError>;
}
pub enum CellValue { Known { bits: u8 }, Unreadable }
```

The constructor checks the lengths and the column index, so a caller that gets the shape
wrong gets an error and not a silently short deck. The counts stay where they are measured
(`ipc::CHARACTERS`, `ipc::BOSSES`), which is the hardcoded-counts rule applied one level up.

```rust
pub fn deck(space: &Space, preset: &Preset) -> Deck;
pub fn draw(deck: &Deck, seed: u64) -> Option<Target>;
```

**The seed comes from `app`.** `roll` never reads the clock and never calls an RNG of its
own: `draw` is a pure function of `(deck, seed)`, which is what makes every test on it
deterministic. Mixing the seed into an index is six lines of splitmix64 — **no new
dependency**; `rand` would be one, for a modulo over 442.

`Deck` carries why the rest is not in it, and that is the point of the type:

```rust
pub struct Deck { pub targets: Vec<Target>, pub excluded: Excluded }
pub struct Excluded { pub taken: usize, pub unreadable: usize, pub locked: usize, pub filtered: usize }
```

Every target falls into exactly one bucket, so `targets.len()` plus the four counts is the
size of the space — a property, tested. A target that disappears without a name is the
failure this shape exists to prevent, and it is what lets the empty-deck message say
*"137 already done, 40 I cannot read, 231 of characters you do not have"* instead of
"nothing to draw".

**The preset** is the four axes the owner chose, all four of them editable:

```rust
pub struct Preset {
    pub characters: Selection,   // All | Only(Vec<u8>)
    pub columns: Selection,
    pub include_taken: bool,     // "only what I'm missing" vs "anything"
    pub only_playable: bool,     // characters you have not unlocked are not a run
}
```

`only_playable` is an option and not a fixed rule, by decision: it defaults on.

**`Unreadable` is never in the deck when `include_taken` is false.** We cannot claim a cell
is missing when the file does not say. With `include_taken` on it is drawable, and the card
says *"I cannot tell whether you have it"* rather than showing a state it invented. That is
roughly 40 targets of 442: Mother and The Beast are still unlocated for The Forgotten and
the nineteen later characters.

## 4. Persistence — migration 6

One preset, always saved, no "save" button; the current draw saved beside it.

```sql
CREATE TABLE roll (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    document TEXT NOT NULL
);
```

One row pinned by the `CHECK`, one JSON document — migration 2's shape, for migration 2's
reason: a second row would be a second answer to "what is the preset". The document is
`{ version, preset, current }`, with `current: Option<Drawn>` and
`Drawn { target, deck_size, drawn_unix }`. `deck_size` is stored because it is the only part
of a draw that cannot be recomputed later: the deck at the moment of the draw is gone once
the save moves.

```rust
pub fn roll(&self) -> Result<Result<roll::Document, roll::DocumentError>, StoreError>;
pub fn set_roll(&self, doc: &roll::Document) -> Result<(), StoreError>;
```

The nested `Result` mirrors `queue()`: "the database will not open" and "the document will
not parse" are different sentences on the screen. No row at all is `Document::default()` —
every character, every column, only what's missing, only playable, nothing drawn.

## 5. The IPC surface

`crates/ipc/src/roll.rs`, and `RollView` is one round trip for the whole screen (N8):

- `drawn: Option<DrawnView>` — the target resolved to names and art (the character's sprite,
  the column's mark art, both through `icon`), its **status recomputed now**, `deckSize` and
  `drawnUnix` as stored.
- `deck: DeckView` — size and the four exclusion counts.
- `characters` / `columns` — id, name, selected, **and how many targets that row currently
  contributes to the deck**, so the filter panel says what each tick is worth.
- `preset: PresetView` — `SelectionView` is tagged (`all` / `only { ids }`), the two booleans
  are booleans.
- `diagnostics: Vec<RollDiagnostic>` — `NoCounterSection`, `DocumentUnreadable`,
  `NoCatalog`, `EmptyDeck`. Expected cases travel in the payload; `Err(IpcError)` is only for
  a command that cannot answer at all.

**Playability** is derived in `ipc` from the catalog's reverse index and section 1: a
character is playable if some achievement that unlocks it is flagged done, or if **no**
achievement unlocks it at all (Isaac). If section 1 was not read, `only_playable` is **not
applied** and a diagnostic says so — a filter that silently keeps everything is worse than
one that admits it is off.

Three commands in `crates/app/src/commands/roll.rs`, each returning the whole `RollView`:
`roll` (read), `roll_draw` (writes a new draw, seeded from the clock here), `set_roll_preset`
(writes the preset). One new event, `ROLL_CHANGED`, mirrored by hand in
`ui/src/lib/window/appEvents.ts` as the other three are: with two windows open, a draw in one
is a read in the other.

## 6. The screen

A route of its own: `RouteName.Roll`, `/progress/roll`, origin `Progress` — so it sits behind
the profile gate like every screen that reads the save. Label **"Stasera"**, the project's
own question, with the dice icon.

Top to bottom: the card (character sprite, the column's mark art, the sentence, the status,
"drawn from 137", when), the **Pesca** / **Pesca di nuovo** button, then the filter panel.
The panel **reuses `ui/src/components/facets/FilterBar.vue`** and the faceting in
`ui/src/lib/facets/` — the sub-project that landed on 2026-09-17 exists precisely so a fourth
screen does not write those words a fourth time. Every change writes immediately: there is no
save button, by decision.

An empty deck is a first-class state, not a disabled button: it says which of the four
exclusions emptied it, with the numbers.

## 7. Degrade, never fail

- **The document does not parse** — the screen opens on the default preset, says the saved
  one could not be read, and **does not overwrite it** until the user changes something. A
  document written by a version that knew more is not garbage to be silently replaced.
- **A document from the future** (`version` higher than ours) is the same case, with its own
  wording.
- **Section 2 unread** — every cell is `Unreadable`: with `include_taken` off the deck is
  empty and says why; with it on, all 442 are drawable and every one of them reads as
  `Unreadable`.
- **No catalog** (the game's archives are not open) — names and art are absent, the draw
  still works: a target is an index pair, not a sprite.

## 8. Tests, written first

**`roll`** — one test per preset axis; the total accounting (`targets + excluded == 442` for
a full space, and the same identity on a small synthetic space); `Unreadable` never in the
missing deck; `Greedier` only on the Greed column; bit 2 (won online) not read as a second
level; `1 -> 2` honoured as `Taken` either way; `draw` deterministic for a fixed seed; `None`
on an empty deck; every target of a small deck reachable across many seeds; `Space::new`
rejecting a wrong length and a Greed column out of range; document round trip, malformed,
and a future version.

**`ipc`** — the JSON shape (`camelCase`, tagged enums, `rename_all_fields` on the struct
variants); the three statuses over the three cell shapes; the per-row deck contributions
summing to the deck; the playability derivation, including the character no achievement
unlocks and the section-1-missing case; each diagnostic.

**`store`** — migration 6 round trip; no row reads as the default document; a corrupt
document reaching the caller as the inner `Err` rather than a panic.

**`ui`** (Vitest) — the panel's select-all / select-none logic, the status sentence per
status, the empty-deck sentence per dominant exclusion.

Then `scripts/test-floor` is raised with the line `scripts/check` prints.

## 9. Non-goals

Named so the next reader does not find them missing: **no named presets** (one preset,
always saved), **no evening of three** (that is the Plan), **no self-imposed rules** ("no
devil deals"), **no seed**, **no integration with the plan queue** — a drawn target is not a
goal until someone asks for that. Each is one sub-project away and none is in this one.

## 10. Documents to update in the same commit

- **`docs/architecture.md`** — redrawn, and **the counts are recounted from the code at the
  commit it lands on rather than taken from here**: against `develop` at `f7c9104` this
  sub-project makes it 17 crates, 36 commands, 4 + 1 events, 15 routes, 6 migrations — but
  `feature/challenges` is unmerged and already adds a command and a route of its own, so the
  arithmetic depends on the order the two land in. What does **not** depend on it: the
  header's counts are **already wrong today**. It claims 30 commands and `generate_handler!`
  on `develop` holds **33**, and the challenges branch redraws nothing, so on its merge the
  drift becomes four. Corrected in the same pass, because redrawing a document against a
  stale tripwire is drawing it wrong twice.
- **`CLAUDE.md`** — a row for `roll` in the modules table, migration 6 in `store`'s row, and
  the crate count in the layout paragraph.
- **`docs/BACKLOG.md`** — the `it.ts:557` entry from §2.
