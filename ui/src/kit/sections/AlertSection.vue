<script setup lang="ts">
import { InfoIcon, TriangleAlertIcon } from '@lucide/vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Alert">
    <Alert>
      <InfoIcon />
      <AlertTitle>Salvataggio riletto</AlertTitle>
      <AlertDescription>Rilette le voci dello slot Rep+ 1.</AlertDescription>
    </Alert>
    <Alert :variant="AlertVariant.Destructive">
      <TriangleAlertIcon />
      <AlertTitle>Catalogo assente</AlertTitle>
      <AlertDescription>
        Senza il gioco installato non si sa cosa sblocca cosa.
      </AlertDescription>
    </Alert>
  </KitSection>
</template>
