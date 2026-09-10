<script setup lang="ts">
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'

defineProps<{ active: boolean }>()
</script>

<template>
  <!-- The active item has the red bar on the left (ButtonVariant.Nav). -->
  <Button
    :variant="ButtonVariant.Nav"
    :size="ButtonSize.Row"
    :aria-current="active ? 'page' : undefined"
    class="gap-2.25 [&_svg]:size-4 [&_svg]:text-subtle-foreground aria-[current=page]:[&_svg]:text-foreground"
  >
    <slot name="icon" />
    <slot />
  </Button>
</template>
