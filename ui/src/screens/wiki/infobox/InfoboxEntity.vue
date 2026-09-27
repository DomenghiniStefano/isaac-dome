<script setup lang="ts">
import { useMessages } from '@/i18n'
import InfoboxRow from '../InfoboxRow.vue'
import type { InfoboxOf } from './links'

defineProps<{ infobox: InfoboxOf<'entity'> }>()
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
    <!-- The only place a variant's own behavior is stated (design decision 2): several
         infoboxes on one page share the page's sections, so a per-variant sentence has
         nowhere else to live but here. -->
    <InfoboxRow
      :label="t('wiki.infobox.behavior')"
      :inline="infobox.behavior"
    />
    <InfoboxRow :label="t('wiki.infobox.pool')" :inline="infobox.pool" />
    <InfoboxRow :label="t('wiki.infobox.replace')" :inline="infobox.replace" />
    <InfoboxRow
      :label="t('wiki.infobox.replaceChance')"
      :inline="infobox.replaceChance"
    />
    <InfoboxRow
      :label="t('wiki.infobox.replaceNotes')"
      :inline="infobox.replaceNotes"
    />
  </dl>
</template>
