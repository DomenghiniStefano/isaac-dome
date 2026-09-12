# Design system, cycle 3.5 — Wiki in tabs, and search (design)

**Date:** 2026-09-12
**Milestone:** design system (`docs/STATUS.md`), cycle 3 of 3, sub-project 5 of 7
**Branch:** `feature/screens-wiki-search`, cut from `develop` (the one-branch-per-sub-project
rule); merged into `develop` once per half, as 3.3a and 3.3b were
**Depends on:** 3.1 (tabs own locations, the sidebar's wiki categories, the navbar's search
trigger that opens nothing yet); cycle 2's `WikiBlocks` / `WikiInline` and the `Command`
primitive; `DESIGN-BRIEF.md` §4.2 (the shell: tabs and search), §5.7 (every page can have its
image), §8 (the wiki detail view); `docs/BACKLOG.md` B5 (global search) and B12 item 4 (the
`Command` filter and async results); `Schermate.dc.html` of the Claude Design export (the Wiki
screen, lines 813–945; the search overlay, lines 1087–1130; `searchIndex()`, lines 1510–1525)
**Status:** every decision below was taken by the author on the owner's delegation ("prosegui
coi lavori", the standing "do as much as you can, we look at it on first launch"). Each is
marked **(delegated)** so the first look can overturn it cheaply.

## What this sub-project is

Two halves, one branch, two merges:

- **A — the Wiki in tabs.** The Wiki route stops being a placeholder. A tab can be a category
  list (the six the sidebar already names) or **one page**: the entity's figure, its infobox
  and its text sections, every reference a link that replaces the page or opens a new tab.
  It is the only part of the app that needs neither the game nor a save, and it says so.
- **B — search.** One index over everything the app knows by name or by text — catalog names,
  achievement conditions, wiki titles, the body of wiki sections — queried by a command,
  shown in the `Ctrl+K` palette from any tab and, in full, on a Search screen. A result is a
  `Target` plus why it matched, and opening it is the same action as following a wiki link.

A is first because B's results open A's pages. Neither adds a table the profile depends on:
the profile only *ranks* B's results ("not done" before "done") and never gates either screen.

## Applicable constraints

1. **Only resolved view-models cross the IPC.** A page is identified by its wiki `Target`,
   the type the dataset already indexes by; no file names, no dataset paths. Icons are
   `isaac://` links served by the icon protocol, as everywhere since C2.
2. **Degrade, never fail.** No game: the Wiki keeps every page, names come from page titles,
   there are no icons at all. No profile: search has no "done / not done" and says so. No
   dataset (a broken binary, the contract still accounts for it): the Wiki says the dataset
   is missing, search runs on catalog names alone.
3. **The dataset is read-only and embedded.** Nothing here talks to the network; the
   attribution line (`wiki.gg · CC BY-SA 4.0`, `dataset/ATTRIBUTION.md`) is on every page.
4. **A tab saves identity, never content** (B6). A page tab holds the page's key, not its
   text; its label is derived when the index is known, and falls back to its category.
5. **In-game names stay in English** (§12); screens, categories, section names and actions
   are messages. The same query hits both, and every result says which section it opens.
6. **The frontend rules** hold; the `Command` primitive gains a prop, not a bypass.
7. **The IPC contract is live**: the new types are handed on in `DESIGN-BRIEF.md` (§8 gains
   the index and the search view) and `design-export` writes the search payload the day it
   runs on a machine with the game.

## Decision 1 — a page is a location (delegated)

`TabLocation.query` grows two optional fields:

```ts
interface TabLocation {
  name: RouteName
  query?: { category?: WikiCategory; page?: string; q?: string }
}
```

- **`page` is a page key**: the wiki `Target` written as one string, the way the design pack
  names its sample files and the dataset keys its bosses — `item:105`, `trinket:97`,
  `achievement:1`, `challenge:19`, `character:0`, `entity:20.0.0`. Two pure functions in
  `lib/wiki/pageKey.ts`, `pageKey(target)` and `parsePageKey(key): Target | null`, tested as
  a round trip over every kind that has a page. Stages, rooms, pickups and transformations
  have no page (`Dataset::entry` returns `None` by construction), so they have no key and
  `parsePageKey` never produces them. An unparseable key is **a page the dataset doesn't
  know**, the same state as a valid key with no entry — never a crash.
- **A page location always carries its category**, `categoryOf(target)` (item → items,
  trinket → trinkets, achievement → achievements, entity → bosses, challenge → challenges,
  character → characters), so the sidebar entry of a page tab stays lit through the existing
  `isEntryActive`, unchanged.
- **The label.** `locationTitle` stays what it is (the category); `tabLabel(location,
  titleOf)` in `stores/tabModel.ts` returns the page's title when `titleOf(key)` knows it and
  the category otherwise. `App.vue` feeds it the wiki store's index, so a page tab reads
  "Brimstone" the moment the index has loaded and "Oggetti" until then. Nothing about the
  page is stored in the tab.
- **`q`** is B's: the query a Search result hands to Unlock or Collection (Decision 8).

The rule "the label and origin icon are derived from the route, never stored" (3.1, Decision
1) holds: derived from the route *and* the index, which is state the app has anyway.

## Decision 2 — the wiki index, one command (delegated)

A new command, **`wiki_index`**, answers once per window with every page the dataset has:

```ts
interface WikiIndex {
  info: WikiInfo                  // §8: loaded (counts, snapshot, patch) or missing
  pages: WikiPageRef[]            // empty when missing
}
interface WikiPageRef {
  target: Target                  // the page's identity, and what wikiEntry accepts
  title: string                   // the page's own title: the English name
  iconUrl: string | null          // null: no catalog, or the game draws nothing for it
}
```

- `ipc::wiki_index(dataset, catalog: Option<&Catalog>, game_updated_unix, icon)` — pure, in
  `crates/ipc/src/wiki.rs` beside `wiki_info`. Pages come out in the dataset's order (by
  kind, then by id), which is the order the category lists show by default.
- **`IconRef::Page { target: Target }`** is the one new variant of the icon reference:
  `page/item/105`, `page/trinket/97`, `page/achievement/1`, `page/challenge/19`,
  `page/character/0`, `page/entity/20/0/0`. `icon_source` resolves it through
  `target_sprite`, `Found` or nothing. `to_path` is exhaustive over `Target` (a target with
  no page has no path, and `parse` refuses it), so a new wiki kind breaks the build here.
  The URL is emitted only when the catalog resolves the sprite: `iconUrl: null` is "no
  picture", and the list draws the placeholder instead of a broken image.
- **Why one index and not a command per category**: 1,727 refs weigh about 200 KB, less
  than one `unlock`; the same map answers the tab labels (Decision 1), the icon of every
  reference inside a page (Decision 4), and "does this target have a page" — three
  questions one load settles.
- The wiki store (`stores/wiki.ts`) loads it on first need and keeps it: the dataset can't
  change while the app runs. `wikiUnavailable` is the store's `Failed` state with a retry.

## Decision 3 — the Wiki screen, three views by query (delegated)

One route, `Wiki`, one screen, three views decided by the location's query:

| query | view |
|---|---|
| none | **the landing**: what the Wiki is, the dataset's provenance, six category cards with their counts |
| `category` | **the list**: every page of the category, a text filter, virtualized |
| `page` (with its category) | **the page** (Decision 4) |

- **The landing** (`/wiki`, reached from the navbar's Wiki section or a bare Wiki tab) says
  what `Schermate.dc.html` says: the dataset is compiled into the binary and works without
  the game and without a save. Then the provenance from `WikiInfo`: snapshot date, last known
  patch, `wiki.gg · CC BY-SA 4.0`, and `gameNewerThanSnapshot` as a note when true ("the game
  is newer than this snapshot: recent changes may be missing"). Six cards, one per category,
  with the count from `info.counts`, each a link to its list. When `info` is `missing`: an
  alert with the reason, no cards.
- **The list**: `ScreenHeader` with the category's icon and title, the eyebrow "N pagine";
  a text field that filters titles (case-insensitive contains, the same `matchesQuery` the
  Unlock filter uses); a virtualized table (`@tanstack/vue-virtual`, as Unlock and the
  Collection) whose row is the icon, the title, and the id in `text-caption`. A click
  navigates the tab to the page; `Ctrl`+click opens it in a new tab (the shell's gesture,
  3.1 Decision 1). Without the catalog every icon is the placeholder and one line above the
  table says why. Empty filter result: the `EmptyCategory` state and a reset.
- **The sidebar's six entries** already point at `{ name: Wiki, query: { category } }`;
  nothing changes there. The navbar's Wiki button keeps switching the sidebar only.

## Decision 4 — the page (delegated)

Reached by `?category=…&page=…`. The wiki store reads `wikiEntry(target)`, one entry per
page key, cached for the window's life (a page never changes at runtime).

**Header**, from `Schermate.dc.html`: the figure on the left, then the title in `text-title`,
and a line with the kind badge (`Oggetto`, `Trinket`, `Achievement`, `Boss`, `Sfida`,
`Personaggio` — the category's singular, a message keyed on the target's kind), `rev.
<revid>` in tabular figures, and the attribution line with the license (a tooltip carries
the full sentence). The figure is a new `WikiFigure` component: the `iconUrl` from the index,
drawn by kind — an item or trinket sprite scaled with `image-rendering: pixelated`
(`PixelSprite`), an achievement in `AchievementArt`'s paper frame (its ratio isn't square),
a boss portrait or character portrait in a square frame; null draws the hatch placeholder,
which the brief distinguishes from a broken image.

**The infobox**, a card under the header, one layout per `Infobox.kind` — exhaustive, so a
new kind breaks the build:

| kind | rows |
|---|---|
| `item`, `trinket` | none: the card isn't drawn |
| `achievement` | description; requirements (inline); unlocks (a ref chip, or "—") |
| `boss` | base HP (or "—"); environment; pool; unlocked by |
| `challenge` | goal; items; trinkets; pickups; health; curse; three flags as badges (blindfolded, shops, treasure rooms); unlocks; unlocked by |
| `character` | health; damage, range, speed, luck, shot speed as a stat strip; pickups; collectibles; unlocked by |

Rows are label + `WikiInline`; a `Target` field becomes a one-ref inline so it links like
any other. Empty inline arrays draw "—", never an empty row.

**The sections**, in the dataset's order, each a heading with the section's message
(`SectionKind` → `wiki.section.*`, the twelve of §8) and its `WikiBlocks`. A page with no
sections shows the dashed note of the export ("this page has no text sections in the
dataset").

**References.** `WikiInline` and `WikiBlocks` emit `navigate: [target, newTab: boolean]`,
the modifier read on the click; the page navigates the tab or opens one. Two resolvers
travel down: `iconFor(target)` (the index's `iconUrl`) and **`canOpen(target)`** — a ref
whose target has no page in the index (a stage, a room, a pickup, or an entity the dataset
lacks) is drawn like a `concept`, dotted and not clickable, instead of leading to a page
that says "unknown". `WikiInline` gains the optional `canOpen` prop; without it every ref
opens, as on the Kit page.

**States**: loading (skeletons for header and two sections); **unknown page** (`wikiEntry`
returns `null`, or the key doesn't parse): the header with the key as title, the
`EmptyCategory` state "Il dataset non conosce questa pagina" and a link back to the
category; **dataset missing** (`wikiUnavailable`): the alert with retry. Changing the
active profile does nothing here: a Wiki tab doesn't depend on it (§4.2).

## Decision 5 — the search index (delegated)

`crates/ipc/src/search.rs`, pure. Two sources, joined per target:

- **The wiki side**, built once from the embedded dataset and cached in a managed
  `SearchState` (the dataset never changes): for every page, its title and the text of each
  section flattened to one string per section — every `text` and every `ref` / `concept`
  label, in order, with headings and table cells included, `edition` inlines unwrapped.
- **The catalog side**, read live at every query (2,000 names, cheap; and "the game isn't
  installed" is never cached): items and trinkets by English name, characters, bosses,
  challenges, achievements with their `text` and `unlock_condition`. A target both sides
  know is **one document**: the catalog's name is its title, the wiki title an alias when
  it differs.

**The query.** Words split on whitespace, matched case-insensitively (ASCII fold, so byte
offsets stay aligned for fragments; a non-ASCII letter matches only itself); **every word
must occur in the same field**. Fields are tried in order and the first that matches names
the hit's `match`: title (or alias), condition, then each section. **One hit per target.**

```ts
interface SearchView {
  query: string
  hits: SearchHit[]              // ranked, at most `limit`
  total: number                  // how many matched before the limit
  diagnostics: SearchDiagnostic[]
}
interface SearchHit {
  target: Target
  title: string
  iconUrl: string | null
  hasPage: boolean               // the dataset has this page: a Wiki destination exists
  match: SearchMatch
  progress: ProgressMark         // 'done' | 'pending' | 'unknown' | 'none'
}
type SearchMatch =
  | { kind: 'title' }
  | { kind: 'condition'; text: string }                 // the achievement's own wording
  | { kind: 'section'; section: SectionKind; before: string; matched: string; after: string }
type SearchDiagnostic =
  | { kind: 'noProfile' }                // no done / not done: every mark is unknown
  | { kind: 'noCatalog' }                // wiki titles only, no icons, no conditions
  | { kind: 'noWiki' }                   // catalog names only, no page to open
  | { kind: 'noAchievementSection' }     // achievements' marks unknown
  | { kind: 'noCollectionSection' }      // items' marks unknown
```

- `ProgressMark` is fieldless, a bare string, `const … as const` in TypeScript. For an
  achievement: `done` / `pending` from section 1; for a collectible: in the collection /
  to find from section 4 (the structural reading, 3.4); `unknown` when there is no profile or
  the section didn't read (the diagnostic says which); `none` for every other kind — a boss
  or a character has no slot to read.
- A section fragment is three strings — the ~60 characters before the match, the matched
  text as it appears on the page, the ~90 after — cut on character boundaries so the
  palette highlights without counting bytes.
- **The command** `search(query: String, limit: usize) -> SearchView` in `crates/app`, with
  the active save's two flag sections when there is a profile (no profile is a diagnostic,
  not an `Err`). An empty query answers no hits and no diagnostics.
- **Measured, not assumed** (B5): `crates/ipc/tests/search_real.rs` builds the wiki side
  from the embedded dataset, prints the build time and the time of three queries on stderr,
  and pins nothing about time — the report records the numbers. If a scan turns out to be
  slow enough to feel, the fallback named in B5 (FTS5 in `store`) is a later migration;
  nothing in the contract above changes.

## Decision 6 — ranking (delegated)

Tiers first, then the profile, then the name:

| tier | match |
|---|---|
| 0 | title equals the query |
| 1 | title starts with the query |
| 2 | a word of the title starts with the query |
| 3 | title contains every word |
| 4 | condition contains every word |
| 5 | a section contains every word |

Within a tier, `pending` before `unknown` and `none`, `done` last — "not done before done"
is what makes the profile ranking information (B5). Then the title, then the target's kind
order and id, so the order is total and the tests can pin it. The frontend never re-sorts:
the palette shows the backend's order, cut per group.

## Decision 7 — the palette (delegated)

`components/search/SearchPalette.vue`, mounted once in `App.vue`, opened by the navbar's
trigger and by **`Ctrl+K`** from anywhere (a `useShortcut` composable on `window`, `keydown`,
`preventDefault`; `EventKey` gains `K`). The `CommandDialog` with the export's placeholder
("Cerca schermate, achievement, oggetti, pagine wiki…"), `Esc` closes.

- **Rows are destinations, not hits.** A pure `searchRows(hits, screens)` turns each hit into
  the rows it can open: a **Wiki** row when `hasPage`; an **Unlock** row for an achievement;
  a **Collezione** row for an item; and, from the frontend alone, a **Schermate** row for
  every route or wiki category whose message contains the query. Groups in that order —
  Schermate, Wiki, Unlock, Collezione — five rows each in the palette, and a last row
  "Tutti i risultati (N)" that opens the Search screen with the query. Every row states its
  section, because "Brimstone" is a page *and* a Collection row (§4.2).
- A row draws the icon (or the group's icon for a screen), the title, a second line — the
  fragment with the match emphasised, or the condition, or the section for a wiki-text hit —
  and the progress mark as a small badge when it isn't `none`.
- `Enter` or a click navigates the active tab to the row's location; **`Ctrl+Enter` or
  `Ctrl`+click opens a new tab**. The palette closes on either.
- **The query is debounced** (`Timing.SearchDebounce`, a motion token) and answers are
  numbered: an answer to an older query than the one typed is dropped, so a slow scan can't
  overwrite a fresh result.
- **The `Command` primitive gains `filter: boolean`, default `true`.** With `false` it
  registers items and groups but scores nothing: every mounted item shows, which is what a
  list the backend already filtered needs. This is the one change B12 item 4 asks for that
  the palette can't do without; the rest of that item (the filter's tests, `autoFocus` as a
  prop, pruning group ids on unmount) stays in B12.

## Decision 8 — the Search screen (delegated)

Route **`Search`**, path `/search`, query `q`; **`TabOrigin.Search`** with the search icon
on the tab, so the mixed bar says what it is. `sectionOfOrigin` returns `null` for it and
`App.vue` keeps whatever section the sidebar was showing: search sits above the two sections
(§4.2), it belongs to neither. It needs no profile.

- `ScreenHeader`, then the query field (`Input`), bound to the location: typing navigates
  the tab (debounced) so the tab *is* the search and survives as one. Then a toggle group
  over the destinations — Wiki, Unlock, Collezione — with counts; the diagnostics as alerts
  (`noProfile` as a note, not a warning: it's the expected state before choosing a save).
- The rows are the palette's rows without the per-group cap, virtualized past sixty, with
  the same click gestures. The command is called with `limit: 300`; when `total` exceeds it
  a line says "mostrati 300 di N: affina la ricerca".
- The palette's "Tutti i risultati" opens this screen in the active tab.

**Unlock and Collection read `q`.** Both screens start their text filter from the
location's `q` when present (a `watch` on the route's query, immediate), so a Search row
opens the list already filtered — B3's "the two meet at the point where a global result
opens the list already filtered". Nothing else changes on those screens.

## Decision 9 — how it degrades

| missing | Wiki | Search |
|---|---|---|
| profile | nothing changes | every mark `unknown`, `noProfile` |
| game (no catalog) | every page, names from titles, no icons, a line on the list | wiki titles and text only; no icons, no conditions; **no Unlock or Collection rows** (those lists have no names to filter by), `noCatalog` |
| dataset | the landing's alert, no lists, no pages | catalog names only, `hasPage` false everywhere, `noWiki` |
| section 1 / section 4 | — | marks `unknown` for that kind, its diagnostic |

## Decision 10 — development without the backend (delegated)

- **The index fixture** is real where the pack is real: pages from `images/INDEX.json`
  (items and trinkets by family and kind, achievements, bosses by their portrait's entity key
  in `source`, characters) plus the eleven sample pages of `wiki/samples/`; titles are the
  pack's names; challenges come from `unlock.json`'s targets. `?wiki=none` answers `info:
  missing`.
- **`wikiEntry`** answers the eleven samples with their real `Entry` and `null` for every
  other key; a console line, once, says the pack carries eleven pages.
- **`search`** scans the fixture index's titles and the eleven samples' sections with the
  tiers of Decision 6, in a small pure module the fixture owns; conditions come from
  `unlock.json`. Declared synthetic in the console like the Collection's flags: the ranking
  that counts is Rust's, and this only lets the palette be looked at.
- **`design-export`** writes `contracts/payload/wiki_index.json` and `search.json` (three
  queries) the day it runs with the game; until then the pack has none and the fixture says
  so.

## i18n

New keys, Italian first, English mirrored: `wiki.*` (intro, provenance lines, kind labels,
`section.*` for the twelve kinds, the infobox labels, the states), `search.*` (placeholder,
groups, "all results", the diagnostics, the limit line), `routes.search`, `shell.searchHint`.
The twelve section names are the export's `SEC_LABEL`; the DLC names already exist
(`components/wiki/dlcNames.ts`).

## Testing

**Rust** (test-first, values from this spec):

- `crates/ipc/tests/wiki_index.rs`: the JSON shape pinned (`target` tagged, `iconUrl` null
  without a catalog); the count per kind equals `info.counts`; the order is by kind then id;
  `IconRef::Page` round-trips for every kind that has a page and refuses `page/stage/…`;
  `icon_source` on a synthetic catalog resolves an item page and answers nothing for a
  challenge without a rewarding achievement.
- `crates/ipc/tests/search.rs`, on a hand-built dataset (`Dataset::empty_for_tests` plus
  entries) and the synthetic catalog the Collection tests use: the shapes pinned; one hit
  per target with the best field; the six tiers in order; every word in the same field;
  the fragment's three parts around the first occurrence; `progress` from the flags and
  `unknown` without them; `limit` and `total`; the five diagnostics; an empty query.
- `crates/ipc/tests/search_real.rs`: on the embedded dataset, "brimstone" ranks the item's
  page first; the timings on stderr.
- `crates/ipc/tests/unlock_size.rs` gains the index's ceiling (256 KB on the embedded
  dataset without icons).

**TypeScript** (Vitest): `pageKey` / `parsePageKey` round trip and refusals; `categoryOf`;
`tabLabel`; `searchRows` (destinations per kind, `hasPage`, the screens' rows, the per-group
cap); `rowLocation`; the palette's stale-answer guard as a pure `latest()` helper; the
`Command` primitive's `filter=false` (every item renders); the wiki list's filter.

**By eye**, on `pnpm ui:dev` through headless Chrome as 3.4 did: the landing, an items list
with 900 rows and its filter, The D6's page with its 26 references and the Monstro infobox,
a ref opening in place and in a new tab, `?catalog=none`, `?wiki=none`, the palette on
"brim" with its four groups, `Ctrl+Enter`, the Search screen with the toggles, an Unlock tab
opened already filtered.

## Files

**Rust:** `crates/ipc/src/wiki.rs` (`wiki_index`, `WikiIndex`, `WikiPageRef`),
`crates/ipc/src/icon.rs` (`IconRef::Page`), `crates/ipc/src/search.rs` (new),
`crates/ipc/src/lib.rs`, `crates/ipc/tests/{wiki_index,search,search_real,unlock_size}.rs`,
`crates/app/src/lib.rs` (`wiki_index`, `search`, `SearchState`),
`crates/design-export/src/payload.rs`.

**TypeScript:** `lib/ipc/types.ts`, `lib/ipc/wiki.ts`, `lib/ipc/search.ts` (new),
`lib/constants/commands.ts`, `lib/wiki/{pageKey,category,listFilter}.ts`,
`lib/search/{rows,latest}.ts`, `stores/{wiki,search}.ts`, `stores/tabModel.ts` (`tabLabel`),
`router/routeTable.ts` (`Search`, `q`, `page`), `router/routes.ts`,
`components/shell/{tabs,tabOriginIcon,sectionNav}.ts`, `components/ui/command/Command.vue`
(`filter`), `components/wiki/{WikiInline,WikiBlocks}.vue` (`newTab`, `canOpen`),
`components/wiki/WikiFigure.vue`, `components/search/{SearchPalette,SearchRow}.vue`,
`composables/useShortcut.ts`, `screens/WikiScreen.vue`, `screens/wiki/{WikiLanding,
WikiCategoryList,WikiPage,WikiInfobox,WikiSections}.vue`, `screens/SearchScreen.vue`,
`screens/search/{SearchToolbar,SearchResults}.vue`, `screens/{UnlockScreen,CollectionScreen}.vue`
(`q`), `App.vue`, `lib/ipc/fixtures/{index,wiki,search}.ts`, `i18n/messages/{it,en}.ts`.

## Out of scope for this sub-project

- Tabs that survive a restart (3.7): the page key is a location, ready to be saved, but
  nothing saves it.
- "Open on the wiki" for `concept` inlines and pages the dataset lacks (the 2026-09-05 wiki
  spec's out-of-scope, unchanged).
- A page's standing in the profile ("you have this", "done"): the search row's mark is the
  only place the two meet; the page itself is the wiki's, profile-free.
- Search over the plan queue, the run archive (M4) and options.
- The rest of B12 item 4.
