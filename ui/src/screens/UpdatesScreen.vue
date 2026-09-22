<script setup lang="ts">
import { DownloadIcon, RefreshCwIcon, TriangleAlertIcon } from '@lucide/vue'
import {
  Alert,
  AlertDescription,
  AlertLive,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Field, FieldDescription, FieldLabel } from '@/components/ui/field'
import { Progress } from '@/components/ui/progress'
import { Switch } from '@/components/ui/switch'
import { HelpTip } from '@/components/ui/tooltip'
import { computed, onMounted, onUnmounted } from 'vue'
import { useMessages } from '@/i18n'
import { ipcErrorParts } from '@/lib/ipc/errorText'
import {
  canCheck,
  canInstall,
  phasePart,
  progressPercent,
  releaseNotes,
} from '@/lib/update/phase'
import { AppEvent, watchAppEvent } from '@/lib/window/appEvents'
import { useSettingsStore } from '@/stores/settings'
import { useUpdateStore } from '@/stores/update'
import ScreenHeader from './ScreenHeader.vue'

const settings = useSettingsStore()
const update = useUpdateStore()
const { t } = useMessages()

// Listened for here rather than in `App.vue`: a download tells every window at every whole
// percentage point, and a window with this screen closed has nothing to draw with it.
let stopWatching: (() => void) | undefined
onMounted(async () => {
  await update.read()
  stopWatching = await watchAppEvent(AppEvent.UpdateChanged, () => {
    void update.read()
  })
})
onUnmounted(() => stopWatching?.())

const phase = computed(() => phasePart(update.view))
const percent = computed(() => progressPercent(update.view))
const notes = computed(() => releaseNotes(update.view))
const failure = computed(() =>
  update.error === null ? null : ipcErrorParts(update.error),
)
</script>

<template>
  <div class="flex h-full flex-col gap-4 overflow-y-auto px-5.5 pt-5 pb-15">
    <ScreenHeader :icon="RefreshCwIcon" :title="t('routes.updates')">{{
      t('updates.intro')
    }}</ScreenHeader>
    <!-- Asking to install nothing is the only error this screen can raise, and it is a defect
         of ours rather than something that happened to the user. Everything they can really
         run into is a phase, said in the line below the switch. -->
    <Alert
      v-if="failure"
      :variant="AlertVariant.Destructive"
      :live="AlertLive.Assertive"
    >
      <TriangleAlertIcon />
      <AlertTitle>{{ t('updates.failedTitle') }}</AlertTitle>
      <AlertDescription>
        <span v-for="part in failure" :key="part.key">{{
          t(part.key, part.params)
        }}</span>
      </AlertDescription>
    </Alert>
    <Alert
      v-else-if="settings.saveFailed"
      :variant="AlertVariant.Destructive"
      :live="AlertLive.Assertive"
    >
      <TriangleAlertIcon />
      <AlertTitle>{{ t('updates.saveFailedTitle') }}</AlertTitle>
      <AlertDescription>{{ t('updates.saveFailed') }}</AlertDescription>
    </Alert>
    <!-- First, and with no network behind it: whatever happened out there, the screen can
         always say which build this is. -->
    <Field>
      <FieldLabel>{{
        t('updates.currentVersion', { version: update.view.currentVersion })
      }}</FieldLabel>
    </Field>
    <Field>
      <div class="flex items-center gap-2">
        <Switch
          id="updates-auto"
          :model-value="settings.autoUpdate"
          :disabled="update.view.unavailable !== null"
          @update:model-value="settings.setAutoUpdate"
        />
        <FieldLabel for="updates-auto">{{ t('updates.autoTitle') }}</FieldLabel>
        <HelpTip>{{ t('updates.autoHint') }}</HelpTip>
      </div>
      <FieldDescription v-if="update.view.unavailable !== null">{{
        t('updates.unsupportedHint')
      }}</FieldDescription>
    </Field>
    <!-- The one line that says where things stand. It is the phase and nothing else: a
         development build says so here instead of saying "up to date", which would be a
         sentence about a check that never happened. -->
    <Field>
      <FieldDescription>{{ t(phase.key, phase.params) }}</FieldDescription>
      <!-- `null` is indeterminate and not zero: the server announced no length, and a bar
           sitting at 0% would be a measurement nobody made. -->
      <Progress
        v-if="percent !== null || update.view.phase.kind === 'downloading'"
        :model-value="percent"
      />
    </Field>
    <Field v-if="notes">
      <FieldLabel>{{ t('updates.notesTitle') }}</FieldLabel>
      <FieldDescription>{{ notes }}</FieldDescription>
    </Field>
    <div class="flex items-center gap-2">
      <Button
        :variant="ButtonVariant.Outline"
        :disabled="!canCheck(update.view)"
        @click="update.check"
      >
        <RefreshCwIcon />
        {{ t('updates.check') }}
      </Button>
      <Button v-if="canInstall(update.view)" @click="update.install">
        <DownloadIcon />
        {{ t('updates.install') }}
      </Button>
    </div>
    <FieldDescription v-if="canInstall(update.view)">{{
      t('updates.installHint')
    }}</FieldDescription>
  </div>
</template>
