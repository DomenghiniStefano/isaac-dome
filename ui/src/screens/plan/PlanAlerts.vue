<script setup lang="ts">
import { DatabaseIcon, InfoIcon, TriangleAlertIcon } from '@lucide/vue'
import { computed } from 'vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import { Button, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import { storeReasonPart } from '@/lib/ipc/errorText'
import type { QueueDiagnostic, StoreReason } from '@/lib/ipc/types'

const props = defineProps<{ diagnostics: QueueDiagnostic[]; busy: boolean }>()
const emit = defineEmits<{ importGoals: [] }>()
const { t } = useMessages()

// The states that stand in for the queue or ask for an action. Completed and unresolved rows
// are said under the rows themselves.
const isAlert = (d: QueueDiagnostic): boolean => {
  switch (d.kind) {
    case 'storeUnavailable':
    case 'unreadable':
    case 'noCatalog':
    case 'goalsPending':
      return true
    case 'completed':
    case 'unresolved':
      return false
    default:
      return assertNever(d)
  }
}

const alerts = computed(() => props.diagnostics.filter(isAlert))

// The reason is a variant, so the sentence is built here from a key and its values —
// "a newer version, 7 against 1" is the translation's word order, not Rust's.
const reasonText = (reason: StoreReason): string => {
  const part = storeReasonPart(reason)
  return t(part.key, part.params)
}
</script>

<template>
  <template v-for="d in alerts" :key="d.kind">
    <Alert
      v-if="d.kind === 'storeUnavailable'"
      :variant="AlertVariant.Destructive"
    >
      <TriangleAlertIcon />
      <AlertTitle>{{ t('plan.alerts.storeUnavailableTitle') }}</AlertTitle>
      <AlertDescription
        >{{ t('ipcErrors.storeUnavailable') }}
        {{ reasonText(d.reason) }}</AlertDescription
      >
    </Alert>
    <Alert v-else-if="d.kind === 'unreadable'">
      <DatabaseIcon />
      <AlertTitle>{{ t('plan.alerts.unreadableTitle') }}</AlertTitle>
      <AlertDescription>{{ t('plan.alerts.unreadable') }}</AlertDescription>
    </Alert>
    <Alert v-else-if="d.kind === 'noCatalog'">
      <InfoIcon />
      <AlertTitle>{{ t('plan.alerts.noCatalogTitle') }}</AlertTitle>
      <AlertDescription>{{ t('plan.alerts.noCatalog') }}</AlertDescription>
    </Alert>
    <Alert v-else-if="d.kind === 'goalsPending'">
      <InfoIcon />
      <AlertTitle class="tabular-nums"
        >{{ t('plan.alerts.goalsPendingTitle') }}: {{ d.count }}</AlertTitle
      >
      <AlertDescription class="flex flex-col items-start gap-2">
        {{ t('plan.alerts.goalsPending') }}
        <Button
          :variant="ButtonVariant.Outline"
          :disabled="busy"
          @click="emit('importGoals')"
          >{{ t('plan.alerts.import') }}</Button
        >
      </AlertDescription>
    </Alert>
  </template>
</template>
