<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { provide, toRef } from 'vue'
import { cn } from '@/lib/cn'
import { TableDensity, tableDensityKey } from './variants'

const props = withDefaults(
  defineProps<{
    class?: HTMLAttributes['class']
    density?: TableDensity
  }>(),
  {
    density: TableDensity.Normal,
  },
)

provide(tableDensityKey, toRef(props, 'density'))
</script>

<template>
  <div
    data-slot="table-container"
    class="relative w-full overflow-x-auto border border-border bg-data"
  >
    <table
      data-slot="table"
      :class="cn('w-full text-row text-foreground', props.class)"
    >
      <slot />
    </table>
  </div>
</template>
