<script setup lang="ts">
import { useMessages } from '@/i18n'
import { useWikiStore } from '@/stores/wiki'
import InfoboxRow from '../InfoboxRow.vue'
import { refOf } from '../infoboxRefs'
import type { InfoboxOf } from './links'

defineProps<{ infobox: InfoboxOf<'achievement'> }>()
const wiki = useWikiStore()
const { t } = useMessages()
</script>

<template>
  <dl class="flex flex-col gap-2">
    <InfoboxRow
      :label="t('wiki.infobox.requirements')"
      :inline="infobox.requirements"
    />
    <InfoboxRow :label="t('wiki.infobox.notes')" :inline="infobox.notes" />
    <InfoboxRow
      :label="t('wiki.infobox.unlocks')"
      :inline="refOf(infobox.unlocks, wiki.titleOf)"
    />
  </dl>
</template>
