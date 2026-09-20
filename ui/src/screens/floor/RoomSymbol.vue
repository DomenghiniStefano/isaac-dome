<script setup lang="ts">
import { computed } from 'vue'
import { roomSymbol } from '@/lib/floor/rooms'
import type { RoomKindView } from '@/lib/ipc/types'

// The drawing a room carries when the game's own icon is not there to carry it. Stroke only
// and `currentColor`, so one path serves every fill: the cell decides the ink, this decides
// the shape.
//
// It draws nothing for the Normal Room, and that is a rendered answer rather than a missing
// one — `v-if` on the path, not a caller who has to remember which kind is bare.

const props = defineProps<{ kind: RoomKindView }>()

const path = computed(() => roomSymbol[props.kind])
</script>

<template>
  <svg
    v-if="path !== ''"
    class="size-floor-symbol stroke-2"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
  >
    <path :d="path" />
  </svg>
</template>
