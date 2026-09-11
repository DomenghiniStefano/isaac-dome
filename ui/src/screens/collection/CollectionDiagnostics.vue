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
import type { CollectionDiagnostic } from '@/lib/ipc/types'

const props = defineProps<{ diagnostics: CollectionDiagnostic[] }>()
const { t } = useMessages()

const has = (kind: CollectionDiagnostic['kind']) =>
  props.diagnostics.some((d) => d.kind === kind)

// A catalog newer than the save coexists with real rows: a line, not an alert.
const note = (d: CollectionDiagnostic): string | null => {
  switch (d.kind) {
    case 'itemsBeyondSlots':
      return `${d.count} ${t('collection.diagnostics.itemsBeyondSlots')}`
    case 'noCatalog':
    case 'noCollectionSection':
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
  <!-- An unread section is a state of the screen, and each alert says in words that unreadable
       is not "never found". -->
  <Alert v-if="has('noCatalog')">
    <InfoIcon />
    <AlertTitle>{{ t('collection.diagnostics.noCatalogTitle') }}</AlertTitle>
    <AlertDescription>{{
      t('collection.diagnostics.noCatalog')
    }}</AlertDescription>
  </Alert>
  <Alert v-if="has('noCollectionSection')" :variant="AlertVariant.Destructive">
    <TriangleAlertIcon />
    <AlertTitle>{{
      t('collection.diagnostics.noCollectionSectionTitle')
    }}</AlertTitle>
    <AlertDescription>{{
      t('collection.diagnostics.noCollectionSection')
    }}</AlertDescription>
  </Alert>
  <Alert v-if="has('noAchievementSection')" :variant="AlertVariant.Destructive">
    <TriangleAlertIcon />
    <AlertTitle>{{
      t('collection.diagnostics.noAchievementSectionTitle')
    }}</AlertTitle>
    <AlertDescription>{{
      t('collection.diagnostics.noAchievementSection')
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
