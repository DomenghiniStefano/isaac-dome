<script setup lang="ts">
import { useShortcut } from '@/composables/useShortcut'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Kbd } from '@/components/ui/kbd'
import { useMessages } from '@/i18n'
import { isTyping } from '@/lib/keyboard/typing'
import { brushFor, paletteKey, paletteOrder, roomFill } from '@/lib/floor/rooms'
import type { RoomKindView } from '@/lib/ipc/types'
import RoomSymbol from './RoomSymbol.vue'

// The brush: fourteen rooms, one per line, against the grid they paint.
//
// **Third arrangement, and the one a drawing program has.** It was a row of swatches above the
// grid, which had no width for a name; then a card of its own on the far side of the screen,
// which had the width and put the tool a page away from the canvas — at most widths the card
// wrapped under the grid, and picking a room meant scrolling past the drawing to reach it. It is
// a tool rail now, attached to the grid's left edge.
//
// **A row is exactly a cell tall, with the grid's own gap between rows**, so the rail's lines run
// level with the grid's and the fourteenth sits beside the controls under it. The swatch is the
// cell this brush paints at the size it will land, the name beside it, the key at the far end
// where a menu keeps its shortcuts.
//
// **The chosen one is the row filled**, the way the sidebar marks the page you are on. An outline
// around the swatch was the earlier mark, and a cream edge on the cream Normal Room — the row the
// screen opens on — is an edge nobody sees; a filled row reads against all fourteen.

defineProps<{
  brush: RoomKindView
  icons: Map<RoomKindView, string>
}>()
const emit = defineEmits<{ pick: [brush: RoomKindView] }>()
const { t } = useMessages()

useShortcut((event) => {
  if (event.ctrlKey || event.altKey || event.metaKey) return false
  if (isTyping(event.target)) return false
  const brush = brushFor(event.key)
  if (brush === undefined) return false
  emit('pick', brush)
  return true
})
</script>

<template>
  <div
    class="grid content-start gap-floor-gap"
    role="group"
    :aria-label="t('floor.rooms')"
  >
    <Button
      v-for="kind in paletteOrder"
      :key="kind"
      :variant="ButtonVariant.Ghost"
      :size="ButtonSize.Brush"
      class="aria-pressed:bg-primary aria-pressed:hover:bg-primary"
      :aria-pressed="brush === kind"
      @click="emit('pick', kind)"
    >
      <span
        aria-hidden="true"
        class="grid size-floor-cell shrink-0 place-items-center"
        :class="roomFill[kind]"
      >
        <RoomSymbol :kind="kind" :url="icons.get(kind) ?? null" />
      </span>
      <span class="flex-1 truncate text-left">{{
        t(`floor.room.${kind}`)
      }}</span>
      <Kbd>{{ paletteKey[kind] }}</Kbd>
    </Button>
  </div>
</template>
