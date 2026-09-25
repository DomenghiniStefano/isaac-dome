<script setup lang="ts">
import { computed } from 'vue'
import ScreenSkeleton from '@/components/data-state/ScreenSkeleton.vue'
import { SkeletonBlock } from '@/components/data-state/skeletonBlock'
import { gateState } from '@/lib/profile/gateView'
import { useProfileStore } from '@/stores/profile'

const profile = useProfileStore()

// Every decision is in `gateView`, where it can be tested: this file only draws the answer.
// Two states since 3.8 — the welcome, above the router, owns the other three, so a screen
// that needs a profile is only ever mounted with one or a moment before knowing.
const state = computed(() => gateState(profile.setup))
</script>

<template>
  <slot v-if="state.kind === 'content'" />
  <!-- The shell's page box pads nothing: the gutter is the screen's, and this stands in for one. -->
  <ScreenSkeleton
    v-else
    untitled
    class="px-5.5 pt-5"
    :blocks="[SkeletonBlock.Band, SkeletonBlock.TallCard]"
  />
</template>
