<script setup lang="ts">
import { computed } from 'vue'
import { useMessages } from '@/i18n'
import InfoboxRow from '../InfoboxRow.vue'
import type { InfoboxOf } from './links'

const props = defineProps<{ infobox: InfoboxOf<'character'> }>()
const { t } = useMessages()

const stats = computed(() => [
  { label: t('wiki.infobox.damage'), value: props.infobox.damage },
  { label: t('wiki.infobox.tears'), value: props.infobox.tears },
  { label: t('wiki.infobox.range'), value: props.infobox.range },
  { label: t('wiki.infobox.speed'), value: props.infobox.speed },
  { label: t('wiki.infobox.luck'), value: props.infobox.luck },
  { label: t('wiki.infobox.shotSpeed'), value: props.infobox.shotSpeed },
])
</script>

<template>
  <dl class="flex flex-col gap-2">
    <InfoboxRow :label="t('wiki.infobox.health')" :inline="infobox.health" />
    <div class="flex flex-wrap gap-4 py-1">
      <span v-for="stat in stats" :key="stat.label" class="flex flex-col gap-1">
        <span class="text-label text-subtle-foreground">{{ stat.label }}</span>
        <span class="text-row text-foreground tabular-nums">{{
          stat.value
        }}</span>
      </span>
    </div>
    <InfoboxRow :label="t('wiki.infobox.pickups')" :inline="infobox.pickups" />
    <InfoboxRow
      :label="t('wiki.infobox.collectibles')"
      :inline="infobox.collectibles"
    />
  </dl>
</template>
