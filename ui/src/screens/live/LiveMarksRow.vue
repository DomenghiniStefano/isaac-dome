<script setup lang="ts">
import MarkCell from '@/components/marks/MarkCell.vue'
import { markArtOf } from '@/components/marks/markVisual'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { useMessages } from '@/i18n'
import type { LiveMarkRow, MarkArtView } from '@/lib/ipc/types'

// One character's row of the completion matrix, drawn with the same cell the matrix uses.
// The screen puts it beside what the run could open, which is the only place in the app
// where "what I am playing" and "what I am missing" are the same question.
defineProps<{ row: LiveMarkRow; bosses: string[]; art: MarkArtView[] }>()
const { t } = useMessages()
</script>

<template>
  <div class="flex items-center gap-3">
    <PixelSprite
      :url="row.headUrl"
      placeholder
      class="size-icon-compact shrink-0"
    />
    <span class="w-32 shrink-0 truncate text-row">{{ row.character }}</span>
    <div class="flex flex-wrap items-center gap-1">
      <MarkCell
        v-for="(cell, i) in row.cells"
        :key="bosses[i] ?? i"
        :cell="cell"
        :art="markArtOf(art[i])"
        :label="bosses[i]"
      />
    </div>
    <span class="ml-auto shrink-0 text-label text-subtle-foreground">{{
      t('live.missing', { n: row.missing })
    }}</span>
  </div>
</template>
