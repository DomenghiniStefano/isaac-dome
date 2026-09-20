<script setup lang="ts">
import { Grid2x2Icon, InfoIcon } from '@lucide/vue'
import { computed } from 'vue'
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Skeleton } from '@/components/ui/skeleton'
import { useOnActiveProfile } from '@/composables/useOnActiveProfile'
import { useMessages } from '@/i18n'
import { completionKpis } from '@/lib/completion/completionView'
import { useCompletionStore } from '@/stores/views'
import { LoadStatus } from '@/stores/loadStatus'
import ScreenHeader from './ScreenHeader.vue'
import CompletionKpis from './completion/CompletionKpis.vue'
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
  <div class="flex h-full flex-col gap-4 overflow-y-auto pt-5 pb-15">
    <ScreenHeader :icon="Grid2x2Icon" :title="t('routes.completion')">{{
      t('completion.intro')
    }}</ScreenHeader>
    <ProfileError
      v-if="completion.status === LoadStatus.Failed"
      :error="completion.error"
      @retry="completion.load()"
    />
    <template v-else-if="completion.view && kpis">
      <CompletionKpis :kpis="kpis" />
      <!-- Nothing readable is a state, not an empty grid: the cells still say "unknown". -->
      <Alert v-if="kpis.readable === 0">
        <InfoIcon />
        <AlertDescription>{{
          t('completion.nothingReadable')
        }}</AlertDescription>
      </Alert>
      <MarksMatrixCard :matrix="completion.view" />
    </template>
    <div v-else class="flex flex-col gap-4">
      <Skeleton class="h-22 w-full" />
      <Skeleton class="h-150 w-full" />
    </div>
  </div>
</template>
