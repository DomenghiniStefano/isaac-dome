<script setup lang="ts">
import { computed } from 'vue'
import type { ListItem, Target } from '@/lib/ipc/types'
import { nameOf } from './nameList'
import WikiBlocks from './WikiBlocks.vue'
import WikiInline from './WikiInline.vue'
import WikiNameIcon from './WikiNameIcon.vue'

const props = defineProps<{
  items: ListItem[]
  iconFor?: (target: Target) => string | null
  canOpen?: (target: Target) => boolean
}>()
const emit = defineEmits<{ navigate: [target: Target, newTab: boolean] }>()

// Each row's picture, resolved once per render: an achievement list runs to twenty rows.
const rows = computed(() =>
  props.items.map((item) => {
    const name = nameOf(item)
    return {
      item,
      target: name?.target ?? null,
      icon: name ? (props.iconFor?.(name.target) ?? null) : null,
    }
  }),
)

const navigate = (target: Target, newTab: boolean) =>
  emit('navigate', target, newTab)
</script>

<template>
  <!-- A list of names is a column of pictures with the names beside them, as the wiki draws
       `{{achievement text}}`: the picture is the bullet. What sits under a name — how the
       achievement is unlocked — lines up with the name, not with the picture, and reads a step
       quieter. Every picture sits in one column as wide as an achievement drawing, sprites
       centred in it, so the names of every list on a page start at the same edge. The text
       column is at least as tall as the picture, so a name alone sits level with it and a long
       condition runs down from its top. -->
  <ul class="flex flex-col gap-3">
    <li v-for="(row, i) in rows" :key="i" class="flex items-start gap-3">
      <span class="flex min-w-achievement-thumb shrink-0 justify-center">
        <WikiNameIcon v-if="row.target" :src="row.icon" :target="row.target" />
      </span>
      <div class="flex min-h-8 min-w-0 flex-col justify-center gap-0.5">
        <span class="text-body">
          <WikiInline
            :inline="row.item.inline"
            :can-open="canOpen"
            @navigate="navigate"
          />
        </span>
        <template v-for="(child, j) in row.item.children" :key="j">
          <p v-if="child.kind === 'paragraph'" class="text-caption">
            <WikiInline
              :inline="child.inline"
              :can-open="canOpen"
              @navigate="navigate"
            />
          </p>
          <WikiBlocks
            v-else
            :blocks="[child]"
            :icon-for="iconFor"
            :can-open="canOpen"
            @navigate="navigate"
          />
        </template>
      </div>
    </li>
  </ul>
</template>
