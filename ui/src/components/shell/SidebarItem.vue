<script setup lang="ts">
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { AriaCurrent } from '@/lib/constants/aria'

// The label is a **value** and not the default slot, because a collapsed sidebar hides the text
// and the row still needs a name: `aria-label` carries it, and a slot cannot be read as one.
//
// No tooltip here. It repeated, in a floating box, the very text the row already shows — and it
// could not be mounted only when the sidebar is collapsed, because whether it is collapsed is a
// CSS fact (spec 3.13a §6) that JavaScript never learns. So it was noise on every hover of the
// open sidebar to serve the narrow one, and the narrow one is served by the accessible name.
defineProps<{ active: boolean; label: string }>()
</script>

<template>
  <!-- The active item has the red bar on the left (ButtonVariant.Nav). -->
  <Button
    :variant="ButtonVariant.Nav"
    :size="ButtonSize.Row"
    :aria-current="active ? AriaCurrent.Page : undefined"
    :aria-label="label"
    class="sidebar-collapsed-center gap-2.25 [&_svg]:size-4 [&_svg]:text-subtle-foreground aria-[current=page]:[&_svg]:text-foreground"
  >
    <slot name="icon" />
    <span class="sidebar-collapsed-hidden truncate">{{ label }}</span>
  </Button>
</template>
