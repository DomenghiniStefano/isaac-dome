<script setup lang="ts">
import { computed, provide } from 'vue'
import type { Component } from 'vue'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import type { Entry, Infobox, Target } from '@/lib/ipc/types'
import { useWikiStore } from '@/stores/wiki'
import InfoboxRow from './InfoboxRow.vue'
import InfoboxAchievement from './infobox/InfoboxAchievement.vue'
import InfoboxBoss from './infobox/InfoboxBoss.vue'
import InfoboxChallenge from './infobox/InfoboxChallenge.vue'
import InfoboxCharacter from './infobox/InfoboxCharacter.vue'
import InfoboxItem from './infobox/InfoboxItem.vue'
import InfoboxTransformation from './infobox/InfoboxTransformation.vue'
import InfoboxTrinket from './infobox/InfoboxTrinket.vue'
import { infoboxLinksKey } from './infobox/links'
import { refOf } from './infoboxRefs'
import { hasCard } from './transformationCard'

// The whole entry, not just its infobox: the description, the editions and "unlocked by" live
// on the entry, because they are not specific to a kind.
const props = defineProps<{
  entry: Entry
  canOpen?: (target: Target) => boolean
}>()
const infobox = computed(() => props.entry.infobox)
const emit = defineEmits<{ navigate: [target: Target, newTab: boolean] }>()
const wiki = useWikiStore()
const { t } = useMessages()

// What every row receives, however deep the kind's body draws it: whether a reference opens
// and the way back up. Getters, so a row reads the props as they are now.
provide(infoboxLinksKey, {
  get canOpen() {
    return props.canOpen
  },
  navigate: (target, newTab) => emit('navigate', target, newTab),
})

// One body per kind. A record and not a chain of `v-else-if`: a new infobox kind fails to
// compile until it says here what it shows.
const bodies: Record<Infobox['kind'], Component> = {
  achievement: InfoboxAchievement,
  boss: InfoboxBoss,
  challenge: InfoboxChallenge,
  character: InfoboxCharacter,
  item: InfoboxItem,
  transformation: InfoboxTransformation,
  trinket: InfoboxTrinket,
}

// A transformation carries a card only where the page filled at least one of its three
// fields — Adult filled none, and an empty card would say the page has nothing. Every other
// kind has one. The quote stays out of an item's card: it is the page's opening line and it is
// drawn on the band (`WikiHero.vue`), not twice.
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
</script>

<template>
  <Card v-if="drawn">
    <CardHeader>
      <CardTitle>{{ t('wiki.infobox.title') }}</CardTitle>
    </CardHeader>
    <CardContent class="flex flex-col gap-3">
      <!-- The facts every kind declares, drawn once: they live on the entry, not in the
           variant, so repeating them per kind would repeat the same markup seven times. -->
      <dl class="flex flex-col gap-2">
        <!-- An achievement has no description from the wiki (its paper line is the quote,
             on the band): "none" here would contradict the two rows below that say what it
             gives and asks. One written by hand in `corrections.json` is still drawn. -->
        <InfoboxRow
          v-if="infobox.kind !== 'achievement' || entry.description.length > 0"
          :label="t('wiki.infobox.description')"
          :inline="entry.description"
        />
        <InfoboxRow
          :label="t('wiki.infobox.unlockedBy')"
          :inline="refOf(entry.unlockedBy, wiki.titleOf)"
        />
      </dl>
      <component :is="bodies[infobox.kind]" :infobox="infobox" />
    </CardContent>
  </Card>
</template>
