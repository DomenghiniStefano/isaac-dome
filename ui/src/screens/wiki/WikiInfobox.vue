<script setup lang="ts">
import { computed } from 'vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import QualityPips from '@/screens/collection/QualityPips.vue'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import { CollectibleTemplate } from '@/lib/ipc/types'
import type { Entry, Inline, Target } from '@/lib/ipc/types'
import { useWikiStore } from '@/stores/wiki'
import InfoboxRow from './InfoboxRow.vue'
import { refsOf } from './infoboxRefs'
import { hasCard } from './transformationCard'

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
const refOf = (target: Target | null): Inline[] =>
  target === null ? [] : refsOf([target], wiki.titleOf)

// A transformation carries a card only where the page filled at least one of its three
// fields — Adult filled none, and an empty card would say the page has nothing.
//
// **Items and trinkets carried none at all until card #57**, and that was a hole and not a
// decision: the type has held their quality, recharge, prices, pools and tags since the
// dataset was first parsed, and on the 719 item pages — the most visited in the app — not
// one of those values ever reached the screen. The quote is the exception and stays out: it
// is the page's opening line and it is drawn on the band (`WikiHero.vue`), not twice.
const drawn = computed(() => {
  switch (infobox.value.kind) {
    case 'item':
    case 'trinket':
      return true
    case 'transformation':
      return hasCard(infobox.value, props.entry)
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
    <CardContent class="flex flex-col gap-3">
      <!-- The three facts every kind declares, drawn once: they live on the entry, not in
           the variant, so repeating them per kind would repeat the same markup four times. -->
      <dl class="flex flex-col gap-2">
        <!-- An achievement has no description from the wiki (its paper line is the quote,
             on the band): "none" here would contradict the two rows below that say what it
             gives and asks. One written by hand in `corrections.json` is still drawn. -->
        <InfoboxRow
          v-if="infobox.kind !== 'achievement' || entry.description.length > 0"
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
        <div class="flex flex-col gap-1.5 border-b border-hairline pb-2">
          <dt class="text-label text-subtle-foreground">
            {{ t('wiki.infobox.restrictions') }}
          </dt>
          <dd class="flex min-w-0 flex-wrap gap-1.5">
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
            class="flex flex-col gap-1"
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
      <dl
        v-else-if="infobox.kind === 'transformation'"
        class="flex flex-col gap-2"
      >
        <!-- `null` says nothing, never "3": the page didn't state a count in a form we can
             read, and defaulting it would be invisible against the pages that do. The row
             goes away with it — the "nessuno" this box draws for an empty value would read
             as "no item is needed", which is a claim about Adult that nobody measured. -->
        <InfoboxRow
          v-if="infobox.requires !== null"
          :label="t('wiki.infobox.requires')"
          :text="String(infobox.requires)"
        />
        <InfoboxRow
          v-if="infobox.contributors.length > 0"
          :label="t('wiki.infobox.contributors')"
          :inline="refsOf(infobox.contributors, wiki.titleOf)"
          v-bind="forward"
        />
        <!-- Fourteen of the sixteen pages say nothing here, and "nessuno" would read as a
             claim that the transformation acts on nothing. -->
        <InfoboxRow
          v-if="infobox.target.length > 0"
          :label="t('wiki.infobox.target')"
          :inline="infobox.target"
          v-bind="forward"
        />
      </dl>
      <!-- An item's numbers, and the first time they have been on screen. Recharge belongs
           to the activated template alone: on a passive the wiki leaves it empty, and an
           empty row there would read as "it recharges, and nobody wrote how fast". The pools
           are stated on 45 of 720 pages, so their absence is the wiki's silence and not the
           item's — the row goes with it rather than declaring none. -->
      <dl v-else-if="infobox.kind === 'item'" class="flex flex-col gap-2">
        <div class="flex flex-col gap-1 border-b border-hairline pb-2">
          <dt class="text-label text-subtle-foreground">
            {{ t('wiki.infobox.quality') }}
          </dt>
          <dd class="min-w-0"><QualityPips :quality="infobox.quality" /></dd>
        </div>
        <InfoboxRow
          v-if="infobox.template === CollectibleTemplate.Activated"
          :label="t('wiki.infobox.recharge')"
          :inline="infobox.recharge"
          v-bind="forward"
        />
        <InfoboxRow
          :label="t('wiki.infobox.devilPrice')"
          :inline="infobox.devilPrice"
          v-bind="forward"
        />
        <InfoboxRow
          :label="t('wiki.infobox.shopPrice')"
          :inline="infobox.shopPrice"
          v-bind="forward"
        />
        <InfoboxRow
          v-if="infobox.pools.length > 0"
          :label="t('wiki.infobox.pools')"
          :inline="infobox.pools"
          v-bind="forward"
        />
        <div v-if="infobox.tags.length > 0" class="flex flex-col gap-1.5 pt-1">
          <dt class="text-label text-subtle-foreground">
            {{ t('wiki.infobox.tags') }}
          </dt>
          <dd class="flex flex-wrap gap-1.5">
            <Badge v-for="tag in infobox.tags" :key="tag">{{ tag }}</Badge>
          </dd>
        </div>
      </dl>
      <dl v-else-if="infobox.kind === 'trinket'" class="flex flex-col gap-2">
        <InfoboxRow
          v-if="infobox.pools.length > 0"
          :label="t('wiki.infobox.pools')"
          :inline="infobox.pools"
          v-bind="forward"
        />
        <div v-if="infobox.tags.length > 0" class="flex flex-col gap-1.5 pt-1">
          <dt class="text-label text-subtle-foreground">
            {{ t('wiki.infobox.tags') }}
          </dt>
          <dd class="flex flex-wrap gap-1.5">
            <Badge v-for="tag in infobox.tags" :key="tag">{{ tag }}</Badge>
          </dd>
        </div>
      </dl>
    </CardContent>
  </Card>
</template>
