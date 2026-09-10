<script setup lang="ts">
import ButtonSection from './sections/ButtonSection.vue'
import KbdSection from './sections/KbdSection.vue'
import SeparatorSection from './sections/SeparatorSection.vue'
import SkeletonSection from './sections/SkeletonSection.vue'
</script>

<template>
  <main
    class="grid min-h-screen grid-cols-3 items-start gap-4 bg-background p-8 text-foreground"
  >
    <ButtonSection />
    <KbdSection />
    <SeparatorSection />
    <SkeletonSection />
  </main>
</template>
