<script setup lang="ts">
import { ArrowDownIcon, ArrowUpIcon, CornerDownLeftIcon } from '@lucide/vue'
import { ref } from 'vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandFooter,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import { Kbd, KbdGroup } from '@/components/ui/kbd'
import { KeyName } from '@/lib/constants/keyNames'
import KitSection from '../KitSection.vue'

const items = ['Brimstone', 'Brimstone Bombs', 'Godhead']
const screens = ['Unlock', 'Completamento']
const paletteOpen = ref(false)
</script>

<template>
  <KitSection title="Command">
    <Command>
      <CommandInput placeholder="Cerca in tutto…" />
      <CommandList>
        <CommandEmpty>Nessun risultato.</CommandEmpty>
        <CommandGroup heading="Oggetti">
          <CommandItem v-for="name in items" :key="name" :value="name">
            {{ name }}
          </CommandItem>
        </CommandGroup>
        <CommandGroup heading="Schermate">
          <CommandItem v-for="name in screens" :key="name" :value="name">
            {{ name }}
          </CommandItem>
        </CommandGroup>
      </CommandList>
      <CommandFooter>
        <span class="flex items-center gap-1">
          <Kbd><ArrowUpIcon /></Kbd><Kbd><ArrowDownIcon /></Kbd> naviga
        </span>
        <span class="flex items-center gap-1">
          <Kbd><CornerDownLeftIcon /></Kbd> apri
        </span>
      </CommandFooter>
    </Command>
    <Button
      :variant="ButtonVariant.Secondary"
      class="w-fit"
      @click="paletteOpen = true"
    >
      <KbdGroup>
        <Kbd>{{ KeyName.Ctrl }}</Kbd>
        <Kbd>{{ KeyName.K }}</Kbd>
      </KbdGroup>
      Palette
    </Button>
    <CommandDialog
      v-model:open="paletteOpen"
      title="Ricerca globale"
      description="Cerca oggetti, schermate e pagine della wiki."
    >
      <CommandInput placeholder="Cerca in tutto…" />
      <CommandList>
        <CommandEmpty>Nessun risultato.</CommandEmpty>
        <CommandGroup heading="Oggetti">
          <CommandItem v-for="name in items" :key="name" :value="name">
            {{ name }}
          </CommandItem>
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  </KitSection>
</template>
