<script setup lang="ts">
import { ArrowDownIcon, ArrowUpIcon, CornerDownLeftIcon } from '@lucide/vue'
import { Kbd, KbdGroup } from '@/components/ui/kbd'
import { KeyName } from '@/lib/constants/keyNames'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Kbd">
    <div
      class="flex flex-wrap items-center gap-3 text-caption text-muted-foreground"
    >
      <KbdGroup>
        <Kbd>{{ KeyName.Ctrl }}</Kbd>
        <Kbd>{{ KeyName.K }}</Kbd>
      </KbdGroup>
      <span class="flex items-center gap-1">
        <Kbd><ArrowUpIcon /></Kbd>
        <Kbd><ArrowDownIcon /></Kbd>
        naviga
      </span>
      <span class="flex items-center gap-1">
        <Kbd><CornerDownLeftIcon /></Kbd>
        apri
      </span>
      <Kbd>{{ KeyName.Esc }}</Kbd>
    </div>
  </KitSection>
</template>
