<script setup lang="ts">
import { computed } from 'vue'
import MarksGrid from '@/components/marks/MarksGrid.vue'
import { markArtOf } from '@/components/marks/markVisual'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import type { MarksMatrix } from '@/lib/ipc/types'
import MatrixLegend from './MatrixLegend.vue'

const props = defineProps<{ matrix: MarksMatrix }>()
const { t } = useMessages()

// The legend draws with the first column's art, when there is some.
const legendArt = computed(() => markArtOf(props.matrix.art[0]))
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle>{{ t('completion.card.title') }}</CardTitle>
    </CardHeader>
    <div class="border-b border-hairline bg-muted px-3 py-1.5">
      <MatrixLegend :art="legendArt" />
    </div>
    <CardContent>
      <MarksGrid :matrix="matrix" />
    </CardContent>
  </Card>
</template>
