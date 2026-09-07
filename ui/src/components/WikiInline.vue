<script setup lang="ts">
import type { Inline, Target } from '../lib/ipc/types'
import { Style } from '../lib/ipc/types'
import { assertNever } from '../lib/assertNever'

defineProps<{ inline: Inline[] }>()
const emit = defineEmits<{ navigate: [target: Target] }>()

// Verification only, not design: what the target carries behind a ref, to check
// that the dataset points where it should.
const targetIdText = (t: Target): string => {
  switch (t.kind) {
    case 'item':
    case 'trinket':
    case 'character':
    case 'achievement':
    case 'transformation':
    case 'entity':
      return String(t.id)
    case 'challenge':
      return String(t.number)
    case 'stage':
    case 'room':
    case 'pickup':
      return t.name
    default:
      return assertNever(t)
  }
}

const textClass = (style: Style) => {
  switch (style) {
    case Style.Plain:
      return ''
    case Style.Bold:
      return 'font-bold'
    case Style.Italic:
      return 'italic'
    default:
      return assertNever(style)
  }
}
</script>

<template>
  <template v-for="(i, idx) in inline" :key="idx">
    <span v-if="i.kind === 'text'" :class="textClass(i.style)">{{
      i.text
    }}</span>
    <button
      v-else-if="i.kind === 'ref'"
      class="underline"
      @click="emit('navigate', i.target)"
    >
      {{ i.label }} ({{ i.target.kind }} {{ targetIdText(i.target) }})
    </button>
    <span v-else-if="i.kind === 'concept'" class="underline decoration-dotted"
      >{{ i.label }}
    </span>
    <span v-else-if="i.kind === 'edition'" class="flex gap-1">
      [{{ i.only.join(', ') }}]
      <WikiInline :inline="i.inline" @navigate="emit('navigate', $event)" />
    </span>
    <span v-else>{{ assertNever(i) }}</span>
  </template>
</template>
