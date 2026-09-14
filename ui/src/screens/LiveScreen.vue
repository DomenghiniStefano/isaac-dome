<script setup lang="ts">
import { ActivityIcon } from '@lucide/vue'
import { computed } from 'vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import KpiTile from '@/components/kpi/KpiTile.vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Skeleton } from '@/components/ui/skeleton'
import { useMessages } from '@/i18n'
import { liveEntries } from '@/lib/diagnostics/live'
import { columnName } from '@/lib/graph/nodeState'
import type { AchievementRef, Target } from '@/lib/ipc/types'
import { LoadStatus } from '@/stores/loadStatus'
import { useLiveStore } from '@/stores/views'
import EntityChip from '@/components/runs/EntityChip.vue'
import ItemChips from '@/components/runs/ItemChips.vue'
import LiveMarksRow from './live/LiveMarksRow.vue'
import ProfileError from './profile/ProfileError.vue'
import ScreenHeader from './ScreenHeader.vue'

const store = useLiveStore()
const { t } = useMessages()

// The archive is watched, not asked for: the app fills it in the background and says so.
void store.load()

const run = computed(() => store.view?.run ?? null)
const opens = computed(() => store.view?.opens ?? [])
const marks = computed(() => store.view?.marks ?? null)

// How many achievements this run could open in total: the number the screen exists for, and
// the only one on it that is a sum rather than a reading.
const openCount = computed(() =>
  opens.value.reduce((n, o) => n + o.achievements.length, 0),
)

// Who is being played, in the words the app has: the matrix row knows the form — Tainted or
// base — while the log's item line only ever writes the base name.
const characterName = computed(
  () =>
    marks.value?.rows[0]?.character ??
    run.value?.character ??
    t('runs.noCharacter'),
)

const text = (a: AchievementRef): string =>
  a.kind === 'known' ? a.text : String(a.slot)
const iconOf = (a: AchievementRef): string | null =>
  a.kind === 'known' ? a.iconUrl : null
// The game's own line about how it is earned, which the tooltip can say before the click.
const conditionOf = (a: AchievementRef): string | null =>
  a.kind === 'known' ? a.condition : null
const targetOf = (a: AchievementRef): Target | null =>
  a.kind === 'known' ? { kind: 'achievement', id: a.id } : null
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
      <template v-if="run !== null">
        <!-- The run itself, as four readings and no percentages: floors walked, items held,
             achievements the log announced during it, and what finishing it would open. -->
        <div class="grid grid-cols-2 gap-3 sm:grid-cols-4">
          <KpiTile :value="run.floors" :label="t('live.floors')" />
          <KpiTile
            :value="run.collected.length"
            :label="t('live.collectedItems')"
          />
          <KpiTile
            :value="run.achievements.length"
            :label="t('live.unlockedHere')"
          />
          <KpiTile :value="openCount" :label="t('live.wouldOpen')" />
        </div>
        <Card>
          <!-- The identity reads left to right like a line of prose: who, on what seed, with
               whom. Centring it would make the seed look like a title. -->
          <CardHeader class="flex-wrap items-center justify-start gap-3">
            <EntityChip
              :target="
                run.characterId === null
                  ? null
                  : { kind: 'character', id: run.characterId }
              "
              :name="characterName"
              :detail="null"
              :icon-url="marks?.rows[0]?.headUrl ?? null"
            />
            <span class="text-label text-subtle-foreground">{{
              run.seedWords
            }}</span>
            <Badge v-if="run.online" :variant="BadgeVariant.Tag">{{
              t('runs.online.online')
            }}</Badge>
          </CardHeader>
          <CardContent class="flex flex-col gap-4">
            <div
              v-if="run.startingItems.length > 0"
              class="flex flex-col gap-1"
            >
              <span class="text-label text-subtle-foreground">{{
                t('live.startingItems')
              }}</span>
              <ItemChips :items="run.startingItems" :held="run.heldActive" />
            </div>
            <div v-if="run.collected.length > 0" class="flex flex-col gap-1">
              <span class="text-label text-subtle-foreground">{{
                t('live.items')
              }}</span>
              <ItemChips :items="run.collected" :held="run.heldActive" />
            </div>
          </CardContent>
        </Card>
        <Card v-if="marks !== null && marks.rows.length > 0">
          <CardHeader>
            <CardTitle>{{ t('live.marks') }}</CardTitle>
          </CardHeader>
          <CardContent class="flex flex-col gap-3">
            <LiveMarksRow
              v-for="row in marks.rows"
              :key="row.character"
              :row="row"
              :bosses="marks.bosses"
              :art="marks.art"
            />
          </CardContent>
        </Card>
      </template>
      <!-- Grouped by the cell it needs: the same boss read once, with everything under it.
           Two columns where there is room: a cell’s list is short, and this screen is read
           while something else has your attention. -->
      <div class="grid gap-4 lg:grid-cols-2">
        <Card v-for="open in opens" :key="`${open.character}-${open.column}`">
          <CardHeader>
            <CardTitle>{{
              t('live.beat', {
                column: columnName[open.column],
                character: open.characterName,
              })
            }}</CardTitle>
          </CardHeader>
          <CardContent class="flex flex-col gap-2">
            <span
              v-for="a in open.achievements"
              :key="text(a.achievement)"
              class="flex items-center gap-2"
            >
              <EntityChip
                :target="targetOf(a.achievement)"
                :name="text(a.achievement)"
                :detail="conditionOf(a.achievement)"
                :icon-url="iconOf(a.achievement)"
              />
              <span class="ml-auto text-label text-subtle-foreground">{{
                a.fanOut > 0
                  ? t('live.opens', { count: a.fanOut })
                  : t('live.opensNothingMore')
              }}</span>
            </span>
          </CardContent>
        </Card>
      </div>
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
