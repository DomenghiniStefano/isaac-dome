<script setup lang="ts">
import { computed } from 'vue'
import EntityChip from '@/components/runs/EntityChip.vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import { useMessages } from '@/i18n'
import { columnName } from '@/lib/graph/nodeState'
import type { AchievementRef, LiveOpen, Target } from '@/lib/ipc/types'
import { SecondLevelView } from '@/lib/ipc/types'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'

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
  return a.kind === 'known' ? a.text : String(a.slot)
}
const conditionOf = (a: AchievementRef): string | null =>
  a.kind === 'known' ? a.condition : null
const targetOf = (a: AchievementRef): Target | null =>
  a.kind === 'known' ? { kind: 'achievement', id: a.id } : null
const iconOf = (a: AchievementRef): string | null =>
  a.kind === 'known' ? a.iconUrl : null

// The second level is said only where it is a different thing to go and do, and in the
// column's own word (B66): Ultra Greedier in Greed, hard elsewhere. Which is which arrives
// from Rust in `secondLevel`, `null` at the base level.
const secondLevelWord: Record<SecondLevelView, MessageKey<MessageSchema>> = {
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
          :target="targetOf(row.achievement)"
          :name="text(row.achievement)"
          :detail="conditionOf(row.achievement)"
          :icon-url="iconOf(row.achievement)"
        />
      </span>
      <span class="truncate px-2 text-row">{{ cellName(row.open) }}</span>
      <span class="px-2 text-label text-subtle-foreground">
        <span v-if="conditionOf(row.achievement)" class="line-clamp-2">{{
          conditionOf(row.achievement)
        }}</span>
        <EmptyValue v-else>{{ t('live.noCondition') }}</EmptyValue>
      </span>
      <span class="px-2 text-right text-label tabular-nums">{{
        row.fanOut > 0 ? row.fanOut : t('live.opensNothingMore')
      }}</span>
    </div>
  </div>
</template>
