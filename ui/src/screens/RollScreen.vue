<script setup lang="ts">
import { vScrollMemory } from '@/directives/scrollMemory'
import ScreenSkeleton from '@/components/data-state/ScreenSkeleton.vue'
import { SkeletonBlock } from '@/components/data-state/skeletonBlock'
import { DicesIcon } from '@lucide/vue'
import { computed } from 'vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import { Button } from '@/components/ui/button'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useMessages } from '@/i18n'
import { rollEntries } from '@/lib/diagnostics/roll'
import { AppEvent } from '@/lib/window/appEvents'
import { useAppEvent } from '@/composables/useAppEvent'
import { LoadStatus } from '@/stores/loadStatus'
import { useRollStore } from '@/stores/roll'
import ProfileError from '@/components/data-state/ProfileError.vue'
import RollCard from './roll/RollCard.vue'
import RollPanel from './roll/RollPanel.vue'
import { rollCardState } from './roll/rollText'
import ScreenHeader from '@/components/screen/ScreenHeader.vue'

const store = useRollStore()
const { t } = useMessages()

useOnActiveProfile(() => store.load())

// A draw made in another window is a read in this one: the document is one row, and the two
// windows must never show two different cards for it.
useAppEvent(AppEvent.RollChanged, () => {
  void store.load()
})

const cardState = computed(() =>
  store.view ? rollCardState(store.view.drawn, store.view.deck) : null,
)
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
    <ScreenSkeleton
      v-else
      :blocks="[SkeletonBlock.ShortCard, SkeletonBlock.Card]"
    />
  </div>
</template>
