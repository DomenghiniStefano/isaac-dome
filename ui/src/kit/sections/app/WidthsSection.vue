<script setup lang="ts">
import type { UnlockNode } from '@/lib/ipc/types'
import UnlockTable from '@/screens/unlock/UnlockTable.vue'
import KitSection from '../../KitSection.vue'

// Three boxes, each its own `page` container, so a compact layout can be judged without resizing
// the window — the thing container queries buy and media queries could not (spec 3.13a §11). The
// widths straddle all three: 500 is below compact (800), 900 is past it and below regular (960),
// and 1300 is past wide (1280).
const widths = [500, 900, 1300]

const node = (id: number, text: string, done: boolean): UnlockNode => ({
  achievement: {
    kind: 'known',
    id,
    text,
    condition: 'Sconfiggi Mom con Isaac',
    iconUrl: null,
  },
  done,
  unlocks: [],
  origin: null,
  missing: [],
  graph: {
    kind: 'computed',
    availableNow: !done,
    blockedBy: 0,
    fanOut: 3,
    stepsMissing: 0,
  },
})

// The third one is long on purpose: at 400 the name column is what is left after the drawing, the
// state and the button, and truncation is the thing to look at.
const nodes = [
  node(1, 'Il Cubo di Ghiaccio', false),
  node(2, 'La Sacca del Diavolo', true),
  node(
    3,
    'Un nome lungo che deve troncarsi quando la colonna si stringe',
    false,
  ),
]
const queued = new Set<number>()
</script>

<template>
  <KitSection title="Larghezze" class="col-span-3">
    <div class="flex items-start gap-4 overflow-x-auto">
      <div
        v-for="width in widths"
        :key="width"
        :style="{ '--kit-width': `${width}px` }"
        class="@container/page flex h-80 w-(--kit-width) shrink-0 flex-col border border-border"
      >
        <UnlockTable
          :nodes="nodes"
          :queued="queued"
          :can-write="false"
          :busy="false"
          :offset="null"
        />
      </div>
    </div>
  </KitSection>
</template>
