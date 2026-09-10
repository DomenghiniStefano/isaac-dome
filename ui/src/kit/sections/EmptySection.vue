<script setup lang="ts">
import { SkullIcon } from '@lucide/vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyMedia,
  EmptyTitle,
} from '@/components/ui/empty'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Empty">
    <Empty>
      <EmptyMedia><SkullIcon /></EmptyMedia>
      <EmptyTitle>Niente qui</EmptyTitle>
      <EmptyDescription
        >Nessuna voce corrisponde ai filtri attivi.</EmptyDescription
      >
      <EmptyContent>
        <Button :variant="ButtonVariant.Outline">Azzera filtri</Button>
      </EmptyContent>
    </Empty>
  </KitSection>
</template>
