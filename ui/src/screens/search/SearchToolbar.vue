<script setup lang="ts">
import {
  ToggleGroup,
  ToggleGroupItem,
  ToggleGroupType,
} from '@/components/ui/toggle-group'
import { computed } from 'vue'
import { useMessages } from '@/i18n'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { RowGroup, rowGroupOrder } from '@/lib/search/rows'

const props = defineProps<{
  counts: Record<RowGroup, number>
  picked: RowGroup[]
  shown: number
  total: number
  limited: boolean
}>()
const emit = defineEmits<{ update: [picked: RowGroup[]] }>()
const { t } = useMessages()

const groupLabel: Record<RowGroup, MessageKey<MessageSchema>> = {
  [RowGroup.Screens]: 'search.groups.screens',
  [RowGroup.Wiki]: 'search.groups.wiki',
  [RowGroup.Unlock]: 'search.groups.unlock',
  [RowGroup.Collection]: 'search.groups.collection',
}

// Only the groups this answer actually has: a toggle reading "Schermate 0" offers a filter
// that empties the screen, which is the case `docs/BACKLOG.md` B29 removed from the Collection.
const groups = computed(() =>
  rowGroupOrder.filter((group) => props.counts[group] > 0),
)

const onUpdate = (value: unknown) =>
  emit(
    'update',
    Array.isArray(value)
      ? value.flatMap((v) => rowGroupOrder.filter((g) => g === v))
      : [],
  )
</script>

<template>
  <div class="flex flex-wrap items-center gap-3">
    <!-- Where a result opens, with how many rows lead there. Nothing picked is all of them. -->
    <ToggleGroup
      :type="ToggleGroupType.Multiple"
      :model-value="picked"
      class="w-fit"
      @update:model-value="onUpdate"
    >
      <ToggleGroupItem
        v-for="group in groups"
        :key="group"
        :value="group"
        class="gap-2"
      >
        {{ t(groupLabel[group]) }}
        <span class="tabular-nums">{{ counts[group] }}</span>
      </ToggleGroupItem>
    </ToggleGroup>
    <span class="text-caption text-subtle-foreground">
      {{
        limited
          ? t('search.limit', { shown, total })
          : t('search.shown', { shown })
      }}
    </span>
  </div>
</template>
