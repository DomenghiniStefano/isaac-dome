<script setup lang="ts">
import type { Component } from 'vue'
import { MinusIcon, SquareIcon, XIcon } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'

const emit = defineEmits<{
  minimize: []
  toggleMaximize: []
  closeWindow: []
}>()
const { t } = useMessages()

interface WindowControl {
  label: MessageKey<MessageSchema>
  icon: Component
  variant: ButtonVariant
  press: () => void
}

const controls: WindowControl[] = [
  {
    label: 'shell.minimize',
    icon: MinusIcon,
    variant: ButtonVariant.Chrome,
    press: () => emit('minimize'),
  },
  {
    label: 'shell.maximize',
    icon: SquareIcon,
    variant: ButtonVariant.Chrome,
    press: () => emit('toggleMaximize'),
  },
  {
    label: 'shell.closeWindow',
    icon: XIcon,
    variant: ButtonVariant.ChromeDanger,
    press: () => emit('closeWindow'),
  },
]
</script>

<template>
  <!-- Three 38px controls (Chrome e Stati.dc.html, "Misure della fascia tab"). With the
       window unfocused the title bar's data-focused dims the glyphs. -->
  <div class="flex items-stretch">
    <Button
      v-for="control in controls"
      :key="control.label"
      :variant="control.variant"
      :size="ButtonSize.Window"
      :aria-label="t(control.label)"
      class="group-data-[focused=false]:text-chrome-inactive-foreground"
      @click="control.press"
    >
      <component :is="control.icon" class="size-2.5" />
    </Button>
  </div>
</template>
