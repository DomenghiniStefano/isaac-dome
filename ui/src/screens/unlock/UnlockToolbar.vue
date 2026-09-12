<script setup lang="ts">
import { XIcon } from '@lucide/vue'
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import {
  ToggleGroup,
  ToggleGroupItem,
  ToggleGroupType,
} from '@/components/ui/toggle-group'
import { useMessages } from '@/i18n'
import type { CharacterForm } from '@/lib/graph/characterName'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { FacetId, UnlockSort, facetOrder } from '@/lib/graph/unlockFilter'
import type { UnlockFilter } from '@/lib/graph/unlockFilter'
import { facetValueLabel } from './facetLabels'

const props = defineProps<{
  shown: number
  total: number
  filter: UnlockFilter
  query: string
  sort: UnlockSort
  // What a character pick is called: its value is an id (`docs/BACKLOG.md` B28).
  characters: Map<string, CharacterForm>
}>()
const emit = defineEmits<{
  'update:query': [query: string]
  'update:sort': [sort: UnlockSort]
  toggle: [facet: FacetId, value: string]
}>()
const { t } = useMessages()

const sorts: UnlockSort[] = [
  UnlockSort.FanOut,
  UnlockSort.Steps,
  UnlockSort.Name,
]
const sortText: Record<UnlockSort, MessageKey<MessageSchema>> = {
  [UnlockSort.FanOut]: 'unlock.sort.fanOut',
  [UnlockSort.Steps]: 'unlock.sort.steps',
  [UnlockSort.Name]: 'unlock.sort.name',
}

// A single-choice group empties when its chosen item is clicked again; a sort always has one.
const onSort = (value: unknown) => {
  const next = sorts.find((s) => s === value)
  if (next) emit('update:sort', next)
}

const chips = computed(() =>
  facetOrder.flatMap((facet) =>
    props.filter.picks[facet].map((value) => ({
      facet,
      value,
      label: facetValueLabel(t, facet, value, props.characters),
    })),
  ),
)
</script>

<template>
  <CardHeader class="flex-wrap">
    <CardTitle class="tabular-nums"
      >{{ shown }} / {{ total }} {{ t('unlock.rows') }}</CardTitle
    >
    <div class="flex flex-wrap items-center gap-2">
      <Input
        :model-value="query"
        :placeholder="t('unlock.search')"
        class="w-search"
        @update:model-value="emit('update:query', String($event))"
      />
      <span class="text-label">{{ t('unlock.sortBy') }}</span>
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
  </CardHeader>
  <div
    v-if="chips.length > 0"
    class="flex flex-wrap items-center gap-1.5 border-b border-hairline bg-muted px-3 py-2"
  >
    <span class="text-label text-subtle-foreground">{{
      t('unlock.activeFilters')
    }}</span>
    <Button
      v-for="chip in chips"
      :key="`${chip.facet}-${chip.value}`"
      :variant="ButtonVariant.Outline"
      :size="ButtonSize.Compact"
      @click="emit('toggle', chip.facet, chip.value)"
      >{{ chip.label }}<XIcon
    /></Button>
  </div>
</template>
