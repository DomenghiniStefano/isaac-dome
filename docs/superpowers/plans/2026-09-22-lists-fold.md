# 3.13b — the three tables fold

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Collection, Challenges and Runs drop their explaining columns below
`--container-compact`, the way Unlock already does, so every list screen is readable at the
640px window floor.

**Architecture:** No new mechanism. 3.13a §7 settled how a column drops — a pair of adjacent
`@utility` declarations plus `@max-compact/page:hidden` on the cells that fall, in the header
and in the row — and migrated Unlock as the proof. This cycle applies it three more times and
adds the one property a stylesheet cannot state about itself: that the narrow template is
genuinely narrower than its full counterpart.

**Tech Stack:** Vue 3 + TypeScript, Tailwind v4 container queries, Vitest,
`ui/scripts/scan-conventions.mjs`.

**Spec:** `docs/superpowers/specs/2026-09-20-responsive-layout-design.md` — §7 (how a column
drops), §9 rule 4 (what gates it), §11 (how it is verified), §12 (what the migration must
measure and write down). This plan is that spec's §12 answered for three more tables.

## Global Constraints

- **The budget is 428px.** 640 window floor (§8) − 168 sidebar at its minimum − 44 of the page
  box's horizontal padding. Every narrow set below was chosen against it, and
  `ui/src/lib/design/thresholds.test.ts` already pins the same arithmetic.
- **Two edits are one change.** A template that drops a track while its cell stays slides every
  cell after it into the wrong column: it reads as a styling bug and is a counting one. Header
  and row are edited in the same commit, never apart.
- **The variant always names its container**: `@max-compact/page:`, never `@max-compact:`
  (3.13a §5).
- **No hardcoded visual constants** (`CLAUDE.md`, frontend rule 2): a width is a token in
  `@theme`, a grid template is a `@utility`. Nothing here introduces a raw px outside the Kit
  page's own box widths, which already exist.
- **No `<style>` in SFCs** (frontend rule 1).
- **Commits**: Conventional Commits, English, `ui` scope, atomic, no co-author trailer.
- The branch is `feature/lists-fold`, in the `C:\Projects\isaac-dome-tabs` worktree. The main
  worktree holds `develop` and another session is working in it — nothing in this plan touches
  it until the merge.

---

### Task 1: The folding tables are named, and the property is held

The three pairs to come need something that fails when a "narrow" template is not narrower, or
when a pair is half-declared. `lib/design/thresholds.ts` is the precedent: the number lives in
the CSS where the layout reads it, and in TypeScript where anything that has to name it reads
it. This does the same for track counts.

**Files:**
- Create: `ui/src/lib/design/tables.ts`
- Create: `ui/src/lib/design/tables.test.ts`

**Interfaces:**
- Consumes: nothing.
- Produces: `FoldingTable` (const object + type, values `'unlock' | 'collection' | 'challenges'
  | 'runs'` once every task has run) and
  `TableTracks: Record<FoldingTable, { full: number; narrow: number }>`. Tasks 2, 3 and 4 each
  add one entry to both before touching the CSS.

- [ ] **Step 1: Write the failing test**

Create `ui/src/lib/design/tables.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import utilities from '@/assets/utilities.css?raw'
import { FoldingTable, TableTracks } from './tables'

// The body of one `@utility grid-cols-<name>` declaration. `grid-cols-collection` and
// `grid-cols-collection-narrow` are told apart by the space before the brace: the narrow one
// has `-narrow` where this pattern wants that space.
const template = (name: string): string => {
  const found = utilities.match(
    new RegExp(String.raw`@utility grid-cols-${name} \{([^}]*)\}`),
  )
  if (found === null) throw new Error(`no @utility grid-cols-${name}`)
  return found[1]
}

// Every track in these templates is either a token or a `minmax()`. Counting them is the whole
// point: a narrow set that is not shorter than its full set is a pair that was never made.
const tracks = (body: string): number =>
  (body.match(/var\(--[^)]*\)|minmax\([^)]*\)/g) ?? []).length

// The pattern of `thresholds.test.ts`: the decision lives in the CSS, where the browser reads
// it, and in TypeScript, where a reader and a test can name it. If the two drift, a column
// leaves a template while its cell stays — which draws as a styling bug and is a counting one
// (spec 3.13a §7).
describe('a folding table', () => {
  it('declares both templates', () => {
    for (const name of Object.values(FoldingTable)) {
      expect(() => template(name)).not.toThrow()
      expect(() => template(`${name}-narrow`)).not.toThrow()
    }
  })

  it('declares the number of tracks the record names', () => {
    for (const name of Object.values(FoldingTable)) {
      expect(tracks(template(name))).toBe(TableTracks[name].full)
      expect(tracks(template(`${name}-narrow`))).toBe(TableTracks[name].narrow)
    }
  })

  it('is narrower when narrow, which is the whole claim', () => {
    for (const name of Object.values(FoldingTable))
      expect(TableTracks[name].narrow).toBeLessThan(TableTracks[name].full)
  })
})
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cd C:\Projects\isaac-dome-tabs && pnpm ui:test -- tables`
Expected: FAIL — `Failed to resolve import "./tables"`.

- [ ] **Step 3: Write the module**

Create `ui/src/lib/design/tables.ts`:

```ts
// The tables that fold below `--container-compact` (spec 3.13a §7). Each is a pair of
// `@utility` declarations in `assets/utilities.css`: the full template, and the narrow one the
// header and the rows take under the threshold.
export const FoldingTable = {
  Unlock: 'unlock',
} as const
export type FoldingTable = (typeof FoldingTable)[keyof typeof FoldingTable]

// How many tracks each template declares, kept here for the reason `thresholds.ts` keeps the
// three widths: so a test can hold what a stylesheet cannot state about itself — that the
// narrow set is genuinely shorter — and so a reader finds the pairs without grepping CSS.
//
// **What survives a fold was decided in 3.13b**: who the row is, how it is doing, and the
// button that acts on it. What falls is what explains the row, and what is derived from it.
export const TableTracks: Record<
  FoldingTable,
  { full: number; narrow: number }
> = {
  [FoldingTable.Unlock]: { full: 7, narrow: 4 },
}
```

- [ ] **Step 4: Run it and watch it pass**

Run: `cd C:\Projects\isaac-dome-tabs && pnpm ui:test -- tables`
Expected: PASS, 3 tests.

- [ ] **Step 5: Commit**

```bash
cd C:\Projects\isaac-dome-tabs
git add ui/src/lib/design/tables.ts ui/src/lib/design/tables.test.ts
git commit -m "test(ui): a narrow grid template has to be narrower than its own table"
```

---

### Task 2: The Collection keeps the sprite, the name and the state

**What falls and why.** Quality, pools and origin. Pools is the widest and says least when
truncated; origin is the least urgent thing about an item; quality is the closest call and it
loses on arithmetic — keeping its 96px would leave the item name 140px at the floor, and most
item names truncate there. What is left spends 192 of the 428: sprite 56 + state 136, the name
taking the rest.

**Files:**
- Modify: `ui/src/lib/design/tables.ts`
- Modify: `ui/src/assets/utilities.css` (after the `grid-cols-collection` block, at line 74)
- Modify: `ui/src/screens/collection/CollectionTable.vue` (the header, and the row's `cn()`)
- Modify: `ui/src/screens/collection/CollectionRow.vue` (the quality, pools and origin cells)
- Test: `ui/src/lib/design/tables.test.ts` — no edit, it reads the record

**Interfaces:**
- Consumes: `FoldingTable`, `TableTracks` from Task 1.
- Produces: `@utility grid-cols-collection-narrow`.

- [ ] **Step 1: Add the entry, which makes the test fail**

In `ui/src/lib/design/tables.ts`, one line in each of the two declarations:

```ts
export const FoldingTable = {
  Unlock: 'unlock',
  Collection: 'collection',
} as const
```

```ts
  [FoldingTable.Collection]: { full: 6, narrow: 3 },
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cd C:\Projects\isaac-dome-tabs && pnpm ui:test -- tables`
Expected: FAIL — `no @utility grid-cols-collection-narrow`.

- [ ] **Step 3: Declare the narrow template**

In `ui/src/assets/utilities.css`, immediately after the `grid-cols-collection` block:

```css
/* The Collection at compact (spec 3.13a §7): the sprite, the name, the state. What falls is the
   quality, the pools and the origin — the pools because they are the widest and say least when
   truncated, the origin because it is the least urgent thing about an item, and the quality on
   arithmetic: its 96px would leave the name 140 of the 428 the window floor gives, and most item
   names truncate there. Each of the three carries `@max-compact/page:hidden` on its cell, in
   `CollectionTable.vue` for the header and `CollectionRow.vue` for the row. */
@utility grid-cols-collection-narrow {
  grid-template-columns:
    var(--spacing-collection-sprite)
    minmax(0, 1fr)
    var(--spacing-collection-state);
}
```

- [ ] **Step 4: Run it and watch it pass**

Run: `cd C:\Projects\isaac-dome-tabs && pnpm ui:test -- tables`
Expected: PASS, 3 tests.

- [ ] **Step 5: Hide the three header cells and switch the template**

In `ui/src/screens/collection/CollectionTable.vue`, the header `div` takes the narrow template
and three of its six spans take the variant:

```html
    <div
      class="grid grid-cols-collection items-center border-b border-hairline bg-muted text-label text-subtle-foreground @max-compact/page:grid-cols-collection-narrow"
    >
      <span />
      <span class="px-2 py-1.5">{{ t('collection.columns.item') }}</span>
      <span class="px-2 py-1.5 @max-compact/page:hidden">{{
        t('collection.columns.quality')
      }}</span>
      <span class="px-2 py-1.5 @max-compact/page:hidden">{{
        t('collection.columns.pools')
      }}</span>
      <span class="px-2 py-1.5 @max-compact/page:hidden">{{
        t('collection.columns.origin')
      }}</span>
      <span class="px-2 py-1.5">{{ t('collection.columns.state') }}</span>
    </div>
```

And the row `div` inside `VirtualRows`, in the same file: the first string of its `cn()` list,
which today reads

```
'absolute inset-x-0 top-0 grid h-row-wide translate-y-(--row-start) grid-cols-collection items-center border-b border-hairline hover:bg-row-hover'
```

becomes

```
'absolute inset-x-0 top-0 grid h-row-wide translate-y-(--row-start) grid-cols-collection items-center border-b border-hairline @max-compact/page:grid-cols-collection-narrow hover:bg-row-hover'
```

- [ ] **Step 6: Hide the three row cells**

In `ui/src/screens/collection/CollectionRow.vue`, three of the six cells take the variant. The
quality cell:

```html
  <span class="px-2 @max-compact/page:hidden">
    <QualityPips :quality="item.quality" />
  </span>
```

The pools cell — only its opening tag changes:

```html
  <span class="flex min-w-0 items-center gap-2 px-2 @max-compact/page:hidden">
```

The origin cell:

```html
  <span
    class="truncate px-2 text-caption text-foreground-soft @max-compact/page:hidden"
    >{{ origin ?? '—' }}</span
  >
```

- [ ] **Step 7: Run the gates**

Run: `cd C:\Projects\isaac-dome-tabs && pnpm scan && pnpm ui:test -- tables && pnpm typecheck`
Expected: PASS. `scan`'s rule 4 is what proves both edits were made — a narrow template in a
file with no `@max-compact/page:hidden` is half the work.

- [ ] **Step 8: Commit**

```bash
cd C:\Projects\isaac-dome-tabs
git add ui/src/lib/design/tables.ts ui/src/assets/utilities.css ui/src/screens/collection/CollectionTable.vue ui/src/screens/collection/CollectionRow.vue
git commit -m "feat(ui): the Collection keeps the sprite, the name and the state when narrow"
```

---

### Task 3: Challenges keeps the number, the name, the state and the button

**What falls and why.** The character it forces and the goal — the two flexible columns, which
are what *explains* a challenge rather than what names it. The queue button stays: a row that
loses its action at the width where it is hardest to reach is a row that does nothing. What is
left spends 224 of the 428: number 48 + state 136 + queue 40.

**Files:**
- Modify: `ui/src/lib/design/tables.ts`
- Modify: `ui/src/assets/utilities.css` (after the `grid-cols-challenges` block)
- Modify: `ui/src/screens/challenges/ChallengesTable.vue` (the header, and the row's `cn()`)
- Modify: `ui/src/screens/challenges/ChallengeRow.vue` (the character and goal cells)

**Interfaces:**
- Consumes: `FoldingTable`, `TableTracks` from Task 1.
- Produces: `@utility grid-cols-challenges-narrow`.

- [ ] **Step 1: Add the entry, which makes the test fail**

```ts
  Challenges: 'challenges',
```

```ts
  [FoldingTable.Challenges]: { full: 6, narrow: 4 },
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cd C:\Projects\isaac-dome-tabs && pnpm ui:test -- tables`
Expected: FAIL — `no @utility grid-cols-challenges-narrow`.

- [ ] **Step 3: Declare the narrow template**

```css
/* Challenges at compact (spec 3.13a §7): the number, the name, the state, and the button that
   puts its reward in the queue. What falls is the character it forces and the goal — the two
   that explain a challenge rather than name it, and the two widest. The button stays: a row
   that loses its action at the width where it is hardest to reach is a row that does nothing.
   Both carry `@max-compact/page:hidden`, in `ChallengesTable.vue` for the header and
   `ChallengeRow.vue` for the row. */
@utility grid-cols-challenges-narrow {
  grid-template-columns:
    var(--spacing-challenge-number)
    minmax(0, 1fr)
    var(--spacing-challenge-state)
    var(--spacing-challenge-queue);
}
```

- [ ] **Step 4: Run it and watch it pass**

Run: `cd C:\Projects\isaac-dome-tabs && pnpm ui:test -- tables`
Expected: PASS, 3 tests.

- [ ] **Step 5: Hide the two header cells and switch the template**

In `ui/src/screens/challenges/ChallengesTable.vue`, the header `div` gains
`@max-compact/page:grid-cols-challenges-narrow` at the end of its class list, and two spans
take the variant:

```html
      <span class="px-2 py-1.5 @max-compact/page:hidden">{{
        t('challenges.columns.character')
      }}</span>
      <span class="px-2 py-1.5 @max-compact/page:hidden">{{
        t('challenges.columns.goal')
      }}</span>
```

The row `div` in the same file — the `v-for` over `rows` — gains
`@max-compact/page:grid-cols-challenges-narrow` beside `grid-cols-challenges` in its `cn()`
list.

- [ ] **Step 6: Hide the two row cells**

In `ui/src/screens/challenges/ChallengeRow.vue`, the character cell:

```html
  <span
    class="min-w-0 truncate px-2 text-caption text-foreground @max-compact/page:hidden"
  >
```

and the goal cell:

```html
  <span class="flex min-w-0 items-center gap-1.5 px-2 @max-compact/page:hidden">
```

Leave the comment above the character cell where it is — it is about the game, not the layout.

- [ ] **Step 7: Run the gates**

Run: `cd C:\Projects\isaac-dome-tabs && pnpm scan && pnpm ui:test -- tables && pnpm typecheck`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
cd C:\Projects\isaac-dome-tabs
git add ui/src/lib/design/tables.ts ui/src/assets/utilities.css ui/src/screens/challenges/ChallengesTable.vue ui/src/screens/challenges/ChallengeRow.vue
git commit -m "feat(ui): a narrow Challenges keeps the row's name and the button that queues it"
```

---

### Task 4: The Run diary keeps the character, the outcome and the floors

**What falls and why.** The seed and where the run came from. The seed is the widest text on the
row and the least scannable; the source is diagnostic — it answers "which log said so", which is
not the question a diary is read with. Floors stays at 64px because how far you got is half of
what a run is. What is left spends 64 of the 428 on a fixed track and gives the rest to the
character and the outcome.

**Files:**
- Modify: `ui/src/lib/design/tables.ts`
- Modify: `ui/src/assets/utilities.css` (after the `grid-cols-runs` block)
- Modify: `ui/src/screens/runs/RunsTable.vue` (the header, and the row `Button`'s `cn()`)
- Modify: `ui/src/screens/runs/RunRow.vue` (the seed and source cells)

**Interfaces:**
- Consumes: `FoldingTable`, `TableTracks` from Task 1.
- Produces: `@utility grid-cols-runs-narrow`.

- [ ] **Step 1: Add the entry, which makes the test fail**

```ts
  Runs: 'runs',
```

```ts
  [FoldingTable.Runs]: { full: 5, narrow: 3 },
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cd C:\Projects\isaac-dome-tabs && pnpm ui:test -- tables`
Expected: FAIL — `no @utility grid-cols-runs-narrow`.

- [ ] **Step 3: Declare the narrow template**

```css
/* The Run diary at compact (spec 3.13a §7): the character, how it ended, the floors. What falls
   is the seed — the widest text on the row and the least scannable — and where the run came
   from, which answers "which log said so" and is not the question a diary is read with. The
   floors stay: how far you got is half of what a run is, and the track is 64px. Both carry
   `@max-compact/page:hidden`, in `RunsTable.vue` for the header and `RunRow.vue` for the row. */
@utility grid-cols-runs-narrow {
  grid-template-columns:
    minmax(0, 1fr)
    minmax(0, 1.4fr)
    var(--spacing-runs-floors);
}
```

- [ ] **Step 4: Run it and watch it pass**

Run: `cd C:\Projects\isaac-dome-tabs && pnpm ui:test -- tables`
Expected: PASS, 3 tests.

- [ ] **Step 5: Hide the two header cells and switch the template**

In `ui/src/screens/runs/RunsTable.vue`, the header `div` gains
`@max-compact/page:grid-cols-runs-narrow`, and two of its five spans take the variant:

```html
      <span class="px-2 py-1.5 @max-compact/page:hidden">{{
        t('runs.column.seed')
      }}</span>
      <span class="px-2 py-1.5 @max-compact/page:hidden">{{
        t('runs.column.source')
      }}</span>
```

The row `Button` in the same file gains `@max-compact/page:grid-cols-runs-narrow` beside
`grid-cols-runs` in its `cn()` list.

- [ ] **Step 6: Hide the two row cells**

In `ui/src/screens/runs/RunRow.vue`, the seed cell:

```html
  <span
    class="truncate px-2 text-label text-subtle-foreground @max-compact/page:hidden"
    >{{ run.seedWords }}</span
  >
```

and the source cell — only its opening tag changes:

```html
  <span class="flex min-w-0 items-center gap-2 px-2 @max-compact/page:hidden">
```

- [ ] **Step 7: Run the gates**

Run: `cd C:\Projects\isaac-dome-tabs && pnpm scan && pnpm ui:test -- tables && pnpm typecheck`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
cd C:\Projects\isaac-dome-tabs
git add ui/src/lib/design/tables.ts ui/src/assets/utilities.css ui/src/screens/runs/RunsTable.vue ui/src/screens/runs/RunRow.vue
git commit -m "feat(ui): a narrow run diary keeps who played, how it ended and how far"
```

---

### Task 5: The Kit shows all four tables at the three widths

3.13a's `WidthsSection.vue` puts `UnlockTable` in three `@container/page` boxes — 500, 900 and
1300 — so a compact layout is judged without resizing the window (§11). Three more tables want
the same three boxes, so the boxes become a component and the section uses it four times.

**Files:**
- Create: `ui/src/kit/KitWidths.vue`
- Modify: `ui/src/kit/sections/app/WidthsSection.vue`

**Interfaces:**
- Consumes: the four table components, and `CollectionItem`, `ChallengeRow`, `RunView`,
  `ItemKindView`, `OriginView` from `@/lib/ipc/types`.
- Produces: `KitWidths`, a component rendering its default slot once inside each of three
  `@container/page` boxes. 3.13c reuses it.

- [ ] **Step 1: Extract the three boxes**

Create `ui/src/kit/KitWidths.vue`, moving the comment and the widths out of `WidthsSection.vue`
unchanged:

```vue
<script setup lang="ts">
// Three boxes, each its own `page` container, so a compact layout can be judged without
// resizing the window — the thing container queries buy and media queries could not (spec
// 3.13a §11). The widths straddle all three thresholds: 500 is below compact (800), 900 is past
// it and below regular (960), and 1300 is past wide (1280).
const widths = [500, 900, 1300]
</script>

<template>
  <div class="flex items-start gap-4 overflow-x-auto">
    <div
      v-for="width in widths"
      :key="width"
      :style="{ '--kit-width': `${width}px` }"
      class="@container/page flex h-80 w-(--kit-width) shrink-0 flex-col border border-border"
    >
      <slot />
    </div>
  </div>
</template>
```

- [ ] **Step 2: Give the section the other three tables**

In `ui/src/kit/sections/app/WidthsSection.vue`, keep the existing `node()` helper and `nodes`
fixture, drop the `widths` constant and its comment (they moved), and add three fixtures:

```ts
const items: CollectionItem[] = [
  {
    id: 1,
    kind: ItemKindView.Passive,
    name: 'Il Cubo di Ghiaccio',
    iconUrl: null,
    quality: 3,
    pools: ['Tesoro', 'Negozio'],
    origin: OriginView.Repentance,
    inCollection: true,
    lock: { kind: 'free' },
  },
  {
    id: 2,
    kind: ItemKindView.Active,
    name: 'Un nome lungo che deve troncarsi quando la colonna si stringe',
    iconUrl: null,
    quality: 0,
    pools: [],
    origin: null,
    inCollection: false,
    lock: { kind: 'free' },
  },
]

const challenges: ChallengeRow[] = [
  {
    number: 1,
    name: 'Pitch Black',
    state: { kind: 'done' },
    rewards: [],
    character: null,
    characterName: 'Isaac',
    goal: null,
    blindfolded: false,
    page: null,
  },
  {
    number: 2,
    name: 'Una sfida dal nome lungo che deve troncarsi quando si stringe',
    state: { kind: 'available' },
    rewards: [],
    character: null,
    characterName: null,
    goal: null,
    blindfolded: null,
    page: null,
  },
]

const runs: RunView[] = [
  {
    source: { kind: 'live' },
    ordinal: 1,
    character: 'Isaac',
    characterId: 0,
    seedWords: 'ABCD 1234',
    online: false,
    outcome: { kind: 'won', ending: 'Mother' },
    floors: 11,
    startingItems: [],
    collected: [],
    heldActive: null,
    achievements: [],
  },
  {
    source: { kind: 'session', name: '2026-09-22' },
    ordinal: 2,
    character: null,
    characterId: null,
    seedWords: 'WXYZ 9876',
    online: true,
    outcome: { kind: 'died', killer: 'Mom' },
    floors: 4,
    startingItems: [],
    collected: [],
    heldActive: null,
    achievements: [],
  },
]
```

Imports to add: `CollectionItem`, `ChallengeRow`, `RunView` as types and `ItemKindView`,
`OriginView` as values from `@/lib/ipc/types`; `CollectionTable` from
`@/screens/collection/CollectionTable.vue`, `ChallengesTable` from
`@/screens/challenges/ChallengesTable.vue`, `RunsTable` from `@/screens/runs/RunsTable.vue`;
`KitWidths` from `../../KitWidths.vue`.

The template becomes four `KitWidths`, one per table:

```html
<template>
  <KitSection title="Larghezze" class="col-span-3">
    <div class="flex flex-col gap-6">
      <KitWidths>
        <UnlockTable
          :nodes="nodes"
          :queued="queued"
          :can-write="false"
          :busy="false"
          :offset="null"
        />
      </KitWidths>
      <KitWidths>
        <CollectionTable
          :items="items"
          :offset="null"
          find-query=""
          :find-current="null"
        />
      </KitWidths>
      <KitWidths>
        <ChallengesTable
          :rows="challenges"
          :queued="[]"
          :can-write="false"
          :busy="false"
        />
      </KitWidths>
      <KitWidths>
        <RunsTable :runs="runs" :selected="null" :offset="null" />
      </KitWidths>
    </div>
  </KitSection>
</template>
```

- [ ] **Step 3: Look at it**

Run: `cd C:\Projects\isaac-dome-tabs && pnpm ui:dev`, open `http://localhost:1420/#kit`, find
"Larghezze". In the 500 box of each of the four tables: the dropped columns are gone, the
remaining cells sit under their own headers, and nothing overflows. This is the check none of
the gates replaces.

Do **not** run `pnpm dev` — its `predev` evicts whoever holds 1420, which may be another
session's server (`CLAUDE.md`). `pnpm ui:dev` is the browser-only one.

- [ ] **Step 4: Run the gates**

Run: `cd C:\Projects\isaac-dome-tabs && pnpm typecheck && pnpm scan && pnpm lint`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
cd C:\Projects\isaac-dome-tabs
git add ui/src/kit/KitWidths.vue ui/src/kit/sections/app/WidthsSection.vue
git commit -m "test(ui): the Kit shows all four folding tables at the three widths"
```

---

### Task 6: Search and the wiki's lists are looked at, and nothing folds

The card promises five screens. Two of them have no columns: `SearchResults` and
`WikiCategoryList` are flex rows that already truncate (`min-w-0 flex-1 truncate`), and every
toolbar around them — `FilterBar`, `SearchToolbar`, the wiki hero — is `flex-wrap` and folds by
itself. Runs' KPI strip already carries `@regular/page:grid-cols-4`. Inventing a fold there
would be a state nobody needs; the honest deliverable is the finding, written where the next
person reads it.

**Files:**
- Modify: `docs/superpowers/specs/2026-09-20-responsive-layout-design.md` (§12)

- [ ] **Step 1: Record the answers**

At the end of §12, add:

```markdown
**3.13b's answers** (2026-09-22), for the three tables that had columns to lose:

| table | narrow set | what falls | fixed px of the 428 |
|---|---|---|---|
| Collection | sprite, name, state | quality, pools, origin | 192 |
| Challenges | number, name, state, queue | character, goal | 224 |
| Runs | character, outcome, floors | seed, source | 64 |

**And two screens that fold nothing.** `SearchResults` and `WikiCategoryList` are not grids:
their rows are flex with `min-w-0 flex-1 truncate`, and the toolbars around them — `FilterBar`,
`SearchToolbar`, the wiki hero — are `flex-wrap`. They were examined and left alone, which is a
finding and not an omission.

**A trap beside them**, because the next person will walk into it: `SearchRow.vue` is rendered
both in the Search screen and inside `SearchPalette`, which is an overlay outside `<main>`. A
`@max-compact/page:` written in that file measures a container the palette does not have, so
there it would silently never match. A fold in a shared row needs its own container, or it needs
to stay out.

**All three tables fold slightly before they have to** — full, they need ~710–750px against an
800px `compact`. That is §5's shared scale working as decided: three sizes and not seven, at the
price of a table or two folding early.
```

- [ ] **Step 2: Check the document references**

Run: `cd C:\Projects\isaac-dome-tabs && node scripts/check-doc-refs.mjs`
Expected: a report with no `NEW` line for the paths this task names. It never fails the run; the
signal is the delta.

- [ ] **Step 3: Commit**

```bash
cd C:\Projects\isaac-dome-tabs
git add docs/superpowers/specs/2026-09-20-responsive-layout-design.md
git commit -m "docs: what the three tables drop, and the two screens that drop nothing"
```

---

### Task 7: The floor, the full check, and the board

**Files:**
- Modify: `scripts/test-floor`
- Modify: `docs/frontend-conventions.md` (the responsive section 3.13a added)

- [ ] **Step 1: Run the whole thing**

Run: `cd C:\Projects\isaac-dome-tabs && pnpm check`
Expected: every gate green except the test-count floor, which prints the line to paste because
the suite grew by Task 1's three tests.

One number to read while it runs: this worktree's `samples/` is a junction to the main
worktree's, so the real-data tests must actually run. If `ISAACDOME_TEST_DECLARATIONS` reports
zero `sample:` lines, stop — the junction is gone and the suite is green on nothing
(`CLAUDE.md`, measured 2026-09-15).

- [ ] **Step 2: Raise the floor**

Paste the line `pnpm check` printed into `scripts/test-floor`.

- [ ] **Step 3: Name the pairs in the conventions**

In `docs/frontend-conventions.md`, under the responsive section's rule about a column dropping
by two edits, add one line: the pairs that exist — `unlock`, `collection`, `challenges`, `runs`
— are listed in `ui/src/lib/design/tables.ts`, and a new one is added there first, because that
record is what makes a half-declared pair fail.

- [ ] **Step 4: Run the whole thing again**

Run: `cd C:\Projects\isaac-dome-tabs && pnpm check`
Expected: all green.

- [ ] **Step 5: Commit**

```bash
cd C:\Projects\isaac-dome-tabs
git add scripts/test-floor docs/frontend-conventions.md
git commit -m "chore: the test floor rises by 3.13b's three UI tests"
```

- [ ] **Step 6: The board**

Card 3.13b is `ytUIMZ2C`. Tick what is really done, move it to `UAT` with the `NEEDS WINDOW`
label, and comment what is still unverified: the three narrow layouts have been seen on the Kit
page at 500px, and **nobody has dragged a real window edge**. What would settle it is one build,
one window, and the four list screens at the 640 floor.

---

## Merge

From the **main worktree**, `C:\Projects\isaac-dome`, which holds `develop` — and only once the
other session's uncommitted work there is settled:

```bash
cd C:\Projects\isaac-dome
git merge --no-ff feature/lists-fold -m "merge: the three tables fold when they are narrow, into develop"
git push origin develop
git branch -d feature/lists-fold
```

Then park `C:\Projects\isaac-dome-tabs` back on a detached `HEAD` at `develop`, the way it was
found:

```bash
cd C:\Projects\isaac-dome-tabs
git checkout --detach develop
```

`docs/architecture.md` is **not** redrawn: no crate, route, command, event or migration moves in
this cycle. Recorded here rather than left to be inferred.
