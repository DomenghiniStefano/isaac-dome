<script setup lang="ts">
import { EraserIcon } from '@lucide/vue'
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
import {
  ERASE_KEY,
  brushFor,
  paletteKey,
  paletteRows,
  roomFill,
} from '@/lib/floor/rooms'
import type { RoomKindView } from '@/lib/ipc/types'
import RoomSymbol from './RoomSymbol.vue'

// The brush, as fourteen swatches in three rows instead of fourteen rows of list. A swatch
// carries the colour and the drawing the cell will get, so what you pick and what you paint
// are the same picture — a list of names asks you to remember the mapping instead.
//
// The name is a hover away rather than printed on all fourteen: a label under every swatch is
// the list again, and the name is wanted once, while choosing. The tooltip is the repo's own
// and not a `title` attribute — that one is invisible to the keyboard and arrives a second
// late with none of our styling.
//
// Erasing is the same control with no kind picked, not a second mode: painting and erasing are
// one gesture with two values, and a separate eraser would let the screen be in a state where
// neither is chosen.

const props = defineProps<{
  brush: RoomKindView | null
  icons: Map<RoomKindView, string>
}>()
const emit = defineEmits<{ pick: [brush: RoomKindView | null] }>()
const { t } = useMessages()

const chosen = computed(() =>
  props.brush === null ? t('floor.erase') : t(`floor.room.${props.brush}`),
)

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
    <div
      v-for="(row, index) in paletteRows"
      :key="index"
      class="flex gap-floor-gap"
    >
      <Tooltip v-for="kind in row" :key="kind">
        <TooltipTrigger as-child>
          <Button
            :variant="ButtonVariant.Cell"
            :size="ButtonSize.Cell"
            class="relative"
            :class="[roomFill[kind], brush === kind ? 'border-highlight' : '']"
            :aria-pressed="brush === kind"
            :aria-label="t(`floor.room.${kind}`)"
            @click="emit('pick', kind)"
          >
            <RoomSymbol :kind="kind" :url="icons.get(kind) ?? null" />
            <span class="absolute right-0 bottom-0 text-micro">{{
              paletteKey[kind]
            }}</span>
          </Button>
        </TooltipTrigger>
        <TooltipContent class="flex items-center gap-2"
          ><span>{{ t(`floor.room.${kind}`) }}</span
          ><Kbd>{{ paletteKey[kind] }}</Kbd></TooltipContent
        >
      </Tooltip>

      <!-- The eraser takes the seat the last row leaves free: it is a brush like the others,
           and putting it anywhere else would make it a mode. -->
      <Tooltip v-if="index === paletteRows.length - 1">
        <TooltipTrigger as-child>
          <Button
            :variant="ButtonVariant.Cell"
            :size="ButtonSize.Cell"
            class="bg-floor-empty"
            :class="brush === null ? 'border-highlight' : ''"
            :aria-pressed="brush === null"
            :aria-label="t('floor.erase')"
            @click="emit('pick', null)"
          >
            <EraserIcon class="size-floor-symbol" />
          </Button>
        </TooltipTrigger>
        <TooltipContent class="flex items-center gap-2"
          ><span>{{ t('floor.erase') }}</span
          ><Kbd>{{ ERASE_KEY }}</Kbd></TooltipContent
        >
      </Tooltip>
    </div>

    <span class="text-caption text-subtle-foreground"
      >{{ t('floor.brush') }}: {{ chosen }}</span
    >
  </div>
</template>
