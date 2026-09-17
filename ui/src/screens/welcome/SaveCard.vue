<script setup lang="ts">
import { TriangleAlertIcon } from '@lucide/vue'
import { computed } from 'vue'
import { Badge } from '@/components/ui/badge'
import { RadioGroupItem } from '@/components/ui/radio-group'
import { i18n, useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { CandidateView } from '@/lib/ipc/types'
import { candidateSourceLabel } from '@/lib/profile/profileLabels'
import { previewLines, unreadableNote } from '@/lib/profile/previewView'
import { editionShort, formatModified } from '@/lib/profile/profileView'

const props = defineProps<{ candidate: CandidateView; selected: boolean }>()
const { t } = useMessages()

const edition = computed(() => editionShort(props.candidate.prefix))
const modified = computed(
  () =>
    formatModified(
      props.candidate.modifiedUnix,
      new Date(),
      i18n.global.locale.value,
    ) ?? t('welcome.card.never'),
)
// Both the lines and the note are decided in `previewView`, where they are tested: a count
// that could not be read draws a dash, never a zero.
const lines = computed(() => previewLines(props.candidate.preview))
const note = computed(() => unreadableNote(props.candidate.preview))
</script>

<template>
  <!-- A label around the radio: the card is the target, and the choice stays a real radio
       group rather than a div that listens for clicks. -->
  <label
    :class="
      cn(
        'flex cursor-pointer flex-col gap-3 border bg-sheet p-4',
        selected ? 'border-selection-edge' : 'border-hairline',
      )
    "
  >
    <span class="flex items-start justify-between gap-2">
      <span class="flex items-center gap-2">
        <RadioGroupItem
          :value="candidate.id"
          :aria-label="`${edition} ${candidate.slot}`"
        />
        <span class="flex flex-col">
          <span class="text-body">{{ edition }}</span>
          <span class="text-caption text-subtle-foreground"
            >{{ t('welcome.card.slot') }} {{ candidate.slot }} ·
            {{ t(candidateSourceLabel[candidate.source]) }}</span
          >
        </span>
      </span>
      <Badge v-if="candidate.suggested">{{
        t('welcome.card.suggested')
      }}</Badge>
    </span>

    <span class="flex flex-col gap-1">
      <span
        v-for="line in lines"
        :key="line.label"
        class="flex items-baseline justify-between gap-3"
      >
        <span class="text-caption text-subtle-foreground">{{
          t(line.label)
        }}</span>
        <span
          v-if="line.done === null"
          class="text-caption text-foreground-soft"
          >{{ t('welcome.card.unread') }}</span
        >
        <span v-else class="text-body tabular-nums"
          >{{ line.done
          }}<span class="text-subtle-foreground">/{{ line.of }}</span></span
        >
      </span>
    </span>

    <span class="flex items-center justify-between gap-2">
      <span class="text-caption text-foreground-soft">{{ modified }}</span>
      <span
        v-if="note"
        class="flex items-center gap-1 text-caption text-destructive-foreground"
      >
        <TriangleAlertIcon class="size-3.5" />
        {{ t(note.key, { count: note.count }) }}
      </span>
    </span>
  </label>
</template>
