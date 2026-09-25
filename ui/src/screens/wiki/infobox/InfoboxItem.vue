<script setup lang="ts">
import QualityPips from '@/components/data-state/QualityPips.vue'
import { useMessages } from '@/i18n'
import { CollectibleTemplate } from '@/lib/ipc/types'
import InfoboxRow from '../InfoboxRow.vue'
import InfoboxTags from './InfoboxTags.vue'
import type { InfoboxOf } from './links'

defineProps<{ infobox: InfoboxOf<'item'> }>()
const { t } = useMessages()
</script>

<template>
  <!-- Recharge belongs to the activated template alone: on a passive the wiki leaves it empty,
       and an empty row there would read as "it recharges, and nobody wrote how fast". The pools
       are stated on 45 of 720 pages, so their absence is the wiki's silence and not the item's
       — the row goes with it rather than declaring none. -->
  <dl class="flex flex-col gap-2">
    <InfoboxRow :label="t('wiki.infobox.quality')" value-class="min-w-0">
      <QualityPips :quality="infobox.quality" />
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
    <InfoboxRow
      v-if="infobox.pools.length > 0"
      :label="t('wiki.infobox.pools')"
      :inline="infobox.pools"
    />
    <InfoboxTags :tags="infobox.tags" />
  </dl>
</template>
