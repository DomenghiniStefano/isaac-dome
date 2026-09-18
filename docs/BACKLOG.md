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

**Every `, then a window` line is gathered in one place**: *"What only a window can say"* in
`docs/STATUS.md`, written on 2026-09-16 because the lines had accumulated across five reports and
nobody could see how much of it there was. The reports keep their own copies as the record of their
day; the gathered list is the live one, and **ticking a line there is what closes it**.

That list and the board's `UAT` were two things until the evening of 2026-09-17, when the owner
asked that **every change pass through Trello**. `UAT` now holds **one card per group** of that
list — twelve of them, one per sub-project — with the group's checks written out in the card's
description. Not one card per check: sixty-five cards would be a wall, and the two copies would
have to be kept in step by hand with nothing checking them.

**So the two are now the same subject in two shapes, and the drift is real**: a check ticked on a
card does not tick itself here, and nothing compares them. The gathered list in `docs/STATUS.md`
stays the one that is *read*, because it is beside the reports that produced it; the card is what
makes the work visible on the board. If they disagree, the file is right about what was written
and the card about what was done.

## Where the state lives, since 2026-09-17

**Whether an entry is open, started, built or closed is on the Trello board, not in this file.**
This document had been keeping it in prose, and it had stopped being true: on the morning of
2026-09-17 the preamble declared the total **eight times** — 25, 28, 29, 30, 30, 29, 28, and "26
that morning" — the bucket list under it summed to 25, and counting the headings gave 24. Three
answers, one file, one day. A running total written by hand is stale the moment the next entry is
opened, and this one had a command in it telling the reader not to trust the list above it.

The division of labour:

- **The board holds what changes.** One card per open entry: the `Needs` line with its reason in
  the description, `### Done when` as a checklist, and what blocks it as labels — `READY`,
  `WAITING`, `NEEDS SAVE`, `NEEDS GAME`, `NEEDS RUN`, `NEEDS STEAM`, `NEEDS WINDOW`, plus `BUG`
  and `STOPPED`. The lists are `Inbox`, `BACKLOG`, `In Progress`, `UAT` and `Done`, and a card's
  position inside a list is its priority — the same rule the `plan` crate states for its own
  queue.
- **This file holds what doesn't.** What an entry is, what already exists, what is missing, what
  it would take to believe it, and how it got here. That is prose, it belongs in git beside the
  code, and no card would carry it.

A heading no longer carries `⏳` or `🟡`: that is the column its card is in. It keeps the
sentence, because *"built on 2026-09-16, not yet seen in a window"* is a dated fact and not a
state. `✅` stays on closed entries, for the same reason — a closed entry never changes again, and
closed entries are history this file keeps rather than state the board tracks.

**Nothing checks the two against each other.** A card and an entry can drift and only a reader
would notice. A script could compare them, but it would need the network and a personal token, so
it could only ever be a report on the machine that has both — never a gate. `scripts/check` has to
stay green on a fresh clone, and there is no CI here by choice.

## How this list keeps growing

The chronicle below is the part worth keeping, and its lesson is in it: almost nothing here was
found by looking for it.

**B34 closed because tagging it meant reading it** and it turned out not to be finished; B38
closed; B44 opened and closed the same hour, as not a defect; then B40 and B43 closed and B42 half
closed on `feature/small-three`, which opened **B45** — four characters with no page, found by the
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

B48, B49, B52 and B53 closed on 2026-09-16. **B53 opened out of B52** — a counter that had been
reporting `{}` since the day it was added — and **B54 out of B53**, which is where this list keeps
finding things. **B55, B56 and B57** came from a sweep of what the machine actually keeps, run
because a claim had been made from two folders and stated as though it came from all of them: none
of the three is a run history, and all three are sources this project did not know it had. **B59**
came out of writing a comment: the convention scanner matches its rules against the raw file, so a
comment naming a forbidden call trips the rule that forbids it.

**The same sweep, run that evening on the second machine**, because a `Needs:` tag is the one thing
in this file that is about a machine and not about the work. It opened **B60** — a log the game
wrote with no run in it, which the suite cannot hold — and it **retagged B57** from `the game` to
`nothing`, because `savedatapath.txt` turns out to outlive the uninstall. One entry changed tag and
one entry is new, and both came from looking at a machine instead of at the list.

**B61** opened the same evening out of the same series: measuring the header's `0x10` meant opening
the slot nobody plays, and it turns out to be the one profile shape this project has never read.
**B63** the morning after, while deciding what to do with the worktrees: a comparison against an
abandoned branch turned up 27 tests that a commit had deleted four days earlier, with every gate
green.

**B60 closed** — the log with no run is in `samples/launches/`, a folder of its own — and its own
premise turned out to be wrong, which the weakened guard proved by going red for the wrong reason.
**B62 opened** out of the first test written for it: a source folded into zero runs caches as one
never folded. **B61 closed** the same evening and stopped being a fixture: an untouched profile is
zero everywhere except the bestiary, which makes it the clean instrument the eleven-chunks gap in
`docs/save-format.md` has been missing. Both closings began with a test going red against an
expectation written an hour earlier. **B62 closed** with migration 5, the one entry in this file
that was waiting on an approval rather than on a machine or on an idea.

**And on 2026-09-17 the top of the board was stale twice in a row.** B3 was picked up as the next
entry and two of its three halves turned out to be closed already — the items are the Collection,
the "search inside a list" is 3.10's filter bar — leaving only the challenges, which became 3.11.
The entry immediately under it, B6, is **built in full** across 3.7a, 3.7b and 3.7c, and what
holds it open is the window nobody has opened, which lives in `docs/STATUS.md` and not here: its
card moved to `UAT` without a line of code. Neither entry had been re-read since it was written —
B3 is from 2026-09-05 and still asked for TanStack Table, which 3.3a had declined for a measured
reason.
**The defence is cheap and it is what happened here**: read the entry against the code before
designing anything. A position in a list is a priority somebody set on a day, and the work done
since can have overtaken it.

The `a measurement` tag is the same subject as *"What only a machine with the game can answer"* in
`docs/STATUS.md`, which collects the ones that are instruments rather than entries. Closed entries
carry no tag: nobody goes looking for a task that is done.

---

## B3 — Lists and search: challenges and items (implementation, after design) ✅ closed on 2026-09-17, built and **not yet seen against the game's own menu**

**Closed as sub-project 3.11.** Spec `docs/superpowers/specs/2026-09-17-challenges-screen-design.md`,
plan `docs/superpowers/plans/2026-09-17-challenges-screen.md`, report
`docs/superpowers/reports/2026-09-17-challenges-report.md`.

**Two of its three halves were already closed, and nobody had read the entry since 2026-09-05.**
The items are the Collection screen; "search inside a list" is the filter bar (3.10) — the entry
was written before either existed, and it still asked for TanStack Table, which 3.3a had already
declined for a measured reason. What was actually missing was the **challenges**: `core-save`
read section 7 and `catalog` parsed `challenges.xml` with their rewards, and nothing in front of
them. The app could say *"Godhead si sblocca con Sfida 33"* and had no way to say what Sfida 33
was, whether you had done it, or what it took.

**What it took to believe it** — three measurements, one of which is deliberately *not* a
confirmation:

- **Challenge `n` is cell `n`, cell 0 unused.** 39 of 39 rows agree, 21 from one side and 18 from
  the other; the off-by-one reading breaks 13 of the same 39. In `docs/save-format.md`, guarded by
  `crates/ipc/tests/challenges_real.rs`, and the guard was mutated to check it goes red.
- **The wiki has a page for all 45**, so every row can carry its conditions — with the guard for
  the day that stops being true.
- **Whether `unlocked_by` is "all of" or "any of" is undecided.** All 13 finished-and-gated
  challenges have every gate done, which satisfies *both* readings. It is read as "all of", the
  blocked rows **name** the achievements they wait for so the claim can be disbelieved, and the
  measurement that would settle it is registered in `docs/STATUS.md`.

---

## B6 — Multiple tabs and session restore (implementation, after design) built across 3.7a, 3.7b and 3.7c on 2026-09-16, **not yet seen in a window**

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

## B11 — Third-party licences travel with the bundle (implementation, packaging) ✅ closed on 2026-09-16

**Closed on 2026-09-16**, and it found an M5 blocker on the way: **`pnpm build` could not produce a
bundle at all.** It compiled and then stopped on `Couldn't find a .ico icon`, although
`crates/app/icons/` holds exactly the default set Tauri looks for. Declaring `bundle.icon`
explicitly fixes it. Nothing ever runs `pnpm build` to the end — `scripts/check` does not, by
design, since it downloads WiX and NSIS and takes minutes — which is why it had been sitting there.

**The second finding could only come from extracting an installer.** A resource's target *name* is
honoured when files are staged beside the exe and **dropped** when they are packaged: mapping
`ATTRIBUTION.md` to `wiki-ATTRIBUTION.md` staged it renamed and shipped it as `ATTRIBUTION.md`,
beside a `license.txt` and a `readme.txt` that no longer said whose they were. So the subject goes
in the **directory** and the basename is left alone — the only shape where the staged layout and
the installed one agree.

Verified end to end: `msiexec /a` on the MSI and `7z l` on the NSIS setup both list
`licenses/determination/license.txt`, `licenses/determination/readme.txt` and
`licenses/wiki/ATTRIBUTION.md`. Commit `fix(app): the installer can be built, and the licences
travel inside it`.

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

## B14 — Choosing the game or saves folder by hand (implementation, cycle 3) ✅ closed on 2026-09-17, built and **not yet seen in a window**

**Needs:** nothing, then a window — it is the *broken* chain it serves, which is this machine's normal state; the dialog plugin and a folder are all it takes.

Logged 2026-09-11, from sub-project 3.1: when the chain breaks (Steam missing, the game not
found, no saves) the profile screen says where and offers "Riprova", but not the two buttons
of `Schermate.dc.html` ("Scegli la cartella del gioco", "Scegli la cartella dei
salvataggi"). A button that does nothing is worse than none, so they wait for what makes
them work: the Tauri dialog plugin with its capability, a command that accepts a folder and
hands back a `SetupState` (the path travels inward only, never back out), the chosen folder
persisted in the settings file, and `discovery` trying it before its own search.

**Closed on 2026-09-17**, inside 3.8 rather than on a branch of its own: the welcome's "nothing
found" branch is where the two buttons belong, and shipping it without them would have been that
very button. Report `docs/superpowers/reports/2026-09-17-welcome-flow-report.md`.

Three things the entry did not know. **Half its backend already existed** — `discovery::Options`
carried `game_dir` and `save_dir`, `scan_override` was written, and `SaveSource::Override` already
reached the wire as `CandidateSource::Manual`. **No capability is needed**: the dialog is opened
in Rust, and capabilities gate `invoke` from the webview, so the npm package is not installed
either. And the folders **cannot live in `ipc::Settings`**, which crosses the IPC — they are in
`settings_file::Stored`, whose `save` preserves them, so moving the scale slider cannot forget the
folder you chose.

**The case the entry did not name, and the app now does**: a folder you pointed at that holds no
save is not "we found nothing". `discovery::Diagnostic::NoSavesInChosenFolder` and
`MissingReason::NoSavesInChosenFolder` keep the two sentences apart, because only one of them is
about a choice the reader made.

It still owes the window, and it is the one line of 3.8 a browser cannot answer: a dialog is not
a fixture.

**Closed on 2026-09-17**, inside 3.8 rather than on a branch of its own: the welcome's "nothing
found" branch is where the two buttons belong, and shipping it without them would have been that
very button. Report `docs/superpowers/reports/2026-09-17-welcome-flow-report.md`.

Three things it did not know. **Half its backend already existed** — `discovery::Options` carried
`game_dir` and `save_dir`, `scan_override` was written, and `SaveSource::Override` already reached
the wire as `CandidateSource::Manual`. **No capability is needed**: the dialog is opened in Rust,
and capabilities gate `invoke` from the webview, so the npm package is not installed either.
And the folders **cannot live in `ipc::Settings`**, which crosses the IPC — they are in
`settings_file::Stored`, whose `save` preserves them so that moving the scale slider cannot
forget the folder you chose.

**The case the entry did not name, and now the app does**: a folder you pointed at that holds no
save is not "we found nothing". `discovery::Diagnostic::NoSavesInChosenFolder` and
`MissingReason::NoSavesInChosenFolder` keep the two sentences apart, because only one of them is
about a choice the reader made.

---

## B15 — Tearing a tab off into its own window, and back (implementation, after 3.7) — built on 2026-09-13, **not yet measured on the machine**

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

## B17 — The profile screen is a welcome flow, not "Screen 0" (implementation, after design) ✅ closed on 2026-09-17, built and **not yet seen in a window**

**Needs:** nothing, then a window — the welcome flow draws against fixtures (`?fixture=pick`); a real preview wants a real profile.

**Item 1 is done** (2026-09-12): the eyebrow "Schermata 0" and the intro that called the
screen a permanent state are gone, and the sidebar hint no longer says the app finds the game
from there. The screen now says what it is for — choose the save you are playing with.
**Item 3 closed on 2026-09-16** with sub-project 3.6a (report
`docs/superpowers/reports/2026-09-16-settings-tabs-report.md`): what remained under Settings was
the Tabs screen the sidebar had been pointing at since 3.1 with nothing behind it, and it exists.
**Item 2 stays open and is the whole of what is left** — the welcome flow with a preview per save.
It was deliberately not designed alongside 3.6a: it is a product decision about the first thing a
stranger sees, and it is the owner's.

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

**Built on 2026-09-17 as sub-project 3.8** — spec
`docs/superpowers/specs/2026-09-17-welcome-flow-design.md`, report
`docs/superpowers/reports/2026-09-17-welcome-flow-report.md`. The four decisions the owner took:
the welcome is a **full-screen takeover**, living as a state **above the router** and never as a
route; it takes both "you have to choose" and "nothing was found"; a card carries three counts
plus a line for what could not be read; and the **same takeover is the way back**, from the
profile indicator, so `ProfileScreen` keeps the chain and the sections and lost the candidate
table.

**The "KPI strip already drawn by Progress" this entry named did not exist** — what exists is
`CompletionKpis` on Completion and the indicator's edition, slot and date — so the strip was
designed rather than reused. It is the same shape of error as B22's item 3 the same morning: an
entry that names something it has not opened.

The line it still owes is the window: the flow draws on `?fixture=pick` in a browser, and nobody
has seen it in the app.

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

## B22 — Two counts per row: normal and hard, where hard implies normal (implementation, `ipc` and `ui`) ✅ closed on 2026-09-17, built and **not yet seen in a window**

**Items 1, 2 and 3 landed on 2026-09-17 in the morning; item 4 the same evening**, with B23,
report `docs/superpowers/reports/2026-09-17-completion-columns-report.md`.

**What item 4 turned out to be, and what it was not.** The entry said "layout from
`Schermate.dc.html`" — that file has **one** total column, so there was no layout to copy and
the shape is a decision this sub-project took: each column carries **its own denominator**
(`12/12 · 12/12`, which is what this entry's own "Done when" writes), the group header became
the matrix's own grid so its two numbers land *under* their headings, and the footer became
**two rows** rather than two numbers stacked in a 40px column — the name column was already
there to say which row is which, and it states the relation the pair exists for.

**One thing only a browser could say, and it changed the markup.** The group header was a flex
row ending in `ml-auto`, which lands at the *container's* right edge; the container is
`min-w-full` so the rows paint the whole card, and it is wider than the tracks. The numbers sat
some 130px past their own columns — invisible in every test, obvious in the first screenshot.
It is a grid now, with everything that is not a total sharing one cell (`col-start-1
-col-end-3`).

**And the colour moved from the row to the number.** It was `tally.complete` — hard everywhere
— painted on the single slot; it is now per number, so a number that fills its denominator is
done and `0/0` is unreadable rather than finished. Isaac reads full at normal and short at
hard, which is exactly what he is. The `0/0` guard is a test, not a hope: an equality alone
calls an unreadable row complete.

What landed earlier the same day:

- **The reading.** `CellStatus.Both` is gone. A cell with bit 1 is `Hard`, whatever bit 0
  says, and the tooltip says "hard" once — `completion.cell.both` is deleted in both
  languages. The legend needed no change at all: it already listed five rows with no "both"
  and drew `bits: 3` as hard, so the drawing had been right and only the *reading* was wrong,
  exactly as the entry's "What we already have" says.
- **The tally.** `{ normal, hard, readable, complete }`, with `hard <= normal <= readable`
  asserted as a property over every row and column of the reference profile — and guarded
  against vacuity, because that inequality holds trivially on a profile that never took a
  hard mark. `complete` is `hard === readable`. **The reference profile loses a complete
  character to that**, 3 → 2: Isaac's last cell is a bare 1, so he has a level everywhere and
  the second level in eleven of twelve. Losing him is the point of the change.
- **The IPC.** The field is on `MarksTotals`, not `CharacterRow` — this entry named the wrong
  struct, and the frontend never read a per-row count off the wire at all, it tallies
  `row.cells` itself. `started` becomes `normal` and `hard`; `types.ts` regenerated,
  `cell_and_totals_json_shape_is_pinned` updated, and `cross_check.rs` now asks the Python
  reference for **both** counts instead of one, which is the assertion that would catch
  `hard` being "fixed" to require bit 0 as well.
- **The KPI strip** splits the same way — `marks at normal` and `marks at hard` over the same
  denominator — because `both` could not survive `CellStatus.Both`. That tile counted the
  overlap of a set with its own superset.

What the grid did **until item 4**: the pair lived in the one number slot the grid already
had, as `normal/readable · hard`. It was truthful and it was not the layout; the second column
is item 4's, and it is there now.

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
  **Measured 2026-09-17 (B58), and it makes rule 1 a little less ours**: a cell goes
  **1 → 2**, four times across the 638-era series. The bare 2 is not a mark taken on hard by
  someone who never took it on normal — it is the normal mark *overwritten*, because the
  value replaces the one before it instead of accumulating. That is only evidence about how
  the file is written, not about what the bits are called, so `MarkLevel` stays `Base` /
  `Second`; but `normal = bit 0 or bit 1` below is now corroborated by the file rather than
  only by the product rule.

### What's missing

1. ✅ **The reading**: `CellStatus.Both` goes; `Hard` means "hard, hence normal", and the
   tooltip and legend say so once ("hard" — the legend's "normale e hard" row disappears).
   `cellReading` is pure and tested: the expected values come from the rule above.
2. ✅ **The tally**: `Tally` becomes `{ normal, hard, readable, complete }` with
   `normal = cells where bit 0 or bit 1`, `hard = cells where bit 1`, so `hard ≤ normal ≤
   readable` always — a property to test on the fixtures and on `marks_real.rs`'s series.
   `complete` means `hard === readable`: the row is done when every boss is done on hard,
   which is what the game's own widget means by a full row. Whether a second, weaker
   "complete at normal" colour exists is the design's call.
3. ✅ **The IPC**: ~~`CharacterRow.started`~~ **`MarksTotals.started`** becomes `normal` and
   `hard` (a contract change, handed on with the mirror in `types.ts` and the JSON shape
   pinned). The struck name is the entry's own error, kept because it is worth knowing why
   it was harmless: `CharacterRow` has no count at all, the frontend tallies `row.cells`
   itself, so the wire only ever carried the whole-matrix totals.
4. ✅ **The grid**: two number columns on the right of every row, two in the group header, two
   rows in the footer — `Schermate.dc.html` has one column, so the shape was decided here and
   not copied (see the head of this entry). The KPI strip was split already: `both` could not
   outlive `CellStatus.Both`, so it went with item 1 rather than waiting here.

### Done when

A row with twelve hard marks reads 12/12 · 12/12; a row with value 2 in one cell reads
1/12 · 1/12, not 0 and 1; the legend has no "normale e hard"; and no test derives its
expected number from the previous single count.

---

## B23 — The Completion KPIs: no "120 celle", no "40 non leggibili" (implementation, after design, with B20 and B22) ✅ closed on 2026-09-17, built and **not yet seen in a window**

**Closed with B22 item 4**, in the same pass and the same branch, report
`docs/superpowers/reports/2026-09-17-completion-columns-report.md`. The strip is **three**
tiles: marks at normal, marks at hard, complete characters.

**The fourth tile was offered by this entry and is declined.** "Characters complete at normal"
would put back, on the same line, the number B22 had just taken away — the reference profile
differs by exactly one character between the two readings (Isaac), and a strip holding both
readings of the same word argues with itself. Three tiles, the same three facts the rows show.

**Where the unreadable count went**, since this entry asks that it not vanish: it is a gap in
our tables and not a fact about the player, so it stays where it happens — every group header
prints its own (`6 non leggibili`, `34 non leggibili`), every unreadable cell says so, and
`readable` is still on the model because the alert above the grid reads it. What left the
model is `unknown` and `cells`, which no tile drew any more; a field nothing reads is the
shape B42 is an entry about.

**B20 is not a prerequisite and never was.** This entry reads as though it waits on the 40
cells being closed; it does not — removing the tile is what stops the 40 from being reported
as the player's progress, and closing them is B20's own business.

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

## B29 — The Collection's filter: no "Faccette", no values with nothing behind them (implementation, `ui`, after design) ✅ closed on 2026-09-17, built and **not yet seen in a window**

**Closed as sub-project 3.10.** Spec `docs/superpowers/specs/2026-09-17-filter-bar-design.md`,
plan `docs/superpowers/plans/2026-09-17-filter-bar.md`, report
`docs/superpowers/reports/2026-09-17-filter-bar-report.md`. One `FilterBar.vue` replaces the
drawer and the toolbar on **three** screens, a `MultiSelect` primitive joins the kit, and the
judgment that used to live in a `computed` is a tested module.

**Two things this entry had wrong, corrected by reading rather than by guessing:**

- **It says "both screens" and the screens are three.** It was written before N3 unified the
  engine and the Run diary became the third list on it. All three moved together — two ways of
  filtering in one app was the thing worth ending.
- **"Faccette" was already gone.** The 2026-09-12 fix had renamed the key's *value* to `Filtri`
  in all three blocks, so the fourth line of the card was satisfied before the work began. The
  keys themselves went with the control that named them.

**And one the work found, which only the new shape made visible**: the state row counted every
row while the dropdowns counted what the rest of the filter leaves, so with a character picked
the Run diary said `2 / 5` above a row summing to 5. The owner's call, mid-work, was to make the
whole bar say one kind of thing; the three screens' own tallies went with the prop they fed.

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

## B39 — A tab carries its state between windows: filters, scroll, what it was showing (implementation, `ui`, after 3.7's shape) built on 2026-09-16, **not yet seen in a window**

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

## B41 — Starting with Windows, so no run is lost to a launch the app missed (implementation, `ipc`, `app` and `ui`, after design) built on 2026-09-16, **not yet seen on an installed build**

**Needs:** nothing, then a window — the registry, a pure `launch_intent`, and a switch. Closing it wants an installed build, a logout and a login.

**Built on 2026-09-16**, `feature/autostart`, plan `docs/superpowers/plans/2026-09-16-autostart.md`.
Everything the design named is in: the plugin behind `#[cfg(not(debug_assertions))]`, the two
commands through `try_state` (never `app.autolaunch()`, which panics when the plugin is not
registered), `ipc::launch_intent` with its three rules, the tray and the archive always and the
window only when the arguments do not say `--silent`, and the switch first on the Background
screen. **What is not done is the looking**: an installed build, a logout and a login, and the
five checks of §7 — they are in *"What only a window can say"* in `docs/STATUS.md` now.

**Two corrections the writing made to the design**, both in the plan's own §"where this departs":
`AutostartReason` is a **bare camelCase string**, because it has no variant carrying data and
`CLAUDE.md` admits no exception; and `available: bool` became
`unavailable: AutostartReason | null`, because a development build and a registry that would not
answer were otherwise the same answer with two different causes.

**And one the owner made to mine.** The error was written fieldless — one write failure, nothing
for a reason to add — and that was wrong for a reason the plugin's own source states: `is_enabled()`
is `value && approved`, so an entry switched off in Task Manager's Startup tab reads as off
**however well the value was written**. A write can be *refused* or *accepted and then ignored*,
the app can tell them apart, and they send the user to two different places. Hence
`AutostartFailure { WriteRefused, WriteIgnored }` on the error, and two sentences under the switch
instead of one.

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

## B42 — Two Cargo tables are downloaded, committed, and read by nothing (implementation, `wiki`, small) half closed on 2026-09-14

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

### Re-measured on 2026-09-16, and the counter was the first thing wrong

`cargo run -p wiki --example probe_discarded` — new, and it exists because **the counter cannot
answer either half of this entry**: it is keyed on the **raw** title while `section_kind`'s
decision is taken on the **normalized** one, and it never says which page lost a section, which is
the only thing that lets a title be placed.

- **63 raw spellings are 59 titles.** `{{dlc|nr}} Gallery` *is* `Gallery`; the two capitalisations
  of `{{dlc+|r}} Behavior in Mausoleum/Gehenna` are one heading counted twice. "63 titles" was
  never a count of titles.
- **51 are seen exactly once, not 54.** The three that left the singles are those two plus
  `In-Game Footage`, a **fourth raw spelling** of in-game footage that the entry's "840 across
  three spellings" had not seen.
- **`Ingame Footage` (2) does not fold in**, and that is a parser fact rather than an oversight:
  it has no hyphen, and `normalize_title` does not collapse `ingame` to `in-game`.
- **Two titles are destroyed by normalization rather than cleaned by it**, which is worth the most
  here. `{{anchor|Unlocking the Forgotten|…}}` on **The Forgotten** normalizes to **the empty
  string** — 29 lines lost under a heading that is only an anchor — and
  `Interactions with {{c|Tainted Eve}}` on **Sumptorium** normalizes to `interactions with`,
  truncated. The normalizer strips a template carrying a **name** exactly as it strips one
  carrying a **marker**, and only the second is what it was written for.
- **The largest single loss is 1339 lines**: `{{dlc+|r}} Monster Replacement Tables` on **D10**.
- The probe walks all 1113 raw pages and reports **2270** dropped against the built dataset's
  2267: three sections sit on pages the build does not turn into entries. Said rather than
  reconciled, because the two are answering different questions.

Every one of the 51 now carries its page and its size, which is what placing them needs. The
placement, the near-misses' verdicts and the second-subject decision are still open — they are no
longer blocked on an instrument.

### The two defects are fixed, and three titles left with them

2026-09-16, `fix(wiki): a template carrying a name is not a marker, and three unlocks come back`.

**The truncation is gone**: a content template leaves its first argument behind, so
`Interactions with {{c|Tainted Eve}}` normalizes to what it says. The marker list is measured
rather than guessed — six template names appear in a heading anywhere in the snapshot, four that
render a marker or nothing and two that render a name. It cannot change which sections are kept,
and that is measured too: exactly one level-2 heading in the snapshot holds a content template,
and it is dropped either way.

**The empty heading stays empty**, because that is what the wiki draws: `{{anchor}}` renders
nothing, so The Forgotten losing its unlock section is a defect **on the page** and not in the
parser. A test pins the emptiness as the answer rather than a parse failure. Correcting it would
need a section-title correction, and `dataset/corrections.json` has no shape for one — it is a
`pageId` and `characters` map and nothing else.

**Three of the 51 left, into `SectionKind::Unlockable`**: Mega Satan's `Unlock` and The Lost's two
editions, each read on its page first, as three exact strings and not a prefix — a spelling nobody
has read stays out, and a test says so. Measured either side on the same snapshot: **3810 kept and
2270 dropped before, 3813 and 2267 after**, which is exactly the three and nothing else.

`diagnostics_are_bounded` went red on it, which is the best moment this entry has had: the
recovered Mega Satan section mentions `{{e|Reward Plate}}`, so unresolved entities went 18 to 19.
Checked rather than assumed — a fourth occurrence of the same three grid-entity keys, not a fourth
key — and the bound moved by one with the reason on it.

### The near-misses are read, and the line is written down

2026-09-16, `feat(wiki): six spellings of a kind come in, and the qualifiers stay out`. The entry
asked for them to be *"either accepted into `section_kind` or refused in writing with the page
that refused them"*, and its own words are the rule that decided it: **`section_kind` is a closed
list of spellings, and these are spellings.** A heading that adds *when*, *where* or a verdict is
not a spelling.

**In**, each with its page: `Syngergies` — a typo, and the only one in the snapshot
(collectible/Blood Bombs); `Items Interactions`, the plural of one already there
(character/Tainted Lost); `Active Item Interactions` (trinket/Found Soul) and `Other Interactions`
(trinket/Broken Remote); `General Strategies` (character/The Lost, which opens with
`{{main|The Lost (Strategy)}}`) and `Tips and strategies` (collectible/Isaac's Heart).

**Out, and a test names each refusal with its page:**

| refused | page | why |
|---|---|---|
| `Interactions with {{c|Tainted Eve}}` | collectible/Sumptorium | names **one subject**; a kind that names a subject stops being a kind |
| `Infinite Synergies` + 3 parenthesised | trinket/Broken Remote | one page keeps four apart, and *infinite*, *conditional*, *pre-Repentance*, *with delay* are the whole of what they say |
| `Behavior in Mausoleum/Gehenna` | boss/Mom, boss/Mom's Heart | adds **where** the behaviour applies |
| `Good Items`, `Bad items`, `Neutral Items`, `Detrimental items` | challenge/Bloody Mary and others | an editorial verdict, which `Notes` does not make — the entry said so and the pages agree |

Measured either side: **3813 kept and 2267 dropped before, 3819 and 2261 after.** Exactly the six.

**42 remain**, and none of them is a near-miss: what is left is the second-subject family and the
genuinely-something-else one.

### Closes when

The **second-subject family has a decision** — `Black Judas` on character/Judas, `Blood Clots` on
character/Tainted Eve, `Dark Esau` on character/Tainted Jacob, `Lazarus Risen` on
character/Lazarus, `The Soul` on character/The Forgotten, `Ultra Greedier` on boss/Ultra Greed,
`Friendly Charger` on collectible/My Shadow, `Special Locusts` on collectible/Abyss, and the two
halves of collectible/Broken Shovel. **A page has one entry and these sections belong to something
else that shares it**, which is a shape question about `Entry` and not a spelling. It is the same
question B53 came from.

The rest are the third family and need no decision, only the record that they were read: page-own
content with no kind behind it — crafting tables, a reverse-engineered algorithm, reroll chains,
pool probabilities, sound tables, poop varieties, a monster replacement table of 1339 lines.

### ✅ The third family is read, and the record is a test — 2026-09-17

All 22 read on their own pages. The record is `a_page_own_heading_is_read_and_stays_out` in
`crates/wiki/src/sections.rs`, not prose: a document does not fail when somebody adds one of
these spellings by reflex, and that test does — shown by adding `"strategy and items"` to the
`Strategies` arm and watching it go red. Each line names its page. They divide four ways, which
is the part worth having:

| family | what they are |
|---|---|
| **game data in a table, and no kind names it** (13) | Bag of Crafting's three, D10's 1339-line replacement table, Tainted ???'s poop tables, Lemegeton's pool probabilities, Damocles' survival table, Spindown Dice's reroll chains, Metronome's per-item effects, Bum Friend's drops, Ultra Hard's modifiers, Mom's Heart's post-5-kills changes, Tainted Lost's item exclusion |
| **the heading is not a heading** (5) | `[[Monsters]]` on boss/Great Gideon is a **link**, and its section has no body at all — the 54 lines are two `=== … Waves ===` tables *under* it; `Videos` is one YouTube embed and `Sounds` a table of `.wav`s, which is what `Gallery` is already dropped for; the two `Combinations` are a single list template each, so the content is not on the page to keep |
| **research that lives off the wiki** (2) | GB Bug's Lua listing, credited to a wiki user; Missing Poster's puzzle lore, pointing at imgur and reddit |
| **an editorial verdict** (2) | `Items` on character/Tainted Eden — "there are some that should be of special notice" is `Good Items` in a shorter word |

**One is a close call and is recorded as one**: `Strategy and Items` (character/Tainted Apollyon)
is strategy prose from the first bullet to the last, and `General Strategies` and `Tips and
strategies` are both in. What keeps it out is the line collectible/Sumptorium drew — the heading
names what the strategy is *about*, and every accepted spelling names only the kind. If that line
ever moves, this is the page to move it on.

### The second-subject decision is still open, and three findings make it cheaper

None of these was measured before today, and each narrows the question:

1. **`entity:Ultra Greedier` is already corrected by hand** in
   `crates/graph/rules/corrections.json`, to `mark: { column: "greed", level: "second" }` — which
   is exactly what bit 1 of the Greed column means, the one bit B58 confirmed on 2026-09-17. The
   correction is hand-written *because* the page's own section is dropped: the wiki states this
   and we re-state it in a rules file.
2. **The game has a player id for four of them.** `crates/catalog/src/heads.rs` names Lazarus
   Risen (11), Black Judas (12), The Soul (17) and Dark Esau (39), and folds the first three onto
   the base character's matrix cell. These are not "a second subject the page mentions" — they
   are subjects the game itself enumerates.
3. **Broken Shovel is not one page with one entry, which inverts this entry's premise.** The
   Cargo table carries `Broken Shovel 1` and `Broken Shovel 2`; `dataset/wiki.json` already holds
   **two** entries titled "Broken Shovel", both built from the one page. The two dropped sections,
   `Activated Collectible` and `Passive Collectible`, are precisely the two halves that tell the
   ids apart — so the two entries carry the same text today where they are meant to differ.

So the family is not one decision. **Six have a subject the game already enumerates** — the four
players, Ultra Greedier as an entity, the two shovel ids — and the shape question for them is
whether an `Entry` may be built from a **section** rather than from a page. **The other three are
not named anywhere in `graph`'s rules or in `catalog`**: `Blood Clots` (character/Tainted Eve),
`Friendly Charger` (collectible/My Shadow) and `Special Locusts` (collectible/Abyss, 173 lines)
are page-own mechanics text. Splitting the family that way is the recommendation; **making the
call is the owner's**, because it decides the shape of `Entry` and nothing should decide that
quietly.

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

### The retention is measured, 2026-09-16, and it is the other way round

Re-measured on the machine with the game, one day after the entry. The commands are here so the
next reading is a paste and not a new instrument:

```sh
S="/c/Program Files (x86)/Steam/logs"
grep -c "AppID 250900 adding PID" "$S/gameprocess_log.txt"
grep -oE '^\[[0-9]{4}-[0-9]{2}-[0-9]{2}' "$S/gameprocess_log.txt" | sort -u | wc -l
grep -oE '^\[[0-9]{4}-[0-9]{2}-[0-9]{2}' "$S/cloud_log.txt" | sort -u | wc -l
grep "gamestate1.dat" "$S/cloud_log.txt"
```

| | window | volume |
|---|---|---|
| `gameprocess_log.txt` | **325 distinct days**, 2025-06-25 → 2026-09-16 | 262 launches of 250900 |
| `cloud_log.txt` | **4 distinct days**, 2026-09-13 → 2026-09-16 | 32 lines naming a `gamestate1.dat` |

**So the asymmetry this entry drew is backwards.** It concluded *"the launch clock is a short
window … while the cloud clock held ~12 months"*, from a machine where the game had been
uninstalled and last played in January 2025. **A log that has stopped being written keeps
everything**: the twelve months there is a frozen file, not a retained window. On a machine in
daily use it is the **launch** clock that survives fifteen months and the **cloud** clock that is
four days deep.

**And the 93 does not reproduce.** Today the same file gives **262** launches with the *same* first
date, 2025-06-26 — so nothing rolled away and the difference is not retention. Whatever 93 counted,
it was not `AppID 250900 adding PID` in this file.

**A run starting is *not* as visible as one ending**, which the entry left open. Over the four-day
window: 6 `Need to delete` → `Delete OK` pairs and only **4** `Need to upload` → `Upload OK` ones,
and the three deletes of 2026-09-13 evening have no upload after 20:58 to pair with. The delete is
emitted when the game removes the mid-run save; the upload only appears if a sync happens to run
while the file exists. **The end of a run is dated reliably, the start is not.**

### Closes when

Either the launch timeline is a source the archive reads, with the three limits above represented
rather than smoothed over, or the entry says in writing why a rolling per-launch clock is not worth
the dependency. **The retention is now measured and it points the other way**: if either clock is
read, the durable one is the per-launch log, and the per-run clock is the one that will not be
there tomorrow.

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

### The cheap half is run, 2026-09-16, and it was worth more than the timeline

`cargo run -p catalog --example probe_steam_schema` — the KeyValues reader lives **in the probe**,
because this entry says the format is a parser the repo would have to justify and an instrument
justifies nothing.

**Three sources, three counts, each inside the next.** Steam's schema holds **641**, the installed
game's XML holds **637**, and the save declares **642**. `only in Steam: [638, 639, 640, 641]`,
`only in the XML: []` — the XML is a strict prefix, so nothing is missing from Steam and four ids
have no name or sprite in the catalog.

**Seven descriptions disagree, and none of them is punctuation:**

| id | the game's XML | Steam |
|---|---|---|
| 404 | Beat the game as Lazarus without losing a life | Beat **Hard mode** as Lazarus without losing a life |
| 470 | Complete the Corpse with **the Forgotten** | Complete the Corpse with **Bethany** |
| 471 | Complete the final chapter with **the Forgotten** | Complete the final chapter with **Bethany** |
| 523 | `They will charge you u` | They will charge you up… for a small fee. |
| 538 | `INVALID_DESCRIPTION` | ??? |
| 406, 410 | the requirement | flavour text |

**404 is a different requirement** and **470/471 name a different character** — on a project that
has already paid twice for character identity. **523 is truncated mid-word and 538 is a
placeholder**, both shipped in the game's own data.

**62 of 637 names are not in the XML's text.** Most are an apostrophe (`Lil Chubby` / `Lil' Chubby`,
`Mamas Boy` / `Mama's Boy`) or a plural (`A Forgotten Horsemen` / `A Forgotten Horseman`), and a
few are a different word entirely: `The Crucifix` / `Celtic Cross`, `Blood Lust` / `Bloody Lust`,
`Demon Isaac` / `Azazel`, and **`The Soul` / `The Lost`**.

**The probe's own first answer was wrong and is worth recording**: it reported 232 descriptions
differing, which were almost all the XML carrying the attribute *empty*. An absence counted as a
disagreement, and the real number is 7 out of 233 the XML fills at all.

### Closes when

The timeline is either a source with a stated scope (Steam-wide, achievement-only) or refused in
writing. **The cross-check is done**; what is left of it is what to do about the seven, which is a
question about which source the app should believe and belongs with B58's era work rather than
here.

---

## B57 — The game writes down where it saves, and `discovery` guessed (implementation, `discovery`, small) ✅ closed on 2026-09-16

**Closed on 2026-09-16.** `discovery` reads `savedatapath.txt` when the install is known and prefers it as a candidate; the search still answers without it, and the test that says so is the one this entry asked for by name. Commit `feat(discovery): the game writes down where it saves, so stop guessing`.

*The entry as it stood, kept because the retag is half of what it is worth:*

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

## B58 — The mark tables were located on the 2026 series and are read on a 2025 save (analysis, `core-save` and `ipc`) ✅ closed on 2026-09-17

**Closed on 2026-09-17, and the hypothesis did not survive the measurement.** The two cells
the entry called anomalous are ordinary, the tables are right, and what was actually broken
was something the entry did not suspect. Three findings, in the order they arrived:

1. **Both "anomalous shapes" occur in the 638 era too**, on a different profile, and one of
   them is an observed *transition*: `[457] 0 → 1`, `Isaac × The Beast`, on 2024-03-05, with
   `[492] 0 → 2` counting it and Eden's mark together. A shape that appears in two eras is
   not an argument for the tables being wrong in one of them.
2. **A cell's value replaces the one before it; the bits are not latched flags.** Four cells
   go **1 → 2** across the 638 series — `Isaac × Greed` and `Cain × Greed` on 2024-02-23,
   `Isaac × TheLamb` and `BlueBaby × BossRush` on 2024-01-29 — which is bit 0 going out as
   bit 1 comes in. So `Isaac × Greed = 2` is a Greedier clear that overwrote the Greed one,
   which is what "Ultra Greedier implies Greed" looks like in a file that overwrites. The
   entry's second row rested on reading "implies" as "accumulates", and
   `docs/save-format.md` had the same reading written into it as the *reason* for a
   property; the reason is corrected there, the property stands.
3. **The bases are measured at both ends, so the era between them is bracketed.** Section 2
   is 496 cells in the 638 era, 521 in the 641, 523 in the 642 — it only grew. The 638 series
   re-derives Mother at **423** and The Beast at **457** by the kill counters, two of three
   windows exact. A cell inserted before 423 by one patch and removed by the next is not how
   patches work. **It is an inference from two measured eras, not a third measurement**: this
   machine holds one 641-era snapshot and therefore no window in that era at all.

**What was actually broken.** The three properties in `crates/ipc/tests/marks_real.rs` that
"keep the tables answerable to the series" walked the `rep+` series **only**, by an explicit
decision in the file. On any machine whose `rep+` series is a single file — which is this one,
and every machine that has not collected 2026 snapshots — they returned early and the tables
were guarded by **nothing**, with the suite green. They now walk each series on its own, which
is what puts the 638 era under them.

**And the off-by-one they still cannot see, which is the part worth keeping.** Moving Mother's
base to **422** leaves every real-data property green here: the counters compare counts, so the
neighbour lights on the same day and the arithmetic works. Only index 188 separates identities,
and the 638 series offers one qualifying window, an Azazel one that never touches Mother. What
catches 422 is arithmetic on the tables — Delirium's 19-block runs 404..=422, so the two would
claim one index. `no_two_cells_of_the_matrix_share_an_index` and
`the_three_derived_blocks_tile_against_their_neighbours` in
`crates/core-save/tests/marks_layout.rs` are that check, and they need **no sample**, which is
why they are the answer: `samples/` is per-machine and a table's arithmetic is not.

**What is still owed, and it is not this entry:** what bits 0 and 1 *mean* outside Greed is
unmeasured, and B22 is where that is owed. Nothing here names them.

*The entry as it stood:*

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

## B59 — The convention scanner read a comment as if it were code (implementation, `ui`, small) ✅ closed on 2026-09-16

**Closed on 2026-09-16.** Comments become spaces before a rule sees a file; the style-block check
opts out and says why, because its exemption *is* a comment. The repository's verdict is unchanged
— 0 either way, and it could not be otherwise — so the pre-pass is worth what its fixtures are
worth: **ten, counted in the scanner's output**, each branch removed in turn to see which go red.
Commit `fix(ui): the convention scanner stops reading a comment as code`.

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

## B62 — A source folded into no runs cannot be told from one never folded (implementation, `store`, needs migration 5) ✅ closed on 2026-09-16

**Closed on 2026-09-16**, the same day it was opened, once migration 5 was approved. It is one
nullable column — `folded_rules_version` on `sources`, written by `cache_runs` inside the
transaction that replaces the fold — and `cached_runs` now answers `Some(vec![])` for a source
these rules read and found no run in, `None` only for one no fold under these rules has touched.
The parked test is back in `crates/log-watch/tests/ingest.rs` and green, and five tests in
`crates/store/tests/archive.rs` hold the three states apart, the migration on somebody's file
among them. Commit `fix(store): a source folded into no run is not a source nobody folded`.

**What the writing added to the plan above.** The entry described two states and there are
**three**, which is why the column is compared and not read as a boolean: *never folded*, *folded
under other rules*, and *folded into nothing under these*. The first two are the same answer to
the caller — fold it — and only the third is new. And a fourth case the entry did not name is the
one that made the extra test worth writing: a source that **used to** hold runs and folds to none
after a rules change leaves no row behind either, so the version on the source is the only thing
that says it was read at all.

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

## B63 — Nothing noticed when a test disappeared (implementation, `scripts/check`, small) ✅ closed on 2026-09-16

**Closed on 2026-09-16.** `scripts/check` totals the tests that *exist* — passed plus failed plus
ignored on the Rust side, Vitest's parenthesised total on the other — against `scripts/test-floor`.
A suite that shrank fails the run; one that grew prints the line to paste. **Both branches were
run**: deleting `crates/discovery/tests/save_data_path.rs` took the total to 992 against a floor of
1004 and the gate said `FAILED: test-count` — 992 being exactly the number this entry measured for
the original incident. What it still cannot see is a floor nobody raises, which needs a per-commit
comparison and therefore CI; that limit is written in `scripts/test-floor` and in `CLAUDE.md`.
Commit `chore: the suite says how many tests exist, and a shorter one fails`.

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

## B64 — The Floor grid draws fourteen room kinds as one grey square, and the legend is a brush (implementation, `ui`, after design)

**Needs:** nothing to write — the screen lives under **Tool** and answers with no game and no save
— **then a window** to judge it, which is the whole point of the entry: it was opened by somebody
looking at it.

Reported by the owner on **2026-09-18**, the first time anybody opened this screen in a window:
*the whole interface needs reviewing, you cannot tell where the rooms are, and the legend needs
improving too*. F1 landed on 2026-09-15 and **wrote no window checks**, so the Floor screen has no
group in `docs/STATUS.md`'s *"What only a window can say"* — nothing was waiting to be looked at,
and what the first look found is this entry. A sub-project that writes no checks does not appear in
that list as a gap; it simply does not appear.

### What the screen does today, in five places

1. **Every painted cell is the same colour.** `ui/src/screens/floor/FloorGrid.vue`'s `fillOf`
   returns `bg-floor-room` for any cell that is not empty and not a candidate, and
   `--color-floor-room` is `var(--color-card)` (`ui/src/assets/theme/floor.css`). Fourteen kinds go
   in through the brush and one square comes out: a Boss room, a Shop and a Secret Room you painted
   yourself are indistinguishable. **The grid cannot show what you drew on it**, which is exactly
   the sentence the owner used.
2. **The start room is invisible.** It carries `aria-current="location"` and nothing else — an
   attribute with no paint. It is not decoration: `floor.diagnostic.noStartRoom` says the Super
   Secret Room is judged only half way without it, so the screen asks for a cell it never shows.
3. **`FloorLegend.vue` is not a legend**, it is the brush picker: a `ToggleGroup` of fourteen labels
   plus *Cancella*. It says what you are **about to paint** and never what the colours on the grid
   **mean**. Nothing on the screen maps `--color-floor-candidate-first/second/third`,
   `--color-floor-empty` or `--color-floor-room` onto words, and nothing says that the `1` in a cell
   is the likeliest or that past the third rank the ramp stops counting — which is a claim the rules
   deliberately do not make, and therefore one the screen has to word.
4. **Only one of the three targets is drawn.** `FloorScreen.vue` passes
   `solutionFor('secret')?.candidates` to the grid, while the three cards below list Secret, Super
   Secret and Ultra Secret. The other two exist as rows of numbers and never touch the map.
5. **A candidate row is keyed by a raw cell index**, 0 to 168, printed as-is. A number is not a
   position: nothing connects "cell 97" to a square on a 13x13 grid, and reading the row lights
   nothing up.

### Why it is a design pass and not five fixes

Four of the five are one missing decision: **what a cell is allowed to say at once**. Today it says
one thing — a rank, or "painted", or "empty". The screen needs it to say two, *what you drew* and
*what the rules make of it*, for three targets, with a start marker, in 2rem squares.

Three constraints the redesign inherits, all of them already paid for:

- **The rank ramp is deliberately none of the state colours** (`floor.css` says so): green, gold,
  brown, grey, red and purple mean done / unlockable now / blocked / unreadable / unexpected /
  challenge everywhere else in the app. A room-kind palette has to stay off them too, or "a Boss
  room" and "unlockable now" become the same colour in a product whose whole subject is the second
  one.
- **Every candidate row carries the quotation and the URL of the rule that lit it.** That is the
  CC BY-SA attribution reaching the person reading the screen, not a layout detail. A denser grid
  must not drop it.
- **F2 will add a sentence to this screen** — *the game generated 19 rooms, you have painted 15* —
  so the shape needs somewhere to put it.

And `docs/frontend-conventions.md` applies unchanged: no `<style>`, no hardcoded visual constants. A
room-kind palette is a token family in `@theme` beside the ranks, not fourteen classes.

### Closes when

- [ ] A painted cell says which kind it is, on the grid, without hovering it.
- [ ] The start room is visible as a room and not only to a screen reader.
- [ ] Something on the screen says what the colours and the numbers mean, in words, including what
      the third step does **not** claim.
- [ ] Each of the three targets can be seen on the grid, or the screen says which one it is drawing.
- [ ] A candidate row and its cell are connected: reading the row finds the square.
- [ ] Judged in a window — and this time the group is added to *"What only a window can say"* before
      the branch closes.

---

## B65 — `Ctrl+Enter` in the palette is swallowed by the listbox, and the footer promises it (bug, `ui`, small)

**Needs:** nothing to fix — the library is in `ui/node_modules` and the palette is one file —
**then a window**, because nothing in this repo can mount a component (see below).

Reported by the owner on **2026-09-18**: in the `Ctrl+K` palette, `Ctrl+Enter` opens no second tab,
and whether plain `Enter` works at all is unclear.

### The first half is certain, and it is in the library

`reka-ui@2.10.4`, `Listbox/ListboxRoot.js` — `onKeydownEnter` opens with
`if (event.ctrlKey || event.metaKey || event.altKey) return;`, **before** it clicks the highlighted
element. The palette's input is a `ListboxFilter`, whose own `handleKeydownEnter` forwards straight
to that same function. So with `Ctrl` held there is no click, no `select`, and
`SearchPalette.openAt` is never reached: the screen is not wrong about what `Ctrl` means, **it never
hears the key**. `useGestureModifiers` reads the modifier correctly and nothing asks it.

`Ctrl`+click on the same row **does** work: that path goes through the item's own click, which is
precisely the one the library filters out for the keyboard.

**And the screen advertises the gesture.** `SearchPalette.vue`'s footer draws `Ctrl` and the return
arrow beside `search.hint.newTab` — *apri in una nuova tab* — which the design asked for
(`docs/superpowers/specs/2026-09-12-screens-wiki-search-design.md`, and the plan
`docs/superpowers/plans/archive/2026-09-12-screens-search.md`). A key drawn in the footer and
swallowed by the library is worse than a missing feature: the screen is documenting a gesture that
does nothing.

### The second half is a prediction, and it needs the window

*Whether `Enter` works* is a fair question and the code answers it only halfway.
`ListboxFilter.handleInput` calls `highlightFirstItem()` on **every keystroke**, while the palette's
rows come back from the backend **120 ms later** (`Timing.SearchDebounce`). So at the moment the
highlight is placed, the rows for what was typed are usually not mounted yet: either nothing is
highlighted, or what is highlighted belongs to the previous answer and unmounts when the new one
lands — and `onKeydownEnter` skips a `highlightedElement` that is no longer `isConnected`.

The Screens group is the exception: those rows are computed synchronously from the typed text
(`matchingScreens`), so a query naming a screen highlights at once. **The predicted shape is
therefore: `Enter` works when the first row is a screen, and does nothing when the answer came from
the backend.** One minute in a window says whether that is right; it is written down here so the
check has something to disagree with.

### What the fix probably looks like

The Enter path has to be taken **before** the library sees it. `useShortcut` already owns `Ctrl+K`
on the window; the palette can own `Ctrl+Enter` the same way, provided the handler can name the row
the highlight is on. The alternative is to stop leaning on the listbox's Enter altogether and hold
the highlighted row ourselves. Either way it is one decision about **who owns the keyboard inside
that dialog**, and it should be taken for `Enter` and `Ctrl+Enter` together rather than patched on
one of them — the two are the same gesture with a modifier, which is why the palette reads the
modifier from the window in the first place.

**Nothing in this repo can pin it red first.** `ui/` runs Vitest with no `@vue/test-utils` and no
DOM environment: no test mounts a component, so the palette's keyboard path has no failing test to
write. The gate is a window. If the fix ends up owning the keyboard, the part worth a unit test is
the pure one — *which row does this keystroke open* — and that is a function, which is where this
project puts anything worth checking.

### Closes when

- [ ] `Ctrl+Enter` on the highlighted row opens it beside the current tab, the palette closes, and
      the active tab has not moved.
- [ ] Plain `Enter` navigates the active tab, **including on a row that arrived after the
      debounce** — the case this entry predicts is broken.
- [ ] Both hints in the footer are true, or the one that is not is taken out.
- [ ] What the fix decides about who owns the keyboard is written where the next list of this kind
      will read it.

---

## B66 — The Completion screen calls bit 1 `hard`, in both languages, where `docs/save-format.md` refuses to (analysis, `ui`)

**Needs:** nothing — it is wording in two files and a read of what the save actually measures.

Found while touching `ui/src/i18n/messages/en.ts` and `it.ts` for 3.12 (Roll), which landed roughly
fifty-seven lines above this block in each file and moved it from where an earlier draft of this
entry pointed. `docs/save-format.md`'s "Counters and marks" section says `graph::rules::MarkLevel`
is named `Base` / `Second` **on purpose**, after the bits and not after a meaning: bit 1 is
measured only in Greed, where it is Ultra Greedier, and "what bit 1 means in the other eleven is
still unmeasured". The Completion screen does not carry that caution one layer up — it names the
column `hard` outright, in five places, in both languages:

- `ui/src/i18n/messages/en.ts:656-662` — `kpi.hard: 'marks at hard'`, `kpi.hardExplain` ("Cells
  with the second level…"), and `kpi.completeExplain`, which reads "every one of their readable
  cells at hard"; further uses at `:670` (`legend.hard`), `:677` (`grid.hard`), `:680`
  (`grid.columnTotalsHard`, "Of those, at hard") and `:689` (`cell.hard: 'hard'`).
- `ui/src/i18n/messages/it.ts:663-669` — the same three keys in Italian (`marchi in hard`,
  `hardExplain`, `completeExplain`), plus `:677`, `:684`, `:687` and `:696` for the same four
  further uses.

`cell.hard` is the one this entry first missed, and it is not a minor one: it is the **per-cell**
status text, rendered through `ui/src/components/marks/MarksGrid.vue:47`
(`[CellStatus.Hard]: 'completion.cell.hard'`) for every individual cell in the matrix, rather
than a column header read once. A fix that renamed `kpi.hard` / `legend.hard` / `grid.hard`
without this one would leave the most-repeated instance of the claim untouched — the exact
half-migration this entry exists to prevent.

So a player reading the screen is told a specific thing — this column is the hard difficulty —
that the crate one door over declines to assert for eleven of the twelve columns it draws. Either
the claim is right and the screen is ahead of a measurement nobody has written down, or the wording
is a guess that slipped past the `MarkLevel::Second` naming it was supposed to defer to.

### What would close it

- **Analysis first**: is bit 1 actually the hard difficulty everywhere, or only demonstrably so in
  Greed? The Greed measurement
  (`winning_greedier_sets_the_second_bit_of_that_characters_greed_cell`,
  `crates/ipc/tests/progress_real.rs`) is the only evidence on file; the other eleven columns have
  none. A matched window on a character clearing hard mode, the way Greed's was found, would settle
  it the same way.
- **If it can't be measured soon**, the wording moves toward the crate's own restraint —
  `kpi.hard` / `legend.hard` / `grid.hard` / `grid.columnTotalsHard` / `cell.hard` become
  something that names the bit rather than the difficulty ("second level", matching
  `MarkLevel::Second`), until the day it is measured and the screen can say `hard` honestly.

### Closes when

- [ ] Bit 1's meaning outside Greed is either measured or explicitly left open in the wording.
- [ ] `en.ts` and `it.ts` agree with whatever `docs/save-format.md` is willing to state.
- [ ] `graph::rules::MarkLevel`'s naming and the Completion screen's naming say the same thing.

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
