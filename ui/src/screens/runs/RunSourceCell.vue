<script setup lang="ts">
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { useMessages } from '@/i18n'
import type { RunView } from '@/lib/ipc/types'
import { sourceText } from '@/lib/runs/runLabels'

// Which log said so, and whether the run was played online.
defineProps<{ run: RunView }>()
const { t } = useMessages()
</script>

<template>
  <span class="flex min-w-0 items-center gap-2">
    <span class="truncate text-label text-subtle-foreground">{{
      t(sourceText[run.source.kind])
    }}</span>
    <Badge v-if="run.online" :variant="BadgeVariant.Tag">{{
      t('runs.online.online')
    }}</Badge>
  </span>
</template>
