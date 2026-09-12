<script setup lang="ts">
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import WikiInline from '@/components/wiki/WikiInline.vue'
import { useMessages } from '@/i18n'
import type { Inline, Target } from '@/lib/ipc/types'

// One row of a page's card: a label and a value that is inline text, plain text, or the
// declared "none" — the box is there, the dataset didn't fill it.
defineProps<{
  label: string
  inline?: Inline[]
  text?: string | null
  iconFor?: (target: Target) => string | null
  canOpen?: (target: Target) => boolean
}>()
const emit = defineEmits<{ navigate: [target: Target, newTab: boolean] }>()
const { t } = useMessages()
</script>

<template>
  <div class="flex gap-4">
    <dt class="w-32 shrink-0 text-label text-subtle-foreground">{{ label }}</dt>
    <dd class="min-w-0 flex-1 text-row">
      <WikiInline
        v-if="inline && inline.length > 0"
        :inline="inline"
        :icon-for="iconFor"
        :can-open="canOpen"
        @navigate="(target, newTab) => emit('navigate', target, newTab)"
      />
      <span v-else-if="text" class="text-foreground-soft">{{ text }}</span>
      <EmptyValue v-else>{{ t('wiki.infobox.none') }}</EmptyValue>
    </dd>
  </div>
</template>
