<script setup lang="ts">
import { computed, ref } from 'vue'
import { MultiSelect } from '@/components/ui/multi-select'
import type { FacetOption } from '@/lib/facets/facetOptions'
import KitSection from '../KitSection.vue'

// Both shapes the control has, because the whole point of it is that they differ: under the
// threshold it is a plain list, over it the search field appears on its own.
const quality = ['0', '1', '2', '3', '4']
const pools = [
  'angel',
  'boss',
  'curse',
  'devil',
  'golden chest',
  'library',
  'planetarium',
  'red chest',
  'secret',
  'shop',
  'treasure',
  'ultra secret',
]

const few = ref<string[]>(['3'])
const many = ref<string[]>([])

const options = (values: string[], picked: string[]): FacetOption[] =>
  values.map((value, i) => ({
    value,
    label: value,
    count: 120 - i * 9,
    picked: picked.includes(value),
  }))

const qualityOptions = computed(() => options(quality, few.value))
const poolOptions = computed(() => options(pools, many.value))
</script>

<template>
  <KitSection title="MultiSelect">
    <div class="flex flex-wrap items-start gap-3">
      <MultiSelect
        label="Qualità"
        :options="qualityOptions"
        :picked="few"
        @update:picked="few = $event"
      />
      <MultiSelect
        label="Pool"
        :options="poolOptions"
        :picked="many"
        @update:picked="many = $event"
      />
    </div>
  </KitSection>
</template>
