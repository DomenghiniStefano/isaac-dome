<script setup lang="ts">
import { ChevronLeftIcon } from '@lucide/vue'
import { computed } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Kbd, KbdGroup } from '@/components/ui/kbd'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { useMessages } from '@/i18n'
import { KeyName } from '@/lib/constants/keyNames'

// The sidebar's own half of folding it: a tab straddling its inner edge, half way down, where the
// hand already goes to drag the width. Always drawn, never only on hover — a control that appears
// under the pointer is one the person who most needs it never finds.
const props = defineProps<{ collapsed: boolean }>()
const emit = defineEmits<{ toggle: [] }>()
const { t } = useMessages()

const label = computed(() =>
  t(props.collapsed ? 'shell.expandSidebar' : 'shell.collapseSidebar'),
)
</script>

<template>
  <!-- Outside every `sidebar-collapsed-hidden`, or it would vanish at the one moment it is needed
       to open the sidebar again. Under the shell's threshold there is no room to open it into, so
       there the tab goes (card #54: that is the floor, not a broken button). -->
  <Tooltip>
    <TooltipTrigger as-child>
      <Button
        :variant="ButtonVariant.SidebarTab"
        :size="ButtonSize.SidebarTab"
        :aria-label="label"
        :aria-expanded="!collapsed"
        :data-collapsed="collapsed"
        class="group/tab absolute top-1/2 left-full z-10 -translate-x-1/2 -translate-y-1/2 @max-sidebar-room/shell:hidden"
        @click="emit('toggle')"
      >
        <ChevronLeftIcon
          class="transition-transform duration-panel ease-panel group-data-[collapsed=true]/tab:rotate-180"
        />
      </Button>
    </TooltipTrigger>
    <!-- To the right, the side the tab sticks out on: above it the sentence covered the sidebar's
         own edge, which is the thing the tab is about. -->
    <TooltipContent side="right" class="flex items-center gap-2">
      {{ label }}
      <KbdGroup>
        <Kbd>{{ KeyName.Ctrl }}</Kbd>
        <Kbd>{{ KeyName.B }}</Kbd>
      </KbdGroup>
    </TooltipContent>
  </Tooltip>
</template>
