<script setup lang="ts">
import ButtonSection from './sections/ButtonSection.vue'
import BadgeSection from './sections/BadgeSection.vue'
import CardSection from './sections/CardSection.vue'
import AlertSection from './sections/AlertSection.vue'
import EmptySection from './sections/EmptySection.vue'
import FieldSection from './sections/FieldSection.vue'
import CheckboxSection from './sections/CheckboxSection.vue'
import RadioGroupSection from './sections/RadioGroupSection.vue'
import SwitchSection from './sections/SwitchSection.vue'
import ToggleGroupSection from './sections/ToggleGroupSection.vue'
import TabsSection from './sections/TabsSection.vue'
import SelectSection from './sections/SelectSection.vue'
import KbdSection from './sections/KbdSection.vue'
import SeparatorSection from './sections/SeparatorSection.vue'
import SkeletonSection from './sections/SkeletonSection.vue'
import SliderSection from './sections/SliderSection.vue'
import TooltipSection from './sections/TooltipSection.vue'
import PopoverSection from './sections/PopoverSection.vue'
import DropdownMenuSection from './sections/DropdownMenuSection.vue'
import CollapsibleSection from './sections/CollapsibleSection.vue'
import DialogSection from './sections/DialogSection.vue'
import CommandSection from './sections/CommandSection.vue'
import TableSection from './sections/TableSection.vue'
import ProgressSection from './sections/ProgressSection.vue'
import TitleBarSection from './sections/app/TitleBarSection.vue'
import NavBarSection from './sections/app/NavBarSection.vue'
import ProfileBlockSection from './sections/app/ProfileBlockSection.vue'
import SectionSidebarSection from './sections/app/SectionSidebarSection.vue'
import KpiTileSection from './sections/app/KpiTileSection.vue'
import MarkCellSection from './sections/app/MarkCellSection.vue'
import WikiSection from './sections/app/WikiSection.vue'
import SearchRowSection from './sections/app/SearchRowSection.vue'
import DataStateSection from './sections/app/DataStateSection.vue'
import CollapsibleCardSection from './sections/app/CollapsibleCardSection.vue'
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
      <SliderSection />
      <CardSection />
      <AlertSection />
      <EmptySection />
      <FieldSection />
      <CheckboxSection />
      <RadioGroupSection />
      <SwitchSection />
      <ToggleGroupSection />
      <TabsSection />
      <SelectSection />
      <TooltipSection />
      <PopoverSection />
      <DropdownMenuSection />
      <CollapsibleSection />
      <DialogSection />
      <CommandSection />
      <TableSection />
      <ProgressSection />
      <h1 class="col-span-3 mt-6 text-heading text-highlight">
        Componenti app
      </h1>
      <TitleBarSection />
      <NavBarSection />
      <SectionSidebarSection />
      <KpiTileSection />
      <MarkCellSection />
      <DataStateSection />
      <WikiSection />
      <SearchRowSection />
      <CollapsibleCardSection />
      <ProfileBlockSection />
    </main>
  </TooltipProvider>
</template>
