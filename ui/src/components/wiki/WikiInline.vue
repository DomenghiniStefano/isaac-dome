<script setup lang="ts">
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { assertNever } from '@/lib/assertNever'
import type { Inline, Target } from '@/lib/ipc/types'
import { Style } from '@/lib/ipc/types'
import { editionLabel } from './editionLabel'

const props = defineProps<{
  inline: Inline[]
  iconFor?: (target: Target) => string | null
}>()
const emit = defineEmits<{ navigate: [target: Target] }>()

const textClass = (style: Style): string => {
  switch (style) {
    case Style.Plain:
      return 'text-foreground-soft'
    case Style.Bold:
      return 'text-foreground'
    case Style.Italic:
      return 'text-foreground-soft italic'
    default:
      return assertNever(style)
  }
}

const icon = (target: Target): string | null => props.iconFor?.(target) ?? null
</script>

<template>
  <!-- Four natures that must tell apart at a glance (Chrome e Stati.dc.html, "Token inline
       della wiki"): a solid underline opens, a dotted one only reads, a reference with no
       sprite leaves no icon hole. Vue condenses whitespace between tags on separate lines,
       so no space lands before a comma. -->
  <template v-for="(token, index) in inline" :key="index">
    <span v-if="token.kind === 'text'" :class="textClass(token.style)">{{
      token.text
    }}</span>
    <Button
      v-else-if="token.kind === 'ref'"
      :variant="ButtonVariant.Ref"
      :size="ButtonSize.Inline"
      @click="emit('navigate', token.target)"
    >
      <img
        v-if="icon(token.target)"
        :src="icon(token.target) ?? undefined"
        alt=""
        class="size-4 pixelated"
      />{{ token.label }}
    </Button>
    <span
      v-else-if="token.kind === 'concept'"
      class="border-b border-dotted border-secondary-edge text-subtle-foreground"
      >{{ token.label }}</span
    >
    <template v-else-if="token.kind === 'edition'">
      <span
        v-if="token.only.length"
        class="border border-secondary-edge px-1.25 text-label text-highlight"
        >{{ editionLabel(token.only) }}</span
      >
      <WikiInline
        :inline="token.inline"
        :icon-for="iconFor"
        @navigate="emit('navigate', $event)"
      />
    </template>
    <span v-else>{{ assertNever(token) }}</span>
  </template>
</template>
