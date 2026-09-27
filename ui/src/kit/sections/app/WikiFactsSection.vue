<script setup lang="ts">
import FactChips from '@/components/wiki/FactChips.vue'
import ProgressBadge from '@/components/wiki/ProgressBadge.vue'
import type { PageFacts, PageProgress } from '@/lib/ipc/types'
import KitSection from '../../KitSection.vue'

// One fixture per `PageFacts` kind (design decision 3), showing what `factChips` reads out of
// it: no drawing, no game and no save here, so these are values written by hand rather than
// read from an entry — the pure function is what this section exercises.
const facts: Array<[string, PageFacts]> = [
  [
    'item (rated, activated, tagged)',
    {
      kind: 'item',
      quality: 4,
      activated: true,
      recharge: '6 rooms',
      shopPrice: '15 coins',
      devilPrice: '2 hearts',
      tags: ['guppy', 'syringe'],
    },
  ],
  ['trinket', { kind: 'trinket', tags: ['spider'] }],
  [
    'achievement',
    { kind: 'achievement', requirement: 'Defeat Mom', unlocks: null },
  ],
  ['boss', { kind: 'boss', baseHp: 300, floors: 'Womb, Utero' }],
  [
    'challenge',
    {
      kind: 'challenge',
      character: { kind: 'character', id: 0 },
      goal: 'Beat the game with only starting items',
      blindfolded: true,
      curse: 'Curse of the Labyrinth',
    },
  ],
  [
    'character (tainted)',
    {
      kind: 'character',
      health: '1 red heart',
      damage: '3.5',
      tears: '2.73',
      range: '13.5',
      speed: '1.0',
      luck: '0',
      shotSpeed: '1.0',
      tainted: true,
    },
  ],
  ['transformation', { kind: 'transformation', requires: 3, contributors: 8 }],
  ['entity', { kind: 'entity', baseHp: null, floors: '' }],
  [
    'article (version)',
    {
      kind: 'article',
      category: 'version',
      version: { number: '1.7.9', date: '2021-04-27' },
    },
  ],
]

// One fixture per `PageProgress` variant (design decision 6), plus the edge each one can be in:
// an achievement not yet done, an item collected but its unlock still locked, a character with
// marks in progress, every `ChallengeStateView`, and a bestiary entry's three tallies. `null`
// draws nothing — the same silence a page with no save chosen keeps.
const progress: Array<[string, PageProgress | null]> = [
  ['achievement — done', { kind: 'achievement', done: true }],
  ['achievement — not done', { kind: 'achievement', done: false }],
  [
    'item — collected, unlocked',
    { kind: 'item', collected: true, unlocked: true, unlockedBy: null },
  ],
  [
    'item — not collected, locked by achievement',
    { kind: 'item', collected: false, unlocked: false, unlockedBy: 12 },
  ],
  [
    'unlockable — unlocked',
    { kind: 'unlockable', unlocked: true, unlockedBy: 7 },
  ],
  [
    'unlockable — locked',
    { kind: 'unlockable', unlocked: false, unlockedBy: 7 },
  ],
  [
    'character — marks in progress',
    { kind: 'character', unlocked: true, marksDone: 3, marksTotal: 12 },
  ],
  [
    'character — no marks at all (vacuity)',
    { kind: 'character', unlocked: false, marksDone: 0, marksTotal: 0 },
  ],
  ['challenge — done', { kind: 'challenge', state: { kind: 'done' } }],
  [
    'challenge — available',
    { kind: 'challenge', state: { kind: 'available' } },
  ],
  [
    'challenge — blocked',
    { kind: 'challenge', state: { kind: 'blocked', missing: [1, 2] } },
  ],
  ['challenge — unknown', { kind: 'challenge', state: { kind: 'unknown' } }],
  [
    'bestiary — met, killed, killed you',
    { kind: 'bestiary', met: 4, killed: 2, killedYou: 1 },
  ],
  ['no save chosen', null],
]
</script>

<template>
  <KitSection title="FactChips, ProgressBadge" class="col-span-3">
    <div class="flex flex-col gap-4">
      <div class="flex flex-col gap-1.5">
        <span class="text-caption text-subtle-foreground">FactChips</span>
        <div class="flex flex-col gap-2">
          <div
            v-for="[label, f] in facts"
            :key="label"
            class="flex flex-col gap-1"
          >
            <span class="text-caption text-faint-foreground">{{ label }}</span>
            <FactChips :facts="f" />
          </div>
        </div>
      </div>

      <div class="flex flex-col gap-1.5">
        <span class="text-caption text-subtle-foreground">ProgressBadge</span>
        <div class="flex flex-col gap-2">
          <div
            v-for="[label, p] in progress"
            :key="label"
            class="flex flex-col gap-1"
          >
            <span class="text-caption text-faint-foreground">{{ label }}</span>
            <ProgressBadge :progress="p" />
          </div>
        </div>
      </div>
    </div>
  </KitSection>
</template>
