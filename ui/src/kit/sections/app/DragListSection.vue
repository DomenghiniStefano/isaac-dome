<script setup lang="ts">
import { computed, ref } from 'vue'
import { Card } from '@/components/ui/card'
import { DragGhost } from '@/components/ui/drag'
import { useDragList } from '@/composables/useDragList'
import { Axis, boxAt } from '@/lib/drag/dragList'
import type { Box, Point } from '@/lib/drag/dragList'
import KitSection from '../../KitSection.vue'

const rows = ref(['Isaac', 'Magdalene', 'Cain'])
const list = ref<HTMLElement | null>(null)

const items = (): HTMLElement[] =>
  list.value
    ? [...list.value.querySelectorAll<HTMLElement>('[data-kit-row]')]
    : []

const drag = useDragList<number>({
  axis: Axis.Y,
  container: list,
  items,
  resolve: (p: Point, boxes: Box[]) => boxAt(boxes, p, Axis.Y),
  commit: (from, to) => {
    if (to === null || to === from) return
    const next = [...rows.value]
    const [row] = next.splice(from, 1)
    if (row) next.splice(to, 0, row)
    rows.value = next
  },
})

const grabbed = computed(() =>
  drag.from.value === null ? null : rows.value[drag.from.value],
)
</script>

<template>
  <KitSection title="DragList">
    <Card class="w-80">
      <div
        ref="list"
        class="flex flex-col"
        @pointermove="drag.move"
        @pointerup="drag.end"
        @pointercancel="drag.end"
      >
        <div
          v-for="(row, index) in rows"
          :key="row"
          data-kit-row
          class="flex h-10 cursor-grab items-center border-b border-hairline px-3 text-label last:border-b-0"
          :class="
            drag.moving.value && drag.from.value === index && 'opacity-disabled'
          "
          @pointerdown="drag.start(index, $event)"
        >
          {{ row }}
        </div>
      </div>
    </Card>
    <DragGhost v-if="drag.ghost.value && grabbed" :box="drag.ghost.value">
      <div class="flex h-full items-center px-3 text-label">{{ grabbed }}</div>
    </DragGhost>
  </KitSection>
</template>
