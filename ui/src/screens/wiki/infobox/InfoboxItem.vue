<script setup lang="ts">
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import QualityChip from '@/components/wiki/QualityChip.vue'
import { useMessages } from '@/i18n'
import { CollectibleTemplate } from '@/lib/ipc/types'
import type { PoolMembershipView } from '@/lib/ipc/types'
import InfoboxRow from '../InfoboxRow.vue'
import InfoboxPools from './InfoboxPools.vue'
import InfoboxTags from './InfoboxTags.vue'
import type { InfoboxOf } from './links'

defineProps<{
  infobox: InfoboxOf<'item'>
  // The game's own pools (design decision 4): the two rows below answer two different
  // questions — this one the game's, `undefined` while `WikiInfobox` is still asking, `null`
  // without the game; `infobox.obtainedFrom` is the wiki's own text and needs neither.
  pools?: PoolMembershipView[] | null
}>()
const { t } = useMessages()
</script>

<template>
  <!-- Recharge belongs to the activated template alone: on a passive the wiki leaves it empty,
       and an empty row there would read as "it recharges, and nobody wrote how fast". -->
  <dl class="flex flex-col gap-2">
    <InfoboxRow :label="t('wiki.infobox.quality')" value-class="min-w-0">
      <QualityChip v-if="infobox.quality !== null" :quality="infobox.quality" />
      <EmptyValue v-else>{{ t('wiki.infobox.none') }}</EmptyValue>
    </InfoboxRow>
    <InfoboxRow
      v-if="infobox.template === CollectibleTemplate.Activated"
      :label="t('wiki.infobox.recharge')"
      :inline="infobox.recharge"
    />
    <InfoboxRow
      :label="t('wiki.infobox.devilPrice')"
      :inline="infobox.devilPrice"
    />
    <InfoboxRow
      :label="t('wiki.infobox.shopPrice')"
      :inline="infobox.shopPrice"
    />
    <InfoboxPools :pools="pools" />
    <!-- A guaranteed source the wiki names in prose (a boss, a machine, another item), never
         a weighted pool — see `wiki::Infobox`'s own doc comment. Absent, like the pools row
         above, when neither the wiki nor the game says anything. -->
    <InfoboxRow
      v-if="infobox.obtainedFrom.length > 0"
      :label="t('wiki.infobox.obtainedFrom')"
      :inline="infobox.obtainedFrom"
    />
    <InfoboxTags :tags="infobox.tags" />
  </dl>
</template>
