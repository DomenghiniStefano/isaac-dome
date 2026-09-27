<script setup lang="ts">
import { useMessages } from '@/i18n'
import InfoboxRow from '../InfoboxRow.vue'
import InfoboxRefRow from './InfoboxRefRow.vue'
import type { InfoboxOf } from './links'

defineProps<{ infobox: InfoboxOf<'achievement'> }>()
const { t } = useMessages()
</script>

<template>
  <dl class="flex flex-col gap-2">
    <InfoboxRow
      :label="t('wiki.infobox.requirements')"
      :inline="infobox.requirements"
    />
    <InfoboxRow :label="t('wiki.infobox.notes')" :inline="infobox.notes" />
    <InfoboxRefRow
      :label="t('wiki.infobox.unlocks')"
      :targets="infobox.unlocks ? [infobox.unlocks] : []"
    />
  </dl>
</template>
