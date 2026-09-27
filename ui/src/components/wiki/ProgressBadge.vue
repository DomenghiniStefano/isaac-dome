<script setup lang="ts">
import { computed } from 'vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { useMessages } from '@/i18n'
import { StateTone } from '@/lib/facets/stateTone'
import type { PageProgress } from '@/lib/ipc/types'
import { progressLines } from '@/lib/wiki/progressLines'

// The save's state for one page, compact (card #90, decision 6): a small row of `Badge`s, one
// per `progressLines` line, in the state tones (`--color-state-done/now/blocked/unknown`).
// `null` — no save chosen, or this page's kind carries no state at all — draws nothing, never
// an empty or a guessed badge.
const props = defineProps<{ progress: PageProgress | null }>()
const { t } = useMessages()

// `progressLines` lives in `lib/`, which may not import a component (CLAUDE.md, the layer
// rule): it names each line's tone as the `StateTone` `lib/facets/stateTone.ts` already
// defines, and this component — allowed to import both — turns that into the `Badge` variant
// that paints it, the same mapping `NodeStateBadge` already draws from `NodeState`.
const badgeVariant: Record<StateTone, BadgeVariant> = {
  [StateTone.Done]: BadgeVariant.Done,
  [StateTone.Now]: BadgeVariant.Now,
  [StateTone.Blocked]: BadgeVariant.Blocked,
  [StateTone.Partial]: BadgeVariant.Partial,
  [StateTone.Unknown]: BadgeVariant.Unknown,
}

const lines = computed(() =>
  props.progress === null ? [] : progressLines(props.progress),
)
</script>

<template>
  <div v-if="lines.length > 0" class="flex flex-wrap items-center gap-1.5">
    <Badge
      v-for="line in lines"
      :key="line.key"
      :variant="badgeVariant[line.variant]"
      >{{ t(line.label.key, line.label.params) }}</Badge
    >
  </div>
</template>
