<script setup lang="ts">
import { computed } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { useMessages } from '@/i18n'
import type { RunView } from '@/lib/ipc/types'
import { outcomeText, sourceText } from '@/lib/runs/runLabels'

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
    return t('runs.killedBy', { killer: outcome.killer })
  return ''
})
</script>

<template>
  <span class="min-w-0 px-2">
    <span v-if="run.character !== null" class="truncate text-row">{{
      run.character
    }}</span>
    <EmptyValue v-else>{{ t('runs.noCharacter') }}</EmptyValue>
  </span>
  <span class="flex min-w-0 items-center gap-2 px-2">
    <Badge :variant="tone[run.outcome.kind]">{{
      t(outcomeText(run.outcome.kind))
    }}</Badge>
    <span class="truncate text-label text-subtle-foreground">{{ detail }}</span>
  </span>
  <span class="px-2 text-right text-row tabular-nums">{{ run.floors }}</span>
  <span
    class="truncate px-2 text-label text-subtle-foreground @max-compact/page:hidden"
    >{{ run.seedWords }}</span
  >
  <span class="flex min-w-0 items-center gap-2 px-2 @max-compact/page:hidden">
    <span class="truncate text-label text-subtle-foreground">{{
      t(sourceText(run.source.kind))
    }}</span>
    <Badge v-if="run.online" :variant="BadgeVariant.Tag">{{
      t('runs.online.online')
    }}</Badge>
  </span>
</template>
