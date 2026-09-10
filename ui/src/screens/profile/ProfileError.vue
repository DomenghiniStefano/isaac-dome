<script setup lang="ts">
import { RefreshCwIcon, TriangleAlertIcon } from '@lucide/vue'
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
import type { IpcError } from '@/lib/ipc/types'

const props = defineProps<{ error: IpcError | null }>()
const emit = defineEmits<{ retry: [] }>()
const { t } = useMessages()

// `null` is a failure that isn't an IpcError at all: the backend never answered.
const message = computed((): string => {
  const e = props.error
  if (e === null) return t('profile.errors.noBackend')
  switch (e.kind) {
    case 'noActiveProfile':
      return t('profile.errors.noActiveProfile')
    case 'unknownProfile':
      return t('profile.errors.unknownProfile')
    case 'unreadableSave':
      return `${t('profile.errors.unreadableSave')} ${e.reason}`
    case 'settingsNotWritable':
      return `${t('profile.errors.settingsNotWritable')} ${e.reason}`
    case 'unknownTarget':
      return t('profile.errors.unknownTarget')
    case 'catalogUnavailable':
      return t('profile.errors.catalogUnavailable')
    case 'storeUnavailable':
      return `${t('profile.errors.storeUnavailable')} ${e.reason}`
    case 'wikiUnavailable':
      return t('profile.errors.wikiUnavailable')
    default:
      return assertNever(e)
  }
})
</script>

<template>
  <Alert :variant="AlertVariant.Destructive">
    <TriangleAlertIcon />
    <AlertTitle>{{ t('profile.errors.title') }}</AlertTitle>
    <AlertDescription class="flex flex-col items-start gap-2">
      {{ message }}
      <Button :variant="ButtonVariant.Outline" @click="emit('retry')">
        <RefreshCwIcon />{{ t('profile.errors.retry') }}
      </Button>
    </AlertDescription>
  </Alert>
</template>
