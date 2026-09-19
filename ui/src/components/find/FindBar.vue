<script setup lang="ts">
import { ChevronDownIcon, ChevronUpIcon, XIcon } from '@lucide/vue'
import { computed, watch } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { useMessages } from '@/i18n'
import { FindAction, findAction } from '@/lib/find/keyboard'
import { Direction, findMatches, stepMatch } from '@/lib/find/matches'
import type { FindRow } from '@/lib/find/matches'

// A bar, not a dialog: the dialog is the palette's shape, and the palette is exactly what this
// is not. It sits over the page without taking it.
//
// The screen brings its rows and nothing else — this component does not know what an item, an
// achievement or a wiki block is. Where the current match should be scrolled to is the
// screen's job too: it owns the virtualizer, so it gets an index and calls `scrollToIndex`.
// The query and the current row are the screen's, not the bar's: the rows have to paint the
// match themselves, and a row cannot ask a sibling what is being searched.
const props = defineProps<{ rows: readonly FindRow[] }>()
const emit = defineEmits<{ move: [index: number]; close: [] }>()
const typed = defineModel<string>('query', { required: true })
const current = defineModel<string | null>('current', { required: true })

const { t } = useMessages()

const state = computed(() =>
  findMatches(props.rows, typed.value, current.value),
)

// The rows can change under an open bar — a filter moved, an answer landed — so the current
// row is read back from the state rather than trusted: `findMatches` already keeps it when it
// survives and falls back to the first match when it does not.
watch(state, (found) => {
  current.value = found.current
  if (found.current === null) return
  const index = props.rows.findIndex((row) => row.key === found.current)
  if (index >= 0) emit('move', index)
})

const step = (direction: Direction) => {
  current.value = stepMatch(state.value, direction).current
}

const onKeydown = (event: KeyboardEvent) => {
  const action = findAction(event)
  if (action === null) return
  event.preventDefault()
  if (action === FindAction.Close) emit('close')
  else if (action === FindAction.Next) step(Direction.Next)
  else step(Direction.Previous)
}

// A screen with nothing to search says so instead of offering a bar that can never match:
// that is the button-that-does-nothing B14 warns about, wearing a different hat.
const barren = computed(() => props.rows.length === 0)
const searching = computed(() => typed.value.trim() !== '')
const nothingFound = computed(() => searching.value && state.value.total === 0)
</script>

<template>
  <div class="flex items-center gap-2">
    <Input
      v-model="typed"
      :aria-label="t('find.label')"
      :placeholder="t('find.placeholder')"
      :disabled="barren"
      @keydown="onKeydown"
    />
    <span class="text-caption text-faint-foreground">
      <template v-if="barren">{{ t('find.nothingToSearch') }}</template>
      <template v-else-if="nothingFound">{{ t('find.empty') }}</template>
      <template v-else-if="searching">{{
        t('find.count', { position: state.position, total: state.total })
      }}</template>
    </span>
    <Button
      :variant="ButtonVariant.Ghost"
      :size="ButtonSize.Compact"
      :disabled="state.total === 0"
      :aria-label="t('find.previous')"
      @click="step(Direction.Previous)"
    >
      <ChevronUpIcon />
    </Button>
    <Button
      :variant="ButtonVariant.Ghost"
      :size="ButtonSize.Compact"
      :disabled="state.total === 0"
      :aria-label="t('find.next')"
      @click="step(Direction.Next)"
    >
      <ChevronDownIcon />
    </Button>
    <Button
      :variant="ButtonVariant.Ghost"
      :size="ButtonSize.Compact"
      :aria-label="t('find.close')"
      @click="emit('close')"
    >
      <XIcon />
    </Button>
  </div>
</template>
