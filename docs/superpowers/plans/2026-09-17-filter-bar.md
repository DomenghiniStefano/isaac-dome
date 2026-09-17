# 3.10 — one filter bar for three lists: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this
> plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** the facet drawer and the filter toolbar become one filter bar — state row, search, the
two controls that matter, everything else behind a fold — on the Collection, Unlock and Runs.

**Architecture:** a screen hands `FilterBar.vue` a description of its facets (order, title, in
view or folded) and the bar draws every control. The judgment lives in a pure module
(`lib/facets/facetOptions.ts`) with Vitest around it; a new kit primitive `MultiSelect` replaces
the columns of checkboxes and is built on the kit's own `Command`.

**Tech Stack:** Vue 3 + TypeScript, Tailwind v4, shadcn-vue on Reka UI 2.10.4, vue-i18n, Vitest.

**Spec:** `docs/superpowers/specs/2026-09-17-filter-bar-design.md`

## Global Constraints

Every task inherits these. They are the repo's, not this plan's — `CLAUDE.md` and
`docs/frontend-conventions.md`.

- **No `<style>` in SFCs.** Dynamic values come from CSS variables bound by the template.
- **No hardcoded visual constants**: no `w-[48px]`, no `opacity-50`, no `:size="16"` on an icon.
  Every visual value is a token in `@theme`. A *behaviour* constant (the search threshold) is a
  named export in its own module, which is not the same thing.
- **No `invoke()` in components**, nothing outside `lib/window/` imports Tauri's window APIs.
  Nothing in this plan touches either.
- **No raw `<button>` / `<input>`**: use `ui/button`, `ui/input`, extend with a prop.
- **No string unions**: `const X = { … } as const`, never `type X = 'a' | 'b'`.
- **No visible string in a template**: every word goes through `t()`. `pnpm scan` checks this.
- **Test-first.** The expected value comes from the spec, never from the code's current output.
- **Commits**: Conventional Commits, `type(scope): subject`, scope `ui`, English, lowercase after
  the colon, no trailing period. **Never** a `Co-Authored-By` trailer or a reference to Claude.
- **Branch**: `feature/filter-bar`, already cut from `develop`. Nothing merges to `master`.
- **`docs/architecture.md` is NOT redrawn by this work.** Its five pinned counts are crates,
  commands, events, routes and store migrations; this sub-project changes none of them. Say so
  rather than wondering.

---

### Task 1: the pure half — options, and when the fold starts open

**Files:**
- Create: `ui/src/lib/facets/facetOptions.ts`
- Create: `ui/src/lib/facets/facetOptions.test.ts`

**Interfaces:**
- Consumes: `Faceting`, `FacetFilter` from `@/lib/facets/faceting` (unchanged).
- Produces:
  - `interface FacetOption { value: string; label: string; count: number; picked: boolean }`
  - `interface FacetSlot<Facet extends string> { facet: Facet; inView: boolean }`
  - `facetOptions<Row, Facet extends string>(faceting, rows, filter, facet, valueLabel): FacetOption[]`
  - `foldStartsOpen<Facet extends string>(slots: FacetSlot<Facet>[], filter: FacetFilter<Facet>): boolean`

- [ ] **Step 1: Write the failing tests**

Create `ui/src/lib/facets/facetOptions.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { createFaceting, emptyFilter } from './faceting'
import { facetOptions, foldStartsOpen } from './facetOptions'
import type { FacetSlot } from './facetOptions'

// Two facets over three rows, which is the smallest shape that has a value nobody can reach:
// `kind: 'trinket'` exists only on a row the pool filter is hiding.
interface Row {
  name: string
  pool: string
  kind: string
}

const Facet = { Pool: 'pool', Kind: 'kind' } as const
type Facet = (typeof Facet)[keyof typeof Facet]

const rows: Row[] = [
  { name: 'sad onion', pool: 'treasure', kind: 'passive' },
  { name: 'the inner eye', pool: 'treasure', kind: 'passive' },
  { name: 'swallowed penny', pool: 'shop', kind: 'trinket' },
]

const faceting = createFaceting<Row, Facet>({
  order: [Facet.Pool, Facet.Kind],
  values: (row, facet) => [row[facet]],
  text: (row) => row.name,
  options: (all, facet) => [...new Set(all.map((row) => row[facet]))],
})

const label = (_facet: Facet, value: string) => `«${value}»`
const empty = () => emptyFilter<Facet>([Facet.Pool, Facet.Kind])

describe('facetOptions', () => {
  it('gives every value its count and its words', () => {
    expect(facetOptions(faceting, rows, empty(), Facet.Kind, label)).toEqual([
      { value: 'passive', label: '«passive»', count: 2, picked: false },
      { value: 'trinket', label: '«trinket»', count: 1, picked: false },
    ])
  })

  // The rule that has never had a test: a value the other facets have emptied is not offered,
  // because picking it could only give nothing (`docs/BACKLOG.md` B29).
  it('drops a value nothing is left behind', () => {
    const filter = { ...empty(), picks: { pool: ['treasure'], kind: [] } }
    expect(
      facetOptions(faceting, rows, filter, Facet.Kind, label).map((o) => o.value),
    ).toEqual(['passive'])
  })

  // …unless it is the one currently picked: dropping it would remove the only control that
  // undoes it, and the list would stay filtered by something invisible.
  it('keeps a picked value even when its count is zero', () => {
    const filter = { ...empty(), picks: { pool: ['shop'], kind: ['passive'] } }
    const options = facetOptions(faceting, rows, filter, Facet.Kind, label)
    expect(options).toEqual([
      { value: 'passive', label: '«passive»', count: 0, picked: true },
    ])
  })
})

describe('foldStartsOpen', () => {
  const slots: FacetSlot<Facet>[] = [
    { facet: Facet.Pool, inView: true },
    { facet: Facet.Kind, inView: false },
  ]

  it('stays closed when nothing behind it is picked', () => {
    expect(foldStartsOpen(slots, empty())).toBe(false)
  })

  it('opens when a folded facet holds a pick, so no filter is hidden', () => {
    const filter = { ...empty(), picks: { pool: [], kind: ['trinket'] } }
    expect(foldStartsOpen(slots, filter)).toBe(true)
  })

  // A pick on a control that is already on screen is not a reason to unfold: the reader can
  // see it.
  it('stays closed when only a facet in view is picked', () => {
    const filter = { ...empty(), picks: { pool: ['shop'], kind: [] } }
    expect(foldStartsOpen(slots, filter)).toBe(false)
  })
})
```

- [ ] **Step 2: Run the tests and watch them fail**

Run: `pnpm ui:test -- facetOptions`
Expected: FAIL — `Failed to resolve import "./facetOptions"`.

- [ ] **Step 3: Write the module**

Create `ui/src/lib/facets/facetOptions.ts`:

```ts
import type { FacetFilter, Faceting } from './faceting'

// What a filter control needs to draw one facet, and the one judgment in the whole bar: which
// values are worth offering. It lives here rather than in the component because it is the part
// worth checking — the frontend's reading of "if a return value is worth checking, it lives in
// a pure crate".

export interface FacetOption {
  value: string
  label: string
  count: number
  picked: boolean
}

// A facet and where the bar draws it: in view at rest, or behind the fold. The screen decides,
// because which filter matters is a fact about the screen and not about the control.
export interface FacetSlot<Facet extends string> {
  facet: Facet
  inView: boolean
}

// The count is over the rows every *other* facet and the search leave: it says what picking the
// value would give, which is not how many rows it gives now.
//
// A value with nothing behind it is not offered at all: it could not be picked, and reading it
// with a 0 beside it is noise. A value that *is* picked stays whatever its count — it is the
// only control that undoes itself, and dropping it would leave the list filtered by something
// the reader cannot see.
export const facetOptions = <Row, Facet extends string>(
  faceting: Faceting<Row, Facet>,
  rows: Row[],
  filter: FacetFilter<Facet>,
  facet: Facet,
  valueLabel: (facet: Facet, value: string) => string,
): FacetOption[] => {
  const counts = faceting.counts(rows, filter, facet)
  const picked = filter.picks[facet]
  return faceting
    .options(rows, facet)
    .map((value) => ({
      value,
      label: valueLabel(facet, value),
      count: counts.get(value) ?? 0,
      picked: picked.includes(value),
    }))
    .filter((option) => option.count > 0 || option.picked)
}

// The fold opens by itself when something behind it is picked. Nothing is stored: a filter that
// is on must be reachable, and that is a property of the current picks, not of what the reader
// did last time.
export const foldStartsOpen = <Facet extends string>(
  slots: FacetSlot<Facet>[],
  filter: FacetFilter<Facet>,
): boolean =>
  slots.some((slot) => !slot.inView && filter.picks[slot.facet].length > 0)
```

- [ ] **Step 4: Run the tests and watch them pass**

Run: `pnpm ui:test -- facetOptions`
Expected: PASS, 6 tests.

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib/facets/facetOptions.ts ui/src/lib/facets/facetOptions.test.ts
git commit -m "test(ui): the options a filter offers, and when the fold must open itself"
```

---

### Task 2: the `MultiSelect` primitive, and its row on the Kit page

**Files:**
- Create: `ui/src/components/ui/multi-select/summary.ts`
- Create: `ui/src/components/ui/multi-select/summary.test.ts`
- Create: `ui/src/components/ui/multi-select/MultiSelect.vue`
- Create: `ui/src/components/ui/multi-select/index.ts`
- Create: `ui/src/kit/sections/MultiSelectSection.vue`
- Modify: `ui/src/kit/KitPage.vue` (import and place the new section beside `SelectSection`)
- Modify: `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts` (the `filters` block)

**Interfaces:**
- Consumes: `FacetOption` from `@/lib/facets/facetOptions` (Task 1).
- Produces:
  - `pickedSummary(labels: string[]): string | null` — the trigger's tail, `null` when nothing
    is picked
  - `SEARCHABLE_FROM: number` — the option count from which the search field appears
  - `MultiSelect.vue`, props
    `{ label: string; options: FacetOption[]; picked: string[] }`, emit
    `'update:picked': [values: string[]]`

- [ ] **Step 1: Write the failing test for the trigger's words**

Create `ui/src/components/ui/multi-select/summary.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { pickedSummary } from './summary'

describe('pickedSummary', () => {
  it('says nothing when nothing is picked, so the trigger reads as the facet alone', () => {
    expect(pickedSummary([])).toBeNull()
  })

  it('names one pick, and two', () => {
    expect(pickedSummary(['3'])).toBe('3')
    expect(pickedSummary(['3', '4'])).toBe('3, 4')
  })

  // A trigger is a button on a wrapping row, not a paragraph: past two the count says more
  // than three truncated names would, and the chips row underneath spells them all out.
  it('stops at two and counts the rest', () => {
    expect(pickedSummary(['angel', 'boss', 'devil'])).toBe('angel, boss +1')
    expect(pickedSummary(['a', 'b', 'c', 'd', 'e'])).toBe('a, b +3')
  })
})
```

- [ ] **Step 2: Run it and watch it fail**

Run: `pnpm ui:test -- summary`
Expected: FAIL — `Failed to resolve import "./summary"`.

- [ ] **Step 3: Write `summary.ts`**

Create `ui/src/components/ui/multi-select/summary.ts`:

```ts
// How many picks the trigger spells out before it starts counting. Two fits a wrapping row of
// controls; the chips under the bar spell out every one of them anyway.
const SPELLED_OUT = 2

// From how many options the search field appears inside the menu. A behaviour, not a visual
// value, so it is a named constant here and not a token in `@theme` — and it is read off the
// list the control was actually given: how many pools or characters exist is a property of the
// user's install, not something a screen may assert on their behalf.
export const SEARCHABLE_FROM = 10

/** The trigger's tail: `Qualità · <this>`. `null` when nothing is picked. */
export const pickedSummary = (labels: string[]): string | null => {
  if (labels.length === 0) return null
  if (labels.length <= SPELLED_OUT) return labels.join(', ')
  return `${labels.slice(0, SPELLED_OUT).join(', ')} +${labels.length - SPELLED_OUT}`
}
```

- [ ] **Step 4: Run it and watch it pass**

Run: `pnpm ui:test -- summary`
Expected: PASS, 3 tests.

- [ ] **Step 5: Add the shared words**

In `ui/src/i18n/messages/it.ts`, add a top-level `filters` block (alphabetical neighbours are not
enforced; put it next to the other shared blocks):

```ts
  filters: {
    more: 'Altri filtri',
    fewer: 'Meno filtri',
    active: 'filtri attivi',
    reset: 'Azzera i filtri',
    inMenu: 'filtra i valori',
    noMatch: 'Nessun valore con questo testo.',
  },
```

The same block in `ui/src/i18n/messages/en.ts`:

```ts
  filters: {
    more: 'More filters',
    fewer: 'Fewer filters',
    active: 'active filters',
    reset: 'Clear filters',
    inMenu: 'filter the values',
    noMatch: 'No value matches that text.',
  },
```

- [ ] **Step 6: Write `MultiSelect.vue`**

Create `ui/src/components/ui/multi-select/MultiSelect.vue`:

```vue
<script setup lang="ts">
import { ChevronDownIcon } from '@lucide/vue'
import { computed } from 'vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Command,
  CommandEmpty,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover'
import { useMessages } from '@/i18n'
import type { FacetOption } from '@/lib/facets/facetOptions'
import { SEARCHABLE_FROM, pickedSummary } from './summary'

// Several values of one set, picked at once, each with what picking it would give.
//
// Built on the kit's own `Command`, which is Reka's `ListboxRoot` with the scored search, the
// keyboard wiring and the empty state already on it: a second listbox beside it would be two
// behaviours to keep agreeing forever.
const props = defineProps<{
  label: string
  options: FacetOption[]
  picked: string[]
}>()
const emit = defineEmits<{ 'update:picked': [values: string[]] }>()
const { t } = useMessages()

// The field appears from the list's own size, never from a flag a screen set: how many pools
// exist is a fact about the user's install.
const searchable = computed(() => props.options.length >= SEARCHABLE_FROM)

const summary = computed(() =>
  pickedSummary(
    props.options.filter((option) => option.picked).map((o) => o.label),
  ),
)

const toggle = (value: string) => {
  emit(
    'update:picked',
    props.picked.includes(value)
      ? props.picked.filter((v) => v !== value)
      : [...props.picked, value],
  )
}
</script>

<template>
  <Popover>
    <PopoverTrigger as-child>
      <Button :variant="ButtonVariant.Outline" class="gap-2">
        <span class="truncate"
          >{{ label }}<template v-if="summary"> · {{ summary }}</template></span
        >
        <ChevronDownIcon />
      </Button>
    </PopoverTrigger>
    <PopoverContent class="p-0">
      <Command class="border-0" :multiple="true">
        <CommandInput v-if="searchable" :placeholder="t('filters.inMenu')" />
        <CommandEmpty>{{ t('filters.noMatch') }}</CommandEmpty>
        <CommandList>
          <CommandItem
            v-for="option in options"
            :key="option.value"
            :value="option.value"
            class="flex items-center gap-2"
            @select.prevent="toggle(option.value)"
          >
            <Checkbox :model-value="option.picked" tabindex="-1" />
            <span class="min-w-0 flex-1 truncate">{{ option.label }}</span>
            <span class="text-label text-subtle-foreground tabular-nums">{{
              option.count
            }}</span>
          </CommandItem>
        </CommandList>
      </Command>
    </PopoverContent>
  </Popover>
</template>
```

- [ ] **Step 7: Write `index.ts`**

Create `ui/src/components/ui/multi-select/index.ts`:

```ts
export { default as MultiSelect } from './MultiSelect.vue'
export { SEARCHABLE_FROM, pickedSummary } from './summary'
```

- [ ] **Step 8: Give it a Kit row**

Create `ui/src/kit/sections/MultiSelectSection.vue`, drawing both shapes — under the threshold
and over it — because the whole point of the control is that they differ:

```vue
<script setup lang="ts">
import { ref } from 'vue'
import KitSection from '../KitSection.vue'
import { MultiSelect } from '@/components/ui/multi-select'
import type { FacetOption } from '@/lib/facets/facetOptions'

const option = (value: string, count: number, picked = false): FacetOption => ({
  value,
  label: value,
  count,
  picked,
})

const few = ref<string[]>(['3'])
const many = ref<string[]>([])

const quality = ['0', '1', '2', '3', '4'].map((q, i) => option(q, 120 - i * 20))
const pools = [
  'angel', 'boss', 'curse', 'devil', 'golden chest', 'library', 'planetarium',
  'red chest', 'secret', 'shop', 'treasure', 'ultra secret',
].map((p, i) => option(p, 60 - i * 4))
</script>

<template>
  <KitSection title="MultiSelect">
    <div class="flex flex-wrap items-start gap-3">
      <MultiSelect
        label="Qualità"
        :options="quality.map((o) => ({ ...o, picked: few.includes(o.value) }))"
        :picked="few"
        @update:picked="few = $event"
      />
      <MultiSelect
        label="Pool"
        :options="pools.map((o) => ({ ...o, picked: many.includes(o.value) }))"
        :picked="many"
        @update:picked="many = $event"
      />
    </div>
  </KitSection>
</template>
```

In `ui/src/kit/KitPage.vue`, import it beside `SelectSection` and place it after that section in
the template:

```ts
import MultiSelectSection from './sections/MultiSelectSection.vue'
```

- [ ] **Step 9: See it in a browser**

Run: `pnpm ui:dev`, open `#kit`, find the MultiSelect row.
Check: Qualità has **no** search field and Pool has one; picking two values makes the trigger
read `Pool · angel, boss`, a third makes it `Pool · angel, boss +1`; typing in Pool narrows the
list and an impossible string shows the empty sentence; the menu closes on Escape and the picks
survive reopening.

- [ ] **Step 10: Commit**

```bash
git add ui/src/components/ui/multi-select ui/src/kit/sections/MultiSelectSection.vue ui/src/kit/KitPage.vue ui/src/i18n/messages/it.ts ui/src/i18n/messages/en.ts
git commit -m "feat(ui): a multi-select on the kit's own command, and the words the bar shares"
```

---

### Task 3: `FilterBar.vue`

**Files:**
- Create: `ui/src/components/facets/FilterBar.vue`
- Modify: `ui/src/components/facets/labels.ts` (`FilterBarLabels` replaces `ToolbarLabels` and
  `DrawerLabels`; both old interfaces stay until Task 7 so the screens keep compiling)

**Interfaces:**
- Consumes: `FacetOption`, `FacetSlot`, `facetOptions`, `foldStartsOpen` (Task 1); `MultiSelect`
  (Task 2); `StateToggle.vue` unchanged.
- Produces: `FilterBar.vue` with props
  ```ts
  {
    shown: number
    total: number
    query: string
    sort?: Sort
    sorts?: Sort[]
    sortText?: Record<Sort, Label>
    rows: Row[]
    faceting: Faceting<Row, Facet>
    filter: FacetFilter<Facet>
    facets: FacetSlot<Facet>[]
    state: {
      facet: Facet
      order: string[]
      counts: Record<string, number>
      dot: Record<string, string>
      text: Record<string, Label>
    }
    title: Record<Facet, Label>
    valueLabel: (facet: Facet, value: string) => string
    labels: FilterBarLabels
  }
  ```
  and emits `'update:query' [string]`, `'update:sort' [Sort]`,
  `'update:picks' [Facet, string[]]`, `reset []`.
  `interface FilterBarLabels { rows: Label; search: Label; sortBy?: Label }`

- [ ] **Step 1: Add the label type**

In `ui/src/components/facets/labels.ts`, add beside the two existing interfaces:

```ts
// What stays a screen's own: the noun for one of its rows, what its search reads, and how it
// words "ordina per". Everything else the bar says is shared and read from `filters.*`, because
// three screens writing "filtri attivi" three times is how two of them end up disagreeing.
export interface FilterBarLabels {
  rows: Label
  search: Label
  // Absent on a list with nothing to choose between: the Run diary's order is the archive's.
  sortBy?: Label
}
```

- [ ] **Step 2: Write `FilterBar.vue`**

Create `ui/src/components/facets/FilterBar.vue`:

```vue
<script setup lang="ts" generic="Row, Facet extends string, Sort extends string">
import { XIcon } from '@lucide/vue'
import { computed, ref, watch } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { MultiSelect } from '@/components/ui/multi-select'
import {
  ToggleGroup,
  ToggleGroupItem,
  ToggleGroupType,
} from '@/components/ui/toggle-group'
import { useMessages } from '@/i18n'
import { facetOptions, foldStartsOpen } from '@/lib/facets/facetOptions'
import type { FacetSlot } from '@/lib/facets/facetOptions'
import type { FacetFilter, Faceting } from '@/lib/facets/faceting'
import StateToggle from './StateToggle.vue'
import type { FilterBarLabels, Label } from './labels'

// Every filter a list has, in one place: the state, the search, the controls that matter on
// this screen, and the rest behind a fold. The screen brings a description of its facets; the
// judgment — which values are worth offering, whether the fold must open — is in
// `lib/facets/facetOptions.ts`, where a test can see it.
//
// Generic over the row, the facet and the sort so a call site stays typed end to end.
const props = defineProps<{
  shown: number
  total: number
  query: string
  // A list with nothing to choose between has no sort group: a single fake option is a control
  // that changes nothing.
  sort?: Sort
  sorts?: Sort[]
  sortText?: Record<Sort, Label>
  rows: Row[]
  faceting: Faceting<Row, Facet>
  filter: FacetFilter<Facet>
  // In order, each marked as in view at rest or behind the fold.
  facets: FacetSlot<Facet>[]
  // The filter that matters more than the others. Its counts are over *every* row, not over
  // what the other facets leave: that is a different question, and the bar must not answer one
  // with the other.
  state: {
    facet: Facet
    order: string[]
    counts: Record<string, number>
    dot: Record<string, string>
    text: Record<string, Label>
  }
  title: Record<Facet, Label>
  // A picked value in words: the Character facet stores ids (`docs/BACKLOG.md` B28), so no
  // component can label one on its own.
  valueLabel: (facet: Facet, value: string) => string
  labels: FilterBarLabels
}>()
const emit = defineEmits<{
  'update:query': [query: string]
  'update:sort': [sort: Sort]
  'update:picks': [facet: Facet, values: string[]]
  reset: []
}>()
const { t } = useMessages()

const open = ref(foldStartsOpen(props.facets, props.filter))
// A filter arriving from elsewhere — a tab restored, a Search row opening this list already
// filtered — must not land behind a closed fold.
watch(
  () => foldStartsOpen(props.facets, props.filter),
  (must) => {
    if (must) open.value = true
  },
)

const inView = computed(() => props.facets.filter((slot) => slot.inView))
const folded = computed(() => props.facets.filter((slot) => !slot.inView))
const optionsOf = (facet: Facet) =>
  facetOptions(
    props.faceting,
    props.rows,
    props.filter,
    facet,
    props.valueLabel,
  )

// A single-choice group empties when its chosen item is clicked again; a sort always has one.
const onSort = (value: unknown) => {
  const next = props.sorts?.find((s) => s === value)
  if (next) emit('update:sort', next)
}

const active = computed(() => props.faceting.activeCount(props.filter))
const chips = computed(() =>
  props.facets.flatMap(({ facet }) =>
    props.filter.picks[facet].map((value) => ({
      facet,
      value,
      label: props.valueLabel(facet, value),
    })),
  ),
)
const drop = (facet: Facet, value: string) =>
  emit(
    'update:picks',
    facet,
    props.filter.picks[facet].filter((v) => v !== value),
  )
</script>

<template>
  <CardHeader class="flex-col items-stretch gap-3">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <CardTitle class="tabular-nums"
        >{{ shown }} / {{ total }} {{ t(labels.rows) }}</CardTitle
      >
      <div
        v-if="sorts && sorts.length > 0 && sortText && labels.sortBy"
        class="flex flex-wrap items-center gap-2"
      >
        <span class="text-label">{{ t(labels.sortBy) }}</span>
        <ToggleGroup
          :type="ToggleGroupType.Single"
          :model-value="sort"
          @update:model-value="onSort"
        >
          <ToggleGroupItem v-for="s in sorts" :key="s" :value="s">{{
            t(sortText[s])
          }}</ToggleGroupItem>
        </ToggleGroup>
      </div>
    </div>
    <StateToggle
      :order="state.order"
      :counts="state.counts"
      :picked="filter.picks[state.facet]"
      :dot="state.dot"
      :text="state.text"
      @update="emit('update:picks', state.facet, $event)"
    />
    <div class="flex flex-wrap items-center gap-2">
      <Input
        :model-value="query"
        :placeholder="t(labels.search)"
        class="w-search"
        @update:model-value="emit('update:query', String($event))"
      />
      <MultiSelect
        v-for="slot in inView"
        :key="slot.facet"
        :label="t(title[slot.facet])"
        :options="optionsOf(slot.facet)"
        :picked="filter.picks[slot.facet]"
        @update:picked="emit('update:picks', slot.facet, $event)"
      />
      <Button
        v-if="folded.length > 0"
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.Compact"
        @click="open = !open"
        >{{ t(open ? 'filters.fewer' : 'filters.more') }}</Button
      >
    </div>
    <div v-if="open && folded.length > 0" class="flex flex-wrap items-center gap-2">
      <MultiSelect
        v-for="slot in folded"
        :key="slot.facet"
        :label="t(title[slot.facet])"
        :options="optionsOf(slot.facet)"
        :picked="filter.picks[slot.facet]"
        @update:picked="emit('update:picks', slot.facet, $event)"
      />
      <Button
        :variant="ButtonVariant.Outline"
        :size="ButtonSize.Compact"
        :disabled="active === 0"
        @click="emit('reset')"
        >{{ t('filters.reset') }}</Button
      >
    </div>
  </CardHeader>
  <div
    v-if="chips.length > 0"
    class="flex flex-wrap items-center gap-1.5 border-b border-hairline bg-muted px-3 py-2"
  >
    <span class="text-label text-subtle-foreground">{{
      t('filters.active')
    }}</span>
    <Button
      v-for="chip in chips"
      :key="`${chip.facet}-${chip.value}`"
      :variant="ButtonVariant.Outline"
      :size="ButtonSize.Compact"
      @click="drop(chip.facet, chip.value)"
      >{{ chip.label }}<XIcon
    /></Button>
  </div>
</template>
```

- [ ] **Step 3: Check it compiles before any screen uses it**

Run: `pnpm typecheck`
Expected: PASS. If `ButtonVariant.Ghost` or `ButtonSize.Compact` do not exist, read
`ui/src/components/ui/button/variants.ts` and use the variants that are there — do not invent one
and do not style the button by hand (frontend rule 4).

- [ ] **Step 4: Commit**

```bash
git add ui/src/components/facets/FilterBar.vue ui/src/components/facets/labels.ts
git commit -m "feat(ui): one filter bar, described by the screen and drawn in one place"
```

---

### Task 4: the Collection on the bar

**Files:**
- Modify: `ui/src/screens/CollectionScreen.vue`
- Modify: `ui/src/screens/collection/collectionLabels.ts`

**Interfaces:**
- Consumes: `FilterBar.vue` (Task 3), `FacetSlot` (Task 1).
- Produces: `collectionSlots: FacetSlot<CollectionFacet>[]`, `barLabels: FilterBarLabels` in
  `collectionLabels.ts`; `drawerFacets`, `toolbarLabels` and `drawerLabels` leave it.

- [ ] **Step 1: Describe the facets**

In `ui/src/screens/collection/collectionLabels.ts`, delete `drawerFacets`, `toolbarLabels` and
`drawerLabels`, and put in their place:

```ts
// Which filters are on screen at rest and which are behind the fold (spec §3). The state has
// its own row and is not one of these: it is the bar's `state` prop.
export const collectionSlots: FacetSlot<CollectionFacet>[] = [
  { facet: CollectionFacet.Quality, inView: true },
  { facet: CollectionFacet.Pool, inView: false },
  { facet: CollectionFacet.Kind, inView: false },
  { facet: CollectionFacet.Origin, inView: false },
]

export const barLabels: FilterBarLabels = {
  rows: 'collection.items',
  search: 'collection.search',
  sortBy: 'collection.sortBy',
}
```

with the imports changed to `import type { FilterBarLabels, Translate } from
'@/components/facets/labels'` and `import type { FacetSlot } from '@/lib/facets/facetOptions'`.

- [ ] **Step 2: Put the bar in the screen**

In `ui/src/screens/CollectionScreen.vue`:

- replace the three imports (`StateToggle`, `FacetDrawer`, `FilterToolbar`) with
  `import FilterBar from '@/components/facets/FilterBar.vue'`
- import `collectionSlots` and `barLabels` instead of `drawerFacets`, `drawerLabels`,
  `toolbarLabels`
- delete `toggle()`: the bar emits whole arrays now, and `setPicks` is all the screen needs
- in the template, delete the `<StateToggle>` and `<FacetDrawer>` blocks that sit outside the
  `<Card>`, and replace `<FilterToolbar …/>` with:

```vue
        <FilterBar
          :shown="rows.length"
          :total="items.length"
          :query="filter.query"
          :sort="sort"
          :sorts="sortOrder"
          :sort-text="sortText"
          :rows="items"
          :faceting="faceting"
          :filter="filter"
          :facets="collectionSlots"
          :state="{
            facet: CollectionFacet.State,
            order: itemStateOrder,
            counts,
            dot: itemStateDot,
            text: itemStateText,
          }"
          :title="collectionFacetTitle"
          :value-label="valueLabel"
          :labels="barLabels"
          @update:query="setQuery"
          @update:sort="setSort"
          @update:picks="setPicks"
          @reset="reset"
        />
```

- [ ] **Step 3: Typecheck, lint and scan**

Run: `pnpm typecheck && pnpm lint && pnpm scan`
Expected: PASS, 0 violations.

- [ ] **Step 4: Look at it**

Run: `pnpm ui:dev`, open the Collection.
Check: the state row, the search and Qualità at rest; `Altri filtri` opens Pool, Tipo, Origine
and `Azzera i filtri`; picking a pool adds a chip and the count in the title moves; Pool has the
search field and Qualità does not.

- [ ] **Step 5: Commit**

```bash
git add ui/src/screens/CollectionScreen.vue ui/src/screens/collection/collectionLabels.ts
git commit -m "feat(ui): the Collection filters from one bar"
```

---

### Task 5: Unlock on the bar

**Files:**
- Modify: `ui/src/screens/UnlockScreen.vue`
- Modify: `ui/src/screens/unlock/facetLabels.ts`

**Interfaces:**
- Consumes: `FilterBar.vue` (Task 3), `FacetSlot` (Task 1).
- Produces: `unlockSlots: FacetSlot<FacetId>[]`, `barLabels: FilterBarLabels`; `drawerFacets`,
  `toolbarLabels` and `drawerLabels` leave `facetLabels.ts`.

- [ ] **Step 1: Describe the facets**

In `ui/src/screens/unlock/facetLabels.ts`, delete `drawerFacets`, `toolbarLabels` and
`drawerLabels` and put in their place:

```ts
// Spec §3: what a node unlocks is the filter a reader reaches for here; the origin and the
// character are behind the fold.
export const unlockSlots: FacetSlot<FacetId>[] = [
  { facet: FacetId.Unlocks, inView: true },
  { facet: FacetId.Origin, inView: false },
  { facet: FacetId.Character, inView: false },
]

export const barLabels: FilterBarLabels = {
  rows: 'unlock.rows',
  search: 'unlock.search',
  sortBy: 'unlock.sortBy',
}
```

- [ ] **Step 2: Put the bar in the screen**

In `ui/src/screens/UnlockScreen.vue`, the same three moves as Task 4 — drop the `StateToggle`,
`FacetDrawer` and `FilterToolbar` imports and blocks, drop `toggle()`, and inside the `<Card>`:

```vue
        <FilterBar
          :shown="rows.length"
          :total="nodes.length"
          :query="filter.query"
          :sort="sort"
          :sorts="sortOrder"
          :sort-text="sortText"
          :rows="nodes"
          :faceting="unlockFaceting"
          :filter="filter"
          :facets="unlockSlots"
          :state="{
            facet: FacetId.State,
            order: stateOrder,
            counts,
            dot: stateDot,
            text: stateText,
          }"
          :title="facetTitle"
          :value-label="valueLabel"
          :labels="barLabels"
          @update:query="setQuery"
          @update:sort="setSort"
          @update:picks="setPicks"
          @reset="reset"
        />
```

- [ ] **Step 3: Typecheck, lint and scan**

Run: `pnpm typecheck && pnpm lint && pnpm scan`
Expected: PASS, 0 violations.

- [ ] **Step 4: Look at it**

Run: `pnpm ui:dev`, open Unlock.
Check: `Cosa sblocca` at rest, `Personaggio` behind the fold and **with** its search field (39
values); a character picked from the fold shows a chip with the character's name and not its id.

- [ ] **Step 5: Commit**

```bash
git add ui/src/screens/UnlockScreen.vue ui/src/screens/unlock/facetLabels.ts
git commit -m "feat(ui): Unlock filters from one bar"
```

---

### Task 6: Runs on the bar, with the outcome promoted to the state row

**Files:**
- Modify: `ui/src/lib/runs/runFacets.ts` (add `outcomeCounts`)
- Modify: `ui/src/lib/runs/runFacets.test.ts` (its test, first)
- Modify: `ui/src/lib/runs/runLabels.ts` (add `outcomeDot`, `outcomeOrder`, `runSlots`,
  `barLabels`)
- Modify: `ui/src/screens/RunsScreen.vue`

**Interfaces:**
- Consumes: `FilterBar.vue` (Task 3), `FacetSlot` (Task 1).
- Produces:
  - `outcomeCounts(runs: RunView[]): Record<string, number>`
  - `outcomeOrder: RunOutcomeView['kind'][]`, `outcomeDot: Record<RunOutcomeView['kind'], string>`
  - `runSlots: FacetSlot<RunFacet>[]`, `barLabels: FilterBarLabels`

- [ ] **Step 1: Write the failing test for the tally**

Append to `ui/src/lib/runs/runFacets.test.ts` (reuse the file's existing `run()` helper; if it
builds a `RunView` differently, follow that file and not this snippet):

```ts
describe('outcomeCounts', () => {
  it('counts every outcome, and says zero for one nothing reached', () => {
    const runs = [run({ outcome: { kind: 'won', ending: 'The Void' } }),
                  run({ outcome: { kind: 'won', ending: 'Mother' } }),
                  run({ outcome: { kind: 'abandoned' } })]
    expect(outcomeCounts(runs)).toEqual({ won: 2, died: 0, abandoned: 1, open: 0 })
  })

  // An empty archive still draws four controls: a state row whose values come and go with the
  // data would move under the reader's cursor.
  it('answers for an archive with nothing in it', () => {
    expect(outcomeCounts([])).toEqual({ won: 0, died: 0, abandoned: 0, open: 0 })
  })
})
```

- [ ] **Step 2: Run it and watch it fail**

Run: `pnpm ui:test -- runFacets`
Expected: FAIL — `outcomeCounts is not exported`.

- [ ] **Step 3: Write `outcomeCounts`**

In `ui/src/lib/runs/runFacets.ts`, export the outcome order and the tally:

```ts
// The four outcomes always have a control, whatever the archive holds: a state row whose values
// appear and disappear with the data moves under the reader's cursor.
export const outcomeOrder: RunOutcomeView['kind'][] = [
  'won',
  'died',
  'abandoned',
  'open',
]

/** How many runs each outcome holds, over every run: the state row counts the archive. */
export const outcomeCounts = (runs: RunView[]): Record<string, number> =>
  Object.fromEntries(
    outcomeOrder.map((kind) => [
      kind,
      runs.filter((run) => run.outcome.kind === kind).length,
    ]),
  )
```

`outcomeOrder` replaces the file's private `outcomeOrder` constant if one is already there — keep
one, exported, and leave the comment that explains why `open` does not sit beside `died`.

- [ ] **Step 4: Run it and watch it pass**

Run: `pnpm ui:test -- runFacets`
Expected: PASS.

- [ ] **Step 5: Give the outcome its squares**

In `ui/src/lib/runs/runLabels.ts`:

```ts
// The outcome is Runs' state (spec §5), and it wears the tones `RunRow.vue` already gives it:
// won is done, died is blocked, abandoned is the dashed partial — a fact, never the unknown
// hatch this design keeps for what it could not read — and open is in progress, never a failure.
export const outcomeDot: Record<RunOutcomeView['kind'], string> = {
  won: 'bg-state-done',
  died: 'bg-state-blocked',
  abandoned: 'border border-dashed border-state-blocked',
  open: 'bg-state-now',
}

export const outcomeTextByKind: Record<RunOutcomeView['kind'], Key> = {
  won: 'runs.outcome.won',
  died: 'runs.outcome.died',
  abandoned: 'runs.outcome.abandoned',
  open: 'runs.outcome.open',
}

export const runSlots: FacetSlot<RunFacet>[] = [
  { facet: RunFacet.Character, inView: true },
  { facet: RunFacet.Online, inView: false },
  { facet: RunFacet.Source, inView: false },
]

export const barLabels: FilterBarLabels = {
  rows: 'runs.rows',
  search: 'runs.search',
}
```

Note the shape: `outcomeText(kind)` stays for `RunRow.vue`, and `outcomeTextByKind` is the same
mapping as a table because `StateToggle` takes a table. Write `outcomeText` as a lookup into it
rather than a second `switch`, so the two can never disagree:

```ts
export const outcomeText = (outcome: RunOutcomeView['kind']): Key =>
  outcomeTextByKind[outcome]
```

- [ ] **Step 6: Put the bar in the screen**

In `ui/src/screens/RunsScreen.vue`: drop the `FacetDrawer` and `FilterToolbar` imports, the
`DrawerLabels`/`ToolbarLabels` import and the two label objects at the bottom of the script, drop
`toggle()`, add `setPicks`:

```ts
const setPicks = (facet: RunFacet, picked: string[]) => {
  filter.value = {
    ...filter.value,
    picks: { ...filter.value.picks, [facet]: picked },
  }
}
const counts = computed(() => outcomeCounts(all.value))
```

and in the template delete the `<FacetDrawer>` block and replace `<FilterToolbar …/>` with:

```vue
        <FilterBar
          :shown="rows.length"
          :total="all.length"
          :query="filter.query"
          :rows="all"
          :faceting="runFaceting"
          :filter="filter"
          :facets="runSlots"
          :state="{
            facet: RunFacet.Outcome,
            order: outcomeOrder,
            counts,
            dot: outcomeDot,
            text: outcomeTextByKind,
          }"
          :title="facetTitle"
          :value-label="valueLabel"
          :labels="barLabels"
          @update:query="setQuery"
          @update:picks="setPicks"
          @reset="reset"
        />
```

- [ ] **Step 7: Typecheck, lint, scan and test**

Run: `pnpm typecheck && pnpm lint && pnpm scan && pnpm ui:test`
Expected: PASS.

- [ ] **Step 8: Look at it**

Run: `pnpm ui:dev`, open Runs (Tool).
Check: the four outcomes as coloured squares with counts, under the KPI tiles; picking `vinta`
moves the `N / M` in the title while the KPI tiles stay still — they count the archive, the row
counts what the filter left; `Personaggio` at rest, `Con chi` and `Da dove` behind the fold.

- [ ] **Step 9: Commit**

```bash
git add ui/src/lib/runs ui/src/screens/RunsScreen.vue
git commit -m "feat(ui): the Run diary filters from one bar, and the outcome is its state"
```

---

### Task 7: the drawer leaves, and the words it took with it

**Files:**
- Delete: `ui/src/components/facets/FacetDrawer.vue`, `ui/src/components/facets/FilterToolbar.vue`
- Modify: `ui/src/components/facets/labels.ts` (drop `ToolbarLabels` and `DrawerLabels`)
- Modify: `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts`
- Modify: `scripts/test-floor` (only if `scripts/check` prints the line to paste)
- Modify: `docs/STATUS.md`, `docs/BACKLOG.md`

- [ ] **Step 1: Delete the two components and the two interfaces**

```bash
git rm ui/src/components/facets/FacetDrawer.vue ui/src/components/facets/FilterToolbar.vue
```

and remove `ToolbarLabels` and `DrawerLabels` from `ui/src/components/facets/labels.ts`.

- [ ] **Step 2: Take the duplicated words out of both locales**

Remove from `runs.`, `collection.` and `unlock.`, in **both** `it.ts` and `en.ts`: `facets`,
`activeFilters`, `noFilters`, `reset`, and `collection.resetFilters`. The empty-state buttons on
the Collection and on Runs read `filters.reset` now — change those two templates with them.

Keep `search.resetFilters` ("Azzera la ricerca"): it is a different sentence about a different
control.

- [ ] **Step 3: Prove nothing still names them**

Run: `grep -rn "activeFilters\|noFilters\|\.facets\b\|resetFilters" ui/src`
Expected: only `search.resetFilters` comes back. `pnpm typecheck` catches the rest — a missing
message key is a type error here, which is why the i18n is typed.

- [ ] **Step 4: The whole gate**

Run: `pnpm check`
Expected: everything green. If it reports the test count rose, paste the line it prints into
`scripts/test-floor` — failing on a rise would fail every commit that adds a test.

- [ ] **Step 5: Write the state down**

In `docs/STATUS.md`, add a group to *"What only a window can say"*, copying the six lines from
§11 of the spec, and raise the count in that section's preamble (it reads "50 in ten groups" —
make it right, and say what the new group is). In `docs/BACKLOG.md`, close B29 with the date, a
pointer to the spec and the report, and the two corrections this work made to the entry itself:
**three screens and not two**, and **"Faccette" was already gone** before the work started.

- [ ] **Step 6: Commit**

```bash
git add -u
git add docs/STATUS.md docs/BACKLOG.md
git commit -m "refactor(ui): the facet drawer leaves, with the words three screens wrote three times"
```

- [ ] **Step 7: Write the report**

Create `docs/superpowers/reports/2026-09-17-filter-bar-report.md`: what was built, what the work
corrected in its own spec and in B29, what a window still has to say, and the measurement worth
keeping (how many lines the three screens lost). Commit it as `docs: …`.

- [ ] **Step 8: The board**

Move the B29 card to `UAT`, tick its four checklist items, and add the `NEEDS WINDOW` label if it
is not already there. The card is `https://trello.com/c/bX7c9CwQ`.

---

## Self-review

**Spec coverage.** §2 → Task 3. §3 → Tasks 4, 5, 6 (the three slot tables). §4 → Task 2. §5 →
Task 6. §6 → Task 1 (`foldStartsOpen`) and Task 3 (the `watch` that keeps a filter from arriving
behind a closed fold). §7 → Tasks 4–6, the bar inside the `Card`. §8 → Task 1. §9 → Task 2
(the block goes in) and Task 7 (the duplicates come out). §10 → nothing to build; the plan touches
neither `faceting.ts` nor a count. §11 → Task 7 step 5.

**Type consistency.** `FacetOption` and `FacetSlot` are defined once in Task 1 and imported by
Tasks 2–6. `FilterBarLabels` is defined in Task 3 and filled in Tasks 4, 5 and 6. The bar emits
`update:picks [facet, values]` and every screen's handler is `setPicks(facet, picked)` — the same
signature the three screens already have.

**One risk named rather than left.** `StateToggle` takes `text: Record<string, Label>`; the
Collection and Unlock pass `Record<ItemState, …>` and `Record<NodeState, …>`, which are records
over a narrower key type and assign cleanly. Runs passes `outcomeTextByKind`, over
`RunOutcomeView['kind']`, for the same reason. If TypeScript refuses one of them, widen at the
call site with a satisfies clause — do **not** loosen the prop to `Record<string, string>`, which
would let a screen pass a raw sentence and lose the i18n typing.
