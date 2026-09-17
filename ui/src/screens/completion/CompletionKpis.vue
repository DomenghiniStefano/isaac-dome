<script setup lang="ts">
import KpiTile from '@/components/kpi/KpiTile.vue'
import { KpiTone } from '@/components/kpi/kpiTone'
import { useMessages } from '@/i18n'
import type { CompletionKpis as Kpis } from '@/lib/completion/completionView'

defineProps<{ kpis: Kpis }>()
const { t } = useMessages()
</script>

<template>
  <!-- Three numbers of three natures, each with its own denominator and none fused into a
       percentage (DESIGN-BRIEF.md §5.3); the explanations live in the tooltips. The count of
       unreadable cells is not one of them (B23): it is a gap in our own tables, not a fact
       about the player, and the grid says it where it happens. -->
  <div class="grid grid-cols-3 gap-2.5">
    <KpiTile
      :value="kpis.normal"
      :denominator="kpis.readable"
      :label="t('completion.kpi.normal')"
    >
      <template #explain>{{ t('completion.kpi.normalExplain') }}</template>
    </KpiTile>
    <KpiTile
      :value="kpis.hard"
      :denominator="kpis.readable"
      :label="t('completion.kpi.hard')"
    >
      <template #explain>{{ t('completion.kpi.hardExplain') }}</template>
    </KpiTile>
    <KpiTile
      :value="kpis.completeCharacters"
      :denominator="kpis.characters"
      :label="t('completion.kpi.complete')"
      :tone="KpiTone.Done"
    >
      <template #explain>{{ t('completion.kpi.completeExplain') }}</template>
    </KpiTile>
  </div>
</template>
