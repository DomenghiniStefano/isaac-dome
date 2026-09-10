<script setup lang="ts">
import { Checkbox, CheckboxState } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Checkbox">
    <div class="flex flex-wrap items-center gap-4">
      <Label
        ><Checkbox :default-value="CheckboxState.Checked" /> mancanti</Label
      >
      <Label><Checkbox /> spoiler</Label>
      <Label>
        <Checkbox :model-value="CheckboxState.Indeterminate" /> parziale
      </Label>
      <Label><Checkbox disabled /> bloccato</Label>
      <Label
        ><Checkbox :default-value="CheckboxState.Checked" disabled /> bloccato e
        fatto</Label
      >
    </div>
  </KitSection>
</template>
