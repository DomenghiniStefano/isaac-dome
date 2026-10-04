<script setup lang="ts">
import { computed } from 'vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { useMessages } from '@/i18n'
import type { RunView } from '@/lib/ipc/types'
import { entityName, whoKilled } from '@/lib/runs/death'
import { outcomeText } from '@/lib/runs/runLabels'

const props = defineProps<{ run: RunView }>()
const { t } = useMessages()

// An outcome is a state, not a score: `open` is the run being played and wears the tone of
// something in progress, never the one failure wears.
const tone: Record<RunView['outcome']['kind'], BadgeVariant> = {
  won: BadgeVariant.Done,
  died: BadgeVariant.Blocked,
  // Abandoned is a fact, not an unknown: the Unknown variant draws the question mark this
  // design uses for what the app could not read, and a run you walked away from is not that.
  abandoned: BadgeVariant.Partial,
  open: BadgeVariant.Now,
}

// What ended the run, in the run's own words: the ending it reached or the thing that killed
// it. Both come from the log and are shown as they were written.
const detail = computed(() => {
  const outcome = props.run.outcome
  if (outcome.kind === 'won')
    return t('runs.endedWith', { ending: outcome.ending })
  if (outcome.kind === 'died')
    return t('runs.killedBy', { killer: entityName(whoKilled(outcome)) })
  return ''
})
</script>

<template>
  <span class="flex min-w-0 items-center gap-2">
    <Badge :variant="tone[run.outcome.kind]">{{
      t(outcomeText[run.outcome.kind])
    }}</Badge>
    <span class="truncate text-label text-subtle-foreground">{{ detail }}</span>
  </span>
</template>
