<script setup lang="ts">
import { computed } from 'vue'
import EditionBadge from '@/components/wiki/EditionBadge.vue'
import FactSheet from '@/components/wiki/FactSheet.vue'
import StateCorner from '@/components/wiki/StateCorner.vue'
import WikiFigure from '@/components/wiki/WikiFigure.vue'
import { FigureSize } from '@/components/wiki/figureSize'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Progress, ProgressSize, ProgressTone } from '@/components/ui/progress'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import { stateBorder } from '@/lib/facets/stateTone'
import type { WikiPageRef } from '@/lib/ipc/types'
import { cardState } from '@/lib/wiki/cardState'
import { pageId } from '@/lib/wiki/wikiLabels'
import { useWikiStore } from '@/stores/wiki'

// One card of the grid, and three kinds of thing told apart by where and how they sit: the
// edition as a tag in the top-left corner, the save's state as a mark in the top-right corner
// and the card's edge in its colour (a bar under the picture when the state is a fraction, a
// character's marks), and the page's own facts as a spec sheet under the name.
const props = defineProps<{ page: WikiPageRef }>()
const emit = defineEmits<{ open: [MouseEvent] }>()
const { t } = useMessages()
const wiki = useWikiStore()

const id = computed(() => pageId(props.page.target))
const state = computed(() => cardState(wiki.progressFor(props.page.target)))
const edge = computed(() =>
  state.value ? stateBorder[state.value.tone] : 'border-hairline',
)
</script>

<template>
  <Button
    :variant="ButtonVariant.Ghost"
    :size="ButtonSize.Row"
    :class="
      cn(
        'relative h-full w-full min-w-0 flex-col items-center justify-start gap-2 overflow-hidden border bg-data p-3 text-center whitespace-normal hover:bg-row-hover',
        edge,
      )
    "
    @click="emit('open', $event)"
  >
    <EditionBadge
      :dlc="page.dlc"
      compact
      class="absolute top-2 left-2 max-w-1/2"
    />
    <StateCorner :state="state" class="absolute top-2 right-2" />
    <WikiFigure
      :target="page.target"
      :url="page.iconUrl"
      :size="FigureSize.Card"
    />
    <Progress
      v-if="state?.fraction"
      :model-value="state.fraction.done"
      :max="state.fraction.total"
      :size="ProgressSize.Micro"
      :tone="ProgressTone.Done"
      class="w-3/4"
    />
    <div class="flex w-full min-w-0 flex-col items-center gap-1">
      <span class="w-full truncate text-body text-foreground">{{
        page.title
      }}</span>
      <span
        v-if="id !== null"
        class="text-micro text-faint-foreground tabular-nums"
        >{{ t('wiki.id', { id }) }}</span
      >
    </div>
    <FactSheet v-if="page.category" :page="page" :category="page.category" />
  </Button>
</template>
