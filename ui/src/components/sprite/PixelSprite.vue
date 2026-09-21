<script setup lang="ts">
import { computed, ref, useSlots, watch } from 'vue'
import { SpriteLayer, layerFor } from './pixelSprite'
import { useUnknownIcon } from './unknownIcon'

const props = defineProps<{ url: string | null; placeholder?: boolean }>()
const slots = useSlots()

// A picture that fails to load reads like one that isn't there: the stand-in, the
// placeholder, or nothing, never a broken-image glyph. The failure belongs to the URL, so a
// new URL tries again.
const failed = ref(false)
const standInFailed = ref(false)
watch(
  () => props.url,
  () => {
    failed.value = false
  },
)

const standIn = useUnknownIcon()
// Which of the three is drawn is decided in `pixelSprite.ts`, where it can be checked.
const layer = computed(() =>
  layerFor({
    url: props.url,
    failed: failed.value,
    standIn: standIn.value,
    standInFailed: standInFailed.value,
    hasOwnFallback: slots.fallback !== undefined,
  }),
)
</script>

<template>
  <!-- A game sprite is pixel art; its size comes from the parent's class, on either root. -->
  <img
    v-if="layer === SpriteLayer.Picture"
    :src="url ?? undefined"
    alt=""
    class="pixelated"
    @error="failed = true"
  />
  <!-- The game's own picture for an item you are not allowed to see, standing in for one we
       could not resolve: an empty square says "this thing has no picture", which is a
       statement about the game where the truth is about us (B69). It is the user's own file
       too, so it can fail as well, and then what is left is the square below. -->
  <img
    v-else-if="layer === SpriteLayer.StandIn"
    :src="standIn ?? undefined"
    alt=""
    class="pixelated"
    @error="standInFailed = true"
  />
  <!-- What stands in for the picture. Empty by default, the hatch when the caller asks for
       it, and whatever the caller puts in the slot when it has something better — the Floor
       grid has its own drawing for every room, and an empty square there would be a cell
       that says nothing rather than one the game has no icon for. That drawing wins over
       the question mark for the same reason: it answers a different question. -->
  <span
    v-else
    aria-hidden="true"
    :class="placeholder ? 'hatch-placeholder' : undefined"
    ><slot name="fallback"
  /></span>
</template>
