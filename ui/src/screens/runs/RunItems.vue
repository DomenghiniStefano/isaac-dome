<script setup lang="ts">
import { computed } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import ItemChips from '@/components/runs/ItemChips.vue'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  ToggleGroup,
  ToggleGroupItem,
  ToggleGroupType,
} from '@/components/ui/toggle-group'
import { useMessages } from '@/i18n'
import type { Message } from '@/i18n/message'
import { assertNever } from '@/lib/assertNever'
import type { RunView } from '@/lib/ipc/types'
import {
  GroupTitleKind,
  RunItemView,
  groupItems,
  runItemViews,
} from '@/lib/runs/itemViews'
import type { GroupTitle } from '@/lib/runs/itemViews'
import { poolLabel } from '@/lib/runs/pools'

// A run's items, as logged by default and three more ways: by what they are, by where they came
// from, by the floor they were taken on. Which way is the tab's, so it survives a tab switch.
const props = defineProps<{ run: RunView; view: RunItemView }>()
const emit = defineEmits<{ 'update:view': [view: RunItemView] }>()
const { t } = useMessages()

const viewLabel: Record<RunItemView, Message> = {
  [RunItemView.AsLogged]: 'runs.itemView.asLogged',
  [RunItemView.ByType]: 'runs.itemView.byType',
  [RunItemView.ByOrigin]: 'runs.itemView.byOrigin',
  [RunItemView.ByFloor]: 'runs.itemView.byFloor',
}

const groups = computed(() => groupItems(props.run, props.view))

// A pool the page does not word is shown as the log wrote it; a floor with no name, by its
// numbers.
const titleText = (title: GroupTitle): string => {
  switch (title.kind) {
    case GroupTitleKind.Message:
      return t(title.message)
    case GroupTitleKind.Pool: {
      const label = poolLabel(title.pool)
      return label === null ? title.pool : t(label)
    }
    case GroupTitleKind.Floor:
      if (title.floor === null) return t('runs.page.beforeFirstFloor')
      return (
        title.floor.name ??
        t('runs.page.floorNumbers', {
          stage: title.floor.stage,
          type: title.floor.stageType,
        })
      )
    default:
      return assertNever(title)
  }
}

const pick = (value: unknown) => {
  if (typeof value !== 'string') return
  const view = runItemViews.find((v) => v === value)
  if (view !== undefined) emit('update:view', view)
}
</script>

<template>
  <Card>
    <CardHeader class="flex flex-wrap items-center justify-between gap-2">
      <CardTitle>{{ t('runs.page.items') }}</CardTitle>
      <ToggleGroup
        :type="ToggleGroupType.Single"
        :model-value="view"
        @update:model-value="pick"
      >
        <ToggleGroupItem v-for="v in runItemViews" :key="v" :value="v">{{
          t(viewLabel[v])
        }}</ToggleGroupItem>
      </ToggleGroup>
    </CardHeader>
    <CardContent class="flex flex-col gap-4">
      <div v-for="(group, at) in groups" :key="at" class="flex flex-col gap-1">
        <span class="text-label text-subtle-foreground">{{
          titleText(group.title)
        }}</span>
        <ItemChips
          v-if="group.items.length > 0"
          :items="group.items"
          :held="run.heldActive"
        />
        <EmptyValue v-else>{{ t('runs.noItems') }}</EmptyValue>
      </div>
      <EmptyValue v-if="groups.length === 0">{{
        t('runs.noItems')
      }}</EmptyValue>
    </CardContent>
  </Card>
</template>
