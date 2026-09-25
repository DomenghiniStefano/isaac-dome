<script setup lang="ts">
import { computed } from 'vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { useMessages } from '@/i18n'
import { useWikiStore } from '@/stores/wiki'
import InfoboxRow from '../InfoboxRow.vue'
import { refOf } from '../infoboxRefs'
import type { InfoboxOf } from './links'

const props = defineProps<{ infobox: InfoboxOf<'challenge'> }>()
const wiki = useWikiStore()
const { t } = useMessages()

// A challenge's restrictions, as the export lists them: only the ones that apply.
const restrictions = computed((): string[] => [
  ...(props.infobox.blindfolded ? [t('wiki.infobox.blindfolded')] : []),
  ...(props.infobox.hasShops ? [] : [t('wiki.infobox.noShops')]),
  ...(props.infobox.hasTreasureRooms
    ? []
    : [t('wiki.infobox.noTreasureRooms')]),
])
</script>

<template>
  <dl class="flex flex-col gap-2">
    <InfoboxRow :label="t('wiki.infobox.goal')" :inline="infobox.goal" />
    <InfoboxRow :label="t('wiki.infobox.items')" :inline="infobox.items" />
    <InfoboxRow
      :label="t('wiki.infobox.trinkets')"
      :inline="infobox.trinkets"
    />
    <InfoboxRow :label="t('wiki.infobox.pickups')" :inline="infobox.pickups" />
    <InfoboxRow :label="t('wiki.infobox.health')" :inline="infobox.health" />
    <InfoboxRow :label="t('wiki.infobox.curse')" :inline="infobox.curse" />
    <InfoboxRow
      :label="t('wiki.infobox.restrictions')"
      class="gap-1.5"
      value-class="flex min-w-0 flex-wrap gap-1.5"
    >
      <Badge
        v-for="restriction in restrictions"
        :key="restriction"
        :variant="BadgeVariant.Challenge"
        >{{ restriction }}</Badge
      >
      <span
        v-if="restrictions.length === 0"
        class="text-row text-foreground-soft"
        >{{ t('wiki.infobox.noRestrictions') }}</span
      >
    </InfoboxRow>
    <InfoboxRow
      :label="t('wiki.infobox.unlocks')"
      :inline="refOf(infobox.unlocks, wiki.titleOf)"
    />
  </dl>
</template>
