<script setup lang="ts">
import { computed } from 'vue'
import HeroBand from '@/components/screen/HeroBand.vue'
import KpiTile from '@/components/kpi/KpiTile.vue'
import { KpiSurface } from '@/components/kpi/kpiSurface'
import { KpiTone } from '@/components/kpi/kpiTone'
import { FigureSize } from '@/components/wiki/figureSize'
import WikiFigure from '@/components/wiki/WikiFigure.vue'
import ProfileFact from '@/components/data-state/ProfileFact.vue'
import { TabOrigin } from '@/lib/shell/tabs'
import { tabOriginIcon } from '@/lib/shell/tabOriginIcon'
import { useFormat } from '@/composables/useFormat'
import { useMessages } from '@/i18n'
import type { CategorySample } from '@/lib/ipc/types'
import { mosaicSamples } from '@/lib/wiki/landing'
import type { Progress } from '@/lib/wiki/progress'

// The landing's own opening band (design decision 8b): the mosaic leads because pictures are
// what the owner asked for ("le persone piacciono colori ed immagini"), the title and intro
// stay exactly what they were, and the totals — pages, snapshot, patch — are the same facts
// the provenance card gives in full, condensed to the three worth reading before opening a
// category. The overall bar only draws with a save: `overallProgress` already answers `null`
// for "no save chosen" the same way `categoryProgress` does per category.
const props = defineProps<{
  samples: CategorySample[]
  totalPages: number
  snapshot: string
  patch: string
  overall: Progress | null
}>()

const { t } = useMessages()
const fmt = useFormat()

// Eight fills two rows of four at the grid's own narrow break without crowding the text
// beside it; `mosaicSamples` still trims to whatever `samples` actually holds.
const MosaicMax = 8
const mosaic = computed(() => mosaicSamples(props.samples, MosaicMax))
</script>

<template>
  <HeroBand>
    <div
      class="relative flex flex-col gap-5 @regular/page:flex-row @regular/page:items-center"
    >
      <div
        class="grid grid-cols-4 gap-2 @regular/page:shrink-0 @regular/page:grid-cols-4"
      >
        <WikiFigure
          v-for="sample in mosaic"
          :key="sample.category"
          :target="sample.target"
          :url="sample.iconUrl"
          :category="sample.category"
          :size="FigureSize.Card"
        />
      </div>
      <div class="flex min-w-0 flex-1 flex-col gap-4">
        <div class="flex items-center gap-4">
          <component
            :is="tabOriginIcon[TabOrigin.Wiki]"
            class="size-8 shrink-0 text-foreground-soft"
          />
          <div class="flex min-w-0 flex-col gap-1.5">
            <h1 class="text-title text-foreground">{{ t('routes.wiki') }}</h1>
            <p class="max-w-200 text-body text-subtle-foreground">
              {{ t('wiki.intro') }}
            </p>
          </div>
        </div>
        <div class="flex flex-wrap gap-x-8 gap-y-3">
          <ProfileFact
            :label="t('wiki.landing.totalPages')"
            :value="fmt.count(totalPages)"
          />
          <ProfileFact
            :label="t('wiki.provenance.snapshot')"
            :value="snapshot"
          />
          <ProfileFact :label="t('wiki.provenance.patch')" :value="patch" />
        </div>
        <KpiTile
          v-if="overall"
          class="max-w-70"
          :value="overall.done"
          :denominator="overall.total"
          :label="t('wiki.landing.overallProgress')"
          :tone="KpiTone.Done"
          :surface="KpiSurface.Bare"
        />
      </div>
    </div>
  </HeroBand>
</template>
