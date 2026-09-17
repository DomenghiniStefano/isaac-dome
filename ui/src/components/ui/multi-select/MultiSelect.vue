<script setup lang="ts">
import { ChevronDownIcon } from '@lucide/vue'
import { computed } from 'vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Command,
  CommandEmpty,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@/components/ui/popover'
import { useMessages } from '@/i18n'
import type { FacetOption } from '@/lib/facets/facetOptions'
import { SEARCHABLE_FROM, pickedSummary } from './summary'

// Several values of one set, picked at once, each with what picking it would give.
//
// Built on the kit's own `Command`, which is Reka's `ListboxRoot` with the scored search, the
// keyboard wiring and the empty state already on it: a second listbox beside it would be two
// behaviours to keep agreeing forever. `multiple` and the toggling selection are ListboxRoot's
// own, forwarded — the control never computes the next set of values itself.
const props = defineProps<{
  label: string
  options: FacetOption[]
  picked: string[]
}>()
const emit = defineEmits<{ 'update:picked': [values: string[]] }>()
const { t } = useMessages()

// The field appears from the list's own size, never from a flag a screen set: how many pools
// exist is a fact about the user's install, not something a screen may assert for them.
const searchable = computed(() => props.options.length >= SEARCHABLE_FROM)

const summary = computed(() =>
  pickedSummary(
    props.options.filter((option) => option.picked).map((o) => o.label),
  ),
)

// Reka hands back whatever the list holds; the values here are always strings, and one that
// isn't is dropped rather than coerced into a pick nobody made.
const onValue = (value: unknown) =>
  emit(
    'update:picked',
    Array.isArray(value) ? value.filter((v) => typeof v === 'string') : [],
  )
</script>

<template>
  <Popover>
    <PopoverTrigger as-child>
      <Button :variant="ButtonVariant.Outline" class="gap-2">
        <span class="truncate"
          >{{ label
          }}<template v-if="summary"> · {{ summary }}</template></span
        >
        <ChevronDownIcon />
      </Button>
    </PopoverTrigger>
    <PopoverContent class="p-0">
      <Command
        multiple
        :model-value="picked"
        class="border-0"
        @update:model-value="onValue"
      >
        <CommandInput v-if="searchable" :placeholder="t('filters.inMenu')" />
        <CommandEmpty>{{ t('filters.noMatch') }}</CommandEmpty>
        <CommandList>
          <CommandItem
            v-for="option in options"
            :key="option.value"
            :value="option.value"
          >
            <Checkbox :model-value="option.picked" tabindex="-1" />
            <span class="min-w-0 flex-1 truncate">{{ option.label }}</span>
            <span class="text-label text-subtle-foreground tabular-nums">{{
              option.count
            }}</span>
          </CommandItem>
        </CommandList>
      </Command>
    </PopoverContent>
  </Popover>
</template>
