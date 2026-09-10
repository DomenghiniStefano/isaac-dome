<script setup lang="ts">
import { MinusIcon, SquareIcon, XIcon } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'

const emit = defineEmits<{
  minimize: []
  toggleMaximize: []
  closeWindow: []
}>()
const { t } = useMessages()
</script>

<template>
  <!-- Three 38px controls (Chrome e Stati.dc.html, "Misure della fascia tab"). With the
       window unfocused the title bar's data-focused dims the glyphs. -->
  <div class="flex items-stretch">
    <Button
      :variant="ButtonVariant.Chrome"
      :size="ButtonSize.Window"
      :aria-label="t('shell.minimize')"
      class="group-data-[focused=false]:text-chrome-inactive-foreground"
      @click="emit('minimize')"
    >
      <MinusIcon class="size-2.5" />
    </Button>
    <Button
      :variant="ButtonVariant.Chrome"
      :size="ButtonSize.Window"
      :aria-label="t('shell.maximize')"
      class="group-data-[focused=false]:text-chrome-inactive-foreground"
      @click="emit('toggleMaximize')"
    >
      <SquareIcon class="size-2.5" />
    </Button>
    <Button
      :variant="ButtonVariant.ChromeDanger"
      :size="ButtonSize.Window"
      :aria-label="t('shell.closeWindow')"
      class="group-data-[focused=false]:text-chrome-inactive-foreground"
      @click="emit('closeWindow')"
    >
      <XIcon class="size-2.5" />
    </Button>
  </div>
</template>
