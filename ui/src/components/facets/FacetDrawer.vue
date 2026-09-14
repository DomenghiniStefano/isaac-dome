<script setup lang="ts" generic="Row, Facet extends string">
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import {
  CardCollapsible,
  CardCollapsibleContent,
  CardCollapsibleTrigger,
} from '@/components/ui/card'
import { Checkbox } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import { useMessages } from '@/i18n'
import type { FacetFilter, Faceting } from '@/lib/facets/faceting'
import type { DrawerLabels, Label as MessageLabel } from './labels'

const props = defineProps<{
  rows: Row[]
  faceting: Faceting<Row, Facet>
  // The facets this drawer holds. The state has its own control above the table, so it is a
  // screen's decision which facets come here and not the whole order.
  facets: Facet[]
  filter: FacetFilter<Facet>
  title: Record<Facet, MessageLabel>
  // A value in words: the Character facet stores ids (`docs/BACKLOG.md` B28) and a quality is
  // a number, so no component can label a value on its own.
  valueLabel: (facet: Facet, value: string) => string
  labels: DrawerLabels
}>()
const emit = defineEmits<{
  toggle: [facet: Facet, value: string]
  reset: []
}>()
const { t } = useMessages()

// One column per facet shown: the count is data, and reaches the grid as a CSS variable rather
// than as a `grid-cols-N` typed once per screen.
const columns = computed(() => ({ '--facet-columns': props.facets.length }))

// Each count is over the rows every other facet and the search leave: it says what picking the
// value would give. A value that would give nothing, and isn't picked, is not offered at all:
// it could not be picked, and reading it with a 0 beside it is noise (`docs/BACKLOG.md` B29).
const drawn = computed(() =>
  props.facets.map((facet) => {
    const counts = props.faceting.counts(props.rows, props.filter, facet)
    const picked = props.filter.picks[facet]
    return {
      facet,
      values: props.faceting
        .options(props.rows, facet)
        .map((value) => ({
          value,
          label: props.valueLabel(facet, value),
          count: counts.get(value) ?? 0,
          picked: picked.includes(value),
        }))
        .filter((option) => option.count > 0 || option.picked),
    }
  }),
)

const active = computed(() => props.faceting.activeCount(props.filter))
</script>

<template>
  <!-- Schermate.dc.html, "Faccette": the facets live in a drawer, not in a panel always open. -->
  <CardCollapsible>
    <CardCollapsibleTrigger>
      {{ t(labels.facets) }}
      <template #summary>{{
        active > 0
          ? `${t(labels.activeFilters)}: ${active}`
          : t(labels.noFilters)
      }}</template>
    </CardCollapsibleTrigger>
    <CardCollapsibleContent class="flex flex-col gap-3">
      <div class="grid grid-cols-facets gap-4" :style="columns">
        <div
          v-for="column in drawn"
          :key="column.facet"
          class="flex min-w-0 flex-col gap-1.5"
        >
          <span class="text-label text-subtle-foreground">{{
            t(title[column.facet])
          }}</span>
          <Label
            v-for="entry in column.values"
            :key="entry.value"
            class="flex items-center gap-2"
          >
            <Checkbox
              :model-value="entry.picked"
              @update:model-value="emit('toggle', column.facet, entry.value)"
            />
            <span
              class="min-w-0 flex-1 truncate text-caption text-foreground"
              >{{ entry.label }}</span
            >
            <span class="text-label text-subtle-foreground tabular-nums">{{
              entry.count
            }}</span>
          </Label>
        </div>
      </div>
      <Button
        :variant="ButtonVariant.Outline"
        :size="ButtonSize.Compact"
        class="self-start"
        :disabled="active === 0"
        @click="emit('reset')"
        >{{ t(labels.reset) }}</Button
      >
    </CardCollapsibleContent>
  </CardCollapsible>
</template>
