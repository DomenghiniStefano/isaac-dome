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
    <!-- The only bar in the repo with a hatched segment, so it is the only one that has to
         say so out loud: `value-text` becomes `aria-valuetext` and replaces the percentage a
         screen reader would compute, which would count the 40 as not done. -->
    <Progress
      :model-value="166"
      :unknown="40"
      :max="408"
      value-text="166 su 408, 40 non leggibili"
    />
    <p class="text-label text-muted-foreground">
      40 celle su 408 non sono leggibili · nessuna percentuale totale
    </p>
  </KitSection>
</template>
