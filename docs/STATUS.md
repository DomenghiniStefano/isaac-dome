# Progress status — IsaacDome

**Where the project is.** This file answers one question and should stay short enough to be
re-read: milestones, what is blocked, what needs a measurement, what is worth investigating.

| | |
|---|---|
| what the project **knows** about the game and its files | `CLAUDE.md` and the documents it points at |
| what is **still to do** | `docs/BACKLOG.md`, every entry tagged with what it needs |
| what was **already done** | `docs/completed/` — the journal, the modules, the cleanup, the merges |
| the **design** | `docs/PROJECT.md`, and `docs/superpowers/specs/` per sub-project |
| what an execution **measured** | `docs/superpowers/reports/` |
| what **nobody has looked at yet** | *"What only a window can say"*, below — the built app in front of a pair of eyes |

**Nothing in `docs/completed/` is re-checked as the project moves**: it is searched, never read.
What a session learns that will matter again is promoted out of it — a fact about the game to
`CLAUDE.md`, a thing still to do to the backlog, a change of state to here.

**This file was 4493 lines on 2026-09-15**, of which 87% was a record of finished work, and the
state sat underneath it: M4 read "designed, not started" with all nine of its sub-items closed,
and nobody scrolled far enough to find out.

**Integration branch:** `develop`. **`master` is the repository's public face and is kept level
with it**; a release will be a **tag** on `master`, not the act of moving the branch. Changed on
2026-09-16, and the reason is worth keeping because the old rule was quietly costing something:
`master` was the **default branch on GitHub** — `git ls-remote --symref origin HEAD` says so — and
under "`master` only receives releases" it had sat at the import of 2026-09-01 for two weeks, so
everyone opening the repository met seven files, an Italian README and a `docs/progetto.html` that
no longer exists, **733 commits** behind. A convention that reserves a branch for releases costs
nothing while releases exist; with none yet, it was an empty promise paid for by the landing page.

Getting there took two moves, both on 2026-09-16 and worth telling apart. **The two branches had no
common ancestor at all**: `develop` was re-rooted on 2026-09-07 with a fresh history while `master`
kept the 2026-09-01 import, so *any* update to `master` would have needed
`--allow-unrelated-histories` or a force-push over published history. One
`git merge -s ours --allow-unrelated-histories master` on `develop` fixed that: it records `master`
as a second parent and **changes no file** — the tree SHA is identical either side of it — so
nothing was rewritten and nothing force-pushed. Only then was `master → develop` an ordinary
`--ff-only`, which is how it was brought level.

**A rebase was the alternative and was rejected**: replaying 728 commits of the shared integration
branch and force-pushing it, to gain an ancestor that is an abandoned scaffold.
**Branches from 3.4 on: one per sub-project, cut from `develop`** (decided 2026-09-11). Nobody
commits on `develop` directly: it is where finished work lands, through a `--no-ff` merge with
`scripts/check` green. A sub-project gets its own `feature/<name>` branch — Collection is
`feature/screens-collection` — so each merge is one sub-project, its diff stays reviewable, and
a piece that has to be redone is thrown away without touching the others.
`feature/design-system-screens` had grown to hold 3.1 through 3.3b under a name that no longer
said what it carried; it is fully merged and kept, its deletion waiting for the owner.
---

## Milestones

- [x] **M0 — Format spike**
      `.dat` format decoded and verified on 28 real saves, working Python parser,
      counters labeled, marks matrix rebuilt, log verified.
- [ ] **M1 — Rust parser, discovery, unpack, Completion screen** ← in progress, and what keeps it
      open is now the wiki polish listed under `wiki` — and **the window nobody has opened on any
      of it**.
      **The small follow-ups landed on 2026-09-18**, on `feature/small-follow-ups`: seven of
      B12's ten design-system items and both halves of **B65**, the palette's `Ctrl+Enter` — one
      commit each, and no sub-project number, because they are backlog entries and not a screen.
      Plan `docs/superpowers/plans/2026-09-18-small-follow-ups.md`. Three of the four things the
      entries did not know are in B12's own entry; the fourth is that **the two `text-sm` in
      `WantAnswer.vue` had been generating no CSS at all**, which nothing would ever have said —
      the scanner rule written the same hour is what found them. B12 keeps items 3, 10 and half
      of 4, all three of which need a window before anything can be decided.
      **3.12 landed on 2026-09-18**: the Roll screen — *Stasera* / *Tonight* — draws one target
      from the completion matrix, kept in a new pure crate (`roll`: the space, the deck a preset
      leaves in it, and the draw itself, a function of `(deck, seed)` with the clock kept in
      `app` so the crate has none of its own) and one document — `store` migration 6, a single
      row pinning the preset and the current draw together, because a second row would be a
      second answer to "what is the preset". Spec
      `docs/superpowers/specs/2026-09-17-roll-design.md`, plan
      `docs/superpowers/plans/2026-09-18-roll.md`.
      **This machine cannot look at half of it.** It holds the saves and not the installed game,
      so `NoCatalog` and `PlayabilityUnknown` are the ordinary path here rather than an edge
      case, and the card's `headUrl` and `artUrl` come back `null` on every draw — only the
      fallback-outfit path has ever run. `crates/ipc/tests/roll_real.rs` still ran on real data:
      48 `sample:` lines across its 3 tests, zero skips, because the deck and the draw need only
      the save, never the archives.
      **3.9 landed on 2026-09-17**, the evening of the same day: the Completion screen counts
      twice. B22's item 4 — the matrix's two number columns, each over the cells it can read —
      and B23 — the strip is three tiles and no longer counts cells — closed together, because
      they are one design pass and the entries say so. Plan
      `docs/superpowers/plans/2026-09-17-completion-columns.md`, report
      `docs/superpowers/reports/2026-09-17-completion-columns-report.md`.
      **`Schermate.dc.html` had one column**, so there was nothing to copy: the shape is the
      sub-project's own, and the one defect of it was found by a browser rather than by a test —
      the group header's totals landed 130px past their own columns, because a flex row's
      `ml-auto` ends at the container and the container is wider than the tracks. Nothing red,
      and nothing a unit test can see.
      **B17's welcome flow landed on 2026-09-17 as sub-project 3.8**, with B14 folded
      into the same branch: the app opens by asking which save you are playing with and showing
      what each one holds, a full-screen takeover that is a **state above the router** and never a
      route — here a route is a tab, and 3.7b's session document would have saved and restored it.
      The preview travels **with** the candidates in one command (N8's rule), computed from the
      `.dat` alone so it answers with no game installed, and a count the file did not let us read
      says so rather than showing a zero. B14 came with it because the "nothing found" branch is
      where its two buttons belong, and shipping that branch without them is the
      button-that-does-nothing its own entry warns about. Spec
      `docs/superpowers/specs/2026-09-17-welcome-flow-design.md`, plan
      `docs/superpowers/plans/2026-09-17-welcome-flow.md`, report
      `docs/superpowers/reports/2026-09-17-welcome-flow-report.md`. **Nobody has opened a window on
      it**, and the folder dialog is the one check the documentation could not settle on paper.
      Everything the title names is done: the parser, discovery, unpack and the
      Completion screen, which landed as sub-project 3.2 on 2026-09-11.
      **3.6a landed on 2026-09-16**, the half that could be decided: the Tabs settings screen the
      sidebar had been pointing at since 3.1 with nothing behind it, the switch moved off
      Background where it never belonged, and the session's diagnostic beside it. Spec
      `docs/superpowers/specs/2026-09-16-settings-tabs-design.md`, report
      `docs/superpowers/reports/2026-09-16-settings-tabs-report.md`. **3.6 is not closed**: the
      welcome flow is untouched on purpose.
      **3.7 is one spec and three branches, and all three landed on 2026-09-16** (spec 2026-09-15,
      `docs/superpowers/specs/2026-09-15-tabs-session-design.md`). **3.7a landed on 2026-09-16** —
      a history entry is `{ location, view? }`, so a tab keeps its facets, its sort, its selected
      row and its scroll across a tear-off and a restart; report
      `docs/superpowers/reports/2026-09-16-tabs-own-their-state-report.md`. **Nobody has opened a
      window on it**, which is what B39 stays open for. **3.7b landed on 2026-09-16** — the
      session is windows of tabs, version 2 of the document, and **the window that writes it is
      elected** instead of being `main`: `main` can be closed while other windows live, so under
      the old rule the session stopped being written from that moment, silently, with the app
      alive in the tray to prove it. Restoring is the tear-off's own machinery run against the
      document, and a remembered box is clamped onto a monitor that exists. Report
      `docs/superpowers/reports/2026-09-16-tabs-windows-report.md`. **Nobody has opened a window
      on this one either**, and here that gap costs most: what it fixes is a failure that is
      invisible on screen by definition. **3.7c landed the same evening** — the sidebar you sized
      is the sidebar you get back, as a named key beside `windows` that costs no migration and no
      version bump; report `docs/superpowers/reports/2026-09-16-tabs-sizes-report.md`. It stores
      **one** of §8's two keys: `tables` is not added, because nothing in the app produces a table
      size — B27's resizable tables are not built, and that is why **B27 stays open** with two of
      its three parts untouched.
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
- [x] **M3 — Derived plan** — **closed on 2026-09-18, and it had been done since 2026-09-11.**
      The **plan queue** landed on 2026-09-08: an ordered series of achievements whose order is
      yours and can never contradict the graph, report in
      `docs/superpowers/reports/2026-09-07-plan-queue-report.md`. The line under it read *"what
      remains of M3 is the screen, which waits for the design system"* until today — and the
      screen landed with 3.3 on 2026-09-11 (`05dadcb`, *the Plan, a queue you drag and a repair
      that says where it stopped*): `PlanScreen.vue` with `QueueCard`, `QueueRow`,
      `ProposalAside` and `QueueFootnotes`, on `/progress/plan`, behind `plan`, `add_goal`,
      `remove_goal` and the five `queue_*`.
      **It is the third headline found stale on 2026-09-17**, after B3 and B6, and the three
      share one cause: the sentence that says what remains was true when it was written and
      nobody read it again against the code. A milestone says "in progress" for as long as
      somebody believes its last paragraph.
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
            **The first window opened on this screen was the owner's, on 2026-09-18, and it found
            it unreadable** → **B64**. Fourteen room kinds paint the same grey square, so the grid
            cannot show what you drew on it; the start room is marked for a screen reader and for
            nobody else; and what is called the legend is the brush picker, so no colour on that
            screen is ever explained. **F1 wrote no window checks**, which is why none of this was
            waiting in *"What only a window can say"* — a sub-project that draws a screen and
            gathers no checks does not appear there as a gap, it does not appear at all.
      - [ ] **F2 — the log's half**: `N rooms in M loops` read back, so the screen can say the
            game generated 19 rooms and you have painted 15. **`run`'s half landed on
            2026-09-16**, `feature/floor-rooms-read`: an eleventh event, and a floor that carries
            `Generated` — `NotSaid`, `Once` or `Several` — which is three states and not an
            `Option<u32>` for the reason `graph` has `Partial`.
            **The measurement the entry asked for is done, and it made the question narrower
            than the entry had it.** There is one generation pass per floor in five of the six
            logs, and the corpus holds **one** multi-pass floor: `m_Stage 4, m_StageType 4` —
            Mines II, the only floor in it with an area of its own — whose two passes report
            the *same* 19 rooms. So "first" and "last" cannot be told apart on this data, and
            nothing picks: both are kept and the reader has to say which it took.
            **And a mode the entry did not know about**: Greed describes no generation at all —
            seven floors, zero `generate...` lines — and it is not the online that silences it,
            since the other `[Net]` log describes all eleven of its floors. A Greed floor has no
            number, which is not a floor of no rooms.
            The block and its traps are in `docs/log-format.md`.
            **What is left is the screen**, and it is blocked on one thing nobody has measured:
            whether the count includes the Secret and Super Secret rooms, which the game places
            *after* the summary line. That screen exists to find the secret room, so the
            subtraction says two different things depending on the answer. The instrument is a
            floor painted to exhaustion in the game.
- [ ] **B41 — starting with Windows — landed on 2026-09-16**, `feature/autostart`, and it belongs
      to M4's watcher rather than to a milestone of its own: the hole it closes is narrow and
      real, a game launch followed by another game launch with the app never having run in
      between, and `log.txt` is rewritten every time. The registry is the only source of truth —
      `settings.json` gains nothing — a login launch is silent, and the switch is inert in
      development builds because `current_exe()` there is `target\debug\app.exe`. Plan
      `docs/superpowers/plans/2026-09-16-autostart.md`. **Nobody has seen it**: it needs an
      installed build, a logout and a login, which is what its five lines below are for.
- [ ] **M5 — Public release**. **The app can be packaged since 2026-09-16, and could not before**:
      `pnpm build` compiled and then stopped on `Couldn't find a .ico icon`, with
      `crates/app/icons/` holding exactly the set Tauri looks for. Declaring `bundle.icon`
      explicitly fixes it, and the first MSI and NSIS installers this project has ever produced
      came out of that run. **Nothing runs `pnpm build` to the end** — `scripts/check` does not, by
      design, since it downloads WiX and NSIS and takes minutes — so the next thing that breaks
      packaging will be just as quiet. B11 closed with it: the font's licence and readme and
      `dataset/ATTRIBUTION.md` are inside both installers, checked by extracting them.

---

## What is already built

**The record of finished work lives in `docs/completed/`**, and left this file on 2026-09-16:

| file | what it holds | was |
|---|---|---|
| `docs/completed/modules.md` | M1's modules as they were built, one section per crate | 532 lines here |
| `docs/completed/cleanup.md` | the structural cleanup, N1 to N8, all done | 387 |
| `docs/completed/handoff.md` | what closed the structural base and went to design | 127 |
| `docs/completed/2026-09.md` | the session journal | 2869 |

That is **3915 of the 4493 lines this file had**: 87% of the document that is supposed to say
where the project is was a record of work already finished, and the state sat underneath it.

None of it is re-checked as the project moves. **Searched, never read** — and what a session
learns that will matter again is promoted out of it rather than left there: a fact about the game
to `CLAUDE.md` and what it points at, a thing still to do to `docs/BACKLOG.md`, a change of state
to this file.

## Open blockers

- [x] ~~**`Archive::open` reads the whole file into memory.**~~ **Resolved on 2026-09-06** (C1
      in `docs/completed/quality-review.md`). `open` reads the header and the index and keeps the `File`
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
      **Resolved on 2026-09-08** (C2 in `docs/completed/quality-review.md`), and not the way that entry
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

      *Read again on 2026-09-16, and it is that same machine* — worth recording because the
      paragraph above was written as an observation and is now a **second point on the same
      machine**, eight days apart. Still no game, so `samples/packed` is still absent. What
      changed is the other half: the eleven uncopied backups went into `samples/`, and a full
      `scripts/check` came out **all green, 143 skips, 283 real files touched**. The 143 is not
      comparable to that day's "~60 of ~81" — the suite has grown by three crates since — and
      no before/after was measured at this level, only at `core-save`'s (below). What the list
      says instead is where the skips live, counted from the summary: **94 of the 143 name
      `samples/packed`** — one missing junction — and **21 name `samples/logs/`**, a folder
      this machine does not have at all. Two absences, 115 of the 143. The remaining 28 are
      saves and windows of the 2026 eras (12 and 7), five `config.a`, the two "a series of one"
      and the vacuity guard that goes with them, and one "Isaac (250900) not installed".

      **A `sample:` line does not mean the test used it** — found on 2026-09-17 closing B58, and
      it is the sharper version of everything above. The three properties in
      `crates/ipc/tests/marks_real.rs` that keep the mark tables answerable to the series each
      began with `dated_series(SERIES)`, which **declares** every file it hands back, and then
      `if files.len() < 2 { return; }`. On this machine that series is one file. So the run
      printed `sample: 20250112.rep+persistentgamedata1.dat` three times, counted three real-data
      declarations, emitted **no skip**, passed — and guarded nothing. That is not a thin sample
      reporting itself honestly; it is a test claiming a file it never compared. The declaration
      is made where the sample is *opened*, and a property needs **two** of them: any test that
      walks windows can fall into the same gap, and the shape to look for is a `sample:` count
      that does not move when a property stops being able to run. The three now walk each series
      on its own and declare a skip when the coverage is not there.
- [ ] **`samples/` never contained the M0 collection.** The 28 saves over 14 months used to
      decode the format live outside the repo, and the folder is git-ignored: whoever
      clones has none, and the suite has to stay green anyway. Two sources worth knowing
      about when stocking a machine, both free of the game itself:
      `Documents\…\Binding of Isaac Repentance\` keeps the dated backups the game writes on
      its own (14 snapshots of one `rep_` profile across Jan–Mar 2024 on the 2026-09-08
      machine, of which only 3 had been copied over), and Steam's
      `userdata\<id>\250900\remote\` holds the live profiles. A denser series costs nothing
      but copying, and every copied snapshot is one more comparison the properties can make.

      **Done on the 2026-09-16 machine, and it is the same machine**: the 11 uncopied backups
      are now in `samples/`, so the `rep_` series is **15 files across Jan 2024 – Jun 2024**
      instead of 3. The entry stays open because this is per-machine and the other half is
      still short: the `rep+` series is **one** file, so `comparable_series` skips it and every
      property about *change* on the Repentance+ profile still runs on nothing.

      What the copy bought, measured rather than assumed — `core-save` alone, counted from
      `ISAACDOME_TEST_DECLARATIONS` before and after, suite green either way:

      | | `sample:` | `skip:` |
      |---|---|---|
      | before | 72 | 3 |
      | after | 276 | 3 |

      The skips did not move, which is the honest half of the result: no test switched on that
      was off. What changed is how much each property that was **already running** had to hold
      over — the same green, on five times the series.

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
`nothing`, `a real save`, `the game`, `a measurement`. The vocabulary is at the top of that file;
since 2026-09-17 the counts are on the board, because a total written by hand goes stale the moment
the next entry is opened. What lives here rather than there is what is an **instrument**
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
      **Measured on 2026-09-16, and a count of launches is refuted too.** The owner launched the
      game, sat at the title screen, opened the file select, loaded slot 1 and quit, without
      playing. **Cell 2 did not move, and neither did any other cell of any section.** Four
      snapshots taken at each step and read with `matched_window`, which covers the four sections
      still called `Unknown` — the instrument the 2026-09-08 error exists for.
      The numbers agreed before the behaviour did: **cell 2 stands at 121** while Steam's
      `gameprocess_log.txt` records **262 launches** of 250900 on this machine since 2025-06-26
      (B55), so it could not have been counting them for this profile's lifetime either.
      **What moves it happens while playing**, which is all the 2026-09-15 window ever said: it
      rose by one, and it contained a whole run. Sections 5 and 9 keep their printed names and
      stay `Unknown`.
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
      **The death happened on 2026-09-16, and tally 4 is named.** One deliberate death in the
      Basement, and across the whole save **three keys moved and all three are one entity**,
      `15.0.0` = Clotty: tally 1 `+2`, tally 3 `+5`, **tally 4 `2 → 3`, exactly one, on the
      killer, and on no other key anywhere**. `DEATHS [10]` 269 → 270 says the death reached the
      file. **Tally 4 counts the deaths an entity caused you**, and it is the first of the four
      to earn a name.
      **And the same window separates 1 from 2 for the first time**: tally 1 gained 2 while
      **tally 2 gained nothing**, in a window with kills in it. *"1 and 2 move constantly and in
      both modes"* was true of every window collected only because every one of them had both
      moving; one short run ending in a death is the case that tells them apart. What 2 is waits
      on another window, and 3 with it — though read on one entity this one says 5, 2 and 1 for
      3, 1 and 4, which is the shape B9 always asked for and never had with a *named* killer.
      New instrument: `cargo run -p core-save --example bestiary_diff -- <before> <after>`,
      which reports every key that moved, per tally — `matched_window` only ever said whether a
      tally moved, never which entity moved it, and that was the whole question.
      **A second death the same evening, to a different enemy, is the one that closes it**: Judas
      killed by a Ministro (`305.0.0`), and tally 4 gains a **new key** at 1 while Clotty stays at
      3. That rules out the weak reading — *"it is Clotty's row that moves"* — which one window
      could not. Records went 159 → 160, which is the growth pattern the hypothesis rested on.
      **Three of the four are named by the end of the evening.** Six windows, each with a counted
      number of a named entity — the name from the log's `Spawn Entity with Type(n)` lines, the
      count from the player — say **tally 1 is what you met and tally 2 is what you killed**. The
      window that separates them is the one nobody had ever taken: *a known number left alive*.
      Three Round Worms and two Stoneys met, two worms killed and nothing else: `1` gained 3 and 2,
      `2` gained 2 and nothing. The three death windows, where the owner died on purpose without
      killing anything, are the ones that had read as *"1 counts kills and 2 is asleep"*.
      **A champion counts on the entity's own key**: four Skinnies spawned as `226.0.0`, one of
      them purple, and the file counted four on one row.
      **What is left is tally 3**, which moved `+5` on Clotty and `+2` on Ministro and **zero in
      the other four windows**, including every one with kills in it — and is smaller than tally 1
      on 308 of their 311 shared keys.
      **And a contradiction that is recorded rather than resolved**: across the whole save tally 2
      is larger than tally 1 on 213 keys, smaller on 132, equal on 91 — and *met* can never be
      fewer than *killed*. The six windows describe today's rule and the accumulated totals do not
      obey it. One of the two probably changed meaning in a patch; that is a hypothesis, and what
      is not is the consequence: **the totals may not be used to reason about meaning, only the
      movements may.**
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
      **It moves without any play at all, measured 2026-09-16**, and that corrects both numbers
      above. A launch with no run — title screen, file select, slot 1 loaded, quit — moved it
      **45,441 → 45,449, +8**, and moved **nothing else in the whole file**. It is the only thing
      in the save that a bare launch touches.
      Three writes, and the file select is not one of them: **+4** at the title screen, **0** at
      the file select (the file is byte-identical), **+2** when slot 1 is loaded, **+2** on quit.
      **So both windows above contained a launch, and the launch is most of the smaller one.**
      Subtracting it, the Greed run is about **+3** and the solo run to The Void about **+190** —
      the ratio between the two runs is not 11 to 198 but roughly **3 to 190**, which is a
      different shape from the one the paragraph above reasons about. Whatever it counts, it is
      not run-shaped and it is not idle either.
- [ ] **Whether a challenge's `unlocked_by` means "all of these achievements" or "any of them".**
      Opened 2026-09-17 with 3.11, and it is the one thing the save cannot answer: the file
      records what you *finished*, never what the game *offered*. On the reference profile all 13
      finished challenges that have gates have every gate done — which is consistent with both
      readings, because "all done" satisfies "any", so the corpus cannot separate them and saying
      it confirms "all of" would be the trap this section exists to avoid.
      **The instrument is the game's own challenge menu**: find a challenge with *some* of its
      gates done — the Challenges screen now lists its missing ones by name — and look at whether
      the menu offers it. If it does, `unlocked_by` is "any of", and the screen is calling
      blocked something you could have played tonight. One challenge answers it.
- [ ] **Whether `19 rooms in 12 loops` counts the Secret and Super Secret rooms.** Opened
      2026-09-16 with F2's first half, and it is the one thing between `run`'s count and a
      sentence on the Floor screen. The game places the two secret rooms in the `placing
      rooms...` phase, **after** the summary line, so the number may or may not include them —
      and that screen exists to find the secret room, which makes "the game generated 19, you
      painted 15" mean two different things. **Nothing in the log settles it**: the summary is
      the only total the file states, and `place_room: shape N` counts only the rooms that are
      not one cell. The instrument is a floor painted to exhaustion — walk every room of one
      floor, secret rooms included, count them, and read the line for that `Level::Init`. One
      floor answers it; a second on the alternative path would say whether the rule holds there
      too.
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

## What only a window can say

**The second bucket of the same kind**, collected here on 2026-09-16 because it had accumulated
in five reports and nobody could see how much of it there was. It was **33 lines** that day,
which is the number that made collecting it worth a section: five reports each said "half a
report" and sounded like a footnote, and the sum is a session's work. **It is 40 in eight groups
at the end of the same day**, and the growth is the point rather than a complaint — three
sub-projects landed since, and each of them can only be finished by somebody looking. It is the
sibling of the section above and it earns the same warning, which that one paid for: *an
instrument list that nobody re-reads at the instrument is a list of things that may already be
done* — three of its seven were, and nobody knew until somebody stood at the machine and read it.
**46 in nine groups on 2026-09-17**, when 3.8 added six of its own: the growth keeps being the
point, and the new group is the first one a *browser* can answer most of — `pnpm ui:dev` draws the
welcome's every state with no game, no save and no Tauri, and only its last two lines need the
real window. **50 in ten groups the same evening**, with 3.9's four — and that group is the first
whose sub-project **already looked**: a browser found and fixed an alignment defect there before
the lines were written, which is why there are four of them and not eight. Looking early does not
empty this list, it shortens it.
**59 in twelve groups on 2026-09-17**, with 3.11's five — and one of them is not a check but the
measurement §4 of that spec waits on, which is why it appears in both lists.
**54 in eleven groups**, with 3.10's four, and that group is the shortest yet for
the same reason carried further: a browser drove all three bars while they were being written, so
what a *window* still owes is only what a browser cannot judge — the widths, the wrapping, and
whether a word is the right word. The five checks that would otherwise be here were answered on
the spot, and one of them turned into a decision the owner took mid-work (the state row's counts,
§1 of the spec).

**2026-09-18 says the thing this list cannot say about itself.** The owner opened the Floor screen
and the `Ctrl+K` palette and found a defect in each — **B64** and **B65** — and neither screen has
a group below, nor ever had one: F1 (2026-09-15) and the search sub-project both shipped without
writing a single check. **A screen that writes no checks does not appear here as a gap; it does not
appear at all.** The counts above are of checks somebody wrote down, never of the screens that owe
one, and that is the one blind spot the section cannot see from the inside. Both entries close with
a line that puts their group here before the branch does.

**65 in twelve groups, the same day**, with 3.12's seven. One of the seven carries a label none of
the others do — `NEEDS GAME` — because it is not "nobody has looked yet", it is "this machine
cannot": the card's character head and mark symbol need a catalog this session never had. The
other six are the ordinary kind, answerable by anyone with the built app and a save.

**72 in thirteen groups on 2026-09-18**, with the small follow-ups' seven — and that group is the
answer to the paragraph above it. It is the first one written by work that had **no screen of its
own**: B12 and B65 are a design-system list and a keyboard defect, and the group exists because
the entry that reported the defect said the screen owed one. Four of its lines are the palette's
keyboard, which is exactly the kind of thing this repo cannot test and only a window can judge.

**This is the live list; the reports are the record.** Each line below was written by the
sub-project that produced it and is unticked *there* too, but a report is true on its day and is
never re-checked — so ticking a line **here** is what closes it, and the report stays as it was
written. This is `docs/BACKLOG.md`'s `, then a window` tag, gathered.

**Since 2026-09-17 every group below is also a card in the board's `UAT`**, one per group, with
its checks copied into the card's description — the owner's rule is that every change passes
through Trello, and work waiting to be looked at is work. **Nothing keeps the two in step**: a
check ticked on a card does not tick itself here. This file stays the one that is *read*, because
it sits beside the reports that produced each line; the card is what makes the work visible
on the board.

**Nothing here is blocked on thinking or on code.** Every line needs the built app in front of a
pair of eyes, and most of them need it for one minute. `pnpm dev` from the root; `predev` clears
a tray app and whoever holds port 1420 on its own. **One group is the exception and says so**:
starting with Windows is registered only in a release build, so those five need `pnpm build`, a
logout and a login.

### First, and it happens once

Three lines, and what they share is that they run against **data that already exists on this
machine** — a session, a database, an archive written by the version before. Each of them can be
read wrong exactly once, and then the evidence is gone.

**One launch answers all three**, which is the trap: the next start of the app migrates the
database, refolds the archive and restores the session, and after that every one of them looks
like it has always been that way. **And it does not have to be the installed build** — `pnpm dev`
opens the same `app_data_dir()`, so the first `pnpm dev` of the next session spends all three
without asking. Read them on whichever launch comes first, before touching anything else — and if
the archive matters more than the minute it costs, copy
`%APPDATA%\dev.isaacdome.app\isaacdome.db` first (Tauri's `app_data_dir()`, and the identifier is
in `crates/app/tauri.conf.json`).

- [ ] **The session on disk was written by version 1 of the document**, and the first launch after
      3.7b reads it. The tabs that were open must still be there. It is the only line in this
      section whose failure costs a real person something, and it cannot be run twice.
      → `docs/superpowers/reports/2026-09-16-tabs-windows-report.md`
- [ ] **Migration 5 runs on the real `isaacdome.db`** (B62). The Plan's goals, the plan queue, the
      window session and the whole run archive are still there afterwards: the column is nullable
      and additive and a test builds a version-4 file to prove it, but the file that test builds
      is not the one on this machine.
- [ ] **The rules file went from 2 to 3** with F2, which **invalidates every folded run in the
      cache on purpose**. So the first launch refolds the whole archive: the Runs screen must come
      back holding the same runs it held, and the totals must not move. If a run is missing after
      that, the fold lost it — and the cache that would have hidden the loss is exactly what the
      version bump threw away.

### The small follow-ups (B12 and B65) — a keyboard nobody can test, and three colours

The first group that comes from a **defect the owner found in a window**, which is why it exists
at all: the search sub-project never wrote a check, and B65 says so. Four of the seven are the
palette's keyboard and none of them can be pinned red first — `ui/` mounts no component — so the
decidable half was moved into a pure function with its own tests and **the rest is this list**.
The last three are the cheap kind: two need only `pnpm ui:dev`, and one needs a screen reader.

- [ ] **`Ctrl+Enter` on the highlighted row opens it beside the active tab**, the palette closes,
      and the tab you were on has not moved. This is the gesture the footer has been drawing
      since the palette existed, and it did nothing at all until 2026-09-18.
- [ ] **Plain `Enter` on a row that came from the backend.** Type something whose first row is
      *not* a screen — a wiki page or a collection row — wait out the 120 ms, press `Enter`: the
      active tab navigates. B65 predicted this was broken and the fix does not prove it was, so
      this line is now a regression check rather than a diagnosis.
- [ ] **The highlight stays where you put it.** Arrow down two rows, then type one more letter
      that keeps that row in the answer: the highlight must still be on it, not back at the top.
      That is `keyAfterAnswer`'s only promise and a window is the only place it shows.
- [ ] `Ctrl+Enter` on the **last** row — *tutti i risultati* — opens the Search screen beside the
      tab, with the query in it. It is the one row that is not a result, and it takes a different
      path through the same handler.
- [ ] **The Kit's Progress bar, read by Narrator**: "166 su 408, 40 non leggibili", and not
      "40%". `aria-valuetext` is the only part of that bar no eye can check, and the only bar in
      the repo with a hatched segment is that one.
- [ ] **A Select's hover is the colour it was.** `--color-field-hover` is `--color-row-hover`'s
      value under a name of its own, so the check is that *nothing moved* — on the Kit page, a
      trigger hovered beside a table row.
- [ ] The two paragraphs in the **Want** answer — "modo 1 di 2" and the unknown-steps line — are
      the size they were: their `text-sm` generated nothing and is `text-body` now. A browser
      answers it (`pnpm ui:dev`).

### The Challenges screen (3.11, B3) — the four conditions against the game's own menu

A browser drew every state, including the two a real profile never shows together: a blocked row
naming its gates, and the row the wiki knows nothing about. What is left needs the game running
beside the app, or a person's judgment.

- [ ] **the conditions against the game's own challenge menu**: is `Samson, fino a Mom's Heart,
      bendata` what the menu says? Four rows is enough to know whether the wiki's words and the
      game's agree
- [ ] **a challenge with only *some* of its gates done, checked against the menu.** This is the
      one that settles §4: if the game offers it, `unlocked_by` is "any of" and the screen is
      calling it blocked wrongly. It is also in *"What only a machine with the game can answer"*
- [ ] the reward added to the Plan from here, and the row saying it is queued afterwards
- [ ] forty-five rows without a virtual list, on a short window: the page scrolls and the bar's
      state row stays where it was
- [ ] the row's goal at a narrow width: the inline text truncates instead of pushing the state
      badge out of the row
      → `docs/superpowers/reports/2026-09-17-challenges-report.md`

### The one filter bar (3.10, B29) — what a browser could not judge

A browser drove all three bars as they were written, on the fixtures: the fold, the search that
appears by itself, the counts a menu promises against the rows the list then shows, and the chip
that survives the fold being closed. What is left needs a real window, or a person's judgment.

- [ ] the three bars in a **narrow** window: the dropdowns wrap, the chips row stays readable, and
      the sort group does not collide with the `N / M` beside it
- [ ] **does the fold's button read as a control?** It is a ghost button between the last dropdown
      and nothing, and at rest it can read as a label. A judgment, and it is the owner's
- [ ] **the menu clears what you typed when you pick a value.** It is `CommandItem`'s own
      behaviour, the one the search palette wants; in a multi-select it means picking two values
      out of thirty costs typing twice. Whether that is worth owning the rows for is a call
- [ ] the real Pool and Personaggio lists — 30-odd and 39 values against the fixtures' 12 and 10 —
      with the menu open on a short window: it must scroll and not push the page
      → `docs/superpowers/reports/2026-09-17-filter-bar-report.md`

### The Completion screen's two columns (3.9, B22 item 4 and B23)

**The group that was looked at first**: `pnpm ui:dev` with `?fixture=active` drew every state
this sub-project produces, and it found the one defect — the group header's totals landing past
their own columns — before the branch was merged. What is left is what a fixture cannot reach
and what a person has to judge.

- [ ] **a save with an unreadable section**: the pair reads `0/0` in the faint colour and not as
      a finished row. The tone is pinned by a test and no fixture draws it on screen — only a
      profile whose section is missing does
- [ ] the matrix in a narrow window: the card scrolls sideways and the two columns stay attached
      to the last boss instead of drifting away from it
- [ ] **does the footer's two rows read as one statement?** `Personaggi con il marchio` and
      `Di cui in hard` — a judgement, and it is the owner's. The alternative was two numbers
      stacked inside each 40px column, which fits and says nothing about which is which
- [ ] **far down the matrix, nothing says which of the two columns you are reading.** Whether
      that wants a sticky heading is a design call nobody has made, and it only shows up on a
      screen shorter than 34 rows
      → `docs/superpowers/reports/2026-09-17-completion-columns-report.md`

### The welcome flow (3.8, with B14) — the first thing a stranger meets

The only group here that a **browser** can answer most of: `pnpm ui:dev` and `?fixture=pick`,
`?fixture=none`, `?fixture=active` draw every state with no game, no save and no Tauri. The last
two lines are the exception and need the real window, because a dialog is not a fixture.

- [ ] the takeover on a first launch: no sidebar, no tabs, and the window still moves and closes
- [ ] the four card shapes, all drawn by `?fixture=pick` — whole, a count that could not be read,
      unreadable cells, and a file that could not be parsed at all
- [ ] the picker reopened from the profile indicator over a settled profile, and `Annulla`
      closing it without changing anything
- [ ] two windows with no profile: choosing in one frees the other, through `ProfileChanged`
- [ ] **the folder dialog opens and the app stays responsive while it is up** — the one check the
      Tauri documentation could not settle on paper, which is why the commands are `async` with
      the callback form rather than `blocking_pick_folder`
- [ ] a folder chosen that holds no save says so, and does not read as "we found nothing"
      → `docs/superpowers/reports/2026-09-17-welcome-flow-report.md`

### The Tabs settings screen (3.6a)

- [ ] the Settings sidebar's fourth entry opens a screen and not a placeholder
- [ ] the switch turns the session off and on, and the failure alert appears when the write fails
- [ ] ~~Background reads as one subject now that it holds one switch~~ — **the premise went the
      same day**: B41 put a second switch there, and the intro had to grow a second clause to
      cover it ("when the app starts, and what it does when you close the last window"). The
      question is the same one and the answer may now be different: **does Background still read
      as one screen, or is "when it starts" a screen of its own?** A judgment, and it is the
      owner's.
- [ ] the "what is saved" lines read as sentences and not as a list of fields
      → `docs/superpowers/reports/2026-09-16-settings-tabs-report.md`

### The sidebar's width (3.7c) — one line here is a *decision*, not a check

- [ ] the sidebar sized, the app closed and reopened: it comes back at that width, on the first
      paint and not by snapping to it a beat later
- [ ] **two windows open: dragging one window's edge moves the other's.** This is decision 1 of
      3.7c and the thing to judge rather than verify — the document holds one number, so two
      windows holding two would mean it silently keeps whichever was written last. If it reads as
      wrong, the fix is a width per window in the document, not a second hidden value.
- [ ] a torn-off window opens with its creator's sidebar and not with the default
- [ ] a width stored by a build with other bounds opens inside today's
      → `docs/superpowers/reports/2026-09-16-tabs-sizes-report.md`

### Windows of tabs (3.7b) — the sub-project whose failure is invisible by definition

- [ ] two windows open, the app closed and reopened: two windows come back, in their places,
      holding what they held
- [ ] **`main` closed while a second window lives**: the session keeps being written, and
      reopening the app afterwards comes back to what the survivor was holding. This is the whole
      reason 3.7b exists — before it, the session stopped being written from that moment, in
      silence, with the app alive in the tray to prove it.
- [ ] a minimized window is still in the document after a restart — what `labels()` was added for
- [ ] a window remembered on a screen that is no longer plugged in opens where it can be reached
- [ ] **a window restored on a scaled monitor is the size it was, not twice it** — the one bug the
      physical/logical asymmetry produces, and the one no test here can see
- [ ] a tab torn off into a new window, then the app closed: the new window comes back
      → `docs/superpowers/reports/2026-09-16-tabs-windows-report.md`

### A tab's own state (3.7a)

- [ ] a filtered, sorted, scrolled tab dragged into another window arrives filtered, sorted and
      scrolled where it was
- [ ] back and forward inside a tab restore the facets each entry was read with
- [ ] the app closed with several tabs open reopens them showing what they were showing, and
      going back in a restored tab lands on the screen's empty state — §6 of the spec says this is
      what bounding the document costs
- [ ] twenty tabs shrink, then the strip scrolls, the active one is always visible, and a tab can
      still be torn off a scrolled strip
- [ ] on `?catalog=none` and `?fixture=none` a restored tab still opens and says what it has
      → `docs/superpowers/reports/2026-09-16-tabs-own-their-state-report.md`

### Starting with Windows (B41, built 2026-09-16) — an **installed** build, a logout and a login

**Not a `pnpm dev` run**, and that is the point of the last line: the plugin is registered only
in a release build, on purpose. Straight from §7 of
`docs/superpowers/specs/2026-09-14-autostart-design.md`.

- [ ] switch on, then `reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v IsaacDome`.
      **Read the value, do not reason about it**: the path is written unquoted by the plugin, so
      an install under `C:\Program Files\…` is the case that has to be *seen* starting
- [ ] log out and back in: no window, the icon in the tray, and a run played immediately after is
      in the archive
- [ ] Task Manager → Startup apps → disable IsaacDome, then open the Background screen: the switch
      is off. **Then turn it on from the app**: it must refuse, and say to look in the Startup tab
      — this is the case that gave the error its second variant
- [ ] switch off, then off again: no error either time
- [ ] a `pnpm dev` run: the switch is disabled with its own line under it, and the registry is
      untouched
      → `docs/superpowers/plans/2026-09-16-autostart.md`

### The tear-off gesture (B15, built 2026-09-13 and never run)

**The oldest unseen work in the repo**, and the one open question is whether WebView2 keeps
delivering pointer events with the cursor outside the window. `lib/window/pointerSource.ts` is
written as if the answer were yes, behind an interface that is the only thing the other answer
changes. Eleven checks, from Task 17 of
`docs/superpowers/plans/archive/2026-09-13-drag-and-windows.md`:

- [ ] tear a tab off onto the empty desktop: a window opens under the cursor, sized like the origin
- [ ] drag it back over the first window's strip: the marker appears between the tabs, the release
      merges it there
- [ ] dock into a **second** secondary window, not only into `main`
- [ ] the last tab of a window dropped on the **desktop**: a no-op, nothing opens, nothing closes
- [ ] the last tab of a **secondary** window dropped on another window's **strip**: it joins, and
      the window it left closes — the two are not the same rule
- [ ] a secondary window whose last tab is docked elsewhere closes; `main` keeps a fresh tab
- [ ] two windows on the Plan: a move in one is visible in the other
- [ ] change the profile in one window: the other's indicator and screens follow
- [ ] change the scale in one window: the other follows
- [ ] two monitors at different scale factors: the drop lands where the cursor is, not offset
- [ ] the target window closed **while** a tab is in flight: the tab stays where it was, no crash
- [ ] cold `pnpm dev`: how long the first tear-off's preview takes, and the second

### The Roll screen (3.12) — what a save without a catalog can settle, and what needs one

A browser cannot mount it — `useRollStore` calls three commands through `invoke`, and there is no
fixture path for that outside Tauri here — so every line below needs the built app, and the last
one needs the game installed too.

- [ ] the layout
- [ ] the sentence the card shows for each diagnostic `RollDiagnostic` can carry
- [ ] the draw button in both states — idle and mid-draw
- [ ] the empty-deck state and its four sentences
- [ ] the preset panel's counts moving as a tick is toggled
- [ ] two windows agreeing through `roll-changed`: a draw in one is a read in the other
- [ ] **NEEDS GAME** — the character head and the mark's own symbol on the card. `headUrl` and
      `artUrl` come back `null` without a catalog, so only the fallback-outfit path has been
      looked at; this line stays open until a machine with the game draws the real ones

### One that is not a window, and is here because it is the same kind of answer

- [ ] **the game rewrites `savedatapath.txt` on every launch** (B57). The parser and the fallback
      are covered by tests; that the file is refreshed is the one sentence in the entry that no
      test here can hold. Start the game, then read the file's modified time.

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
