<script setup lang="ts">
import { computed } from 'vue'
import HeroBand from '@/components/screen/HeroBand.vue'
import { Progress, ProgressTone } from '@/components/ui/progress'
import { toneClass } from '@/components/ui/chip'
import WikiFigure from '@/components/wiki/WikiFigure.vue'
import { FigureSize } from '@/components/wiki/figureSize'
import { useMessages } from '@/i18n'
import type { Target } from '@/lib/ipc/types'
import { wikiCategoryTitle } from '@/router/routeTable'
import type { WikiCategory } from '@/router/routeTable'
import { toneOfCategory } from '@/lib/wiki/tone'

// The band a category list opens on: the same picture the landing tile draws for this
// category, at the largest frame the app has, on the category's colour, its name, how many
// pages it holds, and — with a save that has anything to say about it — the "N of M" bar
// `categoryProgress` computes (`lib/wiki/progress.ts`). `representative` is the page the
// picture was drawn for; `category` decides the fallback icon when there is no picture.
const props = defineProps<{
  category: WikiCategory
  representative: Target | null
  sampleUrl: string | null
  count: number
  progress: { done: number; total: number } | null
}>()
const { t } = useMessages()

const tone = computed(() => toneClass[toneOfCategory(props.category)])

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
  <HeroBand :class="tone">
    <div class="flex flex-wrap items-center gap-5">
      <WikiFigure
        :target="representative"
        :url="sampleUrl"
        :size="FigureSize.Hero"
        :category="category"
      />
      <div class="relative flex min-w-0 flex-1 flex-col gap-2">
        <h1 class="text-title">
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
    </div>
  </HeroBand>
</template>
