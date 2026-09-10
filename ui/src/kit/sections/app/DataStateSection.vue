<script setup lang="ts">
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import UnreadableValue from '@/components/data-state/UnreadableValue.vue'
import KitSection from '../../KitSection.vue'
</script>

<template>
  <KitSection title="Stati del dato">
    <span class="text-row text-foreground-soft">6 stanze</span>
    <EmptyValue>non nel dataset</EmptyValue>
    <UnreadableValue>il salvataggio non ce lo dice</UnreadableValue>
    <EmptyCategory>Nessuna pagina di Trinket fra i campioni.</EmptyCategory>
  </KitSection>
</template>
