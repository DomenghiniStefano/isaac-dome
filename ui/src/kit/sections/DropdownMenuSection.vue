<script setup lang="ts">
import { Badge, BadgeVariant } from '@/components/ui/badge'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Dropdown menu">
    <!-- The shape the blocked badge uses: a label per group, an entry per thing in the way,
         and one entry with nowhere to go — which has to stay readable. -->
    <DropdownMenu>
      <DropdownMenuTrigger as-child>
        <Badge :variant="BadgeVariant.Blocked" tabindex="0"
          >bloccato da 3</Badge
        >
      </DropdownMenuTrigger>
      <DropdownMenuContent>
        <DropdownMenuLabel>Personaggi</DropdownMenuLabel>
        <DropdownMenuItem>Tainted Lost</DropdownMenuItem>
        <DropdownMenuLabel>Boss</DropdownMenuLabel>
        <DropdownMenuItem>Delirium</DropdownMenuItem>
        <DropdownMenuLabel>Condizioni</DropdownMenuLabel>
        <DropdownMenuItem disabled>greedmode</DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  </KitSection>
</template>
