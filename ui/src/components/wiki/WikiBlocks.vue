<script setup lang="ts">
import { computed } from 'vue'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { assertNever } from '@/lib/assertNever'
import { cn } from '@/lib/cn'
import type { Block, Target } from '@/lib/ipc/types'
import WikiInline from './WikiInline.vue'

const props = defineProps<{
  blocks: Block[]
  iconFor?: (target: Target) => string | null
}>()
const emit = defineEmits<{ navigate: [target: Target] }>()

// What every nested WikiInline and WikiBlocks receives: the icon resolver and the way back
// up. One object, so adding to it is one edit, not one per block kind.
const forward = computed(() => ({
  iconFor: props.iconFor,
  onNavigate: (target: Target) => emit('navigate', target),
}))
</script>

<template>
  <div class="flex flex-col gap-2 text-row">
    <template v-for="(block, index) in blocks" :key="index">
      <p v-if="block.kind === 'paragraph'">
        <WikiInline :inline="block.inline" v-bind="forward" />
      </p>
      <component
        :is="block.ordered ? 'ol' : 'ul'"
        v-else-if="block.kind === 'list'"
        :class="
          cn(
            'flex flex-col gap-1 pl-4 text-foreground-soft',
            block.ordered ? 'list-decimal' : 'list-disc',
          )
        "
      >
        <li v-for="(item, i) in block.items" :key="i">
          <WikiInline :inline="item.inline" v-bind="forward" />
          <WikiBlocks
            v-if="item.children.length"
            :blocks="item.children"
            v-bind="forward"
            class="mt-1"
          />
        </li>
      </component>
      <Table v-else-if="block.kind === 'table'">
        <TableHeader>
          <TableRow>
            <TableHead v-for="(cell, i) in block.header" :key="i">
              <WikiInline :inline="cell" v-bind="forward" />
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="(row, i) in block.rows" :key="i">
            <TableCell v-for="(cell, j) in row" :key="j">
              <WikiInline :inline="cell" v-bind="forward" />
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
      <h3
        v-else-if="block.kind === 'heading' && block.level <= 3"
        class="text-control text-highlight"
      >
        <WikiInline :inline="block.inline" v-bind="forward" />
      </h3>
      <h4
        v-else-if="block.kind === 'heading'"
        class="text-caption text-highlight"
      >
        <WikiInline :inline="block.inline" v-bind="forward" />
      </h4>
      <p v-else>{{ assertNever(block) }}</p>
    </template>
  </div>
</template>
