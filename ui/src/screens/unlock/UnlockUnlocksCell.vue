<script setup lang="ts">
import { computed } from 'vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import { useMessages } from '@/i18n'
import { targetName } from '@/lib/graph/characterName'
import type { UnlockNode } from '@/lib/ipc/types'

// What an achievement unlocks: the first thing, drawn when it is an item, and how many more.
const props = defineProps<{ node: UnlockNode }>()
const { t } = useMessages()

const first = computed(() => props.node.unlocks[0] ?? null)
const more = computed(() => Math.max(0, props.node.unlocks.length - 1))
</script>

<template>
  <span class="flex min-w-0 items-center gap-2">
    <template v-if="first">
      <PixelSprite
        v-if="first.kind === 'item'"
        :url="first.iconUrl"
        placeholder
        class="size-8 shrink-0"
      />
      <span class="truncate text-caption text-foreground">{{
        targetName(t, first)
      }}</span>
      <span
        v-if="more > 0"
        class="shrink-0 text-label text-subtle-foreground tabular-nums"
        >+{{ more }}</span
      >
    </template>
    <EmptyValue v-else>{{ t('unlock.unlocksNothing') }}</EmptyValue>
  </span>
</template>
