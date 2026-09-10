<script setup lang="ts">
import type { ProgressRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { ProgressIndicator, ProgressRoot } from 'reka-ui'
import { computed } from 'vue'
import { cn } from '@/lib/cn'
import { progressShares, progressBounds } from './progressShares'
import type { ProgressSize, ProgressTone } from './variants'
import { progressIndicatorVariants, progressVariants } from './variants'

const props = withDefaults(
  defineProps<
    ProgressRootProps & {
      unknown?: number
      size?: ProgressSize
      tone?: ProgressTone
      class?: HTMLAttributes['class']
    }
  >(),
  {
    modelValue: 0,
    max: 100,
    unknown: 0,
    size: undefined,
    tone: undefined,
  },
)

const delegatedProps = reactiveOmit(
  props,
  'class',
  'unknown',
  'modelValue',
  'max',
  'size',
  'tone',
)

const bounds = computed(() => progressBounds(props.modelValue ?? 0, props.max))

const shares = computed(() =>
  progressShares(props.modelValue ?? 0, props.unknown, props.max),
)
// The widths are computed, the vocabulary isn't: CSS variables bound here, consumed by
// w-(--progress-value) and w-(--progress-unknown) in the classes.
const shareVariables = computed(() => ({
  '--progress-value': `${shares.value.value}%`,
  '--progress-unknown': `${shares.value.unknown}%`,
}))
</script>

<template>
  <ProgressRoot
    data-slot="progress"
    v-bind="delegatedProps"
    :model-value="bounds.value"
    :max="bounds.max"
    :style="shareVariables"
    :class="cn(progressVariants({ size }), props.class)"
  >
    <ProgressIndicator
      data-slot="progress-indicator"
      :class="progressIndicatorVariants({ tone })"
    />
    <div
      data-slot="progress-unknown"
      class="h-full w-(--progress-unknown) hatch-unknown"
    />
  </ProgressRoot>
</template>
