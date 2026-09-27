<script setup lang="ts">
import { computed } from 'vue'
import EditionBadge from '@/components/wiki/EditionBadge.vue'
import FactChips from '@/components/wiki/FactChips.vue'
import ProgressBadge from '@/components/wiki/ProgressBadge.vue'
import WikiFigure from '@/components/wiki/WikiFigure.vue'
import { FigureSize } from '@/components/wiki/figureSize'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import type { WikiPageRef } from '@/lib/ipc/types'
import { pageId } from '@/lib/wiki/wikiLabels'
import { useWikiStore } from '@/stores/wiki'

// One card of the grid (card #90, decision 4): the picture as large as the card, the name,
// the id, the edition badge, the kind's own facts as chips (`FactChips` already reads
// `PageFacts` into them, `factChips.ts`) and the save's state. `limit` keeps a page with a
// long tag list from pushing the progress badge out of the card.
const ChipLimit = 4

const props = defineProps<{ page: WikiPageRef }>()
const emit = defineEmits<{ open: [MouseEvent] }>()
const { t } = useMessages()
const wiki = useWikiStore()

const id = computed(() => pageId(props.page.target))
const progress = computed(() => wiki.progressFor(props.page.target))
</script>

<template>
  <Button
    :variant="ButtonVariant.Ghost"
    :size="ButtonSize.Row"
    class="h-full w-full flex-col items-center justify-start gap-2 border-hairline bg-data p-3 text-center hover:bg-row-hover"
    @click="emit('open', $event)"
  >
    <WikiFigure
      :target="page.target"
      :url="page.iconUrl"
      :size="FigureSize.Card"
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
      <EditionBadge :dlc="page.dlc" />
    </div>
    <FactChips :facts="page.facts" :limit="ChipLimit" class="justify-center" />
    <ProgressBadge :progress="progress" />
  </Button>
</template>
