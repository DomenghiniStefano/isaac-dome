<script setup lang="ts">
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Switch">
    <div class="flex flex-col gap-3">
      <Label><Switch :default-value="true" /> Riapri le tab all'avvio</Label>
      <Label><Switch /> Apri in secondo piano</Label>
      <Label><Switch disabled /> Non disponibile</Label>
      <Label
        ><Switch :default-value="true" disabled /> Acceso ma bloccato</Label
      >
    </div>
  </KitSection>
</template>
