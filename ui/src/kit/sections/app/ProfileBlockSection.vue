<script setup lang="ts">
import ProfileBlock from '@/components/graph/ProfileBlock.vue'
import type { UnlockNode } from '@/lib/ipc/types'
import KitSection from '../../KitSection.vue'

// Four states the block has to hold, written here rather than taken from a fixture: the Kit
// is where a state that a real profile may not currently be in can still be looked at — and
// the young-profile measurement of 2026-09-13 is exactly that kind of state.
const base: UnlockNode = {
  achievement: {
    kind: 'known',
    id: 484,
    text: 'You unlocked "The Lost"',
    condition: 'Arriva a Home e usa la Red Key',
    iconUrl: null,
  },
  done: false,
  unlocks: [
    {
      kind: 'character',
      id: 31,
      name: 'The Lost',
      tainted: true,
      page: { kind: 'character', id: 31 },
    },
  ],
  origin: null,
  missing: [],
  graph: {
    kind: 'computed',
    availableNow: true,
    blockedBy: 0,
    fanOut: 23,
    stepsMissing: 0,
  },
}

const blocked: UnlockNode = {
  ...base,
  missing: [
    {
      kind: 'character',
      id: 10,
      name: 'The Lost',
      tainted: false,
      page: { kind: 'character', id: 10 },
    },
    { kind: 'boss', id: 3, name: 'Nameless', page: null },
    { kind: 'counter', label: 'Hush', current: 0, atLeast: 1 },
  ],
  graph: {
    kind: 'computed',
    availableNow: false,
    blockedBy: 3,
    fanOut: 23,
    stepsMissing: 7,
  },
}

const done: UnlockNode = { ...base, done: true }

const leaf: UnlockNode = {
  ...base,
  unlocks: [],
  graph: {
    kind: 'computed',
    availableNow: true,
    blockedBy: 0,
    fanOut: 0,
    stepsMissing: 0,
  },
}
</script>

<template>
  <KitSection title="Il tuo profilo (pagina achievement)">
    <ProfileBlock :node="base" :queued="false" :can-add="true" :busy="false" />
    <ProfileBlock
      :node="blocked"
      :queued="false"
      :can-add="false"
      :busy="false"
    />
    <ProfileBlock :node="done" :queued="false" :can-add="false" :busy="false" />
    <ProfileBlock :node="leaf" :queued="true" :can-add="false" :busy="false" />
  </KitSection>
</template>
