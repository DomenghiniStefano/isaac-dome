<script setup lang="ts">
import { computed, ref } from 'vue'
import { HelpTip } from '@/components/ui/tooltip'
import { useMessages } from '@/i18n'
import { EventKey } from '@/lib/constants/eventKeys'
import { currentFactor } from '@/lib/scale/apply'
import { SidebarWidth, clampSidebarWidth } from '@/lib/shell/sidebarWidth'

defineProps<{ title: string; hint?: string }>()
const width = defineModel<number>('width', { required: true })
const { t } = useMessages()

const resizing = ref<{ startX: number; startWidth: number } | null>(null)
// The width is kept in the pixels the design is drawn in and multiplied by the interface's
// scale here: a sidebar that stayed 212px while the text inside it doubled would cut the
// labels off (cycle 3.5c).
const widthVariable = computed(() => ({
  '--sidebar-width': `calc(${width.value}px * var(--app-scale, 1))`,
}))

const onPointerDown = (e: PointerEvent) => {
  if (e.button !== 0) return
  resizing.value = { startX: e.clientX, startWidth: width.value }
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
}

const onPointerMove = (e: PointerEvent) => {
  const r = resizing.value
  if (!r) return
  // The pointer travels in the screen's pixels, the width is kept in the design's: at 200%
  // an inch of mouse would otherwise move the edge twice as far as the cursor.
  width.value = clampSidebarWidth(
    r.startWidth + (e.clientX - r.startX) / currentFactor(),
  )
}

const onPointerUp = () => {
  resizing.value = null
}

// The arrows move the edge a step; no other key moves it.
const keyStep: Partial<Record<string, number>> = {
  [EventKey.ArrowRight]: SidebarWidth.Step,
  [EventKey.ArrowLeft]: -SidebarWidth.Step,
}

const onKeydown = (e: KeyboardEvent) => {
  const step = keyStep[e.key]
  if (step === undefined) return
  e.preventDefault()
  width.value = clampSidebarWidth(width.value + step)
}
</script>

<template>
  <!-- One sidebar, its content decided by the section (Chrome e Stati.dc.html, "Sidebar di
       sezione"). The width travels as a CSS variable bound here, never an inline pixel. -->
  <!-- Collapsed to its icons when the shell is too narrow for it, or when somebody asked for it
       (spec 3.13a §6). The inline style above sets the *variable*, never the width, so a variant
       class wins by ordinary cascade — which is what lets the collapse be pure CSS, set off by
       the shell's `data-sidebar="collapsed"` and touching nothing else. -->
  <!-- Folding moves in the sheet's five steps (Motion.dc.html), whichever input asked for it. Not
       while the edge is being dragged: there every pixel of the pointer would become a 200ms
       animation, and the edge would trail the cursor instead of sitting under it. -->
  <aside
    :style="widthVariable"
    :data-resizing="resizing !== null"
    class="relative flex w-(--sidebar-width) shrink-0 border border-secondary bg-data transition-[width] duration-sheet ease-sheet group-data-[sidebar=collapsed]/shell:w-sidebar-icons data-[resizing=true]:transition-none @max-sidebar-room/shell:w-sidebar-icons"
  >
    <!-- Clipped, so that while the width folds the edge passes over the labels instead of the
         labels spilling past it onto the page. The column and not the aside: the edge tab hangs
         outside the aside, and clipping it would cut the tab in half. -->
    <div class="flex min-w-0 flex-1 flex-col overflow-hidden">
      <div
        class="flex items-center sidebar-collapsed-center gap-2 px-3 pt-2.5 pb-2"
      >
        <span class="shrink-0 text-highlight [&_svg]:size-3.5"
          ><slot name="icon"
        /></span>
        <!-- Truncated like the entries below, for the few frames an unfolding sidebar is narrower
             than its own title. -->
        <span
          class="sidebar-collapsed-hidden min-w-0 truncate text-caption tracking-caps text-foreground uppercase"
          >{{ title }}</span
        >
        <HelpTip v-if="hint" class="sidebar-collapsed-hidden">{{
          hint
        }}</HelpTip>
      </div>
      <slot />
    </div>
    <div
      role="separator"
      aria-orientation="vertical"
      tabindex="0"
      :aria-label="t('shell.resizeSidebar')"
      :aria-valuemin="SidebarWidth.Min"
      :aria-valuemax="SidebarWidth.Max"
      :aria-valuenow="width"
      :data-resizing="resizing !== null"
      class="sidebar-collapsed-hidden w-1 shrink-0 cursor-col-resize hover:bg-input data-[resizing=true]:bg-input"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
      @dblclick="width = SidebarWidth.Default"
      @keydown="onKeydown"
    />
    <!-- Whatever hangs off the inner edge: the aside is `relative` for it, and it is drawn last
         so it sits over the resize handle it shares the edge with. -->
    <slot name="edge" />
  </aside>
</template>
