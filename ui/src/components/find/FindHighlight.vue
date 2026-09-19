<script setup lang="ts">
import { computed } from 'vue'
import { segments } from '@/lib/find/highlight'

// Draws a row's text with the matched parts painted. The splitting is `lib/find/highlight`'s
// and tested there; this only chooses the paint, so a row using it reads exactly like one that
// does not while nothing is being searched.
//
// Two colours and not one: every match is marked, and the one you are standing on is marked
// louder. A count that says "3 di 17" while all seventeen look the same has told you a number
// you cannot act on.
const props = defineProps<{ text: string; query: string; current?: boolean }>()

const pieces = computed(() => segments(props.text, props.query))
</script>

<template>
  <template v-for="(piece, at) in pieces" :key="at">
    <mark
      v-if="piece.match"
      :class="
        current
          ? 'bg-find-current text-find-current-foreground'
          : 'bg-find-match text-find-match-foreground'
      "
      >{{ piece.text }}</mark
    >
    <template v-else>{{ piece.text }}</template>
  </template>
</template>
