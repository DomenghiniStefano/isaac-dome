<script setup lang="ts">
import { ref } from 'vue'
import { Slider } from '@/components/ui/slider'
import KitSection from '../KitSection.vue'

// Positions, not values: the ladder the app drives it with is unevenly spaced, and the
// slider's own spacing is even (Schermate.dc.html is silent, Discord's zoom is the model).
const step = ref([3])
const steps = ['50%', '75%', '100%', '150%', '200%']
</script>

<template>
  <KitSection title="Slider">
    <div class="flex w-80 flex-col gap-2">
      <Slider v-model="step" :min="0" :max="4" :step="1" />
      <div class="flex justify-between text-label text-subtle-foreground">
        <span v-for="(label, index) in steps" :key="label">{{
          index === step[0] ? label : '·'
        }}</span>
      </div>
    </div>
    <Slider
      :model-value="[2]"
      :min="0"
      :max="4"
      :step="1"
      disabled
      class="w-80"
    />
  </KitSection>
</template>
