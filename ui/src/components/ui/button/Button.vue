<script setup lang="ts">
import type { PrimitiveProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { Primitive } from 'reka-ui'
import { computed } from 'vue'
import { cn } from '@/lib/cn'
import { buttonType } from './buttonType'
import type { ButtonSize, ButtonVariant } from './variants'
import { buttonVariants } from './variants'

interface Props extends PrimitiveProps {
  variant?: ButtonVariant
  size?: ButtonSize
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), {
  as: 'button',
})

// A caller that really wants a submit still gets one: a fallthrough attribute is merged after
// the component's own bindings, so `type="submit"` on the call site wins over this.
const type = computed(() => buttonType(props.as, props.asChild))
</script>

<template>
  <Primitive
    data-slot="button"
    :data-variant="variant"
    :data-size="size"
    :as="as"
    :as-child="asChild"
    :type="type"
    :class="cn(buttonVariants({ variant, size }), props.class)"
  >
    <slot />
  </Primitive>
</template>
