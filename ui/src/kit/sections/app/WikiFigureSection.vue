<script setup lang="ts">
import { Chip } from '@/components/ui/chip'
import EditionBadge from '@/components/wiki/EditionBadge.vue'
import { FigureSize } from '@/components/wiki/figureSize'
import QualityChip from '@/components/wiki/QualityChip.vue'
import WikiFigure from '@/components/wiki/WikiFigure.vue'
import { Dlc } from '@/lib/ipc/types'
import type { Target } from '@/lib/ipc/types'
import { toneOfCategory, toneOfEdition } from '@/lib/wiki/tone'
import { WikiCategory } from '@/router/routeTable'
import KitSection from '../../KitSection.vue'

// No drawing ships in the repo (constraint 3), so `url` is always `null` here, exactly what
// every machine without the game sees: this is the fallback path, on purpose — the category
// icon at every size, centred, never a hole.
const sizes = Object.values(FigureSize)

const targets: Array<[string, Target]> = [
  ['item (sprite)', { kind: 'item', id: 105 }],
  ['achievement (painting)', { kind: 'achievement', id: 3 }],
  ['character (portrait)', { kind: 'character', id: 0 }],
  ['stage (no category icon)', { kind: 'stage', name: 'Basement' }],
]

const editions = Object.values(Dlc)
const qualities = [-1, 0, 1, 2, 3, 4]
const categories = Object.values(WikiCategory)
</script>

<template>
  <KitSection
    title="WikiFigure, Chip, EditionBadge, QualityChip"
    class="col-span-3"
  >
    <div class="flex flex-col gap-4">
      <div
        v-for="[label, target] in targets"
        :key="label"
        class="flex flex-col gap-1.5"
      >
        <span class="text-caption text-subtle-foreground">{{ label }}</span>
        <div class="flex flex-wrap items-end gap-3">
          <WikiFigure
            v-for="size in sizes"
            :key="size"
            :target="target"
            :url="null"
            :size="size"
          />
        </div>
      </div>

      <div class="flex flex-col gap-1.5">
        <span class="text-caption text-subtle-foreground"
          >Chip — every edition tone</span
        >
        <div class="flex flex-wrap gap-2">
          <Chip v-for="dlc in editions" :key="dlc" :tone="toneOfEdition(dlc)">{{
            dlc
          }}</Chip>
        </div>
      </div>

      <div class="flex flex-col gap-1.5">
        <span class="text-caption text-subtle-foreground">EditionBadge</span>
        <div class="flex flex-wrap gap-4">
          <EditionBadge :dlc="[Dlc.Repentance]" />
          <EditionBadge :dlc="[Dlc.Rebirth, Dlc.Afterbirth]" />
          <EditionBadge :dlc="[]" />
        </div>
      </div>

      <div class="flex flex-col gap-1.5">
        <span class="text-caption text-subtle-foreground">QualityChip</span>
        <div class="flex flex-wrap gap-2">
          <QualityChip v-for="q in qualities" :key="q" :quality="q" />
          <QualityChip :quality="null" />
        </div>
      </div>

      <div class="flex flex-col gap-1.5">
        <span class="text-caption text-subtle-foreground"
          >Chip — every category tone</span
        >
        <div class="flex flex-wrap gap-2">
          <Chip
            v-for="category in categories"
            :key="category"
            :tone="toneOfCategory(category)"
            >{{ category }}</Chip
          >
        </div>
      </div>
    </div>
  </KitSection>
</template>
