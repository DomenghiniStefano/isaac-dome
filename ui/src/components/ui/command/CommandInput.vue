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

const props = defineProps<
  ListboxFilterProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
const forwardedProps = useForwardProps(delegatedProps)

const { filterState } = useCommand()
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
      :class="
        cn(
          'w-full bg-transparent text-body text-foreground outline-none placeholder:text-faint-foreground',
          props.class,
        )
      "
    />
  </div>
</template>
