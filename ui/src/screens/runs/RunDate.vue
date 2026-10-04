<script setup lang="ts">
import { computed } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { useFormat } from '@/composables/useFormat'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import type { RunView } from '@/lib/ipc/types'
import { RunDateKind, runDate } from '@/lib/runs/runDate'

// The day a run was played, and in the tooltip the time and what that time is: a session's
// is when it started, a launch's is when its log was last written.
const props = defineProps<{ run: RunView }>()
const { t } = useMessages()
const format = useFormat()

const date = computed(() => runDate(props.run))
const day = computed(() =>
  date.value.kind === RunDateKind.Undated ? null : format.date(date.value.at),
)
const detail = computed(() => {
  const d = date.value
  switch (d.kind) {
    case RunDateKind.Started:
      return t('runs.date.started', { time: format.time(d.at) })
    case RunDateKind.Written:
      return t('runs.date.written', { time: format.time(d.at) })
    case RunDateKind.Undated:
      return t('runs.date.undated')
    default:
      return assertNever(d)
  }
})
</script>

<template>
  <Tooltip>
    <TooltipTrigger as-child>
      <span
        class="truncate px-2 text-label text-subtle-foreground tabular-nums"
      >
        <template v-if="day !== null">{{ day }}</template>
        <EmptyValue v-else>{{ t('runs.date.none') }}</EmptyValue>
      </span>
    </TooltipTrigger>
    <TooltipContent>{{ detail }}</TooltipContent>
  </Tooltip>
</template>
