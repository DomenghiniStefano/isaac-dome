<script setup lang="ts">
import { RefreshCwIcon, TriangleAlertIcon } from '@lucide/vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import { Button, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import type { MissingReason, SetupDiagnostic } from '@/lib/ipc/types'
import { missingReasonLabel } from '@/lib/profile/profileLabels'

defineProps<{ reason: MissingReason; diagnostics: SetupDiagnostic[] }>()
const emit = defineEmits<{ retry: [] }>()
const { t } = useMessages()

const diagnosticText = (d: SetupDiagnostic): string => {
  switch (d.kind) {
    case 'steamNotFound':
      return t('profile.diagnostics.steamNotFound')
    case 'gameNotFound':
      return t('profile.diagnostics.gameNotFound')
    case 'noSavesFound':
      return t('profile.diagnostics.noSavesFound')
    case 'unreadablePath':
      return `${t('profile.diagnostics.unreadablePath')} · ${d.name} · ${d.reason}`
    case 'malformedManifest':
      return `${t('profile.diagnostics.malformedManifest')} · ${d.name}`
    default:
      return assertNever(d)
  }
}
</script>

<template>
  <!-- The chain broke: say where, and what we tried. Choosing a folder by hand needs a
       command that accepts a path; until then there is no button that does nothing. -->
  <div class="flex flex-col gap-3">
    <Alert :variant="AlertVariant.Destructive">
      <TriangleAlertIcon />
      <AlertTitle>{{ t('profile.none.title') }}</AlertTitle>
      <AlertDescription>{{ t(missingReasonLabel[reason]) }}</AlertDescription>
    </Alert>
    <div class="flex flex-col gap-3 border border-border bg-sheet p-3">
      <div>
        <Button :variant="ButtonVariant.Outline" @click="emit('retry')">
          <RefreshCwIcon />{{ t('profile.none.retry') }}
        </Button>
      </div>
      <div
        v-if="diagnostics.length"
        class="flex flex-col gap-1 border border-hairline bg-data px-3 py-2.5"
      >
        <span class="text-label text-subtle-foreground">{{
          t('profile.none.diagnostics')
        }}</span>
        <span
          v-for="(d, i) in diagnostics"
          :key="i"
          class="text-caption text-foreground-soft"
          >{{ diagnosticText(d) }}</span
        >
      </div>
    </div>
  </div>
</template>
