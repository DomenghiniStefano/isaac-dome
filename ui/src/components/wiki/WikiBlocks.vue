<script setup lang="ts">
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

defineProps<{
  blocks: Block[]
  iconFor?: (target: Target) => string | null
}>()
const emit = defineEmits<{ navigate: [target: Target] }>()
</script>

<template>
  <div class="flex flex-col gap-2 text-row">
    <template v-for="(block, index) in blocks" :key="index">
      <p v-if="block.kind === 'paragraph'">
        <WikiInline
          :inline="block.inline"
          :icon-for="iconFor"
          @navigate="emit('navigate', $event)"
        />
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
          <WikiInline
            :inline="item.inline"
            :icon-for="iconFor"
            @navigate="emit('navigate', $event)"
          />
          <WikiBlocks
            v-if="item.children.length"
            :blocks="item.children"
            :icon-for="iconFor"
            class="mt-1"
            @navigate="emit('navigate', $event)"
          />
        </li>
      </component>
      <Table v-else-if="block.kind === 'table'">
        <TableHeader>
          <TableRow>
            <TableHead v-for="(cell, i) in block.header" :key="i">
              <WikiInline
                :inline="cell"
                :icon-for="iconFor"
                @navigate="emit('navigate', $event)"
              />
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="(row, i) in block.rows" :key="i">
            <TableCell v-for="(cell, j) in row" :key="j">
              <WikiInline
                :inline="cell"
                :icon-for="iconFor"
                @navigate="emit('navigate', $event)"
              />
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
      <h3
        v-else-if="block.kind === 'heading' && block.level <= 3"
        class="text-control text-highlight"
      >
        <WikiInline
          :inline="block.inline"
          :icon-for="iconFor"
          @navigate="emit('navigate', $event)"
        />
      </h3>
      <h4
        v-else-if="block.kind === 'heading'"
        class="text-caption text-highlight"
      >
        <WikiInline
          :inline="block.inline"
          :icon-for="iconFor"
          @navigate="emit('navigate', $event)"
        />
      </h4>
      <p v-else>{{ assertNever(block) }}</p>
    </template>
  </div>
</template>
