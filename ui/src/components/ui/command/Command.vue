<script setup lang="ts">
import type { ListboxRootEmits, ListboxRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { ListboxRoot, useFilter, useForwardPropsEmits } from 'reka-ui'
import { reactive, ref, useTemplateRef, watch } from 'vue'
import { cn } from '@/lib/cn'
import type { CommandFilterState } from './context'
import { provideCommandContext } from './context'
import { scoreItems } from './filter'

// `filter: false` leaves the list as it is: the caller has already filtered it, and the
// palette's rows must not be scored a second time. `search` is the typed text, exposed so a
// caller can debounce it and send it somewhere — the palette asks the backend with it.
const props = withDefaults(
  defineProps<
    ListboxRootProps & {
      class?: HTMLAttributes['class']
      filter?: boolean
      search?: string
    }
  >(),
  {
    modelValue: '',
    highlightOnHover: true,
    filter: true,
  },
)
const emits = defineEmits<
  ListboxRootEmits & { 'update:search': [value: string] }
>()

const delegatedProps = reactiveOmit(props, 'class', 'filter', 'search')
const forwarded = useForwardPropsEmits(delegatedProps, emits)

const allItems = ref<Map<string, string>>(new Map())
const allGroups = ref<Map<string, Set<string>>>(new Map())

const { contains } = useFilter({ sensitivity: 'base' })
const filterState = reactive<CommandFilterState>({
  search: props.search ?? '',
  filtered: { count: 0, items: new Map(), groups: new Set() },
})

const filterItems = () => {
  Object.assign(
    filterState.filtered,
    scoreItems(
      allItems.value,
      allGroups.value,
      filterState.search,
      props.filter,
      contains,
    ),
  )
}

// Items mount and unmount as the answer changes, so their count is part of what to score.
watch(
  [() => filterState.search, () => props.filter, () => allItems.value.size],
  filterItems,
)

watch(
  () => props.search,
  (value) => {
    if (typeof value === 'string' && value !== filterState.search)
      filterState.search = value
  },
)
watch(
  () => filterState.search,
  (value) => emits('update:search', value),
)

provideCommandContext({ allItems, allGroups, filterState })

// The listbox owns the highlight — it draws it and the arrows move it — but it only ever
// places it by itself on the first item, and a caller whose rows arrive after the keystroke
// needs to put it back. One function out, nothing else: a caller cannot reach the collection,
// the DOM element, or anything it could hold a stale reference to.
const listbox = useTemplateRef('listbox')
defineExpose({
  highlightItem: (value: string) => listbox.value?.highlightItem(value),
})
</script>

<template>
  <ListboxRoot
    ref="listbox"
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
