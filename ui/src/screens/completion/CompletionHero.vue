<script setup lang="ts">
import { Grid2x2Icon } from '@lucide/vue'
import KpiTile from '@/components/kpi/KpiTile.vue'
import { KpiSurface } from '@/components/kpi/kpiSurface'
import { KpiTone } from '@/components/kpi/kpiTone'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { Progress, ProgressTone } from '@/components/ui/progress'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { useMessages } from '@/i18n'
import type { CompletionKpis as Kpis } from '@/lib/completion/completionView'
import ScreenHeader from '@/components/screen/ScreenHeader.vue'
import HeroBand from '@/components/screen/HeroBand.vue'

defineProps<{ kpis: Kpis; widgetUrl: string | null }>()
const { t } = useMessages()
</script>

<template>
  <!-- The band the screen opens on (card #58), the same one a wiki page opens on. -->
  <HeroBand>
    <div class="relative flex flex-col gap-5 @regular/page:flex-row">
      <!-- The game's own completion widget, composed by the icon protocol out of the paper
           and the symbols the columns have earned (`ipc::widget_source`).
           **Drawn only when there is one, and only where there is room for it.** Without
           the game, a 192px frame holding a placeholder is a hole the size of the picture,
           and a screen that looks like it is missing something reads worse than one that
           never promised it. Stacked, it costs about 200px of height on the screen whose
           whole point is the grid underneath — the emblem is a reward, not something anyone
           has to read, so it is the first thing to fold when the room runs out
           (spec 3.13a). The text simply takes the width in both cases. -->
      <div
        v-if="widgetUrl"
        class="hidden size-completion-widget shrink-0 place-items-center @regular/page:grid"
      >
        <PixelSprite :url="widgetUrl" class="size-full" />
      </div>
      <div class="flex min-w-0 flex-1 flex-col gap-4">
        <ScreenHeader :icon="Grid2x2Icon" :title="t('routes.completion')">{{
          t('completion.intro')
        }}</ScreenHeader>
        <!-- The one number the screen was opened to find out. Hard over readable and not
             normal over readable: a cell is done when it is done on hard (B22), which is
             also what the emblem beside it draws. -->
        <Tooltip>
          <TooltipTrigger as-child>
            <div tabindex="0" class="flex max-w-200 flex-col gap-2">
              <div class="flex items-baseline gap-2">
                <span class="text-headline text-foreground tabular-nums">{{
                  kpis.hard
                }}</span>
                <span class="text-kpi text-subtle-foreground tabular-nums"
                  >/ {{ kpis.readable }}</span
                >
                <span class="text-label text-muted-foreground">{{
                  t('completion.headline')
                }}</span>
              </div>
              <Progress
                :model-value="kpis.hard"
                :max="kpis.readable"
                :tone="ProgressTone.Primary"
                :aria-label="t('completion.headline')"
              />
            </div>
          </TooltipTrigger>
          <TooltipContent>{{ t('completion.kpi.hardExplain') }}</TooltipContent>
        </Tooltip>
        <!-- The three that are left, bare: three bordered sheets on a lit band read as
             three windows cut into it (`kpiSurface.ts`). Rows and columns are the same
             reading along the two axes of the grid below. -->
        <div class="flex flex-wrap gap-x-10 gap-y-4">
          <KpiTile
            :value="kpis.normal"
            :denominator="kpis.readable"
            :label="t('completion.kpi.normal')"
            :surface="KpiSurface.Bare"
          >
            <template #explain>{{
              t('completion.kpi.normalExplain')
            }}</template>
          </KpiTile>
          <KpiTile
            :value="kpis.completeCharacters"
            :denominator="kpis.characters"
            :label="t('completion.kpi.complete')"
            :tone="KpiTone.Done"
            :surface="KpiSurface.Bare"
          >
            <template #explain>{{
              t('completion.kpi.completeExplain')
            }}</template>
          </KpiTile>
          <KpiTile
            :value="kpis.completeColumns"
            :denominator="kpis.columns"
            :label="t('completion.kpi.columns')"
            :tone="KpiTone.Done"
            :surface="KpiSurface.Bare"
          >
            <template #explain>{{
              t('completion.kpi.columnsExplain')
            }}</template>
          </KpiTile>
        </div>
      </div>
    </div>
  </HeroBand>
</template>
