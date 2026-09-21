<script setup lang="ts">
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { AriaCurrent } from '@/lib/constants/aria'

// The label is a **value** and no longer the default slot, because the tooltip needs the same
// text the row shows and a slot cannot be read as one.
//
// The tooltip is mounted always, not only when the sidebar is collapsed: whether it is collapsed
// is a CSS fact (spec 3.13a §6) that JavaScript never learns, so "only when collapsed" would need
// exactly the measurement §2 rejected. It earns its place either way — a label in a sidebar the
// user narrowed is truncated, and wants the tooltip as much as a hidden one does.
defineProps<{ active: boolean; label: string }>()

// **The listener has to be put on the button by hand.** `Tooltip` is this component's root and
// it renders no element of its own, so Vue has nothing to inherit the parent's `@click` onto
// and **drops it without a word** — the sidebar stopped navigating on 2026-09-20 the moment the
// tooltip was wrapped around the button, and nothing failed: not the type checker, not `pnpm
// scan`, not a test, because the repo has no component tests by design. The navbar never broke
// because it binds its click straight onto a `Button`.
//
// `$attrs` and not an explicit `click` emit: a row also carries `aria-*` and whatever a caller
// adds, and forwarding the lot keeps this from being the same bug again the next time something
// is passed down.
defineOptions({ inheritAttrs: false })
</script>

<template>
  <Tooltip>
    <TooltipTrigger as-child>
      <!-- The active item has the red bar on the left (ButtonVariant.Nav). -->
      <Button
        v-bind="$attrs"
        :variant="ButtonVariant.Nav"
        :size="ButtonSize.Row"
        :aria-current="active ? AriaCurrent.Page : undefined"
        class="sidebar-collapsed-center gap-2.25 [&_svg]:size-4 [&_svg]:text-subtle-foreground aria-[current=page]:[&_svg]:text-foreground"
      >
        <slot name="icon" />
        <span class="sidebar-collapsed-hidden truncate">{{ label }}</span>
      </Button>
    </TooltipTrigger>
    <TooltipContent>{{ label }}</TooltipContent>
  </Tooltip>
</template>
