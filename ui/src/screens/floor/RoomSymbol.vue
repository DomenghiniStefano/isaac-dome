<script setup lang="ts">
import { computed } from 'vue'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { roomSymbol } from '@/lib/floor/rooms'
import type { RoomKindView } from '@/lib/ipc/types'

// What a room wears inside its cell: **the game's own minimap icon when the game is here**,
// and our drawing when it is not.
//
// The two are not a preference, they are the same screen on two machines. The icon is the
// picture the player already reads on the minimap; the path is stroke-only and follows
// `currentColor`, so one drawing serves every fill — the cell decides the ink, this decides
// the shape.
//
// The fallback is a slot rather than a branch on purpose: an icon that fails to load falls
// back to exactly the same drawing as an icon that was never there.

const props = defineProps<{ kind: RoomKindView; url?: string | null }>()

const path = computed(() => roomSymbol[props.kind])
</script>

<template>
  <!-- The box is the cell's own size in both branches, so the cell does not jump when the game
       is there; the drawing and the icon are each centred inside it.

       **The zoom is on the picture and not on the box.** The icon arrives trimmed to its own
       drawing (`ipc::trim_opaque`), so it is a different number of pixels wide for every room
       kind and there is no width to write down — it is drawn at twice its own pixels, whatever
       those are, and the box centres what comes out. The fallback drawing keeps its own size
       and must not be doubled, which is why the utility is aimed at the image alone. -->
  <div
    class="grid size-floor-icon shrink-0 place-items-center [&>img]:pixel-2x"
  >
    <PixelSprite :url="url ?? null">
      <template #fallback>
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
    </PixelSprite>
  </div>
</template>
