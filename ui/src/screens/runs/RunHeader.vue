<script setup lang="ts">
import { CheckIcon, CopyIcon } from '@lucide/vue'
import { computed, ref } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import EntityChip from '@/components/runs/EntityChip.vue'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardContent } from '@/components/ui/card'
import { useFormat } from '@/composables/useFormat'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import { copyText } from '@/lib/clipboard/copyText'
import { Timing } from '@/lib/constants/timing'
import type { RunView } from '@/lib/ipc/types'
import { entityName } from '@/lib/runs/death'
import { RunDateKind, runDate } from '@/lib/runs/runDate'
import { outcomeText, sourceText } from '@/lib/runs/runLabels'

// Who was played, how it ended, when, and the seed to play it again. The ending and what killed
// you are the run's own words — the ending's name, the entities of the death line — and each
// entity is a chip that opens its wiki page when the wiki has one.
const props = defineProps<{ run: RunView }>()
const { t } = useMessages()
const format = useFormat()

const tone: Record<RunView['outcome']['kind'], BadgeVariant> = {
  won: BadgeVariant.Done,
  died: BadgeVariant.Blocked,
  abandoned: BadgeVariant.Partial,
  open: BadgeVariant.Now,
}

const when = computed(() => {
  const d = runDate(props.run)
  switch (d.kind) {
    case RunDateKind.Started:
      return `${format.date(d.at)} · ${t('runs.date.started', { time: format.time(d.at) })}`
    case RunDateKind.Written:
      return `${format.date(d.at)} · ${t('runs.date.written', { time: format.time(d.at) })}`
    case RunDateKind.Undated:
      return t('runs.date.undated')
    default:
      return assertNever(d)
  }
})

// The seed is what a player types to play the run again. The button says whether the copy took
// for a moment, then goes back to saying what it does.
const CopyState = { Idle: 'idle', Copied: 'copied', Failed: 'failed' } as const
type CopyState = (typeof CopyState)[keyof typeof CopyState]
const copyState = ref<CopyState>(CopyState.Idle)
const copySeed = async () => {
  copyState.value = (await copyText(props.run.seedWords))
    ? CopyState.Copied
    : CopyState.Failed
  setTimeout(() => (copyState.value = CopyState.Idle), Timing.CopiedNotice)
}
const copyLabel = computed(() => {
  switch (copyState.value) {
    case CopyState.Idle:
      return t('runs.copy')
    case CopyState.Copied:
      return t('runs.copied')
    case CopyState.Failed:
      return t('runs.copyFailed')
    default:
      return assertNever(copyState.value)
  }
})
</script>

<template>
  <Card>
    <CardContent class="flex flex-col gap-4 pt-4">
      <div class="flex items-center gap-3">
        <PixelSprite
          :url="run.characterHeadUrl"
          placeholder
          class="size-sprite shrink-0"
        />
        <div class="flex min-w-0 flex-col gap-1">
          <span v-if="run.character !== null" class="text-title">{{
            run.character
          }}</span>
          <EmptyValue v-else>{{ t('runs.noCharacter') }}</EmptyValue>
          <span class="text-label text-subtle-foreground">{{ when }}</span>
        </div>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        <Badge :variant="tone[run.outcome.kind]">{{
          t(outcomeText[run.outcome.kind])
        }}</Badge>
        <span v-if="run.outcome.kind === 'won'" class="text-row">{{
          t('runs.endedWith', { ending: run.outcome.ending })
        }}</span>
        <template v-if="run.outcome.kind === 'died'">
          <span class="text-label text-subtle-foreground">{{
            t('runs.page.killedBy')
          }}</span>
          <EntityChip
            :target="run.outcome.killer.page"
            :name="entityName(run.outcome.killer)"
            :detail="run.outcome.killer.raw"
            :icon-url="run.outcome.killer.iconUrl"
          />
          <template v-if="run.outcome.spawner !== null">
            <span class="text-label text-subtle-foreground">{{
              t('runs.page.spawnedBy')
            }}</span>
            <EntityChip
              :target="run.outcome.spawner.page"
              :name="entityName(run.outcome.spawner)"
              :detail="run.outcome.spawner.raw"
              :icon-url="run.outcome.spawner.iconUrl"
            />
          </template>
        </template>
      </div>

      <div class="flex flex-wrap items-center gap-3">
        <span class="text-row tabular-nums">{{ run.seedWords }}</span>
        <Button
          :variant="ButtonVariant.Outline"
          :size="ButtonSize.Compact"
          class="gap-2"
          @click="copySeed"
        >
          <CheckIcon v-if="copyState === CopyState.Copied" />
          <CopyIcon v-else />{{ copyLabel }}
        </Button>
        <Badge :variant="BadgeVariant.Tag">{{
          run.online ? t('runs.online.online') : t('runs.online.solo')
        }}</Badge>
        <span class="text-label text-subtle-foreground">{{
          t('runs.page.floors', { count: run.floors })
        }}</span>
        <span class="text-label text-subtle-foreground">{{
          t(sourceText[run.source.kind])
        }}</span>
      </div>
    </CardContent>
  </Card>
</template>
