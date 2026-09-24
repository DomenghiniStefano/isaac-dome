<script setup lang="ts">
import { WantDiagnostic } from '@/lib/ipc/types'
import type { UnlockNode } from '@/lib/ipc/types'
import { WantBlockKind } from '@/lib/graph/wantBlocks'
import type { WantBlock } from '@/lib/graph/wantBlocks'
import WantAnswer from '@/screens/goals/WantAnswer.vue'
import KitSection from '../../KitSection.vue'

// The four states a want's answer can be in, written here rather than taken from a fixture:
// three of them are states a given profile is not in today, and they still have to be looked
// at. Measured 2026-09-13: on an advanced profile the deepest chain is one step, so `chain`
// with two steps is a state the live data would never show.
const node = (id: number, text: string, done = false): UnlockNode => ({
  achievement: { kind: 'known', id, text, condition: null, iconUrl: null },
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

const want = node(509, 'You unlocked "Greedier!"')

const block = (
  kind: WantBlockKind,
  extra: Partial<WantBlock> = {},
): WantBlock => ({
  kind,
  achievement: 509,
  node: want,
  steps: [],
  unknown: 0,
  queueable: false,
  ...extra,
})

const chain = block(WantBlockKind.Chain, {
  steps: [
    node(1, 'You unlocked "Magdalene"'),
    node(2, 'You unlocked "The Polaroid"'),
  ],
  queueable: true,
})
const unreadable = block(WantBlockKind.Chain, {
  steps: [node(1, 'You unlocked "Magdalene"')],
  unknown: 2,
  queueable: true,
})
const now = block(WantBlockKind.AvailableNow, { queueable: true })
const done = block(WantBlockKind.Done, { node: node(509, 'Greedier!', true) })
const noProfile = block(WantBlockKind.NoProfile)
</script>

<template>
  <KitSection title="Voglio… (risposta)">
    <WantAnswer
      :blocks="[chain]"
      :banner="null"
      :can-write="true"
      :busy="false"
    />
    <WantAnswer
      :blocks="[unreadable]"
      :banner="null"
      :can-write="true"
      :busy="false"
    />
    <WantAnswer
      :blocks="[now]"
      :banner="null"
      :can-write="true"
      :busy="false"
    />
    <WantAnswer
      :blocks="[done]"
      :banner="null"
      :can-write="false"
      :busy="false"
    />
    <WantAnswer
      :blocks="[noProfile]"
      :banner="WantDiagnostic.NoProfile"
      :can-write="false"
      :busy="false"
    />
    <!-- Two ways in: 14 of the 45 challenges are named by two achievements. -->
    <WantAnswer
      :blocks="[chain, now]"
      :banner="null"
      :can-write="true"
      :busy="false"
    />
    <WantAnswer
      :blocks="[]"
      :banner="WantDiagnostic.NothingUnlocks"
      :can-write="false"
      :busy="false"
    />
  </KitSection>
</template>
