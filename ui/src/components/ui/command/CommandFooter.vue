<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <!-- Ours, not the registry's: the palette's count and key hints (Shadcn Kit.dc.html). -->
  <div
    data-slot="command-footer"
    :class="
      cn(
        'flex items-center justify-between gap-2 border-t border-hairline px-3 py-2 text-label text-subtle-foreground',
        props.class,
      )
    "
  >
    <slot />
  </div>
</template>
