<script setup lang="ts">
import { InfoIcon } from '@lucide/vue'
import { computed, watch } from 'vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import ProfileBlock from '@/components/graph/ProfileBlock.vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Skeleton } from '@/components/ui/skeleton'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { useMessages } from '@/i18n'
import { achievementNode } from '@/lib/graph/achievementNode'
import { nodeSlot } from '@/lib/graph/unlockFacets'
import type { Target } from '@/lib/ipc/types'
import { canQueue, isQueued, queuedIds } from '@/lib/plan/queueRows'
import { categoryOf, pageLocation } from '@/lib/wiki/category'
import { parsePageKey } from '@/lib/wiki/pageKey'
import { RouteName } from '@/router/routeTable'
import type { TabLocation, WikiCategory } from '@/router/routeTable'
import { useQueueStore } from '@/stores/queue'
import { useTabsStore } from '@/stores/tabs'
import { useGraphStore } from '@/stores/views'
import { useWikiStore } from '@/stores/wiki'
import WikiHero from './WikiHero.vue'
import WikiInfobox from './WikiInfobox.vue'
import WikiOutline from './WikiOutline.vue'
import WikiSections from './WikiSections.vue'

const props = defineProps<{
  pageKey: string
  category: WikiCategory | null
}>()
const wiki = useWikiStore()
const tabs = useTabsStore()
const graph = useGraphStore()
const queue = useQueueStore()
const { t } = useMessages()

// The key is the tab's; a key that doesn't parse is a page the dataset doesn't know, the
// same state as a valid key with no entry (spec 3.5, Decision 1).
const target = computed(() => parsePageKey(props.pageKey))
watch(
  target,
  (value) => {
    if (value) void wiki.loadEntry(value)
  },
  { immediate: true },
)

// `undefined` while the page is being read; `null` when the dataset lacks it.
const entry = computed(() =>
  target.value === null ? null : wiki.entry(props.pageKey),
)
const unknown = computed(() => entry.value === null)
const category = computed(() =>
  target.value ? categoryOf(target.value) : props.category,
)
const title = computed(
  () =>
    entry.value?.title ??
    wiki.titleOf(props.pageKey) ??
    t('wiki.states.unknownTitle'),
)
const icon = computed(() => (target.value ? wiki.iconFor(target.value) : null))

// A reference replaces the page in this tab, or opens one beside it with Ctrl: the same
// action as opening a search result (DESIGN-BRIEF.md §4.2).
const onNavigate = (next: Target, newTab: boolean) => {
  const location = pageLocation(next)
  if (location === null) return
  if (newTab) tabs.open(location)
  else tabs.navigate(location)
}
const back = () => {
  if (category.value)
    tabs.navigate({ name: RouteName.Wiki, query: { category: category.value } })
}

// The profile's half of an achievement page (spec §3). This screen deliberately does **not**
// load the graph: the wiki is reachable without a profile, and a wiki tab must not pull a
// profile-shaped command. It reads what the progress screens have already put there, and
// `achievementNode` answers `null` for every state where the block would lie.
const node = computed(() =>
  achievementNode(graph.view?.unlock ?? null, target.value),
)
const queued = computed(() => queuedIds(queue.view))
const canAdd = computed(
  () =>
    queue.view?.storeAvailable === true &&
    node.value !== null &&
    canQueue(node.value, queued.value),
)
const onOpen = (location: TabLocation, newTab: boolean) => {
  if (newTab) tabs.open(location)
  else tabs.navigate(location)
}
</script>

<template>
  <!-- The scrolling box takes the shell's gutter back and hands it to its children, so the
       opening band is the full width of the page without overflowing it (`WikiLanding.vue`
       records what the other way round cost). -->
  <div class="-mx-5.5 flex h-full flex-col overflow-y-auto pb-15">
    <WikiHero
      :target="target"
      :title="title"
      :icon="icon"
      :category="category"
      :entry="entry"
      :page-key="pageKey"
      :icon-for="wiki.iconFor"
      :can-open="wiki.hasPage"
      @navigate="onNavigate"
    />
    <div class="flex flex-col gap-5 px-5.5 pt-5">
      <!-- Above the wiki's own answer, and outside it: what the profile knows does not depend
           on the dataset. A page the dataset has never heard of still has a state, still says
           what it unlocks, and can still go in the Plan (spec §3). -->
      <ProfileBlock
        v-if="node"
        :node="node"
        :queued="isQueued(node, queued)"
        :can-add="canAdd"
        :busy="queue.busy"
        @add="queue.add(nodeSlot(node))"
        @navigate="onOpen"
      />
      <template v-if="unknown">
        <EmptyCategory
          >{{ t('wiki.states.unknown') }}
          {{ t('wiki.states.unknownHint') }}</EmptyCategory
        >
        <Button
          v-if="category"
          :variant="ButtonVariant.Outline"
          class="w-fit"
          @click="back"
          >{{ t('wiki.back') }}</Button
        >
      </template>
      <div v-else-if="entry === undefined" class="flex flex-col gap-4">
        <Skeleton class="h-40 w-full" />
        <Skeleton class="h-40 w-full" />
      </div>
      <!-- The card and the index come first in the document and last on a wide page: stacked,
           the facts belong above the prose, and side by side they belong beside it. One
           `flex-row-reverse` says both, where two orders would need two templates. -->
      <div
        v-else-if="entry"
        class="flex flex-col items-start gap-5 @regular/page:flex-row-reverse @regular/page:gap-6"
      >
        <aside
          class="flex w-full flex-col gap-4 @regular/page:sticky @regular/page:top-0 @regular/page:w-wiki-aside @regular/page:shrink-0"
        >
          <WikiInfobox
            :entry="entry"
            :icon-for="wiki.iconFor"
            :can-open="wiki.hasPage"
            @navigate="onNavigate"
          />
          <WikiOutline :sections="entry.sections" />
          <Tooltip>
            <TooltipTrigger as-child>
              <span
                class="flex w-fit cursor-help items-center gap-1.5 text-caption text-subtle-foreground"
                tabindex="0"
                ><InfoIcon class="size-3" />{{
                  t('wiki.provenance.license')
                }}</span
              >
            </TooltipTrigger>
            <TooltipContent class="max-w-80 text-caption text-foreground">{{
              t('wiki.provenance.licenseLong')
            }}</TooltipContent>
          </Tooltip>
        </aside>
        <div class="min-w-0 flex-1">
          <WikiSections
            :sections="entry.sections"
            :icon-for="wiki.iconFor"
            :can-open="wiki.hasPage"
            @navigate="onNavigate"
          />
        </div>
      </div>
    </div>
  </div>
</template>
