<script setup lang="ts">
import { useShortcut } from '@/composables/useShortcut'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Kbd } from '@/components/ui/kbd'
import { useMessages } from '@/i18n'
import { isTyping } from '@/lib/keyboard/typing'
import { brushFor, paletteKey, paletteOrder, roomFill } from '@/lib/floor/rooms'
import type { RoomKindView } from '@/lib/ipc/types'
import RoomSymbol from './RoomSymbol.vue'

// The brush: fourteen rooms, one per line, beside the grid they paint.
//
// It was a single row of swatches above the grid, and it cost the name. Fourteen squares wide
// leaves no room to write anything, so the name was a hover away and the shortcut was a digit
// printed on top of the room's own colour — barely there on the Treasure Room's gold. A column
// has the width the row never had: **the key, the picture and the name on the same line**, and
// nothing has to be remembered or hovered to be read.
//
// It sits on the right because the right-hand column was holding the least of the screen while
// the grid's own controls crowded the top of the left one. The order is the keys' order, 1 to 9
// and on, which is only a reading order if there is a direction to read in — down the column
// now rather than across the row.
//
// **The chosen one is marked outside its own fill**: a cream border on the cream Normal Room is
// a border nobody sees, and that is the row the screen opens on. An outline with an offset sits
// on the card behind the row, where one colour reads against all fourteen.

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
  <div class="flex flex-col gap-1">
    <Button
      v-for="kind in paletteOrder"
      :key="kind"
      :variant="ButtonVariant.Ghost"
      :size="ButtonSize.Row"
      class="gap-3"
      :class="
        brush === kind ? 'outline-2 -outline-offset-2 outline-highlight' : ''
      "
      :aria-pressed="brush === kind"
      @click="emit('pick', kind)"
    >
      <Kbd>{{ paletteKey[kind] }}</Kbd>
      <!-- The swatch is the cell this brush paints, at the size it will be: the colour and the
           drawing together, so what you pick and what lands on the grid are one picture. -->
      <span
        aria-hidden="true"
        class="grid size-floor-cell shrink-0 place-items-center"
        :class="roomFill[kind]"
      >
        <RoomSymbol :kind="kind" :url="icons.get(kind) ?? null" />
      </span>
      <span>{{ t(`floor.room.${kind}`) }}</span>
    </Button>
  </div>
</template>
