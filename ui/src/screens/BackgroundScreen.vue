<script setup lang="ts">
import { MonitorDotIcon, TriangleAlertIcon } from '@lucide/vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import { Field, FieldDescription, FieldLabel } from '@/components/ui/field'
import { Switch } from '@/components/ui/switch'
import { useMessages } from '@/i18n'
import { useSettingsStore } from '@/stores/settings'
import ScreenHeader from './ScreenHeader.vue'

const settings = useSettingsStore()
const { t } = useMessages()
</script>

<template>
  <div class="flex max-w-250 flex-col gap-4">
    <ScreenHeader :icon="MonitorDotIcon" :title="t('routes.background')">{{
      t('background.intro')
    }}</ScreenHeader>
    <!-- The switch is moved first and saved after, so a failed write leaves the app behaving
         the way the user just asked and says what didn't happen. -->
    <Alert v-if="settings.saveFailed" :variant="AlertVariant.Destructive">
      <TriangleAlertIcon />
      <AlertTitle>{{ t('background.saveFailedTitle') }}</AlertTitle>
      <AlertDescription>{{ t('background.saveFailed') }}</AlertDescription>
    </Alert>
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
      </div>
      <FieldDescription>{{ t('background.stayHint') }}</FieldDescription>
    </Field>
    <Field>
      <div class="flex items-center gap-2">
        <Switch
          id="background-resume"
          :model-value="settings.resumeTabs"
          @update:model-value="settings.setResumeTabs"
        />
        <FieldLabel for="background-resume">{{
          t('background.resumeTitle')
        }}</FieldLabel>
      </div>
      <FieldDescription>{{ t('background.resumeHint') }}</FieldDescription>
    </Field>
  </div>
</template>
