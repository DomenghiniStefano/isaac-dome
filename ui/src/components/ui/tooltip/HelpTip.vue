<script setup lang="ts">
import { CircleHelpIcon } from '@lucide/vue'
import { useMessages } from '@/i18n'
import Tooltip from './Tooltip.vue'
import TooltipContent from './TooltipContent.vue'
import TooltipTrigger from './TooltipTrigger.vue'

// The one shape an explanation takes: a mark beside the thing it explains, and the sentence a
// hover away. Declared once so the four callers — the screen header, the sidebar's sections and
// the two settings screens — don't each assemble a tooltip by hand and drift apart.
//
// It is a button and not a `title` attribute. `title` is invisible to the keyboard, arrives
// after a second of hover with none of our styling, and never reaches a touch at all; the
// trigger below takes focus, so the sentence is reachable by Tab as well as by pointer.
//
// Wider than the default tooltip because what it holds is prose — the Plan's intro is two
// sentences, and `max-w-xs` would break it into a column.
defineProps<{ label?: string }>()
const { t } = useMessages()
</script>

<template>
  <Tooltip>
    <TooltipTrigger
      :aria-label="label ?? t('ui.explain')"
      class="shrink-0 text-faint-foreground hover:text-foreground"
    >
      <CircleHelpIcon class="size-3.5" />
    </TooltipTrigger>
    <TooltipContent class="max-w-110 text-row"><slot /></TooltipContent>
  </Tooltip>
</template>
