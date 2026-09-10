<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <!-- A focused row keeps the ring inside, so it doesn't push its neighbours. -->
  <tr
    data-slot="table-row"
    :class="
      cn(
        'border-b border-hairline even:bg-row-alt hover:bg-row-hover focus-visible:-outline-offset-2 data-[state=selected]:bg-row-hover',
        props.class,
      )
    "
  >
    <slot />
  </tr>
</template>
