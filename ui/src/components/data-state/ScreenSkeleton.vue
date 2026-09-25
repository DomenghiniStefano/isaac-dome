<script setup lang="ts">
import { Skeleton } from '@/components/ui/skeleton'
import { SkeletonBlock } from './skeletonBlock'

// A screen that has not answered yet: its title bar, then a block for each part it will draw.
// A filtered list — the filter bar and the list under it — is the shape most screens have.
// `untitled` is for a skeleton that stands inside a screen whose title is already drawn, or in
// place of a screen that has none.
withDefaults(defineProps<{ blocks?: SkeletonBlock[]; untitled?: boolean }>(), {
  blocks: () => [SkeletonBlock.FilterBar, SkeletonBlock.List],
  untitled: false,
})
</script>

<template>
  <div class="flex flex-col gap-4">
    <Skeleton v-if="!untitled" class="h-8 w-120" />
    <Skeleton v-for="(block, index) in blocks" :key="index" :class="block" />
  </div>
</template>
