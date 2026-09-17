<script setup lang="ts">
import KpiTile from '@/components/kpi/KpiTile.vue'
import { KpiTone } from '@/components/kpi/kpiTone'
import { useMessages } from '@/i18n'
import type { CompletionKpis as Kpis } from '@/lib/completion/completionView'

defineProps<{ kpis: Kpis }>()
const { t } = useMessages()
</script>

<template>
  <!-- Four numbers of four natures, each with its own denominator and none fused into a
       percentage (DESIGN-BRIEF.md §5.3); the explanations live in the tooltips. -->
  <div class="grid grid-cols-4 gap-2.5">
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
    <KpiTile
      :value="kpis.unknown"
      :denominator="kpis.cells"
      :unit="t('completion.kpi.cells')"
      :label="t('completion.kpi.unknown')"
      :tone="KpiTone.Unknown"
    >
      <template #explain>{{ t('completion.kpi.unknownExplain') }}</template>
    </KpiTile>
  </div>
</template>
