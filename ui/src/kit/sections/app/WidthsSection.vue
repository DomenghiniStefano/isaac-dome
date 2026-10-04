<script setup lang="ts">
import type {
  ChallengeRow,
  CollectionItem,
  RunView,
  UnlockNode,
} from '@/lib/ipc/types'
import { ItemKindView, OriginView } from '@/lib/ipc/types'
import { runsAnswer } from '@/lib/ipc/fixtures/runs'
import ChallengesTable from '@/screens/challenges/ChallengesTable.vue'
import CollectionTable from '@/screens/collection/CollectionTable.vue'
import RunsTable from '@/screens/runs/RunsTable.vue'
import UnlockTable from '@/screens/unlock/UnlockTable.vue'
import KitSection from '../../KitSection.vue'
import KitWidths from '../../KitWidths.vue'

// One fixture per list table. Each carries a row whose name is long on purpose: at 500 the name column is what is left after the fixed
// tracks, and truncation is the thing to look at. The three widths moved into `KitWidths.vue`
// when a second table needed them.

const node = (id: number, text: string, done: boolean): UnlockNode => ({
  achievement: {
    kind: 'known',
    id,
    text,
    condition: 'Sconfiggi Mom con Isaac',
    iconUrl: null,
  },
  done,
  unlocks: [],
  origin: null,
  missing: [],
  graph: {
    kind: 'computed',
    availableNow: !done,
    blockedBy: 0,
    fanOut: 3,
    stepsMissing: 0,
  },
})

const nodes = [
  node(1, 'Il Cubo di Ghiaccio', false),
  node(2, 'La Sacca del Diavolo', true),
  node(
    3,
    'Un nome lungo che deve troncarsi quando la colonna si stringe',
    false,
  ),
]

const items: CollectionItem[] = [
  {
    id: 1,
    kind: ItemKindView.Passive,
    name: 'Il Cubo di Ghiaccio',
    iconUrl: null,
    quality: 3,
    pools: ['Tesoro', 'Negozio'],
    origin: OriginView.Repentance,
    inCollection: true,
    lock: { kind: 'free' },
  },
  {
    id: 2,
    kind: ItemKindView.Active,
    name: 'Un nome lungo che deve troncarsi quando la colonna si stringe',
    iconUrl: null,
    quality: 0,
    pools: [],
    origin: null,
    inCollection: false,
    lock: { kind: 'free' },
  },
]

const challenges: ChallengeRow[] = [
  {
    number: 1,
    name: 'Pitch Black',
    state: { kind: 'done' },
    rewards: [],
    character: null,
    characterName: 'Isaac',
    goal: null,
    blindfolded: false,
    page: null,
  },
  {
    number: 2,
    name: 'Una sfida dal nome lungo che deve troncarsi quando si stringe',
    state: { kind: 'available' },
    rewards: [],
    character: null,
    characterName: null,
    goal: null,
    blindfolded: null,
    page: null,
  },
  // The longest state a challenge can say, in the language that says it longest: the pill
  // goes onto two lines inside its column, never into the next one.
  {
    number: 3,
    name: 'Darkness Falls',
    state: {
      kind: 'blocked',
      missing: Array.from({ length: 12 }, (_, i) => 60 + i),
    },
    rewards: [],
    character: null,
    characterName: 'Eve',
    goal: null,
    blindfolded: false,
    page: null,
  },
]

// The shared fixtures: every case the list has to draw, from a death by a projectile to a launch
// read before the app kept dates.
const runs: RunView[] = runsAnswer().runs
</script>

<template>
  <KitSection title="Larghezze" class="col-span-3">
    <div class="flex flex-col gap-6">
      <KitWidths>
        <UnlockTable :nodes="nodes" />
      </KitWidths>
      <KitWidths>
        <CollectionTable :items="items" find-query="" :find-current="null" />
      </KitWidths>
      <KitWidths>
        <ChallengesTable :rows="challenges" />
      </KitWidths>
      <KitWidths>
        <RunsTable :runs="runs" :selected="null" :offset="null" />
      </KitWidths>
    </div>
  </KitSection>
</template>
