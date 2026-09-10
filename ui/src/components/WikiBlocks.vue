<script setup lang="ts">
import type { Block, Target } from '../lib/ipc/types'
import { assertNever } from '../lib/assertNever'
import WikiInline from './WikiInline.vue'

defineProps<{ blocks: Block[] }>()
const emit = defineEmits<{ navigate: [target: Target] }>()
</script>

<template>
  <template v-for="(b, idx) in blocks" :key="idx">
    <p v-if="b.kind === 'paragraph'">
      <WikiInline :inline="b.inline" @navigate="emit('navigate', $event)" />
    </p>
    <template v-else-if="b.kind === 'list'">
      <ol v-if="b.ordered" class="flex flex-col gap-1">
        <li v-for="(item, i) in b.items" :key="i">
          <WikiInline
            :inline="item.inline"
            @navigate="emit('navigate', $event)"
          />
          <WikiBlocks
            v-if="item.children.length"
            :blocks="item.children"
            @navigate="emit('navigate', $event)"
          />
        </li>
      </ol>
      <ul v-else class="flex flex-col gap-1">
        <li v-for="(item, i) in b.items" :key="i">
          <WikiInline
            :inline="item.inline"
            @navigate="emit('navigate', $event)"
          />
          <WikiBlocks
            v-if="item.children.length"
            :blocks="item.children"
            @navigate="emit('navigate', $event)"
          />
        </li>
      </ul>
    </template>
    <div v-else-if="b.kind === 'table'" class="overflow-x-auto">
      <table>
        <thead>
          <tr>
            <th v-for="(cell, i) in b.header" :key="i" class="px-2 text-left">
              <WikiInline :inline="cell" @navigate="emit('navigate', $event)" />
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="(row, i) in b.rows" :key="i">
            <td v-for="(cell, j) in row" :key="j" class="px-2">
              <WikiInline :inline="cell" @navigate="emit('navigate', $event)" />
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <template v-else-if="b.kind === 'heading'">
      <h3 v-if="b.level === 3" class="text-foreground">
        <WikiInline :inline="b.inline" @navigate="emit('navigate', $event)" />
      </h3>
      <h4 v-else class="text-foreground">
        <WikiInline :inline="b.inline" @navigate="emit('navigate', $event)" />
      </h4>
    </template>
    <p v-else>{{ assertNever(b) }}</p>
  </template>
</template>
