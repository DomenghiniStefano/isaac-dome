<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'
import { TallyTone } from '@/lib/completion/completionView'
import type { TallyColumn } from '@/lib/completion/completionView'

// One tally of the marks grid: a count over what could be read, in the colour of how far it
// got. The grid places it; this only says it.
const props = defineProps<{
  column: TallyColumn
  class?: HTMLAttributes['class']
}>()

// A number that fills its denominator in the done colour, one with no denominator at all
// fainter than one that has progress to show. Gold is not used for "almost there": gold
// means unlockable now, and nothing else. A record over the whole set, so a tone with no
// colour fails to compile.
const toneClass: Record<TallyTone, string> = {
  [TallyTone.Full]: 'text-state-done-foreground',
  [TallyTone.Partial]: 'text-subtle-foreground',
  [TallyTone.Unreadable]: 'text-faint-foreground',
}
</script>

<template>
  <span :class="cn(props.class, 'tabular-nums', toneClass[column.tone])"
    >{{ column.value }}/{{ column.readable }}</span
  >
</template>
