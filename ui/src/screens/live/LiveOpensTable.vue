<script setup lang="ts">
import type { Message } from '@/i18n/message'
import { computed } from 'vue'
import EntityChip from '@/components/runs/EntityChip.vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import { useMessages } from '@/i18n'
import { columnName } from '@/lib/graph/nodeState'
import {
  refCondition,
  refIcon,
  refNumber,
  refTarget,
  refText,
} from '@/lib/graph/achievementNode'
import type { AchievementRef, LiveOpen } from '@/lib/ipc/types'
import { SecondLevelView } from '@/lib/ipc/types'

// Everything the run could open, in one table instead of a card per cell: the cell is a
// column, so the boss is still said once per row and the rows can be read against each other
// — which a stack of cards cannot do.
const props = defineProps<{ opens: LiveOpen[] }>()
const { t } = useMessages()

const rows = computed(() =>
  props.opens.flatMap((open) =>
    open.achievements.map((entry) => ({
      key: `${open.character}-${open.column}-${text(entry.achievement)}`,
      open,
      achievement: entry.achievement,
      fanOut: entry.fanOut,
    })),
  ),
)

function text(a: AchievementRef): string {
  return refText(a) ?? String(refNumber(a))
}

// The second level is said only where it is a different thing to go and do, and in the
// column's own word (B66): Ultra Greedier in Greed, hard elsewhere. Which is which arrives
// from Rust in `secondLevel`, `null` at the base level.
const secondLevelWord: Record<SecondLevelView, Message> = {
  [SecondLevelView.Hard]: 'live.secondLevel.hard',
  [SecondLevelView.UltraGreedier]: 'live.secondLevel.ultraGreedier',
}
const cellName = (open: LiveOpen): string =>
  open.secondLevel === null
    ? columnName[open.column]
    : `${columnName[open.column]} · ${t(secondLevelWord[open.secondLevel])}`
</script>

<template>
  <div class="flex flex-col">
    <div
      class="grid grid-cols-live-opens items-center border-b border-hairline bg-muted text-label text-subtle-foreground"
    >
      <span class="px-2 py-1.5">{{ t('live.column.achievement') }}</span>
      <span class="px-2 py-1.5">{{ t('live.column.cell') }}</span>
      <span class="px-2 py-1.5">{{ t('live.column.condition') }}</span>
      <span class="px-2 py-1.5 text-right">{{ t('live.column.opens') }}</span>
    </div>
    <div
      v-for="(row, i) in rows"
      :key="row.key"
      class="grid grid-cols-live-opens items-center border-b border-hairline"
      :class="i % 2 === 1 ? 'bg-row-alt' : undefined"
    >
      <span class="px-2 py-1">
        <EntityChip
          :target="refTarget(row.achievement)"
          :name="text(row.achievement)"
          :detail="refCondition(row.achievement)"
          :icon-url="refIcon(row.achievement)"
        />
      </span>
      <span class="truncate px-2 text-row">{{ cellName(row.open) }}</span>
      <span class="px-2 text-label text-subtle-foreground">
        <span v-if="refCondition(row.achievement)" class="line-clamp-2">{{
          refCondition(row.achievement)
        }}</span>
        <EmptyValue v-else>{{ t('live.noCondition') }}</EmptyValue>
      </span>
      <span class="px-2 text-right text-label tabular-nums">{{
        row.fanOut > 0 ? row.fanOut : t('live.opensNothingMore')
      }}</span>
    </div>
  </div>
</template>
