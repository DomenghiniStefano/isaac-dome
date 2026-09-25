<script
  setup
  lang="ts"
  generic="Row, Facet extends string, Sort extends string"
>
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
import { foldStartsOpen, stateRowCounts } from '@/lib/facets/facetOptions'
import type { FacetFilter } from '@/lib/facets/faceting'
import { optionsBySlot } from '@/lib/facets/filterBar'
import type { FilterBarDescriptor } from '@/lib/facets/filterBar'
import StateToggle from './StateToggle.vue'

// Every filter a list has, in one place: the state, the search, the controls that matter on
// this screen, and the rest behind a fold. The screen brings a description of its facets — the
// `bar`, one object beside its labels; the judgment — which values are worth offering, whether
// the fold has to open — lives in `lib/facets/facetOptions.ts`, where a test can see it.
//
// Generic over the row, the facet and the sort so a call site stays typed end to end: with
// `string` props each screen would narrow the emitted values back to its own union.
const props = defineProps<{
  bar: FilterBarDescriptor<Row, Facet, Sort>
  // Every row of the list; the count beside the title is how many of them the filter shows.
  rows: Row[]
  filter: FacetFilter<Facet>
  shown: number
  sort?: Sort
  // A picked value in words: the Character facet stores ids (`docs/BACKLOG.md` B28), so no
  // component can label one on its own.
  valueLabel: (facet: Facet, value: string) => string
}>()
const emit = defineEmits<{
  'update:query': [query: string]
  'update:sort': [sort: Sort]
  'update:picks': [facet: Facet, values: string[]]
  reset: []
}>()
const { t } = useMessages()

const open = ref(foldStartsOpen(props.bar.facets, props.filter))
// A filter arriving from elsewhere — a tab restored, a Search row opening this list already
// filtered — must not land behind a closed fold. Only opening is automatic: closing it again
// is the reader's, and a fold that reclosed itself would fight them.
watch(
  () => foldStartsOpen(props.bar.facets, props.filter),
  (must) => {
    if (must) open.value = true
  },
)

const stateCounts = computed(() =>
  stateRowCounts(
    props.bar.faceting,
    props.rows,
    props.filter,
    props.bar.state.facet,
    props.bar.state.order,
  ),
)

const inView = computed(() => props.bar.facets.filter((slot) => slot.inView))
const folded = computed(() => props.bar.facets.filter((slot) => !slot.inView))
const options = computed(() =>
  optionsBySlot(
    props.bar.faceting,
    props.rows,
    props.filter,
    props.bar.facets,
    props.valueLabel,
  ),
)
const optionsOf = (facet: Facet) => options.value.get(facet) ?? []

// The sort group is drawn only for a list that has one, and a way to word "ordina per".
const sorts = computed(() =>
  props.bar.sorts && props.bar.sorts.order.length > 0 && props.bar.labels.sortBy
    ? { ...props.bar.sorts, by: props.bar.labels.sortBy }
    : null,
)

// A single-choice group empties when its chosen item is clicked again; a sort always has one.
const onSort = (value: unknown) => {
  const next = props.bar.sorts?.order.find((s) => s === value)
  if (next) emit('update:sort', next)
}

const active = computed(() => props.bar.faceting.activeCount(props.filter))
const chips = computed(() =>
  props.bar.facets.flatMap(({ facet }) =>
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
        >{{ shown }} / {{ rows.length }} {{ t(bar.labels.rows) }}</CardTitle
      >
      <div v-if="sorts" class="flex flex-wrap items-center gap-2">
        <span class="text-label">{{ t(sorts.by) }}</span>
        <ToggleGroup
          :type="ToggleGroupType.Single"
          :model-value="sort"
          @update:model-value="onSort"
        >
          <ToggleGroupItem v-for="s in sorts.order" :key="s" :value="s">{{
            t(sorts.text[s])
          }}</ToggleGroupItem>
        </ToggleGroup>
      </div>
    </div>
    <StateToggle
      :order="bar.state.order"
      :counts="stateCounts"
      :picked="filter.picks[bar.state.facet]"
      :dot="bar.state.dot"
      :text="bar.state.text"
      @update="emit('update:picks', bar.state.facet, $event)"
    />
    <div class="flex flex-wrap items-center gap-2">
      <Input
        :model-value="filter.query"
        :placeholder="t(bar.labels.search)"
        class="w-search"
        @update:model-value="emit('update:query', String($event))"
      />
      <MultiSelect
        v-for="slot in inView"
        :key="slot.facet"
        :label="t(bar.title[slot.facet])"
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
        :label="t(bar.title[slot.facet])"
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
