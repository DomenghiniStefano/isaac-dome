<script setup lang="ts">
import { vScrollMemory } from '@/directives/scrollMemory'
import { DicesIcon } from '@lucide/vue'
import { computed, onMounted, onUnmounted } from 'vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import { Button } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useMessages } from '@/i18n'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { rollEntries } from '@/lib/diagnostics/roll'
import type { DrawnView } from '@/lib/ipc/types'
import { AppEvent, watchAppEvent } from '@/lib/window/appEvents'
import { LoadStatus } from '@/stores/loadStatus'
import { useRollStore } from '@/stores/roll'
import ProfileError from './profile/ProfileError.vue'
import RollCard from './roll/RollCard.vue'
import RollPanel from './roll/RollPanel.vue'
import { EmptyDeckReason, emptyDeckReason } from './roll/rollText'
import ScreenHeader from './ScreenHeader.vue'

const store = useRollStore()
const { t } = useMessages()

useOnActiveProfile(() => store.load())

// A draw made in another window is a read in this one: the document is one row, and the two
// windows must never show two different cards for it.
let stopRollEvent: (() => void) | undefined
onMounted(async () => {
  stopRollEvent = await watchAppEvent(AppEvent.RollChanged, () => {
    void store.load()
  })
})
onUnmounted(() => stopRollEvent?.())

// The key for each of the four exclusions `emptyDeckReason` can name. A presentational lookup,
// not a judgment: which reason it is comes from the tested pure function, this only routes it
// to a sentence.
const emptyDeckKey: Record<EmptyDeckReason, MessageKey<MessageSchema>> = {
  [EmptyDeckReason.Taken]: 'roll.emptyDeck.taken',
  [EmptyDeckReason.Unreadable]: 'roll.emptyDeck.unreadable',
  [EmptyDeckReason.Locked]: 'roll.emptyDeck.locked',
  [EmptyDeckReason.Filtered]: 'roll.emptyDeck.filtered',
}

// What the card slot shows when there is no drawn target to draw a `RollCard` from: an empty
// deck is a first-class state, not a disabled button, so it says which exclusion emptied it
// and how many — or, failing that, that there was never anything to draw in the first place.
type CardState =
  | { kind: 'drawn'; drawn: DrawnView }
  | { kind: 'emptyDeck'; key: MessageKey<MessageSchema>; count: number }
  | { kind: 'nothingToDeck' }
  | { kind: 'notDrawnYet' }

const cardState = computed((): CardState | null => {
  const view = store.view
  if (!view) return null
  if (view.drawn) return { kind: 'drawn', drawn: view.drawn }
  const reason = emptyDeckReason(view.deck)
  if (reason)
    return {
      kind: 'emptyDeck',
      key: emptyDeckKey[reason],
      count: view.deck[reason],
    }
  return view.deck.size === 0
    ? { kind: 'nothingToDeck' }
    : { kind: 'notDrawnYet' }
})
</script>

<template>
  <div
    v-scroll-memory="'page'"
    class="flex h-full flex-col gap-4 overflow-y-auto px-5.5 pt-5 pb-15"
  >
    <ScreenHeader :icon="DicesIcon" :title="t('routes.roll')">{{
      t('roll.intro')
    }}</ScreenHeader>
    <ProfileError
      v-if="store.status === LoadStatus.Failed"
      :error="store.error"
      @retry="store.load()"
    />
    <template v-else-if="store.view && cardState">
      <RollCard v-if="cardState.kind === 'drawn'" :drawn="cardState.drawn" />
      <EmptyCategory v-else-if="cardState.kind === 'emptyDeck'">{{
        t(cardState.key, { count: cardState.count })
      }}</EmptyCategory>
      <EmptyCategory v-else-if="cardState.kind === 'nothingToDeck'">{{
        t('roll.emptyDeck.nothing')
      }}</EmptyCategory>
      <EmptyCategory v-else>{{ t('roll.notDrawnYet') }}</EmptyCategory>
      <div>
        <Button @click="store.draw()">
          <DicesIcon />{{
            cardState.kind === 'drawn' ? t('roll.drawAgain') : t('roll.draw')
          }}
        </Button>
      </div>
      <RollPanel
        :characters="store.view.characters"
        :columns="store.view.columns"
        :preset="store.view.preset"
        @update:preset="store.setPreset"
      />
      <DiagnosticsList :entries="rollEntries(store.view.diagnostics)" />
    </template>
    <div v-else class="flex flex-col gap-4">
      <Skeleton class="h-8 w-120" />
      <Skeleton class="h-32 w-full" />
      <Skeleton class="h-40 w-full" />
    </div>
  </div>
</template>
