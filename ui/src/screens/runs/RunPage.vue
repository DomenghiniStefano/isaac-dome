<script setup lang="ts">
import { ArrowLeftIcon } from '@lucide/vue'
import { computed } from 'vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import ProfileError from '@/components/data-state/ProfileError.vue'
import ScreenSkeleton from '@/components/data-state/ScreenSkeleton.vue'
import EntityChip from '@/components/runs/EntityChip.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { PageScroll } from '@/components/ui/virtual'
import { useTabView } from '@/composables/useTabView'
import { useMessages } from '@/i18n'
import type { RunItemView } from '@/lib/runs/itemViews'
import { runKey as keyOf } from '@/lib/runs/runKey'
import { RouteName } from '@/router/routeTable'
import { LoadStatus } from '@/stores/loadStatus'
import { useTabsStore } from '@/stores/tabs'
import { useRunsStore } from '@/stores/views'
import RunHeader from './RunHeader.vue'
import RunItems from './RunItems.vue'
import RunRoute from './RunRoute.vue'
import { runsView } from './tabView'

// One run, everything the log said about it. The run is looked up by its key in the archive the
// list already holds; a key that matches nothing — the archive was rebuilt, the run's source is
// gone, a link from an older version — says so, with the way back, rather than showing an empty
// page or the list in silence.
const props = defineProps<{ runKey: string }>()
const store = useRunsStore()
const tabs = useTabsStore()
const { t } = useMessages()
const { reading, update } = useTabView(runsView)

const run = computed(
  () => store.view?.runs.find((r) => keyOf(r) === props.runKey) ?? null,
)
const toList = (event: MouseEvent) =>
  tabs.go({ name: RouteName.Runs }, event.ctrlKey)
const setView = (itemView: RunItemView) => update({ itemView })
</script>

<template>
  <!-- The page scrolls as a whole (`PageScroll`), like every screen. -->
  <PageScroll>
    <div class="flex flex-col gap-4 px-5.5 pt-5 pb-5">
      <div>
        <Button
          :variant="ButtonVariant.Ghost"
          :size="ButtonSize.Compact"
          class="gap-2"
          @click="toList"
          ><ArrowLeftIcon />{{ t('runs.page.backToList') }}</Button
        >
      </div>
      <ProfileError
        v-if="store.status === LoadStatus.Failed"
        :error="store.error"
        @retry="store.load()"
      />
      <ScreenSkeleton v-else-if="store.view === null" untitled />
      <EmptyCategory v-else-if="run === null">{{
        t('runs.page.noSuchRun')
      }}</EmptyCategory>
      <template v-else>
        <RunHeader :run="run" />
        <Card v-if="run.achievements.length > 0">
          <CardHeader>
            <CardTitle>{{ t('runs.page.achievements') }}</CardTitle>
          </CardHeader>
          <CardContent class="flex flex-wrap gap-2">
            <EntityChip
              v-for="a in run.achievements"
              :key="a.id"
              :target="{ kind: 'achievement', id: a.id }"
              :name="a.text ?? t('runs.page.achievementId', { id: a.id })"
              :icon-url="a.iconUrl"
            />
          </CardContent>
        </Card>
        <RunRoute :run="run" />
        <RunItems :run="run" :view="reading.itemView" @update:view="setView" />
      </template>
    </div>
  </PageScroll>
</template>
