# Cycle 3.4 — Collection: Implementation Plan

> **For agentic workers:** executed inline with superpowers:executing-plans, on the owner's
> delegation ("procedi"). Steps use checkbox (`- [ ]`) syntax. Every commit is pushed right away
> (the owner's rule: the remote stays aligned). Branch `feature/screens-collection`.

**Goal:** the Collection on real data — the save's item collection joined with the catalog's
collectibles, one state per item, facets on quality, pool, kind and origin over a virtualized
table.

**Architecture:** a pure `ipc::collection_view` joins section 4 (one slot per collectible id)
and section 1 (the unlocking achievement) with the catalog; a `collection` command wires it and
`design-export` writes it into the pack. The frontend mirrors the types, decides an item's state
and filters in pure functions under `lib/collection/`, and draws a virtualized table like
Unlock's. Until the pack carries `collection.json`, the fixture declares its quality, pools and
collection flags synthetic.

**Tech Stack:** Rust (`crates/ipc`, `crates/app`, `crates/design-export`), Vue 3.5, TypeScript,
Pinia, Tailwind v4, Reka UI, vue-i18n, lodash-es, `@tanstack/vue-virtual` 3.13.37, Vitest.

**Spec:** `docs/superpowers/specs/2026-09-11-screens-collection-design.md`

## Global Constraints

- Collectibles only (passive, active, familiar): trinkets have no slot in section 4.
- Unread is never "not in the collection": a missing section 4 gives `inCollection: null`, never `false`.
- The flag's name is structural: "in the collection" (`in_collection`), never "seen" or "picked up".
- No hardcoded counts: slots from the file, items from the catalog.
- IPC: `#[serde(rename_all = "camelCase")]` on structs; tagged enums with `rename_all_fields`; exhaustive matches; no `unwrap`/`panic` outside tests.
- Frontend: the five rules, `assertNever`, `as const`, every visible string through `useMessages()`; item and pool names are data.
- Fixtures only under `import.meta.env.DEV`; payloads through `import.meta.glob`.
- Checks judged by exit code; never two `vue-tsc --build` at once; Cargo with `CARGO_BUILD_JOBS=4` on this machine.
- Commits: Conventional Commits, English, no `Co-Authored-By` or Claude reference, **pushed after each task**.

---

### Task 1: `ipc::collection_view`

**Files:** Create `crates/ipc/src/collection.rs`, `crates/ipc/tests/collection.rs`, `crates/ipc/tests/collection_real.rs`; Modify `crates/ipc/src/lib.rs`, `crates/ipc/src/graph.rs` (`origin_view` becomes `pub(crate)`)

**Interfaces:** Produces `collection_view(catalog: Option<&Catalog>, items: Option<&[bool]>, achievements: Option<&[bool]>, icon: impl FnMut(&IconRef) -> Option<String>) -> CollectionView`; `CollectionView { items, pools, totals, diagnostics }`; `CollectionItem { id, kind, name, icon_url, quality, pools, origin, in_collection, lock }`; `CollectionTotals { slots, items, in_collection }`; `LockView { Free, Unlocked, Locked, Unknown }`; `CollectionDiagnostic { NoCatalog, NoCollectionSection, NoAchievementSection, ItemsBeyondSlots { count } }`.

- [x] **Step 1: Failing tests** — `crates/ipc/tests/collection.rs`:

```rust
//! The Collection as the UI sees it: section 4 joined with the catalog's collectibles. The
//! tests that matter keep "unread" from ever reading as "not in the collection".

use catalog::Catalog;
use ipc::{collection_view, CollectionDiagnostic, CollectionView, LockView};
use serde_json::{json, to_value};

const ITEMS: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"1\" gfx=\"a.png\" name=\"A\" achievement=\"1\" /><active id=\"2\" gfx=\"b.png\" name=\"B\" /><familiar id=\"5\" gfx=\"c.png\" name=\"C\" achievement=\"2\" /><trinket id=\"1\" gfx=\"t.png\" name=\"T\" /></items>";
const META: &[u8] = b"<items><item id=\"1\" quality=\"4\" tags=\"\"/><item id=\"2\" quality=\"1\" tags=\"\"/></items>";
const POOLS: &[u8] = b"<ItemPools><Pool Name=\"treasure\"><Item Id=\"1\" Weight=\"1\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/><Item Id=\"2\" Weight=\"1\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/></Pool><Pool Name=\"boss\"><Item Id=\"1\" Weight=\"1\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/></Pool><Pool Name=\"angel\"></Pool></ItemPools>";
const ACH: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\"><achievement id=\"1\" text=\"t1\" gfx=\"1.png\" /><achievement id=\"2\" text=\"t2\" gfx=\"2.png\" /></achievements>";

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS.to_vec()),
        "items_metadata.xml" => Some(META.to_vec()),
        "itempools.xml" => Some(POOLS.to_vec()),
        "achievements.xml" => Some(ACH.to_vec()),
        _ => None,
    })
}

// Slots 0..=3: item 1 is in the collection, item 2 isn't, item 5 has no slot.
const SLOTS: [bool; 4] = [false, true, false, false];
// Achievement 1 done, 2 not.
const DONE: [bool; 3] = [false, true, false];

fn view(items: Option<&[bool]>, achievements: Option<&[bool]>) -> CollectionView {
    collection_view(Some(&catalog()), items, achievements, |_| None)
}

#[test]
fn the_shapes_are_pinned() {
    assert_eq!(to_value(LockView::Free).unwrap(), json!({ "kind": "free" }));
    assert_eq!(
        to_value(LockView::Locked { achievement: 2, text: Some("t2".into()) }).unwrap(),
        json!({ "kind": "locked", "achievement": 2, "text": "t2" })
    );
    assert_eq!(
        to_value(CollectionDiagnostic::ItemsBeyondSlots { count: 1 }).unwrap(),
        json!({ "kind": "itemsBeyondSlots", "count": 1 })
    );
    let v = to_value(view(Some(&SLOTS), Some(&DONE))).unwrap();
    assert_eq!(v["items"][0]["inCollection"], json!(true));
    assert_eq!(v["items"][0]["iconUrl"], json!(null));
    assert_eq!(v["totals"]["inCollection"], json!(1));
    assert_eq!(v["items"][2]["kind"], json!("familiar"));
}

#[test]
fn only_collectibles_are_listed_by_id() {
    let v = view(Some(&SLOTS), Some(&DONE));
    assert_eq!(v.items.iter().map(|i| i.id).collect::<Vec<_>>(), vec![1, 2, 5]);
    assert_eq!(v.items[0].name, "A");
}

#[test]
fn the_collection_flag_comes_from_the_slot_and_a_missing_slot_is_null() {
    let v = view(Some(&SLOTS), Some(&DONE));
    assert_eq!(
        v.items.iter().map(|i| i.in_collection).collect::<Vec<_>>(),
        vec![Some(true), Some(false), None]
    );
    assert_eq!((v.totals.slots, v.totals.items, v.totals.in_collection), (4, 3, 1));
    assert_eq!(v.diagnostics, vec![CollectionDiagnostic::ItemsBeyondSlots { count: 1 }]);
}

#[test]
fn an_unread_collection_is_null_everywhere_and_says_so() {
    let v = view(None, Some(&DONE));
    assert!(v.items.iter().all(|i| i.in_collection.is_none()));
    assert_eq!((v.totals.slots, v.totals.in_collection), (0, 0));
    assert_eq!(v.diagnostics, vec![CollectionDiagnostic::NoCollectionSection]);
}

#[test]
fn locks_follow_the_unlocking_achievement() {
    let v = view(Some(&SLOTS), Some(&DONE));
    assert_eq!(
        v.items.iter().map(|i| i.lock.clone()).collect::<Vec<_>>(),
        vec![
            LockView::Unlocked { achievement: 1, text: Some("t1".into()) },
            LockView::Free,
            LockView::Locked { achievement: 2, text: Some("t2".into()) },
        ]
    );
    let unread = view(Some(&SLOTS), None);
    assert_eq!(
        unread.items.iter().map(|i| i.lock.clone()).collect::<Vec<_>>(),
        vec![
            LockView::Unknown { achievement: 1, text: Some("t1".into()) },
            LockView::Free,
            LockView::Unknown { achievement: 2, text: Some("t2".into()) },
        ]
    );
    assert!(unread.diagnostics.contains(&CollectionDiagnostic::NoAchievementSection));
}

#[test]
fn quality_pools_and_origin_come_from_the_catalog() {
    let v = view(Some(&SLOTS), Some(&DONE));
    assert_eq!(
        v.items.iter().map(|i| i.quality).collect::<Vec<_>>(),
        vec![Some(4), Some(1), None]
    );
    assert_eq!(v.items[0].pools, vec!["treasure", "boss"]);
    assert_eq!(v.items[1].pools, vec!["treasure"]);
    assert!(v.items[2].pools.is_empty());
    assert_eq!(v.pools, vec!["treasure", "boss"], "angel holds no listed item");
    assert_eq!(to_value(&v.items[0].origin).unwrap(), json!("rebirth"));
}

#[test]
fn without_a_catalog_only_the_totals_speak() {
    let flags = [false, true, true];
    let v = collection_view(None, Some(&flags), None, |_| None);
    assert!(v.items.is_empty() && v.pools.is_empty());
    assert_eq!((v.totals.slots, v.totals.items, v.totals.in_collection), (3, 0, 2));
    assert_eq!(v.diagnostics, vec![CollectionDiagnostic::NoCatalog]);
}
```

`crates/ipc/tests/collection_real.rs` — properties, skipping without the game:

```rust
//! The Collection against the real catalog (`samples/packed`) and the live profile. Properties,
//! not values: they hold on any profile and any patch.

use catalog::Catalog;
use core_save::{Kind, Save};
use unpack::ResourceSet;

#[test]
fn every_collectible_the_save_has_a_slot_for_reads_as_true_or_false() {
    let Some(packed) = test_support::packed_dir() else { return };
    let Some(sample) = test_support::sample("live.rep+persistentgamedata1.dat") else { return };
    let rs = ResourceSet::open(&packed);
    let c = Catalog::build(|p| rs.read(p));
    let Ok(save) = Save::open(&sample) else {
        test_support::skip("the live profile exists but doesn't read");
        return;
    };
    let items = save.flags(Kind::Items).expect("section 4");
    let v = ipc::collection_view(Some(&c), Some(&items), save.flags(Kind::Achievements).as_deref(), |_| None);
    for item in &v.items {
        let slot = items.get(item.id as usize).copied();
        assert_eq!(item.in_collection, slot, "item {} against its slot", item.id);
    }
    let set = v.items.iter().filter(|i| i.in_collection == Some(true)).count() as u32;
    assert_eq!(v.totals.in_collection, set);
    assert!(v.items.iter().all(|i| i.kind != ipc::ItemKindView::Trinket));
}
```

- [x] **Step 2: Run** `cargo test -p ipc --test collection` → FAIL (no `collection_view`).

- [x] **Step 3: Implement** — in `graph.rs`, `fn origin_view` becomes `pub(crate) fn origin_view`. `crates/ipc/src/collection.rs`:

```rust
//! The Collection: the save's item collection (section 4) joined with the catalog's
//! collectibles. Section 4 holds one slot per collectible id — the catalog's ids are a subset
//! of its slots, and the gaps are exactly the unused ids — so trinkets, which have none, aren't
//! listed. What a set byte means in play isn't measured: the view calls it "in the collection",
//! the section's own name.

use catalog::{AchievementId, Catalog, Item, ItemKind, Language};
use serde::Serialize;

use crate::catalog_view::kind_view;
use crate::graph::origin_view;
use crate::{IconRef, ItemKindView, OriginView};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionView {
    pub items: Vec<CollectionItem>,
    /// The pools any listed item belongs to, each once, in the catalog's order.
    pub pools: Vec<String>,
    pub totals: CollectionTotals,
    pub diagnostics: Vec<CollectionDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionTotals {
    /// Section 4's length; 0 when it wasn't read.
    pub slots: u32,
    pub items: u32,
    pub in_collection: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionItem {
    pub id: u32,
    pub kind: ItemKindView,
    pub name: String,
    pub icon_url: Option<String>,
    pub quality: Option<i8>,
    pub pools: Vec<String>,
    pub origin: Option<OriginView>,
    /// `None` when section 4 wasn't read, or the save has no slot for this id: unread is
    /// never "not in the collection".
    pub in_collection: Option<bool>,
    pub lock: LockView,
}

/// What stands between the item and a run. Tagged: three of the four carry data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum LockView {
    /// Nothing unlocks it: it is in the game from the start.
    Free,
    Unlocked { achievement: u32, text: Option<String> },
    /// Its achievement isn't done: the item can't appear in a run yet.
    Locked { achievement: u32, text: Option<String> },
    /// Section 1 wasn't read: whether the achievement is done isn't known.
    Unknown { achievement: u32, text: Option<String> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum CollectionDiagnostic {
    NoCatalog,
    NoCollectionSection,
    NoAchievementSection,
    /// Collectibles the save has no slot for: a catalog newer than the save.
    ItemsBeyondSlots { count: u32 },
}

pub fn collection_view(
    catalog: Option<&Catalog>,
    items: Option<&[bool]>,
    achievements: Option<&[bool]>,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> CollectionView {
    let slots = items.map_or(0, |f| f.len() as u32);
    let Some(c) = catalog else {
        // No names, no ids to list: the section still says how much of it is set. Slot 0 is
        // no item.
        let set = items.map_or(0, |f| f.iter().skip(1).filter(|b| **b).count() as u32);
        return CollectionView {
            items: Vec::new(),
            pools: Vec::new(),
            totals: CollectionTotals {
                slots,
                items: 0,
                in_collection: set,
            },
            diagnostics: vec![CollectionDiagnostic::NoCatalog],
        };
    };

    let mut listed: Vec<&Item> = c.items().filter(|i| i.kind != ItemKind::Trinket).collect();
    listed.sort_by_key(|i| i.id.0);

    let mut rows = Vec::with_capacity(listed.len());
    let mut beyond = 0u32;
    for i in listed {
        let kind = kind_view(i.kind);
        let in_collection = items.and_then(|f| f.get(i.id.0 as usize).copied());
        if items.is_some() && in_collection.is_none() {
            beyond += 1;
        }
        let mut pools: Vec<String> = Vec::new();
        for m in &i.pools {
            if !pools.contains(&m.pool) {
                pools.push(m.pool.clone());
            }
        }
        rows.push(CollectionItem {
            id: i.id.0,
            kind,
            name: c.text(&i.name, Language::English).to_string(),
            icon_url: icon(&IconRef::Item { kind, id: i.id.0 }),
            quality: i.quality,
            pools,
            origin: i.origin.map(origin_view),
            in_collection,
            lock: lock_of(c, i.unlocked_by, achievements),
        });
    }

    let pools = c
        .pools()
        .iter()
        .map(|p| p.name.clone())
        .filter(|name| rows.iter().any(|r| r.pools.contains(name)))
        .collect();

    let mut diagnostics = Vec::new();
    if items.is_none() {
        diagnostics.push(CollectionDiagnostic::NoCollectionSection);
    }
    if achievements.is_none() {
        diagnostics.push(CollectionDiagnostic::NoAchievementSection);
    }
    if beyond > 0 {
        diagnostics.push(CollectionDiagnostic::ItemsBeyondSlots { count: beyond });
    }

    let in_collection = rows.iter().filter(|r| r.in_collection == Some(true)).count() as u32;
    CollectionView {
        totals: CollectionTotals {
            slots,
            items: rows.len() as u32,
            in_collection,
        },
        items: rows,
        pools,
        diagnostics,
    }
}

/// A slot past section 1's end reads as not done: the save has no record of it.
fn lock_of(c: &Catalog, unlocked_by: Option<AchievementId>, achievements: Option<&[bool]>) -> LockView {
    let Some(a) = unlocked_by else {
        return LockView::Free;
    };
    let achievement = a.0;
    let text = c.achievement(a).map(|x| x.text.clone());
    match achievements.map(|f| f.get(achievement as usize).copied().unwrap_or(false)) {
        None => LockView::Unknown { achievement, text },
        Some(true) => LockView::Unlocked { achievement, text },
        Some(false) => LockView::Locked { achievement, text },
    }
}
```

`lib.rs`: `mod collection;` and `pub use collection::{collection_view, CollectionDiagnostic, CollectionItem, CollectionTotals, CollectionView, LockView};`. If `catalog` doesn't re-export `Item` or `AchievementId` under those names, import them from where `crates/ipc/src/graph.rs` does.

- [x] **Step 4: Run** `cargo test -p ipc --test collection --test collection_real` → green (the real test prints its skip); `cargo fmt`, `cargo clippy -p ipc --all-targets -- -D warnings` by exit code. Commit `feat(ipc): the Collection, the save's item collection joined with the catalog` and push.

### Task 2: The command, the pack's payload, the TypeScript mirror

**Files:** Modify `crates/app/src/lib.rs`, `crates/design-export/src/payload.rs`, `ui/src/lib/ipc/types.ts`, `ui/src/lib/constants/commands.ts`; Create `ui/src/lib/ipc/collection.ts`

**Interfaces:** Consumes Task 1. Produces Tauri `collection() -> Result<ipc::CollectionView, IpcError>`; TS `CollectionView`, `CollectionItem`, `CollectionTotals`, `LockView`, `CollectionDiagnostic`; `Command.Collection = 'collection'`; `collection(): Promise<CollectionView>`.

- [x] **Step 1: Command** — in `crates/app/src/lib.rs`, after `next_steps`:

```rust
#[tauri::command]
fn collection(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
) -> Result<ipc::CollectionView, IpcError> {
    let (_, save) = active_save(&app)?;
    // Game not installed is expected: the view goes out without a catalog and says so.
    let resources = resources.get();
    let catalog = resources.and_then(|rs| state.get_or_build(rs));
    let items = save.flags(Kind::Items);
    let achievements = save.flags(Kind::Achievements);
    Ok(ipc::collection_view(
        catalog,
        items.as_deref(),
        achievements.as_deref(),
        icon_url,
    ))
}
```

and `collection,` in `generate_handler!` after `next_steps`.

- [x] **Step 2: The pack** — in `payload.rs`, after `next_steps.json`:

```rust
    // The Collection needs the save's item collection: without a profile there is nothing
    // to join, and the pack goes without it rather than with a view of nobody's save.
    if let Some((_, s)) = save {
        let items = s.flags(Kind::Items);
        let collection = ipc::collection_view(Some(c), items.as_deref(), flag.as_deref(), canonical);
        record(
            "contracts/payload/collection.json",
            write_json(root, "contracts/payload/collection.json", &collection)?,
        );
    }
```

- [x] **Step 3: TypeScript** — `types.ts`, after the queue block:

```ts
// --- The Collection ---

// What stands between an item and a run. Tagged: three variants carry data.
export type LockView =
  | { kind: 'free' }
  | { kind: 'unlocked'; achievement: number; text: string | null }
  | { kind: 'locked'; achievement: number; text: string | null }
  | { kind: 'unknown'; achievement: number; text: string | null }

// A collectible as the Collection shows it. `inCollection: null` is unread — section 4
// missing, or no slot for this id — never "not in the collection".
export interface CollectionItem {
  id: number
  kind: ItemKindView
  name: string
  iconUrl: string | null
  quality: number | null
  pools: string[]
  origin: OriginView | null
  inCollection: boolean | null
  lock: LockView
}

export interface CollectionTotals {
  slots: number
  items: number
  inCollection: number
}

export type CollectionDiagnostic =
  | { kind: 'noCatalog' }
  | { kind: 'noCollectionSection' }
  | { kind: 'noAchievementSection' }
  | { kind: 'itemsBeyondSlots'; count: number }

export interface CollectionView {
  items: CollectionItem[]
  pools: string[]
  totals: CollectionTotals
  diagnostics: CollectionDiagnostic[]
}
```

`commands.ts`: `Collection: 'collection',`. `lib/ipc/collection.ts`:

```ts
import { Command } from '../constants/commands'
import { call } from './transport'
import type { CollectionView } from './types'

export const collection = (): Promise<CollectionView> => call(Command.Collection)
```

- [x] **Step 4: Verify** `cargo build -p app`, `cargo build -p design-export`, `cargo clippy -p app -p design-export --all-targets -- -D warnings`, `pnpm typecheck` by exit code. Commit `feat(app): the collection command, its pack payload and its TypeScript mirror` and push.

### Task 3: An item's state and the Collection's facets

**Files:** Create `ui/src/lib/collection/itemState.ts`, `itemState.test.ts`, `collectionFilter.ts`, `collectionFilter.test.ts`

**Interfaces:** Produces `ItemState { InCollection, Available, Locked, Unknown }`, `itemStateOrder`, `itemState(item)`, `itemStateCounts(items): Record<ItemState, number>`; `CollectionFacet { State, Quality, Pool, Kind, Origin }`, `collectionFacetOrder`, `QualityValue`, `NoPool = 'none'`, `CollectionSort { Quality, Id, Name }`, `CollectionFilter { query; picks: Record<CollectionFacet, string[]> }`, `defaultCollectionFilter()`, `emptyCollectionFilter()`, `collectionFacetValues(item, facet)`, `matchesCollectionFilter(item, filter)`, `collectionFacetCounts(items, filter, facet)`, `collectionFacetOptions(pools, facet)`, `sortItems(items, sort)`, `activeCollectionFilterCount(filter)`.

- [x] **Step 1: Failing tests** — a shared builder in each test file:

```ts
import type { CollectionItem, LockView } from '@/lib/ipc/types'

const item = (over: Partial<CollectionItem> = {}): CollectionItem => ({
  id: 1,
  kind: 'passive',
  name: 'The Sad Onion',
  iconUrl: null,
  quality: 3,
  pools: ['treasure'],
  origin: 'rebirth',
  inCollection: false,
  lock: { kind: 'free' },
  ...over,
})
const locked: LockView = { kind: 'locked', achievement: 62, text: 'Epic Fetus' }
```

`itemState.test.ts`:

```ts
describe('itemState', () => {
  it('is in the collection whatever the lock says', () => {
    expect(itemState(item({ inCollection: true, lock: locked }))).toBe(ItemState.InCollection)
  })
  it('is to find when free or unlocked', () => {
    expect(itemState(item())).toBe(ItemState.Available)
    expect(itemState(item({ lock: { kind: 'unlocked', achievement: 1, text: null } }))).toBe(ItemState.Available)
  })
  it('is locked when its achievement is not done', () => {
    expect(itemState(item({ lock: locked }))).toBe(ItemState.Locked)
  })
  it('is unknown when the collection or the lock is unread', () => {
    expect(itemState(item({ inCollection: null }))).toBe(ItemState.Unknown)
    expect(itemState(item({ lock: { kind: 'unknown', achievement: 62, text: null } }))).toBe(ItemState.Unknown)
  })
  it('counts every state', () => {
    expect(
      itemStateCounts([item({ inCollection: true }), item(), item({ lock: locked }), item({ lock: locked })]),
    ).toEqual({ inCollection: 1, available: 1, locked: 2, unknown: 0 })
  })
})
```

`collectionFilter.test.ts`:

```ts
const rows = [
  item({ id: 1, name: 'The Sad Onion', quality: 3, pools: ['treasure'], origin: 'rebirth' }),
  item({ id: 2, name: 'The Inner Eye', quality: 2, pools: ['treasure', 'boss'], inCollection: true }),
  item({ id: 168, name: 'Epic Fetus', quality: 4, pools: [], origin: 'rebirth', lock: locked }),
  item({ id: 600, name: 'Unrated', quality: null, pools: ['devil'], origin: null, kind: 'familiar' }),
]
const ids = (xs: CollectionItem[]) => xs.map((x) => x.id)
const pick = (facet: CollectionFacet, values: string[]) => ({
  ...emptyCollectionFilter(),
  picks: { ...emptyCollectionFilter().picks, [facet]: values },
})

describe('the Collection facets', () => {
  it('gives each item its values', () => {
    expect(collectionFacetValues(rows[3], CollectionFacet.Quality)).toEqual(['unrated'])
    expect(collectionFacetValues(rows[2], CollectionFacet.Pool)).toEqual(['none'])
    expect(collectionFacetValues(rows[1], CollectionFacet.Pool)).toEqual(['treasure', 'boss'])
    expect(collectionFacetValues(rows[3], CollectionFacet.Origin)).toEqual(['none'])
    expect(collectionFacetValues(rows[3], CollectionFacet.Kind)).toEqual(['familiar'])
  })
  it('matches any value in a facet, and every facet', () => {
    expect(ids(rows.filter((r) => matchesCollectionFilter(r, pick(CollectionFacet.Pool, ['boss', 'devil']))))).toEqual([2, 600])
    const both = { ...pick(CollectionFacet.Pool, ['treasure']), picks: { ...pick(CollectionFacet.Pool, ['treasure']).picks, quality: ['3'] } }
    expect(ids(rows.filter((r) => matchesCollectionFilter(r, both)))).toEqual([1])
  })
  it('searches the name, case-insensitive', () => {
    expect(ids(rows.filter((r) => matchesCollectionFilter(r, { ...emptyCollectionFilter(), query: 'EYE' })))).toEqual([2])
  })
  it('counts a facet over what the other facets leave', () => {
    const counts = collectionFacetCounts(rows, pick(CollectionFacet.Quality, ['3']), CollectionFacet.Quality)
    expect(counts.get('3')).toBe(1)
    expect(counts.get('4')).toBe(1)
    expect(collectionFacetCounts(rows, pick(CollectionFacet.Quality, ['3']), CollectionFacet.Pool).get('treasure')).toBe(1)
  })
  it('defaults to what has not been found', () => {
    expect(ids(rows.filter((r) => matchesCollectionFilter(r, defaultCollectionFilter())))).toEqual([1, 168, 600])
  })
  it('offers the view pools and then no pool', () => {
    expect(collectionFacetOptions(['treasure', 'boss'], CollectionFacet.Pool)).toEqual(['treasure', 'boss', 'none'])
    expect(collectionFacetOptions([], CollectionFacet.Quality)).toEqual(['4', '3', '2', '1', '0', 'unrated'])
  })
  it('sorts by quality, id and name', () => {
    expect(ids(sortItems(rows, CollectionSort.Quality))).toEqual([168, 1, 2, 600])
    expect(ids(sortItems([...rows].reverse(), CollectionSort.Id))).toEqual([1, 2, 168, 600])
    expect(ids(sortItems(rows, CollectionSort.Name))).toEqual([168, 2, 1, 600])
  })
})
```

(`Unrated` sorts after `The Sad Onion` by name: "the inner eye" < "the sad onion" < "unrated".)

- [x] **Step 2: Run** `pnpm --filter ui exec vitest run src/lib/collection` → FAIL.

- [x] **Step 3: Implement** `itemState.ts`:

```ts
import { countBy } from 'lodash-es'
import { assertNever } from '@/lib/assertNever'
import type { CollectionItem } from '@/lib/ipc/types'

// What a collectible is for this save, one answer for every place that draws it. Unread is
// its own state: a missing section never reads as "not in the collection".
export const ItemState = {
  InCollection: 'inCollection',
  Available: 'available',
  Locked: 'locked',
  Unknown: 'unknown',
} as const
export type ItemState = (typeof ItemState)[keyof typeof ItemState]

export const itemStateOrder: ItemState[] = [
  ItemState.InCollection,
  ItemState.Available,
  ItemState.Locked,
  ItemState.Unknown,
]

export const itemState = (item: CollectionItem): ItemState => {
  if (item.inCollection === null) return ItemState.Unknown
  if (item.inCollection) return ItemState.InCollection
  const { lock } = item
  switch (lock.kind) {
    case 'free':
    case 'unlocked':
      return ItemState.Available
    case 'locked':
      return ItemState.Locked
    case 'unknown':
      return ItemState.Unknown
    default:
      return assertNever(lock)
  }
}

export const itemStateCounts = (items: CollectionItem[]): Record<ItemState, number> => {
  const counted = countBy(items, itemState)
  const count = (state: ItemState): number => counted[state] ?? 0
  return {
    [ItemState.InCollection]: count(ItemState.InCollection),
    [ItemState.Available]: count(ItemState.Available),
    [ItemState.Locked]: count(ItemState.Locked),
    [ItemState.Unknown]: count(ItemState.Unknown),
  }
}
```

`collectionFilter.ts`:

```ts
import { countBy, sortBy, sumBy } from 'lodash-es'
import { assertNever } from '@/lib/assertNever'
import { OriginValue } from '@/lib/graph/unlockFilter'
import type { CollectionItem } from '@/lib/ipc/types'
import { ItemKindView } from '@/lib/ipc/types'
import { ItemState, itemState, itemStateOrder } from './itemState'

export const CollectionFacet = {
  State: 'state',
  Quality: 'quality',
  Pool: 'pool',
  Kind: 'kind',
  Origin: 'origin',
} as const
export type CollectionFacet = (typeof CollectionFacet)[keyof typeof CollectionFacet]

export const collectionFacetOrder: CollectionFacet[] = [
  CollectionFacet.State,
  CollectionFacet.Quality,
  CollectionFacet.Pool,
  CollectionFacet.Kind,
  CollectionFacet.Origin,
]

// items_metadata.xml rates collectibles 0 to 4; an item it doesn't rate is a value of its own.
export const QualityValue = {
  Four: '4',
  Three: '3',
  Two: '2',
  One: '1',
  Zero: '0',
  Unrated: 'unrated',
} as const
export type QualityValue = (typeof QualityValue)[keyof typeof QualityValue]
const qualityOrder: QualityValue[] = Object.values(QualityValue)

// Pool names are the game's own strings; an item in none is a value of its own.
export const NoPool = 'none'

const kindOrder: string[] = [ItemKindView.Passive, ItemKindView.Active, ItemKindView.Familiar]
const originOrder: string[] = Object.values(OriginValue)

export const CollectionSort = { Quality: 'quality', Id: 'id', Name: 'name' } as const
export type CollectionSort = (typeof CollectionSort)[keyof typeof CollectionSort]

export interface CollectionFilter {
  query: string
  picks: Record<CollectionFacet, string[]>
}

export const emptyCollectionFilter = (): CollectionFilter => ({
  query: '',
  picks: {
    [CollectionFacet.State]: [],
    [CollectionFacet.Quality]: [],
    [CollectionFacet.Pool]: [],
    [CollectionFacet.Kind]: [],
    [CollectionFacet.Origin]: [],
  },
})

// The screen opens on what hasn't been found: to find, and locked.
export const defaultCollectionFilter = (): CollectionFilter => {
  const empty = emptyCollectionFilter()
  return {
    ...empty,
    picks: { ...empty.picks, [CollectionFacet.State]: [ItemState.Available, ItemState.Locked] },
  }
}

export const collectionFacetValues = (item: CollectionItem, facet: CollectionFacet): string[] => {
  switch (facet) {
    case CollectionFacet.State:
      return [itemState(item)]
    case CollectionFacet.Quality:
      return [item.quality === null ? QualityValue.Unrated : String(item.quality)]
    case CollectionFacet.Pool:
      return item.pools.length > 0 ? item.pools : [NoPool]
    case CollectionFacet.Kind:
      return [item.kind]
    case CollectionFacet.Origin:
      return [item.origin ?? OriginValue.None]
    default:
      return assertNever(facet)
  }
}

const matchesQuery = (item: CollectionItem, query: string): boolean => {
  const wanted = query.trim().toLowerCase()
  return wanted === '' || item.name.toLowerCase().includes(wanted)
}

const matchesFacets = (item: CollectionItem, filter: CollectionFilter, facets: CollectionFacet[]): boolean =>
  matchesQuery(item, filter.query) &&
  facets.every((facet) => {
    const picked = filter.picks[facet]
    return picked.length === 0 || collectionFacetValues(item, facet).some((v) => picked.includes(v))
  })

export const matchesCollectionFilter = (item: CollectionItem, filter: CollectionFilter): boolean =>
  matchesFacets(item, filter, collectionFacetOrder)

// A value's count leaves its own facet out: it says how many rows picking it would give.
export const collectionFacetCounts = (
  items: CollectionItem[],
  filter: CollectionFilter,
  facet: CollectionFacet,
): Map<string, number> => {
  const others = collectionFacetOrder.filter((f) => f !== facet)
  const values = items
    .filter((item) => matchesFacets(item, filter, others))
    .flatMap((item) => collectionFacetValues(item, facet))
  return new Map(Object.entries(countBy(values)))
}

export const collectionFacetOptions = (pools: string[], facet: CollectionFacet): string[] => {
  switch (facet) {
    case CollectionFacet.State:
      return itemStateOrder
    case CollectionFacet.Quality:
      return qualityOrder
    case CollectionFacet.Pool:
      return [...pools, NoPool]
    case CollectionFacet.Kind:
      return kindOrder
    case CollectionFacet.Origin:
      return originOrder
    default:
      return assertNever(facet)
  }
}

// Every order ends on the id, so equal rows never swap between two renders. Unrated items go
// after quality 0.
export const sortItems = (items: CollectionItem[], sort: CollectionSort): CollectionItem[] => {
  switch (sort) {
    case CollectionSort.Quality:
      return sortBy(items, [(i) => -(i.quality ?? -1), 'id'])
    case CollectionSort.Id:
      return sortBy(items, 'id')
    case CollectionSort.Name:
      return sortBy(items, [(i) => i.name.toLowerCase(), 'id'])
    default:
      return assertNever(sort)
  }
}

export const activeCollectionFilterCount = (filter: CollectionFilter): number =>
  sumBy(collectionFacetOrder, (facet) => filter.picks[facet].length)
```

(`OriginValue` in `unlockFilter.ts` already holds the four origins and `none`; `ItemKindView` is imported as a value.)

- [x] **Step 4: Run** the tests → green; typecheck, lint, scan, format by exit code. Commit `feat(ui): a collectible's state and the Collection's facets, as pure functions` and push.

### Task 4: The Collection's fixture

**Files:** Create `ui/src/lib/ipc/fixtures/collection.ts`, `collection.test.ts`; Modify `ui/src/lib/ipc/fixtures/index.ts`

**Interfaces:** Consumes `graphAnswers`, `packIconUrl`. Produces `CollectionSource { Pack, Synthetic }`, `collectionSource(): CollectionSource`, `collectionAnswer({ withArt, withCatalog, collectionRead }): CollectionView`; the `collection` handler; `?collection=unread`.

- [x] **Step 1: Failing test** — `collection.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import type { CollectionItem } from '../types'
import { CollectionSource, collectionAnswer, collectionSource } from './collection'

const index = import.meta.glob<{ family: string; id: number; kind?: string; name?: string }[]>(
  '../../../../../design-export/isaacdome-design-pack/images/INDEX.json',
  { eager: true, import: 'default' },
)
const packItems = (Object.values(index)[0] ?? []).filter((e) => e.family === 'item' && e.kind !== 'trinket')
const byId = (items: CollectionItem[], id: number) => items.find((i) => i.id === id)

describe('the Collection fixture', () => {
  const view = collectionAnswer({ withArt: false, withCatalog: true, collectionRead: true })

  it('declares its source: synthetic until the pack carries collection.json', () => {
    expect(collectionSource()).toBe(CollectionSource.Synthetic)
  })

  it("lists the pack's collectibles by id, with their names and kinds", () => {
    expect(view.items).toHaveLength(packItems.length)
    expect(view.items.every((i) => i.kind !== 'trinket')).toBe(true)
    expect(byId(view.items, 1)).toMatchObject({ name: 'The Sad Onion', kind: 'passive' })
  })

  it('takes its locks from the unlock payload', () => {
    // contracts/payload/unlock.json: achievement 6 (done) unlocks Cube of Meat, 62 (not done)
    // unlocks Epic Fetus.
    expect(byId(view.items, 73)?.lock).toMatchObject({ kind: 'unlocked', achievement: 6 })
    expect(byId(view.items, 168)?.lock).toMatchObject({ kind: 'locked', achievement: 62 })
    expect(byId(view.items, 1)?.lock).toEqual({ kind: 'free' })
  })

  it('never puts a locked item in the collection', () => {
    expect(view.items.filter((i) => i.lock.kind === 'locked').every((i) => i.inCollection === false)).toBe(true)
  })

  it('answers an unread collection as null, and says so', () => {
    const unread = collectionAnswer({ withArt: false, withCatalog: true, collectionRead: false })
    expect(unread.items.every((i) => i.inCollection === null)).toBe(true)
    expect(unread.diagnostics).toEqual([{ kind: 'noCollectionSection' }])
  })

  it('answers without a catalog with totals only', () => {
    const none = collectionAnswer({ withArt: false, withCatalog: false, collectionRead: true })
    expect(none.items).toEqual([])
    expect(none.diagnostics).toEqual([{ kind: 'noCatalog' }])
  })
})
```

Before running, confirm the two lock examples against `unlock.json` (a node whose `unlocks` holds `{ itemKind: 'familiar', id: 73 }` with `done: true`, and one holding `{ itemKind: 'passive', id: 168 }` with `done: false`); the expectation follows the payload.

- [x] **Step 2: Run** `pnpm --filter ui exec vitest run src/lib/ipc/fixtures/collection.test.ts` → FAIL.

- [x] **Step 3: Implement** `collection.ts`:

```ts
import type { CollectionItem, CollectionView, ItemKindView, LockView, OriginView } from '../types'
import { graphAnswers } from './graph'
import { packIconUrl } from './graphArt'

// Development only. The pack's real collection.json once design-export has written it; until
// then the items come from the pack's image index (ids, kinds, names: real), their locks from
// unlock.json (real), and quality, pools and the collection flag from the id (synthetic).
const payloads = import.meta.glob<CollectionView>(
  '../../../../../design-export/isaacdome-design-pack/contracts/payload/collection.json',
  { eager: true, import: 'default' },
)
interface IndexEntry {
  family: string
  id: number
  kind?: string
  name?: string
}
const indexes = import.meta.glob<IndexEntry[]>(
  '../../../../../design-export/isaacdome-design-pack/images/INDEX.json',
  { eager: true, import: 'default' },
)

export const CollectionSource = { Pack: 'pack', Synthetic: 'synthetic' } as const
export type CollectionSource = (typeof CollectionSource)[keyof typeof CollectionSource]

const packed = (): CollectionView | null => Object.values(payloads)[0] ?? null

export const collectionSource = (): CollectionSource =>
  packed() ? CollectionSource.Pack : CollectionSource.Synthetic

export interface CollectionAnswerOptions {
  withArt: boolean
  withCatalog: boolean
  collectionRead: boolean
}

const collectibleKinds: string[] = ['passive', 'active', 'familiar']
const isCollectible = (kind: string | undefined): kind is ItemKindView =>
  kind !== undefined && collectibleKinds.includes(kind)

// crates/catalog/src/origin.rs: the last collectible id of each DLC.
const originOf = (id: number): OriginView | null => {
  if (id <= 0) return null
  if (id <= 341) return 'rebirth'
  if (id <= 440) return 'afterbirth'
  if (id <= 552) return 'afterbirthPlus'
  if (id <= 732) return 'repentance'
  return null
}

const syntheticPools = ['treasure', 'boss', 'shop', 'devil', 'angel', 'secret', 'library']
const poolsOf = (id: number): string[] => {
  if (id % 11 === 0) return []
  const first = syntheticPools[id % 7] ?? 'treasure'
  const second = syntheticPools[(id + 3) % 7] ?? 'boss'
  return id % 3 === 0 ? [first, second] : [first]
}
const qualityOf = (id: number): number | null => (id % 23 === 0 ? null : id % 5)

// The locks the reference profile's unlock view implies: an item a node unlocks is unlocked or
// locked by that node's done; any other item is free.
const locksFrom = (): Map<string, LockView> => {
  const nodes = graphAnswers({ withArt: false, withCatalog: true }).unlock.nodes
  return new Map(
    nodes.flatMap((node) =>
      node.achievement.kind === 'known'
        ? node.unlocks.flatMap((t) => {
            if (t.kind !== 'item' || node.achievement.kind !== 'known') return []
            const achievement = node.achievement.id
            const text = node.achievement.text
            const lock: LockView = node.done
              ? { kind: 'unlocked', achievement, text }
              : { kind: 'locked', achievement, text }
            return [[`${t.itemKind}-${t.id}`, lock] as const]
          })
        : [],
    ),
  )
}

const synthetic = (withArt: boolean, collectionRead: boolean): CollectionView => {
  const locks = locksFrom()
  const entries = (Object.values(indexes)[0] ?? [])
    .filter((e) => e.family === 'item' && isCollectible(e.kind))
    .sort((a, b) => a.id - b.id)
  const items: CollectionItem[] = entries.flatMap((e) => {
    if (!isCollectible(e.kind)) return []
    const lock = locks.get(`${e.kind}-${e.id}`) ?? { kind: 'free' }
    const link = `isaac://item/${e.kind}/${e.id}`
    return [
      {
        id: e.id,
        kind: e.kind,
        name: e.name ?? '',
        iconUrl: withArt ? packIconUrl(link) : null,
        quality: qualityOf(e.id),
        pools: poolsOf(e.id),
        origin: originOf(e.id),
        inCollection: collectionRead ? lock.kind !== 'locked' && e.id % 3 !== 0 : null,
        lock,
      },
    ]
  })
  const pools = syntheticPools.filter((p) => items.some((i) => i.pools.includes(p)))
  return {
    items,
    pools,
    totals: {
      slots: collectionRead ? 733 : 0,
      items: items.length,
      inCollection: items.filter((i) => i.inCollection === true).length,
    },
    diagnostics: collectionRead ? [] : [{ kind: 'noCollectionSection' }],
  }
}

let warned = false

export const collectionAnswer = ({
  withArt,
  withCatalog,
  collectionRead,
}: CollectionAnswerOptions): CollectionView => {
  if (!withCatalog)
    return {
      items: [],
      pools: [],
      totals: { slots: collectionRead ? 733 : 0, items: 0, inCollection: 0 },
      diagnostics: [{ kind: 'noCatalog' }],
    }
  const pack = packed()
  if (pack) return pack
  if (!warned && import.meta.env.DEV && typeof window !== 'undefined') {
    warned = true
    console.warn('Collection fixture: quality, pools and collection flags are synthetic until the design pack carries collection.json')
  }
  return synthetic(withArt, collectionRead)
}
```

(733 is the fixture's stand-in for the reference save's section 4, which the pack's `save_summary.json` declares; read it from there if the lint for magic numbers complains.)

`index.ts`: `const CollectionParam = 'collection'`; `collectionRead = (): boolean => query().get(CollectionParam) !== 'unread'`; a lazy `const collection = async () => { const { collectionAnswer } = await import('./collection'); return collectionAnswer({ withArt: artShown(), withCatalog: catalogShown(), collectionRead: collectionRead() }) }`; handler `[Command.Collection]: (_args, scenario) => whenActive(scenario, () => collection())`.

- [x] **Step 4: Run** the fixture tests and `src/lib/ipc` → green; typecheck, lint, scan, format. Commit `feat(ui): the Collection's fixture, real names and locks, declared synthetic flags` and push.

### Task 5: The Collection screen

**Files:** Create `ui/src/stores/collection.ts`, `ui/src/screens/CollectionScreen.vue`, `ui/src/screens/collection/{CollectionStateToggle.vue, CollectionFacetDrawer.vue, CollectionToolbar.vue, CollectionTable.vue, CollectionRow.vue, CollectionDiagnostics.vue, QualityPips.vue, collectionLabels.ts, collectionLayout.ts, collectionLayout.test.ts}`; Modify `spacing.css`, `utilities.css`, `StoreId`, `routes.ts`, `routeTable.ts`, `it.ts`, `en.ts`

**Interfaces:** Consumes Tasks 2–4.

- [x] **Step 1: The row height, pinned** — `collectionLayout.ts` exports `collectionRowHeight = 40`; its test reads `theme/spacing.css?raw` and expects `--spacing-row-wide: 40px`. Run → FAIL, write, → green.

- [x] **Step 2: Tokens** — `spacing.css`: `--spacing-collection-sprite: 56px; --spacing-collection-quality: 96px; --spacing-collection-origin: 104px; --spacing-collection-state: 136px; --spacing-quality-pip: 6px;` under a comment "the Collection's table (Schermate.dc.html, Collezione)". `utilities.css`:

```css
/* The Collection's table: the sprite, the name, quality, pools, origin and state. */
@utility grid-cols-collection {
  grid-template-columns:
    var(--spacing-collection-sprite)
    minmax(0, 1.4fr)
    var(--spacing-collection-quality)
    minmax(0, 1fr)
    var(--spacing-collection-origin)
    var(--spacing-collection-state);
}
```

- [x] **Step 3: Store** — `StoreId.Collection: 'collection'`; `stores/collection.ts` as `stores/completion.ts`: `view`, `status`, `error`, `load()` reading `collection()`.

- [x] **Step 4: Messages** — `collection.*` in both languages: `intro` (the export's line and that trinkets have no collection state), `state.{inCollection: 'in collezione', available: 'da trovare', locked: 'bloccato', unknown: 'non leggibile'}`, `facets`, `facet.{state, quality, pool, kind, origin}`, `quality.unrated: 'non valutato'`, `pool.none: 'nessun pool'`, `items: 'oggetti'`, `search: 'cerca un oggetto'`, `sortBy`, `sort.{quality, id, name}`, `columns.{item, quality, pools, origin, state}`, `id: 'id'`, `lockedBy: 'si sblocca con'`, `noResults`, `resetFilters`, `noFilters`, `activeFilters`, `reset`, `diagnostics.{noCatalogTitle, noCatalog, noCollectionSectionTitle, noCollectionSection, noAchievementSectionTitle, noAchievementSection, itemsBeyondSlots}`; `placeholder.collection` goes.

- [x] **Step 5: Parts**
  - `collectionLabels.ts`: `collectionFacetTitle: Record<CollectionFacet, MessageKey>`, `itemStateText: Record<ItemState, MessageKey>`, `collectionFacetValueLabel(t, facet, value)` — state through `itemStateText`, quality `unrated` through `collection.quality.unrated` else the number, pool `none` through `collection.pool.none` else the name, kind through `unlockKindText` (`@/components/graph/unlockKindText`), origin as `facetLabels.ts` does (the DLC names, `graph.originNone`).
  - `CollectionStateToggle.vue`: Unlock's `StateToggle` over `itemStateOrder`, squares `bg-state-done`, `bg-state-now`, `bg-state-blocked`, `hatch-unknown border border-dashed border-state-unknown`.
  - `CollectionFacetDrawer.vue`: Unlock's `FacetDrawer` over Quality, Pool, Kind, Origin (`grid-cols-4`), with `collectionFacetCounts` and `collectionFacetOptions(view.pools, facet)`.
  - `CollectionToolbar.vue`: Unlock's toolbar with `collection.*` texts and `CollectionSort`.
  - `QualityPips.vue`: prop `quality: number | null`; four `size-quality-pip` squares, the first `quality` in `bg-foreground-soft`, the rest `bg-hairline`, then the number in `text-label tabular-nums`; "—" in `text-subtle-foreground` when null.
  - `CollectionRow.vue`: prop `item`; cells — `PixelSprite :url="item.iconUrl" placeholder class="size-8"`; name `text-row truncate` over `text-micro text-faint-foreground` "id N · kind"; `QualityPips`; first pool and "+N" (or `EmptyValue` "nessun pool"); origin name or "—"; the state `Badge` (`Done`, `Now`, `Blocked`, `Unknown` by `itemState`) inside a `Tooltip` whose content, for a locked or unlocked item, is "si sblocca con «text»" (the achievement's text, or "achievement N").
  - `CollectionTable.vue`: Unlock's `UnlockTable` with `grid-cols-collection`, `collectionRowHeight`, `CollectionRow`, keyed by `item.id`.
  - `CollectionDiagnostics.vue`: `noCatalog`, `noCollectionSection`, `noAchievementSection` as `Alert`s; `itemsBeyondSlots` as a line.

- [x] **Step 6: Screen** — `CollectionScreen.vue`: `useOnActiveProfile(() => store.load())`; `filter = ref(defaultCollectionFilter())`, `sort = ref(CollectionSort.Quality)`; `counts = itemStateCounts(items)`; `rows = sortItems(items.filter(matchesCollectionFilter), sort)`; header (`LayersIcon`, `routes.collection`, `collection.intro`), diagnostics, state toggle, drawer, a `Card` with toolbar and table or the no-results state with "Azzera i filtri" (which returns to `emptyCollectionFilter()`), `ProfileError`, `Skeleton`s. `routes.ts` maps `RouteName.Collection`; `routeTable.ts` drops its `routeArrives` entry.

- [x] **Step 7: Verify** — typecheck, lint, format, scan, `ui:test` by exit code. Commit `feat(ui): the Collection, what this save's collection doesn't hold` and push.

### Task 6: Looked at, handed on, checked

- [x] **Step 1: visual check** — `pnpm ui:dev` and a throwaway headless-Chrome script in the job's temp folder: the state counts and the default pick (to find + locked), the synthetic warning in the console, a facet's counts moving as another is picked, quality pips, a locked item's tooltip naming its achievement, the search, scrolling to the last row, `?collection=unread` (every state unreadable and the alert), `?catalog=none`, `?art=none`. Findings fixed with a test where they are logic, recorded in the spec.
- [x] **Step 2: documents** — `DESIGN-BRIEF.md` §7.7 (the Collection's contract, handed on) and §3's row 5; `docs/STATUS.md` (3.4 ticked, the session log, the pack waiting for `pnpm design:export` to carry `collection.json`); `docs/frontend-conventions.md` (`lib/collection/`, `screens/collection/`, the fixture's declared synthetic source); `CLAUDE.md` (the `ipc` row and the State paragraph); the spec's deviations; this plan's checkboxes.
- [x] **Step 3: production build** — no fixture, pack payload or image in `ui/dist`.
- [ ] **Step 4: `sh scripts/check`** → "all green". Commit `docs: cycle 3.4 lands, the Collection` and push; merge `feature/screens-collection` into `develop` with `--no-ff` ("merge: screens 3.4, the Collection, into develop") through a temporary worktree, push `develop`.
