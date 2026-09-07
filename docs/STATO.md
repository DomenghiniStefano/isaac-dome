# Progress status — IsaacDome

Single source of truth for the project status. Updated every session.
Narrative summary and design decisions live elsewhere: `docs/PROJECT.md` (project),
`docs/superpowers/specs/` (module design), `docs/superpowers/plans/` (plans and reports).
The quality tasks that came out of the 2026-09-05 review (local verification, scanner, IPC
contract, memory, test data) live in `docs/MIGLIORIE.md`, with closing criteria and order.

**Integration branch:** `develop`. `master` is stopped at the initial commit.
**Wiki dataset merged** into `develop` on 2026-09-06 (`feature/wiki-dataset`, 29 commits,
suite green on the merge result, review of the whole branch closed). The local branch was
deleted; on origin its last published version remains.
**Last update:** 2026-09-08

---

## Milestones

- [x] **M0 — Format spike**
      `.dat` format decoded and verified on 28 real saves, working Python parser,
      counters labeled, marks matrix rebuilt, log verified.
- [ ] **M1 — Rust parser, discovery, unpack, Completion screen** ← in progress
- [x] **M2 — Unlock graph** (2026-09-07). The Unlock *section* is frontend work and
      waits for the design system; the graph behind it is done — report in
      `docs/superpowers/plans/2026-09-07-unlock-graph-report.md`.
- [ ] **M3 — Derived plan** ← in progress. The **plan queue** is done (2026-09-08): an
      ordered series of achievements whose order is yours and can never contradict the
      graph. Report in `docs/superpowers/plans/2026-09-07-plan-queue-report.md`. What
      remains of M3 is the screen, which waits for the design system.
- [ ] **M4 — Log watcher and run archive**
- [ ] **M5 — Public release**

---

## M1 — detail

Every module follows the same cycle: design spec → TDD plan → implementation →
report → post-review fixes.

### `core-save` — `.dat` parser, read-only ✅

- [x] Design spec — `docs/superpowers/specs/2026-09-01-core-save-parser-design.md`
- [x] TDD plan — `docs/superpowers/plans/2026-09-01-core-save-parser.md`
- [x] Implementation — `crates/core-save/` (`parse.rs`, `section.rs`, `diff.rs`)
- [x] Report — `docs/superpowers/plans/2026-09-01-core-save-parser-report.md`
- [x] Post-review fixes
- [x] `Cargo.lock` checked in for reproducible builds (`da615dd`)

### `discovery` — finds Steam, the game, the saves ✅

- [x] Design spec — `docs/superpowers/specs/2026-09-01-discovery-design.md`
- [x] TDD plan — `docs/superpowers/plans/2026-09-01-discovery.md`
- [x] DLC → edition map
- [x] Save file name parsing
- [x] `appmanifest .acf` parsing (installdir + DLC)
- [x] Save scanning (multi-account userdata, Documents, override)
- [x] Steam location (`steamlocate` + `winreg` HKCU fallback)
- [x] `discover()` orchestration + integration test on the real machine
- [x] Report — `docs/superpowers/plans/2026-09-01-discovery-report.md`
- [x] Post-review fixes (registry fallback, malformed `.acf` fallback, degradation)

### `unpack` — targeted extraction from `.a` archives ✅

- [x] Design spec — `docs/superpowers/specs/2026-09-01-unpack-design.md`
      (ARCH000 format, djb2 + FNV hashing — **the spec needs a correction**: it gives the
      version as a constant, while byte `0x07` is actually the compression mode)
- [x] TDD plan — `docs/superpowers/plans/2026-09-01-unpack.md`
- [x] Scaffold, public types, hashing
- [x] Report — `docs/superpowers/plans/2026-09-01-unpack-report.md`
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
- [x] TDD plan (A) — `docs/superpowers/plans/2026-09-03-catalog-a.md`, 10 tasks
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
- [x] Report — `docs/superpowers/plans/2026-09-03-catalog-a-report.md`
- [x] **Plan B** (prepares M2, and **closes the static data base**): `metadata.rs`
      (quality and tags, actually in `items_metadata.xml` not `items.xml`); `achievements.rs`
      with the unlock-condition comments and the `achievement="N"` link from items;
      `itempools.rs` (`Item.pools`); `challenges.rs`; `bossportraits.rs`; `origin`
      (source DLC from id ranges); cross-check against the save (721
      collectibles out of 733 slots, 11 unused ids identified; 637 achievements out of 642
      slots). Found and fixed a real bug in the challenge parser (mixed separators,
      negative ids); fixed four measurement numbers that were wrong in the brief, which had
      propagated into the spec. 81 tests in the `catalog` crate (51 unit, 15 on `build.rs`, 15 on `real_data.rs`).
- [x] TDD plan (B) — `docs/superpowers/plans/2026-09-04-catalog-b.md`, 8 tasks
- [x] Report (B) — `docs/superpowers/plans/2026-09-04-catalog-b-report.md`
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
- [x] TDD plan in 12 tasks — `docs/superpowers/plans/2026-09-02-app-shell-ipc.md`
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
- [x] TDD plan in 12 tasks — `docs/superpowers/plans/2026-09-05-wiki-dataset.md`
- [x] Report — `docs/superpowers/plans/2026-09-05-wiki-dataset-report.md`
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
- [ ] Resolver and parser polish deferred from the review (entity aliases; a
      named-content template that opens on a list line and closes many lines below,
      invisible to `blocks.rs`, which parses each entry on its own; a table cell
      split on `||` without counting `{{…}}` braces): listed by theme in the execution
      report, "What's left out" section.
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
- [ ] shadcn-vue, Reka UI, Pinia, vue-i18n, TanStack — deliberately out of the first step:
      they arrive with the design system, and getting ahead of it would mean guessing the
      tokens
- [ ] Design system (in progress on Claude Design, outside this repo)

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
      (`docs/superpowers/plans/2026-09-05-graph-contracts.md`) executed the same day,
      report in `docs/superpowers/plans/2026-09-05-graph-contracts-report.md`. Types in `ipc`
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
- [ ] **4. Handoff to Claude Design** — screens 0, 4 and 5 on real data, the other four
      on fixed contracts. From here M2 proceeds in parallel with the webapp without touching the
      types the frontend consumes.
      **The material has been ready since 2026-09-06**: `DESIGN-BRIEF.md` rewritten around the two
      sections (Wiki and Progress) and the `design-export/isaacdome-design-pack/` package, 423 files, generated
      by `pnpm design:export`. What's left is the actual handoff, which is an action outside the
      repo.

Doesn't block the handoff but blocks Collection and Unlock: **`Archive::open` loads 1.3 GB**
to extract sprites (see open blockers), and the graph commands do this on every
call. Needs solving before a screen asks for a hundred icons at once, i.e. before
building the Collection screen, not before designing it.

---

## Open blockers

- [x] ~~**`Archive::open` reads the whole file into memory.**~~ **Resolved on 2026-09-06** (C1
      in `docs/MIGLIORIE.md`). `open` reads the header and the index and keeps the `File`
      open; a resource's bytes are read on demand, with positional reads
      (`seek_read` / `read_at`), because the `ResourceSet` is shared across commands and a
      shared file cursor would be a race. An entry's data upper bound is the next offset
      after it: the compressed length isn't stored anywhere in the format.
      Opening all eight archives (19,473 entries) went from ~1.3 GB to a **1.4 MB** peak, measured
      by a global allocator in `crates/unpack/tests/streaming.rs`; the spec's stated
      threshold is 64 MB. On the `app` side, `ResourcesState` opens the set once for all
      commands (`extraction_report`, `unlock`, `next_steps`, `plan`, `add_goal`,
      `remove_goal`).
- [ ] **`unlock` serializes 641 nodes with base64 icons inline in each row**, i.e. a payload
      of megabytes on every call. Fine for the verification screen, not for the real
      one: the final Unlock screen will want icons separated from the rows (fetched
      separately, only for visible elements) instead of embedded in every node. This is C2 in
      `docs/MIGLIORIE.md`, and it changes the IPC contract: do it when the real frontend
      begins, so the TypeScript type only changes once.
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
- [x] ~~**Isaac isn't installed on this machine.**~~ **Resolved.** The game is installed at
      `D:\SteamLibrary\steamapps\common\The Binding of Isaac Rebirth`, Repentance+ edition,
      with all the `.a` files in `resources\packed`. `discovery` finds it correctly on the
      second library and isn't confused by the leftover on `C:`.
- [ ] **`samples/` doesn't contain the saves from the M0 collection.** The 28 saves over 14 months
      used to decode the format aren't on this machine. Only the local Steam Cloud profiles
      are present (June 2024 and January 2025).

## To investigate

- [ ] **Backlog of registered, not-yet-started tasks: `docs/BACKLOG.md`** (2026-09-05).
      ~~B1 analysis on sources for item effects~~ **closed on 2026-09-05**: the
      source is wiki.gg (CC BY-SA 4.0, not the fandom copy), dataset in the build, report in
      `docs/superpowers/plans/2026-09-05-b1-fonti-effetti-report.md`. **Implementation closed
      the same day** (at night): crate `wiki`, tool `wiki-snapshot`, embedded dataset —
      report in `docs/superpowers/plans/2026-09-05-wiki-dataset-report.md`. All that's left is the
      design of the screen that shows the text. ~~B2 challenge rewards~~ **closed on
      2026-09-05** (three forms in the
      file, not one: see the entry); B3 lists and search after design; B4 is M2.
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
- [ ] **`discovery` and the leftover game folder.** Verify that an orphaned
      `steamapps\common\...` (with no executable) isn't reported as a valid
      installation. *Still open*: now that a real install exists on `D:`,
      `discovery` picks that one — the "only the leftover" case is no longer reproducible on
      this machine and needs covering with a synthetic fixture.
- [x] ~~**Real-data tests that skip silently.**~~ Addressed in `c9faa1e`
      (non-silent skip). The suite is now at **94 tests, 0 failed**.
- [ ] **Tests that pass on an unrepresentative sample.** A more insidious variant of the
      point above, surfaced on 2026-09-03: `unpack`'s "real archive" tests genuinely
      run, but all of them on `config.a` — 24 entries, the only archive the decompressor
      handled at the time. No skip, nothing red, and the module's main function doesn't work.
      To be treated as a rule: **a real-data test must declare which portion of the
      real domain it runs on.**
- [ ] **Two real `ipc` tests skip on this machine** (found on 2026-09-05):
      `crates/ipc/tests/real_saves.rs` and `cross_check.rs` look for
      `20250112.rep+persistentgamedata1.dat` and `20240606.rep_persistentgamedata1.dat`, which
      are no longer in `samples/` (there are `20250626.rep+…` and `live.rep+…` instead). The skip is
      declared and legitimate, but the "242 tests, 0 failed" doesn't count them: either the samples need
      restoring or the tests need rewriting against the ones present.
- [ ] **Rename `samples/live.rep+persistentgamedata1.dat` with its date** (2026-08-31),
      like the other samples: the pinned numbers (379 completed, 169 of 171) are a fixture from a known
      era and a file called "live" invites overwriting it. The tests that reference it by name need
      updating too: `crates/ipc/tests/graph_real.rs` and
      `crates/catalog/tests/real_data.rs`.

---

## Session log

### 2026-09-08 — the plan queue

New pure crate `crates/plan` (26 tests), `store` migration 2, the `ipc` view-model, five
Tauri commands and the TypeScript mirror. Spec
`docs/superpowers/specs/2026-09-07-plan-queue-design.md`, report
`docs/superpowers/plans/2026-09-07-plan-queue-report.md`.

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
`docs/superpowers/plans/2026-09-07-unlock-graph-report.md`; spec in
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
      `design-export`'s generated `LEGGIMI.md` → `README.md` in the generator source (the
      checked-in `design-export/isaacdome-design-pack/` copy is stale until the tool runs
      again on a machine with the game installed — this environment doesn't have it).
- [x] **Every crate** (`app`, `catalog`, `core-save`, `design-export`, `discovery`, `ipc`,
      `store`, `test-support`, `unpack`, `wiki`, `wiki-snapshot`), `ui/`, `reference/`, and
      the dev-tool shell scripts, translated. `cargo fmt`, `cargo clippy --all-targets -D
      warnings`, and `cargo test --workspace -- --nocapture` all clean, one pre-existing
      failure aside (below). `pnpm typecheck`, `lint`, `format:check`, `scan` all clean too.
- [x] **What deliberately stayed in Italian**, and why it isn't a leftover: business-facing
      diagnostic text the app already returns today pending real i18n (e.g. `store_reason`'s
      `"database di una versione più nuova (N > M)"`; `ui/src/App.vue`'s verification-screen
      copy, which carries its own declared exemption in `scan-conventions.mjs`); and test
      *input data* chosen to look like garbage on purpose (an invalid VDF/JSON literal, a
      lone accented character fed through percent-encoding). Translating those would either
      misrepresent what the running app actually shows today, or change what a test is
      exercising. Same treatment applied consistently to the `docs/superpowers/plans/`
      files that mirror this code in illustrative snippets.
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
- [x] **Delirium isn't a symbol, it's the background.** It looked like a gap — eleven layers and
      none called Delirium — until it turned out the mark is the cell's **bloodied
      sheet**. With Delirium out of the symbols, the last pairing (`Cross` → The
      Lamb) closes by elimination. `symbolSource` in `marks.json` says for every row whether
      it's read from the file, referenced, or inferred.
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

Twelve of the fourteen tasks in `docs/MIGLIORIE.md` closed (A1 disappeared along with CI).
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
      `docs/superpowers/plans/2026-09-05-b1-fonti-effetti-report.md`. The original entry compared
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
- [ ] Side finding: **the repository declares no license** (no `LICENSE` file). To
      decide before the first installer, independently of B1.

### 2026-09-05 (night) — wiki dataset

- [x] **Implementation of B1 closed**, TDD plan in 12 tasks, all passed review.
      Full report — `docs/superpowers/plans/2026-09-05-wiki-dataset-report.md`. Born: the
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
- [x] `docs/STATO.md` (this entry and the `wiki` module in "M1 — detail"), `BACKLOG.md`,
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
      `docs/superpowers/plans/2026-09-05-graph-contracts-report.md`.
- [x] **A number in the plan corrected by execution, not adjusted after the fact**: the slots unknown
      to the catalog are **4**, not 5 (`c1d492e`, `814e7cb`): the 5 came from 642 − 637 and counted
      slot 0, which isn't a node. The test was stopped and investigated before touching the
      fixture.
- [x] **The plan degrades instead of failing** (`2724acd`, `591f6c2`, `9db537b`): a `store`
      row with an unreadable target doesn't wipe out the goals but reaches the UI by id;
      a database that won't open arrives as a diagnostic with a reason, and `storeAvailable` is
      derived from a single source.
- [x] `docs/STATO.md`, `DESIGN-BRIEF.md` (new §7 with the contracts, sections renumbered),
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
- [x] TDD plan (B) and report — `docs/superpowers/plans/2026-09-04-catalog-b.md`,
      `docs/superpowers/plans/2026-09-04-catalog-b-report.md`. Full suite: **196 tests,
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
      Report — `docs/superpowers/plans/2026-09-03-catalog-a-report.md`. Final suite:
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
      (`docs/superpowers/plans/2026-09-02-app-shell-ipc.md`). Tasks 1–7 are Rust and need
      no Node: executable right away.
- [x] Spec corrected on five points that came up while writing the plan: `modifiedUnix` instead of an
      already-formatted date, `CandidateSource` not exposing the Steam account id,
      `SectionCount` with no translated label, the `Cell::Unexpected` variant, and criterion 6 made
      precise (a display-only `pathHint` isn't a violation).
- [x] **Node 22 LTS and pnpm via corepack** (E3). Node 22.22.0 and pnpm 10.33.0 were already on
      the machine; on 2026-09-06 the major version was pinned in `package.json` (`engines`) with
      `engine-strict=true` in `.npmrc`, so anyone cloning on a different major finds out right away
      instead of hitting a build error.
