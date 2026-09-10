<script setup lang="ts">
import { CheckIcon, LockIcon, StarIcon } from '@lucide/vue'
import {
  ToggleGroup,
  ToggleGroupItem,
  ToggleGroupType,
  ToggleSize,
} from '@/components/ui/toggle-group'
import KitSection from '../KitSection.vue'

const Filter = { All: 'all', Missing: 'missing', Done: 'done' } as const
const Status = { Done: 'done', Now: 'now', Blocked: 'blocked' } as const
</script>

<template>
  <KitSection title="Toggle group">
    <ToggleGroup :type="ToggleGroupType.Single" :default-value="Filter.All">
      <ToggleGroupItem :value="Filter.All">Tutti</ToggleGroupItem>
      <ToggleGroupItem :value="Filter.Missing">Manca</ToggleGroupItem>
      <ToggleGroupItem :value="Filter.Done">Fatto</ToggleGroupItem>
    </ToggleGroup>
    <ToggleGroup
      :type="ToggleGroupType.Single"
      :size="ToggleSize.Icon"
      :default-value="Status.Done"
    >
      <ToggleGroupItem :value="Status.Done" aria-label="Fatto">
        <CheckIcon />
      </ToggleGroupItem>
      <ToggleGroupItem :value="Status.Now" aria-label="Sbloccabile ora">
        <StarIcon />
      </ToggleGroupItem>
      <ToggleGroupItem :value="Status.Blocked" aria-label="Bloccato">
        <LockIcon />
      </ToggleGroupItem>
    </ToggleGroup>
  </KitSection>
</template>
