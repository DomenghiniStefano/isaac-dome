<script setup lang="ts">
import { computed, ref } from 'vue'
import { useMessages } from '@/i18n'
import { EventKey } from '@/lib/constants/eventKeys'
import { currentFactor } from '@/lib/scale/apply'
import { SidebarWidth, clampSidebarWidth } from './sidebarWidth'

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

const onKeydown = (e: KeyboardEvent) => {
  const step =
    e.key === EventKey.ArrowRight
      ? SidebarWidth.Step
      : e.key === EventKey.ArrowLeft
        ? -SidebarWidth.Step
        : 0
  if (step === 0) return
  e.preventDefault()
  width.value = clampSidebarWidth(width.value + step)
}
</script>

<template>
  <!-- One sidebar, its content decided by the section (Chrome e Stati.dc.html, "Sidebar di
       sezione"). The width travels as a CSS variable bound here, never an inline pixel. -->
  <aside
    :style="widthVariable"
    class="flex w-(--sidebar-width) shrink-0 border border-secondary bg-data"
  >
    <div class="flex min-w-0 flex-1 flex-col">
      <div class="flex items-center gap-2 px-2.75 pt-2.5 pb-1.5">
        <span class="text-highlight [&_svg]:size-3.5"
          ><slot name="icon"
        /></span>
        <span class="text-caption tracking-caps text-foreground uppercase">{{
          title
        }}</span>
      </div>
      <p v-if="hint" class="px-2.75 pb-2.25 text-label text-faint-foreground">
        {{ hint }}
      </p>
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
      class="w-1.25 shrink-0 cursor-col-resize hover:bg-input data-[resizing=true]:bg-input"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
      @dblclick="width = SidebarWidth.Default"
      @keydown="onKeydown"
    />
  </aside>
</template>
