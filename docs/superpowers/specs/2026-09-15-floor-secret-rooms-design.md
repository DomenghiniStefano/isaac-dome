# Floor — the grid you paint, and where the secret room is

**Status:** four decisions taken in conversation on 2026-09-15, on a request that names its
own reference (`https://tboisecretroomfinder.com/`). Everything past §1 follows from them and
was not walked through one by one — it is the part most worth disagreeing with.

**Why now.** B8 closed on 2026-09-08 with three ideas the log unblocked, and this was the one
it parked: *"a secret-room-finder-style helper (depends on rooms, and on verifying how those
tools actually work — the belief to check is that they reason on the map you have already
explored plus placement rules, **not** on the seed)"*. That belief was checked on 2026-09-15
by reading the reference site: it asks the player to paint the minimap and ranks the free
cells by how many rooms touch them. No seed anywhere. The belief holds, so the idea is
buildable — and what the log does and does not carry, §2, is now measured rather than assumed.

---

## 1. The decision this reopens, out loud

B8 also wrote: *"a secret room finder is **a different product**. IsaacDome answers 'what am I
missing, and what's worth playing tonight'; that answers 'where is the secret room right
now'."* That sentence is still true, and this spec does not pretend otherwise. What changed is
that the owner asked for it anyway, which is a scope decision and theirs to make.

So it is taken in the open: **IsaacDome gains a companion screen whose question is not the
app's thesis.** The cost of leaving it implicit would be the next reader finding a screen the
backlog says should not exist. `docs/BACKLOG.md` B8 gets a line pointing here; it is not
deleted, because the reasoning in it is what makes the new decision legible.

The four decisions:

1. **The grid is painted by hand.** The log cannot draw it (§2). The log verifies it.
2. **It is its own route, linked from Live** — not a card inside Live. A 13x13 grid does not
   fit under Live's KPI row.
3. **The rules come from the game, cited.** Not from the reference site's tiers, which are
   numbers whose derivation we do not have. §4.
4. **1x1 cells now, room shapes later** — with the model able to hold a shape from day one.

## 2. What `log.txt` carries about a floor — measured 2026-09-15

Read across the five real logs in `samples/logs/` (two solo runs, one online co-op, one
session of deaths, one Mega Satan run).

**It does not carry room positions.** This is the fact the whole design turns on, and it is
worth being exact about what was looked at:

- `CURRENT ROOM INDEX 84` is printed **once per floor**, right after `Map Generated in N
  Loops`. 42 occurrences across the five logs, **never a value other than 84**.
- Every room change prints `[Frame N] Starting room transition (type 0)` then
  `Room 1.88(New Room)` — the room's `type.variant` and its name. **No index, no coordinate.**
  So the only position the game ever states is the one it states before you have moved.

That 84 is the **start room** follows from where it is printed.

**The width is no longer ours to infer.** This paragraph read "that the grid is 13 wide … is an
inference, not a measurement" until Task 1's research pass, on 2026-09-15, found the wiki stating
the number: *"locations on the **13x13 border** where a red room would normally open to an I AM
ERROR room are allowed"*
(`docs/superpowers/reports/2026-09-15-secret-room-rules.md` §3). That is a citation, not a
measurement of ours — but it is the game's own documentation, and it is the same standing as
every rule in `placement.json`.

What stays an inference is **the centre**: 84 being `6*13 + 6` rests on that width plus the
assumption that the start room sits in the middle, and nothing read so far says so. One real
floor painted by hand against the game's own minimap confirms it — a measurement, on the machine
with the game, not an assumption in the code.

**What it does carry, and nobody has used yet:**

| line | what it is |
|---|---|
| `Level::Init m_Stage 4, m_StageType 4 Seed …` | already an event: the floor |
| `generate...` / `place_room: shape 12` / `19 rooms in 12 loops` | one **attempt** at generating the floor |
| `Map Generated in 2 Loops` | the attempt that was kept |
| `Room 1.2(Start Room)`, `Room 5.5212(Great Gideon)` | the rooms visited, by type and name |

A floor prints **several** `N rooms in M loops` lines — the failed attempts are in the file
next to the accepted one. Which one describes the floor actually being played is a judgment,
so it belongs in `run`'s fold, and it is **measured against the five logs**, not assumed from
the ordering. Until that measurement exists, no count reaches the screen.

Room names carry a shape hint (`New Room (L2)`, `New Room (wide)`) and `place_room: shape N`
names shapes directly. Neither is used by this sub-project; both are why §3 keeps a shape in
the model.

## 3. The grid — a new pure crate, `crates/floor`

A crate rather than a module of `run`, because nothing here comes from the log: it is the
user's drawing plus the game's placement rules, and `run` is the archive.

```
Grid          13 x 13 cells, index = y * WIDTH + x
Cell          Empty | Room { kind, shape }
RoomKind      Start | Normal | Boss | Treasure | Shop | Curse | Challenge | Sacrifice
              | Arcade | Library | Miniboss | Secret | SuperSecret | UltraSecret
Shape         Single            (the only variant this sub-project draws)
```

`Shape` exists with one variant on purpose: decision 4. A one-variant enum costs a line and
buys the `match` that will break the build when the second variant arrives — which is exactly
what "1x1 now, shapes later" has to mean if it is not to become a rewrite.

**`neighbours(index)` does not wrap the row.** A cell at `x = 0` has no left neighbour, one at
`x = 12` has no right one, and a flat array makes both mistakes silently. This is the first
property in the test file, before any rule exists.

The solver:

```rust
pub fn candidates(grid: &Grid, target: Target) -> Solution
pub enum Target { Secret, SuperSecret, UltraSecret }
pub struct Candidate { pub cell: u16, pub neighbours: u8, pub rank: u8, pub applied: Vec<RuleId> }
pub struct Solution { pub candidates: Vec<Candidate>, pub unresolved: Vec<Unresolved> }
```

**No `Tier`.** Its variants could only be invented names for probability bands, and the repo's
rule is not to name a thing from a guess. A candidate carries a count and a rank — the rank
being its position in the preference order the cited rule states — and the screen colours by
the rank. Decided while writing the plan, 2026-09-15.

`applied` is not decoration: the screen shows *why* a cell is lit, and a candidate that cannot
name a rule is a candidate we should not be drawing. `unresolved` is `graph`'s `Partial` in
this domain — a rule the research pass found but the grid cannot evaluate (something the user
has not painted, a constraint about a room kind the grid does not model) **says so**. It is
never silently dropped, because a dropped constraint reads as "nothing in the way", and this
repo has paid for that reading once already.

## 4. The rules, and the research pass that precedes them

`crates/floor/rules/placement.json`, embedded with `include_str!` the way `graph/rules/` is.
Every rule carries **its source**: the wiki page it came from and the date it was read.

**The first task of the plan is not code.** It is a report,
`docs/superpowers/reports/<the day it is written>-secret-room-rules.md`, that reads the wiki's Secret Room
/ Super Secret Room / Ultra Secret Room pages and writes down each placement constraint with
its citation, and — as importantly — **what it could not source**.

The rule that governs the rest: **what that pass cannot cite does not enter the file.** The
screen then lights fewer cells, which is the correct failure. The alternative is a
`placement.json` whose entries are a mix of quotations and recollection, indistinguishable
from each other three months later — the exact shape of the mistake that named section 3
`PerChar` and section 6 `CardsPills`.

The reference site's tiers ("3+ adjacent normal rooms, ~11.5x more likely") are **evidence
that a ranking exists**, not a source for one. They may be quoted in the report as a thing
observed on 2026-09-15; they may not be copied into the rules file.

## 5. What the log adds — the verification half

The screen's own sentence: *the game generated 19 rooms; you have painted 15.* Three pieces:

- **New patterns** in `crates/run/rules/events.json` (version 3): `roomsGenerated`
  (`(?<rooms>\d+) rooms in (?<loops>\d+) loops`) and `mapGenerated`. Neither anchored at the
  start of a line — the `[INFO] - ` prefix is part of the line and some lines carry a frame
  marker on top of it.
- **`run::Floor` gains** `generated_rooms: Option<u32>` and the room kinds seen. The fold
  decides which attempt counted (§2), `None` when the log gave it nothing.
- **`LiveView` gains `currentFloor`** — stage, stage type, and that count. Live does not say
  today what floor is being played, which is a gap in a screen that calls itself a dashboard;
  this closes it, and the new screen reads the same view-model rather than a second one.

**Two sentences the screen may not say yet.** Whether `19 rooms` includes the secret rooms is
*unmeasured*. So: no "4 rooms left to find", no progress bar, no percentage. The screen
reports the number in the game's own words and lets the player do the subtraction — the same
discipline that keeps completion percentages off the Completion screen.

**A new `Level::Init` clears the grid.** It is the one thing a website cannot do, and it is
the entire argument for this living inside IsaacDome rather than in a browser tab.

## 6. What crosses the IPC

`crates/ipc/src/floor.rs`, pure, generated into `ui/src/lib/ipc/types.ts` by `pnpm ipc:types`
like everything else.

- In: the painted grid — 169 cells, each `null` or a room kind. Not a path, not a file.
- Out: `FloorSolutionView { candidates, unresolved, diagnostics }`, `camelCase`, tagged enums
  with struct variants, bare camelCase strings for the fieldless ones (`RoomKindView`,
  `TargetView`), exactly as `ItemKindView` and `StepsBasis` already are.
- Command: `floor_candidates(grid) -> Result<FloorSolutionView, IpcError>`. The Tauri crate is
  wiring; the answer worth checking is in `floor` and `ipc`.

The log half arrives through the existing `live` command, not a second one. **This is not the
N8 trap**: N8 forbids two commands answering about *the same* profile read, and these two
answers are about different things — one is the user's drawing, one is the log. Said here
because the shape looks like the forbidden one and the next reader will ask.

## 7. The screen

Route `RouteName.Floor`, its own entry in the navigation, `routes.floor` in i18n, and a line
in Live that leads to it.

- 13x13 of square cells. Paint by dragging the pointer, right-click erases, a room-kind picker
  beside the grid. Painting is not a list drag — `useDragList` is for reorderable lists and
  does not apply.
- **No `<style>`, no hardcoded visual constant.** The tiers are tokens in `@theme`
  (one family, declared once), never a hex or an opacity in a template. Cell size is a token.
- Every lit cell can say the rule that lit it. A tooltip or a side list, not a colour alone:
  the colour is the summary, the rule is the answer.
- State lives in a Pinia store, **not in SQLite**. A painted floor is a scratchpad that the
  next `Level::Init` throws away; a `store` migration for it would outlive the thing it holds.
- Fixtures for `pnpm ui:dev`: `?floor=empty|painted`, plus the existing `?fixture=none` so the
  screen is drawable on a machine with no game.

## 8. Tests

- **Grid properties, before any rule**: a middle cell has 4 neighbours, an edge cell 3, a
  corner 2, and no neighbour is ever on another row. Test-first, from the spec.
- **One table case per cited rule.** The expected value comes from the report of §4, and the
  case names the source. A rule with no test is a rule nobody can argue with.
- **The room count, on the five real logs**, through `test-support` — which log it read on
  stderr, `skip:` when `samples/logs/` is absent. The suite stays green on a fresh clone.
- Vitest for the indexing and the painting (a drag paints the cells it crosses, once each);
  the Kit page for presentation.
- `pnpm check` before anything is called done, and `-- --nocapture` to see what skipped.

## 9. Out of scope, named so nobody has to guess

- **Generating the map from the seed.** B8's reasoning stands unchanged and this spec does not
  reopen it: it would make the source of truth our clone's fidelity instead of the user's own
  files.
- **Room shapes** (decision 4) — the model holds one, the screen draws 1x1.
- **A second, always-on-top window.** Offered and declined; the screen is reached like every
  other screen. If it is ever wanted, it is a window sub-project, not a line in this one.
- **Persisting a painted floor** across app restarts.
- **Any claim about how many rooms are left**, until §5's measurement exists.

## 10. The two sub-projects

**F1 — the grid and the rules.** The research report, `crates/floor`, `ipc::floor`, the
command, the route, the screen, the tokens, the tests. Touches nothing M4 owns, so it cannot
break the archive, and at the end of it the screen is usable exactly as the reference site is.

**F2 — the log half.** The two new patterns and the rules version bump, `run::Floor`'s new
fields and the measurement that decides which attempt counts, `LiveView.currentFloor`, the
count beside the grid, and the clearing on `Level::Init`.

**Done when**, for F1: a floor painted by hand lights the cells the cited rules say it should,
each lit cell names its rule, the screen degrades with no catalog and no profile, and
`pnpm check` is green with the skips counted.

**Done when**, for F2: the count the screen shows is the one the five real logs yield under
the fold's rule, a new floor empties the grid, and no sentence on the screen subtracts two
numbers that have not been shown to be about the same set.
