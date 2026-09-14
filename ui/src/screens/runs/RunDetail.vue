<script setup lang="ts">
import { computed } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import ItemChips from '@/components/runs/ItemChips.vue'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import type { RunView } from '@/lib/ipc/types'

// The chosen run, opened under the list rather than inside it. A virtualized list measures
// its rows at one height — that is what lets it draw 22 of 641 — so a row that grows would
// have to be measured one by one, which is a different mechanism from the one this repo has.
// The detail is the same click, with room for the items.
const props = defineProps<{ run: RunView }>()
const { t } = useMessages()

const groups = computed(() => [
  { title: 'runs.startingItems' as const, items: props.run.startingItems },
  { title: 'runs.collected' as const, items: props.run.collected },
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
        <ItemChips
          v-if="group.items.length > 0"
          :items="group.items"
          :held="run.heldActive"
        />
        <EmptyValue v-else>{{ t('runs.noItems') }}</EmptyValue>
      </div>
      <div v-if="run.heldActive !== null" class="flex flex-col gap-1">
        <span class="text-label text-subtle-foreground">{{
          t('runs.heldActive')
        }}</span>
        <ItemChips :items="[run.heldActive]" :held="run.heldActive" />
      </div>
    </CardContent>
  </Card>
</template>
