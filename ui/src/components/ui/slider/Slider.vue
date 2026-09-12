<script setup lang="ts">
import type { SliderRootEmits, SliderRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import {
  SliderRange,
  SliderRoot,
  SliderThumb,
  SliderTrack,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  SliderRootProps & { class?: HTMLAttributes['class'] }
>()
const emits = defineEmits<SliderRootEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <!-- A flat track and a square thumb, like every other control in the kit: no radius, no
       transition, the focus ring the app's one ring. The track is the data surface, what is
       behind the thumb is the accent. -->
  <SliderRoot
    data-slot="slider"
    v-bind="forwarded"
    :class="
      cn(
        'relative flex w-full cursor-pointer touch-none items-center select-none data-[disabled]:cursor-not-allowed',
        props.class,
      )
    "
  >
    <SliderTrack
      data-slot="slider-track"
      class="relative h-1.5 w-full grow border border-input bg-data group-data-[disabled]:bg-muted"
    >
      <SliderRange
        data-slot="slider-range"
        class="absolute h-full bg-primary"
      />
    </SliderTrack>
    <SliderThumb
      data-slot="slider-thumb"
      class="block size-3.5 border border-primary-edge bg-foreground data-[disabled]:border-secondary data-[disabled]:bg-faint-foreground"
    />
  </SliderRoot>
</template>
