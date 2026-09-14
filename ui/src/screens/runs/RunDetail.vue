<script setup lang="ts">
import { computed } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import type { RunItemRef, RunView } from '@/lib/ipc/types'

// The chosen run, opened under the list rather than inside it. A virtualized list measures
// its rows at one height — that is what lets it draw 22 of 641 — so a row that grows would
// have to be measured one by one, which is a different mechanism from the one this repo has.
// The detail is the same click, with room for the items.
const props = defineProps<{ run: RunView }>()
const { t } = useMessages()

// An item with no name is an item the catalog could not name — the game is not installed —
// and it is shown by its number rather than hidden: an id is what we know about it.
const label = (item: RunItemRef): string =>
  item.name ?? t('runs.unnamedItem', { id: item.id })

const groups = computed(() => [
  { title: 'runs.startingItems' as const, items: props.run.startingItems },
  { title: 'runs.collected' as const, items: props.run.collected },
  {
    title: 'runs.heldActive' as const,
    items: props.run.heldActive === null ? [] : [props.run.heldActive],
  },
])
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle>{{ run.seedWords }}</CardTitle>
    </CardHeader>
    <CardContent class="flex flex-col gap-4">
      <div
        v-for="group in groups"
        :key="group.title"
        class="flex flex-col gap-1"
      >
        <span class="text-label text-subtle-foreground">{{
          t(group.title)
        }}</span>
        <div v-if="group.items.length > 0" class="flex flex-wrap gap-2">
          <span
            v-for="item in group.items"
            :key="item.id"
            class="rounded-input border border-hairline px-2 py-0.5 text-label"
            >{{ label(item) }}</span
          >
        </div>
        <EmptyValue v-else>{{ t('runs.noItems') }}</EmptyValue>
      </div>
    </CardContent>
  </Card>
</template>
