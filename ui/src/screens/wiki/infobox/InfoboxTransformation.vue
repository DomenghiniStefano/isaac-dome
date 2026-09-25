<script setup lang="ts">
import { useMessages } from '@/i18n'
import { useWikiStore } from '@/stores/wiki'
import InfoboxRow from '../InfoboxRow.vue'
import { refsOf } from '../infoboxRefs'
import type { InfoboxOf } from './links'

defineProps<{ infobox: InfoboxOf<'transformation'> }>()
const wiki = useWikiStore()
const { t } = useMessages()
</script>

<template>
  <dl class="flex flex-col gap-2">
    <!-- `null` says nothing, never "3": the page didn't state a count in a form we can
         read, and defaulting it would be invisible against the pages that do. The row
         goes away with it — the "nessuno" this box draws for an empty value would read
         as "no item is needed", which is a claim about Adult that nobody measured. -->
    <InfoboxRow
      v-if="infobox.requires !== null"
      :label="t('wiki.infobox.requires')"
      :text="String(infobox.requires)"
    />
    <InfoboxRow
      v-if="infobox.contributors.length > 0"
      :label="t('wiki.infobox.contributors')"
      :inline="refsOf(infobox.contributors, wiki.titleOf)"
    />
    <!-- Fourteen of the sixteen pages say nothing here, and "nessuno" would read as a
         claim that the transformation acts on nothing. -->
    <InfoboxRow
      v-if="infobox.target.length > 0"
      :label="t('wiki.infobox.target')"
      :inline="infobox.target"
    />
  </dl>
</template>
