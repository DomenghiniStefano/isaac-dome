<script setup lang="ts">
import WikiBlocks from '@/components/wiki/WikiBlocks.vue'
import WikiInline from '@/components/wiki/WikiInline.vue'
import type { Block, Inline, Target } from '@/lib/ipc/types'
import { Dlc, Style } from '@/lib/ipc/types'
import KitSection from '../../KitSection.vue'
import { kitMarkArt } from '../../markArt'

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
  [{ kind: 'ref', target: { kind: 'pickup', name: 'Chest' }, label: 'Chest' }],
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

// A stand-in for cycle 3's icon protocol: items get a sprite, everything else none, so a
// reference with an icon and one without sit side by side.
const iconFor = (target: Target): string | null =>
  target.kind === 'item' ? (kitMarkArt.heart?.normal ?? null) : null
</script>

<template>
  <KitSection title="Wiki" class="col-span-2">
    <div class="flex flex-col gap-2 text-row">
      <p v-for="(line, i) in natures" :key="i">
        <WikiInline :inline="line" :icon-for="iconFor" />
      </p>
    </div>
    <p class="text-row">
      <WikiInline :inline="paragraph" :icon-for="iconFor" />
    </p>
    <WikiBlocks :blocks="blocks" :icon-for="iconFor" />
  </KitSection>
</template>
