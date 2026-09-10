<script setup lang="ts">
import ButtonSection from './sections/ButtonSection.vue'
import BadgeSection from './sections/BadgeSection.vue'
import CardSection from './sections/CardSection.vue'
import AlertSection from './sections/AlertSection.vue'
import EmptySection from './sections/EmptySection.vue'
import KbdSection from './sections/KbdSection.vue'
import SeparatorSection from './sections/SeparatorSection.vue'
import SkeletonSection from './sections/SkeletonSection.vue'
</script>

<template>
  <main
    class="grid min-h-screen grid-cols-3 content-start items-start gap-4 bg-background p-8 text-foreground"
  >
    <ButtonSection />
    <BadgeSection />
    <CardSection />
    <AlertSection />
    <EmptySection />
    <KbdSection />
    <SeparatorSection />
    <SkeletonSection />
  </main>
</template>
