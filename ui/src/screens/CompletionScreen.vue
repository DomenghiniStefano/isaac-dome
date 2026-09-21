<script setup lang="ts">
import { InfoIcon } from '@lucide/vue'
import { computed } from 'vue'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Skeleton } from '@/components/ui/skeleton'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
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
  <!-- The shell's gutter is taken back by this box and handed to its children, so the band
       can be the full width of the page without overflowing it — the same move the wiki's
       screens make, and `WikiLanding.vue` records what doing it the other way round cost.
       The screen does not scroll: the band stays, and the matrix takes the height that is
       left and scrolls inside itself (spec 3.13a, card #58). -->
  <div class="-mx-5.5 flex h-full min-h-0 flex-col overflow-hidden">
    <div v-if="completion.status === LoadStatus.Failed" class="px-5.5 pt-5">
      <ProfileError :error="completion.error" @retry="completion.load()" />
    </div>
    <template v-else-if="completion.view && kpis">
      <CompletionHero :kpis="kpis" :widget-url="completion.view.widgetUrl" />
      <div class="flex min-h-0 flex-1 flex-col gap-3 px-5.5 pt-4 pb-5">
        <!-- Nothing readable is a state, not an empty grid: the cells still say "unknown". -->
        <Alert v-if="kpis.readable === 0">
          <InfoIcon />
          <AlertDescription>{{
            t('completion.nothingReadable')
          }}</AlertDescription>
        </Alert>
        <MarksMatrixCard :matrix="completion.view" class="min-h-0 flex-1" />
      </div>
    </template>
    <div v-else class="flex flex-col gap-4 px-5.5 pt-5">
      <Skeleton class="h-52 w-full" />
      <Skeleton class="min-h-0 flex-1" />
    </div>
  </div>
</template>
