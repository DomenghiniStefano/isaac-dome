<script setup lang="ts">
import { computed } from 'vue'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { Card, CardContent } from '@/components/ui/card'
import { i18n, useMessages } from '@/i18n'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { assertNever } from '@/lib/assertNever'
import { columnName } from '@/lib/graph/nodeState'
import type { DrawnView } from '@/lib/ipc/types'
import { formatRelativeDay } from '@/lib/profile/profileView'
import { statusText } from './rollText'

// The one card on screen: what to play tonight, drawn from the deck. Both images degrade to
// the fallback outfit `PixelSprite` already draws for a `null` url — on a machine with no
// game installed, which they always are here, exactly like the completion matrix's own heads
// and symbols.
const props = defineProps<{ drawn: DrawnView }>()
const { t } = useMessages()

interface Sentence {
  key: MessageKey<MessageSchema>
  params: Record<string, unknown>
}

// A mark names its column, a Greedier says so it is one — there is no level number, because
// the second level only exists in the Greed column (`roll::Target`'s own reasoning, carried
// into the view).
const sentence = computed((): Sentence => {
  const { target, character } = props.drawn
  switch (target.kind) {
    case 'mark':
      return {
        key: 'roll.card.mark',
        params: { character, column: columnName[target.column] },
      }
    case 'greedier':
      return { key: 'roll.card.greedier', params: { character } }
    default:
      return assertNever(target)
  }
})

// `drawnUnix` is always set on a real draw, so this never falls back to the empty string in
// practice — the type stays honest about the one case (`unix === null`) that cannot reach it.
const when = computed(
  () =>
    formatRelativeDay(
      props.drawn.drawnUnix,
      new Date(),
      i18n.global.locale.value,
    ) ?? '',
)
</script>

<template>
  <Card>
    <CardContent class="flex flex-wrap items-center gap-4">
      <div class="flex shrink-0 items-end gap-2">
        <PixelSprite
          :url="drawn.headUrl"
          placeholder
          class="size-wiki-figure"
        />
        <PixelSprite :url="drawn.artUrl" placeholder class="size-mark-symbol" />
      </div>
      <div class="flex min-w-0 flex-1 flex-col gap-1">
        <span class="text-heading text-foreground">{{
          t(sentence.key, sentence.params)
        }}</span>
        <span class="text-row text-subtle-foreground">{{
          t(statusText(drawn.status))
        }}</span>
        <span class="text-caption text-subtle-foreground"
          >{{ t('roll.card.drawnFrom', { size: drawn.deckSize }) }} ·
          {{ when }}</span
        >
      </div>
    </CardContent>
  </Card>
</template>
