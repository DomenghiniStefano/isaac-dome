# Cycle 3.5b — search: report

**Date:** 2026-09-12
**Branch:** `feature/screens-search`, cut from `develop`
**Plan:** `docs/superpowers/plans/2026-09-12-screens-search.md` (14 tasks, all closed)
**Spec:** `docs/superpowers/specs/2026-09-12-screens-wiki-search-design.md`, Decisions 5–10

## What landed

One index over everything the app knows by name or by text, a command that queries it, and two
ways to ask: the `Ctrl+K` palette from any tab, and a Search screen that is itself a tab.

- **`crates/ipc/src/search.rs`**, pure. The wiki side is built once from the embedded dataset —
  each page becomes its title plus one flattened string per section, with headings, `ref` and
  `concept` labels, table cells and nested list blocks all read, and `edition` inlines
  unwrapped because the page shows their words. The catalog side is read at every query, never
  cached, because "the game isn't installed" is not an answer to keep.
- **A target both sides know is one document.** The catalog's name is the title — it is what
  the game itself calls the thing — and the wiki's title becomes an alias when it differs, so a
  name is searchable from either side and listed once.
- **Six tiers, then the profile, then the name.** Not done before done is what makes the
  profile part of the ranking (B5). Every word must be in the **same field**: "monstro spits"
  finds nothing, because no one field holds both. The order is total, so the tests pin it.
- **The command** `search(query, limit)`, with `SearchState` holding the wiki side for the
  window's life. No profile is a diagnostic inside the answer, never an `Err`.
- **The palette and the screen** turn each hit into the **destinations** it opens — Wiki,
  Unlock, Collezione — plus the Screens rows the frontend answers itself. Five per group in the
  palette, uncapped on the screen, virtualized past sixty.

## What was measured

`crates/ipc/tests/search_real.rs` prints, and pins nothing about time:

```
search: index of 1727 pages built in 15 ms
search: "brimstone"   → 222 hits of 222 in 19 ms
search: "the lost"    → 224 hits of 224 in 16 ms
search: "mom's heart" → 148 hits of 148 in 15 ms
```

A debug build, on this machine, over the whole embedded dataset and the installed game's
catalog. **B5's fallback is not needed**: an FTS5 table in `store` would buy nothing a 15 ms
scan doesn't already give, and nothing in the contract would change if it ever were. What the
test pins is the answer: "brimstone" ranks the item's own page first.

The wiki index's ceiling joined `unlock_size.rs`: 256 KB on the embedded dataset without icons.

## Three corrections to the spec, written back into it

1. **`SearchDiagnostic` is a bare string**, not a tagged object. The spec drew it tagged; the
   repo has zero tagged unit enums, and a second convention for the same thing is how a
   TypeScript `switch` falls into no branch.
2. **`Command` gains two props, not one.** `filter` alone can't carry the palette: the typed
   text has to leave the primitive to be debounced and sent to the backend, so `search` is a
   model too.
3. **The search state is a composable, not a store.** The palette and the Search screen ask
   different questions at the same time; one Pinia store would have each overwrite the other's
   answer.

## Four defects the eye found, all fixed at the root

- **A virtualized table didn't measure again when the interface changed size.** Cycle 3.5c
  unified the number the rows are positioned with, but nothing told the virtualizer to
  re-measure: change the scale with a table already open and the rows are drawn one height and
  spaced another — at 175% Unlock's rows were 70px tall and 80px apart, overlapping with
  nothing failing. **This was not new to search**: Unlock, the Collection and the Wiki's lists
  all had it. One composable now owns the pattern (`useScaledRows`), and the watch that
  re-measures is stated in a test of its own.
- **A palette row opened twice.** Reka replays the click on the item, so a click handler fires
  for both. The row opens on the list's own `select` now; `select` does not carry the modifier,
  so `Ctrl` is read from the window (`useGestureModifiers`), which is where the gesture lives.
- **A list opened on a name showed nothing.** The Collection defaults to "to find" and
  "locked"; an item asked for by name is often already in the collection, so clicking a row
  answered "0 of 721". A search result is a request and the default states are a convenience:
  `filterForQuery` drops them.
- **Every row carried the same badge.** With no profile every mark is `unknown`, so the badge
  said the same thing on every row while the diagnostic already said it once — the case B29
  removed from the Collection. The mark is drawn only when it tells this row apart.

Two smaller ones: the result rows were centred (a pressable primitive is centred by the user
agent), and the group toggles listed "Schermate 0".

## Seen by eye

Through Playwright on `pnpm ui:dev`: the palette on "brim" with its four groups and the
per-group cap; `Enter` navigating the active tab and `Ctrl`+click opening one beside it; the
Search screen with its toggles, counts and virtualized rows; a Collection row opening the list
at **1 / 721** on the item asked for; `?catalog=none` (Wiki rows only, no icons);
`?wiki=none` (catalog names only, nothing offering a page); `?fixture=none`'s `noProfile` note;
and the rows at 150% and 175% measuring 60/60 and 98/98 — height and spacing agreeing.

**Not seen in a real Tauri window**: the command against the real index, and the ranking on a
profile that actually has marks. The development server's fixture ranks synthetically and says
so in the console; the ranking that counts is Rust's, and it is the one the tests pin.

## Files

**Rust:** `crates/ipc/src/search.rs` (new), `crates/ipc/src/lib.rs`, `crates/ipc/src/wiki.rs`
and `crates/ipc/src/target_sprite.rs` (two helpers made crate-visible),
`crates/ipc/tests/{search,search_real,unlock_size}.rs`, `crates/app/src/lib.rs`,
`crates/design-export/src/payload.rs`.

**TypeScript:** `lib/ipc/{types,search}.ts`, `lib/constants/{commands,eventKeys,timing}.ts`,
`lib/search/{rows,latest,queryParam}.ts`, `lib/scale/rows.ts`,
`composables/{useSearch,useGestureModifiers,useScaledRows}.ts`,
`components/ui/command/{Command.vue,CommandDialog.vue,filter.ts}`,
`components/search/{SearchPalette,SearchRow}.vue`,
`screens/SearchScreen.vue`, `screens/search/{SearchToolbar,SearchResults,SearchDiagnostics}.vue`,
`screens/{UnlockScreen,CollectionScreen,WikiScreen}.vue`,
`screens/{unlock/UnlockTable,collection/CollectionTable,wiki/WikiCategoryList}.vue`,
`lib/collection/collectionFilter.ts`, `router/{routeTable,routes}.ts`,
`components/shell/{tabs,tabOriginIcon,sectionNav}.ts`, `App.vue`,
`lib/ipc/fixtures/{search,wiki,index}.ts`, `kit/sections/app/SearchRowSection.vue`,
`assets/theme/spacing.css`, `i18n/messages/{it,en}.ts`.

## Suite

`scripts/check` green: `cargo fmt --check`, `cargo clippy --all-targets -D warnings`,
`cargo test --workspace` (13 new tests in `search.rs`, 4 unit tests on the matcher, 1 timed
real-data test, 1 index ceiling), `pnpm typecheck`, `pnpm ui:test` (292), `pnpm lint`,
`pnpm format:check`, `pnpm scan` (0 violations). Skips on real data: 7, all named.
