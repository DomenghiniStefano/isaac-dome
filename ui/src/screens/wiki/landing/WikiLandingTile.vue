<script setup lang="ts">
import { computed } from 'vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Progress, ProgressSize, ProgressTone } from '@/components/ui/progress'
import { toneClass } from '@/components/ui/chip'
import { FigureSize } from '@/components/wiki/figureSize'
import WikiFigure from '@/components/wiki/WikiFigure.vue'
import { useFormat } from '@/composables/useFormat'
import { useMessages } from '@/i18n'
import type { CategorySample } from '@/lib/ipc/types'
import type { Progress as ProgressValue } from '@/lib/wiki/progress'
import { toneOfCategory } from '@/lib/wiki/tone'
import { type WikiCategory, wikiCategoryTitle } from '@/router/routeTable'

// One category, on its own accent (design decision 8): the picture leads, large and centred,
// the name and page count under it, and — only with a save, `progress` is `null` for both "no
// save" and "the save has nothing to say about this category" (`categoryProgress`'s own
// doc) — the "N of M" bar in the state's done colour. `sample` is `undefined` only before the
// index has loaded (the skeleton draws instead, in `WikiLanding.vue`).
const props = defineProps<{
  category: WikiCategory
  sample: CategorySample | undefined
  pageCount: number
  progress: ProgressValue | null
}>()
defineEmits<{ open: [MouseEvent] }>()

const { t } = useMessages()
const fmt = useFormat()

const title = computed(() => t(wikiCategoryTitle[props.category]))
const tone = computed(() => toneClass[toneOfCategory(props.category)])
</script>

<template>
  <Button
    :variant="ButtonVariant.Outline"
    :class="[
      'h-auto flex-col items-center gap-2 border p-4 text-center whitespace-normal',
      tone,
    ]"
    @click="$emit('open', $event)"
  >
    <WikiFigure
      :target="sample?.target ?? null"
      :url="sample?.iconUrl ?? null"
      :category="category"
      :size="FigureSize.Tile"
    />
    <span class="text-heading">{{ title }}</span>
    <span class="text-caption tabular-nums opacity-muted">{{
      t('wiki.pages', { n: fmt.count(pageCount) })
    }}</span>
    <div v-if="progress" class="flex w-full flex-col gap-1">
      <Progress
        :model-value="progress.done"
        :max="progress.total"
        :size="ProgressSize.Micro"
        :tone="ProgressTone.Done"
        :aria-label="t('wiki.landing.categoryProgress', { category: title })"
      />
      <span class="text-micro tabular-nums opacity-muted">{{
        t('wiki.landing.progressOf', {
          done: fmt.count(progress.done),
          total: fmt.count(progress.total),
        })
      }}</span>
    </div>
  </Button>
</template>
