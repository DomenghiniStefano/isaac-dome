<script setup lang="ts">
import SearchRow from '@/components/search/SearchRow.vue'
import type { SearchHit } from '@/lib/ipc/types'
import { RowGroup } from '@/lib/search/rows'
import type { SearchRow as Row } from '@/lib/search/rows'
import { RouteName } from '@/router/routeTable'
import KitSection from '../../KitSection.vue'

// The palette can't be opened here, so its four row shapes are drawn on their own: a title
// match, a condition, a section fragment, and a screen.
const hit = (over: Partial<SearchHit>): SearchHit => ({
  target: { kind: 'item', id: 105 },
  title: 'The D6',
  iconUrl: null,
  hasPage: true,
  match: { kind: 'title' },
  progress: 'pending',
  ...over,
})

const rows: Row[] = [
  {
    kind: 'hit',
    key: 'title',
    group: RowGroup.Wiki,
    hit: hit({}),
    location: { name: RouteName.Wiki },
  },
  {
    kind: 'hit',
    key: 'condition',
    group: RowGroup.Unlock,
    hit: hit({
      target: { kind: 'achievement', id: 1 },
      title: 'You unlocked The D6',
      match: { kind: 'condition', text: "Defeat Mom's Heart 10 times" },
      progress: 'done',
    }),
    location: { name: RouteName.Unlock },
  },
  {
    kind: 'hit',
    key: 'section',
    group: RowGroup.Wiki,
    hit: hit({
      match: {
        kind: 'section',
        section: 'effects',
        before: 'Rerolls the items in the ',
        matched: 'Treasure',
        after: ' Room',
      },
      progress: 'none',
    }),
    location: { name: RouteName.Wiki },
  },
  {
    kind: 'screen',
    key: 'screen',
    group: RowGroup.Screens,
    entry: {
      key: 'route-collection',
      label: 'routes.collection',
      text: 'Collezione',
      location: { name: RouteName.Collection },
    },
    location: { name: RouteName.Collection },
  },
]
</script>

<template>
  <KitSection title="Riga di ricerca">
    <div class="flex w-full max-w-150 flex-col gap-1">
      <div v-for="row in rows" :key="row.key" class="flex px-3 py-2">
        <SearchRow :row="row" />
      </div>
    </div>
  </KitSection>
</template>
