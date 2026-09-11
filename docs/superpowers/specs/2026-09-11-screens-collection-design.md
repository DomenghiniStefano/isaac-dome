# Design system, cycle 3.4 — Collection (design)

**Date:** 2026-09-11
**Milestone:** design system (`docs/STATUS.md`), cycle 3 of 3, sub-project 4 of 7
**Branch:** `feature/screens-collection`, cut from `develop` (the one-branch-per-sub-project rule)
**Depends on:** 3.3a (the node, Unlock's facets and virtualized table); `DESIGN-BRIEF.md` §3
(screen 5), §5.5 (quality, pools, origin), §6's "Quality and pool" facet; `Schermate.dc.html`
of the Claude Design export (Collezione, lines 998–1046; `COLLECTION` and `Q_META`, lines
1442–1465); the catalog's spec (`2026-09-03-catalog-design.md`, the section 4 cross-check)
**Status:** every decision below was taken by the author on the owner's delegation
("procedi"). Each is marked **(delegated)** so the first look can overturn it cheaply.

## What this sub-project is

The screen that answers "what haven't I found yet": the collectibles the active save's item
collection doesn't hold, by quality and pool. Unlike 3.3, **there is no command to draw it
from**: the save's section 4 and the catalog's items have never been joined on the IPC. So
this sub-project brings a contract, a command, a design-pack payload and a screen.

## Applicable constraints

1. **A section's meaning is structural or measured, never guessed** (CLAUDE.md). Section 4 is
   the "item collection": 733 one-byte slots, one per collectible id, the catalog's ids a subset
   of the slots with the gaps exactly the unused ids (catalog spec, cross-check). What a set byte
   means in play — picked up, or merely seen — is not measured; the contract calls it **in the
   collection**, the file's own name, and the screen says "in collezione".
2. **Trinkets are not in section 4**: the 733 slots are collectible ids. A trinket has no
   collection state to show, so the Collection lists collectibles only (passive, active,
   familiar) and says so.
3. **Degrade, never fail**: no catalog, section 4 unread, section 1 unread, a catalog newer than
   the save — each is a state on screen, from the view's diagnostics. **Unread is not "not in
   the collection"**: a missing section gives `null`, never `false`.
4. **No hardcoded counts**: the slots are read from the file, the items from the catalog.
5. **No game asset in the package**: icons are `isaac://item/<kind>/<id>` links, as on Unlock.
6. **The IPC contract is live**: the new types are handed on in `DESIGN-BRIEF.md` (§7.7) and
   exported to the design pack as a real payload.

## Decision 1 — the contract (delegated)

A new module `crates/ipc/src/collection.rs`, mirrored in `ui/src/lib/ipc/types.ts`:

```ts
interface CollectionView {
  items: CollectionItem[]      // collectibles only, by id
  pools: string[]              // the pools any listed item belongs to, in the catalog's order
  totals: { slots: number; items: number; inCollection: number }
  diagnostics: CollectionDiagnostic[]
}

interface CollectionItem {
  id: number
  kind: ItemKindView           // 'passive' | 'active' | 'familiar', never 'trinket'
  name: string                 // English, as everywhere a game name is data
  iconUrl: string | null       // isaac://item/<kind>/<id>
  quality: number | null       // items_metadata.xml; null when the file doesn't rate it
  pools: string[]              // itempools.xml names, untranslated: game jargon
  origin: OriginView | null
  inCollection: boolean | null // null: section 4 unread, or no slot for this id
  lock: LockView
}

// What stands between the item and a run. Tagged: three variants carry data.
type LockView =
  | { kind: 'free' }                                                // nothing unlocks it
  | { kind: 'unlocked'; achievement: number; text: string | null }  // its achievement is done
  | { kind: 'locked'; achievement: number; text: string | null }    // not done: it can't appear
  | { kind: 'unknown'; achievement: number; text: string | null }   // section 1 unread

type CollectionDiagnostic =
  | { kind: 'noCatalog' }                         // no names, no items: only the totals
  | { kind: 'noCollectionSection' }               // inCollection is null on every item
  | { kind: 'noAchievementSection' }              // every lock with an achievement is unknown
  | { kind: 'itemsBeyondSlots'; count: number }   // collectibles newer than the save
```

- `collection_view(catalog, items: Option<&[bool]>, achievements: Option<&[bool]>, icon)` —
  pure, in `ipc`, the flags as `Save::flags` gives them. Without a catalog: no items, `totals`
  with the slots and the set bytes, `noCatalog`.
- `totals.inCollection` counts the catalog's collectibles whose slot is set; `totals.items`
  the collectibles listed; `totals.slots` the section's length (0 when unread).
- **A new command `collection`** in `crates/app`, wired like `unlock`: the active save's two
  flag sections, the catalog from `CatalogState`, `icon_url`. `Err` only for no active profile.
- **`design-export` writes `contracts/payload/collection.json`** from the same function, so the
  next pack carries the real answer.

## Decision 2 — an item's state (delegated)

One pure function, `itemState(item)`, like 3.3a's `nodeState`:

| state | when | badge | label |
|---|---|---|---|
| `InCollection` | `inCollection` true | `Done` (tick) | "in collezione" |
| `Available` | false, and the lock is `free` or `unlocked` | `Now` (star) | "da trovare" |
| `Locked` | false, and the lock is `locked` | `Blocked` (lock) | "bloccato" |
| `Unknown` | `inCollection` null, or false with an `unknown` lock | `Unknown` ("?") | "non leggibile" |

Gold is right for `Available`: it is the Collection's "can be done tonight". A locked item's
tooltip names its achievement ("si sblocca con «…»").

## Decision 3 — the screen (delegated)

From `Schermate.dc.html`, lines 998–1046, laid out like Unlock rather than as the export's card
grid: the export itself notes that "at full size they are N, virtualized — the same rule as
Unlock", and a table of 720 rows is what 3.3a already virtualizes.

- **Header**: title, the export's line — the collectibles this save's collection doesn't hold,
  by quality and pool, not a wall of icons; quality and pool are read from the game's files,
  not guessed from the name — and the note that trinkets have no collection state.
- **State toggle** (the 3.3a `ToggleGroup`): in collection, to find, locked, unreadable, with
  counts. **Default: to find and locked** — "what haven't I found".
- **Facet drawer**: quality (0–4 and "non valutato"), pool (the view's `pools` and "nessun
  pool"), kind (passive, active, familiar), origin DLC (the four and "non indicata"). Each count
  over the items that match every other facet and the search, as on Unlock.
- **Results card**: "N / M oggetti"; a search over the name; sort — **qualità** (highest first,
  then id; the default), **id**, **nome**; the active picks as removable chips.
- **The table**, virtualized at `row-wide`:

  | column | content |
  |---|---|
  | sprite | the item at 32px through `PixelSprite`, or the placeholder |
  | Oggetto | the name, and "id N · kind" under it |
  | Qualità | four pips filled up to the quality, and the number; "—" when unrated |
  | Pool | the first pool, and "+N" |
  | DLC | the origin, or "—" |
  | Stato | the state badge; the locked tooltip names the achievement |

- **No results**: `EmptyCategory` and "Azzera i filtri".
- **Diagnostics** above the results: `noCatalog`, `noCollectionSection`, `noAchievementSection`
  as `Alert`s, each saying in words what the missing piece means; `itemsBeyondSlots` as one line
  under the table.

New tokens: the quality pip's side and the table's fixed columns; `grid-cols-collection`
composes them, like `grid-cols-unlock`.

## Decision 4 — state and pure logic

- **`stores/collection.ts`** (Pinia): `view`, status, error; `load()` on the active profile, as
  `stores/completion.ts` does.
- **`lib/collection/`**, pure, tested first:
  - `itemState(item)`, `stateCounts(items)`;
  - `CollectionFacet` (State, Quality, Pool, Kind, Origin), `facetValues`, `matchesFilter`,
    `facetCounts` (excluding its own facet), `facetOptions`, `sortItems`, the search.
  Unlock's filter functions are not generalized: two screens, two small modules, until a third
  one shows what is really shared.

## Decision 5 — fixtures (delegated)

The design pack has no `collection.json` yet — it can only be written on a machine with the game
and a save, which this one isn't. So `lib/ipc/fixtures/collection.ts`:

- **reads `contracts/payload/collection.json` when the pack has it**, through the same glob as
  the other payloads, and answers it as it is;
- **until then derives the items from the pack**: ids, kinds, names and sprites from
  `images/INDEX.json` (the `item` family, trinkets left out); **locks from `unlock.json`**, real —
  an item some node unlocks is `unlocked` or `locked` by that node's `done`, and `free`
  otherwise; **quality, pools and `inCollection` synthetic and deterministic**, from the id. The
  module exports which source it used, its test pins it, and the development server logs once
  that the Collection's quality, pools and collection flags are synthetic.
- `?collection=unread` answers section 4 unread (every `inCollection` null,
  `noCollectionSection`); `?catalog=none` answers without a catalog.

## i18n

`collection.*`: the header, states, facets and their values, sort, columns, diagnostics. Item
names and pool names are data, in English.

## Testing

Rust, test-first, on a synthetic catalog built like `crates/ipc/tests/queue.rs`'s:

- **The JSON shapes are pinned**: an item, each `LockView` variant, each diagnostic.
- **Trinkets are left out**; items come out by id.
- **`inCollection`** true and false from the slot; `null` for an id past the slots, counted in
  `itemsBeyondSlots`; `null` everywhere with `noCollectionSection` when section 4 is unread.
- **Locks**: no achievement is `free`; done is `unlocked`; not done is `locked`; section 1
  unread is `unknown` with `noAchievementSection`.
- **Quality, pools, origin** from the catalog; `pools` lists each pool once, in catalog order.
- **No catalog**: no items, totals from the section, `noCatalog`.
- **On real data** (skips here, runs where the game is): every catalog collectible has a
  non-null `inCollection`, and `totals.inCollection` equals the set bytes at the catalog's ids.

Vitest, test-first:

- **`itemState`** — each row of Decision 2's table; **`stateCounts`**.
- **Facets** — values for an unrated item, an item in no pool, a `null` origin; any within a
  facet, all across facets; counts excluding their own facet; the search; the three sorts.
- **The fixture** — the synthetic source is declared; ids, kinds and names match
  `images/INDEX.json`; no trinket; an item a done node unlocks is `unlocked`, one a node not done
  unlocks is `locked`; `?collection=unread` answers `null` and the diagnostic.

Visual checks on the development server: the four state counts and the default pick, a facet's
counts moving as another is picked, quality pips, a locked item's tooltip, the search, the
virtualized scroll to the last row, `?collection=unread`, `?catalog=none`, `?art=none`.

## Out of scope for this sub-project

- **Trinkets** — no collection state in the save.
- **Adding a locked item's achievement to the Plan's queue** from its row: one call away, left
  for when the Collection has been looked at.
- **An item's own page** and opening its wiki page (3.5).
- **Tags** from `items_metadata.xml` as a facet.
