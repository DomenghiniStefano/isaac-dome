<script setup lang="ts">
import { Skeleton } from '@/components/ui/skeleton'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Skeleton">
    <div class="flex items-center gap-3">
      <Skeleton class="size-10" />
      <div class="flex flex-1 flex-col gap-2">
        <Skeleton class="h-3 w-3/4" />
        <Skeleton class="h-3 w-1/2" />
      </div>
    </div>
  </KitSection>
</template>
