<script setup lang="ts">
import { LayoutPanelTopIcon, TriangleAlertIcon } from '@lucide/vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertLive,
  AlertVariant,
} from '@/components/ui/alert'
import { Field, FieldLabel } from '@/components/ui/field'
import { Switch } from '@/components/ui/switch'
import { HelpTip } from '@/components/ui/tooltip'
import { useMessages } from '@/i18n'
import { sessionStopped } from '@/lib/window/sessionHealth'
import { useSettingsStore } from '@/stores/settings'
import ScreenHeader from './ScreenHeader.vue'

// The session's one honest place: the switch that decides whether it is kept, and what it holds
// when it is. The switch came from the Background screen, whose subject is the tray — an intro
// that had to say "and what you find when you open it again" was carrying two subjects.
const settings = useSettingsStore()
const { t } = useMessages()

// Said as sentences, not as a schema: somebody reading a settings page is not reading a document
// format. The list grew with 3.7b and 3.7c, and it is the only place in the app that says what is
// written down about how you were working.
const kept = [
  'tabsSettings.keptWindows',
  'tabsSettings.keptTabs',
  'tabsSettings.keptReading',
  'tabsSettings.keptSidebar',
] as const
</script>

<template>
  <div class="flex max-w-250 flex-col gap-4">
    <ScreenHeader
      :icon="LayoutPanelTopIcon"
      :title="t('routes.tabsSettings')"
      >{{ t('tabsSettings.intro') }}</ScreenHeader
    >
    <!-- The one write error that is not swallowed, next to the switch that turns the thing off.
         It lived in the About dialog from 3.7b until this screen existed to hold it. -->
    <Alert v-if="sessionStopped" :variant="AlertVariant.Destructive">
      <TriangleAlertIcon />
      <AlertTitle>{{ t('tabsSettings.stoppedTitle') }}</AlertTitle>
      <AlertDescription>{{ t('tabsSettings.stopped') }}</AlertDescription>
    </Alert>
    <!-- The switch is moved first and saved after, so a failed write leaves the app behaving
         the way the user just asked and says what didn't happen. -->
    <Alert
      v-if="settings.saveFailed"
      :variant="AlertVariant.Destructive"
      :live="AlertLive.Assertive"
    >
      <TriangleAlertIcon />
      <AlertTitle>{{ t('tabsSettings.saveFailedTitle') }}</AlertTitle>
      <AlertDescription>{{ t('tabsSettings.saveFailed') }}</AlertDescription>
    </Alert>
    <Field>
      <div class="flex items-center gap-2">
        <Switch
          id="tabs-resume"
          :model-value="settings.resumeTabs"
          @update:model-value="settings.setResumeTabs"
        />
        <FieldLabel for="tabs-resume">{{
          t('tabsSettings.resumeTitle')
        }}</FieldLabel>
        <!-- The list of what is kept belongs to this switch and nothing else, so it is in the
             switch's own explanation rather than in a block below it with a heading of its
             own: a heading is what you give a section, and this was never one. -->
        <HelpTip>
          <span class="flex flex-col gap-1.5">
            <span>{{ t('tabsSettings.resumeHint') }}</span>
            <span
              class="text-label tracking-caps text-subtle-foreground uppercase"
              >{{ t('tabsSettings.keptTitle') }}</span
            >
            <span
              v-for="line in kept"
              :key="line"
              class="text-caption text-foreground-soft"
              >{{ t(line) }}</span
            >
          </span>
        </HelpTip>
      </div>
    </Field>
  </div>
</template>
