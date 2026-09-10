<script setup lang="ts">
import type { PrimitiveProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { Primitive } from 'reka-ui'
import { computed } from 'vue'
import { cn } from '@/lib/cn'
import { badgeIcons } from './icons'
import { BadgeVariant, badgeVariants } from './variants'

const props = withDefaults(
  defineProps<
    PrimitiveProps & {
      variant?: BadgeVariant
      class?: HTMLAttributes['class']
    }
  >(),
  {
    as: 'span',
    variant: BadgeVariant.Tag,
  },
)

const delegatedProps = reactiveOmit(props, 'class', 'variant')
const icon = computed(() => badgeIcons[props.variant])
</script>

<template>
  <Primitive
    data-slot="badge"
    :data-variant="variant"
    v-bind="delegatedProps"
    :class="cn(badgeVariants({ variant }), props.class)"
  >
    <component :is="icon" v-if="icon" />
    <slot />
  </Primitive>
</template>
