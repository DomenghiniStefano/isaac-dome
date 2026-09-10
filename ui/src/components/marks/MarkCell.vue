<script setup lang="ts">
import { ChevronUpIcon, TriangleAlertIcon } from '@lucide/vue'
import { computed } from 'vue'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { Cell } from '@/lib/ipc/types'
import type { MarkArt } from './markVisual'
import { MarkTier, markBarShare, markVisual } from './markVisual'

const props = defineProps<{
  cell: Cell
  art: MarkArt | null
  label?: string
}>()
const { t } = useMessages()

const visual = computed(() => markVisual(props.cell))

const third = computed(() => {
  const v = visual.value
  return (v.kind === 'empty' || v.kind === 'marked') && v.third
})

const symbol = computed(() => {
  const v = visual.value
  if (v.kind !== 'marked' || !props.art) return null
  return v.tier === MarkTier.Hard ? props.art.hard : props.art.normal
})

// The fallback's bar height travels as a CSS variable, read by h-(--mark-bar).
const barHeight = computed(() => {
  const v = visual.value
  return { '--mark-bar': v.kind === 'marked' ? markBarShare[v.tier] : '0%' }
})

const accessibleName = computed(() => {
  if (props.label === undefined) return undefined
  return third.value ? `${props.label}, ${t('marks.thirdLevel')}` : props.label
})
</script>

<template>
  <!-- Chrome e Stati.dc.html, "Cella della matrice", at cycle 2's scale: a 16px symbol at 2x
       on flat paper; with no art, the bars of Tokens.dc.html. Without a label the cell is
       decoration and the text beside it speaks. -->
  <div
    :role="label === undefined ? undefined : 'img'"
    :aria-label="accessibleName"
    :aria-hidden="label === undefined ? true : undefined"
    :style="barHeight"
    :class="
      cn(
        'relative grid size-mark-cell shrink-0 place-items-center border',
        visual.kind === 'empty' && 'border-hairline bg-data',
        visual.kind === 'marked' &&
          (symbol
            ? 'border-transparent bg-mark-paper'
            : 'border-input bg-data'),
        visual.kind === 'unknown' &&
          'border-dashed border-state-unknown hatch-unknown text-row text-muted-foreground',
        visual.kind === 'unexpected' &&
          'border-state-unexpected bg-state-unexpected-surface text-state-unexpected-foreground',
      )
    "
  >
    <template v-if="visual.kind === 'marked'">
      <img
        v-if="symbol"
        :src="symbol"
        alt=""
        class="size-mark-symbol pixelated"
      />
      <template v-else>
        <span class="absolute inset-x-0 bottom-0 h-(--mark-bar) bg-primary" />
        <ChevronUpIcon
          v-if="visual.tier === MarkTier.Hard"
          class="relative size-3.5 text-foreground"
        />
      </template>
    </template>
    <span v-else-if="visual.kind === 'unknown'">?</span>
    <TriangleAlertIcon
      v-else-if="visual.kind === 'unexpected'"
      class="size-4"
    />
    <span
      v-if="third"
      :class="
        cn(
          'absolute right-0.75 size-1.5',
          symbol ? 'bottom-0.75 bg-background' : 'top-0.75 bg-highlight',
        )
      "
    />
  </div>
</template>
