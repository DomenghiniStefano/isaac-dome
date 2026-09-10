<script setup lang="ts">
import { ChevronDownIcon } from '@lucide/vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Checkbox, CheckboxState } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import {
  Popover,
  PopoverContent,
  PopoverTitle,
  PopoverTrigger,
} from '@/components/ui/popover'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Popover">
    <Popover>
      <PopoverTrigger as-child>
        <Button :variant="ButtonVariant.Secondary" class="w-fit">
          Filtri
          <ChevronDownIcon class="size-3" />
        </Button>
      </PopoverTrigger>
      <PopoverContent>
        <PopoverTitle>Filtri</PopoverTitle>
        <div class="flex flex-col gap-2.5 p-3">
          <Label>
            <Checkbox :default-value="CheckboxState.Checked" /> Sbloccabili ora
          </Label>
          <Label><Checkbox /> Solo Hard mode</Label>
        </div>
      </PopoverContent>
    </Popover>
  </KitSection>
</template>
