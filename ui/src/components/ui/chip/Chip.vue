<script setup lang="ts">
import type { PrimitiveProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { Primitive } from 'reka-ui'
import { cn } from '@/lib/cn'
import type { Tone } from '@/lib/wiki/tone'
import { chipVariants } from './variants'

// A coloured, inline label for a fact the wiki, the game or the save already states: an
// edition, a quality tier, a category. Unlike `Badge`, which colours a state we computed
// (done, blocked, unknown), a `Chip`'s colour is data itself — so it takes a `tone`, not a
// `variant`, and carries no icon of its own.
const props = withDefaults(
  defineProps<
    PrimitiveProps & {
      tone: Tone
      class?: HTMLAttributes['class']
    }
  >(),
  { as: 'span' },
)

const delegatedProps = reactiveOmit(props, 'class', 'tone')
</script>

<template>
  <Primitive
    data-slot="chip"
    :data-tone="tone"
    v-bind="delegatedProps"
    :class="cn(chipVariants({ tone }), props.class)"
  >
    <slot />
  </Primitive>
</template>
