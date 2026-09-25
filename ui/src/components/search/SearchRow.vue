<script setup lang="ts">
import { computed } from 'vue'
import AchievementArt from '@/components/graph/AchievementArt.vue'
import { ArtSize } from '@/components/graph/artSize'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import { ProgressMark } from '@/lib/ipc/types'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { rowGroupLabel } from '@/lib/search/rows'
import type { SearchRow } from '@/lib/search/rows'
import { sectionText } from '@/lib/wiki/wikiLabels'

const props = defineProps<{ row: SearchRow }>()
const { t } = useMessages()

// What the row is called: a screen's name is a message, everything else is data in English.
const title = computed(() =>
  props.row.kind === 'screen' ? t(props.row.entry.label) : props.row.hit.title,
)

const iconUrl = computed(() =>
  props.row.kind === 'hit' ? props.row.hit.iconUrl : null,
)

// An achievement's or a challenge's picture is a drawing, not a sprite: dark strokes on
// transparency at its own ratio, which on the dark theme needs the mark paper under it — the
// way every table and the wiki draw it (`AchievementArt`, `WikiFigure`). Drawn as a sprite it
// was squeezed into a square and lost against the background.
const painting = computed(
  () =>
    props.row.kind === 'hit' &&
    (props.row.hit.target.kind === 'achievement' ||
      props.row.hit.target.kind === 'challenge'),
)

// The mark is drawn only when it says something about *this* row. `none` is a boss or a
// character, which has no slot anywhere; `unknown` reads the same on every row of the answer —
// with no profile that is all of them — and what it would say is already said once, by the
// diagnostic (`docs/BACKLOG.md` B29: a value with nothing behind it isn't listed).
type Told = typeof ProgressMark.Done | typeof ProgressMark.Pending

const isTold = (p: ProgressMark): p is Told =>
  p === ProgressMark.Done || p === ProgressMark.Pending

const mark = computed((): Told | null =>
  props.row.kind === 'hit' && isTold(props.row.hit.progress)
    ? props.row.hit.progress
    : null,
)

const markVariant: Record<Told, BadgeVariant> = {
  [ProgressMark.Done]: BadgeVariant.Done,
  [ProgressMark.Pending]: BadgeVariant.Now,
}

const markText: Record<Told, MessageKey<MessageSchema>> = {
  [ProgressMark.Done]: 'search.progress.done',
  [ProgressMark.Pending]: 'search.progress.pending',
}

interface Detail {
  before: string
  matched: string
  after: string
}

// The second line says **why** this matched: the page's own words around the match, the
// achievement's condition, or — for a title match — nothing to add.
const detail = computed((): Detail | null => {
  if (props.row.kind === 'screen') return null
  const match = props.row.hit.match
  switch (match.kind) {
    case 'title':
      return null
    case 'condition':
      return { before: match.text, matched: '', after: '' }
    case 'section':
      return {
        before: `${t(sectionText[match.section])} · ${match.before}`,
        matched: match.matched,
        after: match.after,
      }
    default:
      return assertNever(match)
  }
})
</script>

<template>
  <div class="flex min-w-0 flex-1 items-center gap-3">
    <!-- One slot as wide as a drawing for every row, so the titles stay in one column
         whether the row leads with a drawing or a sprite. -->
    <span class="flex w-achievement-thumb shrink-0 justify-center">
      <AchievementArt v-if="painting" :url="iconUrl" :size="ArtSize.Thumb" />
      <PixelSprite
        v-else
        :url="iconUrl"
        placeholder
        class="size-icon-compact"
      />
    </span>
    <div class="flex min-w-0 flex-1 flex-col">
      <span class="truncate text-row text-foreground">{{ title }}</span>
      <span v-if="detail" class="truncate text-caption text-subtle-foreground">
        {{ detail.before
        }}<span class="text-highlight">{{ detail.matched }}</span
        >{{ detail.after }}
      </span>
    </div>
    <span class="shrink-0 text-caption text-faint-foreground">{{
      t(rowGroupLabel[row.group])
    }}</span>
    <Badge v-if="mark" :variant="markVariant[mark]">{{
      t(markText[mark])
    }}</Badge>
  </div>
</template>
