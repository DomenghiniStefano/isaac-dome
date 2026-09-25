<script setup lang="ts">
import { vScrollMemory } from '@/directives/scrollMemory'
import { InfoIcon } from '@lucide/vue'
import { computed, watch } from 'vue'
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import ProfileBlock from '@/components/graph/ProfileBlock.vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import ScreenSkeleton from '@/components/data-state/ScreenSkeleton.vue'
import { SkeletonBlock } from '@/components/data-state/skeletonBlock'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { useMessages } from '@/i18n'
import { achievementNode } from '@/lib/graph/achievementNode'
import { nodeNumber } from '@/lib/graph/achievementNode'
import { canQueue, isQueued } from '@/lib/plan/queueRows'
import { categoryOf } from '@/lib/wiki/category'
import { parsePageKey } from '@/lib/wiki/pageKey'
import { RouteName } from '@/router/routeTable'
import type { WikiCategory } from '@/router/routeTable'
import { useQueueOffer } from '@/composables/useQueueOffer'
import { useTabsStore } from '@/stores/tabs'
import { useGraphStore } from '@/stores/views'
import { useWikiStore } from '@/stores/wiki'
import ProfileError from '../profile/ProfileError.vue'
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
const { queue, queued, canWrite } = useQueueOffer()
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

// A page that would not load is this page's failure, not the wiki's (card #80, R9): the retry
// asks for this page again, and every other tab keeps what it shows.
const retry = () => {
  if (target.value) void wiki.loadEntry(target.value)
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

const canAdd = computed(
  () =>
    canWrite.value && node.value !== null && canQueue(node.value, queued.value),
)
</script>

<template>
  <!-- The gutter is the children's, so the opening band is the full width of the page without
       overflowing it (`WikiLanding.vue` records what the other way round cost). -->
  <div
    v-scroll-memory="'page'"
    class="flex h-full flex-col overflow-y-auto pb-15"
  >
    <WikiHero
      :target="target"
      :title="title"
      :icon="icon"
      :category="category"
      :entry="entry"
      :page-key="pageKey"
      :icon-for="wiki.iconFor"
      :can-open="wiki.hasPage"
      @navigate="tabs.openPage"
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
        @add="queue.add(nodeNumber(node))"
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
      <ProfileError
        v-else-if="wiki.pageFailed(pageKey)"
        :error="wiki.pageError(pageKey)"
        :title="t('wiki.states.pageFailedTitle')"
        @retry="retry"
      />
      <ScreenSkeleton
        v-else-if="entry === undefined"
        untitled
        :blocks="[SkeletonBlock.Card, SkeletonBlock.Card]"
      />
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
            @navigate="tabs.openPage"
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
        <!-- A measure, not a width: wiki prose runs to 90 characters a line at the page's
             full width, which is past what anyone reads comfortably. Capped, the space that
             is left sits between the text and the column beside it. -->
        <div class="max-w-200 min-w-0 flex-1">
          <WikiSections
            :sections="entry.sections"
            :icon-for="wiki.iconFor"
            :can-open="wiki.hasPage"
            @navigate="tabs.openPage"
          />
        </div>
      </div>
    </div>
  </div>
</template>
