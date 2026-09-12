<script setup lang="ts">
import { computed } from 'vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { useMessages } from '@/i18n'
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import {
  NodeState,
  RequirementKind,
  missingGroups,
  nodeState,
} from '@/lib/graph/nodeState'
import type { UnlockNode } from '@/lib/ipc/types'

const props = defineProps<{ node: UnlockNode }>()
const { t } = useMessages()

const state = computed(() => nodeState(props.node))
const groups = computed(() => missingGroups(props.node, t))

const variant: Record<NodeState, BadgeVariant> = {
  [NodeState.Done]: BadgeVariant.Done,
  [NodeState.Now]: BadgeVariant.Now,
  [NodeState.Blocked]: BadgeVariant.Blocked,
  [NodeState.Partial]: BadgeVariant.Partial,
}

const stateText: Record<NodeState, MessageKey<MessageSchema>> = {
  [NodeState.Done]: 'graph.state.done',
  [NodeState.Now]: 'graph.state.now',
  [NodeState.Blocked]: 'graph.state.blocked',
  [NodeState.Partial]: 'graph.state.partial',
}

const kindText: Record<RequirementKind, MessageKey<MessageSchema>> = {
  [RequirementKind.Character]: 'graph.why.character',
  [RequirementKind.Boss]: 'graph.why.boss',
  [RequirementKind.Challenge]: 'graph.why.challenge',
  [RequirementKind.Item]: 'graph.why.item',
  [RequirementKind.Gate]: 'graph.why.gate',
  [RequirementKind.Unknown]: 'graph.why.unknown',
}

// "bloccato da 2": the number is the graph's, the words are the messages'.
const label = computed(() =>
  state.value === NodeState.Blocked
    ? `${t(stateText[state.value])} ${props.node.graph.blockedBy}`
    : t(stateText[state.value]),
)
</script>

<template>
  <!-- The badge says the state, its tooltip says why: what stands in the way, by kind, as
       DESIGN-BRIEF.md §7.1 asks ("1 character and 2 bosses", not "blocked by 3"). -->
  <Tooltip :disabled="groups.length === 0">
    <TooltipTrigger as-child>
      <Badge
        :variant="variant[state]"
        :tabindex="groups.length > 0 ? 0 : undefined"
        >{{ label }}</Badge
      >
    </TooltipTrigger>
    <TooltipContent class="flex max-w-80 flex-col gap-1.5">
      <span class="text-caption text-foreground">{{
        t('graph.why.title')
      }}</span>
      <div
        v-for="group in groups"
        :key="group.kind"
        class="flex flex-col gap-0.5"
      >
        <span class="text-label text-subtle-foreground">{{
          t(kindText[group.kind])
        }}</span>
        <span class="text-caption text-foreground-soft">{{
          group.names.join(', ')
        }}</span>
      </div>
    </TooltipContent>
  </Tooltip>
</template>
