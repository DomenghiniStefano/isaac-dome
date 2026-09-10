<script setup lang="ts">
import { computed, useSlots } from 'vue'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { assertNever } from '@/lib/assertNever'
import { cn } from '@/lib/cn'
import { kpiBar } from './kpiBar'
import { KpiTone } from './kpiTone'

const props = withDefaults(
  defineProps<{
    value: number
    denominator?: number | null
    unit?: string
    label: string
    tone?: KpiTone
  }>(),
  { denominator: null, unit: undefined, tone: KpiTone.Progress },
)
const slots = useSlots()

const bar = computed(() => kpiBar(props.value, props.denominator))
const barWidth = computed(() => ({ '--kpi-bar': `${bar.value ?? 0}%` }))

const barClass = computed(() => {
  switch (props.tone) {
    case KpiTone.Progress:
      return 'bg-primary'
    case KpiTone.Done:
      return 'bg-state-done'
    case KpiTone.Unknown:
      return 'bg-faint-foreground'
    default:
      return assertNever(props.tone)
  }
})
</script>

<template>
  <!-- A number, its unit, the label, a micro bar (Chrome e Stati.dc.html, "KPI"). No prose:
       the explanation lives in the tooltip. -->
  <Tooltip :disabled="!slots.explain">
    <TooltipTrigger as-child>
      <div
        :tabindex="slots.explain ? 0 : undefined"
        class="flex flex-col border border-border bg-sheet px-3.25 pt-2.75 pb-3"
      >
        <div class="flex items-baseline gap-1.25">
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
        <span class="mt-1.75 text-label text-muted-foreground">{{
          label
        }}</span>
        <div
          v-if="bar !== null"
          :style="barWidth"
          class="mt-2 flex h-1 border border-secondary bg-data"
        >
          <div :class="cn('w-(--kpi-bar)', barClass)" />
        </div>
      </div>
    </TooltipTrigger>
    <TooltipContent><slot name="explain" /></TooltipContent>
  </Tooltip>
</template>
