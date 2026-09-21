<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import MarksGrid from '@/components/marks/MarksGrid.vue'
import { markArtOf } from '@/components/marks/markVisual'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { MarksMatrix } from '@/lib/ipc/types'
import MatrixLegend from './MatrixLegend.vue'

const props = defineProps<{
  matrix: MarksMatrix
  class?: HTMLAttributes['class']
}>()
const { t } = useMessages()

// The legend draws with the first column's art, when there is some.
const legendArt = computed(() => markArtOf(props.matrix.art[0]))
</script>

<template>
  <Card :class="cn('min-h-0', props.class)">
    <CardHeader>
      <CardTitle>{{ t('completion.card.title') }}</CardTitle>
    </CardHeader>
    <div class="shrink-0 border-b border-hairline bg-muted px-3 py-1.5">
      <MatrixLegend :art="legendArt" />
    </div>
    <!-- **This is the matrix's viewport** (card #58): it takes the height the screen has
         left and scrolls on both axes, which is what lets the grid pin its boss header and
         its name column. `p-0` is not a tidy-up — a padding here would be scrolled past,
         and the rows would show in the strip above the pinned header. The grid holds its
         own inset instead. -->
    <CardContent class="min-h-0 flex-1 overflow-auto p-0">
      <MarksGrid :matrix="matrix" />
    </CardContent>
  </Card>
</template>
