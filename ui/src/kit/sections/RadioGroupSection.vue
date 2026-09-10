<script setup lang="ts">
import { Label } from '@/components/ui/label'
import { RadioGroup, RadioGroupItem } from '@/components/ui/radio-group'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Radio group">
    <RadioGroup default-value="normal">
      <Label><RadioGroupItem value="normal" /> Normale</Label>
      <Label><RadioGroupItem value="hard" /> Hard</Label>
      <Label><RadioGroupItem value="greed" /> Greed</Label>
    </RadioGroup>
    <RadioGroup default-value="hard" disabled>
      <Label><RadioGroupItem value="normal" /> bloccato</Label>
      <Label><RadioGroupItem value="hard" /> bloccato e scelto</Label>
    </RadioGroup>
  </KitSection>
</template>
