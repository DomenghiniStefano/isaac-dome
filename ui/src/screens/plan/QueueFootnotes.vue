<script setup lang="ts">
import { CheckIcon, Trash2Icon } from '@lucide/vue'
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import type { QueueDiagnostic, UnlockNode } from '@/lib/ipc/types'
import { achievementText } from '@/lib/plan/queueRows'

const props = defineProps<{
  diagnostics: QueueDiagnostic[]
  nodes: UnlockNode[]
  busy: boolean
}>()
const emit = defineEmits<{ remove: [achievement: number] }>()
const { t } = useMessages()

const nameOf = (id: number): string =>
  achievementText(props.nodes, id) ?? `${t('plan.achievement')} ${id}`

// A row never leaves the queue without a word: the ones closed by playing, and the ones the
// catalog no longer knows, which stay in the file until removed.
const closed = computed((): string | null => {
  const done = props.diagnostics.flatMap((d) =>
    d.kind === 'completed' ? [d] : [],
  )[0]
  if (!done) return null
  const wanted =
    done.wanted.length > 0
      ? ` · ${t('plan.completed.wanted')}: ${done.wanted.map(nameOf).join(', ')}`
      : ''
  return `${t('plan.completed.closed')}: ${done.count}${wanted}`
})

const unresolved = computed(() =>
  props.diagnostics.flatMap((d) =>
    d.kind === 'unresolved' ? [d.achievement] : [],
  ),
)
</script>

<template>
  <div
    v-if="closed || unresolved.length > 0"
    class="flex flex-col gap-1.5 border-t border-hairline px-3 py-2 text-caption text-foreground-soft"
  >
    <span v-if="closed" class="flex items-center gap-2"
      ><CheckIcon class="size-3.5 shrink-0 text-state-done-foreground" />{{
        closed
      }}</span
    >
    <span v-for="id in unresolved" :key="id" class="flex items-center gap-2">
      {{ t('plan.achievement') }} {{ id }} {{ t('plan.unresolved') }}
      <Button
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.Compact"
        :disabled="busy"
        @click="emit('remove', id)"
        ><Trash2Icon class="text-destructive" />{{
          t('queue.removeShort')
        }}</Button
      >
    </span>
  </div>
</template>
