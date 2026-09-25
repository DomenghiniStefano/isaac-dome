<script setup lang="ts">
import { useMessages } from '@/i18n'
import InfoboxRow from '../InfoboxRow.vue'
import InfoboxTags from './InfoboxTags.vue'
import type { InfoboxOf } from './links'

defineProps<{ infobox: InfoboxOf<'trinket'> }>()
const { t } = useMessages()
</script>

<template>
  <dl class="flex flex-col gap-2">
    <InfoboxRow
      v-if="infobox.pools.length > 0"
      :label="t('wiki.infobox.pools')"
      :inline="infobox.pools"
    />
    <InfoboxTags :tags="infobox.tags" />
  </dl>
</template>
