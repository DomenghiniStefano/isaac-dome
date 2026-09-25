<script setup lang="ts">
import { computed } from 'vue'
import PixelSprite from '@/components/sprite/PixelSprite.vue'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { useGestureModifiers } from '@/composables/useGestureModifiers'
import { useMessages } from '@/i18n'
import type { Target } from '@/lib/ipc/types'
import { pageLocation } from '@/lib/wiki/category'
import { useTabsStore } from '@/stores/tabs'

// One thing in a run — an item, an achievement — with its picture, its name, what the app
// knows about it before you click, and the page it opens.
//
// The tooltip says what a click cannot: the id the log carried, and for an achievement the
// game's own line about how it is earned. A chip with no page stays a chip: it does not
// pretend to be a link, because a link that goes nowhere is worse than plain text.
const props = defineProps<{
  target: Target | null
  name: string
  /** What the app knows beyond the name: an id, a condition, both. */
  detail?: string | null
  iconUrl: string | null
  /** The active actually being carried, which a run can hold exactly one of. */
  highlighted?: boolean
}>()
const { t } = useMessages()
const tabs = useTabsStore()
const { ctrl } = useGestureModifiers()

const location = computed(() =>
  props.target === null ? null : pageLocation(props.target),
)

// The app's one gesture: a click navigates the active tab, Ctrl opens the page beside it.
const open = () => {
  const to = location.value
  if (to) tabs.go(to, ctrl.value)
}
</script>

<template>
  <Tooltip>
    <TooltipTrigger as-child>
      <component
        :is="location === null ? 'span' : 'button'"
        :type="location === null ? undefined : 'button'"
        class="flex items-center gap-1.5 rounded-input border px-2 py-1 text-left"
        :class="[
          highlighted ? 'border-highlight' : 'border-hairline',
          location === null ? undefined : 'hover:bg-row-hover',
        ]"
        @click="open"
      >
        <PixelSprite
          :url="iconUrl"
          placeholder
          class="size-icon-compact shrink-0"
        />
        <span class="text-label">{{ name }}</span>
      </component>
    </TooltipTrigger>
    <TooltipContent>
      <span class="flex flex-col">
        <span>{{ name }}</span>
        <span v-if="detail" class="text-subtle-foreground">{{ detail }}</span>
        <span v-if="location !== null" class="text-subtle-foreground">{{
          t('runs.openPage')
        }}</span>
      </span>
    </TooltipContent>
  </Tooltip>
</template>
