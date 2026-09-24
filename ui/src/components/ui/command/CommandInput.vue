<script setup lang="ts">
import type { ListboxFilterProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { SearchIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import { ListboxFilter, useForwardProps } from 'reka-ui'
import { cn } from '@/lib/cn'
import { useCommand } from './context'

defineOptions({
  inheritAttrs: false,
})

// `selectOnFocus` selects what is already written when the input takes the focus, so the first
// key typed replaces it instead of being appended — for a caller that opens on a remembered
// query. The listbox's auto-focus is a focus like any other, so opening counts.
const props = defineProps<
  ListboxFilterProps & {
    class?: HTMLAttributes['class']
    selectOnFocus?: boolean
  }
>()

const delegatedProps = reactiveOmit(props, 'class', 'selectOnFocus')
const forwardedProps = useForwardProps(delegatedProps)

const { filterState } = useCommand()

const onFocus = (event: FocusEvent) => {
  if (!props.selectOnFocus) return
  if (event.target instanceof HTMLInputElement) event.target.select()
}
</script>

<template>
  <!-- A plain row, not the registry's InputGroup: the caret is the focus here, so the
       input itself carries no ring. -->
  <div
    data-slot="command-input-wrapper"
    class="flex items-center gap-2 border-b border-hairline px-3 py-2"
  >
    <SearchIcon class="size-3.5 shrink-0 text-muted-foreground" />
    <ListboxFilter
      v-bind="{ ...forwardedProps, ...$attrs }"
      v-model="filterState.search"
      data-slot="command-input"
      auto-focus
      @focus="onFocus"
      :class="
        cn(
          'w-full bg-transparent text-body text-foreground outline-none placeholder:text-faint-foreground',
          props.class,
        )
      "
    />
  </div>
</template>
