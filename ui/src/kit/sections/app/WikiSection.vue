<script setup lang="ts">
import WikiBlocks from '@/components/wiki/WikiBlocks.vue'
import WikiInline from '@/components/wiki/WikiInline.vue'
import type { Block, Inline, Target, UnlockNode } from '@/lib/ipc/types'
import { Dlc, Style } from '@/lib/ipc/types'
import KitSection from '../../KitSection.vue'

const d6: Target = { kind: 'item', id: 105 }
const plain = (text: string): Inline => ({
  kind: 'text',
  text,
  style: Style.Plain,
})

const natures: Inline[][] = [
  [
    plain('testo semplice '),
    { kind: 'text', text: 'e in grassetto', style: Style.Bold },
  ],
  [{ kind: 'ref', target: d6, label: 'The D6' }],
  [{ kind: 'ref', target: { kind: 'concept', name: 'Chest' }, label: 'Chest' }],
  [{ kind: 'concept', page: 'Devil Room', label: 'Devil Room' }],
  [{ kind: 'edition', only: [Dlc.Repentance], inline: [] }],
]

const paragraph: Inline[] = [
  plain("Al momento dell'uso, ripiazza tutti gli oggetti nella stanza. "),
  { kind: 'ref', target: d6, label: 'The D6' },
  plain(' si ricarica in '),
  { kind: 'text', text: '6 stanze', style: Style.Bold },
  plain('. Nella '),
  { kind: 'concept', page: 'Devil Room', label: 'Devil Room' },
  plain(" l'effetto è identico. "),
  {
    kind: 'edition',
    only: [Dlc.RepentancePlus, Dlc.Repentance],
    inline: [plain(' cambia il conteggio.')],
  },
]

const blocks: Block[] = [
  { kind: 'heading', level: 3, inline: [plain('Note')] },
  { kind: 'paragraph', inline: paragraph },
  {
    kind: 'list',
    ordered: false,
    items: [
      {
        inline: [plain('Non ripiazza gli oggetti del negozio.')],
        children: [],
      },
      {
        inline: [
          { kind: 'ref', target: d6, label: 'The D6' },
          plain(' con Car Battery: due lanci.'),
        ],
        children: [],
      },
    ],
  },
  // A list of names, as `{{achievement text}}` becomes: a picture per row, the name beside it
  // and how it is unlocked under the name.
  { kind: 'heading', level: 3, inline: [plain('Sbloccabile')] },
  {
    kind: 'list',
    ordered: false,
    items: [
      {
        inline: [
          {
            kind: 'ref',
            target: { kind: 'achievement', id: 3 },
            label: 'Judas',
          },
        ],
        children: [
          {
            kind: 'paragraph',
            inline: [
              plain('Defeat '),
              {
                kind: 'ref',
                target: { kind: 'entity', id: 84, variant: 0, subtype: 0 },
                label: 'Satan',
              },
            ],
          },
        ],
      },
      {
        inline: [
          {
            kind: 'ref',
            target: { kind: 'achievement', id: 1 },
            label: 'I RULE!',
          },
        ],
        children: [
          {
            kind: 'paragraph',
            inline: [
              {
                kind: 'edition',
                only: [Dlc.Afterbirth],
                inline: [
                  plain(' Defeat ??? or The Lamb, then Satan as Isaac.'),
                ],
              },
              plain(' '),
              {
                kind: 'edition',
                only: [Dlc.Repentance, Dlc.RepentancePlus],
                inline: [plain(' Defeat Mega Satan and unlock The Negative.')],
              },
            ],
          },
        ],
      },
      {
        inline: [
          {
            kind: 'ref',
            target: { kind: 'achievement', id: 9 },
            label: 'The Negative',
          },
        ],
        children: [],
      },
    ],
  },
  {
    kind: 'list',
    ordered: false,
    items: [
      { inline: [{ kind: 'ref', target: d6, label: 'The D6' }], children: [] },
      {
        inline: [
          {
            kind: 'edition',
            only: [Dlc.Repentance],
            inline: [
              {
                kind: 'ref',
                target: { kind: 'item', id: 25 },
                label: 'Breakfast',
              },
            ],
          },
        ],
        children: [],
      },
    ],
  },
  {
    kind: 'table',
    header: [[plain('Edizione')], [plain('Carica')]],
    rows: [
      [[plain('Normale')], [plain('6')]],
      [
        [{ kind: 'edition', only: [Dlc.RepentancePlus], inline: [] }],
        [plain('4')],
      ],
    ],
  },
]

// No reference carries a sprite here: the app cuts them from the user's own copy of the game
// at runtime, and the Kit page has no copy to cut from.
const iconFor = (): string | null => null

// What a profile says about the three achievements of the list of names: one done, one that can
// be had now, one with something in the way — the three states the badge draws beside a name.
const stateNode = (
  id: number,
  done: boolean,
  availableNow: boolean,
): UnlockNode => ({
  achievement: { kind: 'known', id, text: '', condition: null, iconUrl: null },
  done,
  unlocks: [],
  origin: null,
  missing: [],
  graph: {
    kind: 'computed',
    availableNow,
    blockedBy: availableNow ? 0 : 2,
    fanOut: 0,
    stepsMissing: 0,
  },
})
const nodes = new Map([
  [3, stateNode(3, true, true)],
  [1, stateNode(1, false, true)],
  [9, stateNode(9, false, false)],
])
const nodeFor = (target: Target): UnlockNode | null =>
  target.kind === 'achievement' ? (nodes.get(target.id) ?? null) : null

// Which references lead to a page: items do, a concept has no id to open one with.
const canOpen = (target: Target): boolean => target.kind !== 'concept'
</script>

<template>
  <KitSection title="Wiki" class="col-span-2">
    <div class="flex flex-col gap-2 text-row">
      <p v-for="(line, i) in natures" :key="i">
        <WikiInline :inline="line" :can-open="canOpen" />
      </p>
    </div>
    <p class="text-row">
      <WikiInline :inline="paragraph" :can-open="canOpen" />
    </p>
    <WikiBlocks
      :blocks="blocks"
      :icon-for="iconFor"
      :can-open="canOpen"
      :node-for="nodeFor"
    />
  </KitSection>
</template>
