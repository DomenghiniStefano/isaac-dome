<script setup lang="ts">
import { ActivityIcon } from '@lucide/vue'
import { computed } from 'vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useMessages } from '@/i18n'
import { liveEntries } from '@/lib/diagnostics/live'
import { columnName } from '@/lib/graph/nodeState'
import type { AchievementRef } from '@/lib/ipc/types'
import { LoadStatus } from '@/stores/loadStatus'
import { useLiveStore } from '@/stores/views'
import ProfileError from './profile/ProfileError.vue'
import ScreenHeader from './ScreenHeader.vue'

const store = useLiveStore()
const { t } = useMessages()

// The archive is watched, not asked for: the app fills it in the background and says so.
void store.load()

const run = computed(() => store.view?.run ?? null)
const opens = computed(() => store.view?.opens ?? [])

// An achievement the game file does not name is a slot, and it is shown as one: a row that
// said nothing would hide that this run opens something we cannot name.
const achievementText = (a: AchievementRef): string =>
  a.kind === 'known' ? a.text : String(a.slot)
</script>

<template>
  <div class="flex max-w-250 flex-col gap-4">
    <ScreenHeader :icon="ActivityIcon" :title="t('routes.live')">{{
      t('live.intro')
    }}</ScreenHeader>
    <ProfileError
      v-if="store.status === LoadStatus.Failed"
      :error="store.error"
      @retry="store.load()"
    />
    <template v-else-if="store.view">
      <DiagnosticsList :entries="liveEntries(store.view.diagnostics)" />
      <Card v-if="run !== null">
        <CardHeader>
          <CardTitle>{{ t('live.run') }}</CardTitle>
        </CardHeader>
        <CardContent class="flex flex-wrap items-center gap-3">
          <span class="text-control text-highlight">{{
            run.character ?? t('runs.noCharacter')
          }}</span>
          <span class="text-label text-subtle-foreground"
            >{{ run.floors }} {{ t('live.floors') }}</span
          >
          <span class="text-label text-subtle-foreground">{{
            run.seedWords
          }}</span>
          <Badge v-if="run.online" :variant="BadgeVariant.Tag">{{
            t('runs.online.online')
          }}</Badge>
          <span
            v-if="run.heldActive !== null"
            class="text-label text-subtle-foreground"
            >{{ t('live.heldActive') }}:
            {{ run.heldActive.name ?? run.heldActive.id }}</span
          >
          <span class="text-label text-subtle-foreground"
            >{{ run.collected.length }} {{ t('live.collected') }}</span
          >
        </CardContent>
      </Card>
      <!-- Grouped by the cell it needs: the same boss read once, with everything under it. -->
      <Card v-for="open in opens" :key="`${open.character}-${open.column}`">
        <CardHeader>
          <CardTitle>{{
            t('live.beat', {
              column: columnName[open.column],
              character: open.characterName,
            })
          }}</CardTitle>
        </CardHeader>
        <CardContent class="flex flex-col gap-1">
          <span
            v-for="a in open.achievements"
            :key="achievementText(a)"
            class="text-row"
            >{{ achievementText(a) }}</span
          >
        </CardContent>
      </Card>
      <EmptyCategory v-if="run !== null && opens.length === 0">{{
        t('live.nothing')
      }}</EmptyCategory>
    </template>
    <div v-else class="flex flex-col gap-4">
      <Skeleton class="h-8 w-120" />
      <Skeleton class="h-40 w-full" />
    </div>
  </div>
</template>
