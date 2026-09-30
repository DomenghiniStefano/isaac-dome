<script setup lang="ts">
import { vScrollMemory } from '@/directives/scrollMemory'
import { DatabaseIcon, FolderOpenIcon, TriangleAlertIcon } from '@lucide/vue'
import { computed, onMounted, ref } from 'vue'
import {
  Alert,
  AlertDescription,
  AlertLive,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import ProfileFact from '@/components/data-state/ProfileFact.vue'
import ScreenHeader from '@/components/screen/ScreenHeader.vue'
import { useFormat } from '@/composables/useFormat'
import { useMessages } from '@/i18n'
import type { Message } from '@/i18n/message'
import { dataRow } from '@/lib/data/dataRows'
import { dataLocation, revealDataFile } from '@/lib/ipc/data'
import { asIpcError } from '@/lib/ipc/errors'
import { ipcErrorParts } from '@/lib/ipc/errorText'
import type { DataFile, DataView, IpcError } from '@/lib/ipc/types'

const { t } = useMessages()
const fmt = useFormat()

const view = ref<DataView | null>(null)
// What went wrong the last time a folder was asked for; a load that fails keeps it too.
const failure = ref<{ title: Message; error: IpcError | null } | null>(null)

const load = async () => {
  try {
    view.value = await dataLocation()
  } catch (e) {
    failure.value = { title: 'routes.data', error: asIpcError(e) }
  }
}

const reveal = async (file: DataFile) => {
  failure.value = null
  try {
    await revealDataFile(file)
  } catch (e) {
    failure.value = { title: 'data.revealFailed', error: asIpcError(e) }
  }
}

onMounted(load)

const rows = computed(() =>
  (view.value?.files ?? []).map((f) => dataRow(f, fmt)),
)
const failureParts = computed(() =>
  failure.value === null ? [] : ipcErrorParts(failure.value.error),
)
</script>

<template>
  <div
    v-scroll-memory="'page'"
    class="flex h-full flex-col gap-4 overflow-y-auto px-5.5 pt-5 pb-15"
  >
    <ScreenHeader :icon="DatabaseIcon" :title="t('routes.data')">{{
      t('data.intro')
    }}</ScreenHeader>
    <Alert
      v-if="failure"
      :variant="AlertVariant.Destructive"
      :live="AlertLive.Assertive"
    >
      <TriangleAlertIcon />
      <AlertTitle>{{ t(failure.title) }}</AlertTitle>
      <AlertDescription>
        <span v-for="part in failureParts" :key="part.key">{{
          t(part.key, part.params)
        }}</span>
      </AlertDescription>
    </Alert>
    <Card v-for="row in rows" :key="row.file">
      <CardHeader class="flex-wrap">
        <CardTitle>{{ t(row.title) }}</CardTitle>
      </CardHeader>
      <CardContent class="flex flex-col gap-3">
        <span class="text-label text-subtle-foreground">{{ t(row.hint) }}</span>
        <div class="flex flex-wrap gap-5">
          <div v-if="row.folder" class="flex min-w-0 flex-col gap-1">
            <span class="text-label text-subtle-foreground">{{
              t('data.folder')
            }}</span>
            <span class="text-row break-all text-foreground">{{
              row.folder
            }}</span>
          </div>
          <ProfileFact
            v-if="row.size"
            :label="t('data.size')"
            :value="row.size"
          />
        </div>
        <span
          v-for="part in row.status"
          :key="part.key"
          class="text-label text-subtle-foreground"
          >{{ t(part.key, part.params) }}</span
        >
        <ul v-if="row.contents.length > 0" class="flex flex-col gap-1">
          <li
            v-for="part in row.contents"
            :key="part.key"
            class="text-row text-foreground"
          >
            {{ t(part.key, part.params) }}
          </li>
        </ul>
        <div v-if="row.canReveal">
          <Button :variant="ButtonVariant.Outline" @click="reveal(row.file)">
            <FolderOpenIcon />
            {{ t('data.reveal') }}
          </Button>
        </div>
      </CardContent>
    </Card>
  </div>
</template>
