<script setup lang="ts">
import { InfoIcon } from '@lucide/vue'
import { computed } from 'vue'
import { Alert, AlertDescription } from '@/components/ui/alert'
import ScreenSkeleton from '@/components/data-state/ScreenSkeleton.vue'
import { SkeletonBlock } from '@/components/data-state/skeletonBlock'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { vScrollMemory } from '@/directives/scrollMemory'
import { useMessages } from '@/i18n'
import { completionKpis } from '@/lib/completion/completionView'
import { useCompletionStore } from '@/stores/views'
import { LoadStatus } from '@/stores/loadStatus'
import CompletionHero from './completion/CompletionHero.vue'
import MarksMatrixCard from './completion/MarksMatrixCard.vue'
import ProfileError from './profile/ProfileError.vue'

const completion = useCompletionStore()
const { t } = useMessages()

useOnActiveProfile(() => completion.load())

const kpis = computed(() =>
  completion.view ? completionKpis(completion.view) : null,
)
</script>

<template>
  <!-- The gutter is the children's, so the band can be the full width of the page without
       overflowing it — the same shape as the wiki's screens, and `WikiLanding.vue` records
       what doing it the other way round cost.
       **The screen flows and scrolls, the matrix does not scroll on its own** (card #85). It
       used to be the other way round (card #58): the band stayed and the matrix scrolled in
       the height left under it, which on a small window was three rows under a band that
       never moved. Now the band, the card's title and the legend scroll away with the page,
       and the boss header is the one thing that stays, pinned to the top of the screen. -->
  <div v-scroll-memory="'page'" class="flex h-full flex-col overflow-y-auto">
    <div v-if="completion.status === LoadStatus.Failed" class="px-5.5 pt-5">
      <ProfileError :error="completion.error" @retry="completion.load()" />
    </div>
    <template v-else-if="completion.view && kpis">
      <CompletionHero :kpis="kpis" :widget-url="completion.view.widgetUrl" />
      <div class="flex flex-col gap-3 px-5.5 pt-4 pb-5">
        <!-- Nothing readable is a state, not an empty grid: the cells still say "unknown". -->
        <Alert v-if="kpis.readable === 0">
          <InfoIcon />
          <AlertDescription>{{
            t('completion.nothingReadable')
          }}</AlertDescription>
        </Alert>
        <MarksMatrixCard :matrix="completion.view" />
      </div>
    </template>
    <ScreenSkeleton
      v-else
      untitled
      class="flex-1 px-5.5 pt-5 pb-5"
      :blocks="[SkeletonBlock.Hero, SkeletonBlock.Fill]"
    />
  </div>
</template>
