<script setup lang="ts">
import { PlusIcon } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Button">
    <div class="flex flex-wrap items-center gap-2">
      <Button>Default</Button>
      <Button :variant="ButtonVariant.Secondary">Secondary</Button>
      <Button :variant="ButtonVariant.Outline">Outline</Button>
      <Button :variant="ButtonVariant.Ghost">Ghost</Button>
      <Button :variant="ButtonVariant.Link">Link</Button>
      <Button disabled>Disabled</Button>
      <Button
        :variant="ButtonVariant.Secondary"
        :size="ButtonSize.Icon"
        aria-label="Aggiungi"
      >
        <PlusIcon />
      </Button>
    </div>
  </KitSection>
</template>
