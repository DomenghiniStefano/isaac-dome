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
  <!-- The label sits above its value, not beside it (card #57): the card now lives in a
       column beside the page's text, and a 128px label in a 272px column left the values
       wrapping every second row. Stacked, the value gets the whole width at every size. -->
  <div
    class="flex flex-col gap-1 border-b border-hairline pb-2 last:border-b-0 last:pb-0"
  >
    <dt class="text-label text-subtle-foreground">{{ label }}</dt>
    <dd class="min-w-0 text-row">
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
