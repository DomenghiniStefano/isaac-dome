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
import { SearchDiagnostic } from '@/lib/ipc/types'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'

type Message = MessageKey<MessageSchema>

const props = defineProps<{ diagnostics: SearchDiagnostic[] }>()
const { t } = useMessages()

const title: Record<SearchDiagnostic, Message> = {
  [SearchDiagnostic.NoProfile]: 'search.diagnostics.noProfileTitle',
  [SearchDiagnostic.NoCatalog]: 'search.diagnostics.noCatalogTitle',
  [SearchDiagnostic.NoWiki]: 'search.diagnostics.noWikiTitle',
  [SearchDiagnostic.NoAchievementSection]:
    'search.diagnostics.noAchievementSectionTitle',
  [SearchDiagnostic.NoCollectionSection]:
    'search.diagnostics.noCollectionSectionTitle',
}

const body: Record<SearchDiagnostic, Message> = {
  [SearchDiagnostic.NoProfile]: 'search.diagnostics.noProfile',
  [SearchDiagnostic.NoCatalog]: 'search.diagnostics.noCatalog',
  [SearchDiagnostic.NoWiki]: 'search.diagnostics.noWiki',
  [SearchDiagnostic.NoAchievementSection]:
    'search.diagnostics.noAchievementSection',
  [SearchDiagnostic.NoCollectionSection]:
    'search.diagnostics.noCollectionSection',
}

// `noProfile` is a note, not a warning: it is the expected state before a save is chosen
// (spec 3.5, Decision 8). A section that didn't read is the one thing worth an alarm.
const isWarning = (d: SearchDiagnostic): boolean => {
  switch (d) {
    case SearchDiagnostic.NoAchievementSection:
    case SearchDiagnostic.NoCollectionSection:
      return true
    case SearchDiagnostic.NoProfile:
    case SearchDiagnostic.NoCatalog:
    case SearchDiagnostic.NoWiki:
      return false
    default:
      return assertNever(d)
  }
}

const rows = computed(() =>
  props.diagnostics.map((d) => ({ kind: d, warning: isWarning(d) })),
)
</script>

<template>
  <Alert
    v-for="row in rows"
    :key="row.kind"
    :variant="row.warning ? AlertVariant.Destructive : undefined"
  >
    <TriangleAlertIcon v-if="row.warning" />
    <InfoIcon v-else />
    <AlertTitle>{{ t(title[row.kind]) }}</AlertTitle>
    <AlertDescription>{{ t(body[row.kind]) }}</AlertDescription>
  </Alert>
</template>
