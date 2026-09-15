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
- [ ] **M1 — Rust parser, discovery, unpack, Completion screen** ← in progress, and what keeps
      it open is now **3.6 Settings and About** and **3.7 tabs that survive a restart**, plus the
      wiki polish listed under `wiki`. Everything the title names is done: the parser, discovery,
      unpack and the Completion screen, which landed as sub-project 3.2 on 2026-09-11.
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
- [x] **M4 — Log watcher and run archive** — **closed on 2026-09-15**, when its last open
      measurement was answered. It reads "designed, not started" no longer: that headline
      survived both sub-projects and both screens, and was corrected only when somebody asked
      what stage the project was at and the file could not say.
      All four pieces are in: the pure `run` crate (typed events, the fold, the rules file, and
      the part of tailing that is not I/O), a thin `log-watch`, the store's **fourth** migration
      — `events` as rows, `runs` as a derived cache carrying the rules version that produced it
      — and the two routes, which are `Live` and `Runs` and are real screens under **Tool**.
      Every sub-item below it is closed, which is the check that was run before this box was
      ticked rather than after.
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
      - [x] **Sub-project 2, the two screens — designed 2026-09-14, both merged the same day**,
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
            `run` crate, 41 tests on the day (60 on 2026-09-15), plan
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
      - [x] **The agreement with the game's own counters — closed on 2026-09-15.** It had been
            open as a *measured* result rather than a gap: the spec named 2026-09-08 as the one
            window that could answer it, and measuring it showed the window does not contain its
            own run — the log calls `unlock steam achievement` twice and **no slot of 642 turns
            on** across it. The owner played the run the entry asked for, solo and non-Greed, and
            the window around it says `STREAK_COUNTER` +1 for the one win the log folds to and
            `DEATHS` unmoved for its zero deaths.
            **And the link the archive was missing came with it**: the log's
            `unlock steam achievement '<id>'` and the save's achievement slot are the **same
            number**, which the 2026-09-08 test could only record as unknown. Report
            `docs/superpowers/reports/2026-09-15-window-and-the-eleventh-chunk.md`.
      - [x] The `Live` and `Runs` screens, on the contract this model fixes — both merged on
            2026-09-14 (2a and 2b above), and both moved under **Tool** on 2026-09-15 because
            neither reads the `.dat`.
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
      (ARCH000 format, djb2 + FNV hashing. It **needed** a correction — it gave the version as a
      constant where byte `0x07` is the compression mode — and got one: the spec has read
      `0x07 u8 compression mode / is NOT a version number` since the three modes landed on
      2026-09-03, and explains what it used to say. This line asked for it for twelve days
      after it was done.)
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
      propagated into the spec. 81 tests in the `catalog` crate on 2026-09-04 (51 unit, 15 on `build.rs`, 15 on `real_data.rs`); **103 on 2026-09-15**.
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
`catalog` types into already-resolved JSON. **276 tests on 2026-09-15**; it read "83 tests" from
2026-09-05 until then, which is a count with no era on it in the section that describes the
state. A number here is a fixture of a day, and the day belongs beside it.

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
- [ ] **The snapshots table, decided and still not written**: sections 1 and 4 of a `.dat`,
      with date and origin, imported from `save_backups\` and `online_logs\`.
      **It is not "migration 2" any more** — this line said so until 2026-09-15 and three other
      migrations were written past it: 2 is the plan queue, 3 the window session, 4 the run
      archive. It would be the **fifth**. It also said "arrives with M3", and M3's queue shipped
      on 2026-09-08 without it.
      What it is worth is unchanged and now larger: `save_backups\` holds **40 dated pairs** on
      this machine and `online_logs\` **30 more** profile snapshots, which is a history the app
      shows nothing of.

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
      - [x] *Representing the multi-line wrapper itself* — **closed on 2026-09-15 as B49, and
            the claim below was wrong.** No `Block` variant was needed: a census of every span in
            `dataset/raw/` that opens on one line and closes on another found seventeen, in four
            families, each with a shape the contract already has. The sentence that made this a
            design decision is the one it refuted. What it reads as originally:
            The template's content is block-level, so
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
      - [x] the string unions in `ui/src/lib/ipc/types.ts` — **overtaken**: the file is
            generated from the Rust types since 2026-09-13 and `pnpm scan` covers rules 4 and 5,
            so `CandidateView.prefix` is `SavePrefix` and the six are enums on both sides. The
            list below is the shape of the problem as it was, kept because it names the six:
            `CandidateView.prefix`,
            `MissingReason`, `OriginView`, `ItemKindView`, `CandidateSource`, `StepsBasis`:
            that's six of them since fieldless enums travel as strings, and they will grow —
            they need converting to `const … as const`
            in the modules that own them (rule 5), and `ui/scripts/scan-conventions.mjs` needs
            to gain checks for rules 4 (raw `<button>`/`<input>`: `App.vue` still has one in the
            candidate list) and 5, unchecked by anything today;
      - [x] `next_steps` rebuilds the entire `UnlockView` — **closed on 2026-09-08** by the icon
            protocol (C2): no row carries base64 at all, each carries a reference the Tauri crate
            serves, and `next_steps` went 124 KB to 3 KB. As written it read: "base64 icons twice
            on every load, acceptable for the verification screen, needs rethinking for the real
            one";
      - [ ] in the visual checks the on-screen number is compared against a dated reference
            file, not a constant: the app reads the live save, which changes as you play.
- [x] Real Completion screen — **done as sub-project 3.2** (2026-09-11), with the mark symbols
      and character heads cropped from the user's own sheets. It was webapp work and it started
      after the handoff, exactly as this line said it would.
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
- [ ] **563 archive entries remain unnamed** (out of 19,473, i.e. 2.9%): 251 in
      `afterbirth.a`, 240 in `graphics.a`, the rest scattered. These are files none of our
      sources names. **They don't affect the product**: the catalog is covered at 100% (see below).
      **Re-measured on 2026-09-15** with `cargo run -p unpack --example probe_coverage`: 19,473
      indexed, 18,910 named, 563 missing — the same total twelve days on, so the number is a
      property of the install and not of one afternoon. The per-archive split was off by 3 on
      `afterbirth.a` (248 for 251) and by 1 on `graphics.a` (239 for 240). The total stayed right
      throughout because the third term is "the rest scattered": a sum with one unnumbered
      remainder absorbs any error in the named rows and reports nothing.
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

> **Checked against the machine on 2026-09-15**, which is where this list is meant to be read
> and had never been read from. Two of its seven items were already satisfied and one was asking
> for the wrong thing: the agreement with `STREAK_COUNTER` and `DEATHS` closed that evening with
> a run the owner played, the `samples/packed` junction has been in place since 2026-09-03, and
> the entry asking for a **642 / 523** save wanted an era that has been covered since
> `20260905`. **An instrument list that nobody re-reads at the instrument is a list of things
> that may already be done** — three of seven here, which is the reason this note exists.

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
      **A second window, 2026-09-15**: 45,146 → **45,344**, so **+198** over one solo run that
      reached The Void, against +11 over one online Greed run. And it is not a fixed fraction of
      the tallies either: they gained **1237** (432 + 805) in the first window and **2072**
      (1105 + 937 + 30) in the second, so the tallies-per-trailing ratio goes **112 to 10.5** —
      an eleven-fold move where the tallies themselves not quite doubled.
- [ ] **The 40 unknown cells in the completion matrix** — Mother and The Beast for The
      Forgotten and the 19. One run of Mother with a Tainted character closes the whole
      20 × 2 block, because the base indices are already pinned and only the evidence that
      those cells move is missing.
      **Measured on 2026-09-12, and the absence is now a finding rather than an assumption**:
      a walk of the whole dated series looking for *any* completion of Mother or The Beast by
      those 20 characters found none. The blocker is real, the series cannot close it, and it
      is what keeps 40 nodes of the unlock graph `Partial` instead of answered.
- [x] **A save of the 642 / 523 era in `samples/`** — **already there, and this line asked for
      the wrong era.** `each_era_declares_its_own_counts` reads that row from
      `20260905.rep+persistentgamedata1.dat`, which has been in `samples/` for ten days; the row
      runs. What the suite still declares missing is the **2024** era (638 / 496) and **January
      2025** (641 / 521), and neither can come from this machine: the game's own
      `save_backups\` start at 2025-06-26, so those two belong to the M0 collection that lives
      outside the repo. Corrected on 2026-09-15, having been checked instead of read.
      The rest of the entry still holds: more snapshots of one profile are worth more than more
      profiles, because the comparison properties need two of the same. The series went 37 to 41
      that day, four days of it having sat unread in `save_backups\`.
- [x] **The `samples/packed` junction** to the installed game's `resources\packed` — **in place
      on this machine since 2026-09-03**, which is why `scripts/check` reports 7 skips here and
      not ~60. It stays in the list because it is the first thing to do on *any* machine with the
      game, and it is one command: without it every test on a real archive skips, and with them
      every `catalog`, `ipc` and `graph` test that needs a built catalog. The old second half —
      that it is what `pnpm design:export` needs — went with the export on 2026-09-15.

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

## The journal

**What each session did lives in `docs/completed/`**, one file per month —
`docs/completed/2026-09.md` today. It was 2869 lines of this file until 2026-09-16, **64% of
it**, and the state it was supposed to accompany was underneath it where nobody scrolled: M4
read "designed, not started" with all nine of its sub-items closed.

It is **searched, not read**. Nothing there is re-checked as the project moves, and a `- [ ]`
under a dated heading is a caveat that session recorded about its own work, never an open task.

**What a session learns that will matter again does not stay there.** It is promoted: a fact
about the game or its files goes to `CLAUDE.md` and the documents it points at, a thing still to
do goes to `docs/BACKLOG.md`, and a change of state comes here. The journal keeps only the story
of how it was found.
