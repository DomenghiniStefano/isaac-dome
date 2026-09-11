# Progress status — IsaacDome

Single source of truth for the project status. Updated every session.
Narrative summary and design decisions live elsewhere: `docs/PROJECT.md` (project),
`docs/superpowers/specs/` (module design), `docs/superpowers/plans/` (plans and reports).
The quality tasks that came out of the 2026-09-05 review (local verification, scanner, IPC
contract, memory, test data) live in `docs/IMPROVEMENTS.md`, with closing criteria and order.

**Integration branch:** `develop`. `master` is stopped at the initial commit.
**Wiki dataset merged** into `develop` on 2026-09-06 (`feature/wiki-dataset`, 29 commits,
suite green on the merge result, review of the whole branch closed). The local branch was
deleted; on origin its last published version remains.
**Design system merged** into `develop` on 2026-09-11: cycle 1 (`feature/design-system-foundations`),
then cycle 2 and screens 3.1–3.2 in one merge of `feature/design-system-screens`, which
already held `feature/design-system-components`; suite green on the merge result. The screens
branch carries on with 3.3.
**Last update:** 2026-09-11

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
            `docs/superpowers/plans/2026-09-10-design-system-foundations.md`
      - [x] **2. App components** (2026-09-10) — title bar and tabs, navbar, section
            sidebar, KPI tile, matrix cell in sprites or bars, wiki tokens and blocks, data
            states, collapsible card; presentational, on the Kit page. Spec
            `docs/superpowers/specs/2026-09-10-design-system-components-design.md`, plan
            `docs/superpowers/plans/2026-09-10-design-system-components.md`
      - [ ] **3. Screens** — decomposed into seven sub-projects in
            `docs/superpowers/specs/2026-09-11-screens-shell-profile-design.md`, each with
            its own spec → plan → execution; decisions marked "(delegated)" wait for the
            first-launch review:
            - [x] 3.1 Shell and profile selection (2026-09-11) — tabs owning locations, Vue Router in
                  memory, Pinia, the live window chrome, the profile indicator, the profile
                  screen and its gate, placeholders, fixtures for `pnpm ui:dev`. Plan
                  `docs/superpowers/plans/2026-09-11-screens-shell-profile.md` (its
                  checkboxes are the step-by-step state)
            - [x] 3.2 Completion (2026-09-11) — the marks matrix on the active profile: four
                  KPIs with their denominators, base and Tainted groups, the unknown block,
                  a tooltip per cell, sprites or bars; B13 closed (the marks map in `ipc`, the
                  icon protocol serving crops). Spec
                  `docs/superpowers/specs/2026-09-11-screens-completion-design.md`, plan
                  `docs/superpowers/plans/2026-09-11-screens-completion.md`
            - [x] 3.3 Next steps, Unlock, Plan — split in two halves
                  (`docs/superpowers/specs/2026-09-11-screens-graph-design.md`):
                  - [x] 3.3a the node, Next steps and Unlock (2026-09-11) — a node's state and
                        its why, the Badge's partial state, four facets with data and their
                        counts, search, three sorts, a virtualized table; plan
                        `docs/superpowers/plans/2026-09-11-screens-graph.md`
                  - [x] 3.3b Plan (2026-09-11) — the queue you drag, a repair that says where a
                        row stopped, the diagnostics as alerts and footnotes, the proposal
                        beside it; "in coda" and one click to add on Next steps and Unlock; a
                        move names the row it lands under (`queue_move(achievement, after)`).
                        Spec `docs/superpowers/specs/2026-09-11-screens-plan-design.md`, plan
                        `docs/superpowers/plans/2026-09-11-screens-plan.md`
            - [ ] 3.4 Collection — items by pool and quality
            - [ ] 3.5 Wiki in tabs, search — the `Ctrl+K` palette (B5)
            - [ ] 3.6 Settings and About — provenance, credits, the three promises
            - [ ] 3.7 Tabs that survive a restart (B6)

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

- [ ] **Section 8's index 2** — one solo run with a known ending, watching index 2.
      Twenty minutes. Index 19 is already the identity mapping for cutscene 19; index 2 is
      either cutscene 1 under an off-by-one or a count of launches, and one run separates
      them. Closing it lets sections 5, 8 and 9 be renamed the way 3 and 6 were. *Careful
      with co-op*: five of fifteen co-op windows showed a cutscene in the log and no
      movement at all, because co-op progression goes to the shared profile.
- [ ] **What the bestiary's four tallies count** (B9, structure closed 2026-09-09). Section
      10 holds four lists over the same entities — ids 4, 2, 3, 1 — with a different number
      against each entity in each. A matched window says which is which: kill a known enemy
      a known number of times and see which tally moves by how much. Until then they carry
      the id the file gives them and no name.
- [ ] **What the bestiary's trailing word is.** One word after the last tally, in every
      save, growing 11,343 → 29,725 across the samples we hold. Same instrument: one run,
      see what it moves by. `Save::bestiary_tallies()` already hands it back as a value, so
      this is only a matter of watching it.
- [ ] **The 40 unknown cells in the completion matrix** — Mother and The Beast for The
      Forgotten and the 19. One run of Mother with a Tainted character closes the whole
      20 × 2 block, because the base indices are already pinned and only the evidence that
      those cells move is missing.
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
      `docs/superpowers/plans/2026-09-05-b1-sources-effects-report.md`. **Implementation closed
      the same day** (at night): crate `wiki`, tool `wiki-snapshot`, embedded dataset —
      report in `docs/superpowers/plans/2026-09-05-wiki-dataset-report.md`. All that's left is the
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

- [x] **B8 closed** — report in `docs/superpowers/plans/2026-09-08-b8-log-spike-report.md`,
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
      `docs/superpowers/plans/2026-09-05-b1-sources-effects-report.md`. The original entry compared
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
      `docs/superpowers/plans/2026-09-05-graph-contracts-report.md`.
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
