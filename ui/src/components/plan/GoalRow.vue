<script setup lang="ts">
import {
  ChevronDownIcon,
  GripVerticalIcon,
  ListPlusIcon,
  XIcon,
} from '@lucide/vue'
import { computed } from 'vue'
import AchievementArt from '@/components/graph/AchievementArt.vue'
import NodeStateBadge from '@/components/graph/NodeStateBadge.vue'
import { ArtSize } from '@/components/graph/artSize'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { UnlockNode } from '@/lib/ipc/types'
import type { QueueExtras } from '@/lib/plan/queueExtras'
import type { RowModel } from '@/lib/plan/rowModel'
import type { TabLocation } from '@/router/routeTable'

const props = defineProps<{
  model: RowModel
  /** For `NodeStateBadge`, which builds the *why* menu from the node's own requirements. */
  node: UnlockNode
  /** Present only in the queue: it brings the grip and the number with it. */
  position?: number
  /** Present only in the queue: the badges the queue around the row adds. */
  extras?: QueueExtras
  /** Whether this row offers to join the queue. Said, never inferred: a row in the want's
      chain has neither `extras` nor a `+`, and the block above it carries the one offer. */
  canAdd?: boolean
  busy: boolean
  dragging?: boolean
}>()
const emit = defineEmits<{
  grab: [e: PointerEvent]
  step: [e: KeyboardEvent]
  remove: []
  add: []
  navigate: [location: TabLocation, newTab: boolean]
}>()
const { t } = useMessages()

const opensText = computed(() =>
  props.model.fanOut > 0
    ? t('plan.opens', { count: props.model.fanOut })
    : t('plan.opensNothing'),
)
const open = (newTab: boolean) => {
  if (props.model.location) emit('navigate', props.model.location, newTab)
}
</script>

<template>
  <!-- Five things in the row — where it sits, what it is, how much it opens — and everything
       else behind the disclosure, opened on the row you are looking at and nowhere else
       (spec §4). Eleven fields between 11px and 13px is what this replaces.
       The row is not itself a link: the text is. A row that is a link and contains buttons is
       a trap for the keyboard, and `Ref` is already the app's way of saying "this goes to a
       page". -->
  <Collapsible
    v-slot="{ open: shown }"
    :class="cn('flex flex-col', dragging && 'opacity-disabled')"
  >
    <div class="flex items-center gap-3 px-3 py-2.5">
      <Button
        v-if="position !== undefined"
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.Icon"
        :aria-label="t('plan.row.move')"
        :disabled="busy"
        class="cursor-grab touch-none"
        @pointerdown="emit('grab', $event)"
        @keydown="emit('step', $event)"
      >
        <GripVerticalIcon />
      </Button>
      <span
        v-if="position !== undefined"
        class="w-4 shrink-0 text-right text-label text-subtle-foreground tabular-nums"
        >{{ position }}</span
      >
      <AchievementArt :url="model.art" :size="ArtSize.Thumb" />
      <!-- The box takes the width; the link does not. `Ref` draws its underline on its own
           bottom border, so a button stretched to the row would underline the whole row and
           centre the text inside it. -->
      <div class="flex min-w-0 flex-1">
        <Button
          v-if="model.location"
          :variant="ButtonVariant.Ref"
          :size="ButtonSize.Inline"
          class="min-w-0 shrink truncate text-row"
          @click="open($event.ctrlKey)"
          >{{ model.text }}</Button
        >
        <span v-else class="min-w-0 truncate text-row text-foreground">{{
          model.text
        }}</span>
      </div>
      <!-- One value carrying two readings (spec §4.1): how much it opens, and in its colour
           whether you can play it tonight. The state is also a word inside, so the colour is
           never the only carrier. -->
      <span
        :class="
          cn(
            'shrink-0 text-caption tabular-nums',
            model.playable
              ? 'text-state-now-foreground'
              : 'text-subtle-foreground',
          )
        "
        >{{ opensText }}</span
      >
      <Button
        v-if="canAdd"
        :variant="ButtonVariant.Outline"
        :size="ButtonSize.IconCompact"
        :aria-label="t('queue.add')"
        :disabled="busy"
        @click="emit('add')"
      >
        <ListPlusIcon />
      </Button>
      <CollapsibleTrigger as-child>
        <Button
          :variant="ButtonVariant.Ghost"
          :size="ButtonSize.IconCompact"
          :aria-label="t('plan.detail')"
        >
          <ChevronDownIcon :class="cn(shown && 'rotate-180')" />
        </Button>
      </CollapsibleTrigger>
    </div>
    <!-- The detail hangs under the text, and the text does not start in the same place in
         the two panes: a proposal has no grip and no position. One padding for both was
         measured wrong in either direction on the Kit page before this was two. -->
    <CollapsibleContent
      :class="
        cn(
          'flex flex-col items-start gap-2 px-3 pb-3',
          position === undefined ? 'pl-row-detail-bare' : 'pl-row-detail',
        )
      "
    >
      <!-- The game's own `unlock_condition`. Absent when the file states none: an invented
           sentence would read exactly like a quoted one. -->
      <span
        v-if="model.condition"
        class="text-caption text-subtle-foreground"
        >{{ model.condition }}</span
      >
      <div class="flex flex-wrap items-center gap-1.5">
        <NodeStateBadge :node="node" />
        <Badge v-if="extras?.wanted" :variant="BadgeVariant.Wanted">{{
          t('plan.row.wanted')
        }}</Badge>
        <Badge
          v-for="s in extras?.serves ?? []"
          :key="s.id"
          :variant="BadgeVariant.Tag"
          >{{ t('plan.row.serves') }} «{{
            s.text ?? `${t('plan.achievement')} ${s.id}`
          }}»</Badge
        >
        <Badge
          v-if="(extras?.stepsNotQueued ?? 0) > 0"
          :variant="BadgeVariant.Tag"
          >{{ t('plan.row.outsideQueue') }}: {{ extras?.stepsNotQueued }}</Badge
        >
      </div>
      <!-- Only a row you asked for can leave: one dragged in goes when its wish does. -->
      <Button
        v-if="extras?.wanted"
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.Compact"
        :disabled="busy"
        @click="emit('remove')"
        ><XIcon />{{ t('queue.removeShort') }}</Button
      >
    </CollapsibleContent>
  </Collapsible>
</template>
