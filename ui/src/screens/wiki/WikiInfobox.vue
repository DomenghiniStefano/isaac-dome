<script setup lang="ts">
import { computed } from 'vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import type { Entry, Inline, Target } from '@/lib/ipc/types'
import { pageKey } from '@/lib/wiki/pageKey'
import { useWikiStore } from '@/stores/wiki'
import InfoboxRow from './InfoboxRow.vue'

// The whole entry, not just its infobox: since 2026-09-13 the description, the editions and
// "unlocked by" live on the entry, because they are not specific to a kind.
const props = defineProps<{
  entry: Entry
  iconFor?: (target: Target) => string | null
  canOpen?: (target: Target) => boolean
}>()
const infobox = computed(() => props.entry.infobox)
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
  switch (infobox.value.kind) {
    case 'item':
    case 'trinket':
    // A transformation's card has no rows yet: `requires`, `contributors` and `target` are
    // in the contract and nothing draws them. Until they are drawn, no card — an empty one
    // would say the page has nothing. This variant reached the union when the contract
    // started being generated; before that the mirror did not have it and this switch
    // reached `assertNever`, which throws.
    case 'transformation':
      return false
    case 'achievement':
    case 'boss':
    case 'challenge':
    case 'character':
      return true
    default:
      return assertNever(infobox.value)
  }
})

// A challenge's restrictions, as the export lists them: only the ones that apply.
const restrictions = computed((): string[] => {
  const box = infobox.value
  if (box.kind !== 'challenge') return []
  return [
    ...(box.blindfolded ? [t('wiki.infobox.blindfolded')] : []),
    ...(box.hasShops ? [] : [t('wiki.infobox.noShops')]),
    ...(box.hasTreasureRooms ? [] : [t('wiki.infobox.noTreasureRooms')]),
  ]
})

const stats = computed(() => {
  const box = infobox.value
  if (box.kind !== 'character') return []
  return [
    { label: t('wiki.infobox.damage'), value: box.damage },
    { label: t('wiki.infobox.tears'), value: box.tears },
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
      <!-- The three facts every kind declares, drawn once: they live on the entry, not in
           the variant, so repeating them per kind would repeat the same markup four times. -->
      <dl class="flex flex-col gap-2">
        <InfoboxRow
          :label="t('wiki.infobox.description')"
          :inline="entry.description"
          v-bind="forward"
        />
        <InfoboxRow
          :label="t('wiki.infobox.unlockedBy')"
          :inline="refOf(entry.unlockedBy)"
          v-bind="forward"
        />
      </dl>
      <dl v-if="infobox.kind === 'achievement'" class="flex flex-col gap-2">
        <InfoboxRow
          :label="t('wiki.infobox.requirements')"
          :inline="infobox.requirements"
          v-bind="forward"
        />
        <InfoboxRow
          :label="t('wiki.infobox.notes')"
          :inline="infobox.notes"
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
      </dl>
    </CardContent>
  </Card>
</template>
