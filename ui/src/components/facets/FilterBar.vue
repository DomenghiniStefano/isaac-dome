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
import {
  facetOptions,
  foldStartsOpen,
  stateRowCounts,
} from '@/lib/facets/facetOptions'
import type { FacetSlot } from '@/lib/facets/facetOptions'
import type { FacetFilter, Faceting } from '@/lib/facets/faceting'
import StateToggle from './StateToggle.vue'
import type { FilterBarLabels, Label } from './labels'

// Every filter a list has, in one place: the state, the search, the controls that matter on
// this screen, and the rest behind a fold. The screen brings a description of its facets; the
// judgment — which values are worth offering, whether the fold has to open — lives in
// `lib/facets/facetOptions.ts`, where a test can see it.
//
// Generic over the row, the facet and the sort so a call site stays typed end to end: with
// `string` props each screen would narrow the emitted values back to its own union.
const props = defineProps<{
  shown: number
  total: number
  query: string
  // A list with nothing to choose between has no sort group: the Run diary's order is decided
  // by the archive (`runOrder.ts`), not by the reader, and a single fake option would be a
  // control that changes nothing.
  sort?: Sort
  sorts?: Sort[]
  sortText?: Record<Sort, Label>
  rows: Row[]
  faceting: Faceting<Row, Facet>
  filter: FacetFilter<Facet>
  // The facets, in order, each marked as in view at rest or behind the fold.
  facets: FacetSlot<Facet>[]
  // The filter that matters more than the others (DESIGN-BRIEF.md §6). The screen brings what
  // its states are, what they are called and what colour each square carries; the numbers are
  // the bar's, counted exactly like a dropdown's — see `stateRowCounts`.
  state: {
    facet: Facet
    order: string[]
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
// filtered — must not land behind a closed fold. Only opening is automatic: closing it again
// is the reader's, and a fold that reclosed itself would fight them.
watch(
  () => foldStartsOpen(props.facets, props.filter),
  (must) => {
    if (must) open.value = true
  },
)

const stateCounts = computed(() =>
  stateRowCounts(
    props.faceting,
    props.rows,
    props.filter,
    props.state.facet,
    props.state.order,
  ),
)

const inView = computed(() => props.facets.filter((slot) => slot.inView))
const folded = computed(() => props.facets.filter((slot) => !slot.inView))
const optionsOf = (facet: Facet) =>
  facetOptions(props.faceting, props.rows, props.filter, facet, props.valueLabel)

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
      :counts="stateCounts"
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
        >{{ open ? t('filters.fewer') : t('filters.more') }}</Button
      >
    </div>
    <div
      v-if="open && folded.length > 0"
      class="flex flex-wrap items-center gap-2"
    >
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
