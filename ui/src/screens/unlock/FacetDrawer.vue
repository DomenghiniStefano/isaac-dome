<script setup lang="ts">
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
import { characterForms } from '@/lib/graph/characterName'
import {
  FacetId,
  activeFilterCount,
  facetCounts,
  facetOptions,
} from '@/lib/graph/unlockFilter'
import type { UnlockFilter } from '@/lib/graph/unlockFilter'
import type { UnlockNode } from '@/lib/ipc/types'
import { facetTitle, facetValueLabel } from './facetLabels'

const props = defineProps<{ nodes: UnlockNode[]; filter: UnlockFilter }>()
const emit = defineEmits<{
  toggle: [facet: FacetId, value: string]
  reset: []
}>()
const { t } = useMessages()

// The state has its own control above the table; the drawer holds the other three.
const drawerFacets: FacetId[] = [
  FacetId.Unlocks,
  FacetId.Origin,
  FacetId.Character,
]

// The character facet stores ids: its labels are read from here (`docs/BACKLOG.md` B28).
const characters = computed(() => characterForms(props.nodes))

// Each count is over the rows every other facet and the search leave: it says what picking
// the value would give. A value that would give nothing, and isn't picked, is not offered at
// all: it could not be picked, and reading it with a 0 beside it is noise (B29).
const columns = computed(() =>
  drawerFacets.map((facet) => {
    const counts = facetCounts(props.nodes, props.filter, facet)
    const picked = props.filter.picks[facet]
    return {
      facet,
      values: facetOptions(props.nodes, facet)
        .map((value) => ({
          value,
          label: facetValueLabel(t, facet, value, characters.value),
          count: counts.get(value) ?? 0,
          picked: picked.includes(value),
        }))
        .filter((option) => option.count > 0 || option.picked),
    }
  }),
)

const active = computed(() => activeFilterCount(props.filter))
</script>

<template>
  <!-- Schermate.dc.html, "Faccette": the facets live in a drawer, not in a panel always open. -->
  <CardCollapsible>
    <CardCollapsibleTrigger>
      {{ t('unlock.facets') }}
      <template #summary>{{
        active > 0
          ? `${t('unlock.activeFilters')}: ${active}`
          : t('unlock.noFilters')
      }}</template>
    </CardCollapsibleTrigger>
    <CardCollapsibleContent class="flex flex-col gap-3">
      <div class="grid grid-cols-3 gap-4">
        <div
          v-for="column in columns"
          :key="column.facet"
          class="flex min-w-0 flex-col gap-1.5"
        >
          <span class="text-label text-subtle-foreground">{{
            t(facetTitle[column.facet])
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
        >{{ t('unlock.reset') }}</Button
      >
    </CardCollapsibleContent>
  </CardCollapsible>
</template>
