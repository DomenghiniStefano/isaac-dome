<script setup lang="ts">
import { ListPlusIcon } from '@lucide/vue'
import { computed } from 'vue'
import NodeStateBadge from '@/components/graph/NodeStateBadge.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useMessages } from '@/i18n'
import { NodeState, nodeState } from '@/lib/graph/nodeState'
import { unlockEntries } from '@/lib/graph/unlockEntries'
import { nodeWhy } from '@/lib/graph/whyMenu'
import type { UnlockNode } from '@/lib/ipc/types'
import type { TabLocation } from '@/router/routeTable'

const props = defineProps<{
  node: UnlockNode
  queued: boolean
  canAdd: boolean
  busy: boolean
}>()
const emit = defineEmits<{
  add: []
  navigate: [location: TabLocation, newTab: boolean]
}>()
const { t } = useMessages()

// The page has room, so what is in the way is rows and not a menu: the menu exists because a
// badge is small (spec §3.1). Same model as the badge's, same links, same gesture.
const groups = computed(() => nodeWhy(props.node, t))
const gets = computed(() => unlockEntries(props.node, t))
const fanOut = computed(() => props.node.graph.fanOut)

// Only a **blocked** node has steps left before it. Not `!availableNow`: a node that is done
// is not available either, and the number is then 0 — "ancora 0 sblocchi" under a "Già fatto"
// is a sentence about nothing. `partial` carries no count at all, by construction.
const stepsMissing = computed(() => {
  const { graph } = props.node
  if (nodeState(props.node) !== NodeState.Blocked) return null
  return graph.kind === 'computed' && graph.stepsMissing > 0
    ? graph.stepsMissing
    : null
})

// What the node opens, in the tense the node is in: a done achievement opened, it does not
// open. Read under a "Già fatto", the future tense says there is still something to do.
const opensText = computed(() => {
  const count = fanOut.value
  if (props.node.done)
    return count > 0
      ? t('profileBlock.opened', { count })
      : t('profileBlock.openedNothing')
  return count > 0
    ? t('profileBlock.opens', { count })
    : t('profileBlock.opensNothing')
})

const open = (location: TabLocation | null, newTab: boolean) => {
  if (location) emit('navigate', location, newTab)
}
</script>

<template>
  <!-- What the profile knows, and nothing the wiki already says: the infobox below carries
       the condition, the notes and what the wiki thinks it unlocks (spec §2.1). -->
  <Card>
    <CardHeader>
      <CardTitle>{{ t('profileBlock.title') }}</CardTitle>
    </CardHeader>
    <CardContent class="flex flex-col gap-4">
      <NodeStateBadge :node="node" />

      <section v-if="groups.length > 0" class="flex flex-col gap-2">
        <h3 class="text-label text-subtle-foreground">
          {{ t('profileBlock.missing') }}
        </h3>
        <div
          v-for="group in groups"
          :key="group.label"
          class="flex flex-wrap items-baseline gap-x-2 gap-y-1"
        >
          <span class="text-caption text-faint-foreground">{{
            t(group.label)
          }}</span>
          <template v-for="entry in group.entries" :key="entry.key">
            <Button
              v-if="entry.location"
              :variant="ButtonVariant.Ref"
              :size="ButtonSize.Inline"
              @click="open(entry.location, $event.ctrlKey)"
              >{{ entry.name }}</Button
            >
            <!-- Nowhere to read about it: the name shows, and does not pretend to link. -->
            <span v-else class="text-body text-foreground">{{
              entry.name
            }}</span>
          </template>
        </div>
      </section>

      <section v-if="gets.length > 0" class="flex flex-col gap-2">
        <h3 class="text-label text-subtle-foreground">
          {{ t('profileBlock.unlocks') }}
        </h3>
        <!-- Wider than the rows above on purpose: two adjacent link-styled names with a
             small gap read as one name ("Cube of Meat Ball of Bandages"). -->
        <div class="flex flex-wrap items-baseline gap-x-4 gap-y-1">
          <template v-for="entry in gets" :key="entry.key">
            <Button
              v-if="entry.location"
              :variant="ButtonVariant.Ref"
              :size="ButtonSize.Inline"
              @click="open(entry.location, $event.ctrlKey)"
            >
              <img
                v-if="entry.iconUrl"
                :src="entry.iconUrl"
                alt=""
                class="mr-0.75 inline-block size-4 align-text-bottom pixelated"
              />{{ entry.name }}
            </Button>
            <span v-else class="text-body text-foreground">{{
              entry.name
            }}</span>
          </template>
        </div>
      </section>

      <p class="text-body text-subtle-foreground">
        {{ opensText }}
        <span v-if="stepsMissing !== null">{{
          t('profileBlock.stepsMissing', { count: stepsMissing })
        }}</span>
      </p>

      <span v-if="node.done" class="text-caption text-state-done-foreground">{{
        t('profileBlock.done')
      }}</span>
      <span
        v-else-if="queued"
        class="text-caption text-state-done-foreground"
        >{{ t('queue.inPlan') }}</span
      >
      <Button
        v-else-if="canAdd"
        :variant="ButtonVariant.Outline"
        :size="ButtonSize.Compact"
        :disabled="busy"
        class="w-fit"
        @click="emit('add')"
        ><ListPlusIcon />{{ t('queue.add') }}</Button
      >
    </CardContent>
  </Card>
</template>
