<script setup lang="ts">
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import KitSection from '../KitSection.vue'

const SortOrder = {
  MissingFirst: 'missingFirst',
  DoneFirst: 'doneFirst',
  Alphabetical: 'alphabetical',
} as const
</script>

<template>
  <KitSection title="Select">
    <Select :default-value="SortOrder.MissingFirst">
      <SelectTrigger>
        <SelectValue placeholder="Ordina per…" />
      </SelectTrigger>
      <SelectContent>
        <SelectGroup>
          <SelectLabel>Ordina per</SelectLabel>
          <SelectItem :value="SortOrder.MissingFirst"
            >Mancanti prima</SelectItem
          >
          <SelectItem :value="SortOrder.DoneFirst">Fatti prima</SelectItem>
          <SelectItem :value="SortOrder.Alphabetical">Alfabetico</SelectItem>
        </SelectGroup>
      </SelectContent>
    </Select>
    <Select>
      <SelectTrigger>
        <SelectValue placeholder="Nessuna scelta" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem :value="SortOrder.Alphabetical">Alfabetico</SelectItem>
      </SelectContent>
    </Select>
  </KitSection>
</template>
