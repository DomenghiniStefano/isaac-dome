<script setup lang="ts">
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { AriaCurrent } from '@/lib/constants/aria'

// The label is a **value** and no longer the default slot, because the tooltip needs the same
// text the row shows and a slot cannot be read as one.
//
// The tooltip is mounted always, not only when the sidebar is collapsed: whether it is collapsed
// is a CSS fact (spec 3.13a §6) that JavaScript never learns, so "only when collapsed" would need
// exactly the measurement §2 rejected. It earns its place either way — a label in a sidebar the
// user narrowed is truncated, and wants the tooltip as much as a hidden one does.
defineProps<{ active: boolean; label: string }>()
</script>

<template>
  <Tooltip>
    <TooltipTrigger as-child>
      <!-- The active item has the red bar on the left (ButtonVariant.Nav). -->
      <Button
        :variant="ButtonVariant.Nav"
        :size="ButtonSize.Row"
        :aria-current="active ? AriaCurrent.Page : undefined"
        class="gap-2.25 [&_svg]:size-4 [&_svg]:text-subtle-foreground aria-[current=page]:[&_svg]:text-foreground sidebar-collapsed-center"
      >
        <slot name="icon" />
        <span class="sidebar-collapsed-hidden truncate">{{ label }}</span>
      </Button>
    </TooltipTrigger>
    <TooltipContent>{{ label }}</TooltipContent>
  </Tooltip>
</template>
