# Completed — the modules of M1, as they were built

The per-module record: design spec, TDD plan, implementation, report, post-review fixes, one
section per crate. **Searched, never read.** It left `docs/STATUS.md` on 2026-09-16, where it was
532 lines of a file whose job is to say where the project is — and every module in it is closed.

What is still open about any of them is in `docs/BACKLOG.md`; what the project knows about the
game and its files is in `CLAUDE.md` and the documents it points at.

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

