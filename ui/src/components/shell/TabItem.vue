<script setup lang="ts">
import { XIcon } from '@lucide/vue'
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { DropSide, TabView } from '@/lib/shell/tabs'
import { TabRole } from '@/lib/shell/tabs'
import { tabOriginIcon } from '@/lib/shell/tabOriginIcon'

const props = defineProps<{
  tab: TabView
  active: boolean
  dragging: boolean
  drop: DropSide | null
}>()
const emit = defineEmits<{ select: []; close: [] }>()
const { t } = useMessages()

const icon = computed(() => tabOriginIcon[props.tab.origin])
</script>

<template>
  <!-- Chrome e Stati.dc.html, "Stati della tab". The drop edge sits on the side the tab will
       land on (data-drop), not under the pointer. The close is out of the arrow-key order:
       arrows move between tabs, not into them. A squeezed tab can't hold icon, name and
       close: below tab-narrow an inactive tab drops its close and the active one its icon,
       so nothing spills onto the neighbour. -->
  <div
    :role="TabRole"
    :aria-selected="active"
    :tabindex="active ? 0 : -1"
    :data-drop="drop ?? undefined"
    :class="
      cn(
        '@container/tab flex h-tab max-w-tab-max min-w-tab-min shrink grow-0 basis-tab-max cursor-pointer items-center gap-1.5 overflow-hidden border-x-2 border-t-2 border-b-0 border-transparent pr-1.5 pl-2 text-label text-subtle-foreground select-none tab-intrinsic data-[drop=after]:border-r-selection-edge data-[drop=before]:border-l-selection-edge',
        active && 'border-t-primary bg-sheet text-foreground',
        dragging && 'opacity-disabled',
      )
    "
    @click="emit('select')"
    @mousedown.middle.prevent
    @auxclick.middle="emit('close')"
  >
    <component
      :is="icon"
      :class="
        cn(
          'size-3 shrink-0',
          active
            ? 'text-highlight @max-tab-narrow/tab:hidden'
            : 'text-faint-foreground',
        )
      "
    />
    <span class="min-w-0 flex-1 truncate @max-tab-narrow/tab:hidden">{{
      tab.label
    }}</span>
    <Button
      :variant="ButtonVariant.Chrome"
      :size="ButtonSize.Micro"
      :aria-label="t('shell.closeTab')"
      :class="active ? undefined : '@max-tab-narrow/tab:hidden'"
      tabindex="-1"
      @pointerdown.stop
      @click.stop="emit('close')"
    >
      <XIcon />
    </Button>
  </div>
</template>
