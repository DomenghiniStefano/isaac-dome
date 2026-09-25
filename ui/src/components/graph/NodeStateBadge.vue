<script setup lang="ts">
import type { Message } from '@/i18n/message'
import { computed } from 'vue'
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { useMessages } from '@/i18n'
import { NodeState, nodeState } from '@/lib/graph/nodeState'
import { nodeWhy } from '@/lib/graph/whyMenu'
import type { UnlockNode } from '@/lib/ipc/types'
import WhyMenu from './WhyMenu.vue'

const props = defineProps<{ node: UnlockNode }>()
const { t } = useMessages()

const state = computed(() => nodeState(props.node))
const groups = computed(() => nodeWhy(props.node, t))

const variant: Record<NodeState, BadgeVariant> = {
  [NodeState.Done]: BadgeVariant.Done,
  [NodeState.Now]: BadgeVariant.Now,
  [NodeState.Blocked]: BadgeVariant.Blocked,
  [NodeState.Partial]: BadgeVariant.Partial,
}

const stateText: Record<NodeState, Message> = {
  [NodeState.Done]: 'graph.state.done',
  [NodeState.Now]: 'graph.state.now',
  [NodeState.Blocked]: 'graph.state.blocked',
  [NodeState.Partial]: 'graph.state.partial',
}

// "requisiti mancanti: 2": the number is the graph's, the words are the messages'.
const label = computed(() =>
  state.value === NodeState.Blocked
    ? t(stateText[state.value], { count: props.node.graph.blockedBy })
    : t(stateText[state.value]),
)
</script>

<template>
  <!-- The badge says the state, its menu says why *and* where to read about it: each thing in
       the way opens its wiki page (DESIGN-BRIEF.md §7.1, spec 3.5d). -->
  <WhyMenu :groups="groups" :label="t('graph.why.title')">
    <Badge
      :variant="variant[state]"
      :tabindex="groups.length > 0 ? 0 : undefined"
      >{{ label }}</Badge
    >
  </WhyMenu>
</template>
