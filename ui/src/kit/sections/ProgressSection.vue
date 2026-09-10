<script setup lang="ts">
import { Progress } from '@/components/ui/progress'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Progress">
    <div class="flex items-baseline justify-between">
      <span class="text-caption text-foreground-soft"
        >Marchi almeno iniziati</span
      >
      <span class="text-control">166 / 368 leggibili</span>
    </div>
    <Progress :model-value="166" :unknown="40" :max="408" />
    <p class="text-label text-muted-foreground">
      40 celle su 408 non sono leggibili · nessuna percentuale totale
    </p>
  </KitSection>
</template>
