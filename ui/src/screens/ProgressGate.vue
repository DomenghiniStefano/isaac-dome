<script setup lang="ts">
import { computed } from 'vue'
import { Skeleton } from '@/components/ui/skeleton'
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
  <div v-else class="flex flex-col gap-4">
    <Skeleton class="h-10 w-full" />
    <Skeleton class="h-50 w-full" />
  </div>
</template>
