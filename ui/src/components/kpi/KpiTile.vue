<script setup lang="ts">
import { useSlots } from 'vue'
import { Progress, ProgressSize, ProgressTone } from '@/components/ui/progress'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { cn } from '@/lib/cn'
import { hasKpiBar } from './kpiBar'
import { KpiSurface } from './kpiSurface'
import { KpiTone } from './kpiTone'

withDefaults(
  defineProps<{
    value: number
    denominator?: number | null
    unit?: string
    label: string
    tone?: KpiTone
    surface?: KpiSurface
  }>(),
  {
    denominator: null,
    unit: undefined,
    tone: KpiTone.Progress,
    surface: KpiSurface.Panel,
  },
)
const slots = useSlots()

// The bar's colour follows what the number is. A record over the whole set: a tone with no
// colour fails to compile.
const barTone: Record<KpiTone, ProgressTone> = {
  [KpiTone.Progress]: ProgressTone.Primary,
  [KpiTone.Done]: ProgressTone.Done,
  [KpiTone.Unknown]: ProgressTone.Muted,
}
</script>

<template>
  <!-- A number, its unit, the label, a micro bar (Chrome e Stati.dc.html, "KPI"). No prose:
       the explanation lives in the tooltip. The bar is the Progress primitive at its micro
       size, so the share is computed and clamped in one place. -->
  <Tooltip :disabled="!slots.explain">
    <TooltipTrigger as-child>
      <div
        :tabindex="slots.explain ? 0 : undefined"
        :class="
          cn(
            'flex flex-col',
            surface === KpiSurface.Panel &&
              'border border-border bg-sheet px-3 pt-3 pb-3',
          )
        "
      >
        <div class="flex items-baseline gap-1">
          <span
            :class="
              cn(
                'text-kpi',
                tone === KpiTone.Unknown
                  ? 'text-muted-foreground'
                  : 'text-foreground',
              )
            "
            >{{ value }}</span
          >
          <span
            v-if="unit !== undefined"
            class="text-caption text-subtle-foreground"
            >{{ unit }}</span
          >
          <span
            v-else-if="denominator !== null"
            class="text-caption text-subtle-foreground"
            >/ {{ denominator }}</span
          >
        </div>
        <span class="mt-2 text-label text-muted-foreground">{{ label }}</span>
        <Progress
          v-if="hasKpiBar(denominator)"
          :model-value="value"
          :max="denominator"
          :size="ProgressSize.Micro"
          :tone="barTone[tone]"
          :aria-label="label"
          class="mt-2"
        />
      </div>
    </TooltipTrigger>
    <TooltipContent><slot name="explain" /></TooltipContent>
  </Tooltip>
</template>
