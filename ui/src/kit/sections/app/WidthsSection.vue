<script setup lang="ts">
import type {
  ChallengeRow,
  CollectionItem,
  RunView,
  UnlockNode,
} from '@/lib/ipc/types'
import { ItemKindView, OriginView } from '@/lib/ipc/types'
import ChallengesTable from '@/screens/challenges/ChallengesTable.vue'
import CollectionTable from '@/screens/collection/CollectionTable.vue'
import RunsTable from '@/screens/runs/RunsTable.vue'
import UnlockTable from '@/screens/unlock/UnlockTable.vue'
import KitSection from '../../KitSection.vue'
import KitWidths from '../../KitWidths.vue'

// One fixture per folding table — the four pairs listed in `lib/design/tables.ts`. Each carries
// a row whose name is long on purpose: at 500 the name column is what is left after the fixed
// tracks, and truncation is the thing to look at.

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
const queued = new Set<number>()

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
]

const runs: RunView[] = [
  {
    source: { kind: 'live' },
    ordinal: 1,
    character: 'Isaac',
    characterId: 0,
    seedWords: 'ABCD 1234',
    online: false,
    outcome: { kind: 'won', ending: 'Mother' },
    floors: 11,
    startingItems: [],
    collected: [],
    heldActive: null,
    achievements: [],
  },
  {
    source: { kind: 'session', name: '2026-09-22' },
    ordinal: 2,
    character: null,
    characterId: null,
    seedWords: 'WXYZ 9876',
    online: true,
    outcome: { kind: 'died', killer: 'Mom' },
    floors: 4,
    startingItems: [],
    collected: [],
    heldActive: null,
    achievements: [],
  },
]
</script>

<template>
  <KitSection title="Larghezze" class="col-span-3">
    <div class="flex flex-col gap-6">
      <KitWidths>
        <UnlockTable
          :nodes="nodes"
          :queued="queued"
          :can-write="false"
          :busy="false"
          :offset="null"
        />
      </KitWidths>
      <KitWidths>
        <CollectionTable
          :items="items"
          :offset="null"
          find-query=""
          :find-current="null"
        />
      </KitWidths>
      <KitWidths>
        <ChallengesTable
          :rows="challenges"
          :queued="[]"
          :can-write="false"
          :busy="false"
        />
      </KitWidths>
      <KitWidths>
        <RunsTable :runs="runs" :selected="null" :offset="null" />
      </KitWidths>
    </div>
  </KitSection>
</template>
