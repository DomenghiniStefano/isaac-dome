<script setup lang="ts">
import { Separator } from '@/components/ui/separator'
import { Orientation } from '@/lib/constants/placement'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Separator">
    <p class="text-body text-foreground-soft">Sopra il separatore</p>
    <Separator />
    <div class="flex h-6 items-center gap-3 text-body text-foreground-soft">
      <span>Sinistra</span>
      <Separator :orientation="Orientation.Vertical" />
      <span>Destra</span>
    </div>
  </KitSection>
</template>
