<script setup lang="ts">
import { inject } from 'vue'
import type { HTMLAttributes } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import WikiInline from '@/components/wiki/WikiInline.vue'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { Inline } from '@/lib/ipc/types'
import { infoboxLinksKey } from './infobox/links'

// One row of a page's card: a label and a value that is inline text, plain text, or the
// declared "none" — the box is there, the dataset didn't fill it. A value that is none of the
// three (the quality's pips, a row of badges) is the default slot, laid out by `valueClass`.
const props = defineProps<{
  label: string
  inline?: Inline[]
  text?: string | null
  class?: HTMLAttributes['class']
  valueClass?: string
}>()
const links = inject(infoboxLinksKey, null)
const { t } = useMessages()
</script>

<template>
  <!-- The label sits above its value, not beside it (card #57): the card lives in a column
       beside the page's text, and a 128px label in a 272px column left the values wrapping
       every second row. Stacked, the value gets the whole width at every size. -->
  <div
    :class="
      cn(
        'flex flex-col gap-1 border-b border-hairline pb-2 last:border-b-0 last:pb-0',
        props.class,
      )
    "
  >
    <dt class="text-label text-subtle-foreground">{{ label }}</dt>
    <dd :class="valueClass ?? 'min-w-0 text-row'">
      <slot>
        <WikiInline
          v-if="inline && inline.length > 0"
          :inline="inline"
          :icon-for="links?.iconFor"
          :can-open="links?.canOpen"
          @navigate="(target, newTab) => links?.navigate(target, newTab)"
        />
        <span v-else-if="text" class="text-foreground-soft">{{ text }}</span>
        <EmptyValue v-else>{{ t('wiki.infobox.none') }}</EmptyValue>
      </slot>
    </dd>
  </div>
</template>
