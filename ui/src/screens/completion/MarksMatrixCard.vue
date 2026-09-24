<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { computed } from 'vue'
import MarksGrid from '@/components/marks/MarksGrid.vue'
import { markArtOf } from '@/components/marks/markVisual'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useMessages } from '@/i18n'
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
  <Card :class="props.class">
    <CardHeader>
      <CardTitle>{{ t('completion.card.title') }}</CardTitle>
    </CardHeader>
    <div class="shrink-0 border-b border-hairline bg-muted px-3 py-1.5">
      <MatrixLegend :art="legendArt" />
    </div>
    <!-- Not a scrolling box any more (card #85): the page scrolls, and the grid pins its
         boss header to the top of the screen and scrolls sideways on its own. `p-0` is not
         a tidy-up — the grid's rows paint edge to edge and it holds its own inset. -->
    <CardContent class="p-0">
      <MarksGrid :matrix="matrix" />
    </CardContent>
  </Card>
</template>
