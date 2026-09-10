<script setup lang="ts">
import { Button, ButtonVariant } from '@/components/ui/button'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Tooltip">
    <Tooltip>
      <TooltipTrigger as-child>
        <Button :variant="ButtonVariant.Outline" class="w-fit"
          >Passa qui</Button
        >
      </TooltipTrigger>
      <TooltipContent>Bloccato da 2 voci</TooltipContent>
    </Tooltip>
  </KitSection>
</template>
