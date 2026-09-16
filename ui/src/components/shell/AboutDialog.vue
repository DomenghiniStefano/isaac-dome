<script setup lang="ts">
import { TriangleAlertIcon } from '@lucide/vue'
import { onMounted, ref } from 'vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import { Button, ButtonVariant } from '@/components/ui/button'
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { useMessages } from '@/i18n'
import { appVersion } from '@/lib/app/version'
import { AppName } from '@/lib/constants/app'
import { sessionStopped } from '@/lib/window/sessionHealth'

// What "Informazioni" has to say is short — the name, the version, what the app promises and
// where its text comes from — so it is a dialog over the tab, not a tab of its own
// (`docs/BACKLOG.md` B25).
defineProps<{ open: boolean }>()
const emit = defineEmits<{ 'update:open': [open: boolean] }>()
const { t } = useMessages()

const version = ref<string | null>(null)
onMounted(async () => {
  version.value = await appVersion()
})

// Each promise is a title and what it means: the title alone would read as a slogan, the
// sentence alone as small print.
const promises = [
  {
    title: 'about.promises.readOnlyTitle',
    body: 'about.promises.readOnly',
  },
  {
    title: 'about.promises.offlineTitle',
    body: 'about.promises.offline',
  },
  {
    title: 'about.promises.oneFileTitle',
    body: 'about.promises.oneFile',
  },
] as const
</script>

<template>
  <Dialog :open="open" @update:open="emit('update:open', $event)">
    <DialogContent class="max-w-lg">
      <DialogHeader>
        <DialogTitle>{{ AppName }}</DialogTitle>
      </DialogHeader>
      <DialogDescription>{{ t('about.fanMade') }}</DialogDescription>
      <div class="flex flex-col gap-3 px-4">
        <!-- The one write error that is not swallowed. It says the tabs are safe and that they
             will not come back, because the failure it reports is otherwise invisible until the
             next start. Here and not on a screen: it is the shell's own fact, and one click from
             every window. It moves next to the switch that turns the session off when 3.6
             builds Settings. -->
        <Alert v-if="sessionStopped" :variant="AlertVariant.Destructive">
          <TriangleAlertIcon />
          <AlertTitle>{{ t('about.sessionStoppedTitle') }}</AlertTitle>
          <AlertDescription>{{ t('about.sessionStopped') }}</AlertDescription>
        </Alert>
        <div class="flex flex-col gap-0.75">
          <span class="text-label text-subtle-foreground">{{
            t('about.version')
          }}</span>
          <span class="text-row text-foreground tabular-nums">{{
            version ?? t('about.versionUnknown')
          }}</span>
        </div>
        <div class="flex flex-col gap-2">
          <span
            class="text-label tracking-caps text-subtle-foreground uppercase"
            >{{ t('about.promisesTitle') }}</span
          >
          <div
            v-for="promise in promises"
            :key="promise.title"
            class="flex flex-col gap-0.75"
          >
            <span class="text-control text-foreground">{{
              t(promise.title)
            }}</span>
            <span class="text-caption text-foreground-soft">{{
              t(promise.body)
            }}</span>
          </div>
        </div>
        <div class="flex flex-col gap-1.5">
          <span
            class="text-label tracking-caps text-subtle-foreground uppercase"
            >{{ t('about.creditsTitle') }}</span
          >
          <span class="text-caption text-foreground-soft">{{
            t('about.wikiText')
          }}</span>
          <span class="text-caption text-foreground-soft">{{
            t('about.assets')
          }}</span>
          <span class="text-caption text-foreground-soft">{{
            t('about.font')
          }}</span>
        </div>
      </div>
      <DialogFooter>
        <DialogClose as-child>
          <Button :variant="ButtonVariant.Outline">{{ t('ui.close') }}</Button>
        </DialogClose>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>
