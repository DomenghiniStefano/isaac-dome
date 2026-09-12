<script setup lang="ts">
import AchievementArt from '@/components/graph/AchievementArt.vue'
import { ArtSize } from '@/components/graph/artSize'
import KpiTile from '@/components/kpi/KpiTile.vue'
import MarkCell from '@/components/marks/MarkCell.vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import type { Cell } from '@/lib/ipc/types'

const { t } = useMessages()

// The app's own components, not a picture of them: the preview is at the chosen size
// because everything is, which is the whole point of the mechanism. A mark taken on both
// levels, with no art: the fallback outfit is what a machine without the game draws.
const cell: Cell = { kind: 'known', bits: 3 }
</script>

<template>
  <!-- Pinned at the top, like Discord's: the slider and what it does have to be on screen
       together, so the page scrolls under this. -->
  <Card class="sticky top-0 z-10">
    <CardHeader class="flex-wrap">
      <CardTitle>{{ t('appearance.preview') }}</CardTitle>
      <span class="text-caption text-subtle-foreground">{{
        t('appearance.previewHint')
      }}</span>
    </CardHeader>
    <CardContent class="flex flex-wrap items-center gap-4">
      <KpiTile
        :value="381"
        :denominator="642"
        :label="t('appearance.sample.kpi')"
      />
      <MarkCell :cell="cell" :art="null" />
      <div class="flex items-center gap-2.5">
        <AchievementArt :url="null" :size="ArtSize.Thumb" />
        <span class="flex flex-col">
          <span class="text-row text-foreground">{{
            t('appearance.sample.item')
          }}</span>
          <span class="text-micro text-faint-foreground">{{
            t('appearance.sample.itemHint')
          }}</span>
        </span>
      </div>
      <Badge :variant="BadgeVariant.Now">{{
        t('appearance.sample.badge')
      }}</Badge>
      <Button :variant="ButtonVariant.Outline">{{
        t('appearance.sample.button')
      }}</Button>
    </CardContent>
  </Card>
</template>
