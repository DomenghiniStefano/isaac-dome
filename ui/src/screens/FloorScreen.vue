<script setup lang="ts">
import { Grid3x3Icon } from '@lucide/vue'
import { computed } from 'vue'
import DiagnosticsList from '@/components/diagnostics/DiagnosticsList.vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import KpiTile from '@/components/kpi/KpiTile.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import { floorEntries } from '@/lib/diagnostics/floor'
import { useFloorStore } from '@/stores/floor'
import type { FloorSolutionView, TargetView } from '@/lib/ipc/types'
import FloorGrid from './floor/FloorGrid.vue'
import FloorLegend from './floor/FloorLegend.vue'
import ScreenHeader from './ScreenHeader.vue'

const store = useFloorStore()
const { t } = useMessages()

// The grid answers from the moment it opens: an empty floor is a diagnostic, not a blank.
void store.solve()

const solutions = computed(() => store.view?.solutions ?? [])

const solutionFor = (target: TargetView): FloorSolutionView | null =>
  solutions.value.find((s) => s.target === target) ?? null

// The order the three targets are read in: the one a player looks for on every floor, then
// the one that needs the whole map painted, then the one this grid can only partly judge.
const order: TargetView[] = ['secret', 'superSecret', 'ultraSecret']
</script>

<template>
  <div class="flex h-full flex-col gap-4 overflow-y-auto pt-5 pb-15">
    <ScreenHeader :icon="Grid3x3Icon" :title="t('routes.floor')">{{
      t('floor.intro')
    }}</ScreenHeader>

    <DiagnosticsList
      v-if="store.view"
      :entries="floorEntries(store.view.diagnostics)"
    />

    <!-- The failure is said, and the grid stays: what you painted is yours, and losing it
         because a command did not answer would be the app throwing away your work. -->
    <EmptyCategory v-if="store.failed">{{ t('floor.failed') }}</EmptyCategory>

    <Card>
      <CardHeader class="flex-wrap items-center justify-between gap-3">
        <FloorLegend :brush="store.brush" @pick="store.brush = $event" />
        <Button
          :variant="ButtonVariant.Secondary"
          :size="ButtonSize.Compact"
          @click="store.clear()"
          >{{ t('floor.clear') }}</Button
        >
      </CardHeader>
      <CardContent class="flex flex-col gap-4">
        <FloorGrid
          :cells="store.cells"
          :candidates="solutionFor('secret')?.candidates ?? []"
          @stroke="store.stroke($event)"
        />
        <KpiTile
          :value="store.view?.painted ?? 0"
          :label="t('floor.painted')"
        />
      </CardContent>
    </Card>

    <Card v-for="target in order" :key="target">
      <CardHeader>
        <CardTitle>{{ t(`floor.target.${target}`) }}</CardTitle>
      </CardHeader>
      <CardContent class="flex flex-col gap-3">
        <template v-if="(solutionFor(target)?.candidates.length ?? 0) > 0">
          <!-- Every row carries the sentence that lit it and the page it came from. That is
               the CC BY-SA attribution reaching the person reading the screen, not a layout
               detail: the rules are quotations, and a quotation without its source is not
               one. -->
          <div
            v-for="candidate in solutionFor(target)?.candidates ?? []"
            :key="candidate.cell"
            class="flex flex-col gap-1 border-b border-hairline pb-2"
          >
            <div class="flex items-center gap-3">
              <span class="text-label tabular-nums">{{ candidate.cell }}</span>
              <span class="text-caption text-subtle-foreground"
                >{{ candidate.neighbours }} {{ t('floor.neighbours') }}</span
              >
            </div>
            <div
              v-for="rule in candidate.applied"
              :key="rule.id"
              class="flex flex-col"
            >
              <span class="text-caption text-foreground-soft">{{
                rule.quote
              }}</span>
              <span class="text-caption text-faint-foreground"
                >{{ t('floor.source') }}: {{ rule.url }}</span
              >
            </div>
          </div>
        </template>
        <EmptyCategory v-else>{{ t('floor.none') }}</EmptyCategory>

        <!-- What the grid cannot judge is shown under what it can, never instead of it: a
             rule we cannot evaluate is not a rule that allows everything. -->
        <template v-if="(solutionFor(target)?.unresolved.length ?? 0) > 0">
          <span class="text-label text-subtle-foreground">{{
            t('floor.unresolved')
          }}</span>
          <div
            v-for="item in solutionFor(target)?.unresolved ?? []"
            :key="item.rule"
            class="flex flex-col"
          >
            <span class="text-caption text-foreground-soft">{{
              item.quote
            }}</span>
            <span class="text-caption text-faint-foreground">{{
              item.note
            }}</span>
            <span class="text-caption text-faint-foreground"
              >{{ t('floor.source') }}: {{ item.url }}</span
            >
          </div>
        </template>
      </CardContent>
    </Card>
  </div>
</template>
