<script setup lang="ts">
import { XIcon } from '@lucide/vue'
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { DropSide, TabView } from './tabs'
import { tabOriginIcon } from './tabOriginIcon'

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
       arrows move between tabs, not into them. -->
  <div
    role="tab"
    :aria-selected="active"
    :tabindex="active ? 0 : -1"
    :data-drop="drop ?? undefined"
    :class="
      cn(
        'flex h-tab max-w-tab-max min-w-tab-min flex-1 basis-0 cursor-pointer items-center gap-1.5 border-x-2 border-t-2 border-b-0 border-transparent pr-1.5 pl-2 text-label text-subtle-foreground select-none data-[drop=after]:border-r-selection-edge data-[drop=before]:border-l-selection-edge',
        active && 'border-t-primary bg-sheet text-foreground',
        dragging && 'opacity-disabled',
      )
    "
    @click="emit('select')"
  >
    <component
      :is="icon"
      :class="
        cn(
          'size-3 shrink-0',
          active ? 'text-highlight' : 'text-faint-foreground',
        )
      "
    />
    <span class="min-w-0 flex-1 truncate">{{ tab.label }}</span>
    <Button
      :variant="ButtonVariant.Chrome"
      :size="ButtonSize.Micro"
      :aria-label="t('shell.closeTab')"
      tabindex="-1"
      @pointerdown.stop
      @click.stop="emit('close')"
    >
      <XIcon />
    </Button>
  </div>
</template>
