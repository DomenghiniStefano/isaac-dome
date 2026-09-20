<script setup lang="ts">
import { computed } from 'vue'
import { useShortcut } from '@/composables/useShortcut'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Kbd } from '@/components/ui/kbd'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { useMessages } from '@/i18n'
import { isTyping } from '@/lib/keyboard/typing'
import { brushFor, paletteKey, paletteOrder, roomFill } from '@/lib/floor/rooms'
import type { RoomKindView } from '@/lib/ipc/types'
import RoomSymbol from './RoomSymbol.vue'

// The brush: fourteen swatches in one row, above the grid they paint. A swatch carries the
// colour and the drawing the cell will get, so what you pick and what you paint are the same
// picture — a list of names asks you to remember the mapping instead.
//
// One row and not three. Three rows are a list wearing a grid's clothes: the eye has to find
// the row before it finds the swatch, and the keys already run 1 to 9 left to right, which is
// only true if there is a left to right.
//
// **The chosen one is marked outside its own fill**, not on it: a cream border on the cream
// Normal Room is a border nobody sees, and that is the swatch the screen opens on. An outline
// with an offset sits on the card behind the row, where one colour reads against all fourteen.
//
// The name is a hover away rather than printed on all fourteen — a label under every swatch is
// the list again, and the name is wanted once, while choosing. The tooltip is the repo's own
// and not a `title` attribute: that one is invisible to the keyboard and arrives a second late
// with none of our styling.
//
// **The key sits above its swatch, not inside it.** It used to be a micro digit in the
// bottom-right corner of the square, and it cost twice: the digit landed on whatever colour
// the room happened to be, so on the Treasure Room's gold it was barely there, and the icon
// had to leave it room instead of sitting in the middle. Above the square it is on the card's
// own surface, always the same contrast, and the swatch below it holds nothing but the room.

const props = defineProps<{
  brush: RoomKindView
  icons: Map<RoomKindView, string>
}>()
const emit = defineEmits<{ pick: [brush: RoomKindView] }>()
const { t } = useMessages()

const chosen = computed(() => t(`floor.room.${props.brush}`))

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
  <div class="flex flex-col gap-1.5">
    <div class="flex gap-floor-gap">
      <div
        v-for="kind in paletteOrder"
        :key="kind"
        class="flex flex-col items-center gap-0.5"
      >
        <!-- The button already says the room's name out loud; read on its own this would be a
             stray letter between two of them. -->
        <span aria-hidden="true" class="text-micro text-subtle-foreground">{{
          paletteKey[kind]
        }}</span>
        <Tooltip>
          <TooltipTrigger as-child>
            <Button
              :variant="ButtonVariant.Cell"
              :size="ButtonSize.Cell"
              :class="[
                roomFill[kind],
                brush === kind
                  ? 'outline-2 outline-offset-1 outline-highlight'
                  : '',
              ]"
              :aria-pressed="brush === kind"
              :aria-label="t(`floor.room.${kind}`)"
              @click="emit('pick', kind)"
            >
              <RoomSymbol :kind="kind" :url="icons.get(kind) ?? null" />
            </Button>
          </TooltipTrigger>
          <TooltipContent class="flex items-center gap-2"
            ><span>{{ t(`floor.room.${kind}`) }}</span
            ><Kbd>{{ paletteKey[kind] }}</Kbd></TooltipContent
          >
        </Tooltip>
      </div>
    </div>

    <span class="text-caption text-subtle-foreground"
      >{{ t('floor.brush') }}: {{ chosen }}</span
    >
  </div>
</template>
