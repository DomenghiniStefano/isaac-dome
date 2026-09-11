<script setup lang="ts">
import { TriangleAlertIcon } from '@lucide/vue'
import { computed } from 'vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import { useIpcErrorText } from '@/composables/useIpcErrorText'
import { useMessages } from '@/i18n'
import type { IpcError } from '@/lib/ipc/types'

const props = defineProps<{ error: IpcError | null }>()
const { t } = useMessages()
const { errorText } = useIpcErrorText()

const message = computed((): string => errorText(props.error))
</script>

<template>
  <!-- A write the backend refused: the queue on screen is still the one it had, and this says
       why it didn't change. It goes away with the next write that succeeds. -->
  <Alert :variant="AlertVariant.Destructive">
    <TriangleAlertIcon />
    <AlertTitle>{{ t('queue.errorTitle') }}</AlertTitle>
    <AlertDescription>{{ message }}</AlertDescription>
  </Alert>
</template>
