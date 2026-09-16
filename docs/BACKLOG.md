# Backlog — tasks logged, not started

Things to do that came up during the work and were deliberately set aside. Each entry says what
already exists, what's missing, and what kind of task it is (analysis or implementation). When a task starts
it follows the usual cycle: spec → plan → execution → report, and the entry here closes with a
pointer to the report.

## What an entry needs before you can start it

Every open entry carries a **`**Needs:**` line under its heading**, added 2026-09-14 after a
session opened on a machine with no game installed and spent its first minutes finding out which
tasks were even possible. It answers one question — *what does this need beyond a clone of this
repo?* — and it always says why, because a label with no reason is the thing this project spends
its corrections on.

| value | means | who has it |
|---|---|---|
| `nothing` | pure crates, the frontend, the committed dataset, packaging | anyone who clones |
| `a real save` | a `.dat` in `samples/`, sometimes of a stated era | whoever plays, once |
| `the game` | the installed game: `samples/packed`, the catalog, the art | whoever has it installed |
| `a measurement` | the game **played**: a run, a matched window, `live_probe` | one session at the machine |

`, then a window` is added where finishing it means looking at the built app. It is **not** about
the game — the app runs and degrades without it — it is about the thing no test in this repo does,
which is draw a screen. It is the second most common reason a task cannot be closed remotely, and
the session log is full of entries that say *"not seen in a real Tauri window"*.

**The tag names what it takes to finish and believe the entry, not to start typing.** Plenty of
`the game` work can be written against fixtures; what it cannot do there is be verified, and a
half of a task that cannot be verified is not a half that should be shipped.

**Snapshot of 2026-09-14 (evening)**, 25 open entries — it was 26 that morning. **B34 closed
because tagging it meant reading it** and it turned out not to be finished; B38 closed; B44 opened
and closed the same hour, as not a defect; then B40 and B43 closed and B42 half closed on
`feature/small-three`, which opened **B45** — four characters with no page, found by the
cross-check B42 asked for and not by anyone looking for them — and then **B46**, found by trying
to look at B40's card in a window and discovering nothing in the app can open the page that draws
it. Both came out of checking finished work, which is where this list keeps finding things.
**B45 then closed the same evening it was opened** — the four pages are in the snapshot, with the
guard that makes a missing one loud — and opened **B47**: the same query that explained it found
seven more templates the fetch does not know, and 591 pages behind them.
**B46 closed the same night**, which made B40's card reachable and opened **B48**: an empty
category tells you your search found nothing when you never searched.
**Then the owner opened the app and reported six things in one message**: four were defects, and
two of them were fixed the same hour (the search that did not know the transformations, an
achievement's drawing overlapping the want suggestions). The other two, plus a missing picture,
are **B49**, **B50** and **B51** — and the only reason this list grew by three is that somebody
looked at the screen. **B51 closed within the hour**, and half of what it reported was a defect
of mine from the same afternoon: the card that draws a transformation's rows was suppressed whole,
taking the description and "sbloccato da" — which belong to every kind — down with it.
Regenerate rather than trust this
list — the command
prints each open entry's heading with its tag under it, and was run before it was written down:

```
grep -E '^## B[0-9]+ —|^\*\*Needs:\*\*' docs/BACKLOG.md | grep -A1 '^## ' | grep -B1 Needs
```

**Re-counted on 2026-09-16**, with the command above: **28 open**. B48, B49, B52 and B53
closed before it. **B53 opened out of B52** — a counter that had been reporting `{}`
since the day it was added — and **B54 out of B53**, which is where this list keeps finding things.
**B55, B56 and B57** came from a sweep of what the machine actually keeps, run because a claim had
been made from two folders and stated as though it came from all of them: none of the three is a
run history, and all three are sources this project did not know it had. **B59** is the newest, and
it came out of writing a comment: the convention scanner matches its rules against the raw file,
so a comment naming a forbidden call trips the rule that forbids it.

**Re-counted again the evening of 2026-09-16, on the second machine: 29 open.** The same sweep as
B55–B57, run on the other machine because a `Needs:` tag is the one thing in this file that is
about a machine and not about the work. It opened **B60** — a log the game wrote with no run in
it, which the suite cannot hold — and it **retagged B57** from `the game` to `nothing`, because
`savedatapath.txt` turns out to outlive the uninstall. One entry moved bucket and one entry is new,
and both came from looking at a machine instead of at the list.

**30 with B61**, opened the same evening out of the same series: measuring the header's `0x10`
meant opening the slot nobody plays, and it turns out to be the one profile shape this project has
never read.

**30 with B63**, opened the morning after while deciding what to do with the worktrees: a comparison against an abandoned branch turned up 27 tests that a commit had deleted four days earlier, with every gate green. **29 at the end of that evening**, and the moves are worth more than the total. **B60 closed** —
the log with no run is in `samples/launches/`, a folder of its own — and its own premise turned out
to be wrong, which the weakened guard proved by going red for the wrong reason. **B62 opened** out
of the first test written for it: a source folded into zero runs caches as one never folded.
**B61 closed** the same evening and stopped being a fixture: an untouched profile is zero
everywhere except the bestiary, which makes it the clean instrument the eleven-chunks gap in
`docs/save-format.md` has been missing. Both closings began with a test going red against an
expectation written an hour earlier.

- **`nothing` (20)** — B6, B11, B12, B14, B15, B17, B27, B29, B30, B39, B41, B42, B47, B54, B55, B56, B57, B59, B62, B63
- **`a real save` (4)** — B21, B22, B23, B58
- **`the game` (4)** — B3, B19, B33, B36
- **`a measurement` (2)** — B9, B20

The `a measurement` bucket is the same subject as *"What only a machine with the game can answer"*
in `docs/STATUS.md`, which collects the ones that are instruments rather than entries. Closed
entries carry no tag: nobody goes looking for a task that is done.

---

## B3 — Lists and search: challenges and items (implementation, after design)

**Needs:** the game — it lists challenges and items, and both come out of the catalog the game's XML builds.

Logged on 2026-09-05. The data is there; what's missing is the IPC commands that list challenges and
items with B1's fields and the filterable grid (TanStack Table) on the frontend side. It depends on the
screen design, so it waits on the design system rather than on the handoff — which
happened on 2026-09-09.

The "search" in this entry is the **filter inside a list**; searching across the whole app is a different
matter and is **B5**. The two meet only at the point where a global result opens the list
already filtered.

---

## B6 — Multiple tabs and session restore (implementation, after design)

**Needs:** nothing, then a window — tabs and the session document are `ui` and `store`; whether a restart reopens what you had is seen, not asserted.

Logged on 2026-09-06, an explicit request. The shell behaves **like a browser**: several tabs
open together, you switch between them, reorder them, close them. In the
global settings, a preference: **keep tabs saved when the app closes**, and on restart they
reopen where they were.

### What can live in a tab

Any view, not just the main sections: one of the screens, but also an item's detail,
a wiki page, a node of the unlock tree. Two tabs can show the
same screen with different filters, and two items get compared side by side by switching from one
tab to the other — which is the reason the feature exists.

### The two constraints that shape it

1. **A tab saves the identity of the view, never its content.** Route plus parameters plus serializable
   view state (filters, sorting, position); never the view-model, never base64 icons, never a file
   path. On restore, the tab reloads from the backend just like on first
   launch. It's the same boundary as always: only resolved view-models and opaque ids cross the IPC.
2. **The active profile stays global, not per tab.** §4.1 of `DESIGN-BRIEF.md` establishes
   that every number in the app depends on which save is being read and that the indicator is
   a single one, persistent in the shell. Tabs with different profiles side by side would break that
   promise: switching profile updates **all** the tabs.

### Where it's saved

**Settled on 2026-09-13, and built**: the recommendation below was taken whole, with the
background and tray work (`docs/superpowers/specs/2026-09-13-background-and-tray-design.md`).
The flag is `resumeTabs` in `settings.json`, the tabs are `store` migration 3 — one JSON
document in one row, an object with a `version` so the sidebar width and table sizes of B27 can
join it without a migration.

**Both of its open questions were answered by 3.7a** (2026-09-16, report
`docs/superpowers/reports/2026-09-16-tabs-own-their-state-report.md`). There is **no limit on the
number of tabs**: they shrink to `--spacing-tab-min` — which they already did — and below it the
strip scrolls, with the active tab brought into view, because the shell says it behaves like a
browser and a browser does not refuse to open what you ask it to. And **a restored tab states the
gap** one notch more finely than before: a tab whose *route* is gone still falls whole, since
there is nothing left to open, while a tab whose stored *reading* cannot be read opens on its
screen with the screen's own empty state.

**3.7b closed the second half of it** (2026-09-16, report
`docs/superpowers/reports/2026-09-16-tabs-windows-report.md`): the document is version 2 and its
top level is `windows`, so the drag spec's Decision 8 — *a session is windows of tabs* — is what
the app stores. A version 1 document still opens, as one window, because losing somebody's tabs
on an update is not a thing this app can explain to them afterwards.

**3.7c landed the same evening** (report
`docs/superpowers/reports/2026-09-16-tabs-sizes-report.md`) and the sidebar's width is in the
document too, beside the windows. So **every line of code 3.7 declared is written**.

What keeps this entry open is the **window nobody has opened on any of it**, which is one gap and
not three, and it is the larger half now: 3.7b's own failure mode is a session that stops being
written, which is invisible on screen by definition, so no amount of green says it works.

A declared fork, with a recommendation:

- **The flag** "reopen tabs on startup" in `settings.json` (`crates/app/src/settings_file.rs`),
  next to the other global preferences: it's a preference, and it lives where preferences live.
- **The list of tabs** in a **`store` migration**. It's structured state — one row per
  tab, with its identity serialized the way it already happens for the `TargetKey` of targets — and
  it will grow with the view types. `isaacdome.db` has a versioned schema for exactly this, and it's already the only
  file the app writes. Putting it in `settings.json` would mean evolving by hand a
  format that has no migrations.

### Questions the task has to close

- ✅ **A restored tab that points to something that no longer exists** (3.7a) — the user changed profile or
  edition, the wiki dataset was updated, the item doesn't exist in that version of the
  game. The tab opens **stating the gap**, it doesn't silently vanish and doesn't turn into
  a different view: it's the same principle as "unknown data ≠ zero data".
- ✅ **A limit on the number of tabs** (3.7a): there is none. They shrink, then the strip scrolls.
- **How a new tab opens**: from search (B5), from a wiki link, from a node
  in the tree, and with what gesture — middle click and `Ctrl+click` are the browser-like expectation.
- **What the first launch does**, when there are no saved tabs yet: it opens on *Next
  steps*, a single tab, as it does today.

---

## B9 — Re-identify the save's sections from the game's own names (implementation, delicate)

**Needs:** a measurement — naming a section is reading bytes move while the game runs; three of the four are still `Unknown` for want of a run.

Logged on 2026-09-08, out of the B8 spike. Evidence and the full table in
`docs/superpowers/reports/2026-09-08-b8-log-spike-report.md`.

### What we found

Loading a profile, the game prints `Reading chunk N` followed by that chunk's **name**, in
file order, for **eleven** chunks. Our table in `CLAUDE.md` has ten sections, four of them
marked "to be identified". The four have names, and two of the six we thought we knew are
contradicted:

- `Kind::PerChar` (3, 14 x 4) — the game calls it **Level Counters**, not one value per
  original character. Fourteen stages fits at least as well as fourteen characters.
- `Kind::Unknown5` (5, 7 x 1) — **Mini Bosses**.
- `Kind::CardsPills` (6, 104 x 1) — the game calls it **Bosses**. The catalog has **103**
  bosses; 104 cells fit that better than cards and pills.
- `Kind::Unknown8` (8, 27 x 4) — **Cutscene Counters**.
- `Kind::Unknown9` (9, 2 x 4) — **GameSettings**.
- `Kind::Bestiary` (10) — the game reads **two** chunks here, Special Seed Counters then
  Bestiary Counters. Our section 10's header declares `count=80, f2=320` and the parser
  hands it 11,016 bytes: the payload holds both, with no second header between them.

The four positions we are sure of (1, 2, 4, 7) all agree with the game's names, which is
what makes the rest worth acting on.

### Why it isn't just a rename

**A log line is not a measurement.** The names are strong evidence about *what the game
thinks it is reading*, not proof of what each cell means, and the existing labels may have
come from watching bits flip in M0. Section 6 is the sharp case: if it is Bosses, then
`SaveDiff.cards_pills` — a public field — has been reporting boss kills under a card's name
since the day it was written.

### Measured on 2026-09-08 (evening): steps 1 and 2 are answered

The analysis below used a source nobody had opened yet — **`online_logs\sessions\`**, 21
folders each holding a `log.txt` and two profile snapshots. `persistentgamedata1_end.dat`
is **our** profile (it matches the dated sample of the same day, counter for counter);
`persistentgamedata1_begin.dat` is **the other participant's**, a second real Repentance+
profile that grows from **52 to 105 achievements** across the series. A second profile at
the opposite end of the progression is exactly what these questions needed. The
`sharedsave_*.dat` files in the same folders are **not** in our format — no `ISAACNGSAVE`
magic anywhere in them — so the co-op shared profile stays unread for now.

- **Section 6 is Bosses. Settled, and not by the log.** 104 cells, boss ids 0..103, and the
  catalog has 103 bosses. On the beginner profile **56 of 104 are set, and the 48 that
  aren't are exactly the late and alt-path roster**: The Lamb, Mega Satan, Delirium, Mother,
  Dogma, The Beast and the whole Repentance list from Reap Creep to Cadavra. A player who
  never went down the alt path, read straight off the cell ids. On our own profile 97 of 104
  are set and the seven gaps are *The Matriarch*, *Cadavra*, **Raglich** — a boss that is
  unused in the game — plus four that look like variants recorded under another id
  (*Ultra Greed*, *Ultra Greedier*, *Mom (Mausoleum)*, *Mom's Heart (Mausoleum)*).
  So `SaveDiff.cards_pills` has been reporting boss encounters under a card's name, exactly
  as feared.
- **Section 3 is stages, and "one value per original character" is refuted.** **Index 0 is
  zero in every save we hold** — impossible for a table whose first row would be Isaac, the
  most-played character. Indices 1..12 carry 309, 298, 199, 206, 162, 208, 107, 109, 19, 67,
  50, 137, and index 1 (309) sits right on the number of runs the profile has. In a matched
  window — one game launch, the live save against the last backup — the indices that moved
  are **exactly** the stages the log declared with `Level::Init m_Stage`: 1, 2, 3, 4, 5, 6,
  7, 8, 10, 11. Fourteen cells = stages 1..13 plus an unused 0.
- **Section 8 is cutscenes, with one cell left over.** Same matched window: the log played
  cutscene 1 and cutscene 19, and section 8 moved at **index 19** (+1) and index 2 (+1).
  Index 19 is the identity mapping. Index 2 is the profile's largest cell (110) and rises
  once per launch, which is also how often the log prints `playing cutscene 1` — so either
  it is cutscene 1 under an off-by-one that index 19 contradicts, or it counts launches. One
  more solo run with a known ending separates them.
  *Careful with co-op*: of fifteen windows taken from the session folders, five had a
  cutscene in the log and no movement in section 8 at all. All five are run endings played
  in **online co-op**, whose progression goes to the shared profile. Co-op windows are not
  evidence about the personal save — which is itself worth knowing.
- **Section 10: the proposed split at byte 320 is wrong**, and so was the list that
  replaced it. `count` is 80 and `f2` is 320 in every file, but the payload is not two
  chunks meeting there — the keys run **straight across index 80 without a
  discontinuity**, so byte 320 falls in the middle, not on a boundary. That much holds.
  The reading that followed — "0..19 are zero, 20..24 are five small counters, and from
  index 25 the rest is a sorted key → count list of 1,364 pairs" — does not.

  **Measured on 2026-09-09, on four saves across two editions.** The section is
  **self-describing**, and reading it as one list is what made it look unsorted:

  ```
  words[0..19]   twenty zeros
  words[20]      11            constant in every save
  words[21]      the total, exactly the sum of the four sizes below
  words[22]      4             how many tallies follow
  then 4 x ( id, size, size/4 records of (key, count) )
                 ids 4, 2, 3, 1 in that order in every save
  ```

  A size is in units of two bytes and a record is eight, so `size / 4` records. Inside a
  tally the keys are **strictly ascending and each entity appears once**. Read as a single
  list from index 25, the same bytes show three descents and 445 repeated keys — those are
  the three intermediate `(id, size)` headers and the fact that the same entity is counted
  in several tallies. Both artefacts vanish when the boundaries are read from the file.

  So "20..24 are five counters" was really `11`, the total, the tally count, and then the
  first tally's own `(id, size)`.

  **One word is left over** after the last tally, in every save: 11,343 (Jan 2024) rising
  to 29,725 (Jan 2025). It is not slack — it grows with the profile. It nearly went
  unnoticed: the throwaway script that mapped the layout had an off-by-one that consumed
  it, and it came back only because the Rust reader disagreed with the script.

  The key decodes as before: **`(type << 20) | (variant << 8) | subtype`** — `0x00A00000`
  is type 10 variant 0, `0x02600100` is type 38 variant 1 — and the largest type, 951, is
  inside the game's entity range. What the four tallies count is **not known**: they hold
  the same entities with different numbers against each one. The triple is the same one
  `crates/wiki` already indexes bosses by (`Dataset::boss_key`), so the join exists the day
  the tallies have meanings.

  **Read by `core-save` since 2026-09-09** — `Save::bestiary_tallies()`, module
  `crates/core-save/src/bestiary.rs`, eight real-data properties in
  `crates/core-save/tests/bestiary.rs`. The tallies keep the id the file gives them and are
  given no names, for the same reason sections 5, 8 and 9 are still `Unknown`.

### What is left to do

1. **The one cell in section 8**: solo run, known ending, watch index 2. Twenty minutes.
   Needs a machine with the game running, so it is the item that travels worst.
2. ~~**Then rename**, together.~~ **Sections 3 and 6 renamed on 2026-09-09**, on a machine
   without the game — everything they needed was already measured. `Kind::PerChar` →
   `Kind::LevelCounters`, `Kind::CardsPills` → `Kind::Bosses`, `SaveDiff.cards_pills` →
   `SaveDiff.bosses`, plus the tables in `CLAUDE.md`, `docs/PROJECT.md` and
   `DESIGN-BRIEF.md`. Sections 5, 8 and 9 keep their `Unknown` names on purpose: they have
   the game's word for it and nothing else, which is the distinction the enum's own doc
   comment now states.
   **The blast radius recorded above was wrong in one place.** It said `Kind` "neither
   crosses the IPC nor drives product behaviour"; it *does* cross, inside
   `ipc::SectionCount`, so a variant's name is a wire value. Nothing caught it because the
   TypeScript mirror types that field as `kind: string` rather than a union — a type wide
   enough to hide a contract change. Now pinned by `crates/ipc/tests/summary_shape.rs`,
   ten variants against ten strings, with a second test that goes red if an eleventh
   variant is added without a row.
3. ~~**Decide what section 10 becomes.**~~ **Its structure is read as of 2026-09-09.**
   `Save::bestiary_tallies()` returns the four tallies, each a list of
   `(EntityId, count)` in key order, plus what the section declared about itself and the
   one word the layout doesn't account for. The reading is checked by the invariants that
   found it — the declared total accounts for every tally, the section is consumed but for
   that word, each tally is ascending and holds an entity once — so it keeps working on a
   patch and says so when it can't.
   **What is left needs the game**, and is listed in `docs/STATUS.md`: which of the four
   tallies counts what, and what the trailing word is. Both want a matched window — play a
   run, compare the save against the backup taken before it — which is the same instrument
   that answered sections 3 and 8, and it cannot be done from samples.

### Done when

Every one of the eleven chunks has a name backed by a measurement of its own, the table in
`CLAUDE.md` has no "to be identified" rows left, and no public name in `core-save` says
something the bytes contradict.

### What it is NOT

Not an occasion to start *interpreting* the newly named sections. Naming section 8
"Cutscene Counters" doesn't oblige anyone to decode which cutscene is which; it obliges us
to stop calling it `Unknown8`.

---

## B11 — Third-party licences travel with the bundle (implementation, packaging)

**Needs:** nothing — `bundle.resources` and two licence files; a bundle build says whether they travel.

Logged 2026-09-10: Vite copies only the hashed `determination-*.ttf` into `ui/dist`;
`ui/src/assets/fonts/determination/license.txt` and `readme.txt` stay in the source tree,
while the font's readme requires all files of the archive to accompany any redistribution
and CC BY 3.0 requires attribution. `crates/app/tauri.conf.json` has no `bundle.resources`.
Same gap for `dataset/ATTRIBUTION.md`, which CLAUDE.md says ships in the package. Fix when
packaging: `bundle.resources` (or the files under `ui/public/`), plus the credit line in
About (cycle 3).

---

## B12 — Design system cycle 1 follow-ups (implementation, cycles 2 and 3)

**Needs:** nothing — frontend, `cn()` and the scanner. Items 3 and 10 are contrast and state readability: they need a look before they can be decided, not the game.

Logged 2026-09-10, a list:

1. `cn()` doesn't register `--opacity-*` tokens (`opacity-muted opacity-disabled` both
   survive a merge) — add `classGroups.opacity` read from `theme/opacity.css`, with a
   test.
2. vue-i18n feature flags aren't defined in `ui/vite.config.ts`
   (`__VUE_I18N_LEGACY_API__` stays in the bundle, legacy API not tree-shaken) — add the
   `define` entries per vue-i18n's optimization guide.
3. Keyboard highlight contrast in Select and Command items: `secondary` #3A251D on
   `popover` #1B120E is about 1.28:1; Reka's Select gives items real DOM focus, so the
   highlight replaces the focus ring — back to design (an inset `ring` edge would fit
   "cyan means focus").
4. `Command` filters only when the search text changes; items mounted after a change
   (async palette results) leave their group hidden and `CommandEmpty` beside results;
   `CommandItem` doesn't prune its id from the group set on unmount; no tests for the
   filter; `CommandInput` always auto-focuses (make it a prop) — for the cycle 3 palette.
5. Scanner gaps: the literal-attribute check skips `position`, `align`, `side`
   (constants exist); the `dark:` pattern misses stacked variants like `hover:dark:`;
   `outline-none` on focusable elements isn't scanned; classes of the reset default
   scales (`text-sm`, `rounded-md`, `font-bold`, `shadow-*`) aren't flagged although they
   generate nothing.
6. `SelectTrigger` hover uses `row-hover`, a row's role, on a field — a `field-hover`
   role or `secondary`.
7. `Button` has no default `type="button"`: inside a future `<form>` every Button
   submits.
8. `Alert` uses `role="alert"` for diagnostics rendered with the page (assertive on
   mount); `role="status"` may fit.
9. `Progress` doesn't expose the unknown segment to assistive tech (`getValueLabel` with
   an i18n string).
10. A disabled segmented control loses its on-state: a disabled active `TabsTrigger` or
    pressed `ToggleGroupItem` renders exactly like an unselected one, while `Checkbox`
    keeps its tick and `Switch` its thumb position — back to design (a faint edge or
    underline would keep "which one" readable without reading as enabled).

---

## B14 — Choosing the game or saves folder by hand (implementation, cycle 3)

**Needs:** nothing, then a window — it is the *broken* chain it serves, which is this machine's normal state; the dialog plugin and a folder are all it takes.

Logged 2026-09-11, from sub-project 3.1: when the chain breaks (Steam missing, the game not
found, no saves) the profile screen says where and offers "Riprova", but not the two buttons
of `Schermate.dc.html` ("Scegli la cartella del gioco", "Scegli la cartella dei
salvataggi"). A button that does nothing is worse than none, so they wait for what makes
them work: the Tauri dialog plugin with its capability, a command that accepts a folder and
hands back a `SetupState` (the path travels inward only, never back out), the chosen folder
persisted in the settings file, and `discovery` trying it before its own search.

---

## B15 — Tearing a tab off into its own window, and back (implementation, after 3.7) — 🟡 built on 2026-09-13, **not yet measured on the machine**

**Needs:** nothing, then a window — the one open question is whether WebView2 keeps delivering pointer events outside the window, and only a hand on a mouse answers it.

**Built on `feature/drag-and-windows`, ahead of 3.7 rather than after it** (spec
`docs/superpowers/specs/2026-09-13-drag-and-windows-design.md`): the window port and its fake,
the handshake that seeds a newborn window, docking with the marker drawn in the target's strip,
the two closing rules, the payload-free events that make the profile, the scale and the plan the
app's rather than the window's, the hit test with its DPI conversion, the preview page, and the
gesture itself.

**What is still open is the one thing the entry said would decide it**: whether WebView2 keeps
delivering pointer events with the cursor outside the window. It needs a real window and a hand
on the mouse, and it has not been run. `lib/window/pointerSource.ts` is written as if the answer
were yes, behind an interface that is the only thing the other answer changes, with a 10-second
silence timeout that **cancels** the drag rather than landing a tab nobody released. The plan's
Task 1 is written to be run by the owner; Task 17's eleven checks wait on the same session.

Logged 2026-09-11, a product requirement from the owner: drag a tab out of the window and, on
drop, it opens in a new window; drag it back over the first window's tab strip and the two
merge again — **exactly as a browser does**.

### What we already have

- Tabs own locations (`stores/tabModel.ts`, pure and tested): open, close, move, navigate,
  "the bar is never empty". A tab is `{ id, location }`, never the view's content (B6), which
  is what makes it movable between windows at all.
- `TabStrip` drags within the strip (cycle 2's `moveIndex`); the title bar is ours
  (`decorations: false`, `lib/window/appWindow.ts` the only module that talks to the window).

### What the Tauri 2 documentation says (researched 2026-09-11)

- **A window at runtime:** `new WebviewWindow(label, { url, x, y, width, height, decorations,
  visible })`, permission `core:webview:allow-create-webview-window`; a capability whose
  `"windows"` glob matches the new label covers it without re-registration.
  ([webviewWindow](https://v2.tauri.app/reference/javascript/api/namespacewebviewwindow/),
  [capability](https://v2.tauri.app/reference/acl/capability/))
- **The cursor outside the window:** `cursorPosition()` (JS) / `cursor_position()` (Rust),
  desktop physical pixels, stable since 2.1.0, permission `core:window:allow-cursor-position`.
  ([window API](https://v2.tauri.app/reference/javascript/api/namespacewindow/))
- **Which window is under a point:** `getAllWebviewWindows()` with `outerPosition()` /
  `outerSize()` / `isMinimized()`; **no z-order API** (tauri#5656), so overlapping windows are
  ambiguous and the hit test is ours.
- **Between windows:** `emitTo(label, event, payload)` and `listen`.
- **The two traps.** `setPointerCapture()` in WebView2 is unreliable once the cursor leaves
  the control (microsoft-ui-xaml #8677, #8753), so a DOM-only drag can lose its `pointerup`
  outside the window; and `startDragging()` can't be started on a window that didn't receive
  the mousedown, so the torn-off window follows the cursor through `setPosition` from the
  origin window, not through the OS move.
- **Prior art to evaluate, not yet trusted:** `tauri-plugin-drag-as-window`
  (crabnebula-dev/drag-rs) starts a native OS drag of an element that becomes a window and has
  a `dragBack` for re-docking — which would sidestep the pointer-capture trap. Its maintenance,
  licence and behaviour with `decorations: false` have to be checked by making it answer, on
  the machine, before it enters `Cargo.toml`.

### The constraints that shape it

1. **The active profile is window-global today** (§4.1, §4.2): with two windows it becomes the
   app's, and every window's indicator and Progress screens must follow a change made in
   another. The profile store reads backend state already; what's missing is the event that
   says "it changed".
2. **B6 saves tabs**: with tear-off, a saved session is windows of tabs, not one list. Doing
   B15 after 3.7 means the persisted shape is decided once, knowing it.
3. **A tab dropped outside every window opens a window; a tab dropped on a strip joins it**;
   a window whose last tab leaves closes, and the main window never does (the bar is never
   empty is the same rule, one level up).
4. **Not verifiable on the development server**: fixtures run in one browser tab. The checks
   are a real `pnpm dev` window, with a mouse, on Windows at mixed DPI.

### Questions the task has to close

- Native drag (the plugin) or DOM pointer events with `cursorPosition` polling — decided by a
  spike on the machine, not by the documentation alone.
- The re-dock gesture: hover over the strip with a grace period (the WinUI write-up uses
  ~240 ms so a tab doesn't snap back instantly), or drop only.
- Where a torn-off tab's window opens: under the cursor at the drop, sized like the origin.
- Whether a secondary window has the navbar and sidebar, or tabs and content only.

---

## B17 — The profile screen is a welcome flow, not "Screen 0" (implementation, after design) ⏳ copy done on 2026-09-12

**Needs:** nothing, then a window — the welcome flow draws against fixtures (`?fixture=pick`); a real preview wants a real profile.

**Item 1 is done** (2026-09-12): the eyebrow "Schermata 0" and the intro that called the
screen a permanent state are gone, and the sidebar hint no longer says the app finds the game
from there. The screen now says what it is for — choose the save you are playing with.
**Items 2 and 3 stay open**: the welcome flow with a preview per save, and what remains under
Settings, are a design pass (sub-project 3.6).

Logged 2026-09-12, a product decision from the owner: the profile screen's copy describes a
settings page, and the owner wants the opposite — **on launch the app asks which save to
play with and shows a preview of it**, a welcome flow.

### What we already have

- `screens/ProfileScreen.vue` with `ScreenHeader` eyebrow `profile.eyebrow` ("Schermata 0" /
  "Screen 0") and `profile.intro` ("Non è un passaggio da attraversare una volta: è lo stato
  che decide ogni numero dell'app. Resta consultabile e modificabile per sempre."), in
  `ui/src/i18n/messages/{it,en}.ts`.
- The sidebar hint `sidebar.settingsHint` ("Da qui l'app trova gioco e salvataggi."), wired in
  `shell/sectionNav.ts`.
- The candidate list with its opaque ids, `SetupState`, and auto-selection when there is
  one save (sub-project 3.1).

### What's wrong with the copy, as decided

- "Schermata 0" is a design-file label, not something a user reads.
- "Da qui l'app trova gioco e salvataggi" and the intro are **false** as product: the screen
  isn't a permanent state page one may consult, it's the first thing the user meets and it
  should behave as a welcome, letting them pick the save and see what it holds before the
  rest of the app opens on it.

### What's missing

1. Drop the eyebrow and the intro; rewrite the sidebar hint. Copy comes from the design, not
   from the code.
2. A welcome flow: at launch, with more than one candidate, the app opens on the choice with
   a **preview** per save (the KPI strip already drawn by Progress: achievements, items,
   marks, last played); with one candidate, straight to the app as today. The flow stays
   reachable afterwards from the profile indicator, and B14's manual folders live in it.
3. The settings entry stops pretending to be this screen: what remains under "Impostazioni"
   is decided with the design.

### Done when

A first launch on a machine with two saves shows the two, each with its numbers, and no text
anywhere says "Schermata 0". Design first (`Schermate.dc.html`), then the usual spec → plan.

---

## B19 — The marked cell sits on the game's paper, not on a flat panel (implementation, `ipc` and `ui`)

**Needs:** the game — the paper sheet is a game asset, extracted from the user's copy at runtime, and the entry forbids a colour that resembles it.

Logged 2026-09-12, from the owner's review of Completion: a taken mark is drawn on a flat
light panel, and it should sit on the game's own paper sheet, the one the completion widget
draws under every symbol.

**Confirmed by the owner on 2026-09-12, with the reference**: the design's matrix cell drew
the mark on a sheet **taken from the game**, and that is the sheet to use — not a colour that
resembles it. Same rule as B33 for the achievement drawing.

### What we already have

- `marks/MarkCell.vue` paints a marked cell with `bg-mark-paper`, the token
  `--color-mark-paper: #e9dadf` in `colors.css` — a colour standing in for an image.
- The image exists on the same sheet the symbols come from: `completion_widget/paper_00.png`
  (the plain paper) and `paper_02.png` (the bloodied one, the state the game shows once a
  mark is taken; `design-export`'s `mark_symbol_fallback` describes it). B10's first item
  already measured it: 96×96 with the drawing at `x 0–84, y 3–82`, untrimmed.
- The icon protocol already crops this sheet for `mark/<column>/<tier>` (`ipc::mark_art`,
  `ipc::crop_png`).

### What's missing

1. One more address on the protocol, `mark/paper/<tier>` or the like, cropping the paper
   frame from `completion_widget.anm2` the way the symbols are cropped — the frame read
   from the anm2, not a hand-typed rectangle. Whether plain and bloodied are both used, or
   only one, is the design's call; the tier decides which.
2. `MarkCell` draws the paper as the cell's background image (a CSS variable bound by the
   template, per the frontend rules) and the symbol over it; the token `mark-paper` stays as
   the fallback colour for a machine without the game, or goes if the bars outfit covers it.
3. `mark_art.rs`'s test extends to the paper: it resolves, and to a different rectangle
   from every symbol.

### Done when

On a machine with the game, a marked cell shows the paper under its symbol, and the Kit
page's legend row shows the same. Without the game, nothing changes.

---

## B20 — "Non leggibile" leaves the matrix: close the 40 cells (measurement, then `core-save` and `ui`)

**Needs:** a measurement — one run of Mother with a Tainted character closes the 20 × 2 block; the dated series has been walked and cannot.

Logged 2026-09-12, a product decision from the owner: the matrix shouldn't say "non
leggibile". The state has to be resolved, not restyled.

### What we already have

- **40 cells** the app can't place: Mother and The Beast for The Forgotten and the 19
  Tainted characters, a 20 × 2 block in the bottom-right corner. Spacing puts them inside
  423–490 (Mother for the 14 originals starts at 423, The Beast at 457), but they are zero
  in every save collected, so `counter_index` answers `None` and the cell draws `unknown`
  (CLAUDE.md, "Counters and marks"; `docs/STATUS.md`).
- The rendering of that state: the dashed hatched cell in `MarkCell.vue`, the legend entry
  `completion.legend.unknown`, the KPI `completion.grid.unreadable`, the tooltip
  `completion.cell.unknown` ("la colonna non è localizzata per questo personaggio"), all in
  `i18n/messages/{it,en}.ts`.
- The two properties in `crates/ipc/tests/marks_real.rs` that pin every located base to the
  historical series.

### What's missing

1. **The measurement, which no code can replace**: one run of Mother, then of The Beast, with
   a Tainted character (or The Forgotten), with a backup of the save before and after. The
   cell that changes names the base, and the same three facts that pinned 404, 423 and 457
   have to hold — the achievement, the kill counter rising by one, index 188 naming the row.
   The backups in `save_backups\` are the series; `samples/` gets the two dated files.
2. Then the tables in `core-save` take the two bases, `counter_index` stops returning `None`
   for those rows, and the fixture counts in the spec (408 with 40 unknown) become 408 with
   0 — the `marks_real.rs` properties have to stay green on the whole series.
3. Then the UI drops what only those cells needed: the legend row "non leggibile" and the
   "non leggibili" KPI, whose count would be permanently zero. `MarkCell`'s `unknown` visual
   and the `nothingReadable` message **stay**: they are the degrade-never-fail path for a
   save whose counters section is missing or truncated, and that is a constraint, not a
   label to remove. What changes is that a healthy save never shows it.

### Done when

A full save shows no hatched cell and the legend has four entries; a truncated fixture still
draws every cell unknown and says why.

---

## B21 — A mark taken in multiplayer says so (`ipc` and `ui`; **two of three measured 2026-09-12**)

**Needs:** a real save — the bit only exists from the era the profile first won online, so a 642-era save is what makes the tests speak. The third measurement (local co-op) still wants a run.

Logged 2026-09-12, a product requirement from the owner: the matrix has to show whether a
mark was taken in multiplayer or alone. Today a cell knows only its level.

**Two of the three measurements this entry asked for were made the same day**, on a live
online Greed run; the third was briefly thought closed and is not. The implementation is
unblocked either way — what it draws does not depend on the local co-op answer.

### What was measured

A matched window around one online co-op run — Greed Mode, Cain, won, the whole session
inside the window — plus the dated series and the 22 folders in `online_logs\`.

1. **Bit 2 is "won online".** ✅ The run took `Greed × Cain` from 2 to 7, lighting bit 2 on
   exactly the cell its boss and character name. Across the series every date on which any
   cell gained bit 2 has an `online_logs\` session of the same day (6 of 6, over the period
   those folders cover), while ~60 marks taken on days without one gained bits 0 and 1
   only. The sharpest case is 2026-08-31: same day, same character, `Satan × Magdalene`
   3 → 7 **with** the bit and `Greed × Magdalene` 0 → 3 **without** — so it is a property of
   the run, not of the day.
2. **The personal save does record it.** ✅ Not "zero difference": 20 activity counters, the
   mark itself, and bestiary tallies 1 and 2 all moved. What did *not* move is achievements,
   items, challenges, bosses and sections 3, 5, 8, 9 — which is the half of the old claim
   that was right, and the half that made the whole claim look right.
3. **Local co-op — still open.** ❌ It was briefly called closed on the same day, on the
   argument that a local co-op win is the only thing that makes index 188 name two
   characters at once, which made 2026-07-22 and 2026-09-01 identifiable co-op days. **That
   argument is wrong**: 188 accumulates. A single win adds its bit and leaves the previous
   one standing — `2 → 6` on 2026-09-01, and the `4 → 12` the B8 spike report already
   recorded as "one thing that does not fit" — so two bits is two wins in the window, not
   two players in one run. The evidence that survives is weaker and by elimination: no day
   without an online session has produced a bit 2, and the owner confirms local co-op
   happened at least once. Settling it needs one local co-op win with a snapshot either
   side, the same protocol as point 2.

So the name is **"won online"**, narrower than the "multiplayer" this entry assumed, and
`sharedsave_*.dat` is *not* needed: that branch of the entry is dead.

### Still not closed by this

- **2026-06-29 and 2026-07-06** hold a bit 2 with no session folder. Neither is a
  counterexample — `online_logs\` starts on 2026-08-24, and on 07-06 index 188 names one
  character while two took marks, so the bit sits on a run the mask does not describe. What
  would settle it is knowing whether the game **rotates** `online_logs\` or keeps every
  session: if it rotates, the two holes explain themselves.
- The two **levels** (bits 0 and 1) are still unmeasured. B22 is that question.

### What to implement

- `ipc::marks::Cell::Known` grows the reading beside the level — the field names the bit,
  so `online: bool`, not `multiplayer: bool`.
- Tooltip and legend stop saying "terzo livello, significato non confermato" and say won
  online; the cell gets a visual from the design (a corner glyph or a second head, **not** a
  colour: colour is the level).
- Percentages stay forbidden, but for B22/B23's reason now, not because a bit is unread.

### Done when

The matrix draws "won online" on the cells that carry bit 2, the tooltip and legend stop
calling it a third level, and the reading is pinned by a test on the series — the structural
half already is, by `the_online_bit_never_stands_without_the_cleared_bit` in
`crates/ipc/tests/marks_real.rs`.

---

## B22 — Two counts per row: normal and hard, where hard implies normal (implementation, `ipc` and `ui`)

**Needs:** a real save — "hard implies normal" is logic over bits, but the two counts per row are only answerable against a profile.

Logged 2026-09-12, two product rules from the owner, one entry because they are the same
number:

1. **A mark taken on hard counts as taken on normal too.** Beating a boss on hard is the
   harder of the two, so a character who has the hard mark has the normal one whatever
   bit 0 says.
2. **Rows and footer show two numbers, not one**: how many bosses at normal, how many at
   hard, each over the readable cells.

### What we already have

- One tally everywhere: `started / readable`, started meaning "bit 0 or bit 1"
  (`ui/src/lib/completion/completionView.ts` `tallyOf`, `crates/ipc/src/marks.rs`
  `CharacterRow.started`). `MarksGrid.vue` draws it once per row, once per group header and
  once per boss in the footer; `CompletionKpis.vue` sums it.
- The cell already obeys rule 1 in its drawing: `markVisual.ts` picks the hard sprite when
  bit 1 is set, whatever bit 0 says. What doesn't obey it is the reading: `CellStatus` has
  `Normal`, `Hard` and `Both`, the tooltip says "normale e hard" for 3, and value 2 reads
  "hard" as if normal were missing.
- The observed values include a bare 2: the game does write hard without normal, so rule 1
  is a rule of ours, not a reading of the file. The count keeps the bit; the label doesn't.

### What's missing

1. **The reading**: `CellStatus.Both` goes; `Hard` means "hard, hence normal", and the
   tooltip and legend say so once ("hard" — the legend's "normale e hard" row disappears).
   `cellReading` is pure and tested: the expected values come from the rule above.
2. **The tally**: `Tally` becomes `{ normal, hard, readable, complete }` with
   `normal = cells where bit 0 or bit 1`, `hard = cells where bit 1`, so `hard ≤ normal ≤
   readable` always — a property to test on the fixtures and on `marks_real.rs`'s series.
   `complete` means `hard === readable`: the row is done when every boss is done on hard,
   which is what the game's own widget means by a full row. Whether a second, weaker
   "complete at normal" colour exists is the design's call.
3. **The IPC**: `CharacterRow.started` becomes `normal` and `hard` (a contract change,
   handed on with the mirror in `types.ts` and `summary_shape.rs`'s kin pinned).
4. **The grid**: two number columns on the right of every row, two in the group header, two
   per boss in the footer, and the KPI strip splits the same way. Layout from
   `Schermate.dc.html`, which today has one column.

### Done when

A row with twelve hard marks reads 12/12 · 12/12; a row with value 2 in one cell reads
1/12 · 1/12, not 0 and 1; the legend has no "normale e hard"; and no test derives its
expected number from the previous single count.

---

## B23 — The Completion KPIs: no "120 celle", no "40 non leggibili" (implementation, after design, with B20 and B22)

**Needs:** a real save, then a window — the tiles are what the strip above the matrix reads, and the matrix needs a profile to have a number in it.

Logged 2026-09-12, from the owner's review of the strip above the matrix
(`screens/completion/CompletionKpis.vue`, spec §"KPIs"): of the four tiles, two say nothing
the owner wants to read.

| tile today | value | verdict |
|---|---|---|
| marchi iniziati | started / readable | keeps its place, split by B22 |
| normale + hard | "120 celle", no denominator | **useless**: a count of cells with both bits, meaningful only while the bits were unread |
| personaggi completi | complete / characters | stays |
| non leggibili | "40 / 408 celle" | **not wanted**: the 40 are a gap in our tables (B20), not a fact about the player |

### What replaces them

With B22's two numbers and B20's closed cells the strip is the same three facts the rows
show, summed: **normal marks / total**, **hard marks / total**, **complete characters /
characters** — three tiles, or four with the design's fourth if it has one (the obvious
candidate is "characters complete at normal", the weaker of the two `complete`s). Each keeps
its denominator and no percentage is drawn (§5.3).

The `Unknown` tone and the `unknown` count don't vanish from the model: while a section is
missing the tiles read 0 / 0 and the alert above the grid says why (B20's rule). What goes
is the tile that puts an unreadable count on the same line as the player's progress.

### Done when

The strip on a full save has no tile whose value is a bare count of cells, and none whose
label says "non leggibili". Design first: it's the same pass that draws B22's columns.

---

## B27 — A table fills the page, or the mouse sizes it and the size is remembered per table (implementation, `ui` and `app`)

**Needs:** nothing, then a window — a table that fills the page or is sized by the mouse is judged at the size a window actually has.

Logged 2026-09-12, a product rule from the owner: a table either **scales with the page it
is open in**, or it is **resizable with the mouse**; in the second case its size is saved in
state, **per table**, so it never has to be resized again after a restart.

### What we already have

- Two virtualized tables, Unlock (`screens/unlock/UnlockTable.vue`) and the Collection
  (`screens/collection/CollectionTable.vue`), both scrolling inside a body capped at
  `max-h-unlock-body`, the token `--spacing-unlock-body: 560px`: a fixed height chosen from
  the design file, blind to the window. On a tall window the page ends with empty space
  under the table; on a short one the table scrolls inside a page that also scrolls.
- Their columns are already fluid: `grid-cols-unlock` and `grid-cols-collection` in
  `utilities.css` give the image and badge columns a token width and the text columns `fr`
  shares, so the width follows the page today. What doesn't is the height.
- One resize gesture exists, the sidebar (`shell/SectionSidebar.vue`, pointer events,
  `clampSidebarWidth`), whose width is shell state **not persisted** by decision, deferred to
  the sub-project that saves the session (3.7).
- Persistence: `settings.json` through `ipc::Settings` (one field today) — and the store,
  where a document per concern already lives (the queue).

### The two options, and the order

1. **Fill the page** first, because it is the option that needs no state: the table's body
   takes the height left under the facets and the header (`flex-1 min-h-0` on the page's
   column, the scroller `h-full`), and `--spacing-unlock-body` goes. The Plan's queue and the
   marks matrix are checked against the same rule. TanStack Virtual doesn't care: the
   scroller's height is measured, not declared.
2. **Resizable by hand** where filling isn't right (a table under other content, a split
   pane): a grip on the table's bottom edge, the sidebar's gesture generalised into one
   composable (`useResize`), a minimum and a maximum in tokens, and the height stored **per
   table** under a key that names it (`unlock`, `collection`, …), never a shared number.
3. ✅ **Where it is saved** — **answered by 3.7c** (2026-09-16, report
   `docs/superpowers/reports/2026-09-16-tabs-sizes-report.md`): a **named key beside `windows`**
   in the session document, which costs no migration and no version bump. The sidebar's width is
   in it and is the half of this entry that is done; a table's size joins it under a key that
   names the table, the day a table has one. A missing or unknown key reads as "fill the page".
   The value is in device-independent pixels and is scaled with B26, not stored scaled.

**Two of the three are what is left**, which is worth saying plainly because 3.7c could look like
this entry closing: the sizes have a home now and **nothing produces one**. A `tables` key was
deliberately not added to the document for exactly that reason — a named place for a value nothing
writes is one more thing to read and nothing to store.

### Done when

Unlock and the Collection use the whole height of a tall window and nothing under them
scrolls twice; a table given a grip keeps the size it was dragged to across a restart, each
table its own; and no fixed body height survives in the tokens.

---

## B29 — The Collection's filter: no "Faccette", no values with nothing behind them (implementation, `ui`, after design) ⏳ the two sight fixes done on 2026-09-12

**Needs:** nothing, then a window — one filter bar with a fold and multi-select dropdowns: a new primitive and a design pass, both frontend.

**The two things wrong on sight are fixed** (2026-09-12), on both screens: the drawer says
"Filtri" / "Filters", and a value with nothing behind it is no longer offered at all — it used
to sit there with a 0 and a disabled checkbox. **The shape the owner wants stays open**: one
filter bar with a fold for the less important controls, and multi-select dropdowns in place of
the checkbox columns, which needs a new primitive and the design pass that redraws the bar for
Unlock and the Collection at once.

Logged 2026-09-12, from the owner's review of the Collection's filter drawer
(`components/facets/FacetDrawer.vue`, which was `screens/collection/CollectionFacetDrawer.vue` when this was written and became shared by N8): the presentation is to be redone, and two
things are wrong on sight.

- **"Faccette"** (`collection.facets`, and `unlock.facets` on Unlock) is the design file's
  word for the control, not a word a player uses. The drawer needs a name that says what it
  does — filter — or no title at all.
- **Values that match nothing are listed.** `collectionFacetOptions` offers every value of
  each facet's set, so the quality "non valutato" and the origin "non indicata" appear with
  a 0 and can't be picked (the drawer's own comment: "a value that would give nothing, and
  isn't picked, can't be picked"). The owner reads them as noise: a value with nothing behind
  it in this profile's view is not offered. Same rule for Unlock's drawer, which shares the
  mechanism (`facetLabels.ts`, `FacetDrawer.vue`).

### The shape the owner wants

- **All the filters together**, in one place, not a drawer per screen with columns of
  checkboxes: the state toggle, the search and the facets are one filter bar.
- **The important ones visible, the rest behind "mostra più filtri"** (or the like): the
  bar shows two or three controls at rest and unfolds the others on request; which ones are
  important is decided per screen in the design (for the Collection, the state and the
  quality; for Unlock, the state and what it unlocks).
- **Pool, quality, origin, kind as multi-select dropdowns**: a `Select`-like control that
  takes several values, shows the picked ones on its trigger ("Qualità · 3, 4") and the
  count beside each option, instead of a column of checkboxes. The kit has `Select`
  (single) and `Checkbox`; a multi-select is a new primitive, Reka's `Combobox` or
  `Listbox` with `multiple`, styled by the kit, with its own Kit page row.

The pure functions stay: `collectionFacetCounts` already knows which values are empty and
what picking one would give. What changes is the presentation — the options are the values
with a count, in the catalog's order, inside a multi-select — and the wording, from the
design pass that redraws the bar for both screens at once.

### Done when

Both screens filter from one bar with a fold for the less important controls; pool, quality,
origin and kind are multi-select dropdowns; neither shows a value with 0 unless it is
currently picked; and the word "Faccette" appears nowhere in the app.

---

## B30 — Where the app writes, in Settings, and movable (implementation, `app`, `store` and `ui`)

**Needs:** nothing, then a window — the folder, its size and the button that moves it are `app`, `store` and `ui`; the move is watched once.

Logged 2026-09-12, from the owner while reading the About dialog: About says the app writes
one file, `isaacdome.db`, in the app's data folder — and the owner wants to **see that folder
and be able to move it**, which is a setting, not a credit.

### What goes where

About keeps what **identifies and credits** the app: the name, the version, the fan-made
line, the licences, and the three promises — the promises are the product's contract and the
first thing a stranger should read, so they stay where a stranger looks. What moves is the
**fact about this machine**: which folder, how big, and the button that changes it. A path is
something you act on; a promise is something you read.

### What we already have

- `crates/app/src/lib.rs`: `StoreState` opens `app_data_dir()/isaacdome.db` once, lazily, and
  keeps it in managed state; `settings_file.rs` writes `settings.json` in `app_config_dir()`.
  Neither path has ever crossed the IPC, and **neither may**: a path carries the Windows
  username (CLAUDE.md, "Don't cross the IPC boundary").
- `store` with its versioned schema and two migrations, and the queue as one JSON document.
- The Settings section of the sidebar (Profile, Tabs, Appearance since 3.5c).

### The constraint that shapes it, and the way out

A path can't cross the boundary as a path — but the user has to see *where* their data is, or
the setting is a button with no subject. The way out is the one `discovery` already uses for
save files: **the view carries a hint, not a path** — the folder's display name and its parent
in short form, enough to recognise it, never the full string. The dialog that changes it is
the Tauri dialog plugin (B14 needs the same plugin), and what comes back travels **inward
only**: the frontend asks "move it here" with the handle the dialog gave, and the backend
answers with a new hint.

### What's missing

1. A **Data** page under Settings: the folder's hint, the database's size, and what is in it
   (the queue, the goals — a row each, from `store`), plus the same for `settings.json`.
2. A command that **moves** it: close the handle in `StoreState`, copy the file, verify it
   opens at the destination, write the new location in `settings.json`, and only then remove
   the old one. It is the app's own data, so the order is copy → verify → switch → delete,
   never move → hope.
3. The location in `settings.json` (`dataDir`), read at startup by `StoreState`; absent means
   the default, which is what every install has today.
4. **What happens when the saved folder is gone** on the next launch — an external drive, a
   folder the user deleted: the app says so on the Data page and falls back to the default
   rather than failing to open, and the goals and the queue read as unavailable, which is a
   state `plan` and `queue` already have.

### Done when

The Data page names the folder in words, says how large the file is, and moving it leaves the
queue and the goals intact at the new place; a folder that has gone missing is said, not
crashed on; and no full path has crossed the IPC boundary.

---

## B33 — The achievement drawing sits on the game's own backing, as the game shows it (implementation, `ipc` and `ui`)

**Needs:** the game — the backing "has to be looked for in the game's files", which is the whole point of the entry.

Logged 2026-09-12, from the owner's review of the cards: the achievement picture is drawn on
a flat colour, and it should sit on **the image the game itself puts behind it** — the
right one has to be looked for in the game's files, not approximated.

### What we already have

- `components/graph/AchievementArt.vue` draws the 263×176 achievement picture on
  `bg-mark-paper`, the same flat token B19 replaces under the marks; its own comment says
  "the way the game shows it on a note", which names the intent and not the sheet.
- The picture itself is served through the icon protocol from the user's own copy, so the
  backing can come the same way: a crop of a sheet, resolved by an anm2, addressed by the
  protocol, with the bars fallback when the game isn't there.
- The game shows an achievement in two places, with two backings: the **unlock popup** at
  the end of a run (the achievement drawing on a paper note that slides in) and the
  **achievements page of the pause / stats menu** (the drawing on a paper-like frame per
  slot). Which of the two the card should imitate is the design's call; the sheet behind
  each is the measurement.

### What's missing

1. Find, in `resources/packed`, the anm2 and sheet the game uses for the achievement popup
   and for the achievements menu — a `design-export` probe listing the candidates under
   `gfx/ui/achievement` and `gfx/ui/...menu` with their frame rectangles, so the choice is
   made on the real files. The name is recorded here once it is measured, not guessed now.
2. One more address on the icon protocol for the backing, cropped from its frame; the
   `AchievementArt` component draws it under the picture with the game's own padding, on
   every screen that shows an achievement (Next steps, Unlock, Plan, the Kit page).
3. The flat token stays only as the fallback without the game.

### Done when

On a machine with the game, an achievement card shows the drawing on the same backing the
game does, and the entry names the sheet and frame it came from.

---

## B36 — A mark and a counter say which boss they mean, and link to it (measurement, then `ipc` and `ui`)

**Needs:** the game — the ten bosses' keys come from the dataset and need none, but the counters resolve through the catalog, and that half does.

Logged on 2026-09-12, with 3.5d. `RequirementView::Mark` names a cell of the completion matrix
("beat Delirium with Cain") and `Counter` a threshold on a tally; both draw in the blocked menu
as names you cannot follow, because neither carries a page.

Why it isn't done: the twelve columns are not twelve entities. Ten of them plausibly map to a
boss the dataset has a page for, but **Boss Rush** is a room-and-event and **Greed** is a game
mode, and a column → entity table written from the names would be a curation nobody measured —
exactly what `docs/STATUS.md` keeps `Unknown` rather than guessing. The counters' labels are the
bosses' own names, which is a second, easier case: those could resolve through the catalog.

What it needs: the entity key of each of the ten bosses, taken from the same place
`wiki_target::boss` takes it — the portrait's file name — rather than from a name match; and a
decision, in words, for the two that are not entities.

Closes when: a mark's entry in the blocked menu opens the boss's page for the ten that have one,
the other two say what they are without pretending to be entities, and the mapping is pinned by
a test that reads it from the catalog rather than from a literal table.

---

## B39 — A tab carries its state between windows: filters, scroll, what it was showing (implementation, `ui`, after 3.7's shape) 🟡 built on 2026-09-16, **not yet seen in a window**

**Needs:** nothing, then a window — what a tab carries between windows is `ui`; that it survives the move is seen.

Logged 2026-09-13, from the owner while checking the tear-off: *"si devono tenere anche filtri,
scroll ecc quando tratti uno spostamento di tab"*. A tab dragged into another window arrives at
the right page and **forgets everything about how it was being read** — the Unlock facets, the
search text, the sort, where the virtualized table was scrolled to.

### Why it is not a small fix

What crosses between windows today is `TabSeed` — everything a tab *is*, minus its identity. And
a tab, today, **is a location**: a route name and a query. Everything else lives somewhere that
is not the tab:

- the **facets and the search text** are the screens' own `ref`s, recreated when the screen
  mounts (`screens/unlock/`, `composables/useSearch.ts`);
- the **scroll offset** belongs to the DOM element, and to `@tanstack/vue-virtual`'s measurement
  of it;
- the **view stores** (one `stores/views.ts` since N8, three files when this was written) are per window, and keyed
  by nothing: two tabs on the same screen already share them.

So "a tab keeps its state" means **a tab owns its state**, which is a different shape from the
one the shell has had since 3.1. It is the same shape 3.7 needs in order to save a session — a
tab that can be written down and read back — which is why this waits for that decision rather
than inventing a second one.

### What it probably looks like

1. A tab's state becomes an object it owns: `{ location, view? }`, where `view` is a small,
   serializable record a screen declares for itself (facets, query, sort, scroll offset).
2. A screen reads it on mount and writes it back as it changes — through one composable, so no
   screen invents its own storage, and so the shape is uniform enough for 3.7 to persist.
3. `TabSeed` needs no change at all: it is `Omit<Tab, 'id'>`, so the day a tab holds its view the
   view crosses windows with it. That property was built in on purpose (2026-09-13) and this is
   the case it was built for.
4. Scroll is the awkward one: an offset only means something against a list of the same length,
   so it is restored **after** the data is there, and a list that changed underneath keeps the
   top rather than guessing.

### Done when


**Built on 3.7a** (`feature/tabs-state`), report
`docs/superpowers/reports/2026-09-16-tabs-own-their-state-report.md`. The shape is the one this
entry guessed at: a history entry became `{ location, view? }`, one composable hands a screen its
reading, and `TabSeed` needed no change — which is the property it was written by subtraction to
have. Unlock, the Collection, Runs and Search each declare what they keep; the scroll offset
carries the row count it was taken at, so a list that changed underneath keeps its top.

**It stays open until somebody looks at it**, which is what its own `Needs:` line asks for and
what no test in this repo does. The five things to check are listed at the top of the report,
unticked.

A tab dragged into another window comes back showing what it was showing: the same facets, the
same text in the search, the same sort, and the same place in the list — and the same is true of
a tab that survives a restart, because it is the same mechanism.

---

## B41 — Starting with Windows, so no run is lost to a launch the app missed (implementation, `ipc`, `app` and `ui`, after design)

**Needs:** nothing, then a window — the registry, a pure `launch_intent`, and a switch. Closing it wants an installed build, a logout and a login.

Logged on 2026-09-14, from the owner: *"aggiungiamo opzione avvio al lancio in impostazioni in
modo che a prescindere da quando apro il gioco IsaacDome può essere sempre aperto e leggere tutte
le run"*. Design written the same day —
`docs/superpowers/specs/2026-09-14-autostart-design.md`. It is §11 of the background design coming
due: starting with Windows was deferred there *"to the same conversation as M4's watcher"*, and
the watcher has landed.

### What already exists

Everything except the login entry. The app outlives its windows, the tray brings one back, a
second launch is handed to the instance already running, and `start_archive` backfills
`online_logs\sessions\` and then watches `log.txt` — backfill and live being one function is what
makes a late start harmless *within* a game launch.

### What is missing, precisely

`log.txt` is rewritten at every launch of the game, so the hole is narrow and real: **a game
launch followed by another game launch, with the app never having run in between.** Play, quit,
play again, open IsaacDome — the first session is gone and nothing can bring it back. The switch
closes that one case, and the prose under it has to say that rather than promise "no run is ever
lost".

### The three decisions the design takes

1. **The registry is the only source of truth.** `ipc::Settings` gains no field. `is_enabled()`
   reads `StartupApproved\Run` as well as `Run`, so the app can see the user disable the entry
   from Task Manager's Startup tab — a mirrored boolean in `settings.json` would report "on" for a
   login that never happens.
2. **A login launch is silent.** The plugin writes one argument into the Run value; `setup` builds
   the tray and starts the archive either way, and only opens a window when that argument is
   absent. The decision is `ipc::launch_intent`, a pure function, because `app` is not tested.
3. **The switch is inert in development builds.** `current_exe()` in a `pnpm dev` run is
   `target\debug\app.exe`, and a switch flipped once while testing leaves that path in the
   developer's login, surviving `cargo clean` and failing silently at every boot.

### Closes when

The Background screen carries a third switch, first of the three; an installed build turned on
and logged out of comes back with the icon in the tray, no window, and the archive already
following the log; disabling the entry from Task Manager shows as off the next time the screen is
opened; turning the switch off twice raises nothing; and a `pnpm dev` run leaves the registry
untouched.

**Not closed by the above, and not lost either**: the uninstaller does not remove the Run value —
nothing tells a running app it is being uninstalled, and there is no installer configuration in
the repo yet (`tauri.conf.json` says `"targets": "all"` and nothing more). The day that
configuration is written, an NSIS uninstall hook deletes the value from both keys. That is part of
this entry, not a new one.

---

## B42 — Two Cargo tables are downloaded, committed, and read by nothing (implementation, `wiki`, small) 🟡 half closed on 2026-09-14

**`player.json` has its reader** (`feature/small-three`, `5da5296`): its `parent` is cross-checked
against the `parent` every character page states in its own infobox, and the test goes red the day
the two stop agreeing. Measured on the committed snapshot: **32 named forms carry both, all 32
agree.** Matching the two by `player`'s own `id` column was tried first and is a trap — that column
is as unreliable as the infoboxes' (`Isaac` 14, `Magdalene` 2, the bug the character map exists to
route around) — so an id match agreed by accident on names it was never comparing. Names are what
the two sources share. The check enters through `for_tests`: it asks a question about the data we
ship and no command of the app ever asks it.

**It also found what nobody was looking for**, which is now **B45**: the eight `player` rows it
could not compare are not eight facts but one — **four character pages are missing from the
snapshot**.

**`stage.json` is the open half**, and it is a decision, not code: unlike `player` it has no second
source anywhere in the repo to be checked against, so there is no reader for it to earn. The
recommendation was to drop it from `TABLES` and from `dataset/raw/cargo/`.

**The owner decided on 2026-09-14 to keep it, and this entry stays open with it.** So the third
state the entry was written against — a committed artefact nobody reads — is now a *chosen* state
rather than an unnoticed one, and that is the only part that changed. What would close the entry is
unchanged: either `chapter` earns a reader with a test behind it, or the file goes. Nothing else in
the repo reads `stage:` references by anything but name.

**Needs:** nothing, then a decision — `wiki-snapshot`'s query and `dataset/raw/cargo/`, both committed.

Logged on 2026-09-13, noticed while adding the transformations' five fields to the same
query. `crates/wiki-snapshot/src/api.rs` downloads ten Cargo tables; `Raw::load` puts seven
of them in `Tables`, plus `version`. **`player.json` and `stage.json` are written to
`dataset/raw/cargo/` and nothing ever opens them.**

### Why it is worth an entry rather than a deletion

`player` carries **`parent`** — the relation from a Tainted character to the base form it is
a variant of. That is exactly the fact behind the two identity bugs M2 found on 2026-09-12:
the game gives a Tainted character the base form's name, so by name alone 141 of 396
character references resolved to nothing and "Ultra Greedier as Keeper" picked T. Keeper.
Resolution goes by the wiki's id now, and this table is a second, independent source for the
same relation — `Infobox::Character.parent` reads it from the page, and nobody has checked
the two against each other.

`stage` carries `chapter`, which nothing needs today; the 1444 `stage:` references resolve by
name and have no entries at all.

### What it needs

A decision, not code first: either the two tables earn a reader — `player.parent` as a
cross-check on the character map, with the disagreements counted the way the transformations'
two item lists are — or they leave the query. What must not continue is the third state, a
committed artefact nobody reads and no test would notice going stale.

### Closes when

Either both tables are read by something with a test that would fail if they stopped
agreeing, or they are gone from `TABLES` and from `dataset/raw/cargo/`, with the reason in
the commit body.

---

## B47 — Seven infobox templates the snapshot does not know, and 591 pages behind them (analysis, then a product decision)

**Needs:** nothing — one query against the wiki answers it, and the decision after it is about
what belongs in the package, not about what is possible.

Measured on 2026-09-14 while closing B45, by asking the wiki for every `Template:Infobox *` and
counting the transclusions of each in namespace 0. The snapshot enumerates seven templates. The
wiki has these as well:

| template | pages | what they are |
|---|---|---|
| `Infobox entity` | 247 | entities that are not bosses |
| `Infobox monster` | 126 | ordinary enemies |
| `Infobox pickup` | 97 | pickups |
| `Infobox card` | 66 | cards |
| `Infobox rune` | 28 | runes |
| `Infobox stage` | 27 | stages |
| `Infobox grid entity` | 0 | nothing transcludes it |

`Infobox characters` was the eighth and is B45, closed: 4 pages, and they were the four that
mattered because the game's own save has a cell for each of those characters.

### Why this is an entry and not a task

**It corrects a sentence this repo says often.** `pageKey`, `categoryOf` and `Dataset::entry` all
state that stages, rooms and concepts *have no page* — and for a part of them that is a fact about
**our fetch**, not about the wiki. The 49 `Target::Concept` targets B34 measured are "wiki pages
the game gives no id"; 97 pickup pages, 66 card pages and 28 rune pages are sitting behind a
template nobody enumerates, and the 13 references the graph still cannot interpret live in that
neighbourhood. Whether any of them resolves is unmeasured — that is this entry's analysis half.

**And it is not obviously desirable.** The dataset ships inside the binary. 591 pages is a large
fraction again of the 1113 we carry, for content the app has no screen for: the wiki section is
built around what a profile can unlock, and an enemy's page is not that. Cards and runes are the
plausible exception, because a challenge or an achievement condition names them.

### Closes when

Each of the seven is decided, in writing and with its reason: enumerated, or declared out of
scope. A template we enumerate needs its `InfoboxKind` and its kind's folder; one we decline
needs a line saying why, so the next person measuring this finds the answer instead of the
measurement. `Diagnostics::unknown_infoboxes` (B45) is what will keep either decision honest: a
template we never enumerate never appears there, but one we start fetching without teaching the
parser will.

---

## B54 — 54 sections are discarded once each, and they are not the noise the counter was built for (analysis, then `wiki`)

**Needs:** nothing — the counter is in the committed `dataset/wiki.json`.

Logged on 2026-09-15, out of B53. `discardedSections` reports **2267 occurrences over 63 titles**,
and reading it as one number hides the shape: **2213 of them are six known titles** the build drops
on purpose — `Trivia` 889, in-game footage 840 across three spellings, `Gallery` 359, `References`
102, `Requirements` 15, `Audio` 7 — and the remaining **54 occurrences are 54 distinct titles, each
appearing exactly once**.

A title that appears once is not a category the parser declined. It is one page's own heading, and
the whole section under it is gone. They fall into three families:

- **A second subject the page describes.** `Dark Esau` (28 lines of behaviour on Tainted Jacob's
  page), `Friendly Charger`, `Black Judas`, `Lazarus Risen`, `Blood Clots`. This is the family B53
  came from, and it is the one that needs a decision rather than a line: a page has one entry, and
  these sections belong to something else that shares it.
- **A near-miss on a kind we already have.** `Items Interactions` beside the accepted
  `Item Interactions`, `Active Item Interactions`, `General Strategies`, `Infinite Synergies` and
  its three parenthesised variants. `section_kind` is a closed list of spellings, and these are
  spellings.
- **Genuinely something else.** `Algorithm`, `Modifiers`, `Combinations`,
  `Component Types and Qualities`, `Drops`, `Item Exclusion`.

**The second family is the cheap half and should not be done by reflex**: `Good Items` and
`Bad items` are one page's editorial lists, not `Notes`, and folding them in would put a judgment
into a kind that does not make one. Every spelling added has to be read on its own page first,
which is why this is an analysis entry and not a patch.

**What makes it worth doing at all** is that the counter cannot tell you any of this: 54 against
2267 reads as rounding, and `discardedSections` was built to show a parser losing ground, which is
exactly what a title seen once does not look like. This entry exists so the next reading of that
number starts from the split rather than the total.

### Closes when

Each of the 54 is placed in one of the three families **by reading its page**, the near-misses are
either accepted into `section_kind` or refused in writing with the page that refused them, and the
second-subject family has a decision — with the counts re-measured, because 54 is a number from a
snapshot and the wiki gains headings.

---

## B55 — The log has no clock and Steam keeps one (analysis, then `run` and `log-watch`)

**Needs:** nothing to read the file; **a machine with Steam** to have one at all.

Logged on 2026-09-15. `Steam\logs\gameprocess_log.txt` records every launch and exit of
`isaac-ng.exe` with a wall clock — **93 of them on this machine since 2025-06-26**, in the shape

```
[2026-09-15 20:46:34] AppID 250900 adding PID 4908 as a tracked process "...\isaac-ng.exe"
[2026-09-15 21:04:08] AppID 250900 no longer tracking PID 4908, exit code 0
```

**This is the thing the archive does not have.** `CLAUDE.md` records it about `online_logs\`: *"The
folder's name carries a wall clock, which the log itself does not have"* — and that is true of solo
play too, where there is no folder name either. A run in the archive today can be ordered but not
dated. It was used the day it was found: the window of 2026-09-15 could only be read as one run
because this file said exactly one launch began inside it.

It also gives the **exit code**, which separates a quit from a crash, and the pairing of launches
with `[Continue, …]` seed lines would let the fold tell a relaunch from a quit-to-menu — a
distinction `run::resume` currently cannot make and which is exactly what left cell 2 of section 8
open.

**And there is a per-run clock next to it, which is worth more.** `Steam\logs\cloud_log.txt` records
every sync of the five remote files, including `rep+gamestate1.dat` — the mid-run save the game
creates when a run starts and deletes when it **ends**. Measured on 2026-09-15, 27 lines name that
file, and the end of the run this repo spent the evening on is in there to the second:

```
[2026-09-15 20:46:31] [AppID 250900] File is in sync rep+gamestate1.dat
[2026-09-15 21:04:09] [AppID 250900] Need to delete file rep+gamestate1.dat
[2026-09-15 21:04:10] [AppID 250900] Delete OK for file rep+gamestate1.dat
```

So the delete **is** synced and **is** dated: that is a run ending, not a launch ending, which is
the limit the launch log cannot get past. Still unmeasured: how far back the file is kept (27 lines
is not many for 93 launches, so it is either short or a run rarely crosses a sync), and whether a
run *starting* is as visible as one ending — the line above says "in sync", not "created", because
that run was already open when the game launched.

**Half of that was measured on 2026-09-16, on the second machine, and the two logs came out
nothing alike.** The game has not been launched there since Jan 2025 and has since been
uninstalled, so it is the retention question asked from the other end:

| file | size | AppID 250900 |
|---|---|---|
| `Steam\logs\gameprocess_log.txt` | 6.9 KB | **0 lines** |
| `Steam\logs\cloud_log.txt` | 848 KB | 1760 lines, `2024-01-18` → `2025-01-14`, 181 naming a `gamestate1.dat` |

So the **launch** clock is a short window — it rolled away an entire era of play and says the game
was never run here — while the **cloud** clock held ~12 months on the same machine. That is the
asymmetry this entry asks to decide rather than assume, and it points the same way the paragraph
above does: the per-run clock is worth more than the per-launch one, and it is also the one that
survives. It does not make it a history — 848 KB is still a size, not a promise.

**What has to be decided, not assumed**, and why this is an entry: it is a **rolling** log, so it is
a window and not a history; its path is Steam's, not the game's, and `discovery` finds neither
today; and a per-launch clock is not a per-run clock — a launch holding three runs dates all three
the same. None of that makes it useless, and all of it has to be in the model rather than in the
reader's head.

### Closes when

Either the launch timeline is a source the archive reads, with the three limits above represented
rather than smoothed over, or the entry says in writing why a rolling per-launch clock is not worth
the dependency — with the retention measured, because "93 since June 2025" is one machine on one
day.

---

## B56 — Steam knows when each achievement was unlocked (analysis, then `ipc`)

**Needs:** nothing to read the files; **a machine with Steam**.

Logged on 2026-09-15, from the same sweep as B55. Two files:

- `Steam\appcache\stats\UserGameStats_<accountid>_250900.bin` — 3781 bytes here, binary KeyValues,
  carrying `AchievementTimes`;
- `Steam\appcache\stats\UserGameStatsSchema_250900.bin` — 181 KB, the achievement definitions
  (`Magdalene`, `Basement Boy — Beat basement without taking damage.`, icon hashes).

Together they are an **achievement timeline**: not *what* is unlocked, which the `.dat` already
says better, but **when**. This app's whole question is "what am I missing, and what is worth
playing tonight", and every ordering it offers today — Next steps, the plan queue — is derived from
the graph, never from what the player has actually been doing lately.

**What it is not.** It is Steam's cache of Steam achievements, so it covers the 637 that have a
Steam achievement and says nothing about a profile the game keeps locally; it is per Steam account,
not per save slot, so a machine with two profiles gets one timeline for both; and the format is
binary KeyValues, which is a parser this repo does not have and would have to justify.

**The cheap half first**: the schema file also maps achievement id to the game's own English name
and description, which `catalog` currently reads out of the game's XML. Whether the two agree is a
cross-check that costs one pass and would catch a drift nobody is watching.

### Closes when

The timeline is either a source with a stated scope (Steam-wide, achievement-only) or refused in
writing, and the schema cross-check against `catalog` has been run once either way — a disagreement
there is worth more than the timeline.

---

## B57 — The game writes down where it saves, and `discovery` guesses (implementation, `discovery`, small)

**Needs:** ~~**the game**~~ **nothing** — **retagged 2026-09-16**: the file **outlives the
install**. On the second machine the game is uninstalled (no `appmanifest_250900.acf`) and
`steamapps\common\The Binding of Isaac Rebirth\` is an orphan holding `data`, `mods` and
`savedatapath.txt` — no `resources\`. So the fixture is available on a machine that cannot run
anything else in the `the game` bucket, and writing the parser and its tests needs no install.
What still needs one is watching the file be *rewritten on launch*, which is the sentence below
and not the entry.

Logged on 2026-09-15. `…\common\The Binding of Isaac Rebirth\savedatapath.txt`, rewritten on every
launch:

```
This file is purely informational. Changing it will have no effect on saving or loading data.

Save Data Path: C:\Users\stefa/Documents/My Games/Binding of Isaac Repentance+/
Modding Data Path: D:\SteamLibrary\steamapps\common\The Binding of Isaac Rebirth/mods/
```

**And the second machine holds the other half of the entry's own argument** (2026-09-16), which is
why the retag is worth more than one less skip. Same file, different machine:

```
Save Data Path: C:\Users\stefa/Documents/My Games/Binding of Isaac Repentance/
Modding Data Path: C:\Program Files (x86)\Steam\steamapps\common\The Binding of Isaac Rebirth/mods/
```

No `+`, and `Documents\My Games\` on that machine holds **only** that spelling — so the two
samples are the two branches of the fork the entry says `discovery` cannot see, and both are now
fixtures rather than an argument. The mixed separators reproduce exactly, on a second install and
a different Steam library root, which promotes that from an observation to something a parser may
rely on. The `Modding Data Path` also ends up naming an install the launcher no longer lists —
another reason it is a *candidate to verify*, never the answer.

`discovery` covers four shapes of save location by construction — with and without the `+`, Steam
Cloud and not — and it has to, because it must work at a stranger's house. **This file is the
game's own answer**, and it is the one place that cannot be wrong about the spelling: the two
Documents folders differ by a character, and which one exists depends on a history the app cannot
see.

**It does not replace the search**, and saying so is the point of the entry: the file is in the
install directory, so it exists only when the game is found; it is informational, so a mismatch is
possible in principle; and its separators are mixed (`C:\Users\stefa/Documents/...`), which is a
small parsing fact and a large clue that it is generated text, not a contract. It belongs as a
**first candidate**, checked and then verified like any other, never as the answer.

### Closes when

`discovery` reads it when the install is known, prefers it as a candidate, and still finds the
folder without it — with a test that turns red if the file becomes the only path that works.

---

## B58 — The mark tables were located on the 2026 series and are read on a 2025 save (analysis, `core-save` and `ipc`)

**Needs:** **a real save** of the 641-achievement era — `20250626` on the machine this was written
on, `20250112` on the second one (2026-09-16). **The era is the requirement, not the file**: the
sentence used to name one file and read as a property of `samples/`, which is the one thing in a
`Needs:` line that is about a machine. That there are two independent 641-era saves is worth more
than either — the two anomalous cells below are an argument from **one** profile, and a second one
of the same era either repeats them or ends the hypothesis.

Logged on 2026-09-15, found by a property that would not hold and should not have been made to.

`BLOCKS_14[10] = 423` (Mother) and `BLOCKS_14[11] = 457` (The Beast) came out of the **2026**
historical series, each pinned three ways on a day its cell changed. Nothing checked whether those
indices mean the same thing in the **June 2025** save, which is a different era — it declares 641
achievements where the 2026 ones declare 642, so at least one section's length moved between them,
and the marks live in the section *after* that one.

**The evidence that this is not hypothetical.** Two shapes that occur nowhere in the 2026 saves
occur in `20250626`:

| cell | value | why it is odd |
|---|---|---|
| `Isaac × The Beast` | **1** | bit 0 alone outside Greed — 0 occurrences across the 2026 series |
| `Isaac × Greed` | **2** | bit 1 alone *in* Greed — and in Greed bit 1 is Ultra Greedier, which implies Greed |

Either the 2025 save genuinely holds those combinations, or **those two indices address something
else in that era** and we are reading a neighbour's cell. The second is the cheaper explanation for
a cell that is anomalous in exactly the two columns whose bases were derived rather than
documented — and The Beast is the boss that did not exist before Repentance+.

**What this puts at risk.** `marks_real.rs` walks `dated_series`, which includes the 2025 save, so
every property there is already reading those cells; they pass because they compare counts and
transitions, not values. `ipc`'s matrix would draw a mark for a 2025 profile from the same tables.
Nobody has been told any of this, which is the part worth fixing first.

### Closes when

Either the bases are checked against the 641 era — the section lengths of both saves read side by
side, which is one pass — or the tables declare the era they hold for and everything that walks a
series says which files it may apply them to. A wrong cell here shows a mark nobody earned, which
is the failure this project has already paid for twice.

---

## B59 — The convention scanner reads a comment as if it were code (implementation, `ui`, small)

**Needs:** nothing — the scanner and its rules are `ui/scripts/scan-conventions.mjs`, committed.

Found on 2026-09-16 while writing `useTabView` (3.7a). The rule that forbids calling a Tauri
command outside `lib/ipc/` is `/\binvoke\s*\(/` tested against the **whole file**, so a comment
saying *"a screen never calls `invoke()`"* trips the rule that forbids calling it. The comment was
reworded and the branch is clean; what is left is the shape.

It is small and it is not cosmetic: **a rule that cannot tell a mention from a call will
eventually refuse a correct explanation of itself**, and the reflex it trains — reword the comment
until the scanner stops complaining — is the opposite of what the comment is for. The same holds
for every rule in that file matched against raw text, which is most of them.

### What it probably looks like

Strip line comments and block comments before testing, or test against a body with comment ranges
blanked out. Not a parser: these are `.ts` and `.vue` files and the rules are regexes, so the
cheap version is a pre-pass that blanks `//…`, `/*…*/` and the contents of `<!--…-->`. Strings are
a different question and are deliberately left alone — a rule matching inside a string literal is
usually matching a real thing.

### Closes when

A file whose only `invoke(` is inside a comment scans clean, and a test in the scanner's own
fixtures says so. Every existing rule keeps its current verdict on the repository as it stands —
if blanking comments changes an answer anywhere, that answer is a finding and goes in the report.

---

## B62 — A source folded into no runs cannot be told from one never folded (implementation, `store`, needs migration 5)

**Needs:** nothing to write it — the fix is one nullable column and two functions. What it waits
on is **approval for migration 5**, which is a decision and not a machine.

Found on 2026-09-16, as the first test written for B60. The last line of `cached_runs` is

```rust
Ok((!runs.is_empty()).then_some(runs))
```

so a source that was read and folded into **zero** runs answers `None` — the same value as one
that was never folded, and as one folded under older rules. Reproduced: a launch holding an intro
cutscene and nothing else caches `None` where `Some(0)` is the truth.

**No user sees this today**, which is why it is an entry and not a fix in flight. The only
production caller is `crates/app/src/commands/runs.rs`, and it drops a `None` source from the runs
list — which is the right thing to show for a launch with no run in it anyway. What is already
wrong is the sentence next to it: *"the source will be folded again the next time its log is
read"*. For a runless source that is not a consolation, it is a description of a loop — it will be
folded again, produce nothing again, and cache nothing again, forever.

**Why it needs a migration.** The `runs` table carries `rules_version` **per row**, and zero rows
have nowhere to put one. The fold's version has to live on the source:

```sql
ALTER TABLE sources ADD COLUMN folded_rules_version INTEGER;
```

Nullable and additive, no data rewritten: an existing file gets `NULL`, which reads as "never
folded" and is exactly right for every row that predates it. `cache_runs` sets it, `cached_runs`
returns `Some(vec![])` when it matches and no rows are found.

### Closes when

Migration 5 is approved and applied, `cached_runs` tells the three states apart, the parked test
`a_launch_with_no_run_in_it_caches_an_empty_list_and_not_nothing` is back in
`crates/log-watch/tests/ingest.rs` and green, and the comment in `runs.rs` says which of the three
it is dropping.

---

## B63 — Nothing notices when a test disappears (implementation, `scripts/check`, small)

**Needs:** nothing — `scripts/check` already parses the run's output for the skip summary.

Found on 2026-09-16, while deciding what to do with the worktrees. Comparing the abandoned branch
`develop-old-it` against `develop` turned up one file present there and absent here:
`crates/ipc/tests/profile.rs`, **647 lines, 27 tests**. It was not lost in the re-root of
2026-09-07, which is what it looked like at first — `develop` had it, and

```
ed3c15d 2026-09-12 feat(ipc): answer the graph's profile questions from the save
  crates/ipc/tests/profile.rs  | 647 ------------------------------------------
  crates/ipc/tests/progress.rs |  95 +++++++
```

deleted it in the same diff that added `progress.rs`. The message says why the **new** file could
not be called `profile` — *"`ipc::profile` already means something else … two meanings of the word
in one crate is a trap"* — and the rename took the original with it.

**Every gate stayed green for four days**, and this is the entry: `cargo test` reports what ran,
never what stopped existing. `scripts/check` counts skips and declarations precisely, because a
skip is the silence it was built to hear — but a **deleted** test makes no sound at all. It is not
a dimmed suite, it is a shorter one.

**What was gone**, which is why it is worth a tool and not just a note: `profile_id`'s properties,
`resolve_active`'s choice logic, and the three that guard the rule `CLAUDE.md` states in full — the
candidate view hiding the Steam account id, the Windows username masked whatever the source, and
`setup_state`'s diagnostics leaking neither. `crates/ipc/tests/reasons.rs` keeps one narrow case of
it (an `io::Error`'s message not reaching a `SaveReason`) and nothing else did.

**Recovered on 2026-09-16** and green on today's code: the port needed four `game_data: None` and
two reasons that had become typed since. No regression was hiding — the code kept honouring them —
but for nine days nothing was checking.

### Closes when

A run says how many tests it ran, and a drop is loud. The cheap shape: `scripts/check` already
reads `cargo test`'s output into a file for the skip summary, so it can total the `test result: ok.
N passed` lines and compare against a committed floor, the way `unlock_size.rs` pins a ceiling.
**Measured, and it is one line**: over the same file the script already keeps,

```sh
grep -oP 'test result: ok\. \K\d+' "$output" | awk '{s+=$1} END {print s}'
```

reads **965** before this recovery and **992** after — exactly the 27 that came back. The number
that would have made the deletion loud was one `grep` away from a file the script had already
written.
Whatever the shape, the property is the one this incident breaks: **a commit that removes tests
must not be able to come out greener than one that does not.**

---

## Closed entries

**33 entries have closed**, and they are in `docs/completed/backlog-closed.md` with
the reason and the numbers each one measured. The list below is so that a question starting
"was this ever looked at?" does not need that file opened.

- **B1 — Item detail: primary and secondary effects (an **analysis** task)** — closed 2026-09-05
- **B2 — A challenge's reward** — closed 2026-09-05
- **B4 — An item's unlock tree (this is M2)** — closed 2026-09-07
- **B5 — Global search: one thing searches all of them**
- **B7 — Bringing the comments into English too** — closed 2026-09-07
- **B8 — What a real `log.txt` actually contains (a **spike**, blocks M4)** — closed 2026-09-08
- **B10 — The design export pack: what the design tool had to measure by hand** — closed 2026-09-15
- **B13 — The marks map moves to `ipc` with the Completion screen** — closed 2026-09-11
- **B16 — The brand mark in the navbar is the app icon, and the icon is `primary`** — closed 2026-09-12
- **B18 — No white flash at launch: a splash with the app logo** — closed 2026-09-12
- **B24 — Clicking a section navigates, and the section lights up** — closed 2026-09-12
- **B25 — About is a dialog, not a page** — closed 2026-09-12
- **B26 — Scaling the whole interface from Settings** — closed 2026-09-12
- **B28 — Unlock calls a Tainted character by its base name (bug, `ipc`; the data is right)** — closed 2026-09-12
- **B31 — Dragging a row lifts the whole card, and one component does it everywhere** — closed 2026-09-13
- **B32 — "Prossimi passi" is hard to read: a name that says what it is, and cards rewritten** — closed 2026-09-13
- **B34 — A linked concept with no id is called a `Pickup`, and that name was the whole confusion** — closed 2026-09-14
- **B35 — What a node unlocks links to its page too** — closed 2026-09-13
- **B37 — Searching from the goal: "voglio giocare Greed Mode, cosa devo giocare?"** — closed 2026-09-13
- **B38 — Three pickup quotes ship an undecoded HTML entity** — closed 2026-09-14
- **B40 — A transformation's infobox has no rows** — closed 2026-09-14
- **B43 — Four screens virtualize a list under Unlock's name** — closed 2026-09-14
- **B44 — The `'M` quote looks broken because it is meant to** — closed 2026-09-14
- **B45 — Four characters have no page in the snapshot, and nothing notices** — closed 2026-09-14
- **B46 — A transformation has a page and no way to open it** — closed 2026-09-14
- **B48 — An empty category says "no page with this name" when nothing was searched** — closed 2026-09-15
- **B49 — A block-level template reaches the screen as its own source** — closed 2026-09-15
- **B50 — A transformation has no picture** — closed 2026-09-14
- **B51 — The sentence that says how you become a transformation is thrown away** — closed 2026-09-14
- **B52 — What `n` means in a `{{dlc|…}}` code, and the 1832 spans waiting on it** — closed 2026-09-15
- **B53 — Three pages carry `{{infobox monster}}` and the parser skips them** — closed 2026-09-15
- **B60 — A log with no run in it, and the test that says there is no such log** — closed 2026-09-16, and its own premise was wrong
- **B61 — An empty profile is a shape `samples/` has never held** — closed 2026-09-16, and it is an instrument, not a fixture
