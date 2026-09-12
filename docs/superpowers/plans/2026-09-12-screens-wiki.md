# Cycle 3.5a — the Wiki in tabs: Implementation Plan

> **For agentic workers:** executed inline with superpowers:executing-plans, on the owner's
> delegation ("prosegui coi lavori"). Steps use checkbox (`- [ ]`) syntax. Every commit is
> pushed as it lands (the owner's rule: the remote stays aligned). Branch
> `feature/screens-wiki-search`, half A; half B (search) has its own plan.

**Goal:** the Wiki route stops being a placeholder: a tab is a category list or one page —
figure, infobox, sections, every reference a link — from the embedded dataset, with or
without the game, with or without a profile.

**Architecture:** one new command, `wiki_index`, hands the frontend every page's identity,
title and icon link once per window; `IconRef::Page` lets the icon protocol serve any page's
figure through `target_sprite`. A page is a tab location (`?category=…&page=item:105`), its
label derived from the index. The screen picks landing, list or page from the query; the
page reads `wikiEntry` and draws the infobox per kind and the sections with cycle 2's
`WikiBlocks`.

**Tech Stack:** Rust (`crates/ipc`, `crates/app`, `crates/design-export`), Vue 3.5,
TypeScript, Pinia, Vue Router, Tailwind v4, Reka UI, vue-i18n, `@tanstack/vue-virtual`,
Vitest.

**Spec:** `docs/superpowers/specs/2026-09-12-screens-wiki-search-design.md` (Decisions 1–4,
9, 10)

## Global Constraints

- Only resolved view-models cross the IPC: a page is a wiki `Target`; icons are `isaac://` links.
- Degrade, never fail: no game → names from titles, no icons; no dataset → the landing's alert; unknown page → a state, not a crash.
- A tab saves identity, never content: the page key in the location, the label derived from the index, falling back to the category.
- IPC: `#[serde(rename_all = "camelCase")]`; tagged enums with `rename_all_fields`; exhaustive matches, no `_ =>` on closed enums; no `unwrap`/`panic` outside tests.
- Frontend: the five rules (no `<style>`, no hardcoded visual constants, no `invoke()` in components, no raw `<button>`/`<input>`, no string unions), `assertNever`, every visible string through `useMessages()`; page titles are data, in English.
- Fixtures only under `import.meta.env.DEV`; pack files through `import.meta.glob`.
- Checks judged by exit code; Cargo with `CARGO_BUILD_JOBS=4` on this machine.
- Commits: Conventional Commits, English, no `Co-Authored-By` or Claude reference, **pushed after each task**.

---

### Task 1: `IconRef::Page` — the icon protocol serves any page's figure

**Files:**
- Modify: `crates/ipc/src/icon.rs`
- Test: `crates/ipc/tests/icon.rs`

**Interfaces:**
- Produces: `IconRef::Page { target: Target }` with `to_path` → `page/item/105`, `page/trinket/97`, `page/achievement/1`, `page/challenge/19`, `page/character/0`, `page/entity/20/0/0`; `parse` the inverse; `icon_source(c, &IconRef::Page{..})` → `target_sprite(c, target)`'s `Found` sprite or `None`.

- [ ] **Step 1: Failing tests** — append to `crates/ipc/tests/icon.rs`:

```rust
#[test]
fn a_page_reference_survives_the_round_trip_for_every_kind_that_has_a_page() {
    use ipc::Target;
    let pages = [
        Target::Item { id: 105 },
        Target::Trinket { id: 97 },
        Target::Achievement { id: 1 },
        Target::Challenge { number: 19 },
        Target::Character { id: 0 },
        Target::Entity { id: 20, variant: 0, subtype: 0 },
    ];
    for target in pages {
        let r = IconRef::Page { target: target.clone() };
        let path = r.to_path();
        assert!(path.starts_with("page/"), "{path}");
        assert_eq!(IconRef::parse(&path), Some(r), "round trip of {path:?}");
    }
    assert_eq!(
        IconRef::Page { target: Target::Entity { id: 20, variant: 0, subtype: 0 } }.to_path(),
        "page/entity/20/0/0"
    );
}

#[test]
fn a_target_with_no_page_has_no_path() {
    // Stages, rooms, pickups and transformations have no page in the dataset, so no figure
    // to serve: the handler refuses the string instead of guessing.
    for bad in ["page/stage/Basement", "page/room/x", "page/pickup/Chest", "page/transformation/1", "page/item", "page/entity/20/0", "page/item/1/2"] {
        assert_eq!(IconRef::parse(bad), None, "{bad:?}");
    }
}

#[test]
fn a_page_icon_resolves_through_target_sprite() {
    use ipc::Target;
    let c = catalog();
    let found = icon_source(&c, &IconRef::Page { target: Target::Item { id: 2 } });
    assert_eq!(found.map(|s| s.path.as_str()), Some("gfx/items/collectibles/a.png"));
    assert!(icon_source(&c, &IconRef::Page { target: Target::Item { id: 99 } }).is_none());
    // A challenge with no rewarding achievement has no art, and says nothing.
    assert!(icon_source(&c, &IconRef::Page { target: Target::Challenge { number: 1 } }).is_none());
}
```

- [ ] **Step 2: Run** `cargo test -p ipc --test icon` — expected: compile error, no `Page` variant.

- [ ] **Step 3: Implement** in `crates/ipc/src/icon.rs`: add the variant with a doc comment; in `to_path`, `IconRef::Page { target } => format!("page/{}", page_path(target))` where `page_path` is a private fn exhaustive over `Target` returning `Option<String>` (`None` for Stage/Room/Pickup/Transformation — `to_path` for those returns `"page/none"`, a string `parse` refuses, and a debug assertion documents it never happens: the index only ever builds `Page` for targets that have a page); in `parse`, `("page", kind, Some(rest))` collects the remaining segments and matches `(kind, segments)` → `item`/`trinket`/`achievement`/`character` with one id, `challenge` with one number, `entity` with three; anything else `None`. In `icon_source`: `IconRef::Page { target } => match target_sprite(c, target) { TargetSprite::Found(s) => Some(s), TargetSprite::NoArt | TargetSprite::Unknown => None }`.

- [ ] **Step 4: Run** `cargo test -p ipc --test icon` — expected: all pass; `cargo clippy -p ipc --all-targets -- -D warnings` clean.

- [ ] **Step 5: Commit** — `feat(ipc): a page icon reference, served through target_sprite`

---

### Task 2: `ipc::wiki_index`

**Files:**
- Modify: `crates/ipc/src/wiki.rs`, `crates/ipc/src/lib.rs`
- Test: `crates/ipc/tests/wiki_index.rs`

**Interfaces:**
- Produces: `pub fn wiki_index(dataset: Result<&Dataset, &DatasetError>, catalog: Option<&Catalog>, game_updated_unix: Option<u64>, icon: impl FnMut(&IconRef) -> Option<String>) -> WikiIndex`; `WikiIndex { info: WikiInfo, pages: Vec<WikiPageRef> }`; `WikiPageRef { target: Target, title: String, icon_url: Option<String> }`.

- [ ] **Step 1: Failing tests** — `crates/ipc/tests/wiki_index.rs`:

```rust
//! The wiki index: every page the dataset has, once per window. What the tab labels, the
//! category lists and the icon of every reference inside a page are read from.

use catalog::Catalog;
use ipc::{wiki_index, IconRef, Target, WikiIndex};
use serde_json::{json, to_value};
use wiki::{Dataset, Entry, Infobox};

const ITEMS: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"2\" gfx=\"a.png\" name=\"A\" /></items>";

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS.to_vec()),
        _ => None,
    })
}

fn entry(title: &str, infobox: Infobox) -> Entry {
    Entry { title: title.into(), revid: 1, infobox, sections: vec![] }
}

fn dataset() -> Dataset {
    let mut ds = Dataset::empty_for_tests();
    ds.items.insert(2, entry("A", Infobox::Item));
    ds.items.insert(9, entry("Nine", Infobox::Item));
    ds.trinkets.insert(1, entry("T", Infobox::Trinket));
    ds.bosses.insert(Dataset::boss_key(20, 0, 0), entry("Monstro", Infobox::Boss { base_hp: None, environment: vec![], pool: vec![], unlocked_by: None }));
    ds.meta.counts.items = 2;
    ds.meta.counts.trinkets = 1;
    ds.meta.counts.bosses = 1;
    ds
}

fn link(r: &IconRef) -> Option<String> {
    Some(format!("isaac://{}", r.to_path()))
}

#[test]
fn the_shape_is_pinned_and_icons_are_null_without_a_catalog() {
    let ds = dataset();
    let index = wiki_index(Ok(&ds), None, None, link);
    let v = to_value(&index).unwrap();
    assert_eq!(v["info"]["kind"], "loaded");
    assert_eq!(v["pages"][0], json!({ "target": { "kind": "item", "id": 2 }, "title": "A", "iconUrl": null }));
    assert_eq!(v["pages"].as_array().unwrap().len(), 4);
}

#[test]
fn pages_come_out_by_kind_then_by_id_and_carry_the_catalog_s_icon() {
    let ds = dataset();
    let c = catalog();
    let index: WikiIndex = wiki_index(Ok(&ds), Some(&c), None, link);
    let targets: Vec<&Target> = index.pages.iter().map(|p| &p.target).collect();
    assert_eq!(targets, vec![
        &Target::Item { id: 2 }, &Target::Item { id: 9 }, &Target::Trinket { id: 1 },
        &Target::Entity { id: 20, variant: 0, subtype: 0 },
    ]);
    // The catalog knows item 2 and nothing else: one link, three placeholders.
    assert_eq!(index.pages[0].icon_url.as_deref(), Some("isaac://page/item/2"));
    assert!(index.pages[1..].iter().all(|p| p.icon_url.is_none()));
}

#[test]
fn a_missing_dataset_is_an_empty_index_that_says_why() {
    let err = wiki::DatasetError::Malformed { reason: "x".into() };
    let index = wiki_index(Err(&err), None, None, link);
    assert!(index.pages.is_empty());
    assert_eq!(to_value(&index.info).unwrap()["kind"], "missing");
}

#[test]
fn the_embedded_index_counts_match_its_meta_and_stay_small() {
    let ds = Dataset::embedded().expect("embedded dataset");
    let index = wiki_index(Ok(ds), None, None, link);
    let counts = &ds.meta.counts;
    let expected = counts.items + counts.trinkets + counts.achievements + counts.bosses + counts.challenges + counts.characters;
    assert_eq!(index.pages.len() as u32, expected);
    let json = serde_json::to_string(&index).unwrap();
    // A generous ceiling: the index is one load per window and must stay one order of
    // magnitude under `unlock`'s (crates/ipc/tests/unlock_size.rs).
    assert!(json.len() < 256_000, "wiki index is {} bytes", json.len());
}
```

- [ ] **Step 2: Run** `cargo test -p ipc --test wiki_index` — expected: compile error.

- [ ] **Step 3: Implement** in `crates/ipc/src/wiki.rs`:

```rust
/// One page of the dataset: its identity, its own title, and the link to its figure when
/// the catalog draws one.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiPageRef {
    pub target: Target,
    pub title: String,
    pub icon_url: Option<String>,
}

/// Every page the dataset has, once per window (spec 3.5, Decision 2).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WikiIndex {
    pub info: WikiInfo,
    pub pages: Vec<WikiPageRef>,
}

pub fn wiki_index(
    dataset: Result<&Dataset, &DatasetError>,
    catalog: Option<&Catalog>,
    game_updated_unix: Option<u64>,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> WikiIndex {
    let info = wiki_info(dataset, game_updated_unix);
    let Ok(ds) = dataset else {
        return WikiIndex { info, pages: Vec::new() };
    };
    let mut page = |target: Target, entry: &Entry| WikiPageRef {
        icon_url: catalog.and_then(|c| match target_sprite(c, &target) {
            TargetSprite::Found(_) => icon(&IconRef::Page { target: target.clone() }),
            TargetSprite::NoArt | TargetSprite::Unknown => None,
        }),
        title: entry.title.clone(),
        target,
    };
    let mut pages = Vec::new();
    pages.extend(ds.items.iter().map(|(id, e)| page(Target::Item { id: *id }, e)));
    pages.extend(ds.trinkets.iter().map(|(id, e)| page(Target::Trinket { id: *id }, e)));
    pages.extend(ds.achievements.iter().map(|(id, e)| page(Target::Achievement { id: *id }, e)));
    pages.extend(ds.bosses.iter().filter_map(|(key, e)| Some(page(boss_target(key)?, e))));
    pages.extend(ds.challenges.iter().map(|(n, e)| page(Target::Challenge { number: *n }, e)));
    pages.extend(ds.characters.iter().map(|(id, e)| page(Target::Character { id: *id }, e)));
    WikiIndex { info, pages }
}

/// The inverse of `Dataset::boss_key`: `"20.0.0"` → the entity. A key that isn't three
/// numbers is a dataset the build never wrote, and the page is left out rather than guessed.
fn boss_target(key: &str) -> Option<Target> { … }
```

Bosses' `BTreeMap<String, Entry>` sorts as strings ("20.0.0" before "3.0.0"): the test above has one boss and doesn't pin string-vs-numeric order; the list sorts by title on screen anyway (Task 7). Note it in a comment.

- [ ] **Step 4: Run** `cargo test -p ipc --test wiki_index` — pass; clippy clean.

- [ ] **Step 5: Commit** — `feat(ipc): the wiki index, every page's identity, title and icon link`

---

### Task 3: the `wiki_index` command and the pack payload

**Files:**
- Modify: `crates/app/src/lib.rs`, `crates/design-export/src/payload.rs`

- [ ] **Step 1:** In `crates/app/src/lib.rs`, beside `wiki_entry`:

```rust
#[tauri::command]
fn wiki_index(
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<ipc::WikiIndex, IpcError> {
    let d = discover(&Options::default());
    let game_updated_unix = d.game.as_ref().and_then(|g| g.updated_unix);
    // No game is expected: the index goes out with no icon links, and the screen says so.
    let catalog = resources.get().and_then(|rs| state.get_or_build(rs));
    Ok(ipc::wiki_index(wiki::Dataset::embedded(), catalog, game_updated_unix, icon_url))
}
```

Register it in `generate_handler!`. `IconRef::Page` needs no handler change: `icon_bytes` already routes every catalog-backed reference through `icon_source` — add `IconRef::Page { .. }` to that match arm (the match is exhaustive, so the build says where).

- [ ] **Step 2:** In `crates/design-export/src/payload.rs` `export`, after `extraction_report`: `write_json(root, "contracts/payload/wiki_index.json", &ipc::wiki_index(wiki::Dataset::embedded(), Some(catalog), None, link))` following the file's existing pattern for the link closure and the manifest line.

- [ ] **Step 3: Run** `cargo build -p app` and `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test -p design-export`.

- [ ] **Step 4: Commit** — `feat(app): the wiki_index command, and its payload in the design pack`

---

### Task 4: the page key, the category and the tab label (TypeScript, pure)

**Files:**
- Create: `ui/src/lib/wiki/pageKey.ts`, `ui/src/lib/wiki/pageKey.test.ts`, `ui/src/lib/wiki/category.ts`, `ui/src/lib/wiki/category.test.ts`
- Modify: `ui/src/router/routeTable.ts` (`page` in the query), `ui/src/stores/tabModel.ts` (`tabLabel`), `ui/src/stores/tabModel.test.ts`, `ui/src/lib/ipc/types.ts`, `ui/src/lib/ipc/wiki.ts`, `ui/src/lib/constants/commands.ts`

**Interfaces:**
- Produces: `pageKey(target: Target): string | null` (null for kinds with no page); `parsePageKey(key: string): Target | null`; `categoryOf(target: Target): WikiCategory | null`; `pageLocation(target): TabLocation | null` (`{ name: Wiki, query: { category, page } }`); `tabLabel(location, titleOf: (key: string) => string | null): MessageKey | string` — a page's title when known, else `locationTitle(location)`. Types `WikiIndex`, `WikiPageRef`; `wikiIndex(): Promise<WikiIndex>`; `Command.WikiIndex = 'wiki_index'`.

- [ ] **Step 1: Failing tests** — `pageKey.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import type { Target } from '@/lib/ipc/types'
import { pageKey, parsePageKey } from './pageKey'

const pages: [Target, string][] = [
  [{ kind: 'item', id: 105 }, 'item:105'],
  [{ kind: 'trinket', id: 97 }, 'trinket:97'],
  [{ kind: 'achievement', id: 1 }, 'achievement:1'],
  [{ kind: 'challenge', number: 19 }, 'challenge:19'],
  [{ kind: 'character', id: 0 }, 'character:0'],
  [{ kind: 'entity', id: 20, variant: 0, subtype: 0 }, 'entity:20.0.0'],
]

describe('pageKey', () => {
  it.each(pages)('writes %o as %s and reads it back', (target, key) => {
    expect(pageKey(target)).toBe(key)
    expect(parsePageKey(key)).toEqual(target)
  })
  it('has no key for a target with no page', () => {
    expect(pageKey({ kind: 'stage', name: 'Basement' })).toBeNull()
    expect(pageKey({ kind: 'transformation', id: 1 })).toBeNull()
  })
  it('refuses what it never wrote', () => {
    for (const bad of ['', 'item', 'item:', 'item:x', 'stage:Basement', 'entity:20.0', 'entity:20.0.0.0', 'item:1:2', 'ITEM:1']) {
      expect(parsePageKey(bad)).toBeNull()
    }
  })
})
```

`category.test.ts`: `categoryOf` maps the six kinds to `WikiCategory` and the four others to `null`; `pageLocation({kind:'item',id:105})` equals `{ name: 'wiki', query: { category: 'items', page: 'item:105' } }` and is `null` for a stage.

`tabModel.test.ts` gains: `tabLabel` on a page location returns the title when `titleOf` knows the key, `locationTitle`'s message when it doesn't, and the route's message on any non-page location.

- [ ] **Step 2: Run** `pnpm --filter ui exec vitest run src/lib/wiki src/stores` — expected: fail (modules missing).

- [ ] **Step 3: Implement.** `pageKey.ts`: a `switch` on `target.kind` with `assertNever`; `parsePageKey` splits on the first `:`, checks the kind against `PageKind = { Item: 'item', … } as const`, parses decimal integers strictly (`/^\d+$/`), `entity` as three dot-separated. `category.ts`: `categoryOf` and `pageLocation` (imports `RouteName`, `WikiCategory`). `routeTable.ts`: `query?: { category?: WikiCategory; page?: string }`. `tabModel.ts`:

```ts
export const tabLabel = (
  location: TabLocation,
  titleOf: (key: string) => string | null,
): Message | { text: string } => {
  const key = location.query?.page
  const title = key === undefined ? null : titleOf(key)
  return title === null ? locationTitle(location) : { text: title }
}
```

(A page title is data, not a message: the return says which, and `App.vue` translates one and shows the other.) `types.ts`: `WikiPageRef`, `WikiIndex` after `WikiInfo`; `wiki.ts`: `wikiIndex`; `commands.ts`: `WikiIndex: 'wiki_index'`.

- [ ] **Step 4: Run** the same tests — pass; `pnpm typecheck`.

- [ ] **Step 5: Commit** — `feat(ui): a wiki page is a tab location, its label read from the index`

---

### Task 5: the fixtures — the index from the pack, the eleven pages

**Files:**
- Create: `ui/src/lib/ipc/fixtures/wiki.ts`, `ui/src/lib/ipc/fixtures/wiki.test.ts`
- Modify: `ui/src/lib/ipc/fixtures/index.ts`

**Interfaces:**
- Produces: `wikiIndexAnswer({ withArt, withCatalog, withWiki }): WikiIndex`; `wikiEntryAnswer(target): Entry | null`; `?wiki=none` → `info: { kind: 'missing', reason: 'malformed' }`, no pages.

- [ ] **Step 1: Failing tests** — `wiki.test.ts`: the index lists the pack's item 105 with title "The D6" and the pack's boss `entity 20.0.0` "Monstro"; without the catalog every `iconUrl` is null; `wikiEntryAnswer({kind:'item',id:105})` has title "The D6" and `wikiEntryAnswer({kind:'item',id:1})` is null; `withWiki: false` gives a missing info and no pages.

- [ ] **Step 2: Run** — fail.

- [ ] **Step 3: Implement.** Globs: `images/INDEX.json` (families `item` → `item`/`trinket` by `kind`, `achievement`, `boss` → the entity key parsed from `source` `Portrait_<type>.<variant>_…`, `character`), `wiki/samples/*.json` (the `Entry` of each, keyed by the file stem → `pageKey`: `item_105` → `item:105`, `entity_20_0_0` → `entity:20.0.0`), `contracts/payload/unlock.json` for challenge targets (`unlocks` of kind `challenge`: id and name). Titles: the pack's `name`; a sample page's own title wins. `iconUrl`: `packIconUrl`-style rewrite to the pack's single files for items and achievements, `images/boss/<id>` and character portraits for the rest, null when `withArt` is false. Counts for `info` from the pages built. `wikiEntryAnswer` reads the samples map and warns once (`console.warn`) that the pack carries eleven pages. Wire `Command.WikiIndex` and `Command.WikiEntry` in `index.ts` with a `?wiki=none` param (`const WikiParam = 'wiki'`).

- [ ] **Step 4: Run** tests — pass; `pnpm typecheck`; `pnpm lint`.

- [ ] **Step 5: Commit** — `feat(ui): the wiki fixture, the pack's pages and image index as the dataset`

---

### Task 6: the wiki store, and references that open in a tab

**Files:**
- Create: `ui/src/stores/wiki.ts`, `ui/src/components/wiki/WikiFigure.vue`
- Modify: `ui/src/lib/constants/stores.ts` (`Wiki: 'wiki'`), `ui/src/components/wiki/WikiInline.vue`, `ui/src/components/wiki/WikiBlocks.vue`, `ui/src/kit/sections/app/WikiSection.vue`

**Interfaces:**
- Produces: `useWikiStore()` with `index: WikiIndex | null`, `status`, `error`, `loadIndex()` (once; a second call while ready is a no-op), `titleOf(key: string): string | null`, `iconFor(target): string | null`, `hasPage(target): boolean`, `entry(key): Entry | null | undefined` (undefined: not read yet), `loadEntry(target)`. `WikiInline` / `WikiBlocks` emit `navigate: [target: Target, newTab: boolean]` and accept `canOpen?: (target: Target) => boolean`. `WikiFigure` props `{ target: Target; url: string | null }`.

- [ ] **Step 1: Store.** `stores/wiki.ts`: `index`, `status`, `error`, a `Map<string, Entry | null>` of read pages keyed by page key; `titleOf` and `iconFor` through a `Map<string, WikiPageRef>` built once from the index (`computed`); `loadIndex` guards on `status === Ready || Loading`; `loadEntry(target)` reads `wikiEntry` once per key and stores `null` too (a page the dataset lacks is an answer). `wikiUnavailable` sets `Failed` with the error.

- [ ] **Step 2: References.** In `WikiInline.vue`: the `Button` click emits `('navigate', token.target, $event.ctrlKey)`; the ref renders as a `Button` only when `canOpen?.(token.target) ?? true`, otherwise with the concept's dotted span (a `span` with the same classes as `concept`, carrying the icon when there is one). The recursive `WikiInline` for `edition` forwards `canOpen` and re-emits both arguments. `WikiBlocks.vue`: `forward` gains `canOpen` and `onNavigate: (target, newTab) => emit('navigate', target, newTab)`. The Kit's `WikiSection.vue` passes a `canOpen` that says items open and pickups don't, so both natures show.

- [ ] **Step 3: `WikiFigure.vue`.** By `target.kind`: `item`/`trinket` → `PixelSprite` in a `size-wiki-figure` square with `placeholder`; `achievement` → `AchievementArt` with `ArtSize.Card`; `entity`/`character` → an `img` in a `size-wiki-figure` square frame (`bg-data`, the hatch placeholder on null or error — reuse `PixelSprite` without `pixelated`? No: portraits are pixel art too; use `PixelSprite`); the other four kinds → the placeholder. Add `--spacing-wiki-figure: 6rem` to `theme/spacing.css` with a comment (a 32 px sprite at 3×, a 192 px portrait at half).

- [ ] **Step 4: Run** `pnpm typecheck`, `pnpm ui:test`, `pnpm scan`.

- [ ] **Step 5: Commit** — `feat(ui): the wiki store, and references that open in place or in a new tab`

---

### Task 7: the Wiki screen — landing, list, page

**Files:**
- Create: `ui/src/screens/WikiScreen.vue`, `ui/src/screens/wiki/WikiLanding.vue`, `ui/src/screens/wiki/WikiCategoryList.vue`, `ui/src/screens/wiki/WikiPage.vue`, `ui/src/screens/wiki/WikiInfobox.vue`, `ui/src/screens/wiki/WikiSections.vue`, `ui/src/screens/wiki/wikiLabels.ts`, `ui/src/lib/wiki/listFilter.ts`, `ui/src/lib/wiki/listFilter.test.ts`
- Modify: `ui/src/router/routes.ts` (`Wiki` real), `ui/src/router/routeTable.ts` (drop `routeArrives[Wiki]`), `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts`, `ui/src/App.vue` (tab labels through `tabLabel`)

**Interfaces:**
- Consumes: the wiki store, `pageLocation`, `parsePageKey`, `categoryOf`, `tabLabel`.
- Produces: `filterPages(pages: WikiPageRef[], category: WikiCategory, query: string): WikiPageRef[]` — the category's pages, by title, case-insensitive contains, sorted by title.

- [ ] **Step 1: Failing test** — `listFilter.test.ts`: three pages (two items, one boss); `filterPages(pages, 'items', '')` gives the two items sorted by title; `'d6'` gives The D6 only; `'bosses'` with `''` gives Monstro.

- [ ] **Step 2: Run** — fail; implement `listFilter.ts` (uses `categoryOf`); pass.

- [ ] **Step 3: i18n.** `it.ts` gains `wiki: { intro, provenance: { snapshot, patch, patchUnknown, newerGame, license, licenseLong }, categoryCount, listHint, noCatalog, search, noResults, resetFilters, back, kind: { item, trinket, achievement, boss, challenge, character }, revision, section: { effects, notes, synergies, interactions, bugs, behavior, championVersions, damageScaling, strategies, difficulty, reward, unlockable }, infobox: { description, requirements, unlocks, unlockedBy, baseHp, environment, pool, goal, items, trinkets, pickups, health, curse, blindfolded, shops, treasureRooms, damage, range, speed, luck, shotSpeed, collectibles, none }, states: { loading, unknown, unknownHint, noSections, missing } }`; `en.ts` mirrored. The twelve section names from the export's `SEC_LABEL`; `placeholder.wiki` removed from both.

- [ ] **Step 4: `WikiScreen.vue`.** `useRoute()`; `wiki.loadIndex()` on setup; `computed` `page = route.query.page` (string or undefined), `category`; renders `WikiPage` when `page`, `WikiCategoryList` when `category`, else `WikiLanding`. A `ProfileError`-like alert with retry when `wiki.status === Failed`.

- [ ] **Step 5: `WikiLanding.vue`.** `ScreenHeader` (book icon, `routes.wiki`, `wiki.intro`); the provenance as a `Card` of `ProfileFact`-style rows (snapshot date through `formatModified`-like `Intl.DateTimeFormat`, patch or "unknown", the license line with a `Tooltip` carrying `licenseLong`, the newer-game note when `gameNewerThanSnapshot === true`); a grid of six `Card`s (`grid-cols-3`), each a `Button` variant that navigates to `{ name: Wiki, query: { category } }` (Ctrl → `tabs.open`), with the category's icon, title and count. `info.kind === 'missing'` → an `Alert` destructive with `wiki.states.missing`.

- [ ] **Step 6: `WikiCategoryList.vue`.** Props `category`. `ScreenHeader` with `wikiCategoryIcon`, `wikiCategoryTitle`, eyebrow `N pagine`; the `noCatalog` line when every `iconUrl` is null and `info` is loaded; a `Card` with a toolbar (`Input` bound to a local `query`, the shown/total count) and a virtualized list (the `CollectionTable` pattern: `useVirtualizer`, `--unlock-total`, `--row-start`, `h-row-wide`, `max-h-unlock-body`) whose row is a `Button` variant `Row`? — no such variant: the row is a `Button` with `ButtonVariant.Ghost` full-width? Check `button/variants.ts` at execution and use the variant that a table row uses on Unlock (`UnlockRow`'s add button pattern); the row shows `PixelSprite`/`AchievementArt` by kind at `size-8`, the title, and `id N` in `text-micro`. Click → `tabs.navigate(pageLocation(target))`, Ctrl+click → `tabs.open`. Empty → `EmptyCategory` + reset.

- [ ] **Step 7: `WikiPage.vue`.** Props `pageKey: string`. `target = parsePageKey(pageKey)`; `watch(target, immediate)` → `wiki.loadEntry`. Header: `WikiFigure`, title (`text-title`), a `Badge` `Tag` with `wiki.kind.*` by kind, `rev. N` (`tabular-nums text-caption`), the license line. `entry === undefined` → skeletons; `null` or unparseable → `EmptyCategory` `wiki.states.unknown` + hint + a `Button` back to the category (`tabs.navigate({ name: Wiki, query: { category } })`). Else `WikiInfobox` and `WikiSections`. `onNavigate(target, newTab)` → `pageLocation(target)` and `tabs.open` / `tabs.navigate`; `iconFor = wiki.iconFor`, `canOpen = wiki.hasPage`.

- [ ] **Step 8: `WikiInfobox.vue`.** Props `infobox: Infobox`, `iconFor`, `canOpen`, emits `navigate`. A `switch` on `infobox.kind` in the template (`v-if` chain ending in `assertNever`): `item`/`trinket` render nothing; the rows per the spec's table as `<dl>` with `text-label` labels and `WikiInline` values; a `Target | null` field becomes `[{ kind: 'ref', target, label: wiki.titleOf(pageKey(target)) ?? pageKey(target) }]`; empty arrays and null draw `wiki.infobox.none`; the three challenge flags as `Badge`s (`Done` when true, `Tag` when false with the label struck? — no: only the true ones are listed, and a line "nessuna restrizione" when none). Character stats as a five-cell strip of `text-caption` label + `text-row tabular-nums` value.

- [ ] **Step 9: `WikiSections.vue`.** Props `sections: Section[]`, resolvers, emits `navigate`. Each section: `h2` `text-control text-highlight` with `wiki.section.*` (`Record<SectionKind, MessageKey>` in `wikiLabels.ts`) and `WikiBlocks`. No sections → `EmptyCategory` `wiki.states.noSections`.

- [ ] **Step 10: Wire.** `routes.ts`: `[RouteName.Wiki]: WikiScreen`; `routeTable.ts`: remove `routeArrives[RouteName.Wiki]`; `App.vue`: `tabViews` label through `tabLabel(tab.location, wiki.titleOf)` — a `Message` is translated, `{ text }` is shown as is; `useWikiStore` in `App.vue` loads the index lazily: a `watch` on "any tab is a page" is over-engineering — call `wiki.loadIndex()` in `WikiScreen` only, and `titleOf` returns null until then, which the spec allows.

- [ ] **Step 11: Run** `pnpm typecheck`, `pnpm ui:test`, `pnpm lint`, `pnpm format:check`, `pnpm scan`.

- [ ] **Step 12: Commit** — `feat(ui): the Wiki screen, a category list and a page in a tab`

---

### Task 8: looked at, the suite, and the documents

- [ ] **Step 1:** `pnpm ui:dev` in the background; through headless Chrome (`chrome --headless --dump-dom` / the Playwright tools) check: the landing's six counts; `?category=items` lists 909 rows (the pack's) and "d6" filters to one; `page=item:105` shows The D6 with a figure, the kind badge, 26 references with icons, a pickup ref drawn dotted; clicking a ref replaces the page; Ctrl+click opens a tab labelled with the page's title; `page=entity:20.0.0` shows the boss infobox with base HP 250; `page=item:1` says the dataset doesn't know it; `?catalog=none` draws placeholders and the line; `?wiki=none` shows the alert. Record findings in the session log.
- [ ] **Step 2:** `sh scripts/check` — green; list the skips.
- [ ] **Step 3:** `DESIGN-BRIEF.md` §8 gains `WikiIndex` / `WikiPageRef` and the `wiki_index` command; `docs/STATUS.md`: 3.5a checked, session log entry; `docs/BACKLOG.md` B5: A done, B next.
- [ ] **Step 4: Commit** — `docs: cycle 3.5a lands, the Wiki in tabs`; then merge into `develop` with `--no-ff` once `scripts/check` is green on the branch.
