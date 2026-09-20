<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps<{ url: string | null; placeholder?: boolean }>()

// A picture that fails to load reads like one that isn't there: the placeholder, or nothing,
// never a broken-image glyph. The failure belongs to the URL, so a new URL tries again.
const failed = ref(false)
watch(
  () => props.url,
  () => {
    failed.value = false
  },
)
</script>

<template>
  <!-- A game sprite is pixel art; its size comes from the parent's class, on either root. -->
  <img
    v-if="url && !failed"
    :src="url"
    alt=""
    class="pixelated"
    @error="failed = true"
  />
  <!-- What stands in for the picture. Empty by default, the hatch when the caller asks for
       it, and whatever the caller puts in the slot when it has something better — the Floor
       grid has its own drawing for every room, and an empty square there would be a cell
       that says nothing rather than one the game has no icon for. -->
  <span
    v-else
    aria-hidden="true"
    :class="placeholder ? 'hatch-placeholder' : undefined"
    ><slot name="fallback"
  /></span>
</template>
