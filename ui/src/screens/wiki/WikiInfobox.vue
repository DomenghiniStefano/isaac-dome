<script setup lang="ts">
import { computed } from 'vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import type { Infobox, Inline, Target } from '@/lib/ipc/types'
import { pageKey } from '@/lib/wiki/pageKey'
import { useWikiStore } from '@/stores/wiki'
import InfoboxRow from './InfoboxRow.vue'

const props = defineProps<{
  infobox: Infobox
  iconFor?: (target: Target) => string | null
  canOpen?: (target: Target) => boolean
}>()
const emit = defineEmits<{ navigate: [target: Target, newTab: boolean] }>()
const wiki = useWikiStore()
const { t } = useMessages()

// What every row receives: the two resolvers and the way back up.
const forward = computed(() => ({
  iconFor: props.iconFor,
  canOpen: props.canOpen,
  onNavigate: (target: Target, newTab: boolean) =>
    emit('navigate', target, newTab),
}))

// A `Target` field becomes a one-reference inline, so it links like any other; its label is
// the page's title from the index, or its key while the index isn't known.
const refOf = (target: Target | null): Inline[] => {
  if (target === null) return []
  const key = pageKey(target)
  const label = (key === null ? null : wiki.titleOf(key)) ?? key ?? ''
  return [{ kind: 'ref', target, label }]
}

// Items and trinkets carry no card: their infobox is the figure and the title.
const drawn = computed(() => {
  switch (props.infobox.kind) {
    case 'item':
    case 'trinket':
      return false
    case 'achievement':
    case 'boss':
    case 'challenge':
    case 'character':
      return true
    default:
      return assertNever(props.infobox)
  }
})

// A challenge's restrictions, as the export lists them: only the ones that apply.
const restrictions = computed((): string[] => {
  const box = props.infobox
  if (box.kind !== 'challenge') return []
  return [
    ...(box.blindfolded ? [t('wiki.infobox.blindfolded')] : []),
    ...(box.hasShops ? [] : [t('wiki.infobox.noShops')]),
    ...(box.hasTreasureRooms ? [] : [t('wiki.infobox.noTreasureRooms')]),
  ]
})

const stats = computed(() => {
  const box = props.infobox
  if (box.kind !== 'character') return []
  return [
    { label: t('wiki.infobox.damage'), value: box.damage },
    { label: t('wiki.infobox.range'), value: box.range },
    { label: t('wiki.infobox.speed'), value: box.speed },
    { label: t('wiki.infobox.luck'), value: box.luck },
    { label: t('wiki.infobox.shotSpeed'), value: box.shotSpeed },
  ]
})
</script>

<template>
  <!-- One card per kind, exhaustive: a new infobox kind has to say here what it shows. -->
  <Card v-if="drawn">
    <CardHeader>
      <CardTitle>{{ t('wiki.infobox.title') }}</CardTitle>
    </CardHeader>
    <CardContent>
      <dl v-if="infobox.kind === 'achievement'" class="flex flex-col gap-2">
        <InfoboxRow
          :label="t('wiki.infobox.description')"
          :text="infobox.description"
        />
        <InfoboxRow
          :label="t('wiki.infobox.requirements')"
          :inline="infobox.requirements"
          v-bind="forward"
        />
        <InfoboxRow
          :label="t('wiki.infobox.unlocks')"
          :inline="refOf(infobox.unlocks)"
          v-bind="forward"
        />
      </dl>
      <dl v-else-if="infobox.kind === 'boss'" class="flex flex-col gap-2">
        <InfoboxRow
          :label="t('wiki.infobox.baseHp')"
          :text="infobox.baseHp === null ? null : String(infobox.baseHp)"
        />
        <InfoboxRow
          :label="t('wiki.infobox.environment')"
          :inline="infobox.environment"
          v-bind="forward"
        />
        <InfoboxRow
          :label="t('wiki.infobox.pool')"
          :inline="infobox.pool"
          v-bind="forward"
        />
        <InfoboxRow
          :label="t('wiki.infobox.unlockedBy')"
          :inline="refOf(infobox.unlockedBy)"
          v-bind="forward"
        />
      </dl>
      <dl v-else-if="infobox.kind === 'challenge'" class="flex flex-col gap-2">
        <InfoboxRow
          :label="t('wiki.infobox.goal')"
          :inline="infobox.goal"
          v-bind="forward"
        />
        <InfoboxRow
          :label="t('wiki.infobox.items')"
          :inline="infobox.items"
          v-bind="forward"
        />
        <InfoboxRow
          :label="t('wiki.infobox.trinkets')"
          :inline="infobox.trinkets"
          v-bind="forward"
        />
        <InfoboxRow
          :label="t('wiki.infobox.pickups')"
          :inline="infobox.pickups"
          v-bind="forward"
        />
        <InfoboxRow
          :label="t('wiki.infobox.health')"
          :inline="infobox.health"
          v-bind="forward"
        />
        <InfoboxRow
          :label="t('wiki.infobox.curse')"
          :inline="infobox.curse"
          v-bind="forward"
        />
        <div class="flex gap-4">
          <dt class="w-32 shrink-0 text-label text-subtle-foreground">
            {{ t('wiki.infobox.restrictions') }}
          </dt>
          <dd class="flex min-w-0 flex-1 flex-wrap gap-1.5">
            <Badge
              v-for="restriction in restrictions"
              :key="restriction"
              :variant="BadgeVariant.Challenge"
              >{{ restriction }}</Badge
            >
            <span
              v-if="restrictions.length === 0"
              class="text-row text-foreground-soft"
              >{{ t('wiki.infobox.noRestrictions') }}</span
            >
          </dd>
        </div>
        <InfoboxRow
          :label="t('wiki.infobox.unlocks')"
          :inline="refOf(infobox.unlocks)"
          v-bind="forward"
        />
        <InfoboxRow
          :label="t('wiki.infobox.unlockedBy')"
          :inline="refOf(infobox.unlockedBy)"
          v-bind="forward"
        />
      </dl>
      <dl v-else-if="infobox.kind === 'character'" class="flex flex-col gap-2">
        <InfoboxRow
          :label="t('wiki.infobox.health')"
          :inline="infobox.health"
          v-bind="forward"
        />
        <div class="flex flex-wrap gap-4 py-1">
          <span
            v-for="stat in stats"
            :key="stat.label"
            class="flex flex-col gap-0.75"
          >
            <span class="text-label text-subtle-foreground">{{
              stat.label
            }}</span>
            <span class="text-row text-foreground tabular-nums">{{
              stat.value
            }}</span>
          </span>
        </div>
        <InfoboxRow
          :label="t('wiki.infobox.pickups')"
          :inline="infobox.pickups"
          v-bind="forward"
        />
        <InfoboxRow
          :label="t('wiki.infobox.collectibles')"
          :inline="infobox.collectibles"
          v-bind="forward"
        />
        <InfoboxRow
          :label="t('wiki.infobox.unlockedBy')"
          :inline="refOf(infobox.unlockedBy)"
          v-bind="forward"
        />
      </dl>
    </CardContent>
  </Card>
</template>
