<script setup lang="ts">
import {
  CardCollapsible,
  CardCollapsibleContent,
  CardCollapsibleTrigger,
} from '@/components/ui/card'
import KitSection from '../../KitSection.vue'
</script>

<template>
  <KitSection title="Card comprimibile">
    <CardCollapsible :default-open="true">
      <CardCollapsibleTrigger>
        Cambia profilo
        <template #summary>slot 1</template>
      </CardCollapsibleTrigger>
      <CardCollapsibleContent>
        Aperta: il contenuto sta dentro il bordo della card, con lo stesso passo
        di 12px della testata.
      </CardCollapsibleContent>
    </CardCollapsible>
    <CardCollapsible>
      <CardCollapsibleTrigger>
        Rileggi il file
        <template #summary>2 giorni fa</template>
      </CardCollapsibleTrigger>
      <CardCollapsibleContent>
        Il salvataggio viene riletto da capo.
      </CardCollapsibleContent>
    </CardCollapsible>
  </KitSection>
</template>
