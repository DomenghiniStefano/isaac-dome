<script setup lang="ts">
import { useMessages } from '@/i18n'
import InfoboxRow from '../InfoboxRow.vue'
import type { InfoboxOf } from './links'

defineProps<{ infobox: InfoboxOf<'boss'> }>()
const { t } = useMessages()
</script>

<template>
  <dl class="flex flex-col gap-2">
    <InfoboxRow
      :label="t('wiki.infobox.baseHp')"
      :text="infobox.baseHp === null ? null : String(infobox.baseHp)"
    />
    <InfoboxRow
      :label="t('wiki.infobox.environment')"
      :inline="infobox.environment"
    />
    <InfoboxRow :label="t('wiki.infobox.pool')" :inline="infobox.pool" />
  </dl>
</template>
