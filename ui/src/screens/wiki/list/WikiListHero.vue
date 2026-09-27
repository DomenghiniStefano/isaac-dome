<script setup lang="ts">
import { computed } from 'vue'
import HeroBand from '@/components/screen/HeroBand.vue'
import { Progress, ProgressTone } from '@/components/ui/progress'
import WikiFigure from '@/components/wiki/WikiFigure.vue'
import { FigureSize } from '@/components/wiki/figureSize'
import { useMessages } from '@/i18n'
import type { Target } from '@/lib/ipc/types'
import { wikiCategoryIcon, wikiCategoryTitle } from '@/router/routeTable'
import type { WikiCategory } from '@/router/routeTable'

// The band a category list opens on (card #90, decision 8b): the same picture the landing
// tile draws for this category, at the largest frame the app has, the category's name, how
// many pages it holds, and — with a save that has anything to say about it — the "N of M" bar
// `categoryProgress` already computes (`lib/wiki/progress.ts`).
//
// `sample` carries no target of its own (`WikiIndex.samples`, `CategorySample`): a curated
// picture is not one particular page's. `representative` stands in for it so `WikiFigure`
// can still pick the right frame and the right fallback icon — any page of this category
// shares both with the curated sample, since a frame follows the *kind*, not the individual
// page, and every page here is one kind by construction (`page.category`).
const props = defineProps<{
  category: WikiCategory
  representative: Target | null
  sampleUrl: string | null
  count: number
  progress: { done: number; total: number } | null
}>()
const { t } = useMessages()

const progressText = computed(() =>
  props.progress
    ? t('wiki.list.progressOf', {
        done: props.progress.done,
        total: props.progress.total,
      })
    : null,
)
</script>

<template>
  <HeroBand class="flex flex-wrap items-center gap-5">
    <WikiFigure
      v-if="representative"
      :target="representative"
      :url="sampleUrl"
      :size="FigureSize.Hero"
    />
    <span
      v-else
      class="relative grid size-figure-hero shrink-0 place-items-center border border-border tile-wash"
    >
      <component
        :is="wikiCategoryIcon[category]"
        class="size-12 text-foreground-soft"
      />
    </span>
    <div class="relative flex min-w-0 flex-1 flex-col gap-2">
      <h1 class="text-title text-foreground">
        {{ t(wikiCategoryTitle[category]) }}
      </h1>
      <span class="text-caption text-subtle-foreground tabular-nums">{{
        t('wiki.pages', { n: count })
      }}</span>
      <div v-if="progress" class="flex max-w-100 flex-col gap-1.5">
        <Progress
          :model-value="progress.done"
          :max="progress.total"
          :tone="ProgressTone.Done"
          :aria-label="progressText ?? undefined"
        />
        <span class="text-caption text-subtle-foreground tabular-nums">{{
          progressText
        }}</span>
      </div>
    </div>
  </HeroBand>
</template>
