<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { computed, inject } from 'vue'
import { cn } from '@/lib/cn'
import { TableDensity, tableBodyVariants, tableDensityKey } from './variants'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()

const tableDensity = inject(tableDensityKey, undefined)
const density = computed(() => tableDensity?.value ?? TableDensity.Normal)
</script>

<template>
  <tbody
    data-slot="table-body"
    :class="cn(tableBodyVariants({ density }), props.class)"
  >
    <slot />
  </tbody>
</template>
