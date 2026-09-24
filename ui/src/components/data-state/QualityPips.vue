<script setup lang="ts">
import { cn } from '@/lib/cn'

defineProps<{ quality: number | null }>()

// items_metadata.xml rates collectibles 0 to 4: four pips, as many filled as the quality, and
// the number beside them so the pips are never the only way to read it.
const pips = [1, 2, 3, 4]
</script>

<template>
  <span v-if="quality === null" class="text-row text-subtle-foreground">{{
    '—'
  }}</span>
  <span v-else class="flex items-center gap-1">
    <span
      v-for="pip in pips"
      :key="pip"
      aria-hidden="true"
      :class="
        cn(
          'size-quality-pip shrink-0',
          pip <= quality ? 'bg-foreground-soft' : 'bg-hairline',
        )
      "
    />
    <span class="ml-1 text-label text-foreground tabular-nums">{{
      quality
    }}</span>
  </span>
</template>
