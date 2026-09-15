# Progress status — IsaacDome

Single source of truth for the project status. Updated every session.
Narrative summary and design decisions live elsewhere: `docs/PROJECT.md` (project),
`docs/superpowers/specs/` (module design), `docs/superpowers/plans/` (the plan being executed
now, the ones already merged under `archive/`), `docs/superpowers/reports/` (what execution
measured).
The quality tasks that came out of the 2026-09-05 review (local verification, scanner, IPC
contract, memory, test data) live in `docs/IMPROVEMENTS.md`, with closing criteria and order.

**Integration branch:** `develop`. `master` is stopped at the initial commit.
**Wiki dataset merged** into `develop` on 2026-09-06 (`feature/wiki-dataset`, 29 commits,
suite green on the merge result, review of the whole branch closed). The local branch was
deleted; on origin its last published version remains.
**Design system merged** into `develop` on 2026-09-11: cycle 1 (`feature/design-system-foundations`),
then cycle 2 and screens 3.1–3.2 in one merge of `feature/design-system-screens`, which
already held `feature/design-system-components`; suite green on the merge result. Screens 3.3a
and 3.3b followed on the same branch, each merged into `develop` with the suite green (the last,
`ef962c8`, on 2026-09-11).

**Branches from 3.4 on: one per sub-project, cut from `develop`** (decided 2026-09-11). Nobody
commits on `develop` directly: it is where finished work lands, through a `--no-ff` merge with
`scripts/check` green. A sub-project gets its own `feature/<name>` branch — Collection is
`feature/screens-collection` — so each merge is one sub-project, its diff stays reviewable, and
a piece that has to be redone is thrown away without touching the others.
`feature/design-system-screens` had grown to hold 3.1 through 3.3b under a name that no longer
said what it carried; it is fully merged and kept, its deletion waiting for the owner.
**Last update:** 2026-09-14. **All eight cleanup items are done** — N1, N2, N3, N4, N5,
N6, N7 and N8, each on its own branch cut from `develop` — the last two on the evening of
2026-09-14. The test-only public API has one notation
(one `pub mod for_tests` per crate, seven of them); **why a command failed is a variant, not
a sentence** (four enums, the numbers travelling as numbers, the wording in `it.ts` / `en.ts`);
**the Tauri crate is wiring again** (eleven files, none over 220 lines, `cargo test -p app`
reporting zero); one diagnostics list instead of four; one view store instead of three;
**the IPC contract is generated from the Rust types**; and **one faceted list instead of two**,
six components become three and five modules two.
**M4's sub-project 1 is closed**, 1a and 1b both: the run model, then the watcher, the archive
and the fourth migration (`feature/log-watch`). **N8 ran last**, as its own order says — *after
M4* — and closed on the evening of 2026-09-14, which empties the cleanup list.
Order: N1 → N2 → N6 → N4 → N5 → N7 → M4 sub-project 1 → **N3 → N8**. N4 and N5 were pulled
forward because they are frontend, touch no file that branch has, and N7 was blocked then;
**N3 ran before N8 rather than after**, on a machine without the game, because it is the one of
the two that needs neither the game nor a real save to be finished or believed.
**Sub-project 3.5d merged into `develop`** (`4406c49`), suite green on the merge result: a
blocked badge opens a menu whose entries are the wiki pages of what is in the way.
**The wiki's transformations merged into `develop`** on 2026-09-13, suite green on the merge
result: sixteen pages, a seventh `PageKind`, and `Requirement::Threshold` — the shape the
model could never say, *N of these items*. The four nodes behind Guppy and Beelzebub are
answered. **The other half of B34 was written and thrown away**: the thirteen `pickup:`
references are real requirements, not noise, and dropping them would have made their nodes
read *available now*. Report in
`docs/superpowers/reports/2026-09-13-transformations-report.md`.
**M4's first sub-project has its design** (`cac6914`): the run model, the `run` and
`log-watch` crates, and the backfill that makes the archive born full from the logs already
on disk.
**The landing page and the achievement detail are done** on `feature/screens-goals-detail`,
waiting to merge: "Prossimi passi" is **"Obiettivi consigliati"**, grouped by the reason a row
is suggested, and an achievement's wiki page carries a block saying where the profile stands.
**B32 and B35 closed.** Two contract changes travel with it — `NextSteps` in sections with a
`Closeness` basis, and `AchievementRef`'s `hint` renamed `condition` because it now answers
from the wiki where the game file is silent (283 of 637 before, all 637 after). **N7 has that
much more to absorb**, and `UnlockTarget` gained a field as well.
**B40, B43 and half of B42 closed** on `feature/small-three` (2026-09-14, evening), three small
entries run in parallel on disjoint files. The cross-check B42 asked for opened **B45**: four
characters — Jacob & Esau, The Forgotten, Tainted Forgotten, Tainted Lazarus — **have no page in
the wiki snapshot**, 47 requirements point at them, and the fetch cannot see it because it
enumerated the singular `Infobox character` while those four pages carry the plural one, two forms in a block — closed the same evening.

---

## Milestones

- [x] **M0 — Format spike**
      `.dat` format decoded and verified on 28 real saves, working Python parser,
      counters labeled, marks matrix rebuilt, log verified.
- [ ] **M1 — Rust parser, discovery, unpack, Completion screen** ← in progress
- [x] **M2 — Unlock graph** (2026-09-07). The Unlock *section* is frontend work and
      waits for the design system; the graph behind it is done — report in
      `docs/superpowers/reports/2026-09-07-unlock-graph-report.md`.
      **Reopened and closed again on 2026-09-12** for the one kind of prerequisite it could
      not say. Spec `docs/superpowers/specs/2026-09-12-graph-mark-requirements-design.md`,
      plan `docs/superpowers/plans/archive/2026-09-12-graph-mark-requirements.md`.
      - [x] Five targets — Mother, The Beast, Hush, Delirium, Ultra Greedier — held **178 of
            the 195** uninterpreted references, and none of them for want of curation: every
            one was a hand-written `Verdict::Unknown`. They are not behind an achievement,
            they are behind *having played something*, and the model had no case for it.
      - [x] **Uninterpreted references on real data: 195 → 17.** `mark 170, counter 8`, which
            is exactly the split the spec measured: 170 of the references name a character
            alongside the boss, so they are one cell of the completion matrix, and 8 name the
            boss alone, which a kill tally answers. The 17 that remain are the 13 `pickup:`
            and 4 `transformation:` references §6 of the spec leaves out.
      - [x] **17 → 13 on 2026-09-13**: the four `transformation:` references are answered by
            `Requirement::Threshold`, and the wiki's sixteen transformations are in the
            dataset. Report in
            `docs/superpowers/reports/2026-09-13-transformations-report.md`.
            **The remaining 13 are not noise and are not going away**: checked against their
            sentences, `ending`, `Bestiary` and `tainted character` are real requirements the
            model cannot express — dropping them would make their nodes read *available now*.
            B34 is corrected in place.
      - [x] **New measurement**: in the **Greed** column, bit 1 of a cell is **Ultra
            Greedier** — three days, three characters, each time the right character's cell.
            Recorded in `CLAUDE.md` and `reference/isaac_counters.py`, kept by a property in
            `crates/ipc/tests/progress_real.rs`.
      - [x] **The mark layout moved to `core-save`**, where the file's shape belongs; it had
            been living in the view-model that draws the matrix. Completion's tests passed
            unchanged, which is what says no index moved.
      - [x] **Two bugs the tests found, both about a character's identity.** The game gives a
            Tainted character the base form's name, so the name index keeps one of the two:
            by name alone 141 of 396 character references resolved to nothing, and by
            name-first "Ultra Greedier as Keeper" picked row 29 — T. Keeper. Resolution goes
            by the wiki's id first now, pinned by two tests in `crates/graph/tests/build.rs`.
      - [x] **Handed to the design system** on 2026-09-13: `RequirementView` gained `mark` and
            `counter`, and a node held only by one of them is `availableNow` — nothing is
            locked, the content only has to be played. The landing page consumes it as a
            section of its own, **Ci sei quasi**, ordered by how far the tally still is: a
            counter is the only requirement that carries a distance.
            **With the measurement that came out of it**: on the reference profile (379 of 642
            done) the whole view holds **zero** `Counter` requirements — a counter is reported
            only while `current < at_least`, and every threshold there was crossed long ago.
            The co-op partner's profile, the one `online_logs\` leaves at the start of the
            progression, holds 4 and 4 nodes held by nothing else, so the property is asserted
            there; a second test pins *why* the reference profile has none, so that "empty"
            stays a fact about the profile.
- [ ] **M3 — Derived plan** ← in progress. The **plan queue** is done (2026-09-08): an
      ordered series of achievements whose order is yours and can never contradict the
      graph. Report in `docs/superpowers/reports/2026-09-07-plan-queue-report.md`. What
      remains of M3 is the screen, which waits for the design system.
- [ ] **M4 — Log watcher and run archive** ← designed, not started. Four pieces and not a
      screen: a pure `run` crate (typed events, the fold, the rules file, and the part of
      tailing that is not I/O), a thin `log-watch`, the store's third migration — `events`
      as rows, `runs` as a derived cache carrying the rules version that produced it — and
      the two routes the shell already reserves as placeholders.
      - [x] **Sub-project 1, the run model and the log watcher — design** (2026-09-12),
            `docs/superpowers/specs/2026-09-12-m4-run-model-design.md`. Three decisions
            taken in conversation: **backfill everything**, so the archive is born full from
            the `online_logs\` sessions on the disk (22 by that day's count, **28** once 1b
            measured the folder and found it three levels deep); **events are the archive and a
            run is a fold over them**, which makes backfill and live one function instead of
            two paths that have to agree forever; **abandoned runs are kept and marked**, and
            whether they count is measured against the save's own `STREAK_COUNTER [22]`,
            `BEST_STREAK [23]` and `DEATHS [10]` rather than decided. The screens are
            deliberately out of scope: `Live` and `Runs` are both views of this model, and
            designing them first would let a layout shape the IPC contract.
      - [x] **2b — Live** followed it the same evening, `feature/live-screen`: the run being
            watched, and what finishing it would open, grouped by the cell it needs. Plan
            `docs/superpowers/plans/2026-09-14-live-screen.md`, report
            `docs/superpowers/reports/2026-09-14-live-screen-report.md`.
            **The join is one command**, as N8 pointed: the archive's open run and the
            graph's marks have to describe the same profile, and two commands cannot promise
            that. **The rule is narrow** — a run opens an achievement only when *everything*
            still missing from it is a mark for this character — and the test was mutated to
            check the wider rule turns it red.
            **The character stayed ambiguous on purpose.** The log prints a name, the game
            gives a Tainted character the base form's name, so the view holds both forms and
            says it holds both; inferring the right one from the starting items was refused
            twice, in the spec and again here. And the plan's single absence became two:
            *no profile* and *no graph* are different sentences to whoever is reading, which
            the spec's own "the run draws either way" is what made visible.
      - [x] **2a — the Run screen** is done the same evening, `feature/run-screen`: the diary,
            four filters on N3's engine, the totals, and the chosen run's items under the
            list. **Nothing new crossed the IPC**, which is why 2a could be split off at all.
            Plan `docs/superpowers/plans/2026-09-14-run-screen.md`, report
            `docs/superpowers/reports/2026-09-14-run-screen-report.md`.
            **The order was wrong the first time and the way it was wrong is the finding**:
            "newest session first" and "a name that is not a clock keeps its place" read as
            compatible in prose and **contradict each other as one comparison** — with an
            unreadable name in the middle the comparator has a cycle, and `Array.sort` given
            one does not fail, it answers something arbitrary. Only the readable names are
            sorted, into the slots they already hold.
            Three more corrections to its own plan: the detail moved **under** the list
            because a virtualized row is measured at one height; `FilterToolbar`'s sort group
            became optional, because this list has nothing to choose between and a single
            fake option is a control that changes nothing; and two badges wore the `Unknown`
            variant — the question mark this design keeps for what could not be read — for
            **online** and **abandoned**, which are facts. Only a window said so.
      - [ ] **Sub-project 2, the two screens — design taken on 2026-09-14**,
            `docs/superpowers/specs/2026-09-14-m4-run-screens-design.md`. Three decisions in
            conversation plus one word: **Run is a diary** (a row per run, what happened, not
            whether you are improving — the alternative was declined and the offer was weak,
            because with no clock in the log and no floor on a death a "bilancio" could only
            group by killer and character); **Live shows the run in progress and what
            finishing it would open**; **abandoned runs stay in the list, marked**; and the
            list is **filterable**, which lands on N3's unified engine — this is the third
            list it was unified for.
            **The design's hard part is not the join, it is the character.** The archive has
            the name the log prints, `player 0 (Cain)` — where `player` is the *slot*, not an
            id — and the game gives a Tainted character the base form's name. So Live cannot
            say which Cain you are playing: the spec chooses to **say both** rather than infer
            from the starting items, and registers the measurement that would replace the
            ambiguity (one Tainted run with its log kept).
            Run needs **nothing new on the wire**; Live needs the marks a character is still
            missing, and where that join happens is the plan's decision, with N8's lesson
            pointing at the backend.
      - [x] **Sub-project 1 splits in two on 2026-09-13**, when the plan was written and the
            size was visible. **1a — the run model** is done, `feature/run-model`: the pure
            `run` crate, 41 tests, plan
            `docs/superpowers/plans/archive/2026-09-13-run-model.md`, report
            `docs/superpowers/reports/2026-09-13-run-model-report.md`. `Tail` turns bytes into
            lines and reads a shorter file as a relaunch; `rules/events.json` maps a line to
            one of nine events and judges nothing; the fold makes every judgment, and each of
            them is invisible in well-formed data.
            **Three corrections the work made to its own spec**, all from measuring:
            every log line carries `[INFO] - ` (and sometimes `[Frame 74] `), which the spec's
            table and `CLAUDE.md` both dropped — a pattern anchored at the start of a line
            matches nothing; the store migration is the **fourth**, since the window session
            took the third the day before; and `Game Over` appears in **none** of the four
            sample logs, so `Died` has no real-data coverage at all and a test asserts that
            absence rather than letting a green suite imply it.
            **One thing the spec did not have.** The seed line has three kinds — `New`,
            `Continue`, `Net` — and a `Continue` is a run *resumed*, logged with the seed it
            already had. The spec's abandonment rule would have marked a run still being played
            as abandoned and counted it twice, so the fold decides by seed and not by label.
            `Net` is the free discriminator for co-op, which §3's open question about
            `STREAK_COUNTER [22]` will need.
      - [x] **1b — it goes live** (2026-09-13), `feature/log-watch`: `log-watch`, the store's
            fourth migration, run identity, the backfill, the view-models, and the archive
            filling itself at launch. Plan
            `docs/superpowers/plans/archive/2026-09-13-log-watch.md`, report
            `docs/superpowers/reports/2026-09-13-log-watch-report.md`.
            **Five measurements, four of which contradicted a document.** The 4 KiB prefix the
            spec identified a launch by is the machine describing itself — OpenGL, driver, the
            game's DLL path — and the only byte separating two launches inside it is a load
            timing, so identity rests on an **anchor** in the run content instead. The two logs
            of 2026-09-08 are one launch copied twice (a strict prefix), so `samples/logs/` holds
            three launches and no pair of *different* ones. `online_logs\` is not flat and is
            **28 sessions**, not 22: `sessions\`, `desyncs\` holding crash reports, and two more
            sessions nested under `desyncs\sessions\`. And `discovery` could not name the folder
            at all — it reached `My Games` only to look for `.dat` files, which with Steam Cloud
            on it never finds there.
            **The bug a test found is the one that mattered**: `head(log, 4096)` on a file
            shorter than 4 KiB returns the whole file, so the prefix changed with every line the
            game wrote and every read looked like a new source — which would have imported the
            whole archive a second time. The window is stored with its length now.
      - [ ] **The agreement with the game's own counters stays open**, and that is a measured
            result rather than a gap. The spec named 2026-09-08 as the one window that could
            answer it; measured, the window does not contain the run — the log calls
            `unlock steam achievement` twice and **no slot of 642 turns on** across it, while 2
            counters out of 523 move by one each. The backup of the 8th predates the run. One
            solo, non-Greed win with a snapshot either side closes it; `live_probe` takes them.
      - [ ] The `Live` and `Runs` screens, on the contract this model fixes
- [ ] **F — Floor, the companion screen** ← F1 done, F2 open
      - [x] **F1 — the painted grid and the cited rules** (2026-09-15), `feature/floor-grid`.
            A new pure crate `floor` (the 13x13 grid, nine rules read from the wiki with their
            quotations, and a solver that says what it cannot judge), `ipc::floor_view`, one
            wiring command, and the screen — which lives under **Tool**, so it answers with no
            profile and no save. Spec
            `docs/superpowers/specs/2026-09-15-floor-secret-rooms-design.md`, plan
            `docs/superpowers/plans/archive/2026-09-15-floor-grid-and-rules.md`, report
            `docs/superpowers/reports/2026-09-15-floor-report.md`, and the rules with their
            sources in `docs/superpowers/reports/2026-09-15-secret-room-rules.md`.
            **It reopens a decision B8 had closed**, and §1 of the spec says so out loud: a
            secret-room finder is a different product from "what am I missing tonight". The
            owner asked for it anyway, which is a scope call and theirs.
            **Reading the raw wikitext rather than a summary changed the file**: two rules the
            plan's draft did not have — a 1-neighbour Secret Room, and a Super Secret Room that
            "cannot be connected to the Secret Room" — and a summarizer had already returned
            one of the quotations cut in half.
            **One of the two forced a new constraint and a third solver phase.** The wiki says
            2 neighbours is possible "even when" 3+ exist and 1 neighbour "only if" none do:
            same rank, different shape. `NeighbourCountFallback` carries the `3` the sentence
            states, and the fallback resolves **after** the narrowing rules, because "no
            **valid** 3+ location" means none that survived them. Pinned by a test that was
            mutated to check it turns red.
            **The Start Room's membership is unstated and stays that way.** The wiki's `Rooms`
            page files it under neither `Normal` nor `Special`, so `SPECIAL_KINDS` does not
            carry it and a dead end hanging off the start room is not silently decided.
      - [ ] **F2 — the log's half**: `N rooms in M loops` read back, so the screen can say the
            game generated 19 rooms and you have painted 15. **Needs a measurement**: which
            generation attempt in a log describes the floor actually played is a judgment, and
            it belongs in `run`'s fold measured against the five real logs.
- [ ] **M5 — Public release**

---

## M1 — detail

Every module follows the same cycle: design spec → TDD plan → implementation →
report → post-review fixes.

### `core-save` — `.dat` parser, read-only ✅

- [x] Design spec — `docs/superpowers/specs/2026-09-01-core-save-parser-design.md`
- [x] TDD plan — `docs/superpowers/plans/archive/2026-09-01-core-save-parser.md`
- [x] Implementation — `crates/core-save/` (`parse.rs`, `section.rs`, `diff.rs`)
- [x] Report — `docs/superpowers/reports/2026-09-01-core-save-parser-report.md`
- [x] Post-review fixes
- [x] `Cargo.lock` checked in for reproducible builds (`da615dd`)

### `discovery` — finds Steam, the game, the saves ✅

- [x] Design spec — `docs/superpowers/specs/2026-09-01-discovery-design.md`
- [x] TDD plan — `docs/superpowers/plans/archive/2026-09-01-discovery.md`
- [x] DLC → edition map
- [x] Save file name parsing
- [x] `appmanifest .acf` parsing (installdir + DLC)
- [x] Save scanning (multi-account userdata, Documents, override)
- [x] Steam location (`steamlocate` + `winreg` HKCU fallback)
- [x] `discover()` orchestration + integration test on the real machine
- [x] Report — `docs/superpowers/reports/2026-09-01-discovery-report.md`
- [x] Post-review fixes (registry fallback, malformed `.acf` fallback, degradation)

### `unpack` — targeted extraction from `.a` archives ✅

- [x] Design spec — `docs/superpowers/specs/2026-09-01-unpack-design.md`
      (ARCH000 format, djb2 + FNV hashing — **the spec needs a correction**: it gives the
      version as a constant, while byte `0x07` is actually the compression mode)
- [x] TDD plan — `docs/superpowers/plans/archive/2026-09-01-unpack.md`
- [x] Scaffold, public types, hashing
- [x] Report — `docs/superpowers/reports/2026-09-01-unpack-report.md`
- [x] Post-review fixes (absolute-path guard, no-panic test, spec alignment)
- [x] **All three compression modes** (2026-09-03), dispatched on byte `0x07`:
      `Lzw` (already there), `MiniZ` (deflate + ISAAC cipher, `miniz.rs` + `isaac.rs`) and
      `Bogocrypt1` (XOR in groups of 4 with an evolving key, `bogocrypt.rs`).
      `Bogocrypt2` remains unimplemented: no archive in this install uses it,
      so there is nothing to verify it against.
- [x] **Verification across all archives**: a full pass of the filelist's 18,789 paths over
      eight archives → **14,751 resources extracted, 0 failed**, 1.0 GB. The extracted
      sprites are valid PNGs (IHDR and IEND verified, and one checked by eye).
- [x] Test per mode — `crates/unpack/tests/compression_modes.rs`, 5 tests that **declare
      which mode they run on**. Suite: 99 tests, 0 failed.

### `catalog` — normalizes the game's XML files ✅ plan A and B closed

- [x] Design spec — `docs/superpowers/specs/2026-09-03-catalog-design.md` (the whole
      catalog in one spec, two plans: A closes M1, B prepares M2)
- [x] TDD plan (A) — `docs/superpowers/plans/archive/2026-09-03-catalog-a.md`, 10 tasks
- [x] **Character head icons: map written, checked by eye** (task 6). The `Main`
      layer of `gfx/ui/coop menu.anm2` gives 38 frames; cropped from `coop menu.png` (192×224,
      32×32 cells) and inspected one by one. Frame 0 is the menu's "?" placeholder, then the
      sequence follows the order of `players.xml`: Isaac, Magdalene, Cain, Judas, ???, Eve,
      Samson, Azazel, Lazarus, Eden, The Lost, **Lazarus again** (Resurrected Lazarus, id 11),
      **Judas again** (Black Judas, id 12), Lilith, Keeper, Apollyon, The Forgotten,
      **The Forgotten again** (The Soul, id 17), Bethany, Jacob, then the 17 Tainted from
      Isaac to Jacob in the same order. The three repeats fall exactly on the forms that
      reuse the base character's face: that confirms the alignment is correct.
      Esau (id 20) has no cell — in the menu you pick "Jacob & Esau" as one — and the
      sequence closes on Tainted Jacob: Resurrected Tainted Lazarus (38), Dark Esau (39) and
      Tainted Soul (40) stay `head: None`. 37 of 41 characters have a head icon. The
      rule, as a formula: id 0–19 → frame id+1, id 21–37 → frame id; the three
      repeated cells (11/12/17, which reuse the base face) confirm the alignment.
- [x] **Tainted bug found by the real-data test** (task 7): character 38
      (Resurrected Tainted Lazarus) has portrait `PlayerPortrait_Lazarus_b_dead.png`, and the
      initial rule `ends_with("_b.png")` missed it. Fixed with a check for the `_b` token in
      the portrait file name (`_b.png`, `_b_dead.png`), not just the suffix.
- [x] **Found two commented-out elements in `items.xml`** (task 7): `PILLS_HERE_NAME` and
      `TAROT_CARD_NAME` (lines 44 and 62) sit inside a `<!-- ... -->`, not as elements.
      The correct count is **909 items** (425 passive, 170 active, 126 familiars,
      188 trinkets), not 911: the old 911 came from a `grep` over the text that counted
      them by mistake. With this, **all** name keys resolve — no exceptions.
- [x] Implementation (plan A) — 42 tests, 0 failed.
- [x] Report — `docs/superpowers/reports/2026-09-03-catalog-a-report.md`
- [x] **Plan B** (prepares M2, and **closes the static data base**): `metadata.rs`
      (quality and tags, actually in `items_metadata.xml` not `items.xml`); `achievements.rs`
      with the unlock-condition comments and the `achievement="N"` link from items;
      `itempools.rs` (`Item.pools`); `challenges.rs`; `bossportraits.rs`; `origin`
      (source DLC from id ranges); cross-check against the save (721
      collectibles out of 733 slots, 11 unused ids identified; 637 achievements out of 642
      slots). Found and fixed a real bug in the challenge parser (mixed separators,
      negative ids); fixed four measurement numbers that were wrong in the brief, which had
      propagated into the spec. 81 tests in the `catalog` crate (51 unit, 15 on `build.rs`, 15 on `real_data.rs`).
- [x] TDD plan (B) — `docs/superpowers/plans/archive/2026-09-04-catalog-b.md`, 8 tasks
- [x] Report (B) — `docs/superpowers/reports/2026-09-04-catalog-b-report.md`
- [x] **A challenge's reward** (backlog B2, 2026-09-05): `Challenge.rewards`, collected by
      `Catalog::build` from the achievement notes. Three forms in the file, not one: comment
      `Beat Challenge #N` (challenges 1–20), comment `beat Challenge N (Name)` (21–30), attribute
      `steam_description="Complete Challenge N."` with no comment (36–44); challenges 31–35 and 45
      have no trace at all. 39 of 45 challenges have a reward, as measured on the real file. Details in
      `docs/BACKLOG.md`, entry B2.

> **Fully unblocked** (2026-09-03). With `unpack` complete, the DLC XML files can also be
> extracted: `items.xml` from `afterbirthp.a` is 136,047 bytes versus the 51,228 of the base
> version from `config.a`. So there is a real, complete schema to design against, no longer a
> quarter of it guessed at.
>
> To clarify when writing the spec: **precedence between archives**. The same path exists in
> multiple `.a` files and the most recent DLC wins, but the order needs to be fixed and
> verified, not assumed.
### `ipc` — view-model for the interface ✅

Pure crate, no I/O and no Tauri dependency: turns `discovery`, `core-save` and
`catalog` types into already-resolved JSON. **83 tests.**

- [x] Design spec — `docs/superpowers/specs/2026-09-02-app-shell-ipc-design.md`
- [x] TDD plan in 12 tasks — `docs/superpowers/plans/archive/2026-09-02-app-shell-ipc.md`
- [x] Opaque profile id, deterministic and case/separator insensitive
- [x] Presentable candidates, sorted by date, with exactly one suggestion
- [x] Active profile resolution: **no silent fallback**, ever
- [x] Marks tables ported from the Python reference
- [x] Matrix with `Known` / `Unknown` / `Unexpected` cells
- [x] Cross-check against the Python reference: **321 cells, all matching**
- [x] Save summary, install status, settings
- [x] **Graph screen contracts** (2026-09-05, `graph.rs` and `goals.rs`): a single node,
      `UnlockNode`, for Unlock, Next Steps and Plan; four pure functions
      (`unlock_view`, `next_steps`, `plan_view`, `resolve_target`); `Goal`, opaque `GoalId`,
      `TargetKey`, `UnlockTarget`, `GoalView`.
      What the graph couldn't say travelled as a declared `{ kind: "stub" }`, never as a
      value that looks computed. JSON shape pinned, including `unknown`, `stub` and
      `itemKind` (the field is named this way because `kind` is already the tag).
      **`stub` left the wire with M2** (2026-09-07): the graph exists, so a node saying it
      doesn't would be lying. Its place is taken by `Partial`, which is what a node says
      when the graph can't interpret one of its requirements.
- [x] **The goal only persists its identity** (final review, 2026-09-05).
      `TargetKey` — `Item { itemKind, id } | Character | Boss | Challenge` — is `store`'s
      on-disk format and changes **only by adding variants**: a new field there
      would invalidate every saved row. Name and icon (a base64 data URL) are derived from the
      catalog and are resolved on every read via `resolve_target`, the inverse of
      `UnlockTarget::key()`: this way the Plan doesn't show the name from the day the goal
      was created, and the database doesn't turn into an image archive. `PlanView.goals` carries
      `GoalView { id, key, target: Option<UnlockTarget> }`: `target: null` means "not
      resolvable right now" and the goal stays visible and removable, with `NoCatalog` (only
      once) or `UnresolvedGoal { id }` saying which of the two cases applies.
- [x] **One single rule for fieldless enums**: on the wire they are a bare camelCase
      string. `ItemKindView` was tagged (`{"kind":"passive"}`) and became `"passive"`,
      like `OriginView`. Rule now in `CLAUDE.md`; as soon as a variant gains a field,
      the whole enum goes back to being tagged.
- [x] **Unreadable section 1 declared as such**: `unlock_view(catalog, flags: Option<&[bool]>, …)`.
      Before, the app flattened a missing section to an empty vector and the view answered
      `CatalogBeyondSlots { 638 }`, which is false. Now `None` gives zero nodes, zero totals and
      `NoAchievementSection`, with no comparison against the catalog; `Some(&[])` remains the
      degenerate save case with its own diagnostic.
- [x] **Section 1 achievement ↔ slot mapping: `slot[id]`**, pinned on the real profile
      (`samples/live.rep+persistentgamedata1.dat`, 2026-08-31): 169 out of 171 seen items
      have the achievement completed; 642 slots, 379 completed, 637 known to the catalog, **4
      unknown** (slots 638–641, 3 of which completed). Slot 0 is not a node: the nodes are 641
      and `known + unknown = slots − 1`. Spec:
      `docs/superpowers/specs/2026-09-05-graph-contracts-design.md`.

### `store` — the app's persistence ✅ (born together with goals)

A single SQLite file, `isaacdome.db`, in the app's data folder; schema versioned with
`PRAGMA user_version`. Writes exclusively to its own file: it doesn't even know about
`core-save`, and read-only access to saves remains true by construction. **8 tests.**

- [x] Spec — the "Plan: materialized goals" section of
      `docs/superpowers/specs/2026-09-05-graph-contracts-design.md`. The project document
      assigned `store` four responsibilities (run archive, snapshots, catalog, plans);
      goals *are* user plans, so the crate is born here, small, instead of taking on the
      debt of a settings file to migrate away "some day".
- [x] `crates/store`, `rusqlite` with the `bundled` feature (SQLite 3.53.2 compiled into the binary)
- [x] **Migration 1**: table `goals (id, target_json, created_unix, note, seq)`; the target
      is `ipc::TargetKey` serialized as JSON — identity only, never name or icon (see
      `ipc`) — because a column per variant would be a schema that changes with every new kind
      of unlock. Migrations apply in sequence, inside a transaction, and never re-run.
- [x] Minimal API: `Store::open`, `goals() -> GoalsRead { goals, unreadable }`, `add_goal`,
      `remove_goal`. A file written by a newer version is **rejected without touching it**
      (`NewerSchema { found, supported }`); a file that isn't SQLite gives `Unreadable`, not a
      panic; a row whose target can't be read ends up in `unreadable` by id and doesn't hide
      the others; `add_goal` only inserts, and an already-present id is an error (the app
      generates the ids: a collision is a bug, and rewriting the row would erase the goal
      that was already there).
- [x] `Debug` for `Store` written by hand: the derived version would have printed the
      database path (which under the data folder contains the Windows username) through the
      `Debug` of `rusqlite::Connection`. Caught by the review.
- [x] Tests — `crates/store/tests/goals.rs`: schema created at the current version, goals
      that survive reopening without re-running the migration, idempotent removal,
      the same id twice failing while leaving the first row intact, a newer schema
      rejected, a non-SQLite file, an unreadable row among others, and the saved row that
      contains **only the key** (read with rusqlite: no `name`, no `iconUrl`).
- [ ] **Migration 2, decided but not written**: the snapshots table (sections 1 and 4 of a
      `.dat`, with date and origin) imported from `save_backups\` and `online_logs\`. Arrives with M3.

### `wiki` — wiki text as a typed tree, embedded in the binary ✅

Implementation of backlog item B1 (analysis closed on 2026-09-05). Pure crate `wiki` plus a
standalone tool, `wiki-snapshot`, the only place in the repo that talks to the network.

- [x] Design spec — `docs/superpowers/specs/2026-09-05-wiki-dataset-design.md`
- [x] TDD plan in 12 tasks — `docs/superpowers/plans/archive/2026-09-05-wiki-dataset.md`
- [x] Report — `docs/superpowers/reports/2026-09-05-wiki-dataset-report.md`
- [x] `crates/wiki`: tree types (`Entry`, `Section`, `Block`, `Inline`, `Target`,
      `Infobox`, `Dlc`, `Style`), wikitext parser (templates, inline, blocks, sections,
      infoboxes), `Resolver` over the Cargo tables and corrections, `build(raw, corrections) ->
      Dataset`, `Dataset::embedded()`. Reads only `dataset/raw/`, knows nothing of `catalog` or the
      game's archives.
- [x] `crates/wiki-snapshot`: `fetch` (network → `dataset/raw/`) and `build` (`raw/` →
      `dataset/wiki.json`), dependencies only `wiki` and `ureq`; scripts `pnpm wiki:fetch` /
      `pnpm wiki:build`.
- [x] **Edition filter on the `dlc` bitmask** (bit 16 = Repentance+), not a manual
      correction: *Tonsil* isn't a wiki error, the page has two infoboxes (trinket 97 current,
      collectible 474 from Afterbirth+).
- [x] **Our own character map**, in `dataset/corrections.json` (`characters` key, built from
      `players.xml`): the ids in the wiki's `Infobox character` boxes aren't reliable
      (Isaac has Keeper's id, Magdalene has Cain's) and four pages use the plural
      `Infobox characters` template, invisible to the enumeration.
- [x] **`wiki.json` embedded, compressed**: pretty-printed (21,986,302 bytes) in the repo for a
      readable diff, deflated at build time by `crates/wiki/build.rs` (`miniz_oxide`) to 1,585,198 bytes
      behind the `embedded` cargo feature (on by default); `wiki-snapshot` builds without it.
- [x] Derived test (`tests/derived.rs`): `wiki.json` byte-for-byte equal to `build(raw/)`, so
      the two can never drift apart without the suite noticing.

#### The infobox, and the templates we already downloaded (2026-09-13) — phase 1 landed

Spec `docs/superpowers/specs/2026-09-13-wiki-infobox-design.md`, plan
`docs/superpowers/plans/archive/2026-09-13-wiki-infobox.md` (16 tasks, two phases). The parser read
every infobox parameter and then threw most of them away: **907 of 1727 entries reached the
frontend as `{"kind":"item"}` and nothing else**, because `Infobox::Item` and
`Infobox::Trinket` were unit variants. No new download — every byte was already in
`dataset/raw/`, fetched 2026-09-04.

- [x] **`Dlc::parse_codes`**: the `dlc` parameter concatenates codes without a separator (one
      page reads `a+nr`), so the two-character codes are matched first; what matches nothing
      is counted in `Diagnostics::unknown_dlc_codes` rather than dropped.
- [x] **Three facts rose to `Entry`**: `description`, `dlc`, `unlocked_by`. They are not
      specific to a kind — `description` is on 723 collectibles, 184 trinkets, 17 challenges,
      9 achievements — and the edition a thing exists in is a property of the thing.
      `unlocked_by` is documented as "what the wiki states", never "it is free".
- [x] **`Infobox::Item` and `Trinket` carry their box**: quote, template, quality, tags,
      recharge, devil/shop price, pools. **Empty infoboxes: 907 → 0.**
- [x] **`CollectibleTemplate { Passive, Activated }`**, a fieldless enum as a bare camelCase
      string. Not a `bool`: `activated: false` would have meant both "passive" and "familiar",
      since the wiki has no familiar template. `InfoboxKind::of` merged the two names until now.
- [x] **The four parsed kinds keep what they dropped**: boss `stage_hp`/`variant` (30 have a
      variant), challenge `character` (14), character `tears` (12) and `parent` (2).
      `recharge` and the two prices are `Vec<Inline>`, not numbers — the real values include
      `unlimited`, `one time`, `4s` and `{{dlcalt|6|r=4}}`.
- [x] **`tests/no_silent_parameter.rs`**: every wikitext parameter is either in a type or in
      `IGNORED_PARAMS` with a reason. It failed on its first run and found **`notes` on 17
      achievement infoboxes** — prose that existed nowhere else, because achievements are rows
      on storage pages and carry no sections at all.
- [x] **`tests/infobox_filled.rs`**: no entry has an empty infobox, with the counts asserted
      first so an empty dataset cannot pass the question trivially.
- [x] **Six section titles were falling through**: `Unlockable Items`, `How to Acquire`, `Bug`,
      `Interaction`, `Rewards`, `Excluded Items`. Discards: 65 entries / 2248 occurrences →
      59 / 2220. A second test pins the *deliberate* discards (`Trivia` 875, `Gallery` 346,
      `In-game Footage`, `References`) so a later "map everything" cannot quietly undo them.
- [x] **Contract handed on**: `DESIGN-BRIEF.md` and `ui/src/lib/ipc/types.ts` carry the new
      `Entry` and all six variants, checked field-for-field. `WikiInfobox.vue` takes the entry
      instead of the infobox and draws the three common facts once.
- [x] **Phase 2, the templates**: **25 unknown over 1290 occurrences → 17 over 200.** Seven
      taught: `m`/`machine` (a machine or beggar has no id, so `Inline::Concept` — and a new
      `Resolution::Concept` so it is not counted as a *failed* lookup), `ip` (item pools are
      keyed by name), `transformation contribution` (resolves like `{{tf}}`),
      `achievement text` (the only one whose argument is a comma-separated **list**, so it
      cannot go through `resolve`), the two `book of … synergy` templates (text in a named
      `description` parameter, plus the item, because a section is read on its own), and `bc`
      (a champion variant: the index is kept verbatim and **the colour is not invented** —
      which index is which colour lives in the wiki's template and nowhere we can read).
      `crates/wiki/tests/templates_understood.rs` holds both the >50 line and the total.
      What stays unknown is listed in the spec with a reason per group: icons whose word is
      already in the text, editorial marks with no content, and two table generators that
      would need the wiki's data modules.
- [x] **The wiki against the game** (`crates/ipc/tests/wiki_agrees_with_catalog.rs`): quality
      equal (0 of 576 disagree), every wiki tag a word the game's own vocabulary uses (9 of
      714), the game's pickup quote contained in the wiki's (8 of 719). It failed on its first
      run and found **three** defects: `tags` and `quote` were being read as raw text, so
      unparsed wikitext reached the dataset dressed as a tag and as a quote — and fixing those
      exposed a third, older one below.
- [x] **`{{dlc|code|text}}` was dropping its own text** (`crates/wiki/src/inline.rs`). The
      two-argument form is a marker that opens an edition scope; the three-argument form
      carries its own span and closes itself, and its second argument was never read.
      **313 occurrences across the snapshot**, in every kind of parsed text, with no
      diagnostic. `Boomerang tears {{dlc|r|+ DMG up + luck down}}` arrived as "Boomerang tears".
- [x] **The Cargo `dlc` mask is measured** (`crates/ipc/examples/dlc_mask.rs`, no game and no
      network needed). Every mask seen is a contiguous suffix of the editions — 31, 30, 28, 24
      — which is the shape of "valid from this edition onward" and of nothing else; the lowest
      set bit agrees with `catalog::origin_of` on **712 of 720** collectibles. The eight
      exceptions are one five-id editing slip on the wiki (342–346, Blue Cap among them), the
      two id-reuse rows where the id-range lookup is the limited side, and one boundary row.
      `in_current_edition`'s reading was right; the mask stays **unused** here all the same,
      because adopting a second source is its own change. `catalog::origin_of` is now exported.
- [x] `ipc::wiki`: `WikiInfo`, `PatchView`, `WikiCounts`, `wiki_info(dataset, game_updated_unix)`;
      `discovery::GameInstall.updated_unix` to check freshness against the `appmanifest`'s
      `LastUpdated`.
- [x] `app`: command `wiki_entry(target) -> Result<Option<Entry>, IpcError>`,
      `IpcError::WikiUnavailable`, `WikiInfo` in the `extraction_report` payload.
- [x] Frontend: types mirrored in `ui/src/lib/ipc/types.ts`, `wikiEntry` wrapper in
      `ui/src/lib/ipc/wiki.ts`, recursive components `WikiInline.vue`/`WikiBlocks.vue`, "Wiki"
      section in the verification screen (meta, freshness, one item id, the `Entry` rendered
      with navigable `Ref`s).
- [x] **Result**: 719 items, 188 trinkets, 641 achievements, 102 bosses, 45 challenges,
      32 characters; 22 unresolved references (18 `{{e|…}}`, 4 `{{i|…}}`; there were 20 before
      the final fix wave `9ec171c`, which made `parse_inline` recurse into every unknown
      template and into `{{dlcalt|…}}` — references previously trapped in raw text are now
      parsed and simply remain unresolved); 2 pages with no id; snapshot
      2026-09-04T17:33:31Z, known patch v1.9.7.17.
- [x] **A seventh kind, and a fresher snapshot** (2026-09-13, snapshot
      2026-09-13T14:01:45Z). **16 transformations**, each with the count its page states and
      the set of items that counts toward it — the union of the infobox's list and the body's
      own tables, because each loses something the other has: Guppy's infobox omits the
      trinket its body lists. `dataset::SCHEMA_VERSION` 2. Unknown templates 200 → 196, and
      **13 of the 16 pages state their item set twice and differently**, which
      `transformationSourcesDisagree` counts rather than resolves. Adult has neither count
      nor set on purpose: its page is about pills. Report in
      `docs/superpowers/reports/2026-09-13-transformations-report.md`.
- [ ] Resolver and parser polish deferred from the review: listed by theme in the
      execution report, "What's left out" section. **Two of the three named there closed
      on 2026-09-08**, and the third turned out not to be polish at all.
      - [x] *A table cell split on `||` without counting `{{…}}` braces.* `split_cells`
            counts bracket depth; Mystery Egg's `{{e|Mask + Heart||Heart}}` resolves to
            entity 93 instead of leaving two half templates in text nodes.
      - [x] *A lone `}}` line.* It was doing more damage than the report recorded: not
            only a junk paragraph but a **list cut in two**, fifty times over, because
            reaching the paragraph branch is what flushes an open list. Dropped and
            counted in `Diagnostics::orphan_closers`. The raw-syntax count in
            `text_nodes_carry_no_raw_template_syntax` went 125 → 83.
      - [ ] *Representing the multi-line wrapper itself* — **reclassified as a design
            decision, not a parser fix.** The template's content is block-level, so
            expressing it needs a `Block` variant, and `Block` crosses the IPC.
            `column list` (51 occurrences) is pure layout and could be dropped;
            `{{bug|…}}` (4) is not, and the crate already models it specially in the
            single-line case; `Book of … synergy` (7) sits in between. Transparent versus
            modelled changes what the wiki screen can render, so it belongs to that
            screen's design — same rule that keeps C2 deferred.
      - [x] *Entity aliases* — **measured on 2026-09-08, and it wasn't that.** The whole
            unresolved set was 22 references over 7 distinct keys, and the aliases were
            never the problem: three keys were `{{i|1=Name}}`, MediaWiki's explicit
            positional syntax, which `assemble` filed under `named` leaving `args` empty
            (fixed); three are `Killswitch`, `Pressure Plate` and `Reward Plate`, which
            *are* in `entity.json` as aliases of `Buttons` and carry `id: ""` because
            they're grid entities the game gives no `EntityType`; and one is `Tonsil`,
            unresolved on purpose. `unresolved` is now `{e: 18, i: 1}` and that is the
            floor, pinned per template in `diagnostics_are_bounded`.
      - [ ] *Grid entities have no `Target`* — the leftover of the line above, and the
            same shape as the wrapper decision: resolving the three id-less buttons needs
            a `Target` variant, which crosses the IPC. Design, not resolver work.
      - [x] *Unknown tables in `corrections.json` ignored silently* — **guarded on
            2026-09-08.** `CORRECTED_TABLES` names the tables `apply` is called with, and
            two tests make a correction that matches nothing a red instead of a no-op: one
            over the real file, one over a file deliberately wrong in both ways, because
            the first is vacuous while `pageId` is empty.
      - [ ] *Corrections looked up by exact `_pageName` rather than normalized `key()`* —
            still open, and latent for the same reason: `pageId` is empty, so nothing is
            looked up yet. The guard above will catch a title that matches no page; it
            won't catch one that matches except for case.
      - [x] *The alias/title double index untested* — moot: the character half of
            `corrections.json` is a name → id index used by `{{c|…}}` anywhere in the
            text, and `diagnostics_are_bounded` already asserts that no `{{c|…}}` ever
            comes out unresolved, which is stronger than any spelling check.
- [ ] "Open on the wiki" link and runtime dataset update from GitHub: out of scope for
      this cycle, that's design and M5 work.

### Tauri app ✅ (skeleton), Completion screen ⬜

- [x] Tauri 2 + Vue 3 / Vite / Tailwind v4 skeleton
- [x] IPC commands exposing `discovery` and `core-save` to the frontend — four, **none
      taking a path**: the active profile is backend state
- [x] Persisted profile choice
- [x] Verification screen on real data (rough, throwaway)
- [x] `ui/scripts/scan-conventions.mjs`: enforces the seven rules no linter covers
- [x] **App startup verified** (2026-09-03). The window opens, the chain
      Steam → game → save → parser → matrix runs end-to-end on the real profile:
      4 candidates, selection, 10 sections read, a 34 × 10 matrix with 151 marks started out of
      321 readable and the 19 `unknown`s falling exactly on the Delirium column of the third group.
      *(Those are that day's numbers and stay as measured: since 2026-09-08 the matrix is
      34 × 12, 166 started out of 368 readable, 40 unknown.)*
      Required a `pnpm install` in `ui/` (never done before) and a fix to `tauri.conf.json`.
- [x] **Five graph commands** (2026-09-05): `unlock`, `next_steps`, `plan`, `add_goal`,
      `remove_goal` — the last two are the app's first write commands, and they only write to
      `isaacdome.db`. `Store` is opened once in managed state, like the catalog;
      a database that won't open **degrades**: `plan` responds with `storeAvailable: false` and
      a diagnostic with the reason (for a newer schema: "database from a newer version
      (N > M)"), `add_goal` and `remove_goal` respond with `storeUnavailable`. `add_goal` rejects
      a target the catalog doesn't know (`unknownTarget`) and distinguishes the case where it
      can't verify it at all (`catalogUnavailable`, today only when the game is absent). The local
      module `store.rs` (I/O for `settings.json`) became `settings_file.rs`: the name `store` belongs
      to the crate. From the final review: `plan` also degrades when **the query**
      fails, not just when the file won't open (the mapping lives in `plan_parts`, pure and tested
      without Tauri); `add_goal` takes a `TargetKey` and `plan`/`remove_goal` open the
      catalog to resolve the goals they return.
- [x] TypeScript mirror of the contracts in `ui/src/lib/ipc/types.ts`, wrapper in `graph.ts`;
      verification screen extended with Unlock totals, diagnostics and the five Next
      Steps with icon, text and condition. On the live Steam save it shows 381 of 642 completed:
      the `samples/live.rep+persistentgamedata1.dat` snapshot from 2026-08-31, which the
      tests run against, has 379. The number on screen follows the file, it isn't a constant.
- [ ] **Next steps for the webapp**, from the contracts review (2026-09-05):
      - [ ] the string unions in `ui/src/lib/ipc/types.ts` — `CandidateView.prefix`,
            `MissingReason`, `OriginView`, `ItemKindView`, `CandidateSource`, `StepsBasis`:
            that's six of them since fieldless enums travel as strings, and they will grow —
            they need converting to `const … as const`
            in the modules that own them (rule 5), and `ui/scripts/scan-conventions.mjs` needs
            to gain checks for rules 4 (raw `<button>`/`<input>`: `App.vue` still has one in the
            candidate list) and 5, unchecked by anything today;
      - [ ] `next_steps` rebuilds the entire `UnlockView` (base64 icons twice on every
            load): acceptable for the verification screen, needs rethinking for the real one;
      - [ ] in the visual checks the on-screen number is compared against a dated reference
            file, not a constant: the app reads the live save, which changes as you play.
- [ ] Real Completion screen — **this is webapp work**, not base work: starts after the
      handoff to design (see below)
- [x] shadcn-vue, Reka UI, vue-i18n — arrived with the design system's first cycle
      (2026-09-10); Pinia, Vue Router and TanStack arrive with the screens
- [ ] Design system — the Claude Design export arrived on 2026-09-10, built in three cycles:
      - [x] **1. Foundations and primitives** (2026-09-10) — tokens, font, motion, i18n,
            `cn()`, 23 primitives on a development-only Kit page. Spec
            `docs/superpowers/specs/2026-09-10-design-system-foundations-design.md`, plan
            `docs/superpowers/plans/archive/2026-09-10-design-system-foundations.md`
      - [x] **2. App components** (2026-09-10) — title bar and tabs, navbar, section
            sidebar, KPI tile, matrix cell in sprites or bars, wiki tokens and blocks, data
            states, collapsible card; presentational, on the Kit page. Spec
            `docs/superpowers/specs/2026-09-10-design-system-components-design.md`, plan
            `docs/superpowers/plans/archive/2026-09-10-design-system-components.md`
      - [ ] **3. Screens** — decomposed into seven sub-projects in
            `docs/superpowers/specs/2026-09-11-screens-shell-profile-design.md`, each with
            its own spec → plan → execution; decisions marked "(delegated)" wait for the
            first-launch review:
            - [x] 3.1 Shell and profile selection (2026-09-11) — tabs owning locations, Vue Router in
                  memory, Pinia, the live window chrome, the profile indicator, the profile
                  screen and its gate, placeholders, fixtures for `pnpm ui:dev`. Plan
                  `docs/superpowers/plans/archive/2026-09-11-screens-shell-profile.md` (its
                  checkboxes are the step-by-step state)
            - [x] 3.2 Completion (2026-09-11) — the marks matrix on the active profile: four
                  KPIs with their denominators, base and Tainted groups, the unknown block,
                  a tooltip per cell, sprites or bars; B13 closed (the marks map in `ipc`, the
                  icon protocol serving crops). Spec
                  `docs/superpowers/specs/2026-09-11-screens-completion-design.md`, plan
                  `docs/superpowers/plans/archive/2026-09-11-screens-completion.md`
            - [x] 3.3 Next steps, Unlock, Plan — split in two halves
                  (`docs/superpowers/specs/2026-09-11-screens-graph-design.md`):
                  - [x] 3.3a the node, Next steps and Unlock (2026-09-11) — a node's state and
                        its why, the Badge's partial state, four facets with data and their
                        counts, search, three sorts, a virtualized table; plan
                        `docs/superpowers/plans/archive/2026-09-11-screens-graph.md`
                  - [x] 3.3b Plan (2026-09-11) — the queue you drag, a repair that says where a
                        row stopped, the diagnostics as alerts and footnotes, the proposal
                        beside it; "in coda" and one click to add on Next steps and Unlock; a
                        move names the row it lands under (`queue_move(achievement, after)`).
                        Spec `docs/superpowers/specs/2026-09-11-screens-plan-design.md`, plan
                        `docs/superpowers/plans/archive/2026-09-11-screens-plan.md`
            - [x] 3.4 Collection (2026-09-11) — the save's item collection joined with the
                  catalog's collectibles (`ipc::collection_view`, the `collection` command, a
                  pack payload), one state per item, facets on quality, pool, kind and origin
                  over a virtualized table. Spec
                  `docs/superpowers/specs/2026-09-11-screens-collection-design.md`, plan
                  `docs/superpowers/plans/archive/2026-09-11-screens-collection.md`, branch
                  `feature/screens-collection`
            - [x] 3.5 Wiki in tabs, search (2026-09-12, both halves) — spec
                  `docs/superpowers/specs/2026-09-12-screens-wiki-search-design.md`, branch
                  `feature/screens-wiki-search` for A, `feature/screens-search` for B:
                  - [x] 3.5a the Wiki in tabs (2026-09-12) — the landing with the dataset's
                        provenance, a category's list, a page in a tab (figure, infobox per
                        kind, sections, references that open in place or beside), the
                        `wiki_index` command and `IconRef::Page`. Plan
                        `docs/superpowers/plans/archive/2026-09-12-screens-wiki.md`
                  - [x] 3.5b search (2026-09-12) — one index over the catalog's names, the
                        achievements' conditions, the wiki's titles and the body of its
                        sections; `search(query, limit)` with the wiki half built once;
                        six tiers with the profile inside the ranking; the `Ctrl+K` palette
                        and the Search screen, whose rows are **destinations** and not hits.
                        B5 closed, and measured rather than assumed: 1,727 pages indexed in
                        15 ms, a query in 15–19 ms, so the FTS5 fallback is not needed. Plan
                        `docs/superpowers/plans/archive/2026-09-12-screens-search.md`, report
                        `…-screens-search-report.md`, branch `feature/screens-search`
            - [x] **3.5c Interface scale (2026-09-12, B26)** — pulled ahead of 3.6 on the
                  owner's request; numbered `c` because `b` was already the search half
                  of 3.5, which it now precedes. Spec
                  `docs/superpowers/specs/2026-09-12-screens-scale-design.md`, plan
                  `docs/superpowers/plans/archive/2026-09-12-screens-scale.md`. The whole interface
                  scales from Settings **as
                  Discord's zoom level does** — a slider over its eleven steps 50–200,
                  `Ctrl` `+`/`-`, a preview card pinned at the top of the page — persisted
                  in `settings.json`, applied before the first paint. It goes first because
                  it rewrites the token files — px to rem, the
                  root font size as the scale, sprites at whole multiples — and every screen
                  built after it is built on the scaled tokens. Done when the slider moves
                  the whole app with no element left at its old size, on the Kit page and
                  in the built app. Needs a `Slider` primitive (none in the kit yet)
            - [x] **3.5d "Bloccato" says where to go (2026-09-12)** — pulled ahead of 3.6 on
                  the owner's request ("dove c'è scritto bloccato mi serve sempre un link che
                  mi spiega come sbloccarlo, non basta il nome"), because it changes the live
                  IPC contract and every screen that draws a node. A requirement and a
                  collection lock carry `page: Option<Target>`, `Some` only when the embedded
                  dataset really has that page; the catalog → page mapping left `search.rs`
                  for `crates/ipc/src/wiki_target.rs`, one function for both readers. The
                  badge stops being a tooltip and becomes the trigger of a `dropdown-menu`:
                  one label per kind, one entry per blocker, click navigates and Ctrl opens
                  beside — the palette's gesture. An entry with no page stays in the menu,
                  disabled: never a link that leads nowhere. A mark and a counter carry none
                  (B36), and what a node *unlocks* is B35. Spec
                  `docs/superpowers/specs/2026-09-12-blocked-menu-design.md`, plan
                  `docs/superpowers/plans/archive/2026-09-12-blocked-menu.md`, report
                  `…-blocked-menu-report.md`, branch `feature/blocked-menu`
            - [ ] 3.6 Settings and About — provenance, credits, the three promises; About
                  becomes a dialog, not a page (B25); the profile screen becomes a welcome
                  flow (B17); the KPI and matrix changes of B20, B22, B23
            - [ ] 3.7 Tabs that survive a restart (B6) — **the session document and the two
                  settings landed early, on 2026-09-13, with the tray** (`store` migration 3,
                  `lib/window/sessionDocument.ts`, `/settings/background`). What is left for
                  3.7 is the rest of what that document is meant to hold: the sidebar's width
                  and every table's dragged size, per table (B27)

---

## Handoff to design — what closes out the structural base

Decision from 2026-09-04. The webapp starts once the **structural base** is done, and the
structural base is defined as: **all static data normalized and all IPC contracts
fixed**, not "all real data". Two of the seven screens (Run, Live) live on data that
only exists at runtime, and three more (Next Steps, Unlock, Plan) depend on the M2
graph, which is the project's bottleneck. Waiting for that data before designing would mean
waiting for M4.

The "Vue only receives already-resolved JSON" constraint makes this possible: the only
dependency between webapp and backend is the shape of the view-models in `ipc`. Fixing that
shape with fake data behind it is, for design purposes, the same as having the real data;
when the real graph arrives the types don't change.

| Screen | Data today | What unblocks it |
|---|---|---|
| 0 Profile selection | ✅ complete | — |
| 4 Completion | ✅ complete (34 × 10 matrix, head icons) | — |
| 5 Collection | ✅ 909 items with name, sprite, quality, tags and pools | — |
| 2 Unlock | 🟢 designable on the contract: real nodes (name, icon, condition, done, what it unlocks, source DLC) for 641 slots, `graph` stubbed | M2 fills in `graph` without changing the types |
| 1 Next Steps, 3 Plan | 🟢 on the contract: the first 5 not-yet-done with `basis: stub`; real goals saved in `store`, `expansion: stub` | M2 and M3 fill in the stubs |
| 6 Run, 7 Live | 🔴 | M4 — runtime data by definition |

Path, in order:

- [x] **1. `catalog` plan B** — one session. After this, every static datum derivable from the
      game's files is normalized and the crate has its final shape.
- [x] **2. IPC contracts for the M2 and M3 screens** — spec dated 2026-09-05
      (`docs/superpowers/specs/2026-09-05-graph-contracts-design.md`), plan in nine tasks
      (`docs/superpowers/plans/archive/2026-09-05-graph-contracts.md`) executed the same day,
      report in `docs/superpowers/reports/2026-09-05-graph-contracts-report.md`. Types in `ipc`
      and TypeScript mirror in `ui/src/lib/ipc/`: a single node for the three screens, with the
      data that already exists (catalog, save, goals in `store`) **real** and whatever the
      graph doesn't know yet as a **declared** `{ kind: "stub" }` — not "fake data", which
      would be a value that looks computed. The node's three states (done · unlockable
      now · blocked by N), fan-out and missing steps have their place in
      `GraphInfo::Computed` and arrive with M2 without changing a single type. Pinned by tests on the
      JSON shape. The three decisions below are made on real data:
      - [x] Reverse index `AchievementId → what it unlocks`: `Catalog::unlocks`, built once
            in `build`, deterministic order; every `unlocked_by` appears exactly
            once (real test on the sum of edges).
      - [x] Section 1 achievement ↔ slot mapping: **`slot[id]`**, pinned on the real
            profile (169 of 171 seen items with the achievement completed). Slots 638–641 are
            beyond the catalog: 4 `unknown` nodes, not 5 — slot 0 is not a node.
      - [x] `unlock_condition` as displayable text: in the contract it's called `hint`, it's
            `string | null`, and `null` is the normal state for 354 of 637 achievements.
- [x] **3. `DESIGN-BRIEF.md` aligned** — revisited on 2026-09-05 after 1 and 2: §4's status
      light updated with the three graph screens on-contract, new §7 with the TypeScript
      types and the real/stub table, fourth question in §12 (how to draw a node whose graph
      is `stub` without it looking like missing data).
- [x] **4. Handoff to Claude Design** — **done: the package is handed over and the design
      is under way as of 2026-09-09.** Screens 0, 4 and 5 on real data, the other four
      on fixed contracts. From here M2 proceeds in parallel with the webapp without touching the
      types the frontend consumes.
      The material was ready on 2026-09-06 and **had since aged out**: the committed package was
      the output of `a9a32ee`, taken before M2 and M3 landed, where every payload still declared
      `"kind": "stub"` — 641 nodes, `basis: stub`, `expansion: stub` — and `contracts/types.ts`
      was the pre-M2 copy, with `GraphInfo = stub | computed`, no `RequirementView` and no queue.
      Handing that over buys a design of the placeholder. **The four steps below closed on
      2026-09-08**: the package on disk is the post-M2/M3 contract on the real profile, and what
      remained of this item was the act of handing it over, and that happened.

      **From here the IPC contract is live.** The rule in `CLAUDE.md` — the contract does
      not change for the convenience of a screen that doesn't exist yet — stops being a
      precaution and starts being a constraint with someone on the other end of it. Any
      change to a type in `ipc` or to `ui/src/lib/ipc/types.ts` from now on is a change to
      material a design is being built on, and has to be handed on rather than merely
      committed.

      *Known drift at handover:* the rename of sections 3 and 6 (`017131c`, 2026-09-09)
      changed two label strings in `contracts/payload/save_summary.json` —
      `per_char` → `level_counters`, `cards_pills` → `bosses` — and the matching rows of
      the brief's save-format table. No count moved and no TypeScript type changed. If the
      copy in design's hands predates that commit, those two strings are the whole
      difference.
      - [x] **a. A queue payload in `design-export`** (2026-09-08) — `queue.empty.json` and
            `queue.with_rows.json`, built through `plan::Queue::enqueue` and `GraphDeps`,
            i.e. the very functions behind the Tauri command: the package can't show an
            order the app wouldn't produce. Six tests on the pick rule.
            **The first rule was wrong and the real export caught it**: "the first node
            blocked by two or more steps" finds nothing on a profile at 385 achievements,
            and the package came out with a single row — without the one thing the queue
            exists to show. The rule is now "the deepest chain the profile actually has",
            ties broken by the lower id because a committed package has to regenerate
            identically. It degrades to a chain of one instead of to nothing.
            Also here: **`GraphDeps` moved from `crates/app` to `ipc`**, with three tests it
            never had while it sat in the crate that by convention isn't tested.
      - [x] **b. `DESIGN-BRIEF.md` re-aligned on the post-M2/M3 contract** (2026-09-08) —
            fourteen places, all the same point: `{ kind: 'stub' }` has left the wire.
            `GraphInfo` is `computed | partial`; `RequirementView` and the node's `missing[]`
            were missing from the document entirely; `StepsBasis` is `'fanOut'` and Next
            steps changed *meaning*, not just values — a `partial` node is not a step, and
            without a catalog the list is empty with a diagnostic saying so. §7.5's table
            redone, §0/§4/§10/§13 and question 4 rewritten around `partial`. **New §7.6 on
            the queue**, which the brief had nothing about at all: the types, the four states
            of a row, and the rule the whole thing rests on — a move repairs, so there is no
            rejected drop and no error toast to design.
      - [x] **c. `pnpm design:export` re-run over the whole package** (2026-09-08, evening) —
            run on the machine with the game reinstalled, the `samples/packed` junction live
            and the real profile: 8 archives, 19,473 entries, 2,074 images from the archives
            (1,639 in atlases, 435 single files), 48 sheets cut into 3,759 pieces, 10 wiki
            pages, 5,837 files, 36 MB. The package is committed on purpose (see
            `.gitignore`): Claude Design opens it from a fixed path, and it would vanish on a
            branch switch. The package README forbids the assets ending up in a public
            repository — `origin` is private, which is what makes committing them acceptable.
            **The diff says the extraction is deterministic**: 12 files changed and 2 added,
            and not one image byte moved between the `a9a32ee` output and this one. What
            changed is the whole point — `contracts/types.ts`, the unlock payloads, and the
            brief. The profile behind it: **386 of 641 done**, 620 nodes `computed` against
            21 `partial`. Two `"kind": "stub"` strings survive, both `PlanExpansion::Stub` in
            `plan.*.json`, and that one **is** current behaviour: the Plan's expansion is
            genuinely not computed, the queue is what replaced it.
      - [x] **d. The file count, here and in the package README** (2026-09-08) — the README
            was already generated (`readme(img.entries.len())` = 5,833) and correct; the stale
            number was in `DESIGN-BRIEF.md`'s header, which claimed **2035 images**. Now taken
            from the export's own report, and stated as two figures because they are two
            things: **5,833 catalogued images** (2,074 from the archives + 3,759 cut from the
            game's sheets) living in **5,837 files**, since the regular families are packed
            into atlases rather than written one file each.

Doesn't block the handoff but blocks Collection and Unlock: **`Archive::open` loads 1.3 GB**
to extract sprites (see open blockers), and the graph commands do this on every
call. Needs solving before a screen asks for a hundred icons at once, i.e. before
building the Collection screen, not before designing it.

---

## Next up — structural cleanup

Read-only survey of 2026-09-12 over `crates/` and `ui/src`. It found no bug and no broken
rule: no `unwrap()` outside tests, exhaustiveness respected, no `TODO` and no
`@ts-expect-error` except the one that *is* the test, the dev pages behind
`import.meta.env.DEV` with dynamic imports, `en.ts` and `it.ts` in key parity. What it found
is what five screens built well, one after another, each on the shape of the one before,
cost: **the same file exists twice under two names.**

**The rule for this section: unification is measured in files that stop existing.** A task
that adds a shared module and leaves in place the two it generalizes has not been done, it
has doubled. Every item closes on a file count going *down*, and the count is written into
the item. Where a pair must stay two files, the item says which and why, so nobody has to
wonder whether it was forgotten.

> **Corrected on 2026-09-14, by N3, the last item to run under it.** The rule catches what it was
> written against and nothing else. N3 created seven files and deleted three, and left the source
> 78 lines lighter: the duplicated engine became one engine plus the specs, values and helpers it
> had always needed, and no count of files tells that apart from the failure above. **Read it as a
> smell, not a criterion.** What decides an item is its own "done when" — for N3, that no file
> under `screens/collection/` is a copy of one under `screens/unlock/` and that adding a facet to
> one screen touches no file of the other.

Nothing here is a feature and nothing changes what a screen shows. Each item gets its own
branch cut from `develop`, like any sub-project (the rule of 2026-09-11): none of this
starts on top of a sub-project in flight. **N7 is not a new item**: it is B2 of
`docs/IMPROVEMENTS.md`, placed here in the order it has to run in.

**The order was "cheapest and safest first" until M4 got its design (2026-09-12), and that
is what reorders it.** The numbers N1–N8 are names, not positions — renumbering them would
break the references other items and `docs/IMPROVEMENTS.md` make to them — so the order is
written out here instead:

> **N1 → N2 → N7 → N6 → M4 sub-project 1 → N8 → N3, N4, N5**

**N6 and N7 swapped in the event, on 2026-09-13, and not because the argument changed.**
N7 generates TypeScript from the Rust types, and `feature/wiki-infobox` is reshaping
`crates/wiki` — `wiki::Target` is one of the five types generation has to reach. Generating
from a moving target means generating twice, which is the same reason N7 went in front of M4
in the first place. N6 touches no file of that branch, so it went first. **The order below
is the argument; what actually ran is N1 → N2 → N6 → N7.**

Three of the items are cheap *now* and expensive after M4, and that is the whole of the
reason:

- **N7 moves from last to third.** It was last because it is expensive and because
  generating the contract while the error type is still moving means generating it twice —
  the first half of that still holds, which is why N2 keeps its place in front of it. What
  changed is the second half: M4 adds a new family of view-models to the contract, and every
  one of them written before N7 is hand-mirrored into `types.ts` and then regenerated. The
  repository's largest silent risk gets *larger* between now and M4, not smaller.
- **N6 moves in front of M4.** M4 adds commands to `crates/app`. Splitting the 964-line file
  first means they land in `commands/` already; splitting it after means splitting a bigger
  file, and the new commands spend that time in the place the rule says they may not be.
- **N8 moves behind M4, not in front of it.** Its expensive half and M4's watcher are the
  same subject seen twice: a `SaveState` has to invalidate when the `.dat` is rewritten, and
  M4's log watcher exists because the game announces exactly that
  (`Saving PersistentGameData to Steam Cloud: …`, B8 finding (a)). Doing N8 first means
  inventing an mtime heuristic that M4 then replaces with the game's own statement. **N8's
  free half is not held back by this** — `next_steps` taking the `UnlockView` it is a filter
  over is half a screen load for no new state, and can be lifted out whenever.
- **N3, N4 and N5 are untouched by M4** — they are frontend, and M4 sub-project 1 has no
  screen. They keep their place at the end, where they gate 3.6, 3.7 and B3 rather than
  anything here.

- [x] **N1. The names left over.** *Done 2026-09-13, `feature/cleanup-names`.*
      `ICONE_DI_ESEMPIO` is `SAMPLE_ICONS` and `SEGRETO` is `SECRET_PATH`, with the path it
      holds spelled in the language of the assert that reads it. That constant also gained
      the comment it needed: there are **two** asserts, on the whole path and on the word
      inside it, because a `reason` that re-rendered or escaped the path would slip past the
      first on its own. The coupling was load-bearing and unwritten, which is how a rename
      turns a leak test into a test of nothing.
      **One notation, and it is structural**: each crate with a test-only entry point has
      exactly one `pub mod for_tests`, and nothing test-only appears anywhere else in its
      public surface. **Seven crates** have one — `catalog`, `discovery`, `graph`, `ipc`,
      `store`, `unpack`, `wiki`. The inner items went back to being internal:
      `Graph::from_edges` and `from_requirements` are `pub(crate)` constructors again,
      `search`'s `documents` and `progress` are `pub(crate)` instead of wearing
      `#[doc(hidden)]` twins, and `Doc` and `ProgressMark` left `ipc`'s contract — they are
      the return types of test-only calls and nothing else reads them.
      **The survey had found four entry points; there were seven, and the count moved
      twice.** `unpack::__lzw_decompress` and `catalog::__heads_parse` carried the same `__`
      prefix and were not on the list; `catalog`'s serves an **example** rather than a test,
      and the module's doc says so rather than inventing a second word, which would rebuild
      the thing this item removes. The seventh, **`discovery::testing`**, was found a session
      later while N2 typed that crate's diagnostic: a `#[doc(hidden)] pub mod` under a
      different word, matching neither the `__` prefix nor the `_for_tests` suffix the
      closing grep looked for. **The lesson is about the report, not the code**: "one
      notation" was written as a closed fact when what had actually been established was
      "the two spellings I grepped for are gone". A closing criterion states what it can
      see, and this one could not see a third spelling.
      **Closed against** `grep -riE '(icone|segreto|_di_)' crates` and
      `grep -rn 'pub fn __\|as __\|pub mod testing\|_for_tests' crates`, both silent, and no
      `pub use` naming a `for_tests` item. **Over `crates` and not `ui/src`**: the Italian
      locale file legitimately contains the Italian word *icone* in its prose, so a grep that
      spans it can never go quiet — a closing criterion that cannot be met is worse than
      none, because it gets read as "still open" forever.

- [x] **N2. `reason: String` leaves the IPC.** *Done 2026-09-13, `feature/typed-ipc-reasons`.*
      **Four enums, not one.** The item named `StoreReason`; the boundary carried the same
      defect in three more shapes, so `crates/ipc/src/reasons.rs` holds `IoReason`,
      `SaveReason`, `SettingsReason` and `StoreReason`. `IoReason` has no variant with a
      field and is therefore a **bare camelCase string** by the repo's own rule; the other
      three carry one and are tagged. `StoreReason` gained `DataDirNotCreatable` beyond the
      four the item listed — "the folder is unknown" and "the folder would not be created"
      are two things, and the item's list was written from a skim.
      **`store_reason` and `describe_open_error` are gone**, and so are the two Italian
      sentences. The mappings live where they can be tested — `OpenError` and
      `io::ErrorKind` in `ipc`, `StoreError` in `store`, which already depends on it — and
      each is pinned by a **property**, not only by a table: whatever the OS or SQLite
      wrote, none of it survives into the serialized reason.
      **One case was answering with a sentence that hid which case it was.**
      `queue_import_goals` used `"database illeggibile"` in *both* arms: a database that
      would not open and a query that failed read identically, and the reason the first arm
      already had was thrown away. Each says what it knows now. This is the item's own
      charge — an untyped field lets the wording drift — in its worst form, where the drift
      had eaten the information.
      **The defect had an instance outside `IpcError`, and the UI was printing it.**
      `SetupDiagnostic::UnreadablePath` carried `io::Error::to_string()` and
      `NoSavesCard.vue` concatenated it. `discovery::Diagnostic` carries the
      `io::ErrorKind` now. On the way, `Discovery` and `Diagnostic` lost their `Serialize`
      derive: nothing used it, and both hold a `PathBuf`, so serializing either would have
      put a full path on the wire — the one thing the boundary forbids.
      **The frontend maps to keys, not to text.** `ui/src/lib/ipc/errorText.ts` is a pure
      function beside the wire types: `useIpcErrorText` joins the parts it returns and
      `PlanAlerts` asks it for the store reason. It is pure because the UI suite tests logic
      and not components — a composable calling `useI18n` needs an app context, and the
      mapping is the part worth checking. `storeNewerSchema` interpolates `{found}` and
      `{supported}`: the word order around a value is the translation's business.
      **The test walks every variant of every error** and asserts each key exists in `en`
      **and** `it`. A missing key renders as the key itself — it reads as a bug report to
      the user and fails nothing — so a new variant with no text breaks the test rather than
      the app.
      **Done when** — all met: no field of `IpcError` is a `String` the UI concatenates,
      the two functions are gone, and the two anti-leak tests in `app` assert on a variant.
      The last one is what makes them structural: a unit variant has no string for a path to
      hide in, so the test stopped being a search for a word.

- [x] **N3. One faceted list, not two.** *Done 2026-09-14, `feature/shared-facets`.*
      `lib/facets/faceting.ts` holds `createFaceting({ order, values, text, options })`, and
      `components/facets/` holds one drawer, one toolbar and one state toggle. **Six components
      became three**, and the source lost **78 lines net** (551 added, 629 removed, tests
      excluded) while the tests gained 143 — the engine has 167 lines it never had, and 145 lines
      of tests that existed twice are gone.
      **Read literally, this item's own rule says it failed**: "unification is measured in files
      that stop existing", and seven files were created against three deleted. The rule is still
      right about what it was written against — a shared module added *beside* the two it
      generalizes — and it is wrong as arithmetic here, because a duplicated engine that becomes
      one engine plus five specs, values and helpers is more files and less code. The half of the
      item that does answer is the behavioural one it also writes down: **no file under
      `screens/collection/` is a copy of one under `screens/unlock/`, and adding a facet to one
      screen touches no file of the other.** Both hold, and the second is measured rather than
      asserted: `FacetId` appears in exactly four files, all of them Unlock's — none of the
      Collection's, and none of the three shared components. The rest is the diff, above, and it is
      the honest number rather than the flattering one.
      Its tests run on a **row type neither screen owns** — tested through `UnlockNode` the engine
      would be proven to work for Unlock and say nothing about being generic, which is the whole
      claim and the one B3 leans on.
      **Three things the item's own description had wrong**, each found by doing it:
      `matchesQuery` was *not* identical — Unlock searches a text built from three fields, the
      Collection a name — so the search text is a function the spec brings, not a field the engine
      assumes. **Two** values crossed between the screens, not one: `OriginValue` and the kind set,
      the latter read by `CollectionRow.vue`, `QueueRow.vue` and a *component* module, which is the
      clearest sign it was never Unlock's. It is `TargetKind` now, in `lib/ipc/values.ts`, its
      first four values bound to `ItemKindView` rather than retyped. And the two label modules,
      listed as staying two, shared twenty lines nobody counted — `originLabel` and `oneOf` are
      shared now, with the tests neither had.
      **Unlike Unlock's, the Collection's faceting is a factory.** `CollectionView.pools` is the
      catalog's own order filtered to the items listed, built in Rust from `c.pools()`, and it
      cannot be read back off the rows: first appearance in id order is a different sequence. So
      the options take an input the view brings, and `emptyFilter(order)` exists because a filter
      has to be built before any pools do.
      The drawer's column count is `facets.length` reaching the grid as `--facet-columns`, the way
      `MarksGrid` already passes the boss count — and `grid-cols-facets` was **checked in the built
      CSS**, because an `@utility` nothing references generates nothing and a grid with no template
      would have failed silently. The two shared components are the repo's first **generic SFCs**:
      with `string` props each screen would narrow the emitted sort back to its own union, which is
      the same small duplication moved rather than removed.
      **What stays two, and what it cost to know**: `UnlockRow`/`CollectionRow` and
      `UnlockTable`/`CollectionTable`, as the item says. Measuring the tables found 27 of 84 lines
      still shared — not columns but the scroll box around them — and four screens reading
      `--spacing-unlock-body`. Both are **B43**, not this branch: different subject.
      **Not verified here**: nothing in the suite draws these components, and this machine has no
      game. The three unified components are behaviour-preserving by construction — same template,
      same classes — except the drawer's grid, which changed mechanism. One look at `pnpm ui:dev`
      closes that.

      *The original entry, for the record:*
      `ui/src/lib/graph/unlockFilter.ts` and `ui/src/lib/collection/collectionFilter.ts`
      hold `matchesQuery`, `matchesFacet`, `matchesFacets`, the facet counts and the active
      count **identical word for word, comments included**: only the row type differs. The
      same pair repeats four times above them — `FacetDrawer.vue` /
      `CollectionFacetDrawer.vue` (the template differs in `grid-cols-3` against
      `grid-cols-4` and in the i18n prefix), `UnlockToolbar.vue` / `CollectionToolbar.vue`,
      `StateToggle.vue` / `CollectionStateToggle.vue`.
      Two consequences are already on the page: the names defend themselves with prefixes
      (`matchesFilter` against `matchesCollectionFilter`) because the two modules share a
      flat namespace, and `collectionFilter.ts` imports `OriginValue` **from
      `unlockFilter.ts`** — the Collection depends on Unlock's screen module for a value
      that belongs to neither screen, it belongs to the wire. B3 in `docs/BACKLOG.md` would
      write the third copy.
      **What:** `ui/src/lib/facets/` with one engine —
      `createFaceting<Row, Facet>({ order, values, options })` giving
      `{ matches, counts, activeCount }` — and `ui/src/components/facets/` with one drawer,
      one toolbar and one state toggle, each driven by a table. A screen keeps only what is
      genuinely its own: which facets, how to read a row's values, which options, which
      labels. `OriginValue` moves to a shared module beside the wire types. The drawer's
      column count comes from the number of columns instead of a literal typed twice.
      **Five files stop existing** — `collectionFilter.ts`, `collectionFilter.test.ts`,
      `CollectionFacetDrawer.vue`, `CollectionToolbar.vue`, `CollectionStateToggle.vue` —
      and their `unlock/` twins become the shared ones, under names that no longer say
      "unlock". About 250 lines.
      **Staying two on purpose:** `UnlockRow` / `CollectionRow` and `UnlockTable` /
      `CollectionTable`. They draw different columns; one component with a column table
      would be a worse file than the two it replaced.
      **Done when** no file under `screens/collection/` is a copy of one under
      `screens/unlock/`, adding a facet to one screen touches no file of the other, and
      B3's third list costs one spec object.

- [x] **N4. One diagnostics list, not four.** *Done 2026-09-13, `feature/ui-diagnostics`.*
      **All four stopped existing**, not three: `PlanAlerts.vue` went too, and its one
      button — the only alert that asks for something — is handed in through a slot, so the
      Plan keeps the action without keeping a component. `DiagnosticsList` draws; each
      screen keeps a `Record<kind, DiagnosticRow>`.
      **A diagnostic's scalar fields are now the translation's values**, which is what makes
      a kind one row: `{count}` is placed by the string instead of concatenated in front of
      it. That change found a real defect — the four count-bearing strings were sentence
      *fragments* written for `${d.count} ${t(...)}` and carried no placeholder at all, so
      passing the number as a value would have made it vanish with nothing failing. They are
      whole sentences now, and the word order around the number is the translation's
      business: the same argument as N2's `storeNewerSchema`.
      **Two invariants, and each caught something while being written.** Every key a table
      can produce exists in `en` **and** `it`. And every value handed to a translation is
      spent by it — an object or a list has no rendering a translator chose, so `reason` and
      `wanted` are dropped rather than passed unused, and the check is **per entry**, because
      the builder hands a diagnostic's values to the title and the body alike and it is
      enough that one of them places each.
      **Done when** — both met: a new diagnostic kind is one row in one table, and no screen
      owns a component whose job is drawing alerts (`find ui/src/screens -name '*Diagnostics.vue'
      -o -name '*Alerts.vue'` finds nothing).
      **Measured against this section's own rule, and it does not pass it.** 4 files and 303
      lines became **6 files and 278**: lines down 25, **files up 2**. The rule says
      unification closes on a file count going *down*, and here it cannot: one idea needs a
      spec, a component and one table per screen, which is the item's own prescription.
      The rule is a proxy for "did the duplication actually go", and it did — the four copies
      are deleted and `git` records no survivor. **The proxy disagrees with the thing it
      proxies, and the honest entry is this one rather than four tables merged into a file
      nobody wanted just to make a count fall.** N3 and N5 should be measured knowing that:
      their file counts really do fall, because what they delete is a *copy*, not a copy plus
      the machinery that replaced it.

- [x] **N5. The stores stop repeating themselves.** *Done 2026-09-13, `feature/ui-view-stores`.*
      `collection.ts`, `completion.ts` and `graph.ts` are **three lines of `stores/views.ts`**,
      and `LoadStatus` lives in `stores/loadStatus.ts`, which holds nothing else.
      **The factory was not enough on its own, which the item had not seen.**
      `defineViewStore` fits a store whose whole shape is the triad; three others carry more —
      the profile loads two things under one status, the queue clears its mutation state
      first, the wiki skips a read it has already made. They use the half underneath it,
      **`tracked(status, error, read)`**, so the `try` / `catch` is written once for all six
      rather than once for three.
      **One read keeps its own, and the code says why.** `wiki`'s `loadEntry` reports a
      failure *without ever claiming a success*: the status belongs to the index, and a page
      arriving must not mark the index `Ready`. `tracked` cannot express that shape, and
      forcing it would have been a behaviour change wearing a cleanup's clothes. That a
      page's failure lands on the index's `error` is inherited and left alone — changing it
      needs a decision, not a refactor.
      **The graph's two answers became one `view` object**, which is what they always were:
      one read, so `unlock` and `steps` cannot straddle a profile change. Call sites say
      `graph.view?.unlock` instead of `graph.unlock`, and two guards that checked both
      halves now check the one object.
      **Done when** — both met: no store writes that `try` / `catch` by hand, and
      `LoadStatus` is imported from a file that holds nothing else. **Three files stopped
      existing** and three arrived (`views.ts`, `tracked.ts`, `loadStatus.ts`), so the count
      is flat — for the reason N4 records: what is deleted here is a copy *plus* the
      machinery that replaces it.

- [x] **N6. `crates/app` goes back to being wiring.** *Done 2026-09-13, `feature/app-wiring`.*
      964 lines in one file became **eleven, none over 220**: `state.rs` for the six
      `OnceLock`s — the item said five, `SearchState` was not on the list — plus the two
      save reads they share, `icons.rs` for the `isaac://` protocol and its crop, and
      `commands/` with one file per screen family: profile, completion, wiki, graph, queue,
      plan. `lib.rs` keeps `run()` and nothing else.
      **`IpcError` had to move first, and that is what the item had not seen.** The tested
      functions return it, and it lived in the Tauri crate: they could not leave while the
      type they are about could not be imported. It is a wire type and `ipc` is the only
      contract, so that is where it belongs — and N7 will generate it from there.
      **The pure halves went to `store`, not to `ipc`.** `plan_parts`, `store_error` and
      `store_unavailable` all take a `StoreError` or a `GoalsRead`, and `ipc` cannot depend
      on `store` — the dependency runs the other way. They are not wiring either: each has a
      return value worth checking, which is the rule that says they may not stay in `app`.
      `crates/store/src/degrade.rs`, with the five tests that were `app`'s only
      `#[cfg(test)]` block.
      **`icon_url` stays in `app` on purpose**, against the item's list. `CLAUDE.md` already
      records why: on Windows the webview sees a rewritten `http://isaac.localhost` origin,
      and knowing that is the Tauri crate's job, not a pure crate's. It is wiring, not logic
      that escaped — and moving it because a list named it would have undone a decision the
      repo had already argued.
      **Closed against all three criteria, checked rather than assumed**: no `#[cfg(test)]`
      under `crates/app/src/`, largest file **220** lines, and `cargo test -p app` reports
      zero tests because there is nothing left in it to test.

- [x] **N7. `types.ts` generated — this is B2 of `docs/IMPROVEMENTS.md`.**
      *Done 2026-09-13, `feature/generated-contract`.*
      865 hand-written lines mirroring the `#[serde]` attributes, gone: `pnpm ipc:types` runs
      `crates/ipc/src/bin/ipc-types.rs` over an ordered list of the types that cross, then
      prettier. `scripts/check` regenerates into a scratch copy under `ui/` and fails on a
      difference. Spec `docs/superpowers/specs/2026-09-13-generated-contract-design.md`, plan
      `docs/superpowers/plans/archive/2026-09-13-generated-contract.md`.
      **The item said five foreign types; there are nine.** `wiki` contributes `SectionKind`,
      `CollectibleTemplate` and `Style` beside `Target`, and `core_save::marks::CharacterGroup`
      is the ninth. The list in B2 was written from a skim, the same way N2's was — the
      compiler found the rest in one pass, which is the argument for deriving on the real types
      rather than declaring them in the post-step.
      **Four defects the hand-written mirror was hiding**, and they are the item's whole
      justification rather than a bonus:
      - `SectionCount.kind` was `string` — the documented incident, closed by construction.
      - `GameView.edition` was `string` and `dlcs` was `string[]`. Two more of the same shape,
        never noticed because nothing compares them.
      - **`Infobox` had no `transformation` variant.** The Rust has had one since the
        transformations landed on 2026-09-13; `WikiInfobox.vue`'s switch was exhaustive only
        because the type lied, so a transformation page reached `assertNever`, which throws.
        The screen draws no card for that kind yet and says so in place — **B40** registers the
        rows.
      - `ProgressMark` was reachable only through `pub mod for_tests` while being a field of
        `SearchHit`. N1 had moved it there reading it as a test-only return type.
      **One decision the implementation had to take.** `discovery::Dlc` and `wiki::Dlc` are
      different enums with the same name — installed DLCs in `snake_case` against a content's
      origin in `camelCase` — and one file cannot declare both. The installed one is
      `InstalledDlc` in TypeScript through `#[ts(rename)]`, which does not touch the wire.
      **Two facts worth carrying forward.** A `///` on a wire type is UI source now: it becomes
      JSDoc and lives under `pnpm scan`, so `MissingReason` lost an arrow the app's font cannot
      draw. And `ts-rs` prints one permanent warning — it does not parse `#[serde(transparent)]`
      and says so on every build. It is left standing rather than silenced with
      `no-serde-warnings`: the attribute is a no-op on a newtype in serde too, the output is
      right, and the feature that hides this line would hide the next one as well.
      **Done when** — all met: `pnpm ipc:types` leaves `git diff` empty, a wire change with
      every Rust call site still compiling (`#[serde(rename = "bestiaryy")]`) fails both
      `cargo-test` and `ipc-types`, and `pnpm check` is green.

- [x] **N8. The save read once per screen, not twice.** *Closed 2026-09-14, both halves —
      `feature/one-graph-read` then `feature/save-state`.*
      **The two graph screens are one command.** `graph_views` returns the Unlock view and
      the steps together, so a screen load reads the profile **once** — and the "done when"
      below is answered by construction rather than by a counter: there is no second entry
      point to count, because `unlock` stopped being a command. The store's own comment —
      *"one read, because both answers belong to the same profile and asking twice could
      straddle a change"* — was a promise two commands could not keep: between them a save
      written mid-load made the steps describe a profile the list no longer showed.
      **And the profile is kept between commands**, in `ipc::SaveCache` — a cache and not a
      `OnceLock` for one reason: the game rewrites the save while the app is open, so a
      reader that remembers the first read shows a profile that no longer exists. Three
      rules, each with a test **shown able to fail**: the same profile the settings name (no
      name means the active one is whatever is on disk, which is the thing that changes),
      the modified time the cached read saw (which covers the file disappearing — no time is
      not the same time), and never a remembered failure. The time is read **after** the
      load, so a file written *during* the read is already stale rather than trusted as a
      value that saw half of each version. Completion, the Collection, `want` and the wiki's
      progress block stop paying for their own read of the same file in the same second.
      **The counter the item asked for is a call count over the policy**, not over a Tauri
      command: the wiring stays untested by the rule that put it in `crates/app`, and what
      was worth checking moved to the pure crate where a test can reach it.
      **Needs:** the game — its "done when" is a counter in a test showing the `.dat` opened
      once across Next steps and Unlock, and both commands go through the catalog, which on a
      machine without the game skips instead of counting. The free half (`next_steps` taking
      the `UnlockView` it filters) can be written anywhere; it cannot be believed anywhere.
      The vocabulary is the one at the top of `docs/BACKLOG.md`.
      `active_save()` does, on every command that needs the profile: `settings_file::load`
      (I/O), `discover()` (a walk of the Steam libraries), `fs::read` of the whole `.dat`,
      and a full parse. Nothing caches it, while the catalog, the graph, the resources, the
      mark frames and the search index all sit in a `OnceLock`. The worst case is
      measurable rather than theoretical: `next_steps` calls `unlock()` internally and
      `stores/graph.ts` asks for the two in one `Promise.all`, so **discover, parse, 642
      nodes and the evaluation all run twice for one screen load.**
      Two halves at very different prices, and they are not one task by accident:
      - the free half — `next_steps` receives the `UnlockView` it is a filter over instead
        of rebuilding it. Half the work of a screen load, no new state, nothing to
        invalidate.
      - the expensive half — a `SaveState` in `tauri::State`. **Not a blind `OnceLock`:**
        the `.dat` is rewritten while you play, so it invalidates on the file's mtime, and
        "no profile" is never cached — the rule `ResourcesState` already writes down for
        the game not being installed.
      **Done when** loading Next steps and Unlock opens the `.dat` **once**, asserted by a
      counter in a test and not by eye, and a save written while the app is open is seen by
      the next command.

---

## Open blockers

- [x] ~~**`Archive::open` reads the whole file into memory.**~~ **Resolved on 2026-09-06** (C1
      in `docs/IMPROVEMENTS.md`). `open` reads the header and the index and keeps the `File`
      open; a resource's bytes are read on demand, with positional reads
      (`seek_read` / `read_at`), because the `ResourceSet` is shared across commands and a
      shared file cursor would be a race. An entry's data upper bound is the next offset
      after it: the compressed length isn't stored anywhere in the format.
      Opening all eight archives (19,473 entries) went from ~1.3 GB to a **1.4 MB** peak, measured
      by a global allocator in `crates/unpack/tests/streaming.rs`; the spec's stated
      threshold is 64 MB. On the `app` side, `ResourcesState` opens the set once for all
      commands (`extraction_report`, `unlock`, `next_steps`, `plan`, `add_goal`,
      `remove_goal`).
- [x] ~~**`unlock` serializes 641 nodes with base64 icons inline in each row.**~~
      **Resolved on 2026-09-08** (C2 in `docs/IMPROVEMENTS.md`), and not the way that entry
      proposed. Rather than a second command and a cache written in the UI, the app
      registers a **URI scheme**: a row still carries `iconUrl`, but it is a short link
      (`isaac://achievement/19`) that an asynchronous handler serves from the
      `ResourceSet`, so the browser does the lazy loading, the caching and the
      de-duplication. Measured on the real profile: `unlock` **7 MB → 415 KB**, and
      `next_steps` — the opening screen — **124 KB → 3 KB**.
      The TypeScript type didn't change (`iconUrl` is still `string | null`), so this never
      blocked the frontend the way the entry assumed; and no path crosses the boundary,
      because the reference is keyed on ids. `ipc::IconRef` (round trip + `icon_source`, 5
      tests), the ceiling pinned in `crates/ipc/tests/unlock_size.rs`, the platform rewrite
      to `http://isaac.localhost/` confined to `crates/app`.
- [x] ~~**`unpack` implements only one of the three compression schemes.**~~ **Resolved on
      2026-09-03.** The byte at `0x07` in the ARCH000 header wasn't a version, as the
      spec assumed (`0x07 u8 version = 0x01`): it's an **`ArchiveCompressionMode`** and
      it selects the data's algorithm. It's now read from the file and dispatched on.

      | mode | value | archives | outcome |
      |---|---|---|---|
      | `Bogocrypt1` | 0 | `graphics.a`, `music.a` | ✅ implemented |
      | `LZW` | 1 | `config.a`, `fonts.a`, `animations.a` | ✅ already there |
      | `MiniZ` | 2 | `afterbirth.a`, `afterbirthp.a`, `repentance.a` | ✅ implemented |
      | `Bogocrypt2` | 5 | none here | not implemented, nothing to verify it against |

      *Why nobody noticed:* `samples/` contained `config.a` and `fonts.a`,
      two archives with the same mode — exactly the only one implemented. The suite was green on
      a sample that covered a third of the domain.
- [x] ~~**`repentance.a` is indexed but unreachable.**~~ **Resolved on 2026-09-03.**
      It wasn't an incomplete dictionary: **Repentance uses a different root**, `resources-dlc3/`
      instead of `resources/`. With that, 4,159 of 4,180 entries (99.5%) become
      reachable. Fixed by the test `repentance_resources_live_under_a_different_root`.

      > How this defect showed up is worth noting: the index only contains hashes, so
      > **a wrong path doesn't error, it goes silent**. `contains()` returned
      > `false` and it looked like the archive simply didn't have that file. Whenever a resource
      > is "missing", the first suspect should be the name, not the archive.
- [ ] **563 archive entries remain unnamed** (out of 19,473, i.e. 2.9%): 248 in
      `afterbirth.a`, 239 in `graphics.a`, the rest scattered. These are files none of our
      sources names. **They don't affect the product**: the catalog is covered at 100% (see below).
      *A way to unblock this, if ever needed:* a more complete path list, or generating candidates
      from recurring name patterns.
- [ ] **How much real data there is depends on the machine, and has to be re-checked on
      each one.** The project is developed from more than one PC; `samples/` is git-ignored
      and the game install isn't in the repo either, so neither travels. **Nothing in this
      section is a property of the project** — it's what one machine happened to have on
      one day, and a run that skips 60 tests here may skip none there.

      What to run before trusting a green suite, on whatever machine you're on:

      ```sh
      ls samples/                  # which saves, and from which eras (the date is the name)
      ls samples/packed/           # the junction to the game's resources\packed
      sh scripts/check             # the summary at the end counts and lists the skips
      ```

      The two things that switch large parts of the suite on and off:

      - **`samples/packed`**, a junction to the installed game's `resources\packed`.
        Without it every `unpack` test on a real archive skips, and with them every
        `catalog`, `ipc` and `graph` test that needs a built catalog (`build_or_skip`,
        `packed_dir`, `real_catalog`) — `ipc/graph_real.rs` and `catalog/tests/real_data.rs`
        among them. On a machine with the game this is one command to restore.
      - **the saves in `samples/`**, and above all *how many eras* they span. Comparison
        properties need two snapshots of the same profile; era-pinned counts need a file
        from that era. Both shortfalls are declared on stderr, never silent.

      *Observed on the machine used on 2026-09-08* (recorded so a future reading knows what
      the numbers of that day rest on, not as the project's state): no game installed —
      the only Steam library had no `appmanifest_250900.acf` and what remained under
      `steamapps\common\The Binding of Isaac Rebirth` was an orphan with no executable and
      no `resources\` — hence no `samples/packed`, and ~60 of ~81 skips. `samples/` held
      four saves over two eras: `20240118`, `20240305`, `20240606` (`rep_`, Repentance —
      638 achievements / 496 counters) and `20250112` (`rep+`, Repentance+ — 641 / 521),
      counts read with `od` on the headers. No save of the **642 / 523** era, which is why
      `each_era_declares_its_own_counts` carries that row as declared missing coverage —
      on a machine that has one, the row runs and nothing needs changing.
- [ ] **`samples/` never contained the M0 collection.** The 28 saves over 14 months used to
      decode the format live outside the repo, and the folder is git-ignored: whoever
      clones has none, and the suite has to stay green anyway. Two sources worth knowing
      about when stocking a machine, both free of the game itself:
      `Documents\…\Binding of Isaac Repentance\` keeps the dated backups the game writes on
      its own (14 snapshots of one `rep_` profile across Jan–Mar 2024 on the 2026-09-08
      machine, of which only 3 had been copied over), and Steam's
      `userdata\<id>\250900\remote\` holds the live profiles. A denser series costs nothing
      but copying, and every copied snapshot is one more comparison the properties can make.

## What only a machine with the game can answer

Collected here on 2026-09-09 because it had accumulated in five places. Nothing below is
blocked on thinking or on code — each one is an instrument that needs the game installed,
running, or both, and every one of them is cheap once you are at that machine. The
instrument they mostly share is the **matched window**: play a run, then compare the live
save against the dated backup the game wrote before it, and read which cells moved.

**This section is the `a measurement` bucket of `docs/BACKLOG.md`'s tags**, which since
2026-09-14 mark every open entry there with what it needs beyond a clone of the repo —
`nothing`, `a real save`, `the game`, `a measurement`. The vocabulary and the current counts
are at the top of that file. What lives here rather than there is what is an **instrument**
rather than an entry: no code to write, only a thing to go and read. The two that are both —
B9 and B20 — are entries in the backlog and appear here as the measurement they wait on.

- [x] **Whether the archive's runs agree with `STREAK_COUNTER [22]` and `DEATHS [10]`** —
      **answered on 2026-09-15**, by the owner playing one solo, non-Greed win: Eden, Mom,
      Mother and Delirium, ending on The Void. `STREAK_COUNTER` 2 to 3 for the one win the log
      folds to, `DEATHS` unmoved for the zero deaths it holds. Three tests in
      `crates/ipc/tests/runs_real.rs`, each mutated to check it can go red.
      **And the link the archive was missing came free**: the log's
      `unlock steam achievement '<id>'` and the save's achievement slot are the **same
      number** — two unlocks, two slots, the same two ids. The 2026-09-08 test says in as many
      words that the numbering was unknown, because that window holds two unlocks and no slot
      turning on.
      *The window is a day wide, not tight around the run* — the backup of the 14th against the
      live save of the 15th, with `live_probe` not running — and what licenses reading it as one
      run is the single `+1` on the streak, plus Steam's own launch log saying exactly one
      launch began inside it.
- [ ] **Section 8's cell 2** — **the section earned its name on 2026-09-15** and is
      `CutsceneCounters` now, not `Unknown8`: the log plays `playing cutscene 22 (The Void)`
      once and cell 22 rises by exactly one, which is the second point on the identity mapping
      after 19. The game's `Reading chunk 8 / Cutscene Counters` is finally agreeing with
      something instead of standing alone.
      **Cell 2 is still open, and both of the old hypotheses are gone.** It rose by one.
      *Cutscene 1 under an off-by-one* is refuted — identity at 19 and 22 cannot coexist with an
      offset at 1. *A count of launches* survives (exactly one launch began in the window) but
      is not established, because a cutscene 2 could have played in the launch whose log the
      game overwrote. And the sharpest fact against the simple reading: **cell 1 did not move**
      although the Intro, cutscene 1, did play.
      **Two minutes close it**: launch the game, reach the menu, quit without playing. If cell 2
      moves, it counts launches. Sections 5 and 9 keep their printed names and stay `Unknown`.
- [ ] **What the bestiary's four tallies count** (B9, structure closed 2026-09-09;
      **halved on 2026-09-12**). Section 10 holds four lists over the same entities — ids 4,
      2, 3, 1 — with a different number against each entity in each. One matched window on a
      Greed run split them in two: **tallies 1 and 2 moved** (sum +432 and +805, no new
      keys), **3 and 4 did not move at all**. A second window the same day — Judas, **solo**,
      44 minutes — split them again: 1 and 2 moved (+351, +258), **3 moved (+11)**, 4 still
      did not. So the four sort into three behaviours: 1 and 2 move constantly and in both
      modes; 3 moves solo but not in online co-op, like section 3 does; 4 has never been
      seen moving.
      **The hypothesis for 4, with its test**: it counts the times an entity *killed the
      player*. It fits every number — 156 records for 321 deaths, growing slowly and by
      whole new keys across the series (130/240 on 06-29, 142/277 on 08-05, 154/319 on
      09-08) — and above all it explains the two flat windows, because **both sessions had
      zero `Game Over` lines**. One deliberate death falsifies it or confirms it in two
      minutes: tally 4 has to gain exactly 1 on the killer's key. Until then it keeps the id
      the file gives it. Naming 1, 2 and 3 apart still needs the original instrument: kill a
      known enemy a known number of times and read which moves by how much.
      **A third flat window on 2026-09-15, and it is the best of the three**: a whole solo run
      to The Void, tallies 1, 2 and 3 all moving (+1105, +937, +30, and tally 3 gaining four
      new keys), tally 4 not moving a byte — with zero `Game Over` lines and `DEATHS` unmoved,
      so the hypothesis's own precondition was satisfied and the instrument had demonstrably
      spoken. Three for three. It is still not a confirmation, and saying otherwise is the trap
      this section exists to avoid: only a death can produce one.
- [ ] **What the bestiary's trailing word is.** One word after the last tally, in every
      save, growing 11,343 → 29,725 across the samples we hold, and 43,914 → **43,925** over
      the 2026-09-12 window. So it moves **+11 in one Greed run** — small, and not obviously
      proportional to the 805 the tallies gained, which is the useful part: it is not a
      fifth total. `Save::bestiary_tallies()` already hands it back as a value, so this is
      only a matter of watching it over a run whose rooms and floors are counted.
- [ ] **The 40 unknown cells in the completion matrix** — Mother and The Beast for The
      Forgotten and the 19. One run of Mother with a Tainted character closes the whole
      20 × 2 block, because the base indices are already pinned and only the evidence that
      those cells move is missing.
      **Measured on 2026-09-12, and the absence is now a finding rather than an assumption**:
      a walk of the whole dated series looking for *any* completion of Mother or The Beast by
      those 20 characters found none. The blocker is real, the series cannot close it, and it
      is what keeps 40 nodes of the unlock graph `Partial` instead of answered.
- [ ] **A save of the 642 / 523 era in `samples/`** — a copy out of
      `Steam\userdata\<id>\250900\remote\`, named `YYYYMMDD.rep+persistentgamedata1.dat`.
      Costs nothing but the copy, and it switches on the era row in
      `each_era_declares_its_own_counts` that today declares itself as missing coverage.
      More snapshots of one profile are worth more than more profiles: the comparison
      properties need two of the same.
- [ ] **The `samples/packed` junction** to the installed game's `resources\packed`. One
      command, and it switches ~60 skipped tests back on across `unpack`, `catalog`, `ipc`
      and `graph` — plus it is what `pnpm design:export` needs to regenerate the design
      package.

---

## To investigate

- [ ] **Backlog of registered, not-yet-started tasks: `docs/BACKLOG.md`** (2026-09-05).
      ~~B1 analysis on sources for item effects~~ **closed on 2026-09-05**: the
      source is wiki.gg (CC BY-SA 4.0, not the fandom copy), dataset in the build, report in
      `docs/superpowers/reports/2026-09-05-b1-sources-effects-report.md`. **Implementation closed
      the same day** (at night): crate `wiki`, tool `wiki-snapshot`, embedded dataset —
      report in `docs/superpowers/reports/2026-09-05-wiki-dataset-report.md`. All that's left is the
      design of the screen that shows the text. ~~B2 challenge rewards~~ **closed on
      2026-09-05** (three forms in the
      file, not one: see the entry); B3 lists and search after design; B4 is M2; **B8 is
      the spike M4 waits on** — what a real `log.txt` contains after an actual run
      (logged 2026-09-08).
- [x] ~~Red test: `stable_section_counts_match_the_format`.~~ **Resolved: false alarm.**
      The test is correct. The `live.*` fixture is meant to represent "a save from the current
      era"; a January 2025 profile had been put there, which reports 521 counters because
      it predates the patch that added two more. The 523 isn't a count hardcoded in the code
      but an expected value anchored to a fixture from a known era — legitimate. Sample renamed
      with its real date.
- [x] ~~The samples present aren't the ones the tests look for.~~ **Resolved.** Discovered that
      `Documents\My Games\Binding of Isaac Repentance\` contains **28 dated backups**, 14 of
      them from the same profile (`rep_` slot 1) between January and March 2024: a historical series
      the game creates on its own. `samples/` now contains four files — three snapshots of the same
      profile (Jan 18, Mar 5, Jun 6, 2024) plus a Repentance+ profile (Jan 12, 2025). `core-save`'s
      real-data tests were rewritten against these: 8 tests, all actually run.
- [x] ~~**`discovery` and the leftover game folder.**~~ **Closed on 2026-09-08** —
      `crates/discovery/tests/orphan_game.rs` builds the orphan wherever the suite runs,
      instead of relying on a machine that happens to have one. The registered behaviour
      was already right: no `appmanifest_250900.acf`, no installation. It's the *pair* that
      makes it mean something — the same folder with a manifest beside it **is** the
      installation, so Steam's record decides and not the folder's name.
      **Writing it turned up a defect the entry didn't suspect**, and the worse of the two
      directions: with a manifest present and the folder it names gone (an interrupted
      uninstall, a library on a drive that isn't plugged in), `find_game` returned
      `Some(dir)` for a directory that doesn't exist and **no diagnostic at all**.
      Downstream that reads as a failed extraction instead of a game that isn't there —
      false data with a confident face. The scan now skips such a library and keeps going,
      so a stale manifest on one drive can't shadow the game on another. `game_dir`, the
      manual override, is untouched on purpose: a path the user chose comes back even when
      it's empty, so the app can say so.
- [x] ~~**Real-data tests that skip silently.**~~ Addressed in `c9faa1e`
      (non-silent skip). The suite is now at **94 tests, 0 failed**.
- [ ] **Tests that pass on an unrepresentative sample.** A more insidious variant of the
      point above, surfaced on 2026-09-03: `unpack`'s "real archive" tests genuinely
      run, but all of them on `config.a` — 24 entries, the only archive the decompressor
      handled at the time. No skip, nothing red, and the module's main function doesn't work.
      To be treated as a rule: **a real-data test must declare which portion of the
      real domain it runs on.**
- [x] ~~**Two real `ipc` tests skip for want of a sample.**~~ **Closed on 2026-09-08**, in
      the opposite direction to the one the entry expected: rather than hunt for the named
      snapshots, the tests were pointed at eras that are actually stocked. `cross_check.rs`
      runs again — 321 readable cells agreeing with `reference/isaac_save.py` — and its
      pinned numbers now sit next to the file they were measured on, so the pairing can't
      drift again. `ipc/real_saves.rs` needs no sample of its own. Like every real-data
      test it still declares and skips where its sample is absent; the point is that it
      no longer names a file nobody has.
- [x] ~~**Rename `samples/live.rep+persistentgamedata1.dat` with its date.**~~ **Moot as of
      2026-09-08**: the file doesn't exist and no save of that era does. The two tests that
      named it, `ipc/graph_real.rs` and `catalog/tests/real_data.rs`, both skip on
      `samples/packed` first, so fixing the name there would be an unverifiable edit — left
      alone deliberately, and it comes back with the game. The rule the entry was defending
      is now enforced where it belongs: `core-save`'s `ERAS` table pins one count per file
      per era, and `cross_check.rs` states the pairing on the constant itself.

      *The original entry:* rename it with its date (2026-08-31)
      like the other samples: the pinned numbers (379 completed, 169 of 171) are a fixture from a known
      era and a file called "live" invites overwriting it. The tests that reference it by name need
      updating too: `crates/ipc/tests/graph_real.rs` and
      `crates/catalog/tests/real_data.rs`.

---

## Session log

### 2026-09-15 (night, later) — the window that finally held its run, and an eleventh chunk

`feature/window-20260915`. **The owner played one solo, non-Greed run** — Eden, Mom, Mother and
Delirium, ending on The Void — which is the measurement three open items had been waiting for since
2026-09-12. Report `docs/superpowers/reports/2026-09-15-window-and-the-eleventh-chunk.md`.

**M4's last open item is closed**: `STREAK_COUNTER` +1 for the one win the log folds to, `DEATHS`
unmoved for its zero deaths. And the link the archive was missing came free — **the log's
`unlock steam achievement '<id>'` is the save's achievement slot**, two unlocks and the same two
slots, which the 2026-09-08 test could only say was unknown.

**Section 8 earned its name.** The game has always printed `Reading chunk 8 / Cutscene Counters`
and this repo has always refused it, because sections 3 and 6 carried wrong labels for months on
exactly that evidence. Cell 22 rising once for the one `playing cutscene 22` is the measurement.
`Unknown8` is `CutsceneCounters` now; 5 and 9 keep their printed names and stay `Unknown`.

**Cell 2 killed both of its hypotheses and is still open.** *Off-by-one on cutscene 1* is refuted —
identity at 19 and 22 cannot coexist with an offset at 1 — and *a count of launches* survives
without being established. The fact that settles neither and sharpens both: **cell 1 did not move
while the Intro played.**

**Tally 4: a third flat window and the best of the three.** Tallies 1, 2 and 3 all moved (3 gaining
four new keys) and 4 did not, with zero deaths — the instrument spoke and the silence had the
hypothesis's precondition behind it. Three for three, and still not a confirmation.

**The claim that started the sweep was mine and it was too wide.** Asked whether the game keeps a
run history anywhere, an answer was given from two folders in the tone of one given from all of
them. A full read-only sweep upheld it — no per-run record for solo play — and turned up three
sources this project did not know it had: **Steam's launch log**, which is the wall clock
`log.txt` has never had and which was used the same hour to prove one launch in the window;
**Steam's achievement-unlock timestamps**; and **`savedatapath.txt`**, where the game writes down
the save folder `discovery` reconstructs by search. **B55**, **B56**, **B57**.

**And the game names eleven chunks where we parse ten sections** — the bestiary is its chunk *11*,
and its chunk 10 is *Special Seed Counters*, a name this repo has never had. Recorded against B9 as
a lead with its two coincidences (the bestiary payload's constant `words[20] = 11`, and section
10's header describing nothing in the layout we decoded) and explicitly not as a finding.

**One question is the owner's** and is left open rather than guessed: Mother's cell for Eden went
0 to **3**, both bits from a single kill, and whether that measures bit 1 outside Greed depends on
whether "in normal" meant *not Greed* or *Normal difficulty, not Hard*.

### 2026-09-15 (night) — the infobox was the wrong half

**B53 closed as not worth a shape, B54 opened.** No code: the closure is a reading, and it is the
kind this list exists to record.

B53 asked whether an entry should carry a second, subordinate infobox. Reading where the three
`{{infobox monster}}` actually sit answers it without the question: Blood Puppy's is at the **foot
of the page after `{{nav}}`, marked `hidden = yes`** — the wiki does not draw it — and the other two
sit at the head of `== Friendly Charger ==` and `== Dark Esau ==`, **level-2 sections with
unrecognized titles, discarded whole**. Rescuing the box would put a name, an id and a one-line
`behavior` on screen for a subject whose text the app threw away. A card with no page behind it is
worse than no card.

**What the three were pointing at is a section.** `discardedSections` reads 2267 over 63 titles,
and as one number it hides its own shape: **2213 are six titles dropped on purpose** — Trivia 889,
in-game footage 840 over three spellings, Gallery 359, References 102, Requirements 15, Audio 7 —
and the other **54 occurrences are 54 distinct titles, each seen exactly once**. A title seen once
is not a category declined, it is one page's heading with its whole section gone: a second subject
(`Dark Esau`, 28 lines of behaviour), a near-miss spelling (`Items Interactions` beside the
accepted `Item Interactions`), or something genuinely else (`Algorithm`, `Modifiers`, `Drops`).
That is B54, and its cheap half is deliberately **not** done by reflex — `Good Items` and
`Bad items` are one page's editorial lists, not `Notes`.

**The counter could not have said any of this**: 54 against 2267 reads as rounding, and the
diagnostic was built to show a parser losing ground, which is exactly what a heading seen once does
not look like.

### 2026-09-15 (evening, after) — the half of B48 that was reported and not fixed

`fix/empty-lists`, cut from `develop`. `scripts/check` green.

**In the Collection the state was real, not theoretical**, which is the finding. B48 had reported
the shape as "still live in `CollectionScreen` and `UnlockScreen`" without saying whether anybody
could reach it; a machine **without the game** reaches it on the first screen it opens — the view
answers `items: []` with a `noCatalog` diagnostic, and the list under that banner said *"Nessun
oggetto con questi filtri."* with a button to clear them. In **Unlock** it is not reachable:
`withoutCatalog` keeps every node and counts it as unread, so that half is a guard and is written
down as one.

`emptyList` moved out of `lib/wiki/` and takes its two message keys, so the decision stays in one
tested function and each screen names its own sentences. Its second argument stopped being the
query and became **whether anything is filtering**, because here a reset clears the facets too —
and `isFiltering` counts the picks a screen *opens* on, since a reset undoes those as well.

**Seen in a window**, which this repo counts as the other half of a closure and which B48 could
not do: port 1420 was free, so `pnpm ui:dev` evicted nobody. On `?catalog=none` the Collection's
opening filter keeps the button (it clears the two chips the toolbar shows) and the sentence is
right; cleared, the sentence stays and the button is gone. The wiki's empty category still says it
is empty, with no reset, after the move.

**The orphan the `Don't` list warns about is real**: stopping the task that ran `pnpm ui:dev` left
vite holding 1420 as a grandchild under two `cmd.exe`. It was checked by command line before being
stopped, the way `dev-reset.mjs` does it.

### 2026-09-15 (evening) — a dlc code is a range, and `n` means *not in*

`feature/dlc-ranges`, cut from `develop`. **B52 closed**, B53 opened. Report
`docs/superpowers/reports/2026-09-15-dlc-ranges-report.md`. `scripts/check` all green, 7 skips,
1306 real files touched.

**One `?action=raw` answered more than the entry asked.** B52 had it right that `n` modifies the
code beside it and right to refuse to guess which way — "new in" and "not in" have the same shape
and opposite consequences. `Template:Dlc/format` names every code in prose (row 7 is `nr`,
*Removed in Repentance*, `alt=(except in Repentance and Repentance+)`) and `Template:Dlcset` turns
out to be **the whole dictionary**: a `#switch` from thirty codes to a five-bit mask, stating its
own bit order in a comment. Nothing had to be derived, and a summarizer was not used — the Floor
session of the same day had already measured that one returns a quotation cut in half.

**A code is a run of transitions, not a set**, and both readers of one were wrong.
`unknownDlcCodes` **1832 → 0**: all seventeen codes the corpus uses are in the thirty. The
**infobox** was the quiet half — the same splitter read `dlc =`, so `r` lost Repentance+ on 531
pages, `a+` lost three editions on 292, `a` four on 254, **1078 of 1083 too narrow**, and Tonsil's
`a+nr` gained Rebirth and Repentance outright. An entry declaring fewer editions than it has looks
exactly like an entry.

**Abyss was never a contradiction**, which is the part B52 could not see: the wiki intersects a
span with its page's range before drawing an icon, so `nr+` on an item that exists from Repentance
names Repentance — *removed in Repentance+*. 847 of 4831 spans narrow. The context is the
**first** infobox's, because `{{page dlc}}` carries an `{{assert once}}`, and **Ultra Greed is the
page that said so**: two bosses, `a` then `a+`, one set of sections. Read as "each entry's own" it
threw a note away — 1 span in 4831, caught by the new counter on its first run, and 0 after.

**Two things found on the way, both about counters that were not counting.**
`Diagnostics::merge` had been dropping `unknown_infoboxes` since the day B45 added it so a page
with no kind would be *loud*; the dataset shipped `{}`. It destructures with no `..` now, and
reports `infobox monster: 3` — **B53**. And `crates/ipc/examples/dlc_mask.rs` had this same
question open since 2026-09-13: Blue Cap sets the Rebirth bit because its mask is **31**, "no
range declared", not "exists in Rebirth". The Cargo integer *is* `{{dlcset}}`'s output, so 720
live rows now answer for the transcribed switch; mutating one row turns that test red on 173.

**The transcription's own guard found a mistake, in the right half.** Thirty rows copied by hand
are where a typo hides, so a test re-derives each from the transitions its code spells. `ana+`
disagreed on the first run: the state before the first transition is the *opposite* of that
transition, not "present". The table was right and the explanation was wrong.

**Left open on purpose**: `editionLabel` now renders `r` as "Repentance · Repentance+" where the
wiki draws one icon reading *Added in Repentance*. Faithful, longer, and a question about what a
reader wants — it wants a window.

### 2026-09-15 (later) — the `Block` variant that was never needed, and the two losses found underneath it

Three branches in parallel on a machine with **no game and no window** — port 1420 was held by
another session's vite, and `pnpm dev` would have evicted it in silence, so nothing here was seen
drawn. Said plainly because this repo's own backlog counts *"not seen in a real Tauri window"* as
half a closure.

**B49 closed, and the finding is that it needed no contract change.** The entry had carried since
2026-09-08 the claim that saying *"a template wrapping blocks"* needs a `Block` variant — the
sentence that made its second half a design decision and kept it waiting for one. A census of
every span in `dataset/raw/` that opens on one line and closes on another refuted it: seventeen
spans, four families, each with a shape the contract already has. The two `X synergy` templates
open **on a list item** and hold nothing but `**` lines, which is `ListItem { inline, children }`;
all nine multi-line `{{bug|…}}` sit under `== Bugs ==`, which is `SectionKind::Bugs`, and the
single-line case had been dropped inline since `CONTENT_WRAPPERS` existed; `scroll box` is
`column list`'s family and was missing from the list because the families were known from the
offenders they left, which is a different list. Raw template syntax **35 → 9**, and the 9 are
genuine text.

**Two counts in that entry were wrong and could not have been noticed.** Multi-line `{{bug|…}}`
is **9 spans, not 4** — 4 was the count of the offenders they left — and the genuine remainder is
**9, not 8**: the 8 was measured on 2026-09-08, a formula arrived after it, and an assertion
pinned at `<= 35` cannot see its own remainder drift.

**The rule the fix needed and the spec did not know**: the pre-pass may touch only a template
that **spans lines**. 538 of the 547 `{{bug|…}}` close on the line they opened on, most inside a
list item, and moving one of those onto a line of its own cuts the item in two — a pass that
repairs one family by breaking five hundred. The fence is
`a_wrapper_that_closes_on_its_own_line_is_left_where_it_is`, the only one of the four new tests
that was green before the code was written.

**Then reading one `dlc=` opened something bigger.** The arm for `{{bug|…}}` never read a named
parameter, so 204 of 547 showed a defect of one edition to every reader. Going to fix that
surfaced the real number: this parser understands **2434 of the 4168 `{{dlc|…}}`** uses, and each
of the other **1734** opened an `Inline::Edition` with an empty `only` — a span declaring itself
valid in no edition at all. **1690 of them were in the shipped dataset, one edition node in
three**, and the app draws no badge for an empty `only`, so the reader was told nothing and
nothing recorded that a code had been dropped. `Out::close` now unwraps such a frame instead of
emitting it and the code is counted: `meta.diagnostics.unknownDlcCodes` carries **1832**, `nr`
1190 of them. **0 empty edition nodes remain.**

**`Dlc::parse_codes` was deliberately not reused**, and that is the measurement worth keeping. It
splits the *infobox* parameter, where concatenated codes are a set of editions, and would read
`nr` as Rebirth + Repentance. The corpus refuses: every code appears both bare and `n`-prefixed
(`r` 1687 / `nr` 1155, `r+` 446 / `nr+` 207, `a+` 165 / `na+` 149), a bare `n` appears **zero**
times in 4168 uses, and **Abyss** — an item that exists only in Repentance, `dlc = r` in its own
infobox — carries a line marked `{{dlc|nr+}}`, which as a set would be valid in an edition where
the item does not exist. So `n` modifies the code beside it and what it means is unmeasured;
"new in" and "not in" have the same shape and opposite consequences. **B52** carries the
measurement and the single query that answers it.

**And the same arm was losing the other book.** The item is *The* Book of Belial, with the
article; the arm resolved `Book of Belial`, which is not an item, so **33 uses** dropped the
reference and the `": "` that introduces the description — while the Virtues arm beside it worked,
which is why nobody saw it. References in the dataset **27 → 60**. The 34th occurrence of the
string is a different template inside a section the build discards, and `discardedSections`
records it.

**B48 closed on its own branch** (`fix/empty-category`): an empty category now says it is empty
and offers no button that undoes nothing, while a search that genuinely found nothing keeps both.
Five tests on a pure `emptyList(total, query)`, and the two halves are gated separately on purpose
— the sentence follows the total, the button follows the query. It reported that **the same shape
is still live in `CollectionScreen` and `UnlockScreen`**, where a reset is offered even when the
unfiltered list is empty.

**B47's analysis half was measured** against the committed data, and it is sharper than the entry
hoped: `Infobox stage` answers **1552 dead reference tokens with 27 pages** — the best ratio in
the repo by an order of magnitude — `Infobox entity` **3801 occurrences**, `Infobox pickup` with
`card` and `rune` as one `PageKind` **675**, while `Infobox grid entity` has zero transclusions
and can be declined permanently. Two things the entry did not expect: enumerating **all seven**
would take only **6 of the 49** `Target::Concept` targets out of "a page we do not have", and
**none of the 13** references the graph cannot interpret is behind any of them — they are words a
sentence used, not things with a page. **Rooms are covered by none of the seven** (1464
occurrences, 37 names), so "stages, rooms and concepts have no page" would become half true and
wants its own entry. The decision itself is the owner's and is not taken here.

### 2026-09-15 — Floor: the grid you paint, and the rules that light it

`feature/floor-grid`, cut from `develop`, F1 executed end to end. Nine commits. Rust **929
passed, 0 failed, 129 skips and 79 `sample:` lines — the same as `develop`'s**; frontend 499
tests over 71 files; typecheck, lint, `format:check` and `scan` green with no new exemption.

**The research pass is the finding.** Task 1 was run against `?action=raw` rather than a
summarizer, and that alone changed the rules file: `Super_Secret_Room` and `Ultra_Secret_Room`
turned out to be two `#REDIRECT` stubs of 44 bytes (so the plan's single URL was right, not
lazy), two rules the plan's draft did not have came out of the full sentences, and a
`WebFetch` on the same page had already handed back one quotation cut in half at the comma.
For a file whose only purpose is to be citable, that is the whole failure.

**The rule that would not fit forced the only real design change.** "2 neighbor locations are
rare but possible, **even when** there are locations with 3+" and "1 neighbor locations can
**only** happen **if** there are no valid 3+ locations" are not the same shape, and encoding
both as ranks would assert what the second denies. Hence `NeighbourCountFallback`, carrying
the `3` the sentence states — and a third solver phase, because *valid* means "survived the
narrowing", which is not knowable in the pass that proposes. The test for it was **mutated
against a solver that judges too early**: exactly one test went red, eleven stayed green.

**Three token families the plan assumed and this repo does not have**, and the repo won each
time: one theme rather than light/dark, `--spacing-*` rather than `--size-*`, and no
`--color-chart-*` — so the three ranks got a family of their own, deliberately outside the
brief's state colours, since none of done / unlockable now / blocked / unreadable / unexpected
/ challenge means "a cited rule allows a secret room here". `ring-2` is in no file in this
repo, so a candidate is drawn by its fill.

**Two traps in how the work was verified, both worth more than the feature.** `samples/` is
git-ignored, so a second worktree does not get it: the first suite run here reported 929 passed
with **zero** `sample:` lines and nothing red — a green suite that verified nothing, which is
D3's shape exactly. And counting skips by hand from `--nocapture` output is unreliable, because
the harness's stdout and `test-support`'s stderr interleave; three runs read 121, 118 and 168
where the truth was 129, 129 and 175. `scripts/check` never had that problem — it reads
`ISAACDOME_TEST_DECLARATIONS` — and the first of the two is now a line in `CLAUDE.md`'s
**Don't** list.

**Three sessions shared one working directory**, and seven commits in, a third one ran
`git checkout -b` in it: HEAD left `feature/floor-grid` and every tracked file reverted to
`develop`. Nothing was lost — the commits were on origin — but a branch is a variable shared
between sessions when `git worktree list` has one row, and a clean `git status` says only that
your own work is committed. This branch moved to `.claude/worktrees/floor`, hidden through
`.git/info/exclude` so the repository carries nothing.

**The routing was split with the session doing the Tool refactor**, along the line that lets
both branches compile: they own `RouteName.Floor` and the `routes.floor` key their `routeTitle`
cannot typecheck without, this branch owns `[RouteName.Floor]: FloorScreen` and the `floor`
message block. **Their refactor deleted a patch this plan asked for**: with Floor under an
origin whose definition is "does not read the `.dat`", `needsProfile` derives from the origin
with no exception and no comment explaining one.

### 2026-09-15 — a third section, and the exception that was pointing at it

`feature/nav-tool`, cut from `develop` **into a worktree of its own** — three sessions were
sharing this checkout at the time, which is the second half of the entry.

The request was a reordering: Progress before the Wiki, with a **Tool** section between them
holding Live. What made it more than a reordering is what Tool had to be to deserve being a
section at all. `DESIGN-BRIEF.md` §4 has always said a section is a **precondition**, not a
folder, and the two-section table was built on that. The third precondition was already
there, unnamed.

- [x] **The exception was the evidence.** Floor's F1 plan (Task 8 Step 7) had to write
      `needsProfile: routeOrigin[name] === TabOrigin.Progress && name !== RouteName.Floor`,
      and said plainly why: gating Floor behind a profile *"is wrong for this screen"*. An
      exception carved into a derivation reports that the derivation's input is missing a
      case. The input was missing a section — so the fix deletes the exception instead of
      generalising it, and `routes.ts` keeps the line it already had, now with nothing
      carved out of it.
- [x] **Live and Runs were gated for no reason, and both said so in their own source.**
      `live` in `crates/app/src/commands/runs.rs` already returns `LiveGraph::NoProfile` and
      carries the comment *"a missing profile and a missing game are two different sentences,
      and both leave the run on screen"* — a branch `ProgressGate` made unreachable from the
      UI. `RunsScreen.vue` carries *"the archive is not a view of the profile: it exists
      without one."* The `runs` command takes neither the graph nor the profile. Both moved
      to Tool; the split is exact — the five that stay in Progress are the five that open the
      `.dat`, with no judgment call at any row.
- [x] **Floor's route ships here, its screen does not.** F1 was mid-execution in another
      session, so `RouteName.Floor`, its path, title, origin, icon, sidebar entry and the
      `routes.floor` key land in this branch and the `screens` record entry stays with F1.
      Until then Floor renders `PlaceholderScreen`, like every screen not yet built. Agreed
      between the two sessions before either wrote a line, so the shared files have one author
      each.
- [x] **Saved tabs survived without a migration**, and that was checked rather than hoped: a
      tab persists a `TabLocation` — a route *name* and its query — never a path, so
      `/progress/live` becoming `/tool/live` reaches nothing in `store`'s migration 3.
- [x] **The test that counted became a test that asks.** `sectionNav.test.ts` had
      `lists the seven Progress screens`, a count that would have gone green on a wrong answer
      and dies at the next screen either way. It is gone. In its place: every section lists
      every route of its origin in its sidebar — generalised from the Settings-only version
      that already existed — plus the two origin sets named, so adding a screen cannot compile
      without someone answering whether it reads the save. `pnpm ui:test` 486 tests over 69
      files.

**The worktree is the entry's other half.** `git worktree list` gave one row for this repo,
so the branch was a variable shared by three sessions: one executing Floor's F1, one rewriting
the Progress gate into four states, and this one. A `git checkout -b` by any of them moved the
tree under the other two, and it happened once — attributed to the wrong session at first, and
settled by the reflog plus the content of the files left behind rather than by anyone's memory
of what they had run. Nobody lost work, because everything was committed. **The lesson is not
"commit often", it is that a checkout is a write to shared state that leaves no author.** All
three sessions now hold separate worktrees under `.claude/worktrees/`.

### 2026-09-14 (last, evening) — three small entries at once, and the one nobody was looking for

`feature/small-three`, cut from `develop`, on the machine **with** the game and the full sample
series. Three backlog entries with disjoint file sets, run in parallel and committed one per
entry: **B40** (the transformation's infobox card), **B43** (the virtualized scroll box and its
two tokens), **B42** (the `player` Cargo table read by nothing). `pnpm ui:test` 461 tests over
67 files, `cargo test -p wiki` 131, scan / typecheck / lint / format green.

**B40: measuring the dataset before drawing changed what to draw.** The entry asks for three rows
and warns only about `requires` defaulting to "3". On the sixteen pages, `requires` is `null` on
**one** and `target` is empty on **fourteen** — and `InfoboxRow` draws "nessuno" for an empty
value, which is true of a character with no starting items and **a claim nobody measured** about a
transformation. A row is drawn only where the page filled it; Adult, which filled none of the
three, carries no card at all — exactly what the suppressed variant's own comment said an empty
card would do. The decision is a pure function with its test, because **nothing in `ui/` mounts a
component**: no `@vue/test-utils`, no DOM environment, so a rule left in a template is a rule no
test can see.

**B43 closed both halves**, as the entry demands: `VirtualRows` owns the scroll box, the total
height and the window, the four screens lose 26 lines each, the columns stay two files. The token
rename was checked **in the built CSS** and not in the source — a utility nothing references
generates nothing. One thing the extraction had dropped was put back: the comment saying why the
row count is a getter, without which the reason a new filter reaches the virtualizer disappears.

**B42 is half closed, and the half that ran found something else.** `player.parent` and the
`parent` each character page states in its own infobox are two independent statements of one
relation, never compared: **32 named forms carry both and all 32 agree**, now guarded by a test
with a floor on what it compared. Matching them by `player`'s own `id` column was tried first and
is a trap — that column is as unreliable as the infoboxes' (`Isaac` 14, `Magdalene` 2), so an id
match *agreed* by accident on names it was never comparing. The check enters through `for_tests`:
it asks a question about the data we ship, and no command of the app asks it.

**The eight rows it could not compare were not eight facts but one**, and it is now **B45**:
`Jacob & Esau`, `The Forgotten`, `Tainted Forgotten` and `Tainted Lazarus` **have no page in the
snapshot** — 30 files under `dataset/raw/pages/character/`, no entry in `index.json`, while
`player`, a Cargo table rather than a page fetch, knows all four. They resolve by id through
`dataset/corrections.json`, so nothing ever complained; `requirements.json` holds **47 references**
to them (17 The Forgotten, 15 Jacob & Esau, 8 Tainted Forgotten, 7 Tainted Lazarus), every one a
requirement a screen draws with a target whose page cannot be opened. The fetch reports no error
because it enumerates `embeddedin` over `Template:Infobox character`: a page that states its
character another way is not *missed* by that query, it is **not in it**. What B45 asks for is the
guard as much as the four pages — the names the repo knows, counted against the pages fetched.

**The wiki was then asked, and the hypothesis is a measurement.** All four pages transclude
`Template:Infobox characters` — **plural** — which holds two forms in one block, the second's
parameters suffixed ` 2` (`name 2`, `health 2`, `collectibles 2`). They are exactly the four pages
that carry **two characters**, and 4 × 2 is the 8 `player` rows the cross-check could not compare.
Nothing is missing from the wiki: one template name is missing from our query, and the second half
is a shape the parser has never seen — where a page holds two forms today (Judas, Black Judas)
they are two separate blocks, which `extract_infoboxes` already handles.

**`stage.json` is the open half of B42 and stays open on purpose**: unlike `player` it has no
second source to be checked against, so there is no reader for it to earn, and dropping a
committed artefact is the owner's call.

**Then the window, and it answered one half and refused the other.** B43's four screens were
driven in a real browser over the dev fixtures: Unlock, the Collection, a wiki category list and
the search results all resolve `max-height: 560px` — the renamed token reaching the DOM, not just
the built CSS — hold **22 rows** of a 641-row list, and redraw a different window on scroll. B43's
"Closes when" is satisfied in full, which no test in this repo could have said.

**B40's card could not be looked at, and finding out why is the evening's real result.** The dev
fixtures carry no transformation page, so the app itself was built and run — and the page cannot
be opened there either. `pageKey` and `categoryOf` in `ui/src/lib/wiki/` still answer `null` for
a transformation, so `pageLocation` is `null` and **no route in the app reaches one**; the wiki's
six categories do not include them. The premise was true when it was written — the wiki-search
spec of 2026-09-12 says those kinds have no page — and **the transformations sub-project of
2026-09-13 made it false**: `Dataset::entry` answers `Target::Transformation` now. Rust answers,
the frontend never asks, and the `assertNever` guarding those switches cannot see it because the
variant *is* handled, handled as nothing. Registered as **B46**. The card merged this afternoon is
finished and unreachable.

**What this evening did not do:** see the card. It exists, it is tested, and B46 is what stands
between it and a screen.

**Then B45 closed the same evening it was opened**, on `feature/plural-characters`, with the
owner's go-ahead for the refetch. A kind names its **templates**, plural; `{{infobox characters}}`
is split where it is extracted into the two ordinary character infoboxes it stands for, following
the template's own source rather than a guess — `dlc`, `description`, `unlocked by` and `hidden`
are shared, everything else including `id` belongs to one form, because a second form inheriting
`id` would take the first one's identity and both would key the same entry. **34 character pages
instead of 30, 40 forms instead of 32**, and the eight that had no page are pages.

**Both guards, which is the half the entry called worth more than the four pages.** Every
character the repo names has to have a page; every Cargo row with an id has to have one too —
**1610 rows** checked across items, trinkets, achievements, challenges and transformations, with
a floor just under that so a table which stopped being read cannot make the test pass by checking
nothing. And the mechanism that hid it is counted: an infobox whose template name is in no kind
was skipped in silence, so a page could be downloaded and parsed into **zero entries** without a
word. `Diagnostics::unknown_infoboxes`.

**Three expectations moved, and each was attributed rather than absorbed** — the rule this repo
gives itself when a pinned number grows. Characters 32 → 40. Raw template syntax 83 → **87**,
which is **two** nodes and not four: Tainted Lazarus's page carries two multi-line bug templates
and now yields two entries that each carry the page's sections, exactly as Judas and Black Judas
already did; the other three new pages contribute none, measured by attributing every offender to
its entry. And the graph's era, which moved **without a single requirement changing**: those 47
references were resolving by id through `dataset/corrections.json` all along. The graph was right
while the pages were missing, which is precisely why nothing ever complained.

**What the pages cannot say.** `{{infobox characters}}` has no `parent` parameter, so the second
form of a two-character page cannot state what `player.json` knows. The cross-check learned a
third answer — silence, reported apart from disagreement — with the four names pinned in page
order, so a fifth one, a page that *could* state its parent and stopped, goes red instead of
joining a tolerated category.

**And the query that explained B45 opened B47**: seven more `Infobox` templates the fetch does not
enumerate, **591 pages** behind them — 247 entities, 126 monsters, 97 pickups, 66 cards, 28 runes,
27 stages. It corrects a sentence this repo says often: *"that kind has no page"* is, for a part
of them, a fact about our fetch and not about the wiki. Whether any of it belongs in a dataset
that ships inside the binary is a product decision, which is why it is an entry and not a task.

**Then B46 closed too**, on `feature/transformation-pages`, and with it B40's card is reachable.
`pageKey` writes `transformation:1`, `categoryOf` answers a seventh category, and the IPC index
carries the sixteen pages — **it was listing six kinds of seven**, and the test that checks the
index against the dataset did not catch it because *its own sum left out the same kind*. It sums
every count now: a count missing there is a kind the index may quietly stop carrying.

**The design call the entry named answered itself.** `pageLocation` needs a category as much as a
key, so the alternative to a seventh category was leaving the pages shut — the mechanism decided,
not the taste.

**And opening them made them askable, which they are not.** `wantable` and `wantLocation` derived
from `pageKey` deliberately: "has a wiki page" and "the graph can grant it" were the same set of
kinds, and the comment said a second list would be a second answer to one question. B46 ended that
coincidence — nothing unlocks a transformation, you collect three items — so `canBeWanted` is its
own switch, where a `Target` variant added later breaks the build. **The test that caught it was
already written and already right**: it lists a transformation among the hits `wantable` must
drop, and went red the moment the key appeared. `want_view` would have answered `nothingUnlocks`:
true, and a question the app should never have offered.

**The window earned its keep twice more**: the wiki's intro sentence lists what the copy holds and
stopped at achievements, and an empty category draws *"nessuna pagina con questo nome"* with a
reset button when nothing was searched — the Collection's own correction, one screen over, now
**B48**. Neither is visible to any test in this repo.

**Then the owner opened the app and read the screen, and one message was worth more than the
evening's own checking.** Six observations, four of them defects, and the two that were mine to
fix went in the same hour:

- **the global search did not know the transformations.** The wiki index learned about the
  sixteen pages and the search index did not — six kinds of seven, the same arithmetic one layer
  over. A page that exists and cannot be found reads exactly like a page that does not exist,
  which is how it was noticed: typing a name and getting nothing.
- **an achievement's drawing overlapped the want suggestions.** `PixelSprite` draws a picture at
  the size the file is and says so in its own comment — *the size comes from the parent* — and
  `WantBar` gave it none. Item icons are small and looked right by luck; achievement sheets are
  not. It took a transformation's name to put achievements next to items in that list. It
  reproduces on the fixtures, so the fix was seen in a browser and not argued.

The other three are entries. **B49**: Beelzebub's page prints `{{column list | width = 15em |
content = …}}` where the enemies should be — the block-level template defect the suite has counted
for a week, which until now lived in item sections below the fold and is on a short page's first
screen. **B50**: a transformation has no picture anywhere, and whether the game's own archives
hold one is **unmeasured** — a `grep` over the packed archives found nothing and then found
nothing for `gfx/items` either, which is certainly there, so the instrument was mute and says
nothing at all. **B51**: Adult's page never says how you become an adult — the sentence is the
page's *preamble*, which the parser drops by a rule written for "X is a passive item…", and
`transformation::requires` reads that very line for its digit before throwing the sentence away.

**And M4's second sub-project opened and closed, both halves.** The design was taken in
conversation — Run is a diary, Live shows the run in progress *and what finishing it would
open*, abandoned runs stay in the list marked, and the list is filterable — then **2a** was
planned and built the same evening: the archive that fills itself has a screen, and nothing
new crossed the IPC.

**The order of the list was wrong the first time, and that is the evening's last lesson.**
"Newest session first" and "a name that is not a clock keeps its place" read as compatible
in prose and **cannot both be obeyed by one comparison**: with an unreadable name between two
readable ones the comparator has a cycle, and `Array.sort` given a contradictory comparator
does not fail — it answers something arbitrary, which is the worst way to be wrong. Sorting
only the readable names into the slots they already hold is a total order.

**And then the ambiguity ended, because the log was asked.** Live had been saying *"il log
scrive «Cain», e il gioco chiama così 2 personaggi"* — the honest answer while the archive only
had a name. The owner asked for better and one grep over this machine's logs answered:
`Initialized player with Variant 0 and Subtype 3`, and **the subtype is the character's own
id**. A Tainted form has its own, so the game itself tells the two apart.

**What the fold had to learn is the part worth keeping**: the line arrives *before* the seed on a
solo run and *after* it online, so a fold looking only forward loses every solo run and one
looking only back loses every online one. An init with no run yet is held for the run about to
start, and cleared by it. In co-op the line repeats per player and the first is the run's. The
rules version goes to 2, so the store folds again rather than serving runs produced before the
line was read. **The refusal to infer the form from the starting items — written twice — is now
unnecessary rather than overruled**, which is the ending that kind of rule waits for.

**Then Live became a dashboard**, on the owner's ask — *"più carine e complete di dati"*.
Three facts were added to the wire, each because the screen could not say something true
without it: a run item's **sprite**, an offered achievement's **fan-out**, and the
**completion row of the character being played** — two rows when the name reaches two forms.
The row is *copied from the matrix*, never rebuilt: the matrix is the one place that knows
how a cell is read, and a second reading of the same counters would be a second chance to
disagree with the screen that draws them all.

**And two defects of the session's own making, both found by looking.** `size-icon-row` is
not a token, so the character's head was an unsized sprite — the same failure as the want
suggestions hours earlier, from the same cause. And `liveAnswer()` carried **no return
type**, so TypeScript never compared the fixture to the contract: it sat in the old shape,
the screen threw on it, and `pnpm typecheck` stayed green through all of it. Annotating the
fixture found it in one run. **A fixture nobody typed is a fixture nobody checked.**

**Then 2b — Live — the same evening.** The run being watched, and what finishing it would
open, grouped by the cell it needs. The join is **one command**, as N8 pointed. The rule is
narrow — a run opens an achievement only when *everything* still missing from it is a mark
for this character — and the test was mutated to check the wider rule turns it red.

**The character stayed ambiguous, deliberately.** The log prints a name and the game gives a
Tainted character the base form's name, so the screen says *"il log scrive «Cain», e il gioco
chiama così 2 personaggi"* and shows both. Inferring the right one from the starting items
was refused twice — in the spec and again in the code — because it is an inference, and this
repo pays for those. **And the plan's single absence became two**: *no profile* and *no
graph* are different sentences to whoever is reading, which the spec's own "the run draws
either way" is what made visible.

**And the window earned its keep again**: two badges wore the `Unknown` variant — the
question mark this design keeps for *what the app could not read* — on **online** and
**abandoned**, which are facts about a run. Nothing in the suite draws a badge.

**N8 closed, both halves, and the last cleanup item with it.** The two graph screens became
**one command** — a screen load reads the profile once, by construction and not by a counter,
because `unlock` stopped being a command and there is no second entry point left to count.
The frontend's own comment said the two answers belong to the same profile: a promise a pair
of commands could not keep, since a save written between them made the steps describe a
profile the list no longer showed.

**Then the profile itself is kept between commands**, and the interesting part is what makes
it a cache rather than a `OnceLock`: **the game rewrites that file while the app is open**.
Three rules in `ipc::SaveCache`, each with a test that was mutated to check it could fail —
the same profile the settings name, the modified time the cached read saw, and never a
remembered failure. And one detail that is a decision: the time is read **after** the load,
so a save written *during* a read is stale immediately instead of being trusted as a value
that saw half of each version.

**The counter N8 asked for is a call count over the policy**, not over a Tauri command. That
is not a shortcut: `crates/app` is untested by the rule that put the wiring there, so the
part with a return value worth checking moved to the pure crate where a test can reach it —
which is the same rule, applied instead of worked around.

**And B49's layout half closed**: `column list` is unwrapped into the list it already holds,
before the line pass, so Beelzebub's page lists its flies instead of printing the wrapper's
source. Raw template syntax **87 → 35**, one family exactly, and `orphan_closers` **50 → 0** —
the lone `}}` lines it counted were that wrapper's closers. What stays open is the half that
is not layout: `Book of Virtues synergy`, `Book of Belial synergy` and multi-line `{{bug|…}}`
wrap prose, not a list, so unwrapping would lose what they say, and that is the `Block`
variant — a contract change — the entry has asked for since 2026-09-08.

**The fix turned on two newlines, and the second broke a test before it was right.** A
parameter arrives trimmed, so the content's own newline has to be put back; and adding one
after it unconditionally leaves a blank line where the wrapper closed, which flushes the list
— the same cut `a_lone_template_closer_does_not_cut_the_list_in_two` was written against in
September, arriving from the other side. A test written for one mechanism caught a different
one, which is the argument for pinning behaviour rather than implementations.

**B50 closed as a measurement, and the first instrument was mute.** A `grep` over the packed
archives found nothing for `transform` — and nothing for `gfx/items` either, which is certainly
there: **the archives index by hash, not by name**, so that search could never have answered.
Asked properly, through `ResourceSet::read`, this copy of the game holds a **costume** per
transformation (`transformation_adulthood.png`, 3,770 bytes out of `afterbirthp.a`) and an
**animation** (`n020…n034_transformation_*.anm2`) — and no icon. Worse for drawing one: the
twelve names are the game's internal ones for sixteen pages, with holes in the numbering —
`mushroom`, `angel`, `mom`, `poop`, `drugs`, `evilangel`, `iwata` — and deciding which of the
sixteen each is would be a guess of exactly the kind that cost this project sections 3 and 6.
So the picture stays absent, with the measurement written into `icon.rs` beside the decision.

**B51 closed within the hour, and half of it was mine.** The preamble is kept for the
transformation kind alone, ahead of the infobox's `description` — measured first: that
parameter restates the Effects section on all sixteen pages, Guppy's is empty, and none of
them says how the transformation happens. Adult's page now says the pills. **The other half
was a defect of the same afternoon**: `hasRows` decided whether the three transformation rows
had anything to say, and was used to decide whether *the card* is drawn — while the card also
carries the description and "sbloccato da", the two facts that live on the entry. Adult's
description went dark along with the rows it does not have. `hasCard` is a second question
now, and it was shown able to fail before it was believed.

**What confirmed itself**: clicking a contributor opens that item's page, and Guppy is **not**
offered among the things you can want while being findable as a page — the separation made hours
earlier, checked by the person it was made for.

### 2026-09-14 — the suite was green on one machine

`fix/marks-real-partial-series`, cut from `develop`. Opened on a second machine, **without the
game and with four old samples**, to take the work that does not need either. The first thing
that ran was `scripts/check`, and it was **red on a clean `develop`**:
`the_online_bit_never_stands_without_the_cleared_bit` guarded against vacuity with an assert,
and the assert fired.

**The state it fired on is the one the guard was not about.** An empty `samples/` returns early
and never reaches it; the owner's full series holds bit 2 and passes it. The failure needs
exactly the state in between — some dated samples, none from the era the profile first won a run
online — which is this machine (one `rep+` file, 2025-01-12) and every second machine after it.
So the test was green where there was nothing and green where there was everything, and red only
in the middle: a shape no amount of running it on the main machine can show.

The absence is **declared** now, through `test_support::skip`, naming how many samples were read
and the latest of them. `scripts/check` counts those lines, so the missing coverage stays visible
instead of dissolving into an "N passed" — which is the same reason the skip convention exists at
all. The guard keeps its teeth: where a cell does set bit 2 the property asserts exactly as
before, and **on a series that reaches that era, this skip line appearing at all is the
regression**. The reasoning is in the doc comment, not only here.

Baseline after the fix: `pnpm check` green, **127 skips** declared, 79 real files touched.

**Then N3, on `feature/shared-facets`** — chosen because it is the one remaining cleanup item
that needs neither the game nor a save: pure frontend, Vitest, fixtures. Full entry in *Next up —
structural cleanup*. Six commits; **six components become three**, the source loses 78 lines net
and the tests gain 143.

**What the doing corrected in the item that described it.** `matchesQuery` was listed as
identical in the two modules and is not — Unlock searches three fields joined, the Collection a
name — which decides the engine's signature rather than decorating it: written assuming a `name`
field, B3's third list walks into it on day one. Two values crossed between the screens and the
item named one. The two label modules, listed as *staying two*, shared twenty lines the inventory
did not count. **The pattern is the same each time: an inventory taken by reading is right about
what is duplicated and wrong about the edges**, and the edges are where the design decisions are.

**The measuring rule the cleanup section gives itself does not survive its own last item, and
that is worth more than the item.** "Unification is measured in files that stop existing" was
written against a real failure — a shared module added beside the two it generalizes — and it
catches that one. It does not catch this one: seven files created against three deleted, and the
source 78 lines lighter. A duplicated engine becoming one engine plus the specs, values and
helpers it needed all along **is** more files and less code, and no count of files can tell that
apart from the failure it was written to catch. The half that answers is the behavioural one each
item also carries — here, no file under `screens/collection/` is a copy of one under
`screens/unlock/`, and adding a facet to one screen touches no file of the other. **The rule
should be read as a smell, not a criterion**; the criteria are the per-item "done when" lines,
which is where they already are.

**Measuring for that criterion found the next item.** `UnlockTable`/`CollectionTable` stay two, as
the item decides — but 27 of their 84 lines are shared, and they are not columns, they are the
scroll box around them; and `--spacing-unlock-body` is read by four screens. Registered as
**B43**, not folded into the branch: a virtualized list is a different subject from a faceted one.
Registered the same day: the BACKLOG had **two entries numbered B40**. Four documents reference
that number meaning the infobox one, none the other, so the other became **B42**.

**What this machine cannot say.** Nothing in the suite draws these components, and `pnpm ui:dev`
needs eyes. The three unified components are behaviour-preserving by construction — same
template, same classes, same tokens — with one exception that changed mechanism: the drawer's
`grid-cols-3`/`grid-cols-4` became `--facet-columns`. That one was checked as far as a machine
can, in the **built CSS** rather than the source, because an `@utility` nothing references
generates nothing and the grid would have collapsed in silence.

`pnpm check` green on the branch: 448 frontend tests over 64 files, 127 skips declared, and the
generated contract still agreeing — `lib/ipc/values.ts` is hand-written *beside* `types.ts`, and
`pnpm ipc:types` leaves it alone.

**Then every open backlog entry was tagged with what it needs before you can start it** — the
`**Needs:**` line, its vocabulary and its counts are at the top of `docs/BACKLOG.md`, and
`CLAUDE.md` points at them. Sixteen of the twenty-six need nothing but a clone, which is the
answer this session spent its first minutes finding out by hand.

**Tagging B34 meant reading it, and reading it said it was not closed.** The entry looked
finished — half done, half explicitly forbidden — and two things remained, both checked against
the code rather than the prose. `Target::Pickup`, which the 2026-09-13 correction called "the
remaining work", is still there in **22 places**. And `corrections.json` still judges
`transformation:Guppy` and `transformation:Beelzebub` `unknown` for the reason *"the model can't
say 'N of these'"* — which the transformations sub-project made false the same day it was
written: `requirements.json` gives both `at_least: 3`, and `Target::Transformation` resolves
through `threshold()` and never reaches `from_verdict`. **The row has to stay**: `verdict_required`
is `true` for every target by decision, and removing the two turns
`every_target_that_needs_a_verdict_has_one` red — run, not assumed. Only the text is wrong.
The entry now says both at its top, with a `Closes when` that replaces the one belonging to the
plan that was thrown away. **A correction box is where a finished-looking entry hides what it
still owes**, and nothing re-reads it until someone needs the entry.

**Then B34 was closed** (`feature/concept-not-pickup`, 28 files). `Target::Pickup` is
`Target::Concept` — the word `Resolution::Concept` and `Inline::Concept` already used for the
same thing, `inline.rs` defining it as *"a wiki page the game gives no id"* — and the key
followed it into `corrections.json` (49 rows) and the regenerated `requirements.json`, because
the old prefix left in the data file would have kept the lie where a human reads it.

**The measurement the correction asked for, and the mistake inside it.** Of the 49 targets the
variant holds, 4 are rows of the wiki's pickup table and 45 are not — `Hard mode` at 38 uses,
`Completion Mark` at 19, down to `bed` and `technology`. It was first read out as "4 pickups and
45 non-pickups" **and that reading is wrong**: `Coin`, `heart`, `pills` and `Blue Flies` are
pickups by any account and simply are not rows of that Cargo table, which holds the tarot cards
under their formal names. The split is by table membership. The honest reading is the one that
settled the name — the 49 have nothing in common except being wiki pages with no id — and it is
the *stronger* argument, which is why the correction is worth keeping rather than quietly fixing.

**What caught what.** The frontend's `assertNever` turned all nine reading sites of the wire type
into compile errors instead of silent empty branches. `every_target_that_needs_a_verdict_has_one`
went red the moment the key moved on one side only, which is how a half-migration was shown to be
visible rather than assumed to be. And one test had been documenting the mismatch it asserted
past: `only_unreducible_targets_reach_the_inventory` failed saying *"a concept has no id"* while
pinning `pickup:Hard mode`. Both generated artefacts were **rebuilt, never edited** —
`pnpm wiki:build` then `pnpm graph:rules`, offline, neither needing the game.

**A contract change travels with this**, and per the rule at the top of this file it is handed on
rather than merely committed: `Target`'s `pickup` variant is `concept` in
`ui/src/lib/ipc/types.ts`. Nothing in the design depends on the name — the frontend's nine sites
all answer "no page to open" for it — but the wire changed, and that is a fact about the design.

**Then B38** (`feature/decode-entities`): `&comma;`, `&colon;` and `&apos;` decode, the three
pickup quotes read `:(`, "Tears up, you feel forgiven" and "t's broken", and
`grep -c '&[a-zA-Z][a-zA-Z0-9]*;' dataset/wiki.json` is 0. The rebuilt dataset travelled in its
own commit — four lines — the way every regenerated artefact here does, which was checked against
the log rather than assumed: `chore(dataset): rebuild …` commits touch exactly one file.

**The entry prescribed building something that already existed.** `fn entity` in `inline.rs` was
already a closed list of eight, missing exactly the three that ship, so the work was three rows
rather than a decoder. **An inventory written by reading is right about the symptom and wrong
about the shape** — the third time that pattern showed today, after N3's `matchesQuery` and
B34's "orphan" verdicts. Reading the code before the entry is what keeps finding it.

What the entry was right about is the counter: an entity outside the list reaches
`Diagnostics::unknown_entities` now, with the text **kept** rather than dropped, because nothing
counting them is how three of them shipped. It is empty on this snapshot, and a unit test shows
it able to speak rather than leaving an empty counter to be trusted. A run is only counted when
it is shaped like an entity, or an ordinary `&` in prose would fill it.

**And one finding that was not a finding.** Trinket 138's quote reads `t's broken9Reroll your
dest`, and it was filed as **B44 — the wiki's own page is corrupt**, on the evidence that the raw
page is intact and says exactly that on line 4. **The owner asked for the link, and the answer
was in the file already open**, forty lines below the infobox: the page's Trivia says the quote
is a splice of three item descriptions — *"It's broken"*, *"109"*, *"Reroll your destiny"* —
because `'M` is a glitch-themed trinket named after the Generation I Pokémon glitch. **Looking
broken is the content.**

The repo already knew the shape: **B38's own second paragraph** lists "TMTRAINER's deliberately
corrupted string" among the quote disagreements that are not defects — one paragraph above the
work being done when this was filed. Two rules this project already holds, neither applied:
**read the whole page before calling it corrupt**, and **a thing that looks wrong on data we did
not write is a hypothesis until the source is asked**.

B44 closes as not a defect and stays for the wrong diagnosis, which is the useful half. What it
produced is a guard rather than a fix: `the_glitch_themed_trinkets_quote_is_meant_to_look_broken`
pins the string and carries the wiki's explanation, and was shown able to fail — replacing the
expectation with the plausible *"It's broken. Reroll your destiny"* turns it red. A distorted
string with no guard invites exactly one wrong edit, and the next reader would have made it with
the best of intentions.

### 2026-09-13 — the archive fills itself

`feature/log-watch`, cut from `develop`. M4 sub-project **1b**: `discovery` learns where the
game writes, `log-watch` reads it, `store` keeps it in a fourth migration, `ipc` shows it, and a
thread at launch backfills every session and then follows `log.txt`. Full entry with M4 in
*Milestones*; report in `docs/superpowers/reports/2026-09-13-log-watch-report.md`.

**Four of the five measurements contradicted a document, and the fifth was a bug a test found.**
The 4 KiB prefix the spec identified a launch by is the machine describing itself — OpenGL, the
driver, the game's own DLL path — and the only byte separating two launches inside it is
`load archives: N milliseconds`, so identity rests on an **anchor** in the run content instead.
The two logs of 2026-09-08 are one launch copied twice. `online_logs\` is **28 sessions** and
three levels deep, not 22 in a flat folder. `discovery` could not name the folder at all.
And `head(log, 4096)` on a file shorter than 4 KiB returns the whole file, so the prefix changed
with every line the game wrote and **every read looked like a new source** — which would have
imported the whole archive a second time, every time.

**Run against the real folders, which found two more things.** 28 sessions, 0 errors, 95 runs —
16 won, 23 died, 36 abandoned, 20 open — and a second backfill that imports nothing. The first
pass took **212 seconds** because every event was its own commit; with the events and the offset
they belong to written as one transaction it is **0.67 s**. Speed is the smaller half: events
written while the offset stays behind are events the next read files a second time, which is the
duplicate-runs failure by another road.

**And `Died` had coverage all along, in files nobody had copied.** 1a asserted that no log in
`samples/logs/` held a `Game Over` and wrote a test to fail the day one did. It failed here: the
real folder has deaths in 16 of its 28 sessions. The gap was in the sampling, not in the game.
A fifth sample closes it and the guard test is replaced by the real one.

- [ ] **The agreement with the game's own counters could not be measured**, and that is the
      result rather than a gap. The spec named 2026-09-08 as the one window that could answer it;
      the window does not contain its own run — the log calls `unlock steam achievement` twice
      and **no slot of 642 turns on** across it. One solo, non-Greed win with a snapshot either
      side closes it, and `crates/ipc/tests/runs_real.rs` fails the day a window holds a run.
- [ ] **Not seen in a real Tauri window.** The archive was verified against this machine's real
      folders outside the app; the watcher firing while a game is actually being played has not
      been watched, and neither has the `#verify` section drawing it.
- [ ] **Neither screen**, on purpose: `Live` and `Runs` stay placeholders, which is the spec's
      own boundary — a layout written now would get a vote on a contract nobody has used yet.
- [ ] **Identity is still a heuristic**, said plainly rather than hidden: 64 bytes of run content
      at the offset already read. Its failure mode is re-reading a log, never merging two.

### 2026-09-13 — a run, from the game's own log

`feature/run-model`, cut from `develop`. M4 sub-project 1 split in two when the plan was
written and the size was visible: **1a**, the pure `run` crate, is this. 41 tests in `run`
and 9 in `test-support`. The full entry is with M4 in *Milestones*; what belongs here is what
the day found.

**Every correction came from opening the files instead of the notes.** The spec was a day old
and three things in it were already wrong — the migration number, the log lines quoted without
the `[INFO] - ` prefix they all carry, and the claim that the samples cover every outcome but
one. That last one is the worst kind: `Game Over` is in **none** of the four logs, so `Died`
has no real-data coverage at all, and a suite that passed without saying so would have read as
coverage.

**The finding the spec had no line for.** The seed line labels itself `New`, `Continue` or
`Net`. A `Continue` is a run resumed from an earlier launch, logged with the seed it already
had, and the spec's rule — a new seed while the previous run is open means abandoned — would
have marked a run still being played as abandoned and then counted it twice. The fold decides
by seed and not by label, because a label can be a word we have never met. `Net` came free with
it, and it is the discriminator the open question about co-op and `STREAK_COUNTER [22]` needs.

**A test found a bug in the guard that fixes that.** The first version read any repeated seed
as a resumption, including one replayed deliberately after the run had ended. `Open` in that
condition is load-bearing, and it is there because the test was written before the code.

Next is **1b**: the watcher, the fourth migration, run identity, the backfill of the
`online_logs\` sessions, and the agreement with the save's own counters.

### 2026-09-13 — the contract stops being written twice

`feature/generated-contract`, cut from `develop`. N7 / B2: `ui/src/lib/ipc/types.ts` is
generated by `pnpm ipc:types` and `scripts/check` fails when it is stale. The full entry is
with N7 in *Next up*; what belongs here is what the day actually cost and found.

**The work was mechanical and the findings were not.** Deriving `ts-rs` on the real types and
letting the compiler close the set took one pass; reconciling the generated file against the
committed one took the rest, and that reconciliation is where everything was found. Four
defects, none of them visible to a test: two fields typed `string` that are unions, a missing
`Infobox` variant that made a transformation page throw (**B40**), and `ProgressMark` living
behind `for_tests` while crossing the boundary inside `SearchHit`.

**The instrument was made to speak before its silence was trusted.** The gate was checked with
a `#[serde(rename)]` rather than a renamed variant: a renamed variant breaks the build, which
proves nothing about a check. The attribute changes the wire and leaves every call site
compiling — which is the shape of the incident the item exists for — and it failed both
`cargo-test` and `ipc-types`.

**Two things the plan had wrong and the execution corrected.** The equality of generated and
committed cannot be a Rust test, because the committed file is prettier's output: the gate is
where that assertion can live, and the spec was amended before the plan was written. And the
scratch copy cannot live in the system temp — prettier resolves its configuration by walking
up from the file, so a copy anywhere else is formatted by other rules and reads as stale
forever; on Windows it cannot open an MSYS `/tmp` path at all.

Next in the order is **M4 sub-project 1**, which was the whole argument for running N7 first:
every view-model M4 adds to the contract would otherwise have been hand-mirrored and then
regenerated.

### 2026-09-13 — the app outlives its windows

`feature/background-and-tray`, cut from `develop` in the main checkout (not a worktree: the
`samples/packed` junction makes one expensive to throw away). Spec
`docs/superpowers/specs/2026-09-13-background-and-tray-design.md`, plan beside it. An explicit
request from the owner: closing every window must leave the app running, reachable from the
notification area, and launching the executable twice must never make a second process.

- [x] **The exit is prevented, and only the right one.** `RunEvent::ExitRequested` carries
      `code: None` for "the user closed the last window" and `Some` for `AppHandle::exit`
      (`tauri-2.11.5/src/app.rs:225-232`), so the tray's **Quit** needs no flag to get past the
      guard — it simply doesn't match the arm. The one `_ =>` arm the repo allows: `RunEvent` is
      `#[non_exhaustive]` and is not ours.
- [x] **One window recipe, three callers.** `WindowConfig::create: false` plus
      `WebviewWindowBuilder::from_config` means the window's shape stays in `tauri.conf.json` —
      no constant moved into Rust, no third copy of the background colour (B18).
      `crates/app/src/window.rs` is the only place a window is opened: startup, the tray, and
      the second-instance callback. `background.test.ts` now pins `create === false`.
- [x] **The tray, always present.** Left click opens or focuses, right click is Open / Quit, in
      the system's language (`sys-locale`, mapped by the same primary-subtag rule as
      `i18n/locale.ts`). Which window a click means is a pure function in `ipc::tray_action`,
      with `tab-preview` filtered out: focusing the drag's preview would hand the user a ghost.
- [x] **One instance.** `tauri-plugin-single-instance`, registered first, calling the same
      `open_or_focus`.
- [x] **Part of 3.7 landed early: the session document.** `store` migration 3, one JSON document
      in one row, the shape migration 2 already used for the queue. **An object with a version,
      not a bare array**, so 3.7's sidebar width and per-table sizes (B27) join as named keys
      without a migration. Written only by `main`, only as its tabs change, debounced —
      **never on close**, where a write races the webview's teardown. `main` now starts pending
      like every other window and is seeded from its session; an empty seed was already the
      landing tab, so "no session", "the setting is off" and "the document is unreadable" all
      arrive at today's behaviour through code that already existed.
- [x] **Two settings and a screen.** `stayInBackground` and `resumeTabs`, both on by default, in
      `settings.json` where B6 said the flag belongs. `/settings/background`, and a new property
      in `sectionNav.test.ts` — *every Settings route is listed in the Settings sidebar* — which
      caught the screen being reachable only by typing its path. A count would not have.
- [ ] **Not yet checked on the machine.** `pnpm check` is green (436 frontend tests, 7 skips, all
      for samples that were already missing), but nothing here has been clicked: the tray icon,
      the left click, Quit, the second launch, the toast, and the setting turned off. The
      notification is the part most likely not to work — a Windows toast wants a registered
      AppUserModelID, which an installed build has and `pnpm dev` may not — and it degrades to
      the icon's tooltip rather than blocking the exit.

### 2026-09-13 (last) — one drag everywhere, and a tab that leaves the window

`feature/drag-and-windows`, cut from `develop` in its own worktree (`C:/Projects/isaac-dome-drag`)
so the wiki infobox work kept the main checkout. Spec
`docs/superpowers/specs/2026-09-13-drag-and-windows-design.md`, plan and report beside it.
**B31 and B15 together**, because the tear-off *is* the tab strip's drag continued past the edge.

- [x] **B31 closed.** `lib/drag/dragList.ts` (pure, tested), `composables/useDragList.ts` (the
      choreography, plus the `Escape` neither screen had), `components/ui/drag/DragGhost.vue`
      (`aria-hidden`: it is a picture of the row, not a second one). The queue and the strip are
      its two callers; no pointer choreography is left in a screen. **No shadow token was
      invented** — the flat skin is a decision — so the lift is a `primary` border on an opaque
      sheet, looked at on the Kit, in the strip and in the Plan.
- [x] **The multiwindow structure.** `lib/window/` grew from one module to seven: the port and
      its fake (`?windows=fake`), the messages between windows, the seed handshake, the focus
      order (kept in the frontend, not in Rust as the spec had it — every window already watches
      its own focus), the hit test with its DPI conversion, the preview window, the pointer
      source. `tabModel` gained insert/detach/seed, written **by subtraction** (`Omit<Tab, 'id'>`)
      so another session's rewrite of what a tab *is* merges without touching them.
- [x] **What one window writes, every window reads again.** Three events from Rust
      (`crates/app/src/events.rs`), **payload-free by rule**: they say "read again", so no second
      wire shape exists to keep in camelCase and nothing new can leak across the boundary.
- [x] **Two contradictions found while building, both fixed in the contracts, not patched.** The
      spec's two closing rules made each other unreachable — *opening* a window for a tab already
      alone is a no-op, *joining* it to one that exists is not — and the scanner's "window API
      outside `src/lib/window/`" rule matched one namespace of the three, so it promised a check
      it never made. Verified with a bait file before and after.
- [x] **A defect found by running it, not by reading it**: `watchAppEvents` called `listen()`
      with no `isTauri()` guard, which outside Tauri throws from inside a mounted hook and takes
      the shell down. `preview.ts` had the same hole. Fixed; the rule was already ours.
- [ ] **The spike is not run, and it is the one thing that decides a module.** Whether WebView2
      keeps delivering pointer events with the cursor outside the window needs a real window and
      a hand on the mouse. `pointerSource.ts` is written as if the answer were yes, behind the
      interface that is all the other answer changes, with a silence timeout that **cancels** the
      drag rather than landing a tab nobody released.
- [ ] **Not verified on the machine**: the eleven checks of the plan's Task 17 — tear off, dock
      back, the last tab, the window that closes, two windows on the Plan, a profile change, a
      scale change, two monitors at different factors, a target window closed mid-drag.
- [x] **A worktree runs a dimmed suite in silence.** `samples/` is git-ignored, so a fresh
      worktree starts with `144 skipped, 0 real files touched` and says nothing. The samples were
      copied in and the `packed` junction recreated: `7 skipped, 1266 real files touched`.

### 2026-09-13 (later still) — you name what you want, and the app says what to play

`feature/goals-want`, cut from `develop` at `a83a7ef` in its own worktree, `pnpm check` green
on the result: 55 frontend test files / 356 tests, 7 skips all pre-existing and named. Spec
`docs/superpowers/specs/2026-09-13-goals-want-design.md`, report
`docs/superpowers/reports/2026-09-13-goals-want-report.md`. **B37 closed.**

- [x] **The graph reads from the other end.** `crates/ipc/src/want.rs` takes a `wiki::Target`,
      resolves it to every achievement that grants it, and answers with the series still
      missing — four states (`done`, `availableNow`, `chain { steps, unknown }`, `noProfile`),
      none of them deduced from the length of a list, because an empty `missing_chain` means
      four different things.
- [x] **The order is the queue's, asked rather than reinvented.** The preview builds an empty
      throwaway `plan::Queue` and enqueues the want with its chain, so the series shown and the
      series "metti tutto nel Piano" writes are one computation, not two rules to keep aligned.
- [x] **A target names every achievement that grants it.** `achievement_unlocking`'s `.find()`
      became `achievements_unlocking -> Vec<u32>`: **14 of the 45 challenges** are named by
      more than one achievement, so the singular version was hiding a second way in on a third
      of them. The old function stays, rewritten over the new one, and the goals import is
      unchanged.
- [x] **A vacuity guard moved the test, and taught us something about the product.** The
      reference profile's deepest chain is **one step**; measured over five samples the deepest
      anywhere is **four**, on the *youngest* profile (the co-op partner's, 128 done). Chains
      shorten as a profile advances, because an earned prerequisite leaves them. The ordering
      property now runs where there is an order to check, and the Kit page is the only place
      the multi-step shape can be looked at.
- [x] **The want is a place.** `#/goals?want=item:105`, with `pageKey` — the codec wiki pages
      already travel by — so back, forward and tab restore reach it without a line of new
      plumbing, and `wantable()` makes the vocabulary a tested function rather than a filter
      inline in a component.

### 2026-09-13 — the landing page gets a name, and an achievement gets a page

`feature/screens-goals-detail`, cut from `develop` in its own worktree, `pnpm check` green on
the result: 53 frontend test files / 345 tests, 7 skips all named and none of them this
sub-project's. Spec `docs/superpowers/specs/2026-09-13-goals-and-achievement-detail-design.md`,
report `docs/superpowers/reports/2026-09-13-goals-and-achievement-detail-report.md`.
**B32 and B35 closed.**

- [x] **An achievement's wiki page is its detail.** `/wiki?page=achievement:<id>` grows a
      "Il tuo profilo" block — state, what is missing with its links, what you get with its
      links, how much it opens, add to the Plan. No new route and no new tab identity, so
      every link that already pointed at an achievement page became the detail for free. The
      block sits **outside** the branch that draws the wiki's answer: what the profile knows
      does not depend on the dataset, and a page the dataset never heard of still has a state.
- [x] **"Prossimi passi" is now "Obiettivi consigliati"**, grouped by the reason a row is
      there, with a card that leads with *what you get* and links to the detail. `NextSteps`
      became sections on the contract, and `StepsBasis` gained the `Closeness` it was left
      open for.
- [x] **The wait on `feature/wiki-infobox` was imaginary, and was checked rather than
      believed.** The spec said this work had to follow that branch; `git diff
      develop...feature/wiki-infobox --name-only` touches no frontend file at all. Corrected
      in the spec before any code was written.
- [x] **The card's "how" line could not come from the game file.** `achievements.xml` states
      an `unlock_condition` for 283 of 637 achievements and for **16 of the 119 unlockable
      now** — none of the five rows that motivated B32. The wiki answers where the file is
      silent and all **637** now carry a line. `hint` was **renamed** `condition` rather than
      widened in place: the rename is what forced every reader to be revisited, and two nobody
      had listed turned up printing it under the label "indizio del gioco:".
- [x] **`crates/wiki` gained `plain`, not a second copy of it.** The private `flatten` in
      `infobox.rs` is now public API with its own tests, and the copy is gone.
- [ ] **The design pack is five fields behind.** `unlock.json` and `next_steps.json` predate
      the character's form, the requirement's page, the target's page, the steps' sections and
      the resolved condition. All five are backfilled in the fixtures **with a console
      warning**, so the development screens are poorer than the app in ways that are declared
      rather than mysterious. One `pnpm design:export` on a machine with the game clears all
      five, and nothing else does.

### 2026-09-13 (evening) — back and forward, one history per tab

`feature/tab-history`, cut from `develop`, suite green (335 Vitest tests). An explicit
request from the owner: browser navigation, **per tab**, with the side buttons of the mouse
bound by default.

- [x] **A tab is its own little browser.** `Tab` stopped being one location and became
      `entries` plus `index`; `tabLocation(tab)` is what it shows. Navigating stacks and drops
      whatever forward was left, back and forward move the index and do nothing at the ends,
      a tab opened from another starts with a single entry, and the history stops at
      `HistoryDepth` (50) so what sub-project 7 will persist stays small.
- [x] **What counts as a location is the decision, and the owner made it**: the route, the
      wiki category, the page — **not** `q`. The search navigates on every keystroke, so
      without that exclusion "back" walked backwards through what had been typed; with it,
      one back leaves the search for the route the tab came from. Two wiki pages of the same
      category remain two entries: the page is what the tab *is*.
- [x] **Three gestures, one action.** `lib/shell/navigation.ts` reads `Alt` with an arrow and
      the two side buttons (`MouseEvent.button` 3 and 4) into the same `HistoryAction`; a bare
      arrow stays with whatever table has the focus. `usePointerShortcut` is `useShortcut`'s
      twin: it stops the press, where the webview would navigate on its own, and acts on the
      click. The NavBar carries the two arrows, disabled when the stack ends.
- [x] **A finding the unit tests could not have produced.** On the dev server, one back from
      the search bounced straight back to it: the debounced navigation lands *after* the
      gesture and dragged the tab to a screen the user had just left — back was unusable on
      the one screen the rule was written for. `refineTab` is the answer: a keystroke changes
      the entry the tab is showing, and reaches no tab at all when the tab has moved on.
      The bug predates the history — a sidebar click right after typing had the same race —
      but only a back makes it visible.
- [x] `ButtonVariant.Chrome` keeps its transparency when disabled. The base style gives every
      disabled button a filled surface, which inside a bar that has none reads as a box, and
      the chrome had no disabled button until today.
- [x] **WebView2 does deliver buttons 3 and 4 to the DOM**, checked by the owner with a real
      mouse in a `pnpm dev` window on 2026-09-13. The question was open because dispatched
      events prove the wiring and not the delivery: a webview that swallowed the side buttons
      would have left the keyboard and the arrows working and the gesture dead, with the whole
      suite green. No native handler is needed.

### 2026-09-13 (earlier) — N5, and a `try` that was right to stay

`feature/ui-view-stores`, cut from `develop` after N4 merged, suite green.

- [x] **Three stores became three lines.** `collection.ts`, `completion.ts` and `graph.ts`
      wrote the same `view` / `status` / `error` triad and the same `try` / `catch`, with
      only the call in the middle different. `LoadStatus` left `profile.ts` for a file that
      holds nothing else: every other store had been importing a shared enum out of one
      particular store, which said, wrongly, that the profile owns the idea.
- [x] **The factory alone would have left half the duplication standing.** `defineViewStore`
      fits a store whose whole shape is the triad, and three others carry more — the profile
      loads two things under one status, the queue clears its mutation state first, the wiki
      skips a read it has already made. Extracting `tracked(status, error, read)` from under
      the factory is what makes the item's own criterion true: **no** store writes that
      `try` / `catch` by hand, not just the three the item named.
- [ ] **One read kept its `try`, and that is the finding.** `wiki`'s `loadEntry` reports a
      failure *without ever claiming a success* — the status belongs to the index, and a page
      arriving must not mark the index `Ready`. `tracked` always sets `Ready`, so forcing it
      there would have been a behaviour change wearing a cleanup's clothes, and one that
      nothing would have caught: the suite is green either way.
      **What is left open is the thing underneath it**: a page's failure lands on the
      *index's* `error`, so one missing wiki page can make the whole index look failed. That
      is inherited, not decided here, and it needs a decision rather than a refactor —
      `feature/wiki-infobox` is in that code now and is the right place to settle it.
- [x] **The graph's two answers became one `view` object**, which is what they always were:
      a single read, so `unlock` and `steps` cannot straddle a profile change. Two guards
      that checked both halves now check the one object.

### 2026-09-13 (later still) — N4, and a counting rule that disagrees with itself

`feature/ui-diagnostics`, cut from `develop` after N6 merged, suite green.

- [x] **All four components stopped existing**, where the item expected three. `PlanAlerts`
      went too: its button is the only alert that asks for something, and a slot carries it,
      so the Plan keeps the action without keeping a component of its own.
- [x] **A defect the change found rather than caused.** Making a diagnostic's scalar fields
      the translation's values means `{count}` is placed by the string. The four
      count-bearing strings had no placeholder — they were fragments written for
      `${d.count} ${t(...)}` — so the number would have disappeared with **nothing failing**.
      Whole sentences now, the number placed where each language wants it.
- [x] **Two invariants, both of which caught something while being written.** Every key a
      table can produce exists in `en` and `it`. And every value handed to a translation is
      spent by it — which failed twice before it passed: first because `reason` and `wanted`
      are an object and a list, which no sentence can place, and then because the builder
      hands a diagnostic's values to the title *and* the body, so the check belongs to the
      entry and not to each part. A test that has to be argued into shape twice is a test
      that was worth writing.
- [ ] **The section's counting rule does not survive this item, and the entry says so.**
      "Unification is measured in files that stop existing", and N4 goes from **4 files and
      303 lines to 6 files and 278** — lines down, files **up two**. It cannot go the other
      way: one idea needs a spec, a component, and one table per screen, which is the item's
      own prescription. The rule is a proxy for "did the duplication actually go", and it
      did — all four copies are deleted. **Merging the four tables into one file to make the
      count fall would have been gaming a metric, and the count would have been the only
      thing improved.** N3 and N5 delete a *copy* rather than a copy plus its replacement,
      so their counts really do fall; the rule holds there and it is worth keeping for them.

### 2026-09-13 (last) — N6, and a second copy that is not a second-class one

Two sessions wanted the one working copy. The wiki-infobox session needed it intrinsically —
its task 7 compares the wiki against the game's own files — so it kept it, and N6 ran in a
**git worktree** instead, on `feature/app-wiring` cut from `develop`.

- [x] **The worktree limitation was a setup gap, not a fact.** The other session had tried
      one and reported that `samples/` and `node_modules` are missing, so the real-data tests
      skip in silence — the failure `CLAUDE.md` names by incident. But `samples/packed` in the
      main copy *is already a junction* to the installed game: the same mechanism gives a
      worktree the whole folder. 9 MB of loose samples copied, `packed` junctioned,
      `pnpm install`, done.
      **Proved rather than assumed**: `scripts/check` in the worktree reports **7 skips and
      1265 real files touched**, the same numbers as the main copy, with the same three
      reasons — the two absent historical saves. A suite that passes by skipping reports a
      smaller number, so the match is what says the instrument can speak.
- [x] **`IpcError` moved to `ipc` first, and the item had not seen that it had to.** The
      tested functions return it and it lived in the Tauri crate: they could not leave while
      the type they assert on could not be imported. It is a wire type and `ipc` is the only
      contract — where N7 will generate it from.
- [x] **The pure halves went to `store`, not to `ipc`.** `plan_parts`, `store_error` and
      `store_unavailable` all take a `StoreError` or a `GoalsRead`, and the dependency runs
      `store → ipc`, never back. Not wiring either: each has a return value worth checking.
      They live in `crates/store/src/degrade.rs` with the five tests that were `app`'s only
      `#[cfg(test)]` block — the proof, written in-house, that the rule was broken.
- [x] **`icon_url` stayed in `app`, against the item's own list.** `CLAUDE.md` records why:
      on Windows the webview sees a rewritten `http://isaac.localhost` origin, and knowing
      that is the Tauri crate's job. Moving it because a list named it would have undone a
      decision the repo had already argued. A cleanup item is a description, not a warrant.
- [x] **Eleven files, none over 220 lines**, and all three closing criteria checked rather
      than asserted. The item said five `OnceLock`s; there were six — `SearchState` arrived
      with 3.5b and nobody added it to the list. Same shape as N1's count moving twice: a
      survey is where an item starts, not where it ends.

### 2026-09-13 (later) — N2: the reason stops being a sentence

`feature/typed-ipc-reasons`, five commits, suite green. The item asked for one enum and the
boundary turned out to carry the same defect in four shapes.

- [x] **Four enums in `crates/ipc/src/reasons.rs`** — `IoReason`, `SaveReason`,
      `SettingsReason`, `StoreReason`. The numbers travel as numbers: `NewerSchema` carries
      `found` and `supported` instead of the sentence that already spelled them, and the
      sentence is written in `it.ts` / `en.ts` where the word order is the translation's
      business. `IoReason` keeps only the two `io` kinds a user can act on — anything else
      is `Other`, because a third guess at what the OS meant is wording, not information.
- [x] **`store_reason` and `describe_open_error` are gone**, which is what the item asked
      for: they existed only to produce a `String`. The mappings moved to where they can be
      tested — `OpenError` and `io::ErrorKind` in `ipc`, `StoreError` in `store`, which
      already depends on it — and each is kept by a **property**: whatever the OS or SQLite
      wrote, none of it survives into the serialized reason.
- [x] **A sentence had already eaten the information it was hiding.** `queue_import_goals`
      answered `"database illeggibile"` in *both* arms — a database that would not open and
      a query that failed read identically, and the reason the first arm already had was
      discarded to print it. That is the item's own charge (an untyped field lets the
      wording drift) in its worst form, and it is the strongest argument in the whole
      cleanup for typing a field rather than agreeing on its contents.
- [x] **The defect had an instance outside `IpcError`, and the UI was printing it.**
      `SetupDiagnostic::UnreadablePath` carried `io::Error::to_string()` and
      `NoSavesCard.vue` concatenated it onto a label. `discovery::Diagnostic` carries the
      `io::ErrorKind` now. On the way, `Discovery` and `Diagnostic` lost their `Serialize`
      derive: nothing used it, and both hold a `PathBuf` — serializing either would have put
      a full path on the wire, which is the one thing the boundary forbids. An unused derive
      on a type full of paths is a loaded gun, not a leftover.
- [x] **The frontend maps to keys, not to text.** `ui/src/lib/ipc/errorText.ts` is pure and
      sits beside the wire types; `useIpcErrorText` only joins what it returns. Pure because
      the UI suite tests logic and not components — a composable calling `useI18n` needs an
      app context, and the mapping is the part worth checking. The test walks **every**
      variant of every error and asserts each key exists in `en` and in `it`: a missing key
      renders as the key itself, which reads as a bug report to the user and fails nothing.
- [x] **N1's report was corrected, not quietly patched.** It said "one notation" over six
      crates; there were seven. `discovery::testing` — a `#[doc(hidden)] pub mod` under a
      different word — matched neither spelling N1's closing grep looked for, and surfaced
      only because N2 had to touch that crate. The code is fixed and the entry now says what
      happened. **The lesson is about the report**: "one notation" was written as a closed
      fact when what had been established was "the two spellings I grepped for are gone". A
      closing criterion states what it can see, and a grep cannot see a spelling nobody
      thought of.
- [ ] **N7 is next, and it is the reason this went first.** The contract is still
      hand-mirrored: `types.ts` gained the four reason types by hand, which is four more
      chances for the failure `CLAUDE.md` already records. With the error type finally still,
      generating it has nothing left to chase.

### 2026-09-13 — N1, and the order the cleanup runs in

Two things, and the first decided the second. **M4's design reorders the structural
cleanup**, because three of its eight items are cheap now and expensive once M4 is written.
The order is written into the section rather than expressed by renumbering — the numbers are
names, and other items and `docs/IMPROVEMENTS.md` point at them:

> **N1 → N2 → N7 → N6 → M4 sub-project 1 → N8 → N3, N4, N5**

- [x] **N7 moves from last to third.** It was last because it is expensive, and it runs
      after N2 because generating the contract while the error type still moves means
      generating it twice — that half stands. What M4 changes is the other half: M4 adds a
      family of view-models to the contract, and every one written before N7 is hand-mirrored
      into `types.ts` and then regenerated. The repository's largest silent risk gets
      *larger* between now and M4, not smaller.
- [x] **N6 moves in front of M4** so its commands land in `commands/` instead of growing the
      964-line file that then has to be split. **N8 moves behind M4**: its `SaveState` has to
      invalidate when the `.dat` is rewritten, and M4's watcher exists because the game
      announces exactly that — doing N8 first means inventing a heuristic M4 then replaces
      with the game's own statement. N8's free half is not held back by this. N3, N4 and N5
      are frontend and M4 has no screen, so they keep their place.
- [x] **N1 closed** (`feature/cleanup-names`, two commits, suite green). The two Italian
      names went, and the test-only public API went from three notations to one that is
      **structural rather than agreed**: each crate with such an entry point has exactly one
      `pub mod for_tests` and nothing test-only anywhere else in its public surface. Six
      crates have one. The inner items went back to `pub(crate)`, and `Doc` and `ProgressMark`
      left `ipc`'s contract — they were only ever the return types of test-only calls.
- [x] **The survey had counted four such entry points and there were six.**
      `unpack::__lzw_decompress` and `catalog::__heads_parse` wore the same `__` prefix and
      were not on the list. The second serves an **example**, not a test, and the module's
      doc says so rather than inventing a second word: a second word would rebuild exactly
      the problem the item removes. This is the shape the whole section is supposed to have —
      *unification is measured in what stops existing*, and a survey's list is a starting
      point, not the boundary.
- [x] **N1's closing criterion was unmeetable and was corrected before the work, not after.**
      It read `grep -riE '(icone|segreto|_di_)' crates ui/src`, and `ui/src/i18n/messages/it.ts`
      legitimately contains the Italian word *icone* in its prose: the grep could never go
      quiet. A criterion that cannot be met is worse than none — it reads as "still open"
      forever, and the item would have been re-opened by whoever ran it next.

### 2026-09-12 (last) — M4 designed as a model, not as a screen

M4 had been one line in this document since it was written. It is four pieces, and this
session designed the first: `docs/superpowers/specs/2026-09-12-m4-run-model-design.md`.
What it deliberately does **not** design is either screen. `Live` and `Runs` are both views
of this model, and designing them first would let a layout shape the IPC contract — the
failure this repo has a rule against.

- [x] **Three decisions taken in conversation, and the document says which parts are which.**
      Sections 2 to 4 were written after "mi fido, scrivi un doc": they follow from the three
      decisions, but they were not walked through one by one, and the spec's own header names
      them as the part most worth disagreeing with. That line is there so a reader knows where
      the agreement stops.
- [x] **Backfill everything.** The log is rewritten on every launch, so an archive holding
      only what the app was running to see starts empty. `online_logs\` has 22 sessions on the
      disk today and `log.txt` holds the current launch: the archive is born full instead. The
      cost is bought knowingly — two ways to produce a run, and a run identity that cannot
      lean on a clock the log does not have.
- [x] **Events are the archive; a run is a fold over them.** Backfill and live become the same
      function over a stream that is finished in one case and growing in the other. The rules
      file was already required to be updatable without recompiling, so re-derivation is a
      consequence of a decision already taken rather than a feature invented here: `store`'s
      third migration keeps `events` as rows and `runs` as a derived cache carrying the rules
      version that produced it, and a newer rules file invalidates the cache.
- [x] **Abandoned runs are kept and marked, and whether they count is measured, not chosen.**
      The save keeps `STREAK_COUNTER [22]`, `BEST_STREAK [23]`,
      `NEGATIVE_STREAK_COUNTER [113]` and `DEATHS [10]`, so "do abandoned runs count?" is a
      property to test against the game's own numbers rather than a product opinion.
- [x] **The boundary is the repo's own rule, applied to a watcher: in a file watcher the
      parts worth checking are not the ones that touch the disk.** The last line read may be
      half-written, one event spans six physical lines (B8's `Framebuffer Width:` block), and
      a file that got *shorter* is a new launch rather than a file that grew. All three live
      in the pure `run` crate, as `Tail`. `log-watch` keeps `notify`, an offset and a read,
      and is thin enough to go untested like the Tauri crate. Its trigger is B8 finding (a):
      the game logs `Saving PersistentGameData to Steam Cloud: …` 132 times in one run, which
      beats watching the directory because it also names the file.
- [x] **The rules file maps text to events and does nothing else.** Every judgment lives in
      the fold, because the rules file is data a user can edit and a rule that decided
      *meaning* would let a bad file change what a run is — and would put untestable logic
      outside the crate that is tested. Two judgments, both B8's. The starting item is told
      from a treasure-room find by **position** alone — `from pool treasure` lies for Judas's
      Book of Belial, in a line identical in shape to a real pickup — which is the only reason
      `RoomTransition` is an event at all; getting it wrong over-counts finds by one per run,
      with well-formed data and nothing failing. And actives replace one another, so summing
      `Adding collectible` lines gives a player holding five books.
- [ ] **Two weaknesses written into the document rather than left to be discovered.** Run
      identity rests on a **prefix hash** — the log has frame numbers, not timestamps, and a
      seed can be replayed deliberately — which is a heuristic, not an identity, and is stated
      as such so it gets attacked. And the streak property cannot yet be asserted for co-op or
      Greed: today's online Greed win did **not** move counter 22, and nothing so far separates
      "co-op does not count" from "Greed does not". Until it is separated the property holds
      for solo, non-Greed runs only, and the archive does not claim to mirror 22 for the rest.
- [ ] **One log still missing, and one accessor.** `samples/logs/` holds the three the spec
      names — a won solo run with an unlock mid-run, the online Greed win, and a run with
      neither death nor ending, which is the `Open` case. Every outcome except `Died` is
      covered; that one needs one more log. And `test-support` needs an accessor for logs
      beside the ones it has for saves, declaring `sample: …` or `skip: …` on stderr: no test
      opens `samples/logs/` by hand.

### 2026-09-12 (late) — a blocker stops being a dead end

The owner read the app and said the obvious thing: where it says *bloccato* it names what is in
the way and stops there. "Non basta il nome" — and then, on the shape, "una lista di link tipo
menu windows?", which turned out to be the right answer to a problem the design hadn't noticed:
the why lived in a **tooltip**, and a tooltip is not a thing you can click.

So the badge became the trigger of a menu and the tooltip went. Each blocker is an entry that
opens that thing's wiki page — offline, already in the binary. The page is resolved in Rust,
because the frontend holds `{ kind, id }` and a page is keyed `entity:20.0.0`; the mapping that
knows a boss is identified by **the file name of its portrait** already existed inside search's
`documents()` and now lives once, in `crates/ipc/src/wiki_target.rs`. A requirement and a
collection lock carry `page: Option<Target>`, `Some` only when `Dataset::entry` answers, so an
entry the dataset has no page for is present and disabled rather than a link that goes nowhere.

Two risks were written into the spec as things to check rather than assume, and both turned out
benign, measured on the running app: the Plan's drag starts on the grip's own `pointerdown`, so
the badge never steals it (a row dragged onto another still repaired itself and said so); and a
menu open in the virtualized table goes away with its row when the row is recycled.

Two things stayed undone on purpose. A **mark** and a **counter** carry no page: two of the
twelve matrix columns (Boss Rush, Greed) are not entities, and a column → entity table written
from the names would be the kind of curation this document keeps refusing to guess (B36). And
what a node *unlocks* is still text (B35). What could not be checked from here is the last
click in the real Tauri window — it builds and runs, but it can't be driven from a session; on
fixtures every entry is disabled, because the committed design pack predates the field and the
fixture layer says so in the console instead of pretending.

### 2026-09-12 (afternoon, the game running) — the third bit has a name, and a documented fact was wrong

The owner was about to play an online Greed run and asked what the data collection wanted.
The honest answer at the time was "very little": three of the four open measurements ask
for a **solo** run, because this document and CLAUDE.md both said a co-op run leaves the
personal save alone. The run was worth instrumenting only for B21's second question.

That answer was built on a wrong fact, and the run proved it wrong in twenty-six minutes.

- [x] **A matched window on a live online run**, taken with the instrument that was already
      in the repo: `core-save`'s `live_probe`, plus a snapshot before and after. Greed Mode,
      Cain, **won** (`Greed State Set: STATE_TURN_GOLD`, then `playing cutscene 21`), the
      whole session inside the window.
- [x] **"A run ending in co-op leaves the personal counters untouched" is false**, and had
      been stated in three places. The personal save splits: **20 activity counters, the
      completion mark and bestiary tallies 1 and 2 move**; achievements, items, challenges,
      bosses and sections 3, 5, 8, 9 do not. The old claim came from reading "no achievement
      moved" as "nothing moved" — the two halves look identical if you only count unlocks.
      Corrected in CLAUDE.md, here, and in B21.
- [x] **Bit 2 of a completion mark is "won online".** B21 had this as its first question and
      expected to answer it from backups; it was answered from the run instead. The cell
      `Greed × Cain` went 2 → 7 — bit 2 on exactly the boss and character played. The series
      then agreed: 6 of 6 bit-2 dates have an `online_logs\` session, ~60 marks on days
      without one never gained it, and on 2026-08-31 the **same character on the same day**
      took one mark with the bit and one without, so it belongs to the run, not the day.
- [ ] **Local co-op — claimed closed, then reopened the same hour.** The argument was that a
      local co-op win is the only thing lighting two bits at once in index 188, which made
      2026-07-22 and 2026-09-01 identifiable. **It doesn't hold: 188 accumulates** — `2 → 6`
      on 09-01 adds Cain and keeps Magdalene — so two bits is two wins in the window, not two
      players in one run. What survives is by elimination only: no day without an online
      session ever produced a bit 2, and the owner confirms local co-op happened at least
      once. B21 point 3 stays open.
      **The evidence was already in the repo**: the B8 spike report of 2026-09-08 records
      `188: 4 -> 12` on a winning Judas run under "one thing that does not fit". Reading it
      before concluding would have cost five minutes; it was read afterwards, while looking
      up something else. That report can now drop the entry — the misfit was accumulation,
      and what clears the mask is the question that replaces it.
      Index 188's bit → character map *was* re-derived from the series rather than assumed,
      and that part holds: it confirms the map `marks_real.rs` already uses.
- [x] **The reading is pinned by a property, not by a value**:
      `the_online_bit_never_stands_without_the_cleared_bit` in `crates/ipc/tests/marks_real.rs`.
      An online clear is also a clear, so no cell may hold 4 or 6. It carries its own
      vacuity guard — if the series ever stops containing a bit-2 cell the test says so
      instead of passing for free.
- [x] **Bestiary, half a step** (B9): tallies **1 and 2** moved, **3 and 4** did not, which
      rules out "kills" for the second pair. The trailing word moved **+11** in one run.
- [ ] **Two holes left, both honest.** 2026-06-29 and 2026-07-06 carry a bit 2 with no
      session folder; `online_logs\` only starts on 2026-08-24, and on 07-06 index 188 names
      one character while two took marks. What settles them is knowing whether the game
      rotates that folder — not another run.

> The lesson worth keeping is not about co-op. A fact measured once, written into CLAUDE.md,
> and then used to decide what *not* to measure, quietly cost three open questions two
> weeks: every co-op session since 2026-08-24 could have answered them. **"Useless as
> evidence" is a claim that has to be re-measured, not inherited.**

### 2026-09-12 (night) — one index, two ways to ask it

On `feature/screens-search`, sub-project 3.5b, the half of 3.5 that waited for the scale to
rewrite the tokens.

- [x] **One index, two sources, one document per target.** The wiki side is flattened once —
      each page a title plus one string per section, `ref` labels, table cells and nested list
      blocks included, `edition` inlines unwrapped — and the catalog side is read at every
      query, never cached, because "the game isn't installed" is not an answer to keep. A
      target both sides know is **one** row: the catalog's name is its title, the wiki's an
      alias when it differs.
- [x] **Six tiers, then the profile, then the name.** "Not done before done" is what makes the
      profile ranking information rather than a facet (B5). Every word has to be in the **same**
      field: "monstro spits" finds nothing, and that is the point — no single field holds both.
- [x] **Measured, not assumed**, which is what B5 asked for: 1,727 pages indexed in **15 ms**,
      "brimstone" answered in **19 ms** over the whole dataset and the installed catalog, in a
      debug build. **FTS5 in `store` is not needed**; the timings are printed by
      `crates/ipc/tests/search_real.rs` and pinned by nothing, because a time is a machine's.
- [x] **A result is not a row, it is the destinations it opens**: a wiki page, an Unlock row,
      a Collection row, and the screens the frontend names itself. The palette caps each group
      at five and the screen shows them all; the backend's order is never re-sorted.
- [x] **Three corrections to the spec, written back into it**: `SearchDiagnostic` is a bare
      string (the repo has no tagged unit enum, and a second convention is how a `switch` falls
      into no branch); `Command` gains two props, because the typed text has to leave the
      primitive to be debounced; and the search state is a **composable**, not a store — the
      palette and the screen ask different questions at once.
- [x] **A defect older than this sub-project, found by the eye and fixed at the root**: a
      virtualized table never measured again when the interface changed size. 3.5c unified the
      number the rows are positioned with, but nothing told the virtualizer to re-measure, so
      changing the scale with Unlock already open drew 70px rows 80px apart — overlapping, with
      nothing failing. All four tables share one composable now, and the watch is stated in a
      test.
- [x] **Three more the eye found**: a palette row opened twice (Reka replays the click, so the
      row opens on `select` and `Ctrl` is read from the window); a Collection row opened the
      list at "0 / 721", because the screen's default states outrank a name the user just asked
      for; and every row carried the same `non noto` badge, which is B29's case again — a mark
      is drawn only when it tells that row apart.
- [ ] **Not seen in a real Tauri window**: the command against the real index, and the ranking
      on a profile that has marks. The development server ranks synthetically and says so in the
      console; the ranking that counts is Rust's, and the tests pin that one.

### 2026-09-12 (evening) — the interface scales

On `feature/screens-scale`, sub-project 3.5c, ahead of the search half and of 3.6 because it
rewrites the token files and every screen built after it is built on the scaled tokens.

- [x] **A root scale, not the webview's zoom.** `html` is `calc(16px * var(--app-scale))` and
      every size token is in rem, so one custom property carries the whole interface: it works
      on the Kit page, it survives a torn-off window (B15), and it leaves the rules
      inspectable in CSS. The eleven steps are Discord's — 50 · 67 · 75 · 80 · 90 · 100 · 110 ·
      125 · 150 · 175 · 200 — written twice, in `crates/ipc/src/settings.rs` and in
      `lib/scale/steps.ts`, with a test that reads the Rust source: two ladders that drift are
      a slider that saves a value the backend refuses.
- [x] **A value off the ladder reads as 100**, never as the nearest step: 137 is a file we
      didn't write, and "about 125" would draw the app at a size nobody designed. The
      guarantee lives in `Settings::scale()`, so no command can read the field raw.
- [x] **Pixel art keeps whole multiples.** A 32px sprite is drawn at `round(2 × factor)` times
      its size, an integer on the root, and the three tokens that *are* a sprite are measured
      from it — at 125 the multiple rounds up, so a cell in rem would be smaller than the
      symbol inside it.
- [x] **Applied before the first paint**: `main.ts` reads the setting and writes the properties
      before mounting, and the splash of B18 covers the wait. The store applies first and saves
      after — the user asked for the size, so a failed write leaves the app at it and says so.
- [x] **Two defects the scale made visible**, both fixed at the root rather than patched: the
      three virtualized tables positioned their rows with a fixed 40 while the row was drawn
      from the token, so at any scale but 100 they would have overlapped (one `rowWidePx` now,
      in place of three copies of the number); and the sidebar's width was in screen pixels, so
      at 200% its labels were cut off — it is kept in the design's pixels now, multiplied by
      the scale, with the drag dividing the pointer's travel by it.
- [x] **The scanner gained a rule**: a px token in `assets/` needs a reason on the line above.
      Four keep px and say why — the scrollbar, the two radii, the tab's container breakpoint —
      and the sprite tokens are `calc()` on the multiple.
- [x] **Looked at through Playwright**: 100 → 110 → 200 with `Ctrl` `+`, the root at 32px, the
      sidebar at 424px with its labels whole, Unlock's rows 80px tall and 80px apart, the
      preview card pinned while the page scrolls, and the navbar no longer overlapping itself.
- [ ] **Not seen in a real Tauri window**, and the value's survival across a restart is the one
      thing the development server can't show: its fixture keeps the size for the page's life,
      the file is the app's.

### 2026-09-12 (later) — the review's small half, implemented

On `feature/review-fixes`, cut from `develop`: the entries of the review that needed no
measurement and no design pass. Ticked above as they landed; what the boxes below still show
open is either waiting for a run of the game (B19, B20, B21), for a design pass (B17's flow,
B23, B29's bar), or for its own sub-project (B22 with 3.6, B26 as 3.5c, B27 with 3.7).

- [x] **B28, and it was a contract**: `UnlockTarget::Character` and `RequirementView::Character`
      carry `tainted`, the Unlock facet keys on the character's id instead of its name, and the
      label is composed from a message (`Tainted {name}` / `{name} contaminato`) so the word
      order belongs to the translation. `useMessages` learned to take values for a message,
      which it had no reason to before. A real-data test pins the pair 82/484 as two targets
      under one name; the pack's `unlock.json` predates the field, so the fixture fills it as
      the base form and says so once in the console.
- [x] **B24 reverses a decision of the shell spec**, on purpose: clicking Wiki, Progressi or
      the cog moves the active tab to that section's first page at once (`firstEntry`), with
      `Ctrl` opening it beside, and the cog reads as lit while a settings page is open.
- [x] **B25 removes a route**: About is a `Dialog` over the current tab — the name, the version
      read from the bundle through `getVersion()`, the three promises, the attributions. With
      it went `RouteName.About`, `TabOrigin.About` and their placeholder, which the compiler
      listed for us.
- [x] **B16 and B18 are the window's first frame**: the navbar's placeholder square is the
      Dome mark itself, inlined so it takes `currentColor` and keeps dimming with the focus;
      the icon is `primary` now, regenerated for every size from the one SVG with
      `tauri icon`. The window and the document both declare the background token and a splash
      holds the mark until the bundle mounts. Three places hold that colour by necessity, so a
      test reads all three and compares them.
- [x] **The copy the owner called false** (B17, item 1) is gone: no "Schermata 0", no "è lo
      stato che decide ogni numero", and the Settings hint says what is actually under it.
- [x] **B29's two sight fixes**: "Faccette" becomes "Filtri", and a value with nothing behind
      it is no longer listed with a 0 and a disabled checkbox — on Unlock too.
- [ ] **Not seen in a real Tauri window**: the splash, the icon in the taskbar and the
      installer, and the version in the dialog all need a build.

### 2026-09-12 — the owner's first-launch review, as backlog entries

The review the "(delegated)" decisions were waiting for, dictated screen by screen and
written up as B16–B33 in `docs/BACKLOG.md`:

- [x] **Chrome**: the brand mark becomes the app icon, in `primary` (B16); no white flash at
      launch, a splash with the mark (B18); About is a dialog, not a page (B25).
- [x] **Navigation**: clicking a section navigates to its first page at once and the section
      reads as lit — this reverses Decision 5 of the shell spec, on purpose (B24).
- [ ] **Profile**: "Schermata 0" and the settings copy go; the screen is a welcome flow with a
      preview per save (B17).
- [ ] **Completion**: the game's paper under a mark (B19); the 40 unknown cells get measured
      and "non leggibile" leaves a healthy save (B20); a mark taken in multiplayer, bit 2 as
      the candidate to verify (B21); hard implies normal and two counts per row (B22); the
      KPI strip loses "120 celle" and "40 non leggibili" (B23).
- [x] **Scale**: the whole interface scales from Settings, and it is pulled ahead as
      sub-project 3.5c because it rewrites the tokens (B26).
- [ ] **Tables**: a table fills the page or is sized by the mouse, and a dragged size is
      remembered per table, in the place 3.7 chooses for the session (B27).
- [x] **A bug in Unlock, traced**: slots 474–490 are the Tainted characters, whose `text`
      in `achievements.xml` is the base name; the save and the graph are right, the label
      drops the `tainted` flag the marks matrix already uses (B28).
- [ ] **Collection's filter**: "Faccette" goes, and so do values with nothing behind them
      (B29).
- [ ] **Plan**: dragging a row lifts the whole card above the page, the red marker still
      names the landing, and one composable does the drag for the queue and the tab strip
      (B31).
- [ ] **Next steps**: a name that says what the list is ("Obiettivi consigliati"), an intro
      in the player's words, and cards that read what, how, why, add (B32).
- [ ] **Backings from the game**: the achievement drawing sits on the sheet the game puts
      behind it, to be found in the files (B33); Completion's cell on the sheet the design
      already took from the game (B19, confirmed).

### 2026-09-11 (late night) — the Collection

On `feature/screens-collection`, the first branch under the one-per-sub-project rule; every
commit pushed as it lands.

- [x] **A new contract, not only a screen**: nothing joined the save's item collection with the
      catalog. `ipc::collection_view` lists the catalog's collectibles by id with name, icon link,
      quality, pools and origin, whether section 4 holds each one, and the lock its achievement
      puts on it; the `collection` command wires it; `design-export` writes
      `contracts/payload/collection.json`. Handed on in `DESIGN-BRIEF.md` §7.7.
- [x] **Section 4 keeps its structural name**: "in the collection", never "seen" or "picked up",
      which nobody has measured. **Trinkets have no slot** and aren't listed. **Unread is never
      false**: a missing section or a slot past its end gives `null`, with its diagnostic.
- [x] **Seven synthetic Rust tests and one real-data property** (each item against its own slot),
      which skips here: this machine has no game.
- [x] **A collectible's state and the facets are pure functions** (`lib/collection/`): in the
      collection, to find, locked, unreadable; quality with unrated as a value, pool with none,
      kind, origin; counts that leave their own facet out; the screen opens on what hasn't been
      found.
- [x] **The fixture says what it is**: the pack has no `collection.json` yet, so ids, kinds and
      names come from `images/INDEX.json`, locks from `unlock.json` and origins from the catalog's
      id ranges — all real — while quality, pools and the collection flag are synthetic, declared
      by `collectionSource()` and warned once in the console.
- [x] **Looked at through headless Chrome**, 18 checks: 422 / 211 / 88 / 0 and "299 / 721" on the
      default pick, 22 of 721 rows in the DOM, the pips, the synthetic warning, sprites, a locked
      item's tooltip naming its achievement, a pool count falling from 67 to 15 when quality 4 is
      picked, the search, the last row reachable, `?collection=unread` (721 unreadable and the
      alert), `?catalog=none`, `?art=none`. No finding in the app; two in the script.
- [ ] **The pack's real `collection.json`** waits for `pnpm design:export` on a machine with the
      game and a save; until then the Collection has only been seen on synthetic quality, pools
      and flags. Not seen in a real Tauri window either.

### 2026-09-11 (night) — the Plan and the queue

Still on the owner's delegation; every commit pushed as it lands.

- [x] **3.3a merged into `develop`** ("merge: screens 3.3a, Next steps and Unlock, into develop"),
      the screens branch fast-forwarded onto it.
- [x] **A move names the row it lands under** (`plan::Queue::move_after`,
      `queue_move(achievement, after)`). Building the drop found two silent defects in
      `queue_move(achievement, to)`: the view leaves completed and unresolved rows out, so the
      screen's indices are not the document's; and `move_row` applied `to` after taking the
      dragged dependents out, so `[1, 2, 3, 4]` with 2 requiring 1, moved to 2, landed
      `[3, 4, 1, 2]`. Pinned by six cases and a second 500-round property; `QueueView` and the
      saved document are unchanged, no migration.
- [x] **The queue's decisions are pure functions** (`lib/plan/`): the anchor of a drop or of an
      Alt+arrow, the row a move stopped under — read from the answer, not guessed while
      dragging — the summary, the wishes a step serves, what can still be queued.
- [x] **The fixtures keep a queue in memory** with a port of the Rust repair, seeded with the
      design pack's rows and a done row above them, so the hidden-row case is on screen;
      `?queue=empty|unavailable|unreadable` beside `?catalog=none`.
- [x] **The Plan screen**, and "in coda" with one click to add on Next steps and Unlock. The
      IpcError sentences moved to `useIpcErrorText` and `ipcErrors.*`, for the profile and the
      queue alike.
- [x] **Found by a test timing out**: every graph answer with art took 1.7 s, because
      `packIconUrl` scanned all 1,500 image paths for each link — on every Unlock, Next steps
      and queue command of the development server since 3.3a. Indexed once: 2 ms.
- [x] **Looked at through headless Chrome**, 38 checks: the three row kinds and their badges; a
      drag of 480 below 69 bringing 55 along; 55 sent to the top stopping under 480, with the
      hint naming it; Alt+↑; removing a wish taking its step; adding from the proposal, from
      Next steps and from Unlock; the four degraded states.
- [x] **Found by looking, and fixed**: at 248px the proposal truncated every step to "You …";
      it now names what the step unlocks.
- [ ] Not seen with the game installed, nor in a real Tauri window: the drag has only met
      synthetic pointer events.

### 2026-09-11 (evening) — merged into develop, then Next steps and Unlock

On the owner's delegation ("procedi come credi, anche mergiando i vari tree"; "pusha sempre in
modo che rimane allineato"): every commit is pushed as it lands.

- [x] **Merged into `develop`**: cycle 2 and screens 3.1–3.2 in one `--no-ff` merge of
      `feature/design-system-screens`, suite green on the merge result, `develop` pushed and the
      screens branch fast-forwarded onto it.
- [x] **Sub-project 3.3 split in two**: 3.3a (the node, Next steps, Unlock) done, 3.3b (Plan and
      the queue) next. Spec `docs/superpowers/specs/2026-09-11-screens-graph-design.md`.
- [x] **One answer for a node's state** (`lib/graph/nodeState.ts`), and its why grouped by kind
      in the state badge's tooltip. The Badge learns `partial`: the blocked colours, a dashed
      edge, the lock — `DESIGN-BRIEF.md`'s question 4, answered for design to overturn.
- [x] **Unlock's facets are pure functions** (`lib/graph/unlockFilter.ts`), tested first on the
      design pack's committed payload: 387 / 119 / 117 / 18, 231 nodes unlocking nothing
      catalogued, 274 without an origin, ten required characters. TanStack Table was read from
      9.2.4 and left out — its faceted values count an array cell as one value; TanStack
      Virtual renders the body.
- [x] **Without a catalog Unlock still has 641 nodes**: `ipc::unlock_view` sends one unknown,
      partial node per slot, where the brief said "empty nodes"; fixture, screen and brief
      follow the code now.
- [x] **Looked at through headless Chrome**: five steps with their pictures and the no-catalog
      alert; Unlock's four counts, 22 of 641 rows in the DOM, "unlockable now" (119), partial
      only (18, every badge dashed), Rebirth (107) with the other facets' counts moving and
      their zeros disabled, the name sort's last rows (slots 639–641), no catalog (254
      partial), no art, and the why tooltip ("Cosa gli manca · Personaggi · Samson").
- [x] **Found by looking, and fixed**: the fan-out sort put done nodes first; "1 filtri
      attivi"; the progress gate asked for a profile while the profile was still loading (a
      flaw of 3.1); the fixtures imported some 1,500 images to read the profile.
- [x] **Found by `scripts/check`, not by the tasks' own checks**: the partial Badge had no
      classes. The tasks grepped `vue-tsc`'s output for "error TS", which its colours split, so
      four typechecks read nothing. Checks are judged by exit code since.
- [ ] Not seen with the game installed, nor in a real Tauri window.

### 2026-09-11 (later) — Completion: the matrix on the game's own sprites

The second screens sub-project, taken on the owner's delegation ("comincia a fare in
autonomia"): spec, plan and execution in one session, every choice marked "(delegated)" in
`docs/superpowers/specs/2026-09-11-screens-completion-design.md`.

- [x] **B13 closed.** The marks map left `crates/design-export` for `ipc`
      (`mark_art.rs`, beside `BOSSES`): column, anm2 file, layer, frame 0 for normal and 2
      for hard, Delirium on the online lobby's `Background` animation. `crop` moved to
      `ipc::crop_png`; the icon protocol serves `mark/<column>/<tier>` and `head/<row>` as
      crops, through a `MarkFramesState` that never caches the game's absence.
- [x] **The contract changed additively, and was handed on** (`DESIGN-BRIEF.md` §5.1,
      §5.6): `tainted` and `headUrl` on a row, `art` beside `bosses`. `totals.started` now
      counts bit 0 or bit 1: a cell holding only the unconfirmed bit was "started" in the
      total and empty in the grid. No save we hold has such a cell, so no number moved.
- [x] **The screen**: four KPIs with their denominators, a legend, base and Tainted groups
      with their own counts, a tooltip per cell, row and column totals, sprites or bars.
      Every number comes from `lib/completion/completionView.ts`, tested first against
      §5.4's reference profile (166 / 368, 120, 3 / 34, 40 / 408), which the fixtures carry.
- [x] **Looked at, not only typechecked**: with no browser automation in the session, a
      throwaway script drove headless Chrome over the DevTools protocol against
      `pnpm ui:dev` — the KPIs, 408 labelled cells, 40 unknown, 215 images with none broken,
      none under `?art=none`, the gate under `?fixture=pick`. A hard heart that looked dark
      in a downscaled screenshot was measured by file and pixel colour: `heart_02.png`, the
      right tier.
- [ ] **Not run with the game**: this machine has none installed, so `mark_art_real.rs`
      skips, and whether each mark's frame 1 repeats frame 0's rectangle is still open
      (`pnpm design:export --dump gfx/ui/completion_widget.anm2` answers it).
- [ ] **Not seen in a real Tauri window**, like 3.1: first launch.
- [x] **B15 logged**, the owner's request met mid-session: tearing a tab off into its own
      window and dragging it back, with what the Tauri 2 documentation allows and the two
      traps it names (WebView2 pointer capture, `startDragging` on another window).

### 2026-09-11 — the screens' first sub-project: the window becomes the app

Cycle 3 is too big for one spec, so it was split into seven sub-projects
(`docs/superpowers/specs/2026-09-11-screens-shell-profile-design.md`), and the first one
landed on `feature/design-system-screens`. The owner delegated the decisions ("do as much as
you can, we look at it on first launch"): the spec marks each one "(delegated)" so the
first-launch review knows what to question.

- [x] **One transport for every command.** `call()` in `lib/ipc/transport.ts` is `invoke()`
      in Tauri and the fixtures under `pnpm ui:dev`, so the shell can be looked at in a
      browser: `?fixture=none|pick|active`. A command without a fixture throws instead of
      answering something plausible.
- [x] **Tabs own locations; the router renders the active one.** Pure rules in
      `stores/tabModel.ts` (open after the active tab, close to the right neighbour, the bar
      never empty, move, navigate), a Pinia store on top, Vue Router on memory history.
- [x] **The chrome goes live.** `decorations: false`, a capability for minimize, maximize,
      close and dragging; `lib/window/appWindow.ts` is the only module that talks to the
      window, and the scanner now says so.
- [x] **Profile selection is the first real screen**: the Steam → game → saves chain, the
      broken chain with its diagnostics, the candidate table with nothing preselected, the
      active profile with the ten sections read. Progress routes sit behind a gate that shows
      the same selection until a profile is active; the navbar indicator is one click from it.
- [x] **Every other screen is a placeholder** that names the sub-project bringing it.
- [x] The verification page moved to `#verify`, beside the Kit; the scanner's last exemption
      went with it (`0 violations, 0 declared exemptions`).
- [x] Checked on the development server with the `pick` fixture: the gate, choosing a
      profile, the indicator turning active, the profile screen, tabs opening and closing by
      middle click. 103 Vitest tests.
- [ ] **Not checked in a real Tauri window**: `decorations: false`, the window controls and
      dragging the title bar need `pnpm dev` on the machine — first thing at first launch.
- [ ] Choosing a folder by hand is B14: the screen says where the chain broke and offers
      "Riprova", with no button that does nothing.

### 2026-09-10 — the design system's first cycle, and a typecheck that checked nothing

The Claude Design export arrived as a zip of six pages. Read against the brief, it
contradicted itself in thirteen places — two light palettes, three button heights, two
encodings of the matrix cell, a "multiplayer" meaning for the one bit the brief calls
unconfirmed, gold for both "unlockable now" and every heading — and the spec resolves each
and hands the list back to design. One theme, the dark one; Tailwind's default scales reset,
so an off-system class generates nothing; motion on `steps()`; Determination only.

- [x] **`pnpm typecheck` checked no file.** `ui/tsconfig.json` is a solution file, and
      `vue-tsc --noEmit` on it exits 0 with a deliberate type error. Now
      `vue-tsc --build --force`; the existing code had 0 errors in build mode, so nothing
      had been hiding — but nothing would have been caught either.
- [x] **Found while planning, each by making the tool answer**: `shadcn-vue init` rewrites
      `main.css` (so `components.json` is hand-written); the registry's `data-open:` classes
      don't match Reka's `data-state`; vue-i18n's `t()` accepts any string (so
      `useMessages()` narrows the key type); Vitest hands a `?raw` CSS import an empty
      string unless `test.css.include` lists it; unconfigured tailwind-merge drops
      `text-body` beside `text-foreground`; the scanner's visible-text heuristic ended a tag
      at the `>` inside `has-[>svg]:`.
- [x] 23 primitives, dressed, on the Kit page (`pnpm ui:dev`, `#kit`); `App.vue` uses
      them and loses its exemption.
- [x] **Radio group, missed by the catalogue.** Checked against the export before the
      merge: `Schermate.dc.html` draws a radio on every row of the profile candidates, and
      the spec listed Radio neither among the primitives nor among the exclusions. Added
      with the kit's values (16px, 8px dot); the screen's own drawing — 13px, a 2px edge,
      a fill outside the palette — loses to the kit, as every component value does.
- [x] **The 4px grid was 3.5px.** `base.css` set the text size on `html`, which moves
      `rem`, and Tailwind's spacing step is `0.25rem`: measured on the Kit page, a 14px
      checkbox where the kit draws 16, a button's padding at 14, a 10.5px gap. The size
      moved to `body`; after the fix the same measurements read 16, 16 and 12, the kit's
      numbers. `--spacing-sprite` (4rem, "a 32px sprite doubled") and
      `--spacing-achievement` (5.5rem, "half of 176") had been 56px and 77px all along.
      Pinned by `ui/src/assets/base.test.ts`.
- [x] **Cycle 2 — app components.** Scope, cell encoding, bit 2, edition tag, folders and
      shell agreed in conversation; the cell's scale chosen on a comparison page with the
      real sprites (40px, flat paper `#E9DADF` sampled from `paper_00.png`, symbol at 2×)
      and the rest delegated, marked **(delegated)** in the spec to revisit on first
      launch. Thirteen off-palette colours mapped, three colour tokens added.
- [x] **Found on the Kit page, not by the tests**: at the 34px minimum a tab's icon, name
      and close spilled onto the next tab (now a size container that drops the close, or
      the active tab's icon, below 64px); a wiki reference's icon lifted its label off the
      line (`ButtonSize.Inline` is `inline` now). A tab drag that "did nothing" was the
      probe's fault: the mouse pressed coordinates of a section scrolled out of view.
- [x] **Commits rebuilt before pushing**: `git mv` staged two renames that the next
      `git commit` swept into an unrelated commit, leaving two commits whose `App.vue`
      imported a moved file. The unpushed commits were redone so each compiles alone.
- [x] **Cleanup pass**, four independent reviews (reuse, simplification and magic values,
      efficiency, altitude): the KPI bar is the `Progress` primitive (new `size`, `tone`);
      the navbar's section icons come from `tabOriginIcon`; `ButtonSize.Section` and
      `IconCompact` replace classes that overrode sizes at the call site; the container
      token has its own `theme/containers.css`; `TabStrip` reads tab positions once per
      drag and shares `TabRole` with `TabItem`; `WikiInline` resolves each icon once;
      `WikiBlocks` forwards one object; `markBarShare` and `AriaCurrent` name the last
      literals. Skipped: composing `Collapsible` inside `CardCollapsible` (same forwarding
      either way), quarter-step paddings measured from the export, re-rendering every tab
      on a drag move (a handful of tabs).
- [x] **A regression the first squeezed-tab fix had brought in**: making the tab a size
      container zeroes its content width, and the strip, sized by its tabs, collapsed every
      tab to 34px with only three open. Seen on the cleanup pass's screenshots, measured
      (`flex-basis` 150px applied, strip 104px wide), fixed with
      `contain-intrinsic-inline-size` and a shrink-only basis: three tabs at 150px, nine at
      39px with the drag region at its 130px minimum.
- [ ] First launch: look at the delegated choices in the real window.

### 2026-09-09 (last) — the documents catch up with the repository

Six documents claimed a state the project had left, and they disagreed with each other,
which is worse than any one of them being wrong: `README.md` still described the graph
screens as carrying `{ kind: "stub" }` — gone from the wire with M2 — and still had M2
unticked three days after it closed.

- [x] **The handoff is recorded everywhere it was promised**: `CLAUDE.md`'s State section,
      `README.md`, `DESIGN-BRIEF.md`'s delivery path, `docs/IMPROVEMENTS.md` and B3 in
      `docs/BACKLOG.md`. All five said, in their own words, "next step: the handoff".
- [x] **`CLAUDE.md` now states the constraint rather than the precaution.** "The IPC
      contract doesn't change for the convenience of a screen that doesn't exist yet" was
      written about a hypothetical reader; the paragraph now says what it means with a
      design under way — a change is handed on, not merely committed — and names the silent
      case, `core_save::Kind` crossing inside `ipc::SectionCount` under a TypeScript
      `string`.
- [x] **`README.md`'s milestones agree with `STATUS.md` again**, M2 checked with the same
      caveat (the graph is done, the Unlock *section* is frontend work) and M3 carrying the
      queue.
- [x] **`docs/PROJECT.md` §03 was a year behind on the counter tail**: it still called
      404–522 an unidentified mix of families. Rewritten around what was located on
      2026-09-08, with the 40 unread cells named as 40 and not as "roughly eighty".
- [x] **The bestiary gets its own block in `CLAUDE.md`**, because "read the declarations,
      don't assume the shape" is a rule a future session needs before it opens the file,
      not after. The section-10 row in all three tables says so too.
- [x] **A stale doc comment in `crates/ipc/src/graph.rs`** — `basis: Stub` declares it —
      described a variant `StepsBasis` no longer has. Comments rot where no test looks.
- [x] **`B2` in `docs/IMPROVEMENTS.md` moves from "later" to "next"**: generating
      `types.ts` from the Rust types was tidiness while nobody read the contract, and is
      now the thing that would have caught a variant rename reaching the wire unannounced.

The design package's copy of the brief was re-synced with `cp`, which is what
`design-export` does with it.

### 2026-09-09 (later still) — the package is handed over

Recorded rather than done: the handoff happened outside this repo, and step 4 of "Handoff
to design" had stayed unticked while claiming that all that remained was the act itself.
A status file that describes a state the project has left is worse than one that says
nothing.

- [x] **Step 4 closed.** The design is under way. Screens 0, 4 and 5 went over on real
      data, the other four on the fixed contracts.
- [x] **The IPC contract is now live**, which changes what a rule in `CLAUDE.md` means. "The
      contract doesn't change for the convenience of a screen that doesn't exist yet" was a
      precaution about a hypothetical; there is now someone on the other end of it, and a
      change to `ipc` or to `ui/src/lib/ipc/types.ts` has to be handed on and not merely
      committed.
- [x] **One drift is already on the record.** The section 3 and 6 rename earlier today
      (`017131c`) moved two label strings inside the package —
      `per_char` → `level_counters` and `cards_pills` → `bosses`, in
      `contracts/payload/save_summary.json` and the brief's save-format table. No count
      changed and no TypeScript type changed, so if design's copy predates that commit
      those two strings are the entire difference. Noted at the step itself, where someone
      checking the package will actually look.

### 2026-09-09 (later) — the bestiary: four tallies, not one list

B9 recorded section 10 as "twenty zeros, five small counters, then a sorted key → count
list of 1,364 pairs". Probed on four saves across two editions, that reading does not
hold — and the section turned out to be **self-describing**, which is better than a
reading anyone has to trust.

```
words[0..19]   twenty zeros
words[20]      11            constant in every save
words[21]      the total, exactly the sum of the four sizes below
words[22]      4             how many tallies follow
then 4 x ( id, size, size/4 records of (key, count) )
```

- [x] **The three anomalies were one mistake seen from three angles.** Read as a single
      list the keys descend three times, 445 of them repeat, and one word dangles at the
      end. The descents are the three intermediate `(id, size)` headers; the repeats are
      the same entity counted in several tallies; the dangling word is a six-word slip from
      reading four lists as one. Each had a plausible wrong explanation available — "the
      game doesn't sort", "the duplicates are variants", "there's a terminator". What
      settled it was not a better hypothesis but an **arithmetic constraint**: `words[21]`
      is the sum of the sizes, and the tallies consume the section to the word. A reading
      that closes exactly, on four files, is not one reading among several.
- [x] **`Save::bestiary_tallies()`**, module `crates/core-save/src/bestiary.rs`. Eight
      real-data properties plus five unit tests on the packing and on degradation. The
      tallies keep the id the file gives them and get **no names**: the same entity appears
      in all four with different numbers, so they are four counts of one space, and which
      count is which needs the game. Naming them now would be the mistake that had section
      6 reporting bosses as cards.
- [x] **A test caught the throwaway script, not the code.** `the_section_is_consumed…`
      was written expecting zero leftover words, taken from the probe that mapped the
      layout. The reader said one. The probe had an off-by-one that ate exactly that word —
      which is a real word, present in every save and growing 11,343 → 29,725 across the
      samples. It is now carried as a **value** rather than a count, and a property says it
      never goes backwards. A test whose number comes from a script is only as good as the
      script.
- [x] **`EntityId` is the triple `crates/wiki` already indexes bosses by**
      (`Dataset::boss_key`), so the join is there the day the tallies have meanings. Not
      built now: nothing to join *to* yet.
- [x] **`crates/core-save/tests/helpers.rs`** — `series`, `every_dated_save` and the rest
      moved out of `real_saves.rs`, which had grown to 355 lines and was about to be copied
      rather than shared. A helper that declares on stderr which sample it used is exactly
      the thing that must not exist twice with two behaviours. `real_saves.rs` is down to
      280 lines.

**What this did not do**, and it is now written down in "What only a machine with the game
can answer": name the four tallies, and identify the trailing word. Both want a matched
window against a live run.

### 2026-09-09 — B9: the rename two measurements had already paid for

Nothing in this needed the game: the evidence for sections 3 and 6 was measured on
2026-09-08 and written down, and the point of writing it down is that a later session on a
different machine can act on it. This machine has four saves over two eras and no
`samples/packed`.

- [x] **`Kind::PerChar` → `Kind::LevelCounters`, `Kind::CardsPills` → `Kind::Bosses`**, and
      the public `SaveDiff.cards_pills` → `SaveDiff.bosses`. Sections 5, 8 and 9 keep their
      `Unknown` names: they have the game's own word for it — Mini Bosses, Cutscene
      Counters, GameSettings — and a log line is not a measurement. The enum's doc comment
      now *states* that distinction instead of leaving it to a backlog entry, so the next
      reader can't mistake "the game calls it X" for "we checked".
- [x] **The evidence went in first, as a property.** `stage_counters_leave_index_0_unused`
      asserts what refutes the old name: a per-character table's first row is Isaac, and on
      a profile with hundreds of runs it cannot be zero. It is zero in all four saves, and
      **three of them are the 2024 `rep_` snapshots the original measurement never
      covered** — so the property now spans two editions and 14 months rather than one
      profile. Checked by moving the expectation to 1 and watching it go red, not by
      assuming.
- [x] **The recorded blast radius was wrong, and the correction is the useful part.** The
      B8 report had it that `Kind` "neither crosses the IPC nor drives product behaviour".
      It crosses: `ipc::SectionCount` carries it, so `"per_char"` and `"cards_pills"` were
      wire values and this rename changed the JSON the frontend reads. The workspace stayed
      green through it because the TypeScript mirror types that field as `kind: string`
      instead of a union — **a type wide enough to hide a contract change**. Now pinned by
      `crates/ipc/tests/summary_shape.rs`: ten variants against ten strings, plus a test
      that goes red if an eleventh variant arrives without a row.
- [x] **The design package stayed truthful without a re-export.** Two strings in
      `contracts/payload/save_summary.json` and two lines in the brief carried the old
      names. A re-export needs the archives and this machine hasn't got them — but the only
      thing that changed is the label: every count is untouched, and the new strings are
      exactly what the renamed code emits, which is what `summary_shape.rs` now pins. The
      brief's copy inside the package is a byte-for-byte `cp`, which is what
      `design-export` does with it anyway (`crates/design-export/src/main.rs:436`).

**What is deliberately not done.** Section 8's one ambiguous cell needs a solo run with a
known ending, and section 10's records are new capability rather than a rename. Both are
still open in B9, which no longer claims the rename among them.

> Left as a note, not acted on: `crates/core-save/tests/real_saves.rs` is now 355 lines and
> covers several subjects. Splitting it is worth doing, but not inside a rename's diff.

### 2026-09-08 (late night) — the two columns design asked about, and where they were

Design read the package and asked the obvious question: *no Mother and no The Beast?*
Correct, and the answer was in the file all along — under no name.

- [x] **The matrix is 34 × 12.** Delirium for the 19 later characters starts at **404**,
      Mother for the 14 originals at **423**, The Beast at **457**; **491** and **492** are
      those two bosses' kills. Nothing here was guessed: each base is pinned by three
      independent facts on the day a cell changed — an achievement whose *wiki* requirement
      is that boss unlocking the same day, the kill counter rising by exactly as many as the
      new marks, and index **188**, a bitmask of the characters that won the run, which names
      the row. Bethany, Jacob & Esau, T. Cain and T. Azazel each pin Delirium's base
      independently; Magdalene and Cain pin the other two.
- [x] **Two properties guard it**, in `crates/ipc/tests/marks_real.rs`. "No mark without a
      kill of that boss" holds across all 34 snapshots — but it survives a base off by one,
      so a second compares *identities*: when one mark appears and the winner mask names one
      character, they must be the same character. Moving the Mother base by one turns that
      red, which is how it was checked rather than assumed.
- [x] **What stays unread, and says so**: Mother and The Beast for The Forgotten and the 19
      — 40 cells, a 20 × 2 block in the bottom-right corner. The spacing puts them inside
      423..490, but they are zero in every save we hold, so `counter_index` returns `None`
      and the app draws *unknown*. One run of Mother with a Tainted character closes them.
      Matrix: 340 cells with 19 unknown → **408 with 40**, 166 marks started.
- [x] **The package and the brief follow**: twelve columns with their portraits and the two
      symbols that were the only ones left unassigned in `completion_widget` — the knife
      piece and Dad's Note. §5.3 of the brief loses a gap and rewrites another.

**And then B9, the archived analysis, on a source nobody had opened.** `online_logs\` holds
21 sessions, each with a `log.txt` and two profile snapshots — and the two are not one
profile before and after: one is ours, the other is **the co-op partner's, a second real
profile at 52→105 achievements**. A profile at the start of the progression is exactly what
those questions needed.

- [x] **Section 6 is Bosses, not cards and pills** — on the beginner profile 56 of 104 cells
      are set and the 48 that aren't are precisely the late and alt-path roster, Mother and
      The Beast included. `SaveDiff.cards_pills` has been naming boss encounters after cards.
- [x] **Section 3 is stages** — index 0 is zero in every save, which no table starting at
      Isaac could be, and in a matched window the cells that moved are exactly the stages
      the log declared with `Level::Init m_Stage`.
- [x] **Section 8 is cutscenes**, one cell short of certain: the log's cutscene 19 lands on
      index 19, while index 2 rises once per launch and needs one more solo run.
- [x] **Section 10's proposed split at byte 320 is wrong**: the payload is 20 zeros, five
      counters, then a sorted key → count list of 1,364 pairs that runs straight across byte
      320. The key decodes as `(type << 20) | (variant << 8) | subtype` — the bestiary,
      readable.

> Full evidence in `docs/BACKLOG.md` under B9, which now closes on one twenty-minute
> measurement plus the rename.

### 2026-09-08 (late) — C2: the icons leave the payload

The frontend's first real task, taken before any screen exists so the contract changes once.

- [x] **A URI scheme instead of a second command.** The C2 entry asked for `unlock` without
      `iconUrl` plus a command serving icons for a list of ids. That design moves a cache
      into the UI — what's visible, ask for those, keep them, don't ask twice, evict on
      scroll — which is TypeScript we would write and test, in the one place the project
      says receives only resolved JSON. Instead `crates/app` registers an **asynchronous
      URI scheme**: a row still carries `iconUrl`, now a short link
      (`isaac://achievement/19`), and the browser does the lazy loading, the caching and
      the de-duplication for free.
- [x] **Measured, both ends.** The motivation was a measurement, not a hunch: 226 KB of
      base64 inside a 240 KB twenty-node excerpt — **94% of the payload was pictures**. The
      outcome is a measurement too: `unlock` **7 MB → 415 KB**, `next_steps` **124 KB → 3
      KB**. That second one is the app's opening screen.
- [x] **The seam already existed.** All three views take an `icon` closure; only what it
      returns changed, from bytes to a URL. So `ipc` stays pure and platform-free, `app`
      builds the URL — it is the only place that knows Windows rewrites the scheme to
      `http://isaac.localhost/` — and `design-export` passes a closure that still inlines
      real images, because the package is opened from a folder where an `isaac://` link
      resolves to nothing.
- [x] **The reference is keyed on ids, not on the sprite path.** A path-keyed scheme would
      have been simpler and would have put a file path on the wire, which `CLAUDE.md`
      forbids. `IconRef::Item` also carries the **kind**, because 186 ids are shared between
      a collectible and a trinket: an id alone names two different pictures, and the wrong
      one would look perfectly plausible. Pinned by a test.
- [x] **`ipc::IconRef` round-trips.** `to_path` and `parse` live next to each other and are
      tested against each other, because a reference that renders to something the handler
      can't parse is an image that silently never appears — the exact failure `unpack`
      taught us to distrust, where a wrong path doesn't error, it goes quiet.
- [x] **The closing criterion is a test, not a claim** (`crates/ipc/tests/unlock_size.rs`):
      the real payload under a declared ceiling, **no `data:image` anywhere in it**, and at
      least one link actually present — because a ceiling alone would pass just as happily
      on a payload that had lost its icons altogether.
- [x] **The design package renamed its two `unlock` files, and they now tell the truth.**
      `unlock.without_icons.json` used to be "the form the real screen will use"; the real
      screen's form now *has* icons, they're just links. So: `unlock.json` is exactly what
      the app sends, and `unlock.illustrated.json` is twenty nodes with the pictures inlined
      for a human to look at. README regenerated from the exporter's own text.
- [x] **A real-data test kept its meaning instead of its assertion.** `graph_real.rs`
      checked that an icon started with `data:image/png` — "the sprite extracts from the
      real archives". The view no longer extracts, so the assertion became the link *plus* a
      direct `icon_source` + `ResourceSet::read` check. Dropping the second half would have
      left a test that passes on an install whose archives don't hold the sprite at all.

> **At release time:** `tauri.conf.json` carries `"csp": null`, so nothing blocks the scheme
> today. A real CSP must allow `img-src` from `isaac:` and `http://isaac.localhost`, or
> every icon disappears with no console error and no failing test. Written in the code, at
> the registration.

### 2026-09-08 (night) — the package regenerated, with the game back on disk

The game is installed again, so step 4c — the one thing that needed this machine — is
closed and the design package is current.

- [x] **The whole package re-exported and committed**: 8 archives, 19,473 entries, 5,837
      files, 36 MB. **Not one image byte moved** against the `a9a32ee` output: 12 files
      changed, 2 added, all of them contracts, payloads and the brief. Extracting the same
      archives twice gives the same bytes, which is what makes a committed package
      reviewable at all — the diff is only ever the contract.
- [x] **The queue payload shows all four states on real data**: 480 dragged in by 55
      (`wanted: false`, `origins: [55]`), 55 the wish whose chain is queued
      (`stepsNotQueued: 0`), 69 the wish standing alone (`stepsNotQueued: 1`, and `partial`
      into the bargain), plus a `completed` diagnostic for a wish already done. The fifth,
      `Unresolved`, can't be produced from a real catalog by definition.
- [x] **No path leaks in the package**: `setup_state.json` carries `<account>` and no
      Windows username anywhere in the 5,837 files.
- [x] **Four commits of work that was sitting uncommitted**: `GraphDeps` into `ipc` with
      its three tests, the queue payloads in `design-export`, the live probe, and the brief
      plus this document.

> `cargo test --workspace` cannot link `core-save`'s `live_probe` example while the probe
> is running — `LNK1104`, the exe is held open. The suite was verified with
> `--lib --tests --bins`: **zero failures, 7 skips**, all of them samples that exist on no
> machine any more. `scripts/check` will go green again once the probe is stopped.

### 2026-09-08 (evening, at the PC with the game) — what only this machine can answer

Worked from the PC that has Isaac installed, with the game **running** and a run in
progress. What follows could not have been produced anywhere else.

- [x] **The suite's real picture here: 644 real files touched, 15 skips** — against the
      *37 files and 90 skips* measured the same day on the other PC. That gap is the whole
      point of the "re-check on every machine" blocker: `samples/packed` is a live junction
      here, so `unpack`, `catalog`, `ipc` and `graph` all run on the real game.
- [x] **Skips 15 → 7, with a file nobody had collected.** `userdata\` holds a
      `rep_persistentgamedata1.dat` from **2024-06-06**, i.e. the pre-Repentance+ era, which
      `samples/` had none of. Added as `20240606.rep_persistentgamedata1.dat`, it switches on
      the 8 tests that were skipping everywhere with `no dated sample *.rep_…`. No
      cross-edition contamination: `SERIES` is a list and compares only within one.
      The 7 left are honest — `20250112` and `20240118` exist nowhere any more, and one
      snapshot can't be compared with itself.
- [x] **The 521-vs-523 counters failure did not reproduce** — but only because `20250112`,
      the file that causes it, isn't on this machine. Not fixed: out of reach.
- [x] **Fresh samples**: `20260906`, `20260907`, `20260908` copied from `save_backups\`,
      extending the historical series past the 09-05 it stopped at.

**A live probe, and the save turns out to be an event stream.** A throwaway
`core-save/examples/live_probe.rs` watched `log.txt` and both `.dat` files every half second
while a run was in progress, reporting each change through `core_save::diff` (evidence in
`samples/logs/probe-*.tsv`; the log snapshots in `samples/logs/`). What it found:

- **The game rewrites the persistent save every few seconds *during* a run**, not only on
  exit — dozens of writes observed inside one session, one or two counters at a time. The
  project assumes the `.dat` is a between-sessions snapshot; it isn't. A slice of what M4
  wanted from `log-watch` ("something changed, refresh") is available from a format we
  control, instead of from text patterns in a versioned rule file. The caveat belongs next
  to the finding: these are **lifetime** counters, not per-run — "what you have done", never
  "how this run went". That stays the log's job.
- **No torn read in dozens of concurrent reads.** Every `Save::parse` during a write
  succeeded, zero diagnostics. Not proof the write is atomic, but it's the first evidence we
  have on a question that was pure worry before.
- **`log.txt` reaches disk with no perceptible delay** — writes every 0.5–2s, 40 to 1200
  bytes at a time. That answers B8's second open question: a Live screen can be live.
- **The log records rooms**, `Room 1.1075(New Room)` with a frame counter on every
  transition, which answers B8's first question and unblocks anything map-shaped. It also
  carries `Spawn Entity` lines the five documented patterns never mentioned.
- **Zero mod noise on this machine** (no `Lua Debug` at all), unlike the 2024 log B8 was
  written from.

- [x] **B8 closed** — report in `docs/superpowers/reports/2026-09-08-b8-log-spike-report.md`,
      written from one complete run (Judas, hard, Mega Satan, won). Both open questions
      answered: **rooms are logged**, with a frame number on every transition, and the flush
      is immediate. Two findings change M4's shape — the log **announces every save write**
      (132 times in that run), and `from pool X` lies about the character's starting item,
      which only its **position** in the log distinguishes. M4's design can start.
- [x] **The game names the save's sections, and we had two of them wrong.** Loading a
      profile, the log prints `Reading chunk N` plus that chunk's **name**, for **eleven**
      chunks. Our table has ten sections with four marked "to be identified" — they have
      names now, and two labels we thought were settled are contradicted: `Kind::PerChar`
      (3) is **Level Counters**, `Kind::CardsPills` (6) is **Bosses** (104 cells against
      the catalog's 103 bosses), and section 10 holds **two** chunks — Special Seed Counters
      then Bestiary Counters — which is why the game counts eleven where the file has ten
      headers. The four positions we were sure of (1, 2, 4, 7) all agree, which is what
      makes the rest credible. Measured, not inferred: `Save::parse` finds ten headers with
      **zero diagnostics**, while section 10's header declares `count=80, f2=320` and the
      parser hands it **11,016 bytes**. Blast radius is small — `CardsPills` appears only in
      `SaveDiff`'s field name and two test assertions, `PerChar` only in tests, neither
      crosses the IPC — but `cards_pills` is a wrong name in a public type.
      **Logged as B9**, deliberately not fixed here: a log line is evidence about what the
      game thinks it reads, not a measurement of what the cells mean, and the rename has to
      follow a confirmation by content.
- [x] **Three cross-checks the run gave for free**: `unlock steam achievement '19'` states
      the section-1 `slot[id]` mapping in the game's own words, where we had inferred it
      from 169 of 171 items; `MARK/Mega Satan/Judas [119] 0 -> 3` confirms the REPENTOGON
      block table by *doing the thing and watching the right cell*; and, the run being hard
      mode, that `0 -> 3` shows **hard writes both of the mark's bits at once** — which
      `CLAUDE.md` had as an open question.
- [ ] **One discrepancy left open**: `CHARACTER_LAST_RUN_WIN [188] 4 -> 12` on a winning
      Judas run. Judas is id 3 and the old value 4 matches nothing played. Either the
      REPENTOGON label is wrong for that index or it isn't a character id. Closes by
      collecting wins with other characters.

### 2026-09-08 (later still) — the design package is a photograph of an era that ended

- [x] **The package is structurally complete and factually old.** 5835 files, atlases,
      sheets cut from the `.anm2`s, wiki samples, a README that declares its own three
      fallbacks — all fine. But the payloads are the output of `a9a32ee`, from before M2 and
      M3: 641 nodes at `"kind": "stub"`, `basis: stub`, `expansion: stub`.
- [x] **The file dates say the opposite of the truth.** `contracts/types.ts` has an mtime
      *newer* than the source it was copied from: git rewrote it at checkout. The era is
      readable in the content, never in the timestamp — 641 `stub` is a signature no mtime
      can fake.
- [x] **M3 is missing from the package entirely**, not stale: `payload.rs` never wrote a
      queue payload, so the one screen whose rule is genuinely hard to draw has nothing to
      draw from.
- [x] Regeneration written into the handoff entry as four steps, two of which need no
      sample data and can be done anywhere.

### 2026-09-08 (later) — the last three small ones, and one wasn't small

- [x] **The summary that says how much a run didn't verify was itself approximate.** The
      harness writes `test … ok` to stdout while `test-support` declares on stderr, `2>&1`
      merges them without respecting line boundaries, and tests inside a binary interleave
      anyway. Declarations are now mirrored into `ISAACDOME_TEST_DECLARATIONS`, which
      `scripts/check` counts from: **90 skips and 37 real files**, where the old count said
      81 and 35.
- [x] **The mirror took two goes, and the second is the lesson.** `writeln!` emits the text
      and the newline as **two** writes, so a second writer landing between them fuses two
      declarations into one line — which the first version of the fix promptly printed.
      One `write_all` with the newline already in the buffer, and
      `concurrent_appends_never_fuse_two_lines` reproduces it: 600 lines, three threads,
      all intact.
- [x] **`scripts/check`'s Italian identifiers**, a B7 leftover of the same kind as
      `crates/unpack`'s. Renamed while the file was open, in its own commit. `unpack`
      stays as B7 left it.
- [x] **The `discovery` fixture found a defect the entry didn't suspect.** The registered
      case — a leftover folder with no manifest — already behaved. Its mirror image did
      not: manifest present, folder gone, and `find_game` returned a directory that doesn't
      exist with **no diagnostic**. The same function checks exactly this one branch over,
      in the malformed-manifest fallback, which is what makes it an oversight rather than a
      choice. Now it skips that library and keeps looking, so a stale manifest on one drive
      can't hide the game on another.
- [x] **A guard written while it is still vacuous.** A `corrections.json` entry that
      matches nothing produces no error and no effect. `pageId` is empty today, so the
      check over the real file asserts nothing — which is precisely why it costs nothing to
      add now, and it ships with a second test that runs the same check against a file
      deliberately wrong in both ways, so the guard itself is known to work.
- [x] **Four times in one session, measuring first changed the job**: a red test hid 373
      others; a stray `}}` was splitting fifty lists; "the multi-line wrapper" turned out to
      be a contract decision; "entity aliases" turned out to be an argument-parsing bug. The
      name a backlog entry carries is the hypothesis of whoever wrote it, not a diagnosis.

### 2026-09-08 (later) — wiki parser: three fixes and two decisions handed back

- [x] **`||` is a cell separator *and* an empty template argument.** `build_table` split
      on the bare string, so Mystery Egg's `{{e|Mask + Heart||Heart}}` became two half
      templates. `split_cells` counts `{{…}}`/`[[…]]` depth and saturates at zero, so a
      malformed row degrades into one cell instead of none. Two entries, and thirteen
      lines changed in a twenty-two-megabyte dataset.
- [x] **A lone `}}` was doing more damage than anyone had written down.** The review
      recorded a junk `}}` paragraph. It also **cut the list in two** — reaching the
      paragraph branch is exactly what flushes an open list, so every `*` item after a
      `column list` wrapper started a fresh list. Fifty occurrences. Measuring before
      fixing is what turned "a stray text node" into "forty lists silently split".
- [x] **The raw-syntax threshold went 125 → 123 → 83**, and each drop matched the size its
      family predicted — which is the evidence the change hit that family and nothing
      else. What's left is 75 from the wrapper defect proper and 8 `<math>` formulas that
      are genuine text and will never go to zero.
- [x] **`#[serde(default)]` on a new diagnostic counter, and the test that demanded it.**
      `embedded_loads` went red the moment `Diagnostics` grew a field: a dataset built by
      an earlier parser has no such key, and refusing to load a whole snapshot over a
      missing counter is the opposite of degrading.
- [ ] **Handed back rather than decided: how to represent a template that wraps blocks.**
      `column list` (51) is pure layout and could be dropped; `{{bug|…}}` (4) is not, and
      the crate already models it specially when it fits on one line; `Book of … synergy`
      (7) sits in between. Expressing it needs a `Block` variant, and `Block` crosses the
      IPC — so it's the wiki screen's design decision, not something to settle from inside
      `blocks.rs`. Same rule that keeps C2 deferred.
- [x] **The dataset is rebuilt with `pnpm wiki:build`, offline from `dataset/raw/`.** The
      `derived` test went red the instant the parser changed, every time: that is the
      mechanism working, not an obstacle.
- [x] **"Entity aliases" was the wrong name for the last item, and listing the actual
      failures said so in a minute.** All 22 unresolved references come down to 7 distinct
      keys: three `{{i|1=Name}}` (MediaWiki's explicit positional syntax, which `assemble`
      filed under `named` leaving `args` empty — fixed, `unresolved` goes `{e:18, i:4}` →
      `{e:18, i:1}`); three id-less grid entities; and `Tonsil`, which must stay
      unresolved. The aliases were present and correct the whole time.
- [x] **`diagnostics_are_bounded` now asserts per template rather than one loose sum.**
      What's left is a floor, not a score, so the bound should say which family moved —
      and if the item count ever reaches zero that's a bug in the `Tonsil` handling, not
      progress.
- [ ] **A third thing pointing at the same decision**: the id-less buttons need a `Target`
      variant for grid entities, exactly as the block-wrapper needs a `Block` variant.
      Both cross the IPC, both belong to the wiki screen's design.

### 2026-09-08 (later) — the repository gets a licence

- [x] **GPL-3.0-only**, a decision and not a default. The repo declared nothing at all —
      no `LICENSE`, and no `license` field in any of the thirteen crates or either
      `package.json` — which is all rights reserved: no legal fork, no legal contribution,
      and M5 ships an installer. Copyleft over permissive because this is a tool built on
      top of a community's own wiki.
- [x] **Three regimes, not two.** The README described the game's assets and the dataset
      and said nothing about the code. Now: code GPL-3.0-only, `dataset/` keeping the
      CC BY-SA 4.0 it inherits from the wiki, game assets nobody's to license here and
      never shipped. A CC BY-SA dataset inside a GPL program is a collection, not a
      derivative of it — neither licence swallows the other.
- [x] **One declaration, not thirteen**: `[workspace.package]` plus
      `license.workspace = true`. Tauri already defaults its bundle licence to the one in
      `Cargo.toml`, so the config adds only what it can't infer — the copyright line and
      `licenseFile`, so the installer carries the text instead of naming it.
- [x] The licence text was downloaded verbatim from gnu.org and checked (674 lines, all 17
      sections, LF): a licence is the one file where writing it from memory is a defect.

### 2026-09-08 (later) — the test net was lying

Started as the smallest item on the "what can be done without game data" list and turned
into the answer to why that question had to be asked at all. Five commits, no production
code touched: the suite was reporting green on work it had stopped doing.

- [x] **The suite was red, and it had been red before for the same reason.**
      `counters_length_follows_the_header` asserted `declared == 523` **inside** the loop
      over the historical series. A series spans eras by construction, so the assertion
      only held while `samples/` happened to hold nothing but current-era saves. It was
      closed once already, in the entry above under the name
      `stable_section_counts_match_the_format`, by *swapping the fixture* — which treated
      the data and left the defect. The era values now live in an `ERAS` table, one row per
      file; the loop keeps only the property its own name promised.
      **A fix that has to be re-applied by changing data is a symptom fix.**
- [x] **A red test hides the rest of the suite.** `cargo test` stops at the first failing
      binary: `ipc`, `graph`, `plan`, `wiki` and `store` never ran at all. Visible tests
      went from 128 to **501** the moment the red was gone — the workspace was never
      broken, it was unexamined.
- [x] **Two of the most valuable properties had quietly stopped verifying anything.**
      `diff_reports_exactly_the_bits_that_turned_on` and `the_series_never_regresses`
      guarded on `saves.len() < 2` and returned. With one dated sample that fires every
      run: `dated_series` prints `sample:` for the file it found, `windows(2)` yields
      nothing, the harness reports a pass. **This is the `unpack` failure one level up** —
      `test-support` makes the helper declare its slice of the domain, and the test then
      narrowed to none without a word. `comparable_series` says it out loud, per suffix.
- [x] **The series constant was `rep+` only**, so the three dated Repentance saves already
      sitting in `samples/` were read by nothing. `SERIES` is now a list, one per edition,
      compared **inside** each and never across — a `rep_` and a `rep+` snapshot are
      different profiles, and end to end they'd read a change of profile as progress.
      Properties exercised for the first time on the 2024 era, all holding: ten sections in
      order with no diagnostics, the counter vector matching the header at 496 as well as
      at 521, and the counts called version-independent (14 characters, 733 items, 46
      challenges) staying put across the edition boundary — until now an assumption, never
      a measurement.
- [x] **Four tests were pinned to snapshots that exist nowhere** (`20250626`, `20260905`).
      Three never needed them: "the last section ends where the checksum begins" and "a
      save diffed against itself is empty" are properties of the format. The fourth,
      `achievement_count_is_read_from_file_not_hardcoded`, does need two eras that
      disagree — but *any* two do, and 638 → 641 asks the same question of files that are
      here. Pinning it to the newest jump had cost it its data.
- [x] **`ipc/cross_check.rs` was dead, and drifting while dead.** Its `SAMPLE` named a 2026
      file while its assertion still said "measured on the January 2025 sample": the file
      and its fixtures had come apart, and no run could notice, because a missing sample
      skips before reaching them. Pointed at the January 2025 profile it runs and passes —
      321 readable cells agreeing with `reference/isaac_save.py`, `started` at 93, exactly
      the numbers the comment claimed. **The one independent check we own was switched off
      and nobody could tell.**
- [x] **Measured, not asserted:** on that machine's four samples, reads of real files per
      run went from **6 to 37**, and all four saves are now used instead of two. The
      absolute numbers belong to that PC; what generalises is that the same files were
      being read six times fewer than they could be. Test *counts* show none of it — a test
      that skips still reports a pass, which is why 501 passed before and after.
- [ ] **`scripts/check`'s skip summary can't be trusted for counting.** With tests running
      in parallel the `skip:`/`sample:` lines interleave and split (`skip: ok`,
      `okconfig.a` appear in real output), so `grep -c` over them is noise. The kinds are
      still readable, the totals aren't. Fixing it means `--test-threads=1` for that run,
      or emitting the declarations somewhere other than shared stderr.
      **Closed the same day**, the second way: `test-support` mirrors every declaration
      into the file named by `ISAACDOME_TEST_DECLARATIONS`, which `scripts/check` sets and
      counts from. The true figures were 90 skips and 37 real files where the summary had
      been saying 81 and 35.
- [x] **Why the question "what can be done without game data" had an answer at all.** The
      machine in use had no game installed and no `samples/packed`, and its saves covered
      two eras but not the current one. **That is a fact about a PC, not about the
      project** — the work moves across more than one machine, `samples/` is git-ignored
      and the install isn't in the repo, so what a suite actually exercises has to be
      re-checked on each one rather than read off this document. The blocker above now says
      so, and says which commands answer it.
- [ ] **Not done, on purpose**: `ipc/graph_real.rs` and `catalog/tests/real_data.rs` still
      name a `live.*` sample that no longer exists. Both skip on `samples/packed` first, so
      any change there is unverifiable here — it waits for the game.

### 2026-09-08 — the plan queue

New pure crate `crates/plan` (26 tests), `store` migration 2, the `ipc` view-model, five
Tauri commands and the TypeScript mirror. Spec
`docs/superpowers/specs/2026-09-07-plan-queue-design.md`, report
`docs/superpowers/reports/2026-09-07-plan-queue-report.md`.

- [x] **The order is an array, not a `seq` column** — the user's own proposal, and it
      removes a class of bug: a set of sequence numbers can contradict itself, a position
      cannot. A move is one write.
- [x] **The move rule is asymmetric, and the spec had it wrong.** Dependents are dragged;
      prerequisites are a **wall, not cargo** — they never move, they only stop the row
      from rising. Gathering them into a block, as the spec first said, reorders rows the
      user had arranged by hand. Caught by a test on a three-row queue.
- [x] **`wanted` and `origins` are independent**, and a row that is neither is an orphan.
      That single line is the whole removal rule: two wishes sharing a step keep it alive.
- [x] **The unlock graph is almost flat**, and this is the finding that should reach the
      design: across the whole historical series, the deepest missing chain is **3, for one
      node**, and 2 for one more. Everything else is zero or one step. Auto-filling a wish's
      chain will nearly always add zero or one row, so the queue's value is ordering many
      wishes, not unrolling deep trees.
- [x] **A read never writes**: the goals import is its own command, and migration 2 seeds
      nothing — `store` has no catalog and cannot resolve a target to an achievement.
- [ ] **Not done**: the screen. This is the backend and the contract; drag-and-drop and what
      a row looks like belong to the design system.

### 2026-09-07 — M2: the unlock graph

New pure crate `crates/graph`, 42 tests. Full report in
`docs/superpowers/reports/2026-09-07-unlock-graph-report.md`; spec in
`docs/superpowers/specs/2026-09-07-unlock-graph-design.md`. Branch
`feature/unlock-graph`, 13 commits, `pnpm check` green with one pre-existing skip
(the Python cross-check, on a machine with no Python).

- [x] **The backlog's premise (B4) was wrong, and measuring said so before any code.**
      B4 assumed M2 meant parsing the 283 English condition comments out of
      `achievements.xml`. The wiki dataset we already ship carries `requirements` for
      **641 of 641 achievements** as typed refs — the parsing was already done by
      `crates/wiki`. What M2 actually owed was the semantic step and the recursion.
- [x] **The two sources are split, and the split is the design.** *What is needed* comes
      from the wiki's refs; *who unlocks what* comes from the game's own files through
      `catalog`. The graph never invents an edge from the wiki — it reads it from the
      user's installation, with their edition and their DLC.
- [x] **Entity ids are not boss ids.** Gish is entity 43 and boss 19; the bridge is the
      **name**, and it carries 468 of 488 entity refs. Six of the seven wiki/game
      divergences are exactly this, and they are now a permanent test.
- [x] **Result**: 637 nodes, 459 edges, **1,215 of 1,247 requirements resolved (97.4%)**,
      and the 32 that aren't are eight labels each carrying an explicit `unknown` verdict.
      Nothing is uninterpreted by inattention.
- [x] **A fourth verdict, found in curation**: `unknown { reason }` — *judged, and the
      answer is that the model can't say it*. `transformation:Guppy` is genuinely gated,
      but by three items rather than one achievement, and the other three verdicts would
      each have made it lie.
- [x] **`GraphInfo::Stub` left the wire**, with `StepsBasis::Stub`. `PlanExpansion::Stub`
      stays: that's M3.
- [x] **Next Steps changed meaning**: from "the first five not-done in slot order" to
      "what is unlockable now, most fan-out first". Without a catalog it is now **empty**,
      and the view's `NoCatalog` diagnostic says why — a blocked node isn't a step, and a
      node the graph can't vouch for isn't either.
- [x] **The historical series earned its keep**: 31 saves, 302 → 384 achievements done,
      and two properties that hold across every consecutive pair.
- [ ] **Not done**: regenerating `design-export/isaacdome-design-pack/` (the generator now
      puts real graph data in it, so the checked-in copy is a version behind).

### 2026-09-07 — B7 follow-up: the leftovers

The B7 pass below translated the prose but stopped at the edge of the fenced code blocks,
and it never regenerated the design package. Both closed here, plus the source strings B7
had deliberately left alone.

- [x] **Italian file names renamed**: `docs/STATO.md` → `docs/STATUS.md`,
      `docs/MIGLIORIE.md` → `docs/IMPROVEMENTS.md`,
      `…/2026-09-05-b1-fonti-effetti-report.md` → `…-b1-sources-effects-report.md`,
      with the references updated in 19 files, `CLAUDE.md` included.
- [x] **`design-export/isaacdome-design-pack/` realigned.** It was output committed before
      the translation: the generator already copied the English `DESIGN-BRIEF.md` and wrote
      an English `README.md`, but the checked-in pack still held the Italian brief and a
      `LEGGIMI.md` the code no longer produces. Rather than rerun the tool (it needs the
      game installed and rewrites 5,833 images), `documents()` was replicated by hand: the
      three copied files refreshed, `README.md` written from the generator's own
      `readme()` with `{images}` = 5833, `LEGGIMI.md` deleted. The count was confirmed from
      two sides — `INDEX.json` has 5,833 rows and the stale `LEGGIMI.md` printed 5,833 — so
      the result is what a regeneration would produce for those documents.
- [x] **The code blocks inside the plans and specs**, ~250 lines: Rust doc comments,
      `assert!` messages, `git commit -m` messages, Cargo `description` fields shown in
      snippets. The plans' own instruction "messages in Italian" was flipped to English in
      the four files that carried it; the real history has been English and squashed since
      the initial commits, so nothing points at a live commit any more.
- [x] **Italian identifiers in Rust**, which no earlier pass had looked for:
      `crates/ipc/tests/target_sprite_real.rs` (`Conteggio`, `copertura_delle_pagine`,
      `famiglia`, `reale`, …), `crates/unpack/src/arch.rs` (`tabella`, `leggibili`,
      `disponibili`), `crates/ipc/src/target_sprite.rs` (`chiave`, `resto`, `tipo`,
      `variante`), and `ui/scripts/scan-conventions.mjs` (`ECCEZIONI` → `EXEMPTIONS`,
      `motivo` → `reason`, `esente` → `isExempt`, with `CLAUDE.md`,
      `docs/frontend-conventions.md` and `docs/IMPROVEMENTS.md` updated to match).
- [x] **The nine `Cargo.toml` `description` fields** and the whole of `crates/wiki/build.rs`,
      which the B7 pass had skipped entirely.
- [x] **The user-facing strings**: `describe_open_error` and `store_reason` in
      `crates/app/src/lib.rs`, the expectations pinning them in `crates/ipc/tests/graph.rs`,
      and every visible string in `ui/src/App.vue` and comment in `ui/src/assets/main.css`.

### 2026-09-07 — B7: the repo's prose moves to English

Backlog task B7 (registered 2026-09-06) executed: comments, assert/panic/eprintln
messages, and every `.md` file translated from Italian to English. Identifiers were
already English; the app's own UI text is untouched (that's i18n, a separate concern).
Heavily parallelized across dozens of subagents, one per crate and per doc group, given
the size (2,100+ comment lines across 116 Rust files, ~30 `.md` files).

- [x] **`CLAUDE.md` went first**, by hand, not delegated: it governs how work gets done
      here, so the commit-language rule itself had to flip before anything else started,
      per the backlog's own note ("otherwise the repo fills up with two conventions at
      once").
- [x] **`docs/progetto.html` rewritten as `docs/PROJECT.md`**, translated. It loses the
      CSS/styling on purpose — the user asked for a plain Markdown document, not a ported
      stylesheet.
- [x] **`docs/convenzioni-frontend.md` → `docs/frontend-conventions.md`**, and
      `design-export`'s generated `LEGGIMI.md` → `README.md` in the generator source. The
      checked-in `design-export/isaacdome-design-pack/` copy stayed stale until 2026-09-07,
      when the four documents `documents()` copies were brought back in line by hand.
- [x] **Every crate** (`app`, `catalog`, `core-save`, `design-export`, `discovery`, `ipc`,
      `store`, `test-support`, `unpack`, `wiki`, `wiki-snapshot`), `ui/`, `reference/`, and
      the dev-tool shell scripts, translated. `cargo fmt`, `cargo clippy --all-targets -D
      warnings`, and `cargo test --workspace -- --nocapture` all clean, one pre-existing
      failure aside (below). `pnpm typecheck`, `lint`, `format:check`, `scan` all clean too.
- [x] **What deliberately stayed in Italian at the time**: user-facing diagnostic text the
      app returns pending real i18n (`store_reason`, `describe_open_error`, `ui/src/App.vue`'s
      verification-screen copy) and test *input data* chosen to look like garbage on purpose.
      **Reversed on 2026-09-07**, on the user's explicit call: none of that text comes from
      the game, so it is ours to translate. The App.vue exemption in `scan-conventions.mjs`
      stands — it bans visible strings in a template, regardless of their language.
- [x] **Rust identifiers already in English almost everywhere — except `unpack`**, found
      while translating: `disponibili`, `leggibili`, `tabella`, `PRECEDENZA`, `RADICI`,
      `VIVI`, `PICCO`, `SOGLIA_APERTURA`, `controllati`, `mancanti` are Italian variable/
      constant names in that crate. Out of scope for B7 (comments and docs only, never
      identifiers) — worth its own backlog entry if it's worth fixing.
- [ ] **One pre-existing test failure, unrelated to this task**:
      `core_save::real_saves::counters_length_follows_the_header` expects every sample in
      the historical series to declare 523 counters, but the local
      `20250112.rep+persistentgamedata1.dat` declares 521. Confirmed via diff that neither
      the assertion nor the `523` literal were touched by the translation pass. Notably,
      this exact pairing (521 vs. 523, same file) was already investigated on 2026-09-02
      and logged there as "closed: it was a mislabeled fixture, not a bug" — it's back,
      which means either the fixture regressed or that resolution didn't stick. Needs a
      fresh look, not a fix from this session.
- [ ] **Not done**: regenerating `design-export/isaacdome-design-pack/` (needs the game
      installed); nothing else known to be missing — a full sweep for stray Italian across
      every translated file, minus the deliberate exceptions above, comes back clean.

### 2026-09-06 — the illustrated package: everything in the game gets its picture

Second half of the same day. The package grows from 423 to 5,835 files and from "almost
everything" to **everything**, with the gaps closed using art from the game itself and
flagged wherever it's a fallback.

- [x] **`catalog::anm2`**: the game's `.anm2` files say where to cut its sheets, and every
      frame carries the rectangle it draws. Three different shapes (`minimap_icons` with the
      name on the animation, `hudstats` with the frames, `completion_widget` with layers) read by a
      single structure. From this, **48 sheets cut into 3,759 pieces**: the mark symbols, 24
      hearts, 77 minimap icons, the HUD stats.
- [x] **A defect that silently produced wrong data**: every layer declares its own
      `SpritesheetId`, but the cutting always used the file's first sheet. `leaderboardmenu`
      has four, and those pieces came out of the wrong sheet — plausible-looking images taken
      from the wrong place. Pinned by a test.
- [x] **Full coverage**: 909 items, 637 achievements, **103 of 103 bosses**, 41
      characters (+ 37 cropped head icons), **45 of 45 challenges**, 37 milestones. The Beast and
      Cadavra fall back to other game art because their portraits don't resolve; the
      six challenges with no reward take the achievement that unlocks them instead. Every fallback
      is flagged in the index.
- [x] **An item's key is `(type, id)`**, found while regenerating from scratch: 186 ids are
      shared between a collectible and a trinket — item 1 is both *The Sad Onion*
      and *Swallowed Penny*. The index was dropping the type, and a join on id alone would have
      grabbed the wrong sprite half the time.
- [x] ~~**Delirium isn't a symbol, it's the background.**~~ It looked like a gap — eleven
      layers and none called Delirium — and the conclusion was that the mark is the cell's
      **bloodied sheet**. With Delirium out of the symbols, the last pairing (`Cross` → The
      Lamb) closed by elimination. `symbolSource` in `marks.json` says for every row whether
      it's read from the file, referenced, or inferred.
      > **Wrong, corrected on 2026-09-08.** Delirium's mark is a small face with two eyes,
      > and the game names it — in **another file**: `main menu/onlinelobby.anm2`, layer
      > `Completion_Delirium`, the only anm2 that names all twelve. Taking the mark does
      > bloody the sheet, so the observation was real; the inference from it wasn't. It now
      > ships as the row's `symbolFallback`, with the face as the symbol.
      > A second wrong answer along the way is worth recording too: `completion_widget`'s
      > glyph row holds twelve cells and the layers claim eleven, so the leftover at x = 96
      > *looked* like Delirium by elimination. It isn't — it's a drawing we still can't
      > name. Elimination inside one file proves nothing when the answer is in another.
- [x] **Language convention put back in order**: `design-export` was the only crate with
      identifiers and JSON keys in Italian. Now, everywhere, it's **English identifiers,
      Italian prose**; the package uses `images/`, `sheets/`, `data/`, `INDEX.json`.
- [x] **The package is committed to `develop`**, tracked on purpose: design opens it
      from a fixed path. Found while doing this that `design-export/` in `.gitignore`
      wasn't anchored to the root and was keeping the **entire crate**
      `crates/design-export/` out of the repo too, even though `Cargo.lock` referenced it: a fresh clone
      didn't build.

### 2026-09-06 — the design package: two sections, and the images on the wiki pages

The material for the design handoff (step 4 of the path) is ready: the brief rewritten and
a 10 MB ZIP with real data and images. Three pieces of code and a rewrite.

- [x] **The app splits into two sections, Wiki and Progress**, and the split lives in the data:
      the wiki dataset is compiled into the binary, so Wiki works with no game
      installed and no save selected. This surfaced something the brief never mentioned
      anywhere: **on a machine with no Isaac the app isn't empty**. Profile selection
      stops being a separate screen and becomes what Progress shows until there is a
      choice. A single mixed tab bar, search above both, Settings as the page for
      data provenance.
- [x] **`ipc::target_sprite`**: a wiki reference finds its image in the game.
      **1,690 pages out of 1,727** — 719/719 items, 188/188 trinkets, 32/32 characters,
      637/641 achievements, 39/45 challenges, 75/102 bosses. The non-obvious piece is bosses:
      `bossportraits.xml` writes the portrait as `Portrait_20.0_Monstro.png`, i.e. **the entity's
      type and variant are in the file name**, and it's the same key the wiki uses
      for its `{{e|…}}`. Challenges have no in-game art (`gfx/challenge` doesn't exist):
      the only image belonging to a challenge is the achievement that rewards it.
- [x] **Found and closed a personal-data leak on the IPC.** `redacted_path` masked
      the Steam account id but **not the Windows username**: a candidate under Documents came
      out as `C:\Users\<name>\...`, and `rootHint`/`dirHint` were full paths. Same
      rule from `CLAUDE.md` already applied to the account id. The existing test —
      `documents_source_does_not_redact_path` — **encoded the defect as the expectation**:
      rewritten from the spec.
- [x] **`crates/design-export`**, a tool modeled on `wiki-snapshot` (`pnpm design:export`):
      2,035 images extracted with their index (family, id, name, **real dimensions**), the
      real IPC payloads of every command, ten sample wiki pages already paired with their
      image. `unlock` comes out as two files — one with no icons, the shape the real
      screen will use, and one with twenty illustrated nodes — because the package must
      neither hide nor reproduce defect C2.
- [x] **C2 stopped being an isolated defect.** To draw the little icons inside the
      wiki text a command is needed that returns images for a list of references: it's
      the same shape needed to strip the base64 icons out of `unlock`'s 641 rows. One
      command serves both the Wiki and the Unlock grid.

### 2026-09-06 — the completed improvements: no CI, local verification, streaming archives

Twelve of the fourteen tasks in `docs/IMPROVEMENTS.md` closed (A1 disappeared along with CI).
Still open: **B2** (generating `types.ts` from the Rust types, when the webapp starts), and **C2** is
deliberately postponed:
it changes the IPC contract, so it happens once the real frontend begins. The frontend is
frozen by choice: `ui/` only holds the verification page, and the real work starts
after the design handoff.

- [x] **No CI**, decided this session. It's not a gap: the list of checks lives
      in `scripts/check` (also `pnpm check`) and the two fast ones run via the
      pre-commit hook in `scripts/git-hooks/`, installed with `core.hooksPath`. README, `CLAUDE.md`
      and the hook all point to the same script.
- [x] **`cargo test` was hiding `skip:` lines.** The harness captures the output of passing
      tests: the advice "after `cargo test` look for those lines," written in two documents,
      could never have worked. `-- --nocapture` is needed, and `scripts/check` does that and counts them.
- [x] **Historical series in `samples/`**: 33 snapshots of the same profile from 2025-06-26
      to 2026-09-05, plus the `test-support` crate, the only way a test touches
      `samples/`, and it always declares which file it used.
- [x] **Two bugs found by tests that used to skip**: the Microsoft Store alias for
      `python` mistaken for a broken reference, and the 641 → 642 slot boundary in the diff.
- [x] **Streaming archives (C1)**: from ~1.3 GB down to a 1.4 MB peak to open them all, with the
      measurement done by a global allocator in a test.
- [x] **Conventions scanner completed**, and rule 5 applied to wire types too.
      It's the only work touching `ui/` — closed before the decision to freeze the
      frontend, and with no effect on the wire: the serialized strings stay the same.

### 2026-09-06 — two new product requirements: global search and multiple tabs

A documents-only session, no code. Two explicit requests, logged as
**B5** and **B6** in `docs/BACKLOG.md`; both start after the design handoff, like B3.

- [x] **B5 — global search.** Search across everything the app knows: catalog, wiki
      section text, unlock-tree nodes, screens. Two surfaces, a `Ctrl+K` palette and a
      dedicated screen. Most of the material already exists today, and **a result's
      identity already exists**: it's the `Target` the wiki uses for its references, so
      opening a result is the same action as following a link. Missing: an index, an IPC command and
      the two views. To measure before deciding: whether full-text search over ~920 wiki
      pages needs FTS5 in `store` or a plain scan is enough.
- [x] **B6 — multiple tabs and session restore.** The shell behaves like a browser, with
      a global *keep tabs saved on close* setting. A tab is a **location**
      (even an item's detail view or a wiki page, not just a section) and it only saves
      the view's identity, never its content: on restore it reloads from the backend. The active
      profile **stays global**, not per-tab, otherwise the promise in brief §4.1 breaks.
      Recommended persistence: the flag in `settings.json`, the tab list in a
      `store` migration — this would be the third, after goals (1) and snapshots (2).
- [x] Effects on documents: `DESIGN-BRIEF.md` gains **§4.2** (the shell: tabs and search),
      *Search* as an eighth screen, two new states in §10 (no search results, an orphaned
      restored tab), three vocabulary entries and a fifth question in §13 on how the tab
      bar, profile indicator and search entry point coexist in the same strip.
      `docs/PROJECT.md` and `README.md` kept in sync. **No IPC contract changes.**

### 2026-09-05 (night) — B1, sources for item effects

- [x] **Backlog B1 closed**, an analysis task with no code. Report —
      `docs/superpowers/reports/2026-09-05-b1-sources-effects-report.md`. The original entry compared
      against the fandom wiki (CC BY-NC-SA): that's the **abandoned copy** since the 2023
      migration. The living wiki is **wiki.gg**, CC BY-SA 4.0, already cited in `PROJECT.md`
      for the graph; the NC clause no longer applies to us. MediaWiki API with no credentials and **Cargo
      tables** (`collectible`, `trinket`, `achievement`, `entity`, `version`) that resolve
      template links into ids. Coverage measured by id: 719 of 721 items (missing two
      internal variants, 59 and 656), 188 of 188 trinkets; Effects/Notes/Synergies/Interactions/Bugs
      sections consistently present across 723 pages. EID covers 100% even in Italian but **has no license**:
      usable only with the author's consent. Italian fandom wiki: empty skeletons, discarded.
- [x] Decision: dataset **built at compile time and shipped in the package** (explicit
      request), one identified, sequential pass per release, never a per-user download;
      optional update from GitHub. Freshness: snapshot date checked against the `appmanifest`'s
      `LastUpdated`, which `discovery` already opens. Implementation waits for design
      (it's a screen), like B3.
- [x] ~~Side finding: **the repository declares no license** (no `LICENSE` file).~~
      **Closed on 2026-09-08: GPL-3.0-only**, decided rather than defaulted — a fork of a
      tool built on a community's wiki shouldn't be able to close and sell it back. The
      gap was wider than "no `LICENSE`": no `license` field in any of the thirteen crates
      or either `package.json` either. Declared once in `[workspace.package]`, inherited
      everywhere; Tauri reads the bundle licence from `Cargo.toml` on its own, so the
      config only adds the copyright line and the path to the text. The README now
      separates three regimes where it described two — code GPL-3.0, dataset CC BY-SA 4.0
      inherited from the wiki, game assets neither ours nor shipped.

### 2026-09-05 (night) — wiki dataset

- [x] **Implementation of B1 closed**, TDD plan in 12 tasks, all passed review.
      Full report — `docs/superpowers/reports/2026-09-05-wiki-dataset-report.md`. Born: the
      pure crate `wiki` (tree types, wikitext parser, `Resolver`, `build`,
      `Dataset::embedded()`) and the tool `wiki-snapshot` (`fetch`/`build`, the only one talking to the
      network, dependencies only `wiki` and `ureq`).
- [x] Two facts discovered on the first real snapshot, not anticipated by the spec: *Tonsil* isn't
      a wiki error but a page with two infoboxes from different editions
      (filtered on the `dlc` bitmask, not an id correction — also fixed in the B1 report); the
      character ids in the wiki's infoboxes are unreliable (Isaac has Keeper's id,
      Magdalene has Cain's), resolved instead by our own map in
      `dataset/corrections.json` derived from `players.xml`.
- [x] `wiki.json` (21,986,302 bytes, pretty, kept in the repo for a readable diff) is embedded in the
      binary **compressed** (`miniz_oxide` in `build.rs`, 1,585,198 bytes, 14:1 ratio) behind
      the `embedded` cargo feature; a derived test enforces `wiki.json == build(raw/)`.
- [x] `ipc::wiki` (`WikiInfo`, `wiki_entry` command), `discovery::GameInstall.updated_unix`
      for freshness, TypeScript types and a "Wiki" panel in the verification screen with
      recursive components (`WikiInline.vue`, `WikiBlocks.vue`).
- [x] **Result**: 719 items, 188 trinkets, 641 achievements, 102 bosses, 45 challenges,
      32 characters; 22 unresolved references (514 before the character map,
      20 before the final fix wave); 2 pages with no id. Two rounds of fixes to the tool
      (unreviewed pages re-listed by the server in truncated batches) and one parser bug
      found and fixed, not papered over (ids with a platform note read via
      `trim().parse()`, 27 achievements discarded).
- [x] **The final review of the whole branch found exactly one thing, and it was big**: the
      recursion into unknown templates had been *declared done in the documents* without the
      code actually existing, and the derived output still had 314 raw `{{…}}` fragments as
      plain text. Closed by fix wave `9ec171c`: `parse_inline` now recurses into the first argument of any
      unknown template and of `{{bug|…}}` (capped at 8 levels), with a real-data test
      that forbids raw braces in text nodes. Two families of cases remain, listed
      in the `wiki` module section above.
- [x] `docs/STATUS.md` (this entry and the `wiki` module in "M1 — detail"), `BACKLOG.md`,
      `CLAUDE.md`, `DESIGN-BRIEF.md` (new §8) and the spec kept in sync. `README.md` wasn't
      touched by the plan's tasks; its rewrite, independent of this cycle, landed on the branch
      as a separate commit.

### 2026-09-05 (evening) — B2, a challenge's reward

- [x] **Backlog B2 closed**, a bounded task: design in chat, TDD, no spec. The
      original entry's pattern (`Beat Challenge #N`) covered 20 of 39 challenges: the
      Repentance+ file writes the reward in **three forms** (hash-sign comment, comment with
      the name in parentheses, `steam_description` attribute with no comment), and challenges 31–35 and 45
      don't write it at all. The entry's "30 in the base file" count was wrong: it's 20.
- [x] `catalog`: `reward::challenge_beaten` (a strict parser, no `regex`),
      `Achievement.steam_description`, `Challenge.rewards` filled in by `Catalog::build`,
      `Diagnostic::RewardForUnknownChallenge`. Real tests: 39 challenges with a reward, 6 without,
      fixed points 19→62, 36→517, 44→533.
- [x] `ipc`: `UnlockTarget::Challenge { id, name, rewards: Vec<u32> }`, achievement ids
      and not views (the type would otherwise be recursive); `key()` drops it as
      name and icon. `ui/src/lib/ipc/types.ts` kept in sync. JSON pin in `goals.rs`, a synthetic case
      in `graph.rs`, challenge 1 → 89 in `graph_real.rs`.
- [x] Full suite: **253 tests, 0 failed** (`catalog` 94, `ipc` 84), no skips on real
      data; `cargo fmt --check`, `clippy -D warnings`, `pnpm typecheck`, `lint`, `scan`,
      `format:check` all clean.

### 2026-09-05

- [x] **Final review of `catalog` plan B closed** (`f8e971b..6de9cdb`):
      `catalog::Origin` in place of `Edition` (which in `discovery` means the installed
      edition), a test on `items.xml`'s winner containing every id from the base version, polish;
      spec, report, status and brief kept in sync.
- [x] **Spec and plan for the graph contracts** (`22746dc`, `f8dd870`, `555e3b4`): the three
      preliminary decisions made on real data with a probe on the profile (`slot[id]`
      mapping: 169 of 171 versus 145 and 142 for the alternatives); `store` is born with
      goals, not with M4.
- [x] **Graph contracts plan executed** (9 tasks, commits `be8ba80..a439a8a`):
      reverse index in `catalog`; `Goal`/`GoalId`/`UnlockTarget` and the `graph` module in
      `ipc`; `store` crate with migration 1; real tests on the mapping; five commands in
      `app` with `Store` in managed state; TypeScript types and wrapper, and a verification
      screen with Unlock and Next Steps. Report —
      `docs/superpowers/reports/2026-09-05-graph-contracts-report.md`.
- [x] **A number in the plan corrected by execution, not adjusted after the fact**: the slots unknown
      to the catalog are **4**, not 5 (`c1d492e`, `814e7cb`): the 5 came from 642 − 637 and counted
      slot 0, which isn't a node. The test was stopped and investigated before touching the
      fixture.
- [x] **The plan degrades instead of failing** (`2724acd`, `591f6c2`, `9db537b`): a `store`
      row with an unreadable target doesn't wipe out the goals but reaches the UI by id;
      a database that won't open arrives as a diagnostic with a reason, and `storeAvailable` is
      derived from a single source.
- [x] `docs/STATUS.md`, `DESIGN-BRIEF.md` (new §7 with the contracts, sections renumbered),
      `CLAUDE.md` and `README.md` kept in sync: `store` is born, the structural base is closed.
- [x] **Final review of the whole branch and a fix round** (`c5c2ef7..`, six commits in two
      rounds): seven findings accepted in the first round. The two that mattered: `Goal` was persisting the whole `UnlockTarget`
      (name and base64 icon in the database, and a new field would have invalidated every row) →
      `TargetKey` is born, the on-disk format, and `GoalView` resolves name and icon on every
      read; `achievement_flags` flattened an unreadable section 1 to an empty vector,
      making `unlock_view` declare a false `CatalogBeyondSlots { 638 }` → `flags` is now
      `Option` and a missing section declares itself. Also: fieldless enums are bare
      strings on the IPC (including `ItemKindView`, with the rule now in `CLAUDE.md`), `plan` degrades
      on a failed query too, `add_goal` won't overwrite an already-present id, inverse
      `kind_view`/`item_kind` and the shape of `PlanExpansion::Computed` pinned.
      **Round 2**: the new rule had three counterexamples already in-house (`StepsBasis`,
      `CandidateSource`, `MissingReason`, tagged despite having no fields, the first one documented
      as an object two sections after the brief had proclaimed the rule). All three converted,
      with the JSON pins widened — `CandidateSource` hadn't been pinned anywhere at all.
      One rule, zero exceptions.
      Full suite: **242 tests, 0 failed** across 39 binaries (`cargo test --workspace`;
      `catalog` 84, `ipc` 83, `unpack` 26, `core-save` 21, `discovery` 15, `store` 8,
      `app` 5), `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` clean,
      `pnpm typecheck`, `lint`, `scan` and `format:check` clean.

### 2026-09-04/05

- [x] **`catalog` plan B closed**: `metadata.rs` (quality and tags from `items_metadata.xml`),
      `achievements.rs` (text, unlock condition from the comment, links to items),
      `itempools.rs` (`Item.pools`), `challenges.rs` and `bossportraits.rs`, `origin.rs` (source
      DLC), cross-check against the save. Commits `e4866a7..36b395a`.
- [x] **Real bug found by the real-data test**: `challenges.rs` was dropping three challenges out of
      45 due to a different separator (space instead of comma) and negative ids in
      `startingitems` (the game's own convention for "this is a trinket"); fixed.
- [x] **Four numbers in the brief corrected** after checking against the real parser (not `grep`
      over the text): quality −1 (2 → 0, they were `craftquality`), pool orphans (26 → 24),
      id gaps (9 → 11), multi-achievement challenges (13 → 14) — the same methodological error
      as plan A, which had propagated into the spec (`7c0798a`, `b534bd7`).
- [x] TDD plan (B) and report — `docs/superpowers/plans/archive/2026-09-04-catalog-b.md`,
      `docs/superpowers/reports/2026-09-04-catalog-b-report.md`. Full suite: **196 tests,
      0 failed** (`cargo test --workspace`).

### 2026-09-04

- [x] **Fixed the design-handoff point** ("Handoff to design" section): structural
      base = normalized static data + fixed IPC contracts, not "all real data". Four-step
      path: `catalog` plan B → M2/M3 contracts with stubs →
      brief aligned → handoff. M2 then proceeds in parallel with the webapp.
- [x] `DESIGN-BRIEF.md`: Collection status light from "names for 1 in 4 entries" to "909 of 909";
      removed the line about the "two unnamed entries"; added character head icons as the
      matrix row label; header and §11 aligned with the handoff path.
- [x] `CLAUDE.md` and `README.md`: `catalog` is no longer "stalled until there's XML to
      work with"; M1 framing updated.

### 2026-09-03

- [x] `pnpm install` in `ui/` (had never been run: `node_modules` was missing).
      Node 22.22.0 and pnpm 10.33.0 were already present — the "update Node" TODO is closed.
- [x] **Fixed `crates/app/tauri.conf.json`**: `beforeDevCommand` pointed to `../../ui`, but
      Tauri runs that command from `tauri.conf.json`'s *parent folder* (`crates/`), not
      from the config's own folder. It resolved to `C:\Projects\ui` and startup failed immediately.
      Fixed to `../ui`, for both `beforeDevCommand` and `beforeBuildCommand`.
- [x] **First run of the app.** Verified by hand on a real window: discovery, profile
      selection, summary of the 10 sections and the marks matrix, all on real data.
- [x] Verified the game is installed under `D:\SteamLibrary` → closed the M0/M1 blocker on
      its absence.
- [x] Extracted the base edition's XML catalogs from `config.a` into `samples/catalog/`.
- [x] **Discovered that `unpack` decompresses none of the content archives** (0 of 1,873).
      Opened as the main blocker; `unpack` goes back from ✅ to ⚠️.
- [x] `DESIGN-BRIEF.md` updated for Claude Design: real TypeScript contracts instead
      of the JSON sketch, values read from the app, catalog coverage, and an inventory
      of graphic assets with a note that none can be extracted yet.
- [x] Suite verified: **94 tests, 0 failed**, `clippy -D warnings` clean.
- [x] **Resolved the same day.** Cause: byte `0x07` is the compression mode, not a
      version. Added `MiniZ` and `Bogocrypt1`, with dispatch. **14,751 resources extracted,
      0 failed.** Suite from 94 to **99 tests**, `fmt` and `clippy -D warnings` clean.
- [x] `samples/packed` is now a **junction** to the game's `resources/packed` folder:
      the mode tests run on real archives without bringing them into the repo, and skip
      declaring so if the junction is missing.
- [x] Extracted ten real sprites into `samples/sprites/` for design, with real dimensions
      (item icon 32×32, boss portrait 192×192, achievement icon 263×176 **not square**,
      character sheet 512×512, `completion_widget.png` 384×384).
- [x] `DESIGN-BRIEF.md` §5.6 rewritten: from "0 extractable" to an inventory, real dimensions and
      the note that `completion_widget.png` already contains the game's own visual encoding of marks.
- [x] Saved `ui/src/assets/logo.svg` (dome-dog), verified it renders.
- [ ] **Fix the `unpack` spec**: it still states `0x07 u8 version = 0x01`.
- [x] **Found Repentance's root**: `resources-dlc3/`. Index coverage from 92.5%
      to **97.1%** (18,914 of 19,473 entries).
- [x] **Verified the coverage that actually matters**, the catalog's: 909 items
      (425 passive, 170 active, 126 familiars, 188 trinkets), **all with sprites, 0 missing**;
      641 achievements → **641 icons, 0 missing**; 41 characters → **41 portraits**;
      103 bosses → 101 portraits (missing *The Beast* and *Cadavra*, different name).
      Pinned by the test `every_catalog_item_and_achievement_has_its_sprite`.
- [x] `stringtable.sta` (1.36 MB) is readable and contains the keys: **the DLC catalogs use
      localization keys** (`#ISAAC_NAME`, `#THE_SAD_ONION_DESCRIPTION`), not literal names.
      `catalog` will need to resolve them from there. Languages: English, Japanese, Korean,
      Chinese, Russian, German, Spanish, French. **No Italian**: we're adding it ourselves.
- [x] Wrote `samples/filelist-completo.txt`: 20,567 paths **verified against the indexes**.
- [x] **`unpack::ResourceSet`**: the resolver. Opens archives in precedence order
      (Repentance > AB+ > AB > base), tries the `resources-dlc3/` and `resources/` roots,
      and answers a logical path without the caller knowing anything about archives or roots.
      4 real-data tests, including "the most recent DLC wins".
- [x] **Verification screen extended** with the `extraction_report` command: which archives
      opened and their mode, the catalog read from `items.xml` (**909 entries declared**, versus the
      733 slots in the save's section — that's the correct number to display), and 60 real sprites
      extracted at runtime as `data:` URLs. The view-model lives in `ipc::resources`, **declared
      provisional** until `catalog` exists, with hand-written base64 verified against the
      RFC 4648 vectors. Suite: **111 tests**.
- [x] Verified that `gfx/ui/coop menu.png` (a single sheet, not a folder) contains the
      head icons of every character: ideal candidates for the matrix row label.
      Needs slicing by index. And `completion_widget.png` exists in **two versions**: the
      Repentance one (which wins by precedence) has three treatments, the AB+ one has four.
- [x] **Spike on `stringtable.sta`, closed in an hour.** It isn't binary: it's **XML**, UTF-8,
      3,151 keys across 14 categories (`Items`, `Players`, `PocketItems`, `Entities`, …).
      Resolves **all 909 keys** from `items.xml` and 21 of 21 from `players.xml`. Eight
      languages: English, Japanese, Korean, Simplified Chinese, Russian, German, Spanish,
      French. **No Italian** — corrected the earlier line that promised it.
      `achievements.xml` and `challenges.xml` don't use keys: literal English text.
      It only exists in `afterbirthp.a` under `resources/`, but it's up to date (December 2025
      `modified` info) and covers Repentance's items.
      *(Corrected 2026-09-03: the "two pickup placeholders" cited above, `PILLS_HERE_NAME`
      and `TAROT_CARD_NAME`, weren't unresolved keys — they were inside a commented-out element
      of `items.xml`, which the XML parser rightly ignores. The real item count is 909,
      not 911, and all the keys resolve. Found during Task 7 of plan A.)*
- [x] **Found and fixed a `catalog_peek` bug**: it searched for `"<tag "` with a space, and
      Repentance's `items.xml` separates attributes **with tabs** in 233 out of 909
      elements. It skipped them silently: the screen counted 678 items, it's actually **909**
      (425 passive, 170 active, 126 familiars, 188 trinkets). Test went red → green, and the
      sprite coverage test now uses the `items.xml` that wins precedence instead of one picked
      by hand: 909 with sprites, 0 missing.
- [x] **Free graph edges**: 370 of 909 items declare `achievement="N"`, i.e.
      which achievement unlocks them. Together with the comments in `achievements.xml` (the
      current unlock condition in English) that's raw material for M2, and `catalog` needs to
      preserve it.
- [x] Cleaned up the probes in `crates/unpack/examples/`: from twenty-two down to four, now tracked
      and portable (they read `samples/packed`, not an absolute path): `probe_all` (a full
      regression pass), `probe_coverage` (how many entries we can name), `extract_design`
      (reference sprites with dimensions) and `estrai` (pick one resource, from the resolver).
- [x] **Executed `catalog` plan A** (10 tasks, commits `854aa5f`..`a83fff8`): pure crate
      with a `Catalog::build` that degrades, a single XML reader on `quick-xml`, stringtable
      resolved, `items.xml` and `players.xml` normalized, the head-icon spike closed, real-data
      tests, wired into `ipc` and `app`, verification screen with real names.
      Report — `docs/superpowers/reports/2026-09-03-catalog-a-report.md`. Final suite:
      **156 tests, 0 failed** (`cargo fmt --check` and `cargo clippy --all-targets -- -D
      warnings` clean).

### 2026-09-02

- [x] Rebuilt the state from `git log`, specs/plans, `cargo test --workspace` and the filesystem.
- [x] Suite verified: 48 tests, 0 failed (11 of which skipped for lack of real data).
- [x] Confirmed Isaac isn't installed → `catalog` postponed, by shared decision.
- [x] `samples/` repopulated from the local Steam Cloud profiles
      (`20240606.rep_persistentgamedata1.dat`, `20250112.rep+persistentgamedata1.dat`).
- [x] Red test 521 vs 523 investigated and closed: it was a mislabeled fixture, not a bug.
- [x] Wrote `DESIGN-BRIEF.md` (repo root, committed in `bf21545`) for Claude Design: constraints,
      UI stack, status light for the seven screens' data, real JSON contracts, states to
      design, i18n vocabulary.
- [x] Chose the next M1 piece: **Tauri skeleton + IPC commands**, instead of `catalog`.
- [x] Renamed the screen **"Tonight" → "Next Steps"** in `PROJECT.md` and in the brief.
- [x] Removed the promise of a **"real Dead God %"** from `PROJECT.md`: it contradicted
      `CLAUDE.md`, which forbids computing completion percentages while the mark's third bit
      stays unexplained. Replaced with an honest wording.
- [x] Added **Profile selection** to the screen list in `PROJECT.md`, as a
      persistent state rather than a first-run step (explicit requirement).
- [x] `CLAUDE.md` and `README.md` aligned with the real state, pointing to this file.
- [x] Brainstorming and **Tauri skeleton spec** written and committed
      (`docs/superpowers/specs/2026-09-02-app-shell-ipc-design.md`).
- [x] **Frontend conventions** (`docs/frontend-conventions.md`), written against the
      target stack: tokens in `@theme` (one place, no longer two), icons via
      `@lucide/vue` sized with `size-*` instead of the prop, `prettier-plugin-tailwindcss`
      from the first commit. The five non-negotiable rules are also in `CLAUDE.md`.
- [x] **TDD implementation plan** in 12 tasks
      (`docs/superpowers/plans/archive/2026-09-02-app-shell-ipc.md`). Tasks 1–7 are Rust and need
      no Node: executable right away.
- [x] Spec corrected on five points that came up while writing the plan: `modifiedUnix` instead of an
      already-formatted date, `CandidateSource` not exposing the Steam account id,
      `SectionCount` with no translated label, the `Cell::Unexpected` variant, and criterion 6 made
      precise (a display-only `pathHint` isn't a violation).
- [x] **Node 22 LTS and pnpm via corepack** (E3). Node 22.22.0 and pnpm 10.33.0 were already on
      the machine; on 2026-09-06 the major version was pinned in `package.json` (`engines`) with
      `engine-strict=true` in `.npmrc`, so anyone cloning on a different major finds out right away
      instead of hitting a build error.
