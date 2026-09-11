<script setup lang="ts">
import { InfoIcon, TriangleAlertIcon } from '@lucide/vue'
import { computed } from 'vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import type { UnlockDiagnostic } from '@/lib/ipc/types'

const props = defineProps<{ diagnostics: UnlockDiagnostic[] }>()
const { t } = useMessages()

const has = (kind: UnlockDiagnostic['kind']) =>
  props.diagnostics.some((d) => d.kind === kind)

// The two that compare the save with the catalog coexist with real rows: a line, not an alert.
const note = (d: UnlockDiagnostic): string | null => {
  switch (d.kind) {
    case 'slotsBeyondCatalog':
      return `${d.count} ${t('unlock.diagnostics.slotsBeyondCatalog')}`
    case 'catalogBeyondSlots':
      return `${d.count} ${t('unlock.diagnostics.catalogBeyondSlots')}`
    case 'noCatalog':
    case 'noAchievementSection':
      return null
    default:
      return assertNever(d)
  }
}

const notes = computed(() =>
  props.diagnostics.flatMap((d) => {
    const line = note(d)
    return line ? [line] : []
  }),
)
</script>

<template>
  <!-- DESIGN-BRIEF.md §7.2: no catalog and an unread section 1 are states of the screen; the
       second says in words that zero rows is not zero achievements done. -->
  <Alert v-if="has('noCatalog')">
    <InfoIcon />
    <AlertTitle>{{ t('unlock.diagnostics.noCatalogTitle') }}</AlertTitle>
    <AlertDescription>{{ t('unlock.diagnostics.noCatalog') }}</AlertDescription>
  </Alert>
  <Alert v-if="has('noAchievementSection')" :variant="AlertVariant.Destructive">
    <TriangleAlertIcon />
    <AlertTitle>{{
      t('unlock.diagnostics.noAchievementSectionTitle')
    }}</AlertTitle>
    <AlertDescription>{{
      t('unlock.diagnostics.noAchievementSection')
    }}</AlertDescription>
  </Alert>
  <p
    v-for="line in notes"
    :key="line"
    class="text-caption text-subtle-foreground"
  >
    {{ line }}
  </p>
</template>
