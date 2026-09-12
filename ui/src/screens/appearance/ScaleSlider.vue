<script setup lang="ts">
import { computed } from 'vue'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Kbd, KbdGroup } from '@/components/ui/kbd'
import { Slider } from '@/components/ui/slider'
import { i18n, useMessages } from '@/i18n'
import { KeyName } from '@/lib/constants/keyNames'
import { cn } from '@/lib/cn'
import { percentAt, scalePercents, stepIndex } from '@/lib/scale/steps'

const props = defineProps<{ percent: number }>()
const emit = defineEmits<{ pick: [percent: number] }>()
const { t } = useMessages()

// The slider walks positions, not percentages: the ladder is unevenly spaced and the
// slider's own spacing is even, which is what makes 50 and 67 as far apart as 175 and 200.
const position = computed(() => [stepIndex(props.percent)])
const last = scalePercents.length - 1

const label = (percent: number): string =>
  new Intl.NumberFormat(i18n.global.locale.value, {
    style: 'percent',
  }).format(percent / 100)

const onPick = (value: number[] | undefined) => {
  const index = value?.[0]
  if (index !== undefined) emit('pick', percentAt(index))
}
</script>

<template>
  <Card>
    <CardHeader>
      <CardTitle>{{ t('appearance.scaleTitle') }}</CardTitle>
      <span
        class="flex items-center gap-1.5 text-caption text-subtle-foreground"
        >{{ t('appearance.shortcut') }}
        <KbdGroup>
          <Kbd>{{ KeyName.Ctrl }}</Kbd>
          <Kbd>{{ KeyName.Plus }}</Kbd>
        </KbdGroup>
        <KbdGroup>
          <Kbd>{{ KeyName.Ctrl }}</Kbd>
          <Kbd>{{ KeyName.Minus }}</Kbd>
        </KbdGroup>
        <KbdGroup>
          <Kbd>{{ KeyName.Ctrl }}</Kbd>
          <Kbd>{{ KeyName.Zero }}</Kbd>
        </KbdGroup>
        {{ t('appearance.shortcutReset') }}</span
      >
    </CardHeader>
    <CardContent class="flex flex-col gap-2">
      <!-- The step under the thumb is the one in the accent: the others are ticks, so the
           ladder reads as a ladder and not as a ruler. -->
      <div class="flex justify-between">
        <span
          v-for="(step, index) in scalePercents"
          :key="step"
          :class="
            cn(
              'text-label tabular-nums',
              index === position[0]
                ? 'text-highlight'
                : 'text-faint-foreground',
            )
          "
          >{{ index === position[0] ? label(step) : '·' }}</span
        >
      </div>
      <Slider
        :model-value="position"
        :min="0"
        :max="last"
        :step="1"
        :aria-label="t('appearance.scaleLabel')"
        @update:model-value="onPick"
      />
    </CardContent>
  </Card>
</template>
