<script setup lang="ts">
import type { ListboxRootEmits, ListboxRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { ListboxRoot, useFilter, useForwardPropsEmits } from 'reka-ui'
import { reactive, ref, watch } from 'vue'
import { cn } from '@/lib/cn'
import type { CommandFilterState } from './context'
import { provideCommandContext } from './context'

const props = withDefaults(
  defineProps<ListboxRootProps & { class?: HTMLAttributes['class'] }>(),
  {
    modelValue: '',
    highlightOnHover: true,
  },
)
const emits = defineEmits<ListboxRootEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)

const allItems = ref<Map<string, string>>(new Map())
const allGroups = ref<Map<string, Set<string>>>(new Map())

const { contains } = useFilter({ sensitivity: 'base' })
const filterState = reactive<CommandFilterState>({
  search: '',
  filtered: { count: 0, items: new Map(), groups: new Set() },
})

// With an empty search every item shows itself; otherwise score each item once, then
// derive the visible groups and the count from the scores.
const filterItems = () => {
  if (!filterState.search) {
    filterState.filtered.count = allItems.value.size
    return
  }
  const scores = new Map(
    [...allItems.value].map(([id, text]): [string, number] => [
      id,
      contains(text, filterState.search) ? 1 : 0,
    ]),
  )
  filterState.filtered.items = scores
  filterState.filtered.groups = new Set(
    [...allGroups.value]
      .filter(([, itemIds]) =>
        [...itemIds].some((itemId) => (scores.get(itemId) ?? 0) > 0),
      )
      .map(([groupId]) => groupId),
  )
  filterState.filtered.count = [...scores.values()].filter(
    (score) => score > 0,
  ).length
}

watch(() => filterState.search, filterItems)

provideCommandContext({ allItems, allGroups, filterState })
</script>

<template>
  <ListboxRoot
    data-slot="command"
    v-bind="forwarded"
    :class="
      cn(
        'flex size-full flex-col overflow-hidden border border-input bg-popover text-popover-foreground',
        props.class,
      )
    "
  >
    <slot />
  </ListboxRoot>
</template>
