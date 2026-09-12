<script setup lang="ts">
import { onMounted, ref } from 'vue'
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

const promises = [
  'about.promises.readOnly',
  'about.promises.offline',
  'about.promises.oneFile',
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
        <div class="flex flex-col gap-0.75">
          <span class="text-label text-subtle-foreground">{{
            t('about.version')
          }}</span>
          <span class="text-row text-foreground tabular-nums">{{
            version ?? t('about.versionUnknown')
          }}</span>
        </div>
        <div class="flex flex-col gap-1.5">
          <span class="text-label text-subtle-foreground">{{
            t('about.promisesTitle')
          }}</span>
          <ul class="flex list-disc flex-col gap-1 pl-4 text-row">
            <li
              v-for="promise in promises"
              :key="promise"
              class="text-foreground-soft"
            >
              {{ t(promise) }}
            </li>
          </ul>
        </div>
        <div class="flex flex-col gap-1.5">
          <span class="text-label text-subtle-foreground">{{
            t('about.creditsTitle')
          }}</span>
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
