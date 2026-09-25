<script setup lang="ts">
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useGestureModifiers } from '@/composables/useGestureModifiers'
import { useMessages } from '@/i18n'
import type { WhyGroup } from '@/lib/graph/whyMenu'
import { useTabsStore } from '@/stores/tabs'

const props = defineProps<{ groups: WhyGroup[]; label: string }>()
const { t } = useMessages()
const tabs = useTabsStore()

// Reka answers with `select`, which says *that* an entry was chosen and not *how*: reading
// the click instead fires twice, because Reka replays it on the item. So the modifier of the
// gesture comes from the window, exactly as the search palette reads it.
const { ctrl } = useGestureModifiers()
</script>

<template>
  <!-- The badge says the state, its menu says why and where to read about it: one overlay,
       not a tooltip to read and a menu to use (spec 3.5d, Decision 4). A badge with nothing
       to say is not a trigger — it stays the inert badge it is. -->
  <DropdownMenu v-if="props.groups.length > 0">
    <DropdownMenuTrigger as-child>
      <slot />
    </DropdownMenuTrigger>
    <DropdownMenuContent :aria-label="props.label">
      <template v-for="group in props.groups" :key="group.label">
        <DropdownMenuLabel>{{ t(group.label) }}</DropdownMenuLabel>
        <DropdownMenuItem
          v-for="entry in group.entries"
          :key="entry.key"
          :disabled="entry.location === null"
          @select="tabs.go(entry.location, ctrl)"
          >{{ entry.name }}</DropdownMenuItem
        >
      </template>
    </DropdownMenuContent>
  </DropdownMenu>
  <slot v-else />
</template>
