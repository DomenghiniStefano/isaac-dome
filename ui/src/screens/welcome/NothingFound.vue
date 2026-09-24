<script setup lang="ts">
import { FolderSearchIcon, RefreshCwIcon, TriangleAlertIcon } from '@lucide/vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import { Button, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { assertNever } from '@/lib/assertNever'
import { ioReasonKey } from '@/lib/ipc/errorText'
import type { MissingReason, SetupDiagnostic } from '@/lib/ipc/types'
import { missingReasonLabel } from '@/lib/profile/profileLabels'
import { useProfileStore } from '@/stores/profile'

defineProps<{ reason: MissingReason; diagnostics: SetupDiagnostic[] }>()
const emit = defineEmits<{ retry: [] }>()
const profile = useProfileStore()
const { t } = useMessages()

const diagnosticText = (d: SetupDiagnostic): string => {
  switch (d.kind) {
    case 'steamNotFound':
      return t('profile.diagnostics.steamNotFound')
    case 'gameNotFound':
      return t('profile.diagnostics.gameNotFound')
    case 'noSavesFound':
      return t('profile.diagnostics.noSavesFound')
    case 'noSavesInChosenFolder':
      return t('profile.diagnostics.noSavesInChosenFolder')
    case 'unreadablePath':
      return t('profile.diagnostics.unreadablePath', {
        name: d.name,
        reason: t(ioReasonKey(d.reason)),
      })
    case 'malformedManifest':
      return t('profile.diagnostics.malformedManifest', { name: d.name })
    default:
      return assertNever(d)
  }
}
</script>

<template>
  <!-- The chain broke: say where, what we tried, and offer the two folders by hand. Since
       3.8 the buttons work (B14) — before that there were none at all, because a button that
       does nothing is worse than no button. -->
  <div class="flex flex-col gap-3">
    <Alert :variant="AlertVariant.Destructive">
      <TriangleAlertIcon />
      <AlertTitle>{{ t('welcome.nothing.title') }}</AlertTitle>
      <AlertDescription>{{ t(missingReasonLabel[reason]) }}</AlertDescription>
    </Alert>
    <div class="flex flex-col gap-3 border border-border bg-sheet p-3">
      <div class="flex flex-wrap gap-2">
        <Button :variant="ButtonVariant.Outline" @click="emit('retry')">
          <RefreshCwIcon />{{ t('welcome.nothing.retry') }}
        </Button>
        <Button
          :variant="ButtonVariant.Outline"
          @click="profile.pickGameFolder()"
        >
          <FolderSearchIcon />{{ t('welcome.nothing.chooseGame') }}
        </Button>
        <Button
          :variant="ButtonVariant.Outline"
          @click="profile.pickSavesFolder()"
        >
          <FolderSearchIcon />{{ t('welcome.nothing.chooseSaves') }}
        </Button>
      </div>
      <div
        v-if="diagnostics.length"
        class="flex flex-col gap-1 border border-hairline bg-data px-3 py-2.5"
      >
        <span class="text-label text-subtle-foreground">{{
          t('welcome.nothing.diagnostics')
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
