<script setup lang="ts">
import type { ListboxItemEmits, ListboxItemProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit, useCurrentElement } from '@vueuse/core'
import { ListboxItem, useForwardPropsEmits, useId } from 'reka-ui'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { cn } from '@/lib/cn'
import { useCommand, useCommandGroup } from './context'

const props = defineProps<
  ListboxItemProps & { class?: HTMLAttributes['class'] }
>()
const emits = defineEmits<ListboxItemEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)

const id = useId()
const { filterState, allItems, allGroups } = useCommand()
// null: an item outside any group is allowed.
const groupContext = useCommandGroup(null)

// Before its first score an item isn't in the map yet: it renders once to register.
const isRender = computed(
  () => !filterState.search || (filterState.filtered.items.get(id) ?? 1) > 0,
)

const itemRef = ref()
const currentElement = useCurrentElement(itemRef)

const registerInGroup = (groupId: string) => {
  const group = allGroups.value.get(groupId)
  if (group) {
    group.add(id)
    return
  }
  allGroups.value.set(groupId, new Set([id]))
}

onMounted(() => {
  if (!(currentElement.value instanceof HTMLElement)) return
  allItems.value.set(
    id,
    currentElement.value.textContent ?? props.value?.toString() ?? '',
  )
  if (groupContext?.id) registerInGroup(groupContext.id)
})
onUnmounted(() => {
  allItems.value.delete(id)
})
</script>

<template>
  <!-- Reka moves a highlight through the list rather than the focus ring. -->
  <ListboxItem
    v-if="isRender"
    v-bind="forwarded"
    :id="id"
    ref="itemRef"
    data-slot="command-item"
    :class="
      cn(
        'relative flex cursor-default items-center gap-2 px-3 py-2 text-row outline-none select-none data-[disabled]:pointer-events-none data-[disabled]:text-faint-foreground data-[highlighted]:bg-secondary [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*=size-])]:size-3.5',
        props.class,
      )
    "
    @select="() => (filterState.search = '')"
  >
    <slot />
  </ListboxItem>
</template>
