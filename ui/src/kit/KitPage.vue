<script setup lang="ts">
import ButtonSection from './sections/ButtonSection.vue'
import BadgeSection from './sections/BadgeSection.vue'
import CardSection from './sections/CardSection.vue'
import AlertSection from './sections/AlertSection.vue'
import EmptySection from './sections/EmptySection.vue'
import FieldSection from './sections/FieldSection.vue'
import CheckboxSection from './sections/CheckboxSection.vue'
import SwitchSection from './sections/SwitchSection.vue'
import ToggleGroupSection from './sections/ToggleGroupSection.vue'
import TabsSection from './sections/TabsSection.vue'
import SelectSection from './sections/SelectSection.vue'
import KbdSection from './sections/KbdSection.vue'
import SeparatorSection from './sections/SeparatorSection.vue'
import SkeletonSection from './sections/SkeletonSection.vue'
import TooltipSection from './sections/TooltipSection.vue'
import PopoverSection from './sections/PopoverSection.vue'
import CollapsibleSection from './sections/CollapsibleSection.vue'
import DialogSection from './sections/DialogSection.vue'
import CommandSection from './sections/CommandSection.vue'
import TableSection from './sections/TableSection.vue'
import { TooltipProvider } from '@/components/ui/tooltip'
</script>

<template>
  <TooltipProvider>
    <main
      class="grid min-h-screen grid-cols-3 content-start items-start gap-4 bg-background p-8 text-foreground"
    >
      <ButtonSection />
      <BadgeSection />
      <KbdSection />
      <SeparatorSection />
      <SkeletonSection />
      <CardSection />
      <AlertSection />
      <EmptySection />
      <FieldSection />
      <CheckboxSection />
      <SwitchSection />
      <ToggleGroupSection />
      <TabsSection />
      <SelectSection />
      <TooltipSection />
      <PopoverSection />
      <CollapsibleSection />
      <DialogSection />
      <CommandSection />
      <TableSection />
    </main>
  </TooltipProvider>
</template>
