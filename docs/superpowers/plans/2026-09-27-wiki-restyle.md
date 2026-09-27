# Wiki restyle Implementation Plan

> **For agentic workers:** executed as parallel streams by subagents, one worktree per stream,
> managed by the session as PM (the owner's request). Agents never commit; the PM reviews each
> diff, commits, and merges the stream branch into `feature/wiki-restyle`. Steps use checkbox
> (`- [ ]`) syntax for tracking.

**Goal:** every wiki surface shows a readable picture as large as it fits, and everything the
wiki, the game and the save know about the thing.

**Architecture:** `ipc` grows the per-page facts (`PageFacts`, derived from the `Infobox`) and a
`wiki_progress` command carrying the save's state per page, bestiary included. The UI gets one
figure component with the background, centring and integer-scale rules, colour tokens for
edition, quality, category and state, then the landing, the lists (card grid + table, filters,
sorts) and the single pages built on them.

**Tech Stack:** Rust (`ipc`, `core-save`, `app`), ts-rs contract, Vue 3 + Tailwind v4 tokens,
TanStack Virtual, Vitest.

**Spec:** `docs/superpowers/specs/2026-09-27-wiki-restyle-design.md`

## Global Constraints

- `CLAUDE.md` and `docs/frontend-conventions.md` are binding; `pnpm scan` must stay at 0 violations.
- Every visual value is a token in `ui/src/assets/theme/*.css`; no alpha modifiers; one theme; 4 px grid.
- IPC: camelCase, tagged struct variants with `rename_all_fields = "camelCase"`, fieldless = bare
  string, `ui/src/lib/ipc/types.ts` only via `pnpm ipc:types`; a JSON-shape test per new type.
- No `_ =>` on our enums; no `unwrap()` on disk data; functions ≤ ~60 lines; non-test code ≤ ~600
  lines per file; comments say what and why, never card numbers, dates or history.
- Bestiary: tally 1 = met, tally 2 = killed, tally 4 = killed you; tally 3 is never shown or named.
- Without the game: category icon instead of a sprite. Without a save: no profile state, the rest shows.
- Never kill processes by image name (`taskkill /IM`, `Stop-Process -Name`); only a PID you started.
- Full `pnpm check` only once, at the end (PM).

## Review Focus

1. **No save chosen / save section unreadable**: rows and pages render every fact, profile fields
   absent — not "0 of N", not an error. Tested in the progress pure function (Task 2) and the list
   filter (Task 5).
2. **Game not installed**: figures fall back to the category icon at the same size and centring,
   landing tiles too. Tested in the figure scale function (Task 3) and a landing fixture (Task 4).
3. **Base and Tainted forms share a name**: a character's progress must be looked up by id and
   tainted flag, never by name. Tested in Task 2.
4. **Bestiary key collisions**: two pages reaching the same `type.variant.subtype` (Peep Eye / The
   Bloat) — each page gets the tally of its own key, no double counting. Tested in Task 2.
5. **Empty filter results and a table sort on a missing value**: a list whose filter matches
   nothing shows the empty state; sorting by HP puts pages without HP last in both directions.
   Tested in Task 5.

---

## Stream A — contract (`crates/ipc`, `crates/core-save`, `crates/app`) · worktree `isaac-dome-wiki-restyle-ipc`

### Task 1: `PageFacts` and `dlc` on every page reference

**Files:**
- Create: `crates/ipc/src/wiki_facts.rs`
- Modify: `crates/ipc/src/wiki.rs` (`WikiPageRef`, `wiki_index`), `crates/ipc/src/lib.rs`, `crates/ipc/src/contract.rs`
- Test: `crates/ipc/src/wiki_facts.rs` (unit), `crates/ipc/tests/wiki_index.rs` (shape + real data)

**Interfaces:**
- Produces:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum PageFacts {
    Item { quality: Option<i8>, activated: bool, recharge: Option<String>, shop_price: Option<String>, devil_price: Option<String>, tags: Vec<String> },
    Trinket { tags: Vec<String> },
    Achievement { requirement: String, unlocks: Option<Target> },
    Boss { base_hp: Option<u32>, floors: String },
    Challenge { character: Option<Target>, goal: String, blindfolded: bool, curse: String },
    Character { health: String, damage: String, tears: String, range: String, speed: String, luck: String, shot_speed: String, tainted: bool },
    Transformation { requires: Option<u32>, contributors: u32 },
    Entity { base_hp: Option<u32>, floors: String },
    Article { category: Option<wiki::ArticleCategory>, version: Option<VersionFacts> },
}
pub struct VersionFacts { pub number: String, pub date: String } // camelCase
pub fn facts(entry: &wiki::Entry, dataset: &wiki::Dataset) -> PageFacts;
// WikiPageRef gains: pub dlc: Vec<wiki::Dlc>, pub facts: PageFacts
```
Text fields are the inline runs flattened to plain text with the existing plain-text helper
(search `crates/ipc/src/search/text.rs` for it; one definition). `version` comes from the Cargo
`version` table already in the dataset's meta/patches (find where `Meta` keeps patches; match the
article title to the row's `_pageName`).

- [ ] Step 1: failing unit tests in `wiki_facts.rs`: an item fixture entry with quality 3 and `activated` template gives `Item { quality: Some(3), activated: true, .. }`; a character with no declared damage gives the template default already in the entry; an article of category version gets its number and date.
- [ ] Step 2: `cargo test -p ipc wiki_facts` → FAIL (module missing).
- [ ] Step 3: implement `facts` as one exhaustive `match` on `entry.infobox` (no `_`), one helper per variant if over 60 lines.
- [ ] Step 4: add `dlc` and `facts` to `WikiPageRef` in `wiki_index`; JSON-shape test in `tests/wiki_index.rs` pinning `{"kind":"item","quality":3,"activated":true,…}` and camelCase field names; real-data test: The Sad Onion (item 1) has quality 1 and The D6 is activated.
- [ ] Step 5: `cargo test -p ipc`, `cargo clippy -p ipc --all-targets -- -D warnings`, `pnpm ipc:types`, `pnpm typecheck` (fix fixtures in `ui/src/lib/ipc/fixtures/wiki.ts` so they carry `dlc` and `facts`).

### Task 2: `wiki_progress` — the save's state per page, bestiary included

**Files:**
- Create: `crates/ipc/src/wiki_progress.rs`, `crates/ipc/tests/wiki_progress.rs`, `crates/ipc/tests/wiki_progress_real.rs`
- Modify: `crates/ipc/src/lib.rs`, `crates/ipc/src/contract.rs`, `crates/app/src/commands/wiki.rs`, `crates/app/src/lib.rs` (`generate_handler!`), `docs/architecture.md` (command count +1, Wiki route row), `ui/src/lib/ipc/wiki.ts`, `ui/src/lib/ipc/constants/commands.ts`, `ui/src/lib/ipc/fixtures/*`
- Read: `crates/core-save/src/bestiary.rs`, `crates/ipc/src/{collection,challenges,marks}.rs`, `crates/ipc/src/graph/`

**Interfaces:**
- Consumes: the save and catalog the existing `collection` / `challenges` / `marks` / `graph_views` builders read (reuse their builders or the pure functions under them — never re-read the save).
- Produces:
```rust
#[derive(Debug, Clone, PartialEq, Serialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum PageProgress {
    Achievement { done: bool },
    Item { collected: Option<bool>, unlocked: Option<bool>, unlocked_by: Option<u32> },
    Unlockable { unlocked: bool, unlocked_by: u32 },            // trinket, card, rune, character-less unlocks
    Character { unlocked: Option<bool>, marks_done: u32, marks_total: u32 },
    Challenge { state: ChallengeStateView },                     // reuse the existing view type
    Bestiary { met: u32, killed: u32, killed_you: u32 },         // boss and entity
}
#[derive(Serialize, ts_rs::TS)] #[serde(rename_all = "camelCase")]
pub struct PageProgressEntry { pub target: wiki::Target, pub progress: PageProgress }
#[derive(Serialize, ts_rs::TS)] #[serde(rename_all = "camelCase")]
pub struct WikiProgress { pub pages: Vec<PageProgressEntry> }
pub fn wiki_progress(/* the inputs the four views use */) -> WikiProgress;
// Tauri: #[tauri::command] wiki_progress(...) -> Result<Option<WikiProgress>, IpcError>  (None = no save chosen)
// TS: ipc.wikiProgress(): Promise<WikiProgress | null>
```
A page absent from `pages` has no state (transformation, stage, version, pickup, or a section
that could not be read). Bestiary: tally ids 1, 2, 4 from `core-save`'s declared blocks; tally 3
never read into the view. Character: looked up by id and tainted flag.

- [ ] Step 1: failing tests in `tests/wiki_progress.rs` on fixture inputs: an achievement done/not done; an item collected and locked by achievement 5; a trinket unlocked by achievement 7; Isaac vs Tainted Isaac sharing a name get their own marks; a bestiary fixture with tallies 1/2/3/4 on `10.0.0` gives `met, killed, killed_you` from 1, 2, 4 and nothing from 3; two pages on one key each get that key's numbers once; no save → `None`.
- [ ] Step 2: run → FAIL.
- [ ] Step 3: implement the pure function, one helper per kind.
- [ ] Step 4: the Tauri command (wiring only, never caches "no save"), `generate_handler!`, TS wrapper, `docs/architecture.md` redrawn (count + Wiki route row).
- [ ] Step 5: real-data test through `test-support` on the newest dated save: Isaac is unlocked, achievement 1 is done or not as `graph_views` says for the same save (compare the two, don't pin), Clotty `15.0.0` has `killed_you ≥ 3` (the 2026-09-16 measurement).
- [ ] Step 6: `cargo test -p ipc -p core-save`, clippy on `ipc app`, `pnpm ipc:types`, `pnpm typecheck`, `pnpm ui:test`.

## Stream B — UI foundations (`ui/src/assets/theme`, `ui/src/components/wiki/WikiFigure.vue`, `ui/src/components/ui/`) · worktree `isaac-dome-wiki-restyle-ui`

### Task 3: tokens, one figure component, chips and badges

**Files:**
- Modify: `ui/src/assets/theme/colors.css` (edition ×5, quality −1..4, category ×12, each `-surface`/`-foreground`), `ui/src/assets/theme/spacing.css` (`--spacing-figure-row` 4rem, `-card` 8rem, `-tile` 6rem, `-hero` 12rem; retire `--spacing-wiki-row-figure`, `--spacing-wiki-tile` figure use, `--spacing-wiki-hero` once their callers move), `ui/src/components/wiki/WikiFigure.vue`, `ui/src/components/wiki/figureSize.ts`
- Create: `ui/src/components/wiki/figureScale.ts` + `.test.ts`, `ui/src/components/ui/chip/` (a `Chip` primitive with `tone` prop), `ui/src/components/wiki/EditionBadge.vue`, `ui/src/components/wiki/QualityChip.vue`, `ui/src/lib/wiki/tone.ts` + `.test.ts` (edition → tone, quality → tone, category → tone, as `const` objects)
- Modify: `docs/frontend-conventions.md` and `ui/scripts/scan-conventions.mjs` only if a new rule is added (none planned).

**Interfaces:**
- Produces: `WikiFigure` props `{ target, url, size: FigureSize }` where `FigureSize = { Row, Card, Tile, Hero } as const`; `integerScale(nativePx: number, boxPx: number): number` (largest integer ≥1 with `native*scale ≤ box`, `1` when native > box); `Chip` `{ tone: Tone }`; `EditionBadge` `{ dlc: Dlc[] }` (uses `editionAdded`/`editionRemoved`); `QualityChip` `{ quality: number | null }`; `toneOfEdition(d: Dlc): Tone`, `toneOfQuality(q: number): Tone`, `toneOfCategory(c: WikiCategory): Tone`.

- [ ] Step 1: failing Vitest for `integerScale` (32 in 64 → 2, 32 in 70 → 2, 48 in 32 → 1, 16 in 192 → 12) and for the tone maps (every `Dlc`, qualities −1..4, every `WikiCategory` has a tone — exhaustive by iterating `Object.values`).
- [ ] Step 2: run → FAIL.
- [ ] Step 3: implement; `WikiFigure` centres on both axes (`grid place-items-center`), applies `integerScale` to sprites through a CSS variable bound in the template, paints achievement/challenge art on `bg-mark-paper` via `AchievementArt`, and falls back to the category icon at the same box.
- [ ] Step 4: show every size, tone and chip on the Kit page (`#kit`), including a no-game fallback and an achievement on paper; check in the browser (`pnpm ui:dev`, port 1420).
- [ ] Step 5: `pnpm typecheck`, `pnpm ui:test`, `pnpm lint`, `pnpm scan`, `pnpm format:check`.

## Stream C, D, E — screens, after A and B merge into `feature/wiki-restyle` · three worktrees off it

### Task 4 (C): the landing · `ui/src/screens/wiki/WikiLanding.vue` (+ a `landingTile.ts` if logic appears)

- Hero on top: a mosaic of the category samples through `WikiFigure`, the title, totals (pages, snapshot), and with a save the overall progress from `landingProgress` summed over categories.
- Tile: `WikiFigure` at `Tile` centred on `toneOfCategory` surface, name, page count; with a save a progress bar "N of M" from `wiki_progress` (done achievements, collected items, unlocked characters, done challenges, bosses killed at least once, unlocked trinkets/cards/runes); categories without save data show no bar.
- [ ] Vitest for `landingProgress(category, pages, progress | null)` → `{ done, total } | null` (null without save and for categories without state).
- [ ] Browser check with fixtures, with and without a save fixture, without the game.

### Task 5 (D): the lists · `ui/src/screens/wiki/WikiCategoryList.vue`, new `WikiCardGrid.vue`, `WikiTable.vue`, `ui/src/lib/wiki/listFilter.ts` (+ tests), `ui/src/lib/wiki/listColumns.ts` (+ tests)

- A hero on top of every list: the category sample at `Hero` on `toneOfCategory`, name, page count, with a save the "N of M" bar (reuse `landingProgress` from Task 4 — if D starts before C lands, D owns `landingProgress` in `ui/src/lib/wiki/progress.ts` and C imports it).
- Card grid (default) and table, switch remembered per category through the same per-viewer persistence the window session uses (find it in `ui/src/lib/` / `stores/`; if it is the SQLite session, use a key per category; say which).
- Per-category columns and chips from `PageFacts` (spec decision 3 table) + `EditionBadge` + profile state.
- Filters: title, edition, profile state, and kind filters (item quality and activated, tags, character tainted, article category). Sorts: name, id, edition, every numeric fact; missing values last in both directions.
- Both views virtualized with TanStack Virtual (the existing `VirtualRows` or its grid form).
- [ ] Vitest first for `filterPages` and `sortPages` with the Review Focus cases 1 and 5.
- [ ] Browser check on every category.

### Task 6 (E): the single pages · `ui/src/screens/wiki/WikiHero.vue`, `WikiInfobox.vue`, `infobox/*.vue`, `heroSummary.ts`

- Hero: `WikiFigure` at `Hero`, `EditionBadge`, the page's `PageFacts` as chips, and a profile block from `wiki_progress` (per `PageProgress` variant, bestiary as three numbers with their labels).
- Infobox rows keep their data and use `Chip`/tones for quality, edition, tags.
- [ ] Vitest for the profile block's view function (`progressLines(progress) → lines`).
- [ ] Browser check on one page of each kind.

## Finish (PM)

- [ ] Merge C, D, E into `feature/wiki-restyle`; `pnpm check` once; `docs/architecture.md` counts `wiki_progress`.
- [ ] Merge into `develop`, push, card #90 to UAT with `NEEDS WINDOW`, `NEEDS GAME`, `NEEDS SAVE`.
