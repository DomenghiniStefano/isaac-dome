<script setup lang="ts">
import { vScrollMemory } from '@/directives/scrollMemory'
import { MonitorDotIcon, TriangleAlertIcon } from '@lucide/vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertLive,
  AlertVariant,
} from '@/components/ui/alert'
import { Field, FieldDescription, FieldLabel } from '@/components/ui/field'
import { Switch } from '@/components/ui/switch'
import { HelpTip } from '@/components/ui/tooltip'
import { computed, onMounted } from 'vue'
import { autostartFailurePart } from '@/lib/ipc/errorText'
import { useMessages } from '@/i18n'
import { useSettingsStore } from '@/stores/settings'
import ScreenHeader from '@/components/screen/ScreenHeader.vue'

const settings = useSettingsStore()
const { t } = useMessages()

// Read when the screen mounts, never once at startup: the app sits in the tray for days, and
// the Startup tab can have turned the entry off underneath it.
onMounted(() => {
  void settings.refreshAutostart()
})

// The refused write and the ignored one are two different things to go and do, so they are two
// sentences: `null` when the last failure was not this switch's.
const startFailure = computed(() =>
  settings.saveError?.kind === 'autostartNotWritable'
    ? autostartFailurePart(settings.saveError.reason)
    : null,
)
</script>

<template>
  <div
    v-scroll-memory="'page'"
    class="flex h-full flex-col gap-4 overflow-y-auto px-5.5 pt-5 pb-15"
  >
    <ScreenHeader :icon="MonitorDotIcon" :title="t('routes.background')">{{
      t('background.intro')
    }}</ScreenHeader>
    <!-- Two failures, two sentences. The switches below move first and are saved after, so a
         failed write leaves the app behaving the way the user just asked; the one above is
         about a login that has not happened, so a refused write means nothing happened at
         all. -->
    <Alert
      v-if="startFailure"
      :variant="AlertVariant.Destructive"
      :live="AlertLive.Assertive"
    >
      <TriangleAlertIcon />
      <AlertTitle>{{ t('background.startFailedTitle') }}</AlertTitle>
      <AlertDescription>{{ t(startFailure.key) }}</AlertDescription>
    </Alert>
    <Alert
      v-else-if="settings.saveFailed"
      :variant="AlertVariant.Destructive"
      :live="AlertLive.Assertive"
    >
      <TriangleAlertIcon />
      <AlertTitle>{{ t('background.saveFailedTitle') }}</AlertTitle>
      <AlertDescription>{{ t('background.saveFailed') }}</AlertDescription>
    </Alert>
    <!-- First of the three: it is about when the app starts, and the other two are about what
         happens afterwards. -->
    <Field>
      <div class="flex items-center gap-2">
        <Switch
          id="background-start"
          :model-value="settings.autostart"
          :disabled="!settings.autostartAvailable"
          @update:model-value="settings.setAutostart"
        />
        <FieldLabel for="background-start">{{
          t('background.startTitle')
        }}</FieldLabel>
        <HelpTip>{{ t('background.startHint') }}</HelpTip>
      </div>
      <FieldDescription v-if="!settings.autostartAvailable">{{
        t('background.startDev')
      }}</FieldDescription>
      <!-- The two switches are not coupled in code — one switch silently moving another is not
           something this app does — so the combination is stated instead. -->
      <FieldDescription v-else-if="!settings.stayInBackground">{{
        t('background.startWithoutBackground')
      }}</FieldDescription>
    </Field>
    <Field>
      <div class="flex items-center gap-2">
        <Switch
          id="background-stay"
          :model-value="settings.stayInBackground"
          @update:model-value="settings.setStayInBackground"
        />
        <FieldLabel for="background-stay">{{
          t('background.stayTitle')
        }}</FieldLabel>
        <HelpTip>{{ t('background.stayHint') }}</HelpTip>
      </div>
    </Field>
    <!-- "Reopen the tabs" was here until 3.6a and is on the Tabs screen now. It was never this
         screen's subject: the intro had to say "and what you find when you open it again" to
         cover it, which is a sentence carrying two subjects. -->
  </div>
</template>
