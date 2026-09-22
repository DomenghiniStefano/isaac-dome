<script setup lang="ts">
import GoalRow from '@/components/plan/GoalRow.vue'
import { Card } from '@/components/ui/card'
import KitSection from '@/kit/KitSection.vue'
import { NodeState } from '@/lib/graph/nodeState'
import type { UnlockNode } from '@/lib/ipc/types'
import type { QueueExtras } from '@/lib/plan/queueExtras'
import type { RowModel } from '@/lib/plan/rowModel'

// Fixtures, not data: the row draws a model, so the Kit hands it models directly and needs
// no profile, no catalog and no graph. The node is only here for the state badge's menu.
const node = {
  achievement: { kind: 'unknown', slot: 1 },
  done: false,
  unlocks: [],
  origin: null,
  missing: [],
  graph: {
    kind: 'computed',
    availableNow: true,
    blockedBy: 0,
    fanOut: 7,
    stepsMissing: 0,
  },
} as UnlockNode

const model = (over: Partial<RowModel> = {}): RowModel => ({
  text: 'Cuore di Isaac',
  condition: 'Sconfiggi Mom’s Heart 10 volte',
  art: null,
  fanOut: 7,
  playable: true,
  state: NodeState.Now,
  location: null,
  ...over,
})

const extras = (over: Partial<QueueExtras> = {}): QueueExtras => ({
  wanted: false,
  serves: [],
  stepsNotQueued: 0,
  ...over,
})

// The five that can be got wrong, and none of them has a unit test: presentation is looked
// at here or nowhere.
const rows: {
  label: string
  model: RowModel
  extras?: QueueExtras
  position?: number
}[] = [
  {
    label: 'coda · giocabile ora',
    model: model(),
    extras: extras(),
    position: 1,
  },
  {
    label: 'coda · bloccata, due passi fuori',
    model: model({ playable: false, state: NodeState.Blocked }),
    extras: extras({ stepsNotQueued: 2 }),
    position: 2,
  },
  {
    label: 'coda · chiesta, serve a due (uno uscito dalla coda)',
    model: model({ text: 'Polaroid' }),
    extras: extras({
      wanted: true,
      serves: [
        { id: 1, text: 'Il Negativo' },
        { id: 2, text: null },
      ],
    }),
    position: 3,
  },
  {
    label: 'coda · il file non dice come si sblocca',
    model: model({ condition: null }),
    extras: extras(),
    position: 4,
  },
  {
    label: 'consiglio · nessun grip, nessuna posizione, il più al suo posto',
    model: model({ text: 'Fascia di Mamma', fanOut: 12 }),
  },
]
</script>

<template>
  <KitSection title="GoalRow">
    <div v-for="row in rows" :key="row.label" class="flex flex-col gap-1">
      <span class="text-label text-subtle-foreground">{{ row.label }}</span>
      <Card class="flex-col gap-0 p-0">
        <GoalRow
          :model="row.model"
          :node="node"
          :extras="row.extras"
          :position="row.position"
          :busy="false"
        />
      </Card>
    </div>
  </KitSection>
</template>
