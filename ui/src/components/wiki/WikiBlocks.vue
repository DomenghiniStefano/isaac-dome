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
import { isNameList } from './nameList'
import WikiInline from './WikiInline.vue'
import WikiNameList from './WikiNameList.vue'

const props = defineProps<{
  blocks: Block[]
  iconFor?: (target: Target) => string | null
  canOpen?: (target: Target) => boolean
}>()
const emit = defineEmits<{ navigate: [target: Target, newTab: boolean] }>()

// What every nested WikiInline receives: whether a reference opens, and the way back up. One
// object, so adding to it is one edit, not one per block kind. The nested blocks and a list of
// names take the pictures on top of it.
const forward = computed(() => ({
  canOpen: props.canOpen,
  onNavigate: (target: Target, newTab: boolean) =>
    emit('navigate', target, newTab),
}))
</script>

<template>
  <div class="flex flex-col gap-2 text-row">
    <template v-for="(block, index) in blocks" :key="index">
      <p v-if="block.kind === 'paragraph'">
        <WikiInline :inline="block.inline" v-bind="forward" />
      </p>
      <WikiNameList
        v-else-if="block.kind === 'list' && isNameList(block)"
        :items="block.items"
        :icon-for="iconFor"
        v-bind="forward"
      />
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
            :icon-for="iconFor"
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
      <!-- A heading inside the text is the section heading's grammar at a smaller step — the
           band's bar and the highlight — because the pixel font has no weights and a colour
           alone next to `foreground` did not read as a heading (card #73). -->
      <h3
        v-else-if="block.kind === 'heading' && block.level <= 3"
        class="flex items-center gap-2 text-body text-highlight not-first:mt-2"
      >
        <span class="h-3 w-1 shrink-0 bg-band" />
        <span><WikiInline :inline="block.inline" v-bind="forward" /></span>
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
