<script setup lang="ts">
import { computed } from 'vue'
import ItemChips from '@/components/runs/ItemChips.vue'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import type { RunFloorView, RunView } from '@/lib/ipc/types'

// The run floor by floor: the floor's own name when the game's files give one, its numbers when
// they do not — never a name borrowed from a neighbour — the rooms it was built with, and what
// was taken there.
const props = defineProps<{ run: RunView }>()
const { t } = useMessages()

const floors = computed(() =>
  props.run.floorDetails.map((floor, at) => ({
    floor,
    items: props.run.collected.filter((p) => p.floor === at).map((p) => p.item),
  })),
)

const floorTitle = (floor: RunFloorView): string =>
  floor.name ??
  t('runs.page.floorNumbers', { stage: floor.stage, type: floor.stageType })
</script>

<template>
  <Card v-if="floors.length > 0">
    <CardHeader>
      <CardTitle>{{ t('runs.page.route') }}</CardTitle>
    </CardHeader>
    <CardContent class="flex flex-col">
      <div
        v-for="({ floor, items }, at) in floors"
        :key="at"
        class="flex flex-col gap-2 border-b border-hairline py-2 last:border-b-0"
      >
        <div class="flex items-baseline gap-3">
          <span class="text-row">{{ floorTitle(floor) }}</span>
          <span
            v-if="floor.rooms !== null"
            class="text-label text-subtle-foreground"
            >{{ t('runs.page.rooms', { count: floor.rooms }) }}</span
          >
        </div>
        <ItemChips :items="items" :held="run.heldActive" />
      </div>
    </CardContent>
  </Card>
</template>
