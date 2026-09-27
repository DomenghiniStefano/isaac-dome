<script setup lang="ts">
import { provide, ref } from 'vue'
import { vScrollMemory } from '@/directives/scrollMemory'
import { PageScroller } from './pageScroller'

// A screen that scrolls as a whole: the band and the filters go by with the list under them,
// and only what the screen pins (`sticky top-0`, a table's column header) stays in view. Every
// `VirtualRows` inside finds this box through `PageScroller` and virtualizes against it, so a
// screen wraps its content here and does nothing else about scrolling.
//
// The position is this box's, kept by `v-scroll-memory` like any other screen's page: what was
// left is where the page was, band and filters included. The list's own offset cannot stand in
// for it — it is measured against a row count, and a card grid's count moves with the width its
// columns are measured from, so the first count after a back never matched and the page went
// back to the top.
//
// The gutter is the children's, not this box's: a padded box pins its sticky header one
// padding below its edge, and the rows would show through the strip above it.
const box = ref<HTMLElement | null>(null)
provide(PageScroller, box)
</script>

<template>
  <div ref="box" v-scroll-memory="'page'" class="h-full overflow-y-auto">
    <div class="flex flex-col">
      <slot />
    </div>
  </div>
</template>
