<script setup lang="ts">
import KpiTile from '@/components/kpi/KpiTile.vue'
import { KpiTone } from '@/components/kpi/kpiTone'
import KitSection from '../../KitSection.vue'
</script>

<template>
  <KitSection title="KpiTile">
    <div class="grid grid-cols-2 gap-2.5">
      <KpiTile :value="166" :denominator="368" label="marchi iniziati">
        <template #explain>
          Celle con almeno un segno, fra quelle che il salvataggio lascia
          leggere.
        </template>
      </KpiTile>
      <KpiTile :value="120" unit="celle" label="normale + hard" />
      <KpiTile
        :value="3"
        :denominator="34"
        label="personaggi completi"
        :tone="KpiTone.Done"
      />
      <KpiTile
        :value="40"
        :denominator="408"
        unit="celle"
        label="non leggibili"
        :tone="KpiTone.Unknown"
      />
    </div>
  </KitSection>
</template>
