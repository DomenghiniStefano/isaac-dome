<script setup lang="ts">
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { assertNever } from '@/lib/assertNever'
import { cn } from '@/lib/cn'
import type { Inline, Target } from '@/lib/ipc/types'
import { Style } from '@/lib/ipc/types'
import { editionLabel } from './editionLabel'

const props = defineProps<{
  inline: Inline[]
  iconFor?: (target: Target) => string | null
  // Whether a reference leads to a page. Without it every reference opens, as on the Kit
  // page; the wiki store answers from its index, so a stage or an entity the dataset lacks
  // reads like a concept instead of leading to a page that says "unknown".
  canOpen?: (target: Target) => boolean
}>()
// `newTab` is the click's modifier: Ctrl opens the reference beside the page, as a browser
// does with a link.
const emit = defineEmits<{ navigate: [target: Target, newTab: boolean] }>()

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

// Each reference's icon, whether it opens, and whether it follows another reference with
// nothing between — an infobox lists its places as bare references in a row, and drawn
// back to back they read as one word. Resolved once per render instead of once per use in
// the template: a page can carry 185 references.
const refs = computed(() =>
  props.inline.map((token, index) =>
    token.kind === 'ref'
      ? {
          icon: props.iconFor?.(token.target) ?? null,
          opens: props.canOpen?.(token.target) ?? true,
          gap: props.inline[index - 1]?.kind === 'ref',
        }
      : null,
  ),
)
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
      v-else-if="token.kind === 'ref' && refs[index]?.opens"
      :variant="ButtonVariant.Ref"
      :size="ButtonSize.Inline"
      :class="refs[index]?.gap ? 'ml-1' : undefined"
      @click="emit('navigate', token.target, $event.ctrlKey)"
    >
      <img
        v-if="refs[index]?.icon"
        :src="refs[index]?.icon ?? undefined"
        alt=""
        class="mr-0.75 inline-block size-4 align-text-bottom pixelated"
      />{{ token.label }}
    </Button>
    <span
      v-else-if="token.kind === 'ref'"
      :class="
        cn(
          'border-b border-dotted border-secondary-edge text-subtle-foreground',
          refs[index]?.gap && 'ml-1',
        )
      "
      ><img
        v-if="refs[index]?.icon"
        :src="refs[index]?.icon ?? undefined"
        alt=""
        class="mr-0.75 inline-block size-4 align-text-bottom pixelated"
      />{{ token.label }}</span
    >
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
        :can-open="canOpen"
        @navigate="(target, newTab) => emit('navigate', target, newTab)"
      />
    </template>
    <span v-else>{{ assertNever(token) }}</span>
  </template>
</template>
