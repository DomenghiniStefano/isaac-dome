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
import { useIpcErrorText } from '@/composables/useIpcErrorText'
import { useMessages } from '@/i18n'
import type { IpcError } from '@/lib/ipc/types'

// The title names what couldn't be read; the profile's is the default, because most of what
// the app reads is read on the profile.
const props = defineProps<{ error: IpcError | null; title?: string }>()
const emit = defineEmits<{ retry: [] }>()
const { t } = useMessages()
const { errorText } = useIpcErrorText()

const message = computed((): string => errorText(props.error))
const heading = computed(() => props.title ?? t('profile.errors.title'))
</script>

<template>
  <Alert :variant="AlertVariant.Destructive">
    <TriangleAlertIcon />
    <AlertTitle>{{ heading }}</AlertTitle>
    <AlertDescription class="flex flex-col items-start gap-2">
      {{ message }}
      <Button :variant="ButtonVariant.Outline" @click="emit('retry')">
        <RefreshCwIcon />{{ t('profile.errors.retry') }}
      </Button>
    </AlertDescription>
  </Alert>
</template>
