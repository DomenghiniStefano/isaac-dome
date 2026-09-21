# IsaacDome

Desktop app that reads the local files of *The Binding of Isaac: Repentance+* and answers
one question: **what am I missing, and what's worth playing tonight.**

It's not a personal tool. It ships with an installer and has to work with no configuration
on any Steam install of the game. The constraint "has to work at a stranger's house"
drives half the decisions below.

The full project document is in `docs/PROJECT.md`.

---

## Non-negotiable constraints

1. **Read-only on save files.** No write function anywhere in the module that opens the
   `.dat`, by construction. The checksum is never recomputed. A bug here destroys a
   stranger's profile.
2. **No API keys**, neither asked of the user nor embedded in the binary. Only public,
   credential-free APIs are allowed.
3. **No game assets in the package.** Images are extracted from the user's own copy at
   runtime. Same for datasets derived from the wiki, which ship with their own license and
   attribution. **This one is checked since 2026-09-20**, by
   `scripts/check-no-game-assets.mjs` inside `scripts/check`: no tracked picture, sound or
   game archive outside `crates/app/icons/`. It is a gate because for two weeks the promise
   was false and nothing looked — 6065 sprites cut out of the game sat in `design-export/`
   on a public repository, and they had to be removed from the history, not just the tree.
4. **No accounts, no backend, no telemetry.** The app works offline; the network is used for
   exactly two things, and both are the user's to switch off. **Optional dataset updates**, and
   since 2026-09-20 **the app's own updates** — one HTTPS request to `github.com` at launch,
   carrying an IP address and a user agent and nothing of ours. `autoUpdate` in `settings.json`
   is on by default and **off means the request does not happen**, not that it happens quietly.
   Everything about it is in [`docs/release.md`](docs/release.md); the design is
   `docs/superpowers/specs/2026-09-20-app-update-design.md`. A third use is a change to this
   constraint, and it gets written here.
5. **Degrade, never fail.** If a section of the save can't be read, the app still starts,
   shows what it knows, and flags what's missing.

## Stack

- **Frontend**: Vue 3 + TypeScript, Vite, **shadcn-vue** on **Reka UI**, **Tailwind v4**,
  **TanStack Table** (filterable grids), **TanStack Virtual** (long lists: 733 items,
  642 achievements), Pinia, Vue Router, vue-i18n, Lucide (`@lucide/vue`, **not**
  `lucide-vue-next`, deprecated).
  **Target stack, not all of it today's**: since the design system's first cycle
  (2026-09-10) `ui/` has Vue, Vite, Tailwind, `@tauri-apps/api`, shadcn-vue on Reka UI,
  Lucide, vue-i18n and Vitest; Pinia and Vue Router since the screens' first sub-project
  (2026-09-11); TanStack Virtual since 3.3a. TanStack Table was left out of 3.3a — its
  faceted values count an array cell as one value, and Unlock's facets are pure functions —
  and is reconsidered for Collection (3.4).
- **Backend**: Rust inside Tauri 2. Crates: `steamlocate`, `winreg` (fallback),
  `keyvalues-parser`, `quick-xml`, `notify`, `rusqlite` (bundled), `serde`. Tauri plugins:
  `single-instance`, `notification`, `dialog`, and — **registered in release builds only** —
  `autostart` and `updater`.
- **Tooling**: pnpm, Git Flow with `develop` as the integration branch.

**Rust does everything that touches disk. Vue only ever receives resolved JSON**: the
frontend knows nothing about offsets, file names, or log strings.

**Layout.** Sixteen Rust crates live in `crates/` — `core-save`, `discovery`, `unpack`,
`catalog`, `wiki`, `wiki-snapshot`, `graph`, `plan`, `run`, `log-watch`, `floor`, `roll`, `ipc`,
`store`, `app`, `test-support`. `log-watch` and `test-support` were missing from this list
until 2026-09-15; `design-export` was removed from the repository on 2026-09-20. The Tauri crate is
`crates/app`, not `src-tauri`: every `tauri` command needs
`--config crates/app/tauri.conf.json`, and the root scripts already do that (`pnpm dev`,
`pnpm build`). The frontend is the pnpm workspace `ui/`; from the root, `pnpm typecheck`,
`lint`, `scan`, `format:check`, `ui:test` are pass-throughs to `ui/`.

## Modules

**Drawn, in [`docs/architecture.md`](docs/architecture.md)**: four diagrams — the data flow from
the disk to the screens, the crate graph, the seventeen routes with the commands behind each, and
the build. It is the state and not the design, so the table below stays the authority on *what a
module is for*.

**It is kept up to date, and that is not a suggestion.** Its header pins five counts — 16 crates,
41 commands, 6 events, 17 routes, 6 store migrations — so a change that makes one of them wrong
makes the document wrong. **Redraw it in the same commit**, and five things trigger that: a crate
added or removed, a route added or removed from `RouteName`, a command joining or leaving
`generate_handler!`, an event in `crates/app/src/events.rs`, a migration in
`crates/store/src/migrations.rs`. Nothing enforces it — `scripts/check-doc-refs.mjs` catches a
**renamed path** and nothing else, so an edge that stopped existing leaves its arrow drawn on the
page and only a person ever notices. That is the whole risk: a diagram nobody maintains is worse
than no diagram, because it is read as true.

| Module | Responsibility |
|---|---|
| `discovery` | Finds Steam, the game, the saves. Manual fallback at every step. |
| `unpack` | Extracts the game's `.a` archives into the local cache. |
| `core-save` | Parser for the `.dat`, read-only. |
| `run` | Pure crate: `Tail` (bytes into lines, and a shorter file is a relaunch), the rules file that maps a line to one of eleven events and judges nothing, and the fold that makes every judgment a log cannot — the starting item, the active that replaced the last one, the outcome nobody wrote down. Item kinds arrive through a trait, so it never depends on `catalog`. |
| `log-watch` | The half of the archive that touches the disk: positional reads, which folders under `online_logs\` are sessions, `notify` on the **folder** (the game replaces `log.txt`, and a watch on the file goes deaf at the one moment that matters), and the ingest where **backfill and live are one function**. Every judgment is elsewhere: `run::resume` says whether a file is the launch we were reading, `run`'s fold says what a run is, `store` says what is kept. |
| `catalog` | Normalizes the game's XML files. |
| `wiki` | Pure crate: wikitext parser, typed tree, embedded compressed dataset. Only reads `dataset/raw/`. |
| `wiki-snapshot` | Tool, the only one that talks to the network: `pnpm wiki:fetch` / `pnpm wiki:build`. |
| `graph` | Pure crate: turns the wiki's typed requirements into prerequisite edges read from the game's own `unlocked_by` links, and evaluates them against a profile. Two rules files under `rules/`, embedded at build time — `requirements.json` generated by `pnpm graph:rules`, `corrections.json` written by hand. What it can't interpret says so (`Partial`), it never reads as "nothing in the way". |
| `plan` | Pure crate: the plan queue — an ordered series of achievements. One document, not a table: the order is the position in the array. A move never fails, it repairs — dependents are dragged, prerequisites are a wall — and it names the row it lands under (`move_after`), never an index: the view hides completed rows the document keeps. |
| `floor` | Pure crate: the 13x13 grid a player paints and the game's own documented secret-room placement rules, each carrying the sentence it was read from. Knows nothing about the log: the grid is the user's drawing. A rule it cannot evaluate on a painted grid says so (`Unmodelled`), it never reads as "anywhere is allowed". |
| `roll` | Pure crate: the space of targets the completion matrix defines, the deck a preset leaves in it once the excluded rows and columns are named, and the draw — a function of `(deck, seed)`, the clock kept out in `app` so the crate itself has none. The saved document is one preset plus the current draw, never more than that. It knows nothing about the game: no counts, and no index it did not receive — those stay where they were measured, in `ipc`. |
| `ipc` | Pure crate, no I/O: turns `discovery`, `core-save`, `catalog`, `wiki` and `graph` into JSON view-models. It's the only contract between Rust and Vue, generated into `ui/src/lib/ipc/types.ts` by `pnpm ipc:types` (never edited by hand). |
| `app` | Tauri wiring only: commands, managed state, `settings_file.rs`. Not tested. |
| `store` | SQLite, one file (`isaacdome.db`) with a versioned schema. Migration 1: the Plan's goals. Migration 2: the plan queue, as one JSON document. Migration 3: the window session. Migration 4: the run archive — `sources`, `events` (which *are* the archive) and `runs`, a cache carrying the rules version that produced it. A session's key is its folder name; a launch of `log.txt` has **no** name, so its key is `NULL` and two launches are two rows. Migration 5: `folded_rules_version` on the source, because zero cached runs have nowhere to carry one — a source folded into nothing must not read like one nobody folded. Migration 6: `roll`, one pinned document holding the preset and the current draw together — a second row would be a second answer to "what is the preset". Snapshots arrive as a later migration. |
| `test-support` | Dev-dependency only. Access to `samples/` for tests on real data: every function **declares** on stderr which file it used (`sample: …`) or why it skipped (`skip: …`). No real test opens `samples/` by hand. |

---

## What the project knows about the game

Measured, not documented — nobody publishes any of this. It lives in three documents of its own
since 2026-09-16, because it is 224 lines that most sessions never touch and this file is loaded
into every one of them. **The rule stays here; the detail is one `Read` away.**

- **[`docs/save-format.md`](docs/save-format.md)** — the `.dat`: sections, the counters and the
  completion marks, the bestiary. Verified on 41 real saves over fourteen months.
  > **The entry count is read from the file, NEVER hardcoded.** A 2025 save declares 641
  > achievements and a 2026 one 642: a patch added one, and any hardcoded count breaks itself.
  > **A section's name is structural or measured, never taken from a log line** — 3 and 6 carried
  > wrong labels for months on exactly that evidence.
- **[`docs/log-format.md`](docs/log-format.md)** — `log.txt`, the eleven events, the floor-generation block, and the three kinds
  of seed line.
  > **The `[INFO] - ` prefix is part of the line**, and about 1% of lines carry no prefix at all:
  > **no pattern may be anchored at the start of a line**, or it matches nothing.
  > A `Continue` is a run *resumed*; reading one as a fresh start counts a run twice.
- **[`docs/paths.md`](docs/paths.md)** — where Steam, the game and the saves actually are, on the
  machines this has been tried on.
  > With Steam Cloud on, the save is **not** in the Documents folder. `online_logs\` is **not
  > flat**, and reading it as flat finds nothing.

## The board

**Every change passes through Trello, and the board is updated as the work happens — not
afterwards.** The board is `IsaacDome` (`6aabb64f6116b936f1f8a762`); `get_active_board_info`
answers 401, so pass the `boardId` explicitly. What the lists mean, what the labels are and why
the state left `docs/BACKLOG.md` is in that file's *"Where the state lives"* section — **the rule
is here, the detail is one `Read` away.**

Three moves, and none of them is optional:

- **Taking a card on moves it to `In Progress`**, before the first line of code. A card still in
  `BACKLOG` while its branch exists is a board that has stopped describing the work, and the
  board is where "what do I do next" is read from — by the next session, and by the owner.
- **Something learned is a comment on the card, the same day.** A decision taken, a constraint
  measured, a deviation from the spec, a defect found in the plan itself: the card carries it.
  A session's memory does not survive the session; the card does.
- **Finishing moves it to `UAT`, not to `Done`.** `Done` is for work that has been *looked at*.
  A merged branch with a green `pnpm check` is built, not seen — and on a machine without the
  game installed, half of what a screen shows cannot be seen at all. Tick the checklist items
  that are really done, leave open the ones that are not, and say in a comment what is still
  unverified and what would settle it.

The checklist on a card is the plan's task list: tick an item when its task is reviewed and
committed, never when it is merely written. A card whose checklist is fully ticked and whose
last item is a window nobody opened belongs in `UAT` with the `NEEDS WINDOW` or `NEEDS GAME`
label, and the open item is the honest record of why.

## Code rules

### Rust and the IPC boundary

- **Tauri commands**: `Result<T, IpcError>`, never `Result<T, String>`. The error crosses
  the IPC boundary and the frontend has to be able to tell cases apart without parsing
  text.
- **Every struct that crosses the IPC** has `#[serde(rename_all = "camelCase")]`. Without
  it, TypeScript reads `undefined` and nobody notices.
- **Enums on the IPC**: tagged with struct variants (`#[serde(tag = "kind")]`), never
  newtype. Enums we define ourselves use `rename_all = "camelCase"`; the ones that come
  from domain crates (`SavePrefix`, `Edition`, `Dlc`, `Kind`) keep their own `snake_case`
  and the TypeScript types mirror that.
- **One of our fieldless enums isn't tagged: it's a bare camelCase string**
  (`"passive"`, `"repentance"`), and the TypeScript type is a union of values. The tag
  exists to distinguish variants that carry different data; without data it would add a
  key per row and hide the fact that the field is a value, not a discriminator. The moment
  a variant gains a field, the enum becomes tagged — and the TypeScript changes with it, so
  the rule applies to the whole enum, not per variant. **Zero exceptions in the repo**:
  `ItemKindView`, `OriginView`, `StepsBasis`, `CandidateSource`, `MissingReason`, `StatusView`.
  A tagged unit enum showing up again is a bug, not an alternative style — two conventions for the
  same thing means a TypeScript `switch` silently falls into no branch.
- **`rename_all` on an enum does NOT rename the fields inside its struct variants.** It
  renames the variant names. Fields need `rename_all_fields = "camelCase"` **in addition**,
  otherwise `Active { auto_selected }` comes out as `auto_selected` and TypeScript reads
  `undefined` with no error at all. Same failure mode as forgetting `rename_all` on a
  struct, and just as silent: pin it with a test on the JSON shape.
- **Don't cross the IPC boundary**: file paths, offsets, raw bytes. Only already-resolved
  view-models; candidate saves travel with an **opaque id**.
- **Never `format!("{:?}")` to push a type across the IPC.** `Debug` prints *every* field,
  including the ones the boundary forbids: a `PathBuf` in a diagnostic carries the Steam
  account id under `userdata\` and always the Windows username. Every type that goes out
  has to be mapped onto an enum or struct **we define ourselves**, carrying only what's
  needed. On top of that, a `Debug` string isn't translatable, while a tagged enum is.
- **Expected cases aren't errors.** Steam missing, game not installed, an unreadable
  section travel in the payload as readable diagnostics. `Err` is reserved for when the
  command can't answer at all.
- **Never `panic!` or `unwrap()`** outside tests, on data read from disk. A malformed or
  truncated file degrades; it doesn't kill the process.
- **No game file is ever loaded whole into memory.** The `.a` archives are ~1.3 GB total:
  keep the index in RAM and read an entry's bytes when needed. This applies to any new
  source too (`log.txt` in append mode, the backups in `save_backups\`): the file size is
  the user's choice, not ours, and "degrade, never fail" starts with not asking for a
  gigabyte.
- **Shared state across commands is read positionally.** Two Tauri commands can run
  together: on a shared `File`, `seek` + `read` is a race on the cursor, so use
  `seek_read` (Windows) / `read_at` (Unix). This applies whenever a handle lives in
  `tauri::State` instead of inside a function.
- **Expensive resources live in `tauri::State`, opened once** (`CatalogState`,
  `ResourcesState`, `StoreState`). But **an expected failure is never cached**: if the game
  turns out not to be installed, "absent" isn't cached, otherwise someone who installs the
  game with the app open has to restart it to see their data.
- **If a return value is worth checking, it lives in a pure crate.** The Tauri crate keeps
  only the wiring, which isn't tested.
- **Exhaustiveness is mandatory**: no `_ =>` arm on a closed enum. Adding a variant has to
  break the build, not silently produce an empty result.

### Frontend → `docs/frontend-conventions.md`

Five non-negotiable rules, the rest is in the document:

1. **No `<style>` in SFCs** — the only exceptions are `-webkit-app-region`, custom
   `@keyframes`, scrollbar overrides. Dynamic values come from CSS variables bound by the
   template, not from inline pixels.
2. **No hardcoded visual constants** — no `w-[48px]`, `opacity-50`, `duration-150`, no
   `:size="16"` on an icon: every value is a token in `@theme`, declared once in CSS
   (Tailwind v4: no longer also in a config file). `@theme` can span multiple files
   imported from `main.css`, one per token family; what never gets duplicated is the
   token.
3. **No `invoke()` in components** — only typed wrappers in `ui/src/lib/ipc/`. The same for
   windows: **nothing outside `ui/src/lib/window/` imports `@tauri-apps/api`'s `window`,
   `webviewWindow` or `event`**, and every one of those modules degrades outside Tauri instead
   of throwing — `listen()` does not answer "no", it explodes inside whatever hook called it.
   **One composable drags every list** (`useDragList`, decisions in `lib/drag/dragList.ts`): a
   screen brings where its items are and what a drop there means, nothing else.
4. **No raw `<button>` / `<input>`** — use the primitives in `ui/src/components/ui/`, and
   extend them with a prop instead of styling by hand.
5. **No string unions** — `const X = { … } as const`, never `type X = 'a' | 'b'`. Also
   applies to the wire types in `ui/src/lib/ipc/types.ts`: there the distinction between
   value and discriminator is the *Rust* rule, which decides how it serializes, not how the
   TypeScript is declared. The only exception is a tagged union's tag itself
   (`b.kind === 'paragraph'`).

No linter enforces these rules: `ui/scripts/scan-conventions.mjs` (`pnpm scan`) does,
covering all five as of 2026-09-06, plus the ban on visible strings in the template.
Exceptions live in the `EXEMPTIONS` array at the top of the script, per file and with a
reason. Every new rule has to be added there too, otherwise the document promises a check
that never happens.

### Tests

- **Test-first** for code with logic. The expected value comes from the spec, **never
  from the code's current output**.
- A failing test is **first and foremost a hypothesis of a bug in the code**, not an
  expectation to fix.
- Tests on real data **skip with a note** if the sample is missing: `samples/` is
  git-ignored and the suite has to stay green for anyone who clones the repo. Watch the
  reverse too: an "N passed" doesn't say how many were skipped — **nor how many disappeared**.
  On 2026-09-12 a commit added `crates/ipc/tests/progress.rs` and deleted
  `crates/ipc/tests/profile.rs` in the same diff, 647 lines and 27 tests of it, and every gate
  stayed green for four days: **deleting a test fails nothing**. **It fails something since
  2026-09-16** (B63): `scripts/check` totals the tests that *exist* — passed plus failed plus
  ignored on the Rust side, Vitest's parenthesised total on the other — and compares both against
  `scripts/test-floor`. A suite that shrank fails the run; one that grew prints the line to paste,
  because failing on a rise would fail every commit that adds a test. **What it still cannot
  see** is a floor nobody raises: two hundred tests added and fifty later deleted stays quiet, and
  closing that needs a per-commit comparison, which is CI. So a test file that leaves in the same
  commit that adds another is still the shape to look for in a diff.
- **`samples/` is only opened from the `test-support` crate**, never by hand with
  `env!("CARGO_MANIFEST_DIR")`. Its functions always declare the outcome on stderr —
  `sample: <file>` when there is one, `skip: …` when there isn't — because a test on real
  data has to say **which slice of the domain it actually ran on**. This isn't theory: the
  `unpack` tests all ran on `config.a`, the only archive the decompressor could open, and
  the main function was broken with a green suite.
- **"The tool isn't there" is verified by making it answer, not by checking whether the
  command starts.** On Windows, `python` with no Python installed is a *Microsoft Store
  alias*: it starts, prints "install from the Store" and exits with 49. An `Err` from
  `Command::new` never comes; a test that's supposed to tell "absent" from "broken" ends up
  saying "broken". Ask the tool to print a word of our choosing first, and only trust it if
  it prints it.
- **Pinned numbers are a fixture of an era, and the era belongs in the file name.** A
  sample is named `YYYYMMDD.…`; a test that compares two eras names both. Where the
  expected value can be read outside our own code — `od` on the header, the Python
  reference — the comment says how it was derived.
- **On a historical series, write properties, not values**: "the diff reports exactly the
  bits that flip", "the progression never regresses". These hold even when the game
  changes, and they find boundaries a pinned value can't see — that's how the 641-to-642
  slot jump surfaced, where a slot that *didn't exist before* counts as off, not as outside
  the comparison.
- Before declaring anything done: **`pnpm check`** (i.e. `scripts/check`), which runs
  `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --workspace`,
  `pnpm typecheck`, `pnpm ui:test`, `pnpm lint`, `pnpm format:check`, `pnpm scan`,
  `scripts/check-no-game-assets.mjs`, the IPC contract's regeneration,
  `scripts/check-doc-refs.mjs`, and the test-count floor. **There's
  no CI**, by choice: the list of commands lives in that script and nowhere else. The two
  fast ones also run in the pre-commit hook (`git config core.hooksPath scripts/git-hooks`).
- **The document reference report is a report, and never fails the run.** It checks every file
  path the living documents name against the files git tracks, and it exists because on
  2026-09-15 the same defect turned up in all of them: a document names a file, a refactor
  renames it, and nothing notices. Two things it cannot do, which is why it does not gate — it
  cannot tell a name written as *history* from one written as a *promise* (a rename record
  **must** name the file that went), and it cannot read a condition ("only if the spike says
  so"). **The signal is the delta**: what it already knows is listed with a reason each, like
  `scan-conventions.mjs`'s exemptions, so a `NEW` line is a document that drifted and a `GONE`
  line is an exemption to delete.
- Skips on real data print `skip: …` on stderr and pass, but **`cargo test` alone doesn't
  show them**: the harness hides output from passing tests. You need
  `cargo test --workspace -- --nocapture`, which is what `scripts/check` runs before
  counting lines. `samples/packed` is a junction to the installed game's `resources\packed`
  folder, and without it dozens of `unpack`, `catalog` and `ipc` tests skip silently.
- Frontend: Vitest (`pnpm ui:test`) for the logic in `ui/`, test-first like the Rust side;
  presentation is checked on the development-only Kit page (`pnpm ui:dev`, `#kit`).

### Measuring on real data

Most of what this repo knows about the save format was measured, not documented. These
rules are what the measurements cost when they were skipped — each one has an incident
behind it, named so the rule can be argued with.

- **A fact that decides what *not* to measure has to be re-measured before it's trusted.**
  On 2026-09-08 one window said a co-op run left the personal save alone, and that became
  "co-op sessions are useless as evidence". It was wrong, and for two weeks it made every
  co-op session — there were 21 — look not worth instrumenting. A claim of the form "X
  can't tell us anything" is the most expensive kind to get wrong, because nothing after it
  ever tests it. Re-measure it the first time it would save you work.
- **"Nothing moved" is only true if you looked at every section.** That same error came from
  reading "no achievement moved" as "nothing moved": the sections with names were checked,
  the four still called `Unknown` weren't. `matched_window` exists so the whole file is read
  at once — use it instead of `SaveDiff` when the question is whether *anything* changed.
- **A negative result is a result, and silence is not one.** An instrument that reports
  nothing proves nothing until it has been shown able to speak. Before reading a flat line
  as evidence, make the same instrument report a change you already know about.
- **A property over the series needs a vacuity guard.** `bit 2 implies bit 0` holds trivially
  on a profile that has no bit 2 yet — which is every profile at the start. Assert that the
  series actually contains the thing the property is about, the way
  `the_three_located_columns_are_not_dead_cells` guards its neighbour. A test that cannot
  fail is worse than no test: it reports coverage that isn't there.
- **Anything that walks `samples/` uses `test_support::is_dated`, never its own filter.** A
  looser filter picked up a same-day `20260912-pre.…` snapshot, won the dedup because `-`
  sorts before `.`, and compared against the wrong end of the window — no error, just a
  plausible wrong answer. Snapshots that are not points in the series (the "before" half of
  a matched window) live in `samples/windows/`, not beside it.

### Wiki dataset

`dataset/raw/`, `dataset/wiki.json` and `dataset/corrections.json` are committed together:
the `derived` test in `crates/wiki` enforces `wiki.json == build(raw, corrections)`, so the
three can never drift without the suite noticing. The snapshot (`pnpm wiki:fetch` +
`pnpm wiki:build`) is **one pass per release**, never from the app:
`wiki::Dataset::embedded()` only reads the derivative embedded at build time, it never
talks to the wiki at runtime. Source attribution (wiki, URL, license, snapshot date) lives
in `dataset/ATTRIBUTION.md`, CC BY-SA 4.0: it ships in the package.

### Commits

- Conventional Commits, **`type(scope): subject`** — `feat(core-save): …`,
  `fix(discovery): …`, `chore(dataset): …`. The scope is the crate or package
  (`core-save`, `discovery`, `unpack`, `ipc`, `ui`, `wiki-snapshot`);
  drop the parentheses when the change is repo-wide (`docs:`, `chore:`, `build:`).
  Messages in English, atomic commits.
- Integration branch: **`develop`**. Work lands there and stops there: finishing a sub-project is
  merge, push, and done.
- **`master` is the release branch, since 2026-09-20.** It is fast-forwarded to `develop`
  (`--ff-only`) **at the moment of a release**, and the tag goes on it; between releases it sits
  at the last one. So `master` is what somebody who opens the repository gets, and what they get
  is the version they can actually download — which is the whole reason it is not just a mirror
  of `develop`.
  **Two earlier readings, both paid for.** It once said "`master` only receives releases" while no
  release existed, and since `master` is GitHub's default branch the landing page sat 733 commits
  behind on an Italian scaffold. It was then frozen outright on 2026-09-17, which was right while
  nothing shipped and stopped being right on 2026-09-20, when four releases went out in an evening
  and every one of them needed the branch moved. `docs/STATUS.md` has the moves; `docs/release.md`
  has the procedure.
  Nothing enforces any of this: no hook, no branch protection, by decision, the same way there is
  no CI. It holds because it is read.
- **A merged branch is closed in the same breath as the merge**, locally and on the remote —
  unless work continues on it, which is the only exception. A branch that is merged holds nothing
  `develop` does not, *by construction*, so keeping it buys no safety and costs the one thing that
  matters: it hides the branches that **do** carry something. Measured on 2026-09-16, before the
  first cleanup: **24 local branches and 52 on the remote**, and of all of them exactly one held a
  commit that was not in `develop` — finding it meant checking seventy-six by hand.
- **Verify a branch is merged at the moment you delete it, never from a list gathered earlier.**
  Two conditions, both of them: nothing outside `develop`
  (`git rev-list --count develop..<b>`) and nothing outside the remotes
  (`git rev-list --count <b> --not --remotes`). The second is the one that catches a branch whose
  commits live only on this machine, and it is the reason the 2026-09-16 sweep could delete
  seventy-odd refs without losing a line.
- **After a history rewrite, ancestry stops being the test — compare the content.** On 2026-09-20
  the sprites left the *history* and not only the tree (constraint 3), which gave every commit on
  `develop` a new hash. A ref that was not rewritten with it keeps the old ones, so
  `git rev-list --count develop..<b>` reports it as unmerged **for ever**, and the rule above then
  keeps a branch alive that holds nothing. Measured on 2026-09-21: `feature/app-update` counted 1
  and `feature/roll` 2, while the only thing those commits added —
  `docs/superpowers/specs/2026-09-20-app-update-design.md` and
  `docs/superpowers/specs/2026-09-17-roll-design.md` — sat on `develop` as **byte-identical blobs**,
  under the same subjects at `2f51ed6d` and `20f7e72c`. So when the count is non-zero after a
  rewrite, ask what the commits actually *add*: `git rev-parse <commit>:<file>` against
  `git rev-parse develop:<file>`, equal blob ids meaning the branch carries nothing. All three
  branches were deleted on the remote that day, and `origin` now holds `develop` and `master` only.
- **Never** a `Co-Authored-By` trailer or references to Claude, in any commit, PR, or
  issue.

## Don't

- Don't use `nom` for the `.dat`: it's three integers and a slice of bytes,
  `u32::from_le_bytes` and slices are enough.
- Don't hardcode counts of achievements, items, challenges, or characters.
- Don't open saves in write mode, for any reason.
- Don't introduce a "complete" component library (PrimeVue, Element Plus, AG Grid): the
  choice is shadcn-vue precisely to keep the components in the repo.
- Don't assume REPENTOGON is installed: it's an optional bonus, never a promised feature.
- Don't open the screen on a grid of items — that's already the game's own menu.
- Don't convert files to CRLF or touch `.gitattributes`: the repo forces LF in the working
  copy on purpose, otherwise `pnpm format:check` is red on Windows for otherwise-correct
  files.
- Don't add a CI pipeline: **it's a decision**, not an oversight. The checks live in
  `scripts/check` and the pre-commit hook. If that choice ever changes, that's already the
  list a pipeline would run.
- Don't make multi-line edits to Rust code with `perl -0777 -pe 's|…|…|'`: the `|` that
  delimits the substitution is also the one for closures, and the result is a file to
  restore from git. For a block of code, use a targeted text editor; regexes stay for
  single-line substitutions.
- Don't name a section, a bit, or a tally from a guess — and don't leave one named from a
  guess once it's been measured. Sections 3 and 6 carried wrong names for months;
  a mark's bit 2 was "third level, meaning not confirmed" until a matched window said
  "won online". `Unknown` costs nothing and a wrong label costs a re-derivation.
- Don't run `cargo test --workspace` while `live_probe` runs **in the same profile**: the
  example's exe is held open and the link fails with `LNK1104` (seen 2026-09-08). Running
  the probe with `--release` leaves the debug build `scripts/check` uses free, which is the
  cheap way to keep measuring while the suite runs — verified 2026-09-12. Otherwise stop the
  probe and restart it after; the `.dat` watcher covers the gap.
- Don't start `pnpm dev` while a build of the app is sitting in the tray: since 2026-09-13 the
  app survives its last window and `tauri-plugin-single-instance` hands the launch to the
  process that is already there — the new one brings the old window forward and exits, with no
  error and no hint that the code you just wrote never ran. An orphaned vite is the same trap
  from the other side: it is pinned to 1420 by `strictPort`, because `devUrl` names that port
  and nothing else, so the second one can't step aside, only die — and it is a grandchild under
  two `cmd.exe`, which is why killing the session that started it orphans it instead of ending
  it. **Since 2026-09-14 `pnpm dev` clears both on its own**: `predev` runs `scripts/dev-reset.mjs`,
  which ends the tray app and whoever holds 1420 — the latter **only if that process's command
  line points inside this repo**, otherwise it says who has the port and leaves it alone. It
  never fails the build; it is a cleaner, not a gate. What it cannot do is tell your leftovers
  from another session's, so on a machine running several, `pnpm dev` now evicts a dev server
  somebody else is using, in silence. `pnpm dev:reset` is the same thing by hand.

  **Since 2026-09-16 `predev` runs a second cleaner**, `scripts/prune-incremental.mjs`, which
  empties `target/debug/incremental` **once a week** and is silent on every other launch. It is
  there because nothing else does it: Cargo's automatic garbage collection is stable since 1.88
  and cleans `~/.cargo` only, collecting `target/` is still an open issue (rust-lang/cargo#13136),
  and `cargo-sweep` is unmaintained *and* leaves `incremental/` alone by its own issue #50. Left
  to itself the folder reached **16.9 GB across 76,692 files**. Incremental is kept rather than
  disabled because it is worth having — 9s against 15s on `cargo clippy --all-targets` after
  touching one crate, measured both ways round — so the week after a prune costs a few seconds on
  the first build and nothing after. `pnpm prune:incremental` forces it and says what it did;
  `ISAACDOME_INCREMENTAL_MAX_AGE_DAYS` moves the cadence.
- Don't trust a suite run from a **second worktree** until you have put `samples/` back. Its
  *contents* are git-ignored while `samples/.gitkeep` is tracked, so `git worktree add` gives
  you the folder and nothing in it: not "missing, and you notice" but **"present, and it looks
  right"**, which is the worse of the two. Every test on real data then skips there —
  **silently, and still passing**, because a skip passes. Measured on 2026-09-15, same
  branch, same 929 passed either way: in a fresh worktree **79 `sample:` lines became 0** — not
  one test touched real data — while the skips went 129 → 175. The passed count says none of
  it. That is exactly the shape of "green suite, nothing verified" that D3 cost days for.
  Junction it the way `samples/packed` already is
  (`New-Item -ItemType Junction`), and read the count from `ISAACDOME_TEST_DECLARATIONS` —
  `scripts/check` does, because merging `--nocapture` stdout with `test-support`'s stderr
  splits lines and makes a hand-rolled `grep -c '^skip:'` wobble by several either way.
- Don't delete a worktree before unlinking its junctions. The entry above tells you to junction
  `samples/` into every worktree, which turns the cleanup command into a destructive one: both
  `git worktree remove --force` and `Remove-Item -Recurse` can follow a directory junction and
  take the **target's** contents with them — here, the fourteen months of saves that cannot be
  re-collected, and whose absence the suite would report as a skip. Seen on 2026-09-15 while
  tidying up, one command short of it. Remove the reparse point on its own
  (`[System.IO.Directory]::Delete($path, $false)` deletes the link, never what it points at),
  count the target's files either side, and only then delete the tree. List the rest first with
  `Get-ChildItem -Recurse -Force -Attributes ReparsePoint` and check that none points outside
  the worktree — pnpm's `node_modules` holds ~600, all internal, and it is the one that isn't
  that matters. One more thing to know: `git worktree remove` deletes the administrative files
  **before** the directory, so a removal that fails on "Directory not empty" leaves an orphan
  `git worktree list` no longer shows. Finish it by hand, then `git worktree prune`.
- Don't commit by `git add -A` on this repo: specs under `docs/superpowers/` are edited in
  parallel by other sessions, and a clean `git status` at the start of a session is no
  promise it's still clean at the end. Stage by explicit path, and say so when the tree
  holds changes that aren't yours.
- Don't *merge* a clone that predates a history rewrite — **reset it**. A machine whose last pull is
  older than 2026-09-20 holds the pre-rewrite commits, so a pull sets two histories against each
  other that share no ancestor for the same work, and every file both sides touched comes back
  `UU`: on 2026-09-21 that was `CLAUDE.md`, `Cargo.lock`, `scripts/check`, `package.json` and some
  fifty more, with conflict markers landing **inside `CLAUDE.md` itself** — the session read its own
  instructions with `<<<<<<<` in them. The resolution for every one of those files was "take the
  remote", which is a sign it was never a merge. `git fetch origin --prune` then
  `git reset --hard origin/<branch>` per branch is the whole operation; for a branch that is not
  checked out, `git branch -f <b> origin/<b>` moves it without disturbing the worktree. `master`
  was 790 ahead and 940 behind, and those 790 were the purged history, on no remote at all.
  **`git clean -x` is not part of it, ever**: `samples/` is git-ignored, and `-x` deletes the
  fourteen months of saves along with `node_modules`. Re-install after the reset (`pnpm install`,
  `cargo fetch`) — the lockfiles moved with the tree.

## Test data

Put a real save in `samples/` (the folder is git-ignored) and use it as the reference for
the parser's tests. Ideally two different dates, so the diff gets tested too — which is
the mechanism the self-updating plan rests on.

Samples are named with the date, `YYYYMMDD.rep+persistentgamedata1.dat`: the pinned
numbers in the tests are a fixture of a known era, and a file named "live" invites
overwriting it. A test that looks up a sample by name and doesn't find it skips with a
note, it doesn't fail.
