# 3.7a — a tab owns its state: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:executing-plans` to implement
> this plan task by task. Steps use checkbox (`- [ ]`) syntax for tracking. This repo's plans are
> executed **inline**, not one-subagent-per-task.

**Goal:** a tab keeps how it was being read — facets, search text, sort, the selected row, the
scroll position — across a tear-off into another window and across a restart of the app.

**Architecture:** a tab's history entry stops being a bare location and becomes
`{ location, view? }`. The shell types `view` as `unknown` and knows nothing about any screen;
each screen declares a `TabViewSpec<T>` with a validator that can answer "no", and reaches it
through one composable. Because `TabSeed` is `Omit<Tab, 'id'>`, the record crosses to another
window with no line added; because the session document already stores entries, it is persisted
with no migration and no new command.

**Tech Stack:** Vue 3 + TypeScript, Pinia, Vitest, `@tanstack/vue-virtual`. Frontend only —
3.7a touches no Rust, no `crates/`, no IPC command and no SQL.

**Spec:** `docs/superpowers/specs/2026-09-15-tabs-session-design.md` (§3, §4, §7, §11 item 1).

## Global Constraints

Copied from `CLAUDE.md` and `docs/frontend-conventions.md`; every task's requirements include
them.

- **Test-first.** The expected value comes from the spec, never from the code's current output.
  A failing test is first of all a hypothesis of a bug in the code.
- **No `<style>` in SFCs**; dynamic values arrive as CSS variables bound by the template.
- **No hardcoded visual constants** — no `w-[48px]`, no `duration-150`, no `:size="16"`. Every
  value is a token in `@theme`. 3.7a **adds no token**: `--spacing-tab-min` (2.125rem),
  `--spacing-tab-max` (9.375rem) and `--container-tab-narrow` (64px) already exist.
- **No `invoke()` in components**; nothing outside `ui/src/lib/window/` imports
  `@tauri-apps/api`'s `window`, `webviewWindow` or `event`.
- **No raw `<button>` / `<input>`** — the primitives in `ui/src/components/ui/`.
- **No string unions** — `const X = { … } as const`, never `type X = 'a' | 'b'`.
- **No visible string in a template**: every sentence is an i18n key, in `it.ts` **and** `en.ts`.
- **Exhaustiveness**: no catch-all arm on a closed union; `assertNever` instead.
- `pnpm scan` enforces the five rules and has no exemption added by this plan.
- **Commits**: Conventional Commits, `type(scope): subject`, scope `ui`. No `Co-Authored-By`
  trailer, no reference to Claude, ever.
- **Before declaring anything done**: `pnpm check` (`scripts/check`) from the worktree root.

## File Structure

**Created**

| file | responsibility |
|---|---|
| `ui/src/lib/tabs/tabView.ts` | `TabViewSpec<T>`, and the two readers every screen's spec is built from (`readStringArray`, `readFacetFilter`). Pure, no Vue. |
| `ui/src/lib/tabs/tabView.test.ts` | its tests |
| `ui/src/composables/useTabView.ts` | the one way a screen reaches its record: read at mount, write on change, re-read when the entry under it changes |
| `ui/src/composables/useTabView.test.ts` | its tests |
| `ui/src/lib/runs/runKey.ts` | `runKey(run)` — a run is `(source, ordinal)`; today the expression lives inline in `RunsTable.vue` and Runs needs it twice once a selection is remembered |
| `ui/src/lib/runs/runKey.test.ts` | its tests |
| `ui/src/screens/unlock/tabView.ts` | Unlock's spec: filter + sort |
| `ui/src/screens/collection/tabView.ts` | the Collection's spec: filter + sort |
| `ui/src/screens/runs/tabView.ts` | Runs' spec: filter + the selected run's key |
| `ui/src/screens/search/tabView.ts` | Search's spec: the picked groups |
| `ui/src/lib/scale/scrollOffset.ts` | `ScrollOffset`, `readScrollOffset`, `offsetToApply` — an offset means nothing against a list of a different length |
| `ui/src/lib/scale/scrollOffset.test.ts` | its tests |

**Modified**

| file | change |
|---|---|
| `ui/src/stores/tabModel.ts` | `Entry`; every rule that builds or reads one; `entryView`, `setEntryView` |
| `ui/src/stores/tabModel.test.ts` | the new rules |
| `ui/src/stores/tabs.ts` | `fresh()`, and `setView` on the store |
| `ui/src/lib/window/sessionDocument.ts` | read an entry in both shapes; prune to the current entry on write |
| `ui/src/lib/window/sessionDocument.test.ts` | the new rules |
| `ui/src/components/ui/virtual/VirtualRows.vue` | an optional offset in, a change out |
| `ui/src/components/shell/TabStrip.vue` | the strip scrolls; the active tab is brought into view |
| `ui/src/screens/UnlockScreen.vue` | adopts `useTabView` |
| `ui/src/screens/CollectionScreen.vue` | adopts `useTabView` |
| `ui/src/screens/RunsScreen.vue` | adopts `useTabView`; the selection is a key |
| `ui/src/screens/SearchScreen.vue` | adopts `useTabView` |
| `ui/src/screens/runs/RunsTable.vue` | uses `runKey` instead of its inline expression |

---

### Task 1: a history entry is a view

**Files:**
- Modify: `ui/src/stores/tabModel.ts`
- Modify: `ui/src/stores/tabs.ts`
- Test: `ui/src/stores/tabModel.test.ts`

**Interfaces:**
- Consumes: nothing.
- Produces: `interface Entry { location: TabLocation; view?: unknown }`;
  `Tab.entries: Entry[]`; `tabLocation(tab: Tab): TabLocation` (unchanged signature);
  `entryView(tab: Tab): unknown`;
  `setEntryView(state: TabsState, location: TabLocation, view: unknown): TabsState`.

- [ ] **Step 1: Write the failing tests**

In `ui/src/stores/tabModel.test.ts`, change the helpers at the top of the file so an entry is a
view, then add the block below. The helper change is part of this step — the file will not
compile until Step 3, which is the point.

```ts
const at = (name: TabLocation['name']): TabLocation => ({ name })
const entry = (name: TabLocation['name']): Entry => ({ location: at(name) })
const one = (id: string, name: TabLocation['name']): Tab => ({
  id,
  entries: [entry(name)],
  index: 0,
})
```

```ts
describe('the view a tab is holding', () => {
  it('starts with no view at all', () => {
    const state = firstState('a', at(RouteName.Unlock))
    expect(entryView(state.tabs[0]!)).toBeUndefined()
  })

  it('writes the view into the entry the tab is showing', () => {
    const state = setEntryView(
      firstState('a', at(RouteName.Unlock)),
      at(RouteName.Unlock),
      { sort: 'name' },
    )
    expect(entryView(state.tabs[0]!)).toEqual({ sort: 'name' })
    // The location is untouched: a view is not a navigation.
    expect(tabLocation(state.tabs[0]!)).toEqual(at(RouteName.Unlock))
  })

  // The same guard `refineTab` has, and for the same reason: a debounced write can land after
  // the user has gone back or switched tab, and it must reach no tab rather than the wrong one.
  it('refuses a view whose location is not the one the tab is showing', () => {
    const state = firstState('a', at(RouteName.Unlock))
    expect(setEntryView(state, at(RouteName.Collection), { sort: 'name' })).toBe(
      state,
    )
  })

  it('leaves the views of the other entries alone when it writes', () => {
    const start = navigateTab(
      setEntryView(firstState('a', at(RouteName.Unlock)), at(RouteName.Unlock), {
        sort: 'name',
      }),
      at(RouteName.Collection),
    )
    const state = setEntryView(start, at(RouteName.Collection), { sort: 'id' })
    const tab = state.tabs[0]!
    expect(tab.entries[0]?.view).toEqual({ sort: 'name' })
    expect(tab.entries[1]?.view).toEqual({ sort: 'id' })
  })

  // Going back is going back to what you were looking at, which is the whole reason the record
  // sits on the entry and not on the tab.
  it('gives back the view of the entry it returns to', () => {
    const start = navigateTab(
      setEntryView(firstState('a', at(RouteName.Unlock)), at(RouteName.Unlock), {
        sort: 'name',
      }),
      at(RouteName.Collection),
    )
    expect(entryView(backTab(start).tabs[0]!)).toEqual({ sort: 'name' })
  })

  // `tabSeed` is written by subtraction, so this holds without a line being added for it. The
  // test exists because that is the property the tear-off rests on.
  it('carries the view across a tear-off', () => {
    const state = setEntryView(
      firstState('a', at(RouteName.Unlock)),
      at(RouteName.Unlock),
      { sort: 'name' },
    )
    const seed = tabSeed(state.tabs[0]!)
    expect(seed.entries[0]?.view).toEqual({ sort: 'name' })
  })

  // A refinement replaces the entry in place, and the view belongs to the view, not to the
  // query string that refined it.
  it('keeps the view when the same view is refined', () => {
    const start = setEntryView(
      firstState('a', at(RouteName.Unlock)),
      at(RouteName.Unlock),
      { sort: 'name' },
    )
    const state = refineTab(start, {
      name: RouteName.Unlock,
      query: { q: 'brim' },
    })
    expect(entryView(state.tabs[0]!)).toEqual({ sort: 'name' })
    expect(tabLocation(state.tabs[0]!).query?.q).toBe('brim')
  })
})
```

Add `Entry`, `entryView` and `setEntryView` to the import block at the top of the test file.

- [ ] **Step 2: Run the tests and verify they fail**

Run: `pnpm --filter ui test -- tabModel`
Expected: FAIL — `entryView is not a function`, plus type errors on `entries` being locations.

- [ ] **Step 3: Write the implementation**

In `ui/src/stores/tabModel.ts`:

```ts
// A tab's history entry: where it is, and how it was being read there. `view` is `unknown` on
// purpose — the shell stores it and never looks inside, and each screen validates its own
// (`lib/tabs/tabView.ts`). It is optional so that an entry of a screen with nothing to remember
// stays exactly as small as it was.
export interface Entry {
  location: TabLocation
  view?: unknown
}

export interface Tab {
  id: string
  entries: Entry[]
  index: number
}
```

Then, in the same file:

```ts
export const tabLocation = (tab: Tab): TabLocation =>
  tab.entries[tab.index]!.location

export const entryView = (tab: Tab): unknown => tab.entries[tab.index]?.view

export const firstState = (id: string, location: TabLocation): TabsState => ({
  tabs: [{ id, entries: [{ location }], index: 0 }],
  activeId: id,
})
```

`openTab` builds `entries: [{ location }]`. `seedState` is unchanged — it spreads a seed whose
entries are already entries.

`sameView` reads locations, so it takes them rather than entries:

```ts
const goTo = (tab: Tab, location: TabLocation): Tab => {
  if (sameView(tabLocation(tab), location)) {
    const entries = [...tab.entries]
    // The view survives a refinement of the same view: what was refined is the location, and
    // the record describes the reading, not the query that reached it.
    entries[tab.index] = { ...entries[tab.index]!, location }
    return { ...tab, entries }
  }
  const entries = [...tab.entries.slice(0, tab.index + 1), { location }].slice(
    -HistoryDepth,
  )
  return { ...tab, entries, index: entries.length - 1 }
}
```

And the writer, with `refineTab`'s guard:

```ts
// Changing how the entry the tab is showing is being read, and nothing else. The guard is
// `refineTab`'s and exists for the same reason: a debounced write can land after the user has
// gone back or switched tab, and what it says must reach no tab rather than the wrong one.
export const setEntryView = (
  state: TabsState,
  location: TabLocation,
  view: unknown,
): TabsState => {
  const tab = state.tabs.find((each) => each.id === state.activeId)
  if (!tab || !sameView(tabLocation(tab), location)) return state
  const entries = [...tab.entries]
  entries[tab.index] = { ...entries[tab.index]!, view }
  return {
    ...state,
    tabs: state.tabs.map((each) =>
      each.id === tab.id ? { ...each, entries } : each,
    ),
  }
}
```

In `ui/src/stores/tabs.ts`, `fresh()` becomes:

```ts
  const fresh = (): Tab => ({
    id: nextId(),
    entries: [{ location: defaultLocation }],
    index: 0,
  })
```

and the store gains, next to `refine`:

```ts
  // How the active tab's current entry is being read. The rule is `tabModel`'s; this only
  // holds the result, as with every other tab rule.
  const setView = (location: TabLocation, view: unknown): void => {
    state.value = setEntryView(state.value, location, view)
  }
```

Add `setView` to the returned object and `setEntryView` to the imports.

- [ ] **Step 4: Run the tests and verify they pass**

Run: `pnpm --filter ui test -- tabModel`
Expected: PASS, including the tests that were already there — they are what says no rule changed
meaning while the shape did.

- [ ] **Step 5: Make the rest of the app compile**

Run: `pnpm typecheck`
Expected: errors only where an entry was built or read as a bare location. Fix each by hand —
`sessionDocument.ts` is Task 2 and may be left failing typecheck **only** if Task 2 follows
immediately; otherwise fix it here and let Task 2 refine it.

- [ ] **Step 6: Commit**

```bash
git add ui/src/stores/tabModel.ts ui/src/stores/tabModel.test.ts ui/src/stores/tabs.ts
git commit -m "feat(ui): a tab's history entry carries how it was being read"
```

---

### Task 2: the session document reads and prunes the record

**Files:**
- Modify: `ui/src/lib/window/sessionDocument.ts`
- Test: `ui/src/lib/window/sessionDocument.test.ts`

**Interfaces:**
- Consumes: `Entry`, `Tab`, `TabSeed` from Task 1.
- Produces: `readSession(raw: string | null): Session | null` and
  `writeSession(session: Session): string`, both unchanged in signature. The document's
  `Version` stays **1**: an entry gaining an optional key is a part an older app can ignore, and
  the version is bumped only when an older app could read the new shape and be *wrong* about it.

- [ ] **Step 1: Write the failing tests**

```ts
describe('the view a stored entry carries', () => {
  // The document on disk today has entries that are bare locations. Whoever updates the app
  // must not lose the tabs they had open.
  it('reads an entry written before entries had a view', () => {
    const raw = JSON.stringify({
      version: 1,
      tabs: [{ entries: [{ name: RouteName.Unlock }], index: 0 }],
      activeIndex: 0,
    })
    const session = readSession(raw)
    expect(session?.tabs[0]?.entries[0]).toEqual({
      location: { name: RouteName.Unlock },
    })
  })

  it('reads an entry that carries one', () => {
    const raw = JSON.stringify({
      version: 1,
      tabs: [
        {
          entries: [
            { location: { name: RouteName.Unlock }, view: { sort: 'name' } },
          ],
          index: 0,
        },
      ],
      activeIndex: 0,
    })
    expect(readSession(raw)?.tabs[0]?.entries[0]?.view).toEqual({
      sort: 'name',
    })
  })

  // B6's open question, answered one notch finer than the rule above it: a tab whose *route*
  // is gone still falls whole, because there is nothing left to open. A tab whose *record* is
  // unreadable opens on its screen and lets the screen say what it has.
  it('drops an unreadable view and keeps the tab', () => {
    const raw = JSON.stringify({
      version: 1,
      tabs: [
        {
          entries: [{ location: { name: RouteName.Unlock }, view: 'not-an-object' }],
          index: 0,
        },
      ],
      activeIndex: 0,
    })
    const entry = readSession(raw)?.tabs[0]?.entries[0]
    expect(entry?.location).toEqual({ name: RouteName.Unlock })
    expect(entry?.view).toBeUndefined()
  })

  it('still drops the whole tab when a route is gone', () => {
    const raw = JSON.stringify({
      version: 1,
      tabs: [
        { entries: [{ location: { name: 'a-screen-that-left' } }], index: 0 },
        { entries: [{ location: { name: RouteName.Unlock } }], index: 0 },
      ],
      activeIndex: 1,
    })
    expect(readSession(raw)?.tabs).toHaveLength(1)
  })

  // The document is bounded by construction rather than by a number: the cap is 64 KiB and its
  // budget was written for locations alone. Everything in memory keeps its views; only what is
  // stored is pruned.
  it('stores the view of the entry each tab is showing and of no other', () => {
    const tab: TabSeed = {
      entries: [
        { location: { name: RouteName.Unlock }, view: { sort: 'name' } },
        { location: { name: RouteName.Collection }, view: { sort: 'id' } },
      ],
      index: 1,
    }
    const stored = JSON.parse(writeSession({ tabs: [tab], activeIndex: 0 }))
    expect(stored.tabs[0].entries[0].view).toBeUndefined()
    expect(stored.tabs[0].entries[1].view).toEqual({ sort: 'id' })
  })

  it('round-trips what it kept', () => {
    const tab: TabSeed = {
      entries: [{ location: { name: RouteName.Unlock }, view: { sort: 'name' } }],
      index: 0,
    }
    const back = readSession(writeSession({ tabs: [tab], activeIndex: 0 }))
    expect(back?.tabs[0]?.entries[0]?.view).toEqual({ sort: 'name' })
  })
})
```

- [ ] **Step 2: Run the tests and verify they fail**

Run: `pnpm --filter ui test -- sessionDocument`
Expected: FAIL — the reader answers locations where entries are expected, and the writer stores
every view.

- [ ] **Step 3: Write the implementation**

In `ui/src/lib/window/sessionDocument.ts`, keep `readLocation` as it is and add:

```ts
// A stored view, as far as it can be trusted here: an object, and nothing more. What it means
// is the screen's, and the screen validates it (`lib/tabs/tabView.ts`). Anything else — a
// string, a number, `null` — is dropped, and **the entry survives without it**: a tab that
// opens saying what it has beats a tab that vanishes (B6).
const readView = (value: unknown): unknown =>
  typeof value === 'object' && value !== null && !Array.isArray(value)
    ? value
    : undefined

// Two shapes, one reader. Before 3.7a an entry *was* a location, and a document written by that
// version must still open: losing somebody's tabs on an update is not a thing the app can
// explain to them afterwards.
const readEntry = (value: unknown): Entry | null => {
  if (typeof value !== 'object' || value === null) return null
  const { location, view } = value as { location?: unknown; view?: unknown }
  if (location === undefined) {
    const bare = readLocation(value)
    return bare === null ? null : { location: bare }
  }
  const read = readLocation(location)
  if (read === null) return null
  const kept = readView(view)
  return kept === undefined ? { location: read } : { location: read, view: kept }
}
```

`readTab` maps `readEntry` instead of `readLocation`; its "all or nothing" comment stays exactly
as it is, because it is still true of the locations.

The writer prunes:

```ts
// Only the entry each tab is showing keeps its view. `MAX_SESSION_BYTES` is 64 KiB and its
// budget was written for "fifty tabs of route names and queries"; a record on every one of a
// tab's fifty entries is a different sum. Bounding it by construction beats raising a number,
// and what it costs is that going back in a *restored* tab gets the screen's empty state — the
// same thing that happened to every restored tab before 3.7a.
const stored = (tab: TabSeed): TabSeed => ({
  ...tab,
  entries: tab.entries.map((entry, at) =>
    at === tab.index ? entry : { location: entry.location },
  ),
})

export const writeSession = (session: Session): string =>
  JSON.stringify({
    version: Version,
    tabs: session.tabs.map(stored),
    activeIndex: session.activeIndex,
  })
```

Update `Version`'s comment: the sidebar width and table sizes it names are 3.7c, and windows are
3.7b, which is where the number does move.

- [ ] **Step 4: Run the tests and verify they pass**

Run: `pnpm --filter ui test -- sessionDocument`
Expected: PASS, the file's existing tests included.

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib/window/sessionDocument.ts ui/src/lib/window/sessionDocument.test.ts
git commit -m "feat(ui): the session keeps the reading of the entry each tab is showing"
```

---

### Task 3: `TabViewSpec`, and the readers every screen's spec is built from

**Files:**
- Create: `ui/src/lib/tabs/tabView.ts`
- Test: `ui/src/lib/tabs/tabView.test.ts`

**Interfaces:**
- Consumes: `FacetFilter` from `@/lib/facets/faceting`.
- Produces:
  `interface TabViewSpec<T> { empty: () => T; read: (value: unknown) => T | null }`;
  `readString(value: unknown, allowed: readonly string[]): string | null`;
  `readStringArray(value: unknown, allowed?: readonly string[]): string[] | null`;
  `readFacetFilter<F extends string>(value: unknown, order: readonly F[]): FacetFilter<F> | null`.

- [ ] **Step 1: Write the failing tests**

```ts
import { describe, expect, it } from 'vitest'
import { readFacetFilter, readString, readStringArray } from './tabView'

const order = ['state', 'origin'] as const

describe('reading a value written by an older version of this app', () => {
  it('takes a string only from the set that still exists', () => {
    expect(readString('name', ['name', 'id'])).toBe('name')
    expect(readString('fanOut', ['name', 'id'])).toBeNull()
    expect(readString(3, ['name'])).toBeNull()
  })

  it('takes an array of strings, and nothing else', () => {
    expect(readStringArray(['a', 'b'])).toEqual(['a', 'b'])
    expect(readStringArray([])).toEqual([])
    expect(readStringArray(['a', 2])).toBeNull()
    expect(readStringArray('a')).toBeNull()
  })

  // A value that was legal when it was written and is not a facet value any more: dropped,
  // rather than kept as a pick that filters everything away and reads as an empty profile.
  it('drops an array value the screen no longer knows', () => {
    expect(readStringArray(['now', 'gone'], ['now'])).toEqual(['now'])
  })

  it('reads a facet filter and fills the facets the document never had', () => {
    expect(
      readFacetFilter({ query: 'brim', picks: { state: ['now'] } }, order),
    ).toEqual({ query: 'brim', picks: { state: ['now'], origin: [] } })
  })

  // The facets are what `order` says they are: a key from a facet that no longer exists is not
  // part of the faceting, and carrying it would hand the engine a `picks` it cannot index.
  it('drops a facet the screen no longer has', () => {
    expect(
      readFacetFilter({ query: '', picks: { gone: ['x'] } }, order)?.picks,
    ).toEqual({ state: [], origin: [] })
  })

  it('refuses something that is not a filter at all', () => {
    expect(readFacetFilter(null, order)).toBeNull()
    expect(readFacetFilter({ query: 7 }, order)).toBeNull()
  })
})
```

- [ ] **Step 2: Run the tests and verify they fail**

Run: `pnpm --filter ui test -- tabView`
Expected: FAIL — `Cannot find module './tabView'`.

- [ ] **Step 3: Write the implementation**

```ts
import { emptyFilter } from '@/lib/facets/faceting'
import type { FacetFilter } from '@/lib/facets/faceting'

// What a screen declares so that the shell can hand it back a reading it wrote before — before
// a tear-off, or before the app was last closed, or before it was updated. `read` answering
// `null` is not an error: it is this app meeting a record written by a version of itself that
// no longer exists, and the answer to that is the empty reading, never a crash and never a
// filter nobody set.
//
// There is no key. A record lives on a history entry and an entry has exactly one screen, so a
// key would name what the entry already names.
export interface TabViewSpec<T> {
  empty: () => T
  read: (value: unknown) => T | null
}

export const readString = (
  value: unknown,
  allowed: readonly string[],
): string | null =>
  typeof value === 'string' && allowed.includes(value) ? value : null

// `allowed` left out means "any string": a page key or a search query is not drawn from a set
// this side knows. Given one, a value outside it is dropped rather than kept — a pick nobody
// can see filters everything away and reads as an empty screen.
export const readStringArray = (
  value: unknown,
  allowed?: readonly string[],
): string[] | null => {
  if (!Array.isArray(value)) return null
  if (!value.every((each) => typeof each === 'string')) return null
  const strings = value as string[]
  return allowed === undefined
    ? strings
    : strings.filter((each) => allowed.includes(each))
}

// The facets are what `order` says they are. A stored `picks` is read facet by facet through
// it, so a facet that has since been added arrives empty and one that has gone is not carried
// into a `Record` the engine would index by a key that no longer exists.
export const readFacetFilter = <F extends string>(
  value: unknown,
  order: readonly F[],
): FacetFilter<F> | null => {
  if (typeof value !== 'object' || value === null) return null
  const { query, picks } = value as { query?: unknown; picks?: unknown }
  if (typeof query !== 'string') return null
  const empty = emptyFilter<F>([...order])
  if (typeof picks !== 'object' || picks === null) return { ...empty, query }
  const source = picks as Record<string, unknown>
  return {
    query,
    picks: Object.fromEntries(
      order.map((facet) => [facet, readStringArray(source[facet]) ?? []]),
    ) as Record<F, string[]>,
  }
}
```

- [ ] **Step 4: Run the tests and verify they pass**

Run: `pnpm --filter ui test -- tabView`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib/tabs/
git commit -m "feat(ui): what a screen declares so its reading can be given back to it"
```

---

### Task 4: `useTabView`, the one way a screen reaches its record

**Files:**
- Create: `ui/src/composables/useTabView.ts`
- Test: `ui/src/composables/useTabView.test.ts`

**Interfaces:**
- Consumes: `TabViewSpec` (Task 3); `useTabsStore` with `setView` (Task 1).
- Produces: `useTabView<T>(spec: TabViewSpec<T>): Ref<T>`.

- [ ] **Step 1: Write the failing tests**

The store is real — a Pinia store is cheap to build in a test and mocking it would test the
mock. Follow `ui/src/stores/queue.test.ts` for how this repo sets up `createPinia`.

```ts
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent, nextTick } from 'vue'
import { mount } from '@vue/test-utils'
import { RouteName } from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'
import { useTabView } from './useTabView'
import type { TabViewSpec } from '@/lib/tabs/tabView'

interface Reading {
  sort: string
}

const spec: TabViewSpec<Reading> = {
  empty: () => ({ sort: 'fanOut' }),
  read: (value) =>
    typeof value === 'object' && value !== null && 'sort' in value
      ? { sort: String((value as { sort: unknown }).sort) }
      : null,
}

const host = defineComponent({
  setup() {
    return { reading: useTabView(spec) }
  },
  template: '<i />',
})

describe('a screen reaching its own reading', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.useFakeTimers()
  })

  it('opens on the empty reading when the entry has none', () => {
    const tabs = useTabsStore()
    tabs.seed([{ entries: [{ location: { name: RouteName.Unlock } }], index: 0 }], 0)
    const wrapper = mount(host)
    expect(wrapper.vm.reading).toEqual({ sort: 'fanOut' })
  })

  it('opens on what the entry carries', () => {
    const tabs = useTabsStore()
    tabs.seed(
      [
        {
          entries: [
            { location: { name: RouteName.Unlock }, view: { sort: 'name' } },
          ],
          index: 0,
        },
      ],
      0,
    )
    expect(mount(host).vm.reading).toEqual({ sort: 'name' })
  })

  it('opens on the empty reading when the record cannot be read', () => {
    const tabs = useTabsStore()
    tabs.seed(
      [
        {
          entries: [
            { location: { name: RouteName.Unlock }, view: { nothing: 1 } },
          ],
          index: 0,
        },
      ],
      0,
    )
    expect(mount(host).vm.reading).toEqual({ sort: 'fanOut' })
  })

  it('writes what changed back into the entry, once', async () => {
    const tabs = useTabsStore()
    tabs.seed([{ entries: [{ location: { name: RouteName.Unlock } }], index: 0 }], 0)
    const wrapper = mount(host)
    wrapper.vm.reading = { sort: 'name' }
    wrapper.vm.reading = { sort: 'id' }
    await nextTick()
    // Nothing written yet: a burst of changes is one write, as the session's own is.
    expect(tabs.active?.entries[0]?.view).toBeUndefined()
    vi.runAllTimers()
    expect(tabs.active?.entries[0]?.view).toEqual({ sort: 'id' })
  })

  // Going back is going back to what you were looking at. The screen stays mounted when the
  // route does not change, so the composable — not the component's lifecycle — is what has to
  // notice.
  it('re-reads when the tab moves to another entry under it', async () => {
    const tabs = useTabsStore()
    tabs.seed(
      [
        {
          entries: [
            { location: { name: RouteName.Unlock }, view: { sort: 'name' } },
            { location: { name: RouteName.Unlock, query: { q: 'brim' } } },
          ],
          index: 1,
        },
      ],
      0,
    )
    const wrapper = mount(host)
    expect(wrapper.vm.reading).toEqual({ sort: 'fanOut' })
    tabs.back()
    await nextTick()
    expect(wrapper.vm.reading).toEqual({ sort: 'name' })
  })
})
```

- [ ] **Step 2: Run the tests and verify they fail**

Run: `pnpm --filter ui test -- useTabView`
Expected: FAIL — `Cannot find module './useTabView'`.

- [ ] **Step 3: Write the implementation**

```ts
import { useDebounceFn } from '@vueuse/core'
import { ref, watch } from 'vue'
import type { Ref } from 'vue'
import { Timing } from '@/lib/constants/timing'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import { entryView } from '@/stores/tabModel'
import { useTabsStore } from '@/stores/tabs'

// How a screen remembers how it was being read, and the only way it does: a screen never
// reaches into the tab store, exactly as it never calls `invoke()`. What comes back is an
// ordinary ref — the screen writes to it the way it wrote to its own, and where it lands is
// this composable's business.
//
// The debounce is for the burst — a facet click moves a pick and a count in the same breath —
// not for the cost, which is an object assigned into an array.
export const useTabView = <T>(spec: TabViewSpec<T>): Ref<T> => {
  const tabs = useTabsStore()

  const current = (): T => {
    const tab = tabs.active
    return (tab ? spec.read(entryView(tab)) : null) ?? spec.empty()
  }

  const reading = ref(current()) as Ref<T>

  // Which entry we are on: the tab and its position in its own history. A tab going back to
  // the same route keeps the screen mounted, so the component's lifecycle cannot notice it —
  // this can.
  const entryId = () => `${tabs.activeId}#${tabs.active?.index ?? 0}`

  let at = entryId()
  watch(entryId, (id) => {
    at = id
    reading.value = current()
  })

  const remember = useDebounceFn(() => {
    const location = tabs.location
    // The entry moved under the write while it waited. `setView` would refuse it anyway — this
    // only saves asking.
    if (!location || entryId() !== at) return
    tabs.setView(location, reading.value)
  }, Timing.ViewWrite)

  watch(reading, () => void remember(), { deep: true })

  return reading
}
```

`Timing` holds one value today (`SearchDebounce: 120`). Add a second beside it rather than
reusing a number that means something else:

```ts
export const Timing = {
  SearchDebounce: 120,
  // Coalescing a burst — a facet click moves a pick and a count in the same breath. The same
  // 120 as the search, for the same reason, and named separately because the day one moves the
  // other has no reason to. It costs nothing to be small: what it feeds is the session's own
  // 400ms write, so anything under that is invisible.
  ViewWrite: 120,
} as const
```

- [ ] **Step 4: Run the tests and verify they pass**

Run: `pnpm --filter ui test -- useTabView`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add ui/src/composables/useTabView.ts ui/src/composables/useTabView.test.ts ui/src/lib/constants/timing.ts
git commit -m "feat(ui): one composable gives a screen back the reading it left"
```

---

### Task 5: Unlock adopts it

**Files:**
- Create: `ui/src/screens/unlock/tabView.ts`
- Modify: `ui/src/screens/UnlockScreen.vue:57` and `:85`
- Test: `ui/src/screens/unlock/tabView.test.ts`

**Interfaces:**
- Consumes: `TabViewSpec`, `readFacetFilter`, `readString` (Task 3); `useTabView` (Task 4).
- Produces: `interface UnlockReading { filter: UnlockFilter; sort: UnlockSort }` and
  `unlockView: TabViewSpec<UnlockReading>`.

- [ ] **Step 1: Write the failing test**

```ts
import { describe, expect, it } from 'vitest'
import { FacetId, UnlockSort, unlockFaceting } from '@/lib/graph/unlockFacets'
import { unlockView } from './tabView'

describe("Unlock's reading", () => {
  it('is the empty faceting and the default sort when there is nothing to read', () => {
    expect(unlockView.empty()).toEqual({
      filter: unlockFaceting.empty(),
      sort: UnlockSort.FanOut,
    })
  })

  it('reads back a filter and a sort it wrote', () => {
    const reading = {
      filter: { ...unlockFaceting.empty(), query: 'brim' },
      sort: UnlockSort.Name,
    }
    expect(unlockView.read(JSON.parse(JSON.stringify(reading)))).toEqual(reading)
  })

  // A sort that was legal when it was written and is not a sort any more falls back to the
  // default rather than reaching `sortNodes` as a value it has no branch for.
  it('falls back to the default sort when the stored one is gone', () => {
    expect(
      unlockView.read({ filter: unlockFaceting.empty(), sort: 'byVibes' })?.sort,
    ).toBe(UnlockSort.FanOut)
  })

  it('refuses a record that is not a reading', () => {
    expect(unlockView.read({ sort: UnlockSort.Name })).toBeNull()
    expect(unlockView.read(null)).toBeNull()
  })
})
```

- [ ] **Step 2: Run the test and verify it fails**

Run: `pnpm --filter ui test -- unlock/tabView`
Expected: FAIL — `Cannot find module './tabView'`.

- [ ] **Step 3: Write the implementation**

`ui/src/screens/unlock/tabView.ts`:

```ts
import { readFacetFilter, readString } from '@/lib/tabs/tabView'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import { FacetId, UnlockSort, facetOrder, unlockFaceting } from '@/lib/graph/unlockFacets'
import type { UnlockFilter } from '@/lib/graph/unlockFacets'

export interface UnlockReading {
  filter: UnlockFilter
  sort: UnlockSort
}

const sorts = Object.values(UnlockSort)

// The filter is the part a reader can refuse; the sort is a value out of a closed set, and a
// stored one that has left the set falls back to the default rather than reaching `sortNodes`
// as a string it has no branch for.
export const unlockView: TabViewSpec<UnlockReading> = {
  empty: () => ({ filter: unlockFaceting.empty(), sort: UnlockSort.FanOut }),
  read: (value) => {
    if (typeof value !== 'object' || value === null) return null
    const { filter, sort } = value as { filter?: unknown; sort?: unknown }
    const read = readFacetFilter<FacetId>(filter, facetOrder)
    if (read === null) return null
    return {
      filter: read,
      sort: (readString(sort, sorts) as UnlockSort | null) ?? UnlockSort.FanOut,
    }
  },
}
```

In `UnlockScreen.vue`, the two refs become one reading. Replace

```ts
// The filter belongs to this screen: leaving the tab resets it, until tabs keep their state.
const filter = ref<UnlockFilter>(unlockFaceting.empty())
```

and `const sort = ref<UnlockSort>(UnlockSort.FanOut)` with

```ts
// The filter and the sort belong to the tab, not to this component: leaving and coming back —
// through a tear-off, a restart, or the back button — finds them where they were left (B39).
const reading = useTabView(unlockView)
const filter = computed({
  get: () => reading.value.filter,
  set: (value: UnlockFilter) => {
    reading.value = { ...reading.value, filter: value }
  },
})
const sort = computed({
  get: () => reading.value.sort,
  set: (value: UnlockSort) => {
    reading.value = { ...reading.value, sort: value }
  },
})
```

Nothing else in the file changes: every use of `filter.value` and `sort.value` reads and writes
the same way through a writable computed.

- [ ] **Step 4: Run the test and verify it passes**

Run: `pnpm --filter ui test -- unlock/tabView`
Expected: PASS.

- [ ] **Step 5: Check the screen still typechecks and scans**

Run: `pnpm typecheck && pnpm scan`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add ui/src/screens/unlock/tabView.ts ui/src/screens/unlock/tabView.test.ts ui/src/screens/UnlockScreen.vue
git commit -m "feat(ui): Unlock's facets and sort belong to the tab"
```

---

### Task 6: the Collection adopts it

**Files:**
- Create: `ui/src/screens/collection/tabView.ts`
- Modify: `ui/src/screens/CollectionScreen.vue:56` and `:68`
- Test: `ui/src/screens/collection/tabView.test.ts`

**Interfaces:**
- Consumes: the same as Task 5.
- Produces: `interface CollectionReading { filter: CollectionFilter; sort: CollectionSort }`
  and `collectionView: TabViewSpec<CollectionReading>`.

- [ ] **Step 1: Write the failing test**

```ts
import { describe, expect, it } from 'vitest'
import {
  CollectionSort,
  defaultCollectionFilter,
  emptyCollectionFilter,
} from '@/lib/collection/collectionFacets'
import { collectionView } from './tabView'

describe("the Collection's reading", () => {
  // The Collection does **not** open on the empty filter: it opens on what has not been found,
  // and that is the reading a tab with nothing stored starts from.
  it('is the default filter, not the empty one, when there is nothing to read', () => {
    expect(collectionView.empty()).toEqual({
      filter: defaultCollectionFilter(),
      sort: CollectionSort.Quality,
    })
  })

  // And a filter that was deliberately cleared must come back cleared: reading an empty stored
  // filter as "nothing stored" would put the opening filter back on every restart.
  it('reads back a filter that was cleared', () => {
    const reading = {
      filter: emptyCollectionFilter(),
      sort: CollectionSort.Name,
    }
    expect(collectionView.read(JSON.parse(JSON.stringify(reading)))).toEqual(
      reading,
    )
  })

  it('falls back to the default sort when the stored one is gone', () => {
    expect(
      collectionView.read({ filter: emptyCollectionFilter(), sort: 'byVibes' })
        ?.sort,
    ).toBe(CollectionSort.Quality)
  })

  it('refuses a record that is not a reading', () => {
    expect(collectionView.read({ sort: CollectionSort.Name })).toBeNull()
  })
})
```

- [ ] **Step 2: Run the test and verify it fails**

Run: `pnpm --filter ui test -- collection/tabView`
Expected: FAIL — module not found.

- [ ] **Step 3: Write the implementation**

```ts
import { readFacetFilter, readString } from '@/lib/tabs/tabView'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import {
  CollectionFacet,
  CollectionSort,
  collectionFacetOrder,
  defaultCollectionFilter,
} from '@/lib/collection/collectionFacets'
import type { CollectionFilter } from '@/lib/collection/collectionFacets'

export interface CollectionReading {
  filter: CollectionFilter
  sort: CollectionSort
}

const sorts = Object.values(CollectionSort)

// `empty` is the *default* filter and not the empty one: this screen opens on what has not been
// found. `read` never falls back to it — a filter the user cleared is a filter they cleared.
export const collectionView: TabViewSpec<CollectionReading> = {
  empty: () => ({
    filter: defaultCollectionFilter(),
    sort: CollectionSort.Quality,
  }),
  read: (value) => {
    if (typeof value !== 'object' || value === null) return null
    const { filter, sort } = value as { filter?: unknown; sort?: unknown }
    const read = readFacetFilter<CollectionFacet>(filter, collectionFacetOrder)
    if (read === null) return null
    return {
      filter: read,
      sort:
        (readString(sort, sorts) as CollectionSort | null) ??
        CollectionSort.Quality,
    }
  },
}
```

`collectionFacetOrder` is already exported from `ui/src/lib/collection/collectionFacets.ts:22` —
the same list `emptyCollectionFilter` builds from. Import it; do not write a second one.

In `CollectionScreen.vue`, replace the two refs exactly as Task 5 does for Unlock, with
`collectionView` and the `CollectionFilter` / `CollectionSort` types.

- [ ] **Step 4: Run the test and verify it passes**

Run: `pnpm --filter ui test -- collection/tabView`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add ui/src/screens/collection/tabView.ts ui/src/screens/collection/tabView.test.ts ui/src/screens/CollectionScreen.vue ui/src/lib/collection/collectionFacets.ts
git commit -m "feat(ui): the Collection's filter and sort belong to the tab"
```

---

### Task 7: Runs adopts it, and a run's identity stops being an inline expression

**Files:**
- Create: `ui/src/lib/runs/runKey.ts`, `ui/src/lib/runs/runKey.test.ts`
- Create: `ui/src/screens/runs/tabView.ts`, `ui/src/screens/runs/tabView.test.ts`
- Modify: `ui/src/screens/RunsScreen.vue:36-37`, `ui/src/screens/runs/RunsTable.vue:14-17`

**Interfaces:**
- Consumes: Task 3 and Task 4.
- Produces: `runKey(run: RunView): string`;
  `interface RunsReading { filter: FacetFilter<RunFacet>; selected: string | null }`;
  `runsView: TabViewSpec<RunsReading>`.

- [ ] **Step 1: Write the failing tests**

`ui/src/lib/runs/runKey.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import type { RunView } from '@/lib/ipc/types'
import { runKey } from './runKey'

const run = (source: RunView['source'], ordinal: number) =>
  ({ source, ordinal }) as RunView

describe("a run's key", () => {
  it('is its source and its position in that source', () => {
    expect(runKey(run({ kind: 'session', name: '09_14' }, 2))).toBe(
      'session:09_14#2',
    )
  })

  // A launch of `log.txt` has no name — that is why the store keys it `NULL` — so the kind is
  // what tells it from a session, and the two must never collide.
  it('tells a launch from a session that could be named like one', () => {
    expect(runKey(run({ kind: 'live' }, 1))).not.toBe(
      runKey(run({ kind: 'session', name: 'live' }, 1)),
    )
  })
})
```

`ui/src/screens/runs/tabView.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { emptyFilter } from '@/lib/facets/faceting'
import { RunFacet } from '@/lib/runs/runFacets'
import { runsView } from './tabView'

const order = Object.values(RunFacet)

describe("Runs' reading", () => {
  it('is the empty filter and nothing selected when there is nothing to read', () => {
    expect(runsView.empty()).toEqual({
      filter: emptyFilter<RunFacet>(order),
      selected: null,
    })
  })

  // The selection is the run's key and never the run: a view-model is the archive's answer of
  // the moment, and storing one would restore a row that no longer describes anything.
  it('reads back the key of the selected run', () => {
    expect(
      runsView.read({
        filter: emptyFilter<RunFacet>(order),
        selected: 'session:09_14#2',
      })?.selected,
    ).toBe('session:09_14#2')
  })

  it('reads a selection that is not a string as nothing selected', () => {
    expect(
      runsView.read({ filter: emptyFilter<RunFacet>(order), selected: 7 })
        ?.selected,
    ).toBeNull()
  })
})
```

- [ ] **Step 2: Run the tests and verify they fail**

Run: `pnpm --filter ui test -- runKey runs/tabView`
Expected: FAIL — both modules missing.

- [ ] **Step 3: Write the implementation**

`ui/src/lib/runs/runKey.ts` — move the expression out of `RunsTable.vue:17` and keep its comment,
which is the sentence that explains it:

```ts
import type { RunView } from '@/lib/ipc/types'

// A run is `(source, ordinal)`: that pair is its identity in the archive and therefore the only
// thing that can be written down about *which* run. The kind is part of the key — a launch of
// `log.txt` has no name, which is why the store keys it `NULL`, and a session could be called
// anything at all.
export const runKey = (run: RunView): string =>
  `${run.source.kind === 'live' ? 'live:' : `session:${run.source.name}`}#${run.ordinal}`
```

Adjust the test's expected strings if this formatting differs — but **change the test only after
deciding which form is right**, never to match what the code happened to print.

`ui/src/screens/runs/tabView.ts`:

```ts
import { readFacetFilter } from '@/lib/tabs/tabView'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import { RunFacet } from '@/lib/runs/runFacets'
import { emptyFilter } from '@/lib/facets/faceting'
import type { FacetFilter } from '@/lib/facets/faceting'

export interface RunsReading {
  filter: FacetFilter<RunFacet>
  selected: string | null
}

const order = Object.values(RunFacet)

export const runsView: TabViewSpec<RunsReading> = {
  empty: () => ({ filter: emptyFilter<RunFacet>(order), selected: null }),
  read: (value) => {
    if (typeof value !== 'object' || value === null) return null
    const { filter, selected } = value as {
      filter?: unknown
      selected?: unknown
    }
    const read = readFacetFilter<RunFacet>(filter, order)
    if (read === null) return null
    return {
      filter: read,
      selected: typeof selected === 'string' ? selected : null,
    }
  },
}
```

In `RunsScreen.vue`, `filter` becomes a writable computed over the reading as in Task 5, and the
selection stops being a `RunView`:

```ts
// The selected run travels as its key, never as the row: a row is the archive's answer of the
// moment, and a stored one would come back describing a run the fold has since re-derived.
const selectedKey = computed({
  get: () => reading.value.selected,
  set: (value: string | null) => {
    reading.value = { ...reading.value, selected: value }
  },
})
const selected = computed(
  () => rows.value.find((run) => runKey(run) === selectedKey.value) ?? null,
)
```

Every `selected.value = run` in the file becomes `selectedKey.value = runKey(run)`, and
`selected.value = null` becomes `selectedKey.value = null`. `RunsTable.vue` imports `runKey`
instead of declaring its own.

- [ ] **Step 4: Run the tests and verify they pass**

Run: `pnpm --filter ui test -- runKey runs/tabView`
Expected: PASS.

- [ ] **Step 5: Run the whole frontend suite**

Run: `pnpm ui:test`
Expected: PASS — `RunsTable`'s own tests, if any, are what say the key did not change meaning
when it moved.

- [ ] **Step 6: Commit**

```bash
git add ui/src/lib/runs/runKey.ts ui/src/lib/runs/runKey.test.ts ui/src/screens/runs/ ui/src/screens/RunsScreen.vue
git commit -m "feat(ui): a run's key names the selection the tab remembers"
```

---

### Task 8: Search adopts it

**Files:**
- Create: `ui/src/screens/search/tabView.ts`, `ui/src/screens/search/tabView.test.ts`
- Modify: `ui/src/screens/SearchScreen.vue:80`

**Interfaces:**
- Consumes: Task 3 and Task 4.
- Produces: `interface SearchReading { picked: RowGroup[] }`;
  `searchView: TabViewSpec<SearchReading>`.

- [ ] **Step 1: Write the failing test**

```ts
import { describe, expect, it } from 'vitest'
import { RowGroup } from '@/lib/search/rows'
import { searchView } from './tabView'

describe("Search's reading", () => {
  it('is nothing picked when there is nothing to read', () => {
    expect(searchView.empty()).toEqual({ picked: [] })
  })

  it('reads back the groups that were picked', () => {
    expect(searchView.read({ picked: [RowGroup.Wiki] })?.picked).toEqual([
      RowGroup.Wiki,
    ])
  })

  // A group that has since gone is dropped, not kept: a pick nobody can see filters every row
  // away and reads as a search that found nothing.
  it('drops a group it no longer has', () => {
    expect(
      searchView.read({ picked: [RowGroup.Wiki, 'seances'] })?.picked,
    ).toEqual([RowGroup.Wiki])
  })

  it('refuses a record that is not a reading', () => {
    expect(searchView.read({ picked: 'wiki' })).toBeNull()
  })
})
```

- [ ] **Step 2: Run the test and verify it fails**

Run: `pnpm --filter ui test -- search/tabView`
Expected: FAIL — module not found.

- [ ] **Step 3: Write the implementation**

```ts
import { readStringArray } from '@/lib/tabs/tabView'
import type { TabViewSpec } from '@/lib/tabs/tabView'
import { RowGroup } from '@/lib/search/rows'

export interface SearchReading {
  picked: RowGroup[]
}

const groups = Object.values(RowGroup)

// The query is not here: it is in the location already, because a search is a place you can
// link to. What the tab remembers is which groups you narrowed it to.
export const searchView: TabViewSpec<SearchReading> = {
  empty: () => ({ picked: [] }),
  read: (value) => {
    if (typeof value !== 'object' || value === null) return null
    const picked = readStringArray(
      (value as { picked?: unknown }).picked,
      groups,
    )
    return picked === null ? null : { picked: picked as RowGroup[] }
  },
}
```

In `SearchScreen.vue`, `const picked = ref<RowGroup[]>([])` becomes a writable computed over
`useTabView(searchView)`, as in Task 5.

- [ ] **Step 4: Run the test and verify it passes**

Run: `pnpm --filter ui test -- search/tabView`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add ui/src/screens/search/ ui/src/screens/SearchScreen.vue
git commit -m "feat(ui): Search remembers which groups you narrowed it to"
```

---

### Task 9: the scroll offset, in the one component that owns a scroll box

**Files:**
- Create: `ui/src/lib/scale/scrollOffset.ts`, `ui/src/lib/scale/scrollOffset.test.ts`
- Modify: `ui/src/components/ui/virtual/VirtualRows.vue`
- Modify: the four screens' `tabView.ts` and the four templates

**Interfaces:**
- Consumes: Tasks 3 to 8.
- Produces: `interface ScrollOffset { top: number; rows: number }`;
  `readScrollOffset(value: unknown): ScrollOffset | null`;
  `offsetToApply(stored: ScrollOffset | null, rows: number): number | null`.
  `VirtualRows` gains `offset?: ScrollOffset | null` and emits `offsetChange: [ScrollOffset]`.

- [ ] **Step 1: Write the failing tests**

```ts
import { describe, expect, it } from 'vitest'
import { offsetToApply, readScrollOffset } from './scrollOffset'

describe('an offset against the list it was taken from', () => {
  it('is applied when the list is the length it was', () => {
    expect(offsetToApply({ top: 900, rows: 641 }, 641)).toBe(900)
  })

  // An offset only means something against a list of the same length. A filter that changed
  // underneath — a facet picked in another window, a profile reloaded, an archive that grew —
  // keeps the top rather than guessing where 900 pixels now point (B39).
  it('is refused when the list changed underneath', () => {
    expect(offsetToApply({ top: 900, rows: 641 }, 12)).toBeNull()
  })

  it('is nothing when nothing was stored', () => {
    expect(offsetToApply(null, 641)).toBeNull()
  })

  it('reads back what it wrote, and refuses what it did not', () => {
    expect(readScrollOffset({ top: 900, rows: 641 })).toEqual({
      top: 900,
      rows: 641,
    })
    expect(readScrollOffset({ top: '900', rows: 641 })).toBeNull()
    expect(readScrollOffset(null)).toBeNull()
  })

  // A negative or non-finite offset is not a position: it is a number that arrived from
  // somewhere it should not have.
  it('refuses an offset that is not a position', () => {
    expect(readScrollOffset({ top: -1, rows: 3 })).toBeNull()
    expect(readScrollOffset({ top: Number.NaN, rows: 3 })).toBeNull()
  })
})
```

- [ ] **Step 2: Run the tests and verify they fail**

Run: `pnpm --filter ui test -- scrollOffset`
Expected: FAIL — module not found.

- [ ] **Step 3: Write the pure module**

```ts
// Where a list was scrolled to, and the list it was measured against. The second number is what
// makes the first mean anything: an offset is a distance into a list of a given length, and the
// same distance into a different list is a different place.
export interface ScrollOffset {
  top: number
  rows: number
}

const isCount = (value: unknown): value is number =>
  typeof value === 'number' && Number.isFinite(value) && value >= 0

export const readScrollOffset = (value: unknown): ScrollOffset | null => {
  if (typeof value !== 'object' || value === null) return null
  const { top, rows } = value as { top?: unknown; rows?: unknown }
  return isCount(top) && isCount(rows) ? { top, rows } : null
}

// What to scroll to, or nothing at all. Nothing is the honest answer for a list that changed
// underneath: the top of a list you can read beats a position in a list that is not the one the
// number came from.
export const offsetToApply = (
  stored: ScrollOffset | null,
  rows: number,
): number | null =>
  stored !== null && stored.rows === rows ? stored.top : null
```

- [ ] **Step 4: Run the tests and verify they pass**

Run: `pnpm --filter ui test -- scrollOffset`
Expected: PASS.

- [ ] **Step 5: Wire it into `VirtualRows.vue`**

Add to the props and emits:

```ts
const props = defineProps<{
  rows: T[]
  rowPx: (percent: number) => number
  overscan?: number
  /** Where this list was left, and the list that number was measured against. */
  offset?: ScrollOffset | null
}>()

const emit = defineEmits<{ offsetChange: [ScrollOffset] }>()
```

and, after the virtualizer:

```ts
// Restored **after** the rows are there: an offset into an empty list scrolls nothing, and the
// data arrives one tick after the component. Once only — a later change of the stored offset is
// this component's own echo, not somebody moving the list.
let restored = false
watch(
  () => props.rows.length,
  async (rows) => {
    if (restored || rows === 0) return
    const top = offsetToApply(props.offset ?? null, rows)
    restored = true
    if (top === null) return
    await nextTick()
    if (scroller.value) scroller.value.scrollTop = top
  },
  { immediate: true },
)

const onScroll = useDebounceFn(() => {
  if (scroller.value)
    emit('offsetChange', {
      top: scroller.value.scrollTop,
      rows: props.rows.length,
    })
}, Timing.SearchDebounce)
```

and `@scroll="onScroll"` on the scroll box. The template gains no class and no style.

- [ ] **Step 6: Add the offset to the four readings**

Each of the four `tabView.ts` files gains `offset: ScrollOffset | null` in its reading, `null` in
`empty()`, and `readScrollOffset(...) ?? null` in `read`. Each screen binds
`:offset="reading.offset"` and `@offset-change="..."` on its `VirtualRows`.

Extend each screen's `tabView.test.ts` with one case:

```ts
  it('reads back where the list was scrolled to', () => {
    expect(
      unlockView.read({
        filter: unlockFaceting.empty(),
        sort: UnlockSort.FanOut,
        offset: { top: 900, rows: 641 },
      })?.offset,
    ).toEqual({ top: 900, rows: 641 })
  })
```

- [ ] **Step 7: Run the whole frontend suite**

Run: `pnpm ui:test && pnpm typecheck && pnpm scan`
Expected: all green.

- [ ] **Step 8: Commit**

```bash
git add ui/src/lib/scale/scrollOffset.ts ui/src/lib/scale/scrollOffset.test.ts ui/src/components/ui/virtual/VirtualRows.vue ui/src/screens/
git commit -m "feat(ui): a list comes back where it was, or at its top when it changed"
```

---

### Task 10: the strip scrolls, and the active tab is brought into view

**Files:**
- Modify: `ui/src/components/shell/TabStrip.vue`

**Interfaces:**
- Consumes: nothing from earlier tasks. Can be done at any point; it is last because it is the
  one piece whose verification is a window and not a test.

- [ ] **Step 1: Read what already exists before changing it**

`TabItem` carries `max-w-tab-max min-w-tab-min shrink grow-0 basis-tab-max`; `--spacing-tab-min`
and `--spacing-tab-max` are in `ui/src/assets/theme/spacing.css`; `--container-tab-narrow` is in
`containers.css`; the `tab-intrinsic` utility in `utilities.css` exists so a strip sized by its
tabs does not collapse every tab to the minimum. **The shrink is built.** This task adds the
scroll and nothing else, and adds no token.

- [ ] **Step 2: Make the strip scroll**

On the `role="tablist"` element, add `overflow-x-auto` and keep `min-w-0`. The outer flex row
keeps `min-w-0` so the strip can actually be narrower than its content.

- [ ] **Step 3: Bring the active tab into view**

```ts
// A tab that became active without being clicked — restored from a session, docked from
// another window, reached with the arrow keys — can be outside the scrolled strip. Being
// active and invisible is the one state a tab bar must not have.
watch(
  () => props.activeId,
  async () => {
    await nextTick()
    const index = props.tabs.findIndex((tab) => tab.id === props.activeId)
    tabElements()[index]?.scrollIntoView({ block: 'nearest', inline: 'nearest' })
  },
  { immediate: true },
)
```

- [ ] **Step 4: Check the tear-off's geometry still agrees with what is drawn**

`incomingGap` reads `getBoundingClientRect()`, which already accounts for the scroll, and
`useTabDrag` measures the same elements the same way. Read both and confirm no offset is
computed from an element's index or from the strip's left edge; if one is, it is a defect this
task introduces and it is fixed here, not left for the report.

- [ ] **Step 5: Run the suite**

Run: `pnpm ui:test && pnpm typecheck && pnpm scan && pnpm lint && pnpm format:check`
Expected: all green.

- [ ] **Step 6: Commit**

```bash
git add ui/src/components/shell/TabStrip.vue
git commit -m "feat(ui): the tab strip scrolls, and the active tab is never out of sight"
```

---

### Task 11: the whole check, and the half only a window says

**Files:** none.

- [ ] **Step 1: Run the full check from the worktree root**

Run: `pnpm check`
Expected: green. Read the skip count as `scripts/check` reports it — a suite that passes while
every test on real data skipped is the failure mode this repo names by hand.

- [ ] **Step 2: Open the app and look at it**

Run: `pnpm dev` (which clears a tray build and an orphaned vite on 1420 by itself).

Check, and write down what was seen — including anything that was **not** checked:

1. Filter Unlock, drag the tab into a new window: it arrives filtered, sorted the same, scrolled
   where it was.
2. Go back, then forward: the facets of each entry come back with it.
3. Close the app with several tabs open; reopen: the tabs are there, each showing what it was
   showing. Go back in a restored tab: the screen's empty state, which is what §6 of the spec
   says it buys.
4. Open twenty tabs: they shrink, then the strip scrolls; the active one is always visible;
   a tab can still be torn off from a scrolled strip.
5. On `?catalog=none` and `?fixture=none`, a restored tab still opens and says what it has.

- [ ] **Step 3: Write the report**

`docs/superpowers/reports/2026-09-15-tabs-own-their-state-report.md`: what was measured, what
was corrected in the spec while executing, and what was **not** seen in a window. A negative
result is a result; silence is not one.

- [ ] **Step 4: Update the state documents**

`docs/STATUS.md`: 3.7a under M1, with its spec, plan and report. `docs/BACKLOG.md`: B39 closes
with a pointer to the report; B6 keeps only what 3.7b and 3.7c still owe it. A ticked box means
committed work.

- [ ] **Step 5: Commit, then merge**

```bash
git add docs/
git commit -m "docs: 3.7a closes — a tab keeps how it was being read"
```

Then `superpowers:finishing-a-development-branch`: merge into `develop` with `--no-ff`, suite
green **on the merge result**, and only then move this plan to
`docs/superpowers/plans/archive/`.

---

## Self-review

**Spec coverage.** §3 — Tasks 1 and 4. §4's validator and "a bad record falls alone" — Tasks 2
and 3, with the screens in 5 to 8. §4's scroll — Task 9. §7 — Task 10. §6's pruning — Task 2
(the cap is 3.7b's; the pruning that keeps it out of reach is here). §9's declined Floor — no
task, by design, and `stores/floor.ts` is untouched. §11's item 1 — the whole plan.

**Not in this plan, and on purpose:** the session's `version: 2`, the windows, the ledger, the
elected writer, the monitor clamp and the `SessionTooLarge` diagnostic are 3.7b; the sidebar and
table sizes are 3.7c. Task 2 therefore leaves `Version` at 1, which is correct: an entry gaining
an optional key is a part an older app can ignore.

**Checked while reviewing, so the executor does not have to:** `collectionFacetOrder` is
exported already (`collectionFacets.ts:22`); `Timing` holds one value and gains a second in
Task 4; `RunsTable.vue:14-17` is where a run's key lives inline today, with the comment Task 7
moves along with it; `VirtualRows.vue`'s snippet in Task 9 needs `watch`, `nextTick`,
`useDebounceFn`, `Timing`, `offsetToApply` and the `ScrollOffset` type added to its imports.

**The one thing the executor must not decide silently:** if a test written from this plan fails,
it is first of all a hypothesis of a bug in the code. The expected values here come from the
spec. Changing one to match what the code printed is the move this repo has an incident behind —
if a value in this plan is wrong, say which and why in the report.
