# Design system cycle 2 — app components: Implementation Plan

> **For agentic workers:** executed inline with superpowers:executing-plans (the owner
> declined the one-subagent-per-task loop). Steps use checkbox (`- [ ]`) syntax for
> tracking.

**Goal:** the components `Chrome e Stati.dc.html` draws and shadcn doesn't have — title
bar, navbar, section sidebar, KPI tile, matrix cell, wiki tokens and blocks, data states,
collapsible card — dressed, presentational, and shown on the Kit page.

**Architecture:** domain folders under `ui/src/components/` (`shell/`, `marks/`, `kpi/`,
`wiki/`, `data-state/`); each decision a component makes is a pure function beside it,
tested first; everything a component needs from outside arrives as a prop, everything it
reports leaves as an emit. Patterns the primitives don't cover become `ButtonSize` /
`ButtonVariant` entries and Card parts.

**Tech Stack:** Vue 3.5 `<script setup>`, TypeScript 6, Tailwind v4.3 (`@theme`), Reka UI
2.10.4, `@lucide/vue` 1.44, vue-i18n 11.4, Vitest 5, tailwind-merge 3.6.

**Spec:** `docs/superpowers/specs/2026-09-10-design-system-components-design.md`

## Global Constraints

- Presentational only: no store, no router, no `invoke()`, no Tauri window API.
- No IPC change: `Cell`, `Inline`, `Block`, `Target`, `Dlc` as they are in `ui/src/lib/ipc/types.ts`.
- No game asset reaches the production build; mark sprites exist only behind `import.meta.env.DEV` (Kit page).
- No `<style>`; no arbitrary pixel class; no hardcoded opacity or duration; no raw `<button>`/`<input>` outside `src/components/ui/`; no string unions (`as const` objects).
- Every string a component renders itself goes through `useMessages()`; consumer strings arrive as props.
- Gold means "unlockable now"; cyan means focus; a state colour is never reused.
- Commits: Conventional Commits, English, no `Co-Authored-By`, at logical boundaries.
- Before done: `pnpm check` green, and the production build has no Kit module and no mark sprite.

---

### Task 1: Tokens, and `cn()` learns letter spacing

**Files:**
- Modify: `ui/src/assets/theme/colors.css`, `ui/src/assets/theme/spacing.css`, `ui/src/assets/theme/typography.css`
- Modify: `ui/src/lib/design/themeKeys.ts`, `ui/src/lib/cn.ts`
- Test: `ui/src/lib/design/themeKeys.test.ts`, `ui/src/lib/cn.test.ts`

**Interfaces:**
- Produces: colour tokens `titlebar`, `navbar`, `chrome-inactive-foreground`, `mark-paper`; spacing tokens `titlebar`, `tab`, `tab-min`, `tab-max`, `drag-region`, `window-control`, `navbar`, `brand`, `search`, `mark-cell`, `mark-symbol`; `text-kpi`; `tracking-nav`, `tracking-caps`; `ThemeNamespace.Tracking`.

- [ ] **Step 1: Write the failing tests**

In `ui/src/lib/cn.test.ts`, before `it('drops falsy inputs'`:

```ts
  it('lets a later letter spacing replace an earlier one', () => {
    expect(cn('tracking-nav', 'tracking-caps')).toBe('tracking-caps')
  })
```

In `ui/src/lib/design/themeKeys.test.ts`, replace the body of
`it('reads the real theme files as source, not as compiled CSS'` with:

```ts
    expect(typography).toContain('@theme')
    expect(themeKeys(typography, ThemeNamespace.Text)).toEqual([
      'title',
      'kpi',
      'heading',
      'body',
      'control',
      'row',
      'caption',
      'label',
      'micro',
    ])
    expect(themeKeys(typography, ThemeNamespace.Tracking)).toEqual([
      'nav',
      'caps',
    ])
    expect(themeKeys(motion, ThemeNamespace.TransitionDuration)).toEqual([
      'tap',
      'panel',
      'sheet',
      'loop',
    ])
```

- [ ] **Step 2: Run them to see them fail**

Run: `pnpm --filter ui exec vitest run src/lib`
Expected: FAIL — `ThemeNamespace.Tracking` is undefined (type error at run: `--undefined-`
matches nothing, `[]` ≠ `['nav','caps']`), the text list lacks `kpi`, and `cn` keeps both
tracking classes.

- [ ] **Step 3: Add the tokens and the namespace**

`ui/src/assets/theme/colors.css`, after `--color-tooltip: #492e20;`:

```css
  /* The window's chrome (Chrome e Stati.dc.html, "Finestra"). */
  --color-titlebar: #0d0908;
  --color-navbar: #120c0b;
  --color-chrome-inactive-foreground: #5a4a44;
  /* The paper a marked matrix cell is drawn on: the pixel at (42, 42) of the game's
     completion_widget/paper_00.png, sampled through a canvas on 2026-09-10. */
  --color-mark-paper: #e9dadf;
```

`ui/src/assets/theme/spacing.css`, before the closing `}`:

```css
  /* The window (Chrome e Stati.dc.html, "Finestra"): tab strip, tabs, drag region,
     window controls, navbar. */
  --spacing-titlebar: 30px;
  --spacing-tab: 24px;
  --spacing-tab-min: 34px;
  --spacing-tab-max: 150px;
  --spacing-drag-region: 130px;
  --spacing-window-control: 38px;
  --spacing-navbar: 38px;
  --spacing-brand: 150px;
  --spacing-search: 240px;
  /* The completion matrix: a 16px mark symbol at 2x in a 40px cell (cycle 2, Decision 7). */
  --spacing-mark-cell: 40px;
  --spacing-mark-symbol: 32px;
```

`ui/src/assets/theme/typography.css`: after `--text-title--line-height: 1.3;` add

```css
  /* A KPI's number (Chrome e Stati.dc.html, "KPI"). */
  --text-kpi: 26px;
  --text-kpi--line-height: 1;
```

and before the closing `}` of `@theme` add

```css

  /* Letter spacing: the navbar's sections, the sidebar's uppercase headings. */
  --tracking-*: initial;
  --tracking-nav: 0.04em;
  --tracking-caps: 0.07em;
```

`ui/src/lib/design/themeKeys.ts`, in `ThemeNamespace` after `Font: 'font',`:

```ts
  Tracking: 'tracking',
```

`ui/src/lib/cn.ts`, in `extend.theme` after the `font:` line:

```ts
      tracking: themeKeys(typography, ThemeNamespace.Tracking),
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `pnpm --filter ui exec vitest run src/lib`
Expected: PASS (all `cn` and `themeKeys` tests).

- [ ] **Step 5: Commit**

```bash
git add ui/src/assets/theme ui/src/lib/design ui/src/lib/cn.ts ui/src/lib/cn.test.ts
git commit -m "feat(ui): the tokens of the window, the KPI and the matrix cell"
```

---

### Task 2: Button's new sizes and variants, and the collapsible card

**Files:**
- Modify: `ui/src/components/ui/button/variants.ts`
- Create: `ui/src/components/ui/card/CardCollapsible.vue`, `CardCollapsibleTrigger.vue`, `CardCollapsibleContent.vue`
- Modify: `ui/src/components/ui/card/index.ts`

**Interfaces:**
- Produces: `ButtonVariant.Nav | Section | Ref | Chrome | ChromeDanger | Field`; `ButtonSize.Micro | Row | Inline | Window | Compact`; `CardCollapsible` (Reka `CollapsibleRootProps`), `CardCollapsibleTrigger` (default slot = title, `summary` slot), `CardCollapsibleContent`.

- [ ] **Step 1: Replace `ui/src/components/ui/button/variants.ts`**

```ts
import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const ButtonVariant = {
  Default: 'default',
  Secondary: 'secondary',
  Outline: 'outline',
  Ghost: 'ghost',
  Link: 'link',
  // Cycle 2: the chrome and the wiki (Chrome e Stati.dc.html).
  Nav: 'nav',
  Section: 'section',
  Ref: 'ref',
  Chrome: 'chrome',
  ChromeDanger: 'chromeDanger',
  Field: 'field',
} as const
export type ButtonVariant = (typeof ButtonVariant)[keyof typeof ButtonVariant]

export const ButtonSize = {
  Default: 'default',
  Icon: 'icon',
  Micro: 'micro',
  Row: 'row',
  Inline: 'inline',
  Window: 'window',
  Compact: 'compact',
} as const
export type ButtonSize = (typeof ButtonSize)[keyof typeof ButtonSize]

// Shadcn Kit.dc.html, "Button": a flat edge, no radius, and no transition — hover and
// active change colour at once (Motion.dc.html, 0ms). Disabled is its own surface, not an
// opacity. Sizes only size and variants only colour: cva writes the size's classes after
// the variant's, so a height in a variant would lose.
export const buttonVariants = cva(
  'inline-flex shrink-0 cursor-pointer items-center justify-center gap-2 border text-control whitespace-nowrap select-none disabled:pointer-events-none disabled:border-secondary disabled:bg-muted disabled:text-faint-foreground [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*=size-])]:size-4',
  {
    variants: {
      variant: {
        [ButtonVariant.Default]:
          'border-primary-edge bg-primary text-primary-foreground hover:bg-primary-hover active:bg-primary-active',
        [ButtonVariant.Secondary]:
          'border-secondary-edge bg-secondary text-secondary-foreground hover:border-input hover:bg-secondary-hover active:bg-band',
        [ButtonVariant.Outline]:
          'border-input bg-transparent text-foreground hover:bg-secondary active:bg-band',
        [ButtonVariant.Ghost]:
          'border-transparent bg-transparent text-foreground hover:border-secondary-edge hover:bg-secondary active:bg-band',
        [ButtonVariant.Link]:
          'border-transparent bg-transparent text-highlight underline disabled:border-transparent disabled:bg-transparent',
        // A sidebar item: the red bar on the left says which one, the fill alone doesn't
        // on an already dark row.
        [ButtonVariant.Nav]:
          'justify-start border-0 border-l-3 border-transparent bg-transparent text-foreground-soft hover:bg-secondary aria-[current=page]:border-selection-edge aria-[current=page]:bg-primary aria-[current=page]:text-foreground',
        // A navbar section: a cream top edge on the data surface.
        [ButtonVariant.Section]:
          'border-0 border-t-2 border-transparent bg-transparent tracking-nav text-subtle-foreground hover:text-foreground aria-[current=page]:border-highlight aria-[current=page]:bg-data aria-[current=page]:text-highlight',
        // A resolved wiki reference: cream over a solid underline edge.
        [ButtonVariant.Ref]:
          'border-0 border-b border-secondary-edge bg-transparent text-highlight hover:border-highlight',
        [ButtonVariant.Chrome]:
          'border-transparent bg-transparent text-muted-foreground hover:bg-secondary',
        [ButtonVariant.ChromeDanger]:
          'border-transparent bg-transparent text-muted-foreground hover:bg-primary hover:text-primary-foreground',
        // Looks like a field, opens something else: the navbar's search trigger.
        [ButtonVariant.Field]:
          'justify-start border-secondary bg-data text-faint-foreground hover:border-input',
      },
      size: {
        [ButtonSize.Default]: 'h-control px-4',
        [ButtonSize.Icon]: 'size-control',
        [ButtonSize.Micro]:
          'size-3.5 border-0 p-0 [&_svg:not([class*=size-])]:size-2',
        [ButtonSize.Row]:
          'h-auto w-full justify-start px-2.75 py-1.75 text-row',
        [ButtonSize.Inline]: 'h-auto gap-1 p-0 align-baseline text-row',
        [ButtonSize.Window]: 'h-full w-window-control border-0',
        [ButtonSize.Compact]: 'h-7 gap-1.75 px-2.25 text-caption',
      },
    },
    compoundVariants: [
      { variant: ButtonVariant.Link, size: ButtonSize.Default, class: 'px-1' },
    ],
    defaultVariants: {
      variant: ButtonVariant.Default,
      size: ButtonSize.Default,
    },
  },
)
export type ButtonVariants = VariantProps<typeof buttonVariants>
```

- [ ] **Step 2: Create `ui/src/components/ui/card/CardCollapsible.vue`**

```vue
<script setup lang="ts">
import type { CollapsibleRootEmits, CollapsibleRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { CollapsibleRoot, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  CollapsibleRootProps & { class?: HTMLAttributes['class'] }
>()
const emits = defineEmits<CollapsibleRootEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <!-- The configuration screens' brick (Chrome e Stati.dc.html, "Card comprimibile"). Parts,
       not a prop on Card: a prop can't turn a div into Reka's collapsible root. -->
  <CollapsibleRoot
    v-slot="slotProps"
    data-slot="card-collapsible"
    v-bind="forwarded"
    :class="cn('flex flex-col border border-border bg-sheet', props.class)"
  >
    <slot v-bind="slotProps" />
  </CollapsibleRoot>
</template>
```

- [ ] **Step 3: Create `ui/src/components/ui/card/CardCollapsibleTrigger.vue`**

```vue
<script setup lang="ts">
import type { CollapsibleTriggerProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { ChevronRightIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import { CollapsibleTrigger } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  CollapsibleTriggerProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
</script>

<template>
  <!-- The whole header is the trigger. The chevron's turn is the only motion; the summary
       stays on the right, so a closed card still says something. -->
  <CollapsibleTrigger
    data-slot="card-collapsible-trigger"
    v-bind="delegatedProps"
    :class="
      cn(
        'group flex w-full cursor-pointer items-center gap-2.25 bg-muted px-3 py-2.25 text-left text-caption text-foreground-soft data-[state=open]:border-b data-[state=open]:border-secondary data-[state=open]:text-highlight',
        props.class,
      )
    "
  >
    <ChevronRightIcon
      class="size-3 shrink-0 text-subtle-foreground transition-transform duration-panel ease-panel group-data-[state=open]:rotate-90 group-data-[state=open]:text-highlight"
    />
    <span class="flex-1"><slot /></span>
    <span class="text-label text-subtle-foreground"><slot name="summary" /></span>
  </CollapsibleTrigger>
</template>
```

- [ ] **Step 4: Create `ui/src/components/ui/card/CardCollapsibleContent.vue`**

```vue
<script setup lang="ts">
import type { CollapsibleContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { CollapsibleContent } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  CollapsibleContentProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
</script>

<template>
  <CollapsibleContent
    data-slot="card-collapsible-content"
    v-bind="delegatedProps"
    :class="
      cn(
        'animate-panel-open px-3 py-2.75 text-caption text-foreground-soft',
        props.class,
      )
    "
  >
    <slot />
  </CollapsibleContent>
</template>
```

- [ ] **Step 5: Export the parts** — `ui/src/components/ui/card/index.ts` becomes

```ts
export { default as Card } from './Card.vue'
export { default as CardAction } from './CardAction.vue'
export { default as CardCollapsible } from './CardCollapsible.vue'
export { default as CardCollapsibleContent } from './CardCollapsibleContent.vue'
export { default as CardCollapsibleTrigger } from './CardCollapsibleTrigger.vue'
export { default as CardContent } from './CardContent.vue'
export { default as CardFooter } from './CardFooter.vue'
export { default as CardHeader } from './CardHeader.vue'
export { default as CardTitle } from './CardTitle.vue'
```

- [ ] **Step 6: Verify** — Run: `pnpm typecheck && pnpm lint && pnpm scan`. Expected: 0 errors, 0 violations.

- [ ] **Step 7: Commit**

```bash
git add ui/src/components/ui/button/variants.ts ui/src/components/ui/card
git commit -m "feat(ui): Button learns the chrome's sizes and variants, Card a collapsible form"
```

---

### Task 3: The shell's pure logic

**Files:**
- Create: `ui/src/components/shell/tabs.ts`, `ui/src/components/shell/sidebarWidth.ts`, `ui/src/components/shell/navSection.ts`, `ui/src/lib/constants/eventKeys.ts`
- Test: `ui/src/components/shell/tabs.test.ts`, `ui/src/components/shell/sidebarWidth.test.ts`

**Interfaces:**
- Produces: `TabOrigin`, `TabView { id: string; label: string; origin: TabOrigin }`, `DropSide`, `TabDrag.Threshold`, `dropSide(pointerX: number, left: number, width: number): DropSide`, `moveIndex(from: number, target: number, side: DropSide): number`; `SidebarWidth { Min: 168, Default: 212, Max: 420, Step: 16 }`, `clampSidebarWidth(px: number): number`; `NavSection`; `EventKey { ArrowLeft, ArrowRight }`.

- [ ] **Step 1: Write the failing tests**

`ui/src/components/shell/tabs.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { DropSide, dropSide, moveIndex } from './tabs'

describe('dropSide', () => {
  it('lands before a tab when the pointer is on its left half', () => {
    expect(dropSide(110, 100, 40)).toBe(DropSide.Before)
  })

  it('lands after a tab when the pointer is on its right half', () => {
    expect(dropSide(135, 100, 40)).toBe(DropSide.After)
  })

  it('counts the exact middle as after', () => {
    expect(dropSide(120, 100, 40)).toBe(DropSide.After)
  })
})

describe('moveIndex', () => {
  it('moves a tab right, after the target', () => {
    expect(moveIndex(0, 2, DropSide.After)).toBe(2)
  })

  it('moves a tab right, before the target', () => {
    expect(moveIndex(0, 2, DropSide.Before)).toBe(1)
  })

  it('moves a tab left, before the target', () => {
    expect(moveIndex(3, 1, DropSide.Before)).toBe(1)
  })

  it('moves a tab left, after the target', () => {
    expect(moveIndex(3, 1, DropSide.After)).toBe(2)
  })

  it('leaves a tab dropped on itself where it is', () => {
    expect(moveIndex(2, 2, DropSide.Before)).toBe(2)
    expect(moveIndex(2, 2, DropSide.After)).toBe(2)
  })
})
```

`ui/src/components/shell/sidebarWidth.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { clampSidebarWidth } from './sidebarWidth'

describe('clampSidebarWidth', () => {
  it('raises a width below the minimum to the minimum', () => {
    expect(clampSidebarWidth(100)).toBe(168)
  })

  it('keeps the minimum itself', () => {
    expect(clampSidebarWidth(168)).toBe(168)
  })

  it('keeps a width inside the bounds', () => {
    expect(clampSidebarWidth(300)).toBe(300)
  })

  it('lowers a width above the maximum to the maximum', () => {
    expect(clampSidebarWidth(500)).toBe(420)
  })

  it('falls back to the default for a width that is not a number', () => {
    expect(clampSidebarWidth(Number.NaN)).toBe(212)
  })

  it('rounds to a whole pixel', () => {
    expect(clampSidebarWidth(250.6)).toBe(251)
  })
})
```

- [ ] **Step 2: Run them to see them fail**

Run: `pnpm --filter ui exec vitest run src/components/shell`
Expected: FAIL — `Failed to resolve import "./tabs"` / `"./sidebarWidth"`.

- [ ] **Step 3: Implement**

`ui/src/components/shell/tabs.ts`:

```ts
// A tab's origin, drawn as an icon on the tab itself (DESIGN-BRIEF.md §4.2): the mixed bar
// stays readable because every tab says which part of the app it comes from.
export const TabOrigin = {
  Wiki: 'wiki',
  Progress: 'progress',
  Settings: 'settings',
  About: 'about',
} as const
export type TabOrigin = (typeof TabOrigin)[keyof typeof TabOrigin]

// What a tab shows. A view type of the shell, not an IPC one: cycle 3 builds it from the
// tab model.
export interface TabView {
  id: string
  label: string
  origin: TabOrigin
}

export const DropSide = { Before: 'before', After: 'after' } as const
export type DropSide = (typeof DropSide)[keyof typeof DropSide]

// Pixels the pointer travels before a press on a tab becomes a drag: below it, a click.
export const TabDrag = { Threshold: 4 } as const

// Which side of the tab under the pointer a dropped tab lands on: the half decides, and
// the exact middle counts as after.
export const dropSide = (
  pointerX: number,
  left: number,
  width: number,
): DropSide => (pointerX < left + width / 2 ? DropSide.Before : DropSide.After)

// The index a tab ends at when moved from `from` to one side of the tab at `target`. Both
// are positions before the move; taking the tab out first shifts everything after it left.
export const moveIndex = (
  from: number,
  target: number,
  side: DropSide,
): number => {
  if (from === target) return from
  const slot = side === DropSide.Before ? target : target + 1
  return slot > from ? slot - 1 : slot
}
```

`ui/src/components/shell/sidebarWidth.ts`:

```ts
import { clamp } from 'lodash-es'

// The section sidebar's width in pixels (Chrome e Stati.dc.html, "Sidebar di sezione").
// Not a theme token: the width moves at runtime, and its bounds belong to the one function
// that enforces them. The template binds the result as a CSS variable.
export const SidebarWidth = {
  Min: 168,
  Default: 212,
  Max: 420,
  Step: 16,
} as const

// A whole pixel, because a pixel font on half pixels blurs; the default when the input
// isn't a usable number at all.
export const clampSidebarWidth = (px: number): number =>
  Number.isFinite(px)
    ? clamp(Math.round(px), SidebarWidth.Min, SidebarWidth.Max)
    : SidebarWidth.Default
```

`ui/src/components/shell/navSection.ts`:

```ts
// The navbar's two sections (DESIGN-BRIEF.md §4): two preconditions, two places.
export const NavSection = { Wiki: 'wiki', Progress: 'progress' } as const
export type NavSection = (typeof NavSection)[keyof typeof NavSection]
```

`ui/src/lib/constants/eventKeys.ts`:

```ts
// KeyboardEvent.key values components react to. Not key caps (those are KeyName, shown
// to the user): these are what the browser reports, and never displayed.
export const EventKey = {
  ArrowLeft: 'ArrowLeft',
  ArrowRight: 'ArrowRight',
} as const
export type EventKey = (typeof EventKey)[keyof typeof EventKey]
```

- [ ] **Step 4: Run the tests to see them pass**

Run: `pnpm --filter ui exec vitest run src/components/shell`
Expected: PASS (11 tests).

(No commit yet: the shell's logic and components land together in Task 4.)

---

### Task 4: Title bar, navbar, section sidebar

**Files:**
- Create: `ui/src/components/shell/tabOriginIcon.ts`, `TabItem.vue`, `TabStrip.vue`, `WindowControls.vue`, `TitleBar.vue`, `NavBar.vue`, `SectionSidebar.vue`, `SidebarItem.vue`
- Create: `ui/src/lib/constants/app.ts`
- Modify: `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts`

**Interfaces:**
- Consumes: Task 2's `ButtonVariant.Chrome | ChromeDanger | Section | Field | Nav`, `ButtonSize.Micro | Icon | Window | Compact | Row`; Task 3's `TabView`, `TabOrigin`, `DropSide`, `TabDrag`, `dropSide`, `moveIndex`, `SidebarWidth`, `clampSidebarWidth`, `NavSection`, `EventKey`.
- Produces: `TitleBar` props `{ tabs: TabView[]; activeId: string | null; focused: boolean }`, emits `select(id) close(id) move(from,to) add minimize toggleMaximize closeWindow`; `NavBar` props `{ section: NavSection; focused: boolean }`, emits `update:section search settings about`; `SectionSidebar` props `{ title: string; hint?: string }`, `v-model:width`, slots `icon`, default; `SidebarItem` props `{ active: boolean }`, slots `icon`, default.

- [ ] **Step 1: Messages** — `ui/src/i18n/messages/it.ts` becomes

```ts
// Italian is the schema: en.ts is typed against it, so a key missing from English is a
// compile error, not a review note. Item, character and boss names are data, not
// messages: they stay in English and never appear here.
export const it = {
  ui: {
    close: 'Chiudi',
  },
  shell: {
    newTab: 'Nuova tab',
    closeTab: 'Chiudi tab',
    minimize: 'Riduci a icona',
    maximize: 'Ingrandisci',
    closeWindow: 'Chiudi finestra',
    search: 'Cerca in tutto',
    settings: 'Impostazioni',
    about: 'Informazioni',
    resizeSidebar: 'Ridimensiona la barra laterale',
    sections: {
      wiki: 'Wiki',
      progress: 'Progressi',
    },
  },
}

export type MessageSchema = typeof it
```

and `ui/src/i18n/messages/en.ts`

```ts
import type { MessageSchema } from './it'

export const en: MessageSchema = {
  ui: {
    close: 'Close',
  },
  shell: {
    newTab: 'New tab',
    closeTab: 'Close tab',
    minimize: 'Minimize',
    maximize: 'Maximize',
    closeWindow: 'Close window',
    search: 'Search everything',
    settings: 'Settings',
    about: 'About',
    resizeSidebar: 'Resize the sidebar',
    sections: {
      wiki: 'Wiki',
      progress: 'Progress',
    },
  },
}
```

- [ ] **Step 2: `ui/src/lib/constants/app.ts`**

```ts
// The product's name. A name, not a message: it reads the same in every language.
export const AppName = 'IsaacDome'
```

- [ ] **Step 3: `ui/src/components/shell/tabOriginIcon.ts`**

```ts
import type { Component } from 'vue'
import { BookIcon, CogIcon, InfoIcon, ListXIcon } from '@lucide/vue'
import { TabOrigin } from './tabs'

// A tab's origin by shape, not colour. A record over the whole set: an origin with no icon
// fails to compile.
export const tabOriginIcon: Record<TabOrigin, Component> = {
  [TabOrigin.Wiki]: BookIcon,
  [TabOrigin.Progress]: ListXIcon,
  [TabOrigin.Settings]: CogIcon,
  [TabOrigin.About]: InfoIcon,
}
```

- [ ] **Step 4: `ui/src/components/shell/TabItem.vue`**

```vue
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
        cn('size-3 shrink-0', active ? 'text-highlight' : 'text-faint-foreground')
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
```

- [ ] **Step 5: `ui/src/components/shell/TabStrip.vue`**

```vue
<script setup lang="ts">
import { PlusIcon } from '@lucide/vue'
import { nextTick, ref } from 'vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { EventKey } from '@/lib/constants/eventKeys'
import TabItem from './TabItem.vue'
import type { DropSide, TabView } from './tabs'
import { TabDrag, dropSide, moveIndex } from './tabs'

const props = defineProps<{ tabs: TabView[]; activeId: string | null }>()
const emit = defineEmits<{
  select: [id: string]
  close: [id: string]
  move: [from: number, to: number]
  add: []
}>()
const { t } = useMessages()

interface Drag {
  from: number
  startX: number
  moving: boolean
}
interface Drop {
  index: number
  side: DropSide
}

const strip = ref<HTMLElement | null>(null)
const drag = ref<Drag | null>(null)
const drop = ref<Drop | null>(null)

const tabElements = (): HTMLElement[] =>
  strip.value
    ? [...strip.value.querySelectorAll<HTMLElement>('[role="tab"]')]
    : []

const dropAt = (x: number, from: number): Drop | null => {
  for (const [index, el] of tabElements().entries()) {
    const r = el.getBoundingClientRect()
    if (x >= r.left && x < r.right)
      return index === from
        ? null
        : { index, side: dropSide(x, r.left, r.width) }
  }
  return null
}

// A press becomes a drag only past the threshold, and only then is the pointer captured:
// a capture from the first pixel would send the click to the strip instead of the tab.
const onPointerDown = (index: number, e: PointerEvent) => {
  if (e.button !== 0) return
  drag.value = { from: index, startX: e.clientX, moving: false }
}

const onPointerMove = (e: PointerEvent) => {
  const d = drag.value
  if (!d) return
  if (!d.moving) {
    if (Math.abs(e.clientX - d.startX) < TabDrag.Threshold) return
    d.moving = true
    strip.value?.setPointerCapture(e.pointerId)
  }
  drop.value = dropAt(e.clientX, d.from)
}

const onPointerUp = (e: PointerEvent) => {
  const d = drag.value
  const target = drop.value
  drag.value = null
  drop.value = null
  if (strip.value?.hasPointerCapture(e.pointerId))
    strip.value.releasePointerCapture(e.pointerId)
  if (!d?.moving || !target) return
  const to = moveIndex(d.from, target.index, target.side)
  if (to !== d.from) emit('move', d.from, to)
}

const neighbour = (key: string): number | null => {
  const index = props.tabs.findIndex((tab) => tab.id === props.activeId)
  if (index < 0) return null
  if (key === EventKey.ArrowRight)
    return Math.min(index + 1, props.tabs.length - 1)
  if (key === EventKey.ArrowLeft) return Math.max(index - 1, 0)
  return null
}

const onKeydown = (e: KeyboardEvent) => {
  const next = neighbour(e.key)
  const tab = next === null ? undefined : props.tabs[next]
  if (next === null || !tab) return
  e.preventDefault()
  emit('select', tab.id)
  void nextTick(() => tabElements()[next]?.focus())
}
</script>

<template>
  <div class="flex min-w-0 items-end gap-px pl-1.75">
    <div
      ref="strip"
      role="tablist"
      class="flex min-w-0 items-end gap-px"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerUp"
      @keydown="onKeydown"
    >
      <TabItem
        v-for="(tab, index) in tabs"
        :key="tab.id"
        :tab="tab"
        :active="tab.id === activeId"
        :dragging="drag?.moving === true && drag.from === index"
        :drop="drop?.index === index ? drop.side : null"
        @pointerdown="onPointerDown(index, $event)"
        @select="emit('select', tab.id)"
        @close="emit('close', tab.id)"
      />
    </div>
    <Button
      :variant="ButtonVariant.Chrome"
      :size="ButtonSize.Icon"
      :aria-label="t('shell.newTab')"
      class="mb-px ml-0.75 size-5.5"
      @click="emit('add')"
    >
      <PlusIcon class="size-3" />
    </Button>
  </div>
</template>
```

- [ ] **Step 6: `ui/src/components/shell/WindowControls.vue`**

```vue
<script setup lang="ts">
import { MinusIcon, SquareIcon, XIcon } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'

const emit = defineEmits<{
  minimize: []
  toggleMaximize: []
  closeWindow: []
}>()
const { t } = useMessages()
</script>

<template>
  <!-- Three 38px controls (Chrome e Stati.dc.html, "Misure della fascia tab"). With the
       window unfocused the title bar's data-focused dims the glyphs. -->
  <div class="flex items-stretch">
    <Button
      :variant="ButtonVariant.Chrome"
      :size="ButtonSize.Window"
      :aria-label="t('shell.minimize')"
      class="group-data-[focused=false]:text-chrome-inactive-foreground"
      @click="emit('minimize')"
    >
      <MinusIcon class="size-2.5" />
    </Button>
    <Button
      :variant="ButtonVariant.Chrome"
      :size="ButtonSize.Window"
      :aria-label="t('shell.maximize')"
      class="group-data-[focused=false]:text-chrome-inactive-foreground"
      @click="emit('toggleMaximize')"
    >
      <SquareIcon class="size-2.5" />
    </Button>
    <Button
      :variant="ButtonVariant.ChromeDanger"
      :size="ButtonSize.Window"
      :aria-label="t('shell.closeWindow')"
      class="group-data-[focused=false]:text-chrome-inactive-foreground"
      @click="emit('closeWindow')"
    >
      <XIcon class="size-2.5" />
    </Button>
  </div>
</template>
```

- [ ] **Step 7: `ui/src/components/shell/TitleBar.vue`**

```vue
<script setup lang="ts">
import TabStrip from './TabStrip.vue'
import WindowControls from './WindowControls.vue'
import type { TabView } from './tabs'

defineProps<{
  tabs: TabView[]
  activeId: string | null
  focused: boolean
}>()
const emit = defineEmits<{
  select: [id: string]
  close: [id: string]
  move: [from: number, to: number]
  add: []
  minimize: []
  toggleMaximize: []
  closeWindow: []
}>()
</script>

<template>
  <!-- Which pages are open (Chrome e Stati.dc.html, "Finestra"). The drag region is an
       element with a minimum width, not the space left over: with nine tabs the names
       truncate and the region stays whole. Unfocused, accents dim and data doesn't. -->
  <header
    :data-focused="focused"
    class="group flex h-titlebar items-stretch border-b border-hairline bg-titlebar"
  >
    <TabStrip
      :tabs="tabs"
      :active-id="activeId"
      @select="emit('select', $event)"
      @close="emit('close', $event)"
      @move="(from, to) => emit('move', from, to)"
      @add="emit('add')"
    />
    <div data-tauri-drag-region class="min-w-drag-region flex-1" />
    <WindowControls
      @minimize="emit('minimize')"
      @toggle-maximize="emit('toggleMaximize')"
      @close-window="emit('closeWindow')"
    />
  </header>
</template>
```

- [ ] **Step 8: `ui/src/components/shell/NavBar.vue`**

```vue
<script setup lang="ts">
import {
  BookIcon,
  CogIcon,
  InfoIcon,
  ListXIcon,
  SearchIcon,
} from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Kbd, KbdGroup } from '@/components/ui/kbd'
import { useMessages } from '@/i18n'
import { AppName } from '@/lib/constants/app'
import { KeyName } from '@/lib/constants/keyNames'
import { NavSection } from './navSection'

const props = defineProps<{ section: NavSection; focused: boolean }>()
const emit = defineEmits<{
  'update:section': [section: NavSection]
  search: []
  settings: []
  about: []
}>()
const { t } = useMessages()

const current = (s: NavSection) => (props.section === s ? 'page' : undefined)
</script>

<template>
  <!-- Which part of the app I'm in (Chrome e Stati.dc.html, "Finestra"). The search field is
       a trigger: it opens the palette, it isn't the input. -->
  <nav
    :data-focused="focused"
    class="group flex h-navbar items-center gap-2.5 border-b border-hairline bg-navbar pr-2.5"
  >
    <div
      class="flex h-full w-brand shrink-0 items-center gap-2 border-r border-hairline pl-3"
    >
      <span
        class="size-3.5 border-2 border-primary bg-sheet group-data-[focused=false]:border-border"
      />
      <span
        class="text-control text-foreground group-data-[focused=false]:text-faint-foreground"
        >{{ AppName }}</span
      >
    </div>
    <div class="flex h-full items-stretch gap-0.5">
      <Button
        :variant="ButtonVariant.Section"
        :aria-current="current(NavSection.Wiki)"
        class="h-full gap-1.75 px-3.25"
        @click="emit('update:section', NavSection.Wiki)"
      >
        <BookIcon class="size-3.5" />{{ t('shell.sections.wiki') }}
      </Button>
      <Button
        :variant="ButtonVariant.Section"
        :aria-current="current(NavSection.Progress)"
        class="h-full gap-1.75 px-3.25"
        @click="emit('update:section', NavSection.Progress)"
      >
        <ListXIcon class="size-3.5" />{{ t('shell.sections.progress') }}
      </Button>
    </div>
    <div class="min-w-0 flex-1" />
    <Button
      :variant="ButtonVariant.Field"
      :size="ButtonSize.Compact"
      class="w-search min-w-10 shrink"
      @click="emit('search')"
    >
      <SearchIcon class="size-3" />
      <span class="min-w-0 flex-1 truncate text-left">{{
        t('shell.search')
      }}</span>
      <KbdGroup>
        <Kbd>{{ KeyName.Ctrl }}</Kbd>
        <Kbd>{{ KeyName.K }}</Kbd>
      </KbdGroup>
    </Button>
    <div class="flex items-center gap-0.5">
      <Button
        :variant="ButtonVariant.Chrome"
        :size="ButtonSize.Icon"
        :aria-label="t('shell.settings')"
        @click="emit('settings')"
      >
        <CogIcon class="size-3.5" />
      </Button>
      <Button
        :variant="ButtonVariant.Chrome"
        :size="ButtonSize.Icon"
        :aria-label="t('shell.about')"
        @click="emit('about')"
      >
        <InfoIcon class="size-3.5" />
      </Button>
    </div>
  </nav>
</template>
```

- [ ] **Step 9: `ui/src/components/shell/SectionSidebar.vue`**

```vue
<script setup lang="ts">
import { computed, ref } from 'vue'
import { useMessages } from '@/i18n'
import { EventKey } from '@/lib/constants/eventKeys'
import { SidebarWidth, clampSidebarWidth } from './sidebarWidth'

defineProps<{ title: string; hint?: string }>()
const width = defineModel<number>('width', { required: true })
const { t } = useMessages()

const resizing = ref<{ startX: number; startWidth: number } | null>(null)
const widthVariable = computed(() => ({
  '--sidebar-width': `${width.value}px`,
}))

const onPointerDown = (e: PointerEvent) => {
  if (e.button !== 0) return
  resizing.value = { startX: e.clientX, startWidth: width.value }
  ;(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId)
}

const onPointerMove = (e: PointerEvent) => {
  const r = resizing.value
  if (!r) return
  width.value = clampSidebarWidth(r.startWidth + e.clientX - r.startX)
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
        <span class="text-highlight [&_svg]:size-3.5"><slot name="icon" /></span>
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
```

- [ ] **Step 10: `ui/src/components/shell/SidebarItem.vue`**

```vue
<script setup lang="ts">
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'

defineProps<{ active: boolean }>()
</script>

<template>
  <!-- The active item has the red bar on the left (ButtonVariant.Nav). -->
  <Button
    :variant="ButtonVariant.Nav"
    :size="ButtonSize.Row"
    :aria-current="active ? 'page' : undefined"
    class="gap-2.25 [&_svg]:size-4 [&_svg]:text-subtle-foreground aria-[current=page]:[&_svg]:text-foreground"
  >
    <slot name="icon" />
    <slot />
  </Button>
</template>
```

- [ ] **Step 11: Verify** — Run: `pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test`. Expected: all green. If `format:check` fails, run `pnpm --filter ui format` on the touched files and re-check.

- [ ] **Step 12: Commit**

```bash
git add ui/src/components/shell ui/src/lib/constants ui/src/i18n/messages
git commit -m "feat(ui): the shell's title bar, navbar and section sidebar"
```

---

### Task 5: The matrix cell

**Files:**
- Create: `ui/src/components/marks/markVisual.ts`, `ui/src/components/marks/MarkCell.vue`
- Test: `ui/src/components/marks/markVisual.test.ts`
- Modify: `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts`

**Interfaces:**
- Consumes: `Cell` from `@/lib/ipc/types`; tokens `mark-cell`, `mark-symbol`, `mark-paper`.
- Produces: `MarkTier`, `MarkVisual`, `MarkArt { normal: string; hard: string }`, `markVisual(cell: Cell): MarkVisual`; `MarkCell` props `{ cell: Cell; art: MarkArt | null; label?: string }`.

- [ ] **Step 1: Write the failing test** — `ui/src/components/marks/markVisual.test.ts`

```ts
import { describe, expect, it } from 'vitest'
import { MarkTier, markVisual } from './markVisual'

const known = (bits: number) => markVisual({ kind: 'known', bits })

describe('markVisual', () => {
  it('reads 0 as never done', () => {
    expect(known(0)).toEqual({ kind: 'empty', third: false })
  })

  it('reads bit 0 alone as the normal mark', () => {
    expect(known(1)).toEqual({
      kind: 'marked',
      tier: MarkTier.Normal,
      third: false,
    })
  })

  it('reads bit 1 alone as the hard mark, as the game draws it', () => {
    expect(known(2)).toEqual({
      kind: 'marked',
      tier: MarkTier.Hard,
      third: false,
    })
  })

  it('reads both levels as the hard mark', () => {
    expect(known(3)).toEqual({
      kind: 'marked',
      tier: MarkTier.Hard,
      third: false,
    })
  })

  it('carries bit 2 alone on an empty cell', () => {
    expect(known(4)).toEqual({ kind: 'empty', third: true })
  })

  it('carries bit 2 beside the normal mark', () => {
    expect(known(5)).toEqual({
      kind: 'marked',
      tier: MarkTier.Normal,
      third: true,
    })
  })

  it('carries bit 2 beside the hard mark', () => {
    expect(known(6)).toEqual({ kind: 'marked', tier: MarkTier.Hard, third: true })
    expect(known(7)).toEqual({ kind: 'marked', tier: MarkTier.Hard, third: true })
  })

  it('reads a bit above bit 2 as unexpected instead of drawing a guess', () => {
    expect(known(8)).toEqual({ kind: 'unexpected', value: 8 })
  })

  it('keeps an unreadable cell unreadable', () => {
    expect(markVisual({ kind: 'unknown' })).toEqual({ kind: 'unknown' })
  })

  it("keeps the IPC's unexpected value", () => {
    expect(markVisual({ kind: 'unexpected', value: 9 })).toEqual({
      kind: 'unexpected',
      value: 9,
    })
  })
})
```

- [ ] **Step 2: Run it to see it fail**

Run: `pnpm --filter ui exec vitest run src/components/marks`
Expected: FAIL — `Failed to resolve import "./markVisual"`.

- [ ] **Step 3: Implement `ui/src/components/marks/markVisual.ts`**

```ts
import type { Cell } from '@/lib/ipc/types'
import { assertNever } from '@/lib/assertNever'

export const MarkTier = { Normal: 'normal', Hard: 'hard' } as const
export type MarkTier = (typeof MarkTier)[keyof typeof MarkTier]

export type MarkVisual =
  | { kind: 'empty'; third: boolean }
  | { kind: 'marked'; tier: MarkTier; third: boolean }
  | { kind: 'unknown' }
  | { kind: 'unexpected'; value: number }

// The symbol URLs of one column. Normal and hard are two different sprites in the game.
export interface MarkArt {
  normal: string
  hard: string
}

// A cell is a bitmask (DESIGN-BRIEF.md §5.3): bit 0 the normal mark, bit 1 the hard one,
// bit 2 a third level whose meaning is unconfirmed. Hard wins whether or not bit 0 is set:
// `2` is common on real profiles and the game draws the hard sprite for it. Bit 2 travels
// beside the tier, never folded into it. A higher bit is outside what the save is known to
// store, so the cell reads as unexpected instead of being drawn from a guess.
const Bit = { Normal: 1, Hard: 2, Third: 4 } as const
const knownBits = Bit.Normal | Bit.Hard | Bit.Third

export const markVisual = (cell: Cell): MarkVisual => {
  switch (cell.kind) {
    case 'unknown':
      return { kind: 'unknown' }
    case 'unexpected':
      return { kind: 'unexpected', value: cell.value }
    case 'known': {
      const { bits } = cell
      if ((bits & ~knownBits) !== 0) return { kind: 'unexpected', value: bits }
      const third = (bits & Bit.Third) !== 0
      if ((bits & Bit.Hard) !== 0)
        return { kind: 'marked', tier: MarkTier.Hard, third }
      if ((bits & Bit.Normal) !== 0)
        return { kind: 'marked', tier: MarkTier.Normal, third }
      return { kind: 'empty', third }
    }
    default:
      return assertNever(cell)
  }
}
```

- [ ] **Step 4: Run it to see it pass**

Run: `pnpm --filter ui exec vitest run src/components/marks`
Expected: PASS (10 tests).

- [ ] **Step 5: The message** — in `it.ts` after the `shell` block add

```ts
  marks: {
    thirdLevel: 'terzo livello, significato non confermato',
  },
```

and in `en.ts`

```ts
  marks: {
    thirdLevel: 'third level, meaning unconfirmed',
  },
```

- [ ] **Step 6: `ui/src/components/marks/MarkCell.vue`**

```vue
<script setup lang="ts">
import { ChevronUpIcon, TriangleAlertIcon } from '@lucide/vue'
import { computed } from 'vue'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import type { Cell } from '@/lib/ipc/types'
import type { MarkArt } from './markVisual'
import { MarkTier, markVisual } from './markVisual'

const props = defineProps<{
  cell: Cell
  art: MarkArt | null
  label?: string
}>()
const { t } = useMessages()

const visual = computed(() => markVisual(props.cell))

const third = computed(() => {
  const v = visual.value
  return (v.kind === 'empty' || v.kind === 'marked') && v.third
})

const symbol = computed(() => {
  const v = visual.value
  if (v.kind !== 'marked' || !props.art) return null
  return v.tier === MarkTier.Hard ? props.art.hard : props.art.normal
})

// The fallback's bar: a third for the normal mark, two thirds for the hard one.
const barHeight = computed(() => {
  const v = visual.value
  if (v.kind !== 'marked') return { '--mark-bar': '0%' }
  return { '--mark-bar': v.tier === MarkTier.Hard ? '66.667%' : '33.333%' }
})

const accessibleName = computed(() => {
  if (props.label === undefined) return undefined
  return third.value ? `${props.label}, ${t('marks.thirdLevel')}` : props.label
})
</script>

<template>
  <!-- Chrome e Stati.dc.html, "Cella della matrice", at cycle 2's scale: a 16px symbol at 2x
       on flat paper; with no art, the bars of Tokens.dc.html. Without a label the cell is
       decoration and the text beside it speaks. -->
  <div
    :role="label === undefined ? undefined : 'img'"
    :aria-label="accessibleName"
    :aria-hidden="label === undefined ? true : undefined"
    :style="barHeight"
    :class="
      cn(
        'relative grid size-mark-cell shrink-0 place-items-center border',
        visual.kind === 'empty' && 'border-hairline bg-data',
        visual.kind === 'marked' &&
          (symbol ? 'border-transparent bg-mark-paper' : 'border-input bg-data'),
        visual.kind === 'unknown' &&
          'border-dashed border-state-unknown text-row text-muted-foreground hatch-unknown',
        visual.kind === 'unexpected' &&
          'border-state-unexpected bg-state-unexpected-surface text-state-unexpected-foreground',
      )
    "
  >
    <template v-if="visual.kind === 'marked'">
      <img v-if="symbol" :src="symbol" alt="" class="size-mark-symbol pixelated" />
      <template v-else>
        <span class="absolute inset-x-0 bottom-0 h-(--mark-bar) bg-primary" />
        <ChevronUpIcon
          v-if="visual.tier === MarkTier.Hard"
          class="relative size-3.5 text-foreground"
        />
      </template>
    </template>
    <span v-else-if="visual.kind === 'unknown'">?</span>
    <TriangleAlertIcon v-else-if="visual.kind === 'unexpected'" class="size-4" />
    <span
      v-if="third"
      :class="
        cn(
          'absolute right-0.75 size-1.5',
          symbol ? 'bottom-0.75 bg-background' : 'top-0.75 bg-highlight',
        )
      "
    />
  </div>
</template>
```

- [ ] **Step 7: Verify** — Run: `pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test`. Expected: all green.

- [ ] **Step 8: Commit**

```bash
git add ui/src/components/marks ui/src/i18n/messages
git commit -m "feat(ui): MarkCell draws a completion mark in the game's sprites or in bars"
```

---

### Task 6: The KPI tile

**Files:**
- Create: `ui/src/components/kpi/kpiBar.ts`, `ui/src/components/kpi/kpiTone.ts`, `ui/src/components/kpi/KpiTile.vue`
- Test: `ui/src/components/kpi/kpiBar.test.ts`

**Interfaces:**
- Consumes: `progressShares(value, unknown, max)` from `@/components/ui/progress/progressShares`.
- Produces: `kpiBar(value: number, denominator: number | null): number | null`; `KpiTone`; `KpiTile` props `{ value: number; denominator?: number | null; unit?: string; label: string; tone?: KpiTone }`, slot `explain`.

- [ ] **Step 1: Write the failing test** — `ui/src/components/kpi/kpiBar.test.ts`

```ts
import { describe, expect, it } from 'vitest'
import { kpiBar } from './kpiBar'

describe('kpiBar', () => {
  it('is the share of the declared denominator, in percent', () => {
    expect(kpiBar(166, 368)).toBeCloseTo((166 / 368) * 100)
  })

  it('has no bar without a denominator', () => {
    expect(kpiBar(5, null)).toBeNull()
  })

  it('has no bar against a denominator of zero', () => {
    expect(kpiBar(5, 0)).toBeNull()
  })

  it('never passes a full bar', () => {
    expect(kpiBar(500, 368)).toBe(100)
  })
})
```

- [ ] **Step 2: Run it to see it fail**

Run: `pnpm --filter ui exec vitest run src/components/kpi`
Expected: FAIL — `Failed to resolve import "./kpiBar"`.

- [ ] **Step 3: Implement**

`ui/src/components/kpi/kpiBar.ts`:

```ts
import { progressShares } from '@/components/ui/progress/progressShares'

// A KPI's bar is a ratio, so it exists only with a declared denominator (Chrome e
// Stati.dc.html, "KPI"): no denominator, no bar, rather than a bar against a guess.
export const kpiBar = (
  value: number,
  denominator: number | null,
): number | null =>
  denominator === null || !Number.isFinite(denominator) || denominator <= 0
    ? null
    : progressShares(value, 0, denominator).value
```

`ui/src/components/kpi/kpiTone.ts`:

```ts
// What a KPI's number is: progress, something done, or a count of what we can't read — a
// datum like the others, never a progress.
export const KpiTone = {
  Progress: 'progress',
  Done: 'done',
  Unknown: 'unknown',
} as const
export type KpiTone = (typeof KpiTone)[keyof typeof KpiTone]
```

- [ ] **Step 4: Run it to see it pass**

Run: `pnpm --filter ui exec vitest run src/components/kpi`
Expected: PASS (4 tests).

- [ ] **Step 5: `ui/src/components/kpi/KpiTile.vue`**

```vue
<script setup lang="ts">
import { computed, useSlots } from 'vue'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import { assertNever } from '@/lib/assertNever'
import { cn } from '@/lib/cn'
import { kpiBar } from './kpiBar'
import { KpiTone } from './kpiTone'

const props = withDefaults(
  defineProps<{
    value: number
    denominator?: number | null
    unit?: string
    label: string
    tone?: KpiTone
  }>(),
  { denominator: null, unit: undefined, tone: KpiTone.Progress },
)
const slots = useSlots()

const bar = computed(() => kpiBar(props.value, props.denominator))
const barWidth = computed(() => ({ '--kpi-bar': `${bar.value ?? 0}%` }))

const barClass = computed(() => {
  switch (props.tone) {
    case KpiTone.Progress:
      return 'bg-primary'
    case KpiTone.Done:
      return 'bg-state-done'
    case KpiTone.Unknown:
      return 'bg-faint-foreground'
    default:
      return assertNever(props.tone)
  }
})
</script>

<template>
  <!-- A number, its unit, the label, a micro bar (Chrome e Stati.dc.html, "KPI"). No prose:
       the explanation lives in the tooltip. -->
  <Tooltip :disabled="!slots.explain">
    <TooltipTrigger as-child>
      <div
        :tabindex="slots.explain ? 0 : undefined"
        class="flex flex-col border border-border bg-sheet px-3.25 pt-2.75 pb-3"
      >
        <div class="flex items-baseline gap-1.25">
          <span
            :class="
              cn(
                'text-kpi',
                tone === KpiTone.Unknown
                  ? 'text-muted-foreground'
                  : 'text-foreground',
              )
            "
            >{{ value }}</span
          >
          <span
            v-if="unit !== undefined"
            class="text-caption text-subtle-foreground"
            >{{ unit }}</span
          >
          <span
            v-else-if="denominator !== null"
            class="text-caption text-subtle-foreground"
            >/ {{ denominator }}</span
          >
        </div>
        <span class="mt-1.75 text-label text-muted-foreground">{{ label }}</span>
        <div
          v-if="bar !== null"
          :style="barWidth"
          class="mt-2 flex h-1 border border-secondary bg-data"
        >
          <div :class="cn('w-(--kpi-bar)', barClass)" />
        </div>
      </div>
    </TooltipTrigger>
    <TooltipContent><slot name="explain" /></TooltipContent>
  </Tooltip>
</template>
```

- [ ] **Step 6: Verify** — Run: `pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test`. Expected: all green.

- [ ] **Step 7: Commit**

```bash
git add ui/src/components/kpi
git commit -m "feat(ui): KpiTile, a number with a declared denominator or no bar"
```

---

### Task 7: The wiki's tokens and blocks

**Files:**
- Create: `ui/src/components/wiki/dlcNames.ts`, `ui/src/components/wiki/editionLabel.ts`
- Test: `ui/src/components/wiki/editionLabel.test.ts`
- Move and rewrite: `ui/src/components/WikiInline.vue` → `ui/src/components/wiki/WikiInline.vue`; `ui/src/components/WikiBlocks.vue` → `ui/src/components/wiki/WikiBlocks.vue`
- Modify: `ui/src/App.vue` (the import), `ui/scripts/scan-conventions.mjs` (WikiInline's exemption)

**Interfaces:**
- Consumes: `Inline`, `Block`, `Target`, `Style`, `Dlc` from `@/lib/ipc/types`; `ButtonVariant.Ref`, `ButtonSize.Inline`; the `Table` primitive.
- Produces: `editionLabel(only: Dlc[]): string`; `WikiInline` props `{ inline: Inline[]; iconFor?: (target: Target) => string | null }`, emit `navigate(target)`; `WikiBlocks` props `{ blocks: Block[]; iconFor?: … }`, emit `navigate(target)`.

- [ ] **Step 1: Write the failing test** — `ui/src/components/wiki/editionLabel.test.ts`

```ts
import { describe, expect, it } from 'vitest'
import { Dlc } from '@/lib/ipc/types'
import { editionLabel } from './editionLabel'

describe('editionLabel', () => {
  it('draws no tag for no edition', () => {
    expect(editionLabel([])).toBe('')
  })

  it('names a single edition', () => {
    expect(editionLabel([Dlc.Repentance])).toBe('Repentance')
  })

  it('orders editions by release, whatever the order given', () => {
    expect(editionLabel([Dlc.RepentancePlus, Dlc.Afterbirth])).toBe(
      'Afterbirth · Repentance+',
    )
  })

  it('names a repeated edition once', () => {
    expect(editionLabel([Dlc.Rebirth, Dlc.Rebirth])).toBe('Rebirth')
  })
})
```

- [ ] **Step 2: Run it to see it fail**

Run: `pnpm --filter ui exec vitest run src/components/wiki`
Expected: FAIL — `Failed to resolve import "./editionLabel"`.

- [ ] **Step 3: Implement**

`ui/src/components/wiki/dlcNames.ts`:

```ts
import { Dlc } from '@/lib/ipc/types'

// The editions' names as the game prints them. Game names stay in English in both
// languages, like item names, so they're data rather than messages.
export const dlcNames: Record<Dlc, string> = {
  [Dlc.Rebirth]: 'Rebirth',
  [Dlc.Afterbirth]: 'Afterbirth',
  [Dlc.AfterbirthPlus]: 'Afterbirth+',
  [Dlc.Repentance]: 'Repentance',
  [Dlc.RepentancePlus]: 'Repentance+',
}
```

`ui/src/components/wiki/editionLabel.ts`:

```ts
import { Dlc } from '@/lib/ipc/types'
import { dlcNames } from './dlcNames'

// The tag an edition-scoped passage carries: the editions' names in release order (the
// declaration order of Dlc), each once. No edition gives no text, and no tag is drawn.
export const editionLabel = (only: Dlc[]): string =>
  Object.values(Dlc)
    .filter((dlc) => only.includes(dlc))
    .map((dlc) => dlcNames[dlc])
    .join(' · ')
```

- [ ] **Step 4: Run it to see it pass**

Run: `pnpm --filter ui exec vitest run src/components/wiki`
Expected: PASS (4 tests).

- [ ] **Step 5: Move the two components**

```bash
git mv ui/src/components/WikiInline.vue ui/src/components/wiki/WikiInline.vue
git mv ui/src/components/WikiBlocks.vue ui/src/components/wiki/WikiBlocks.vue
```

- [ ] **Step 6: Rewrite `ui/src/components/wiki/WikiInline.vue`**

```vue
<script setup lang="ts">
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { assertNever } from '@/lib/assertNever'
import type { Inline, Target } from '@/lib/ipc/types'
import { Style } from '@/lib/ipc/types'
import { editionLabel } from './editionLabel'

const props = defineProps<{
  inline: Inline[]
  iconFor?: (target: Target) => string | null
}>()
const emit = defineEmits<{ navigate: [target: Target] }>()

const textClass = (style: Style): string => {
  switch (style) {
    case Style.Plain:
      return 'text-foreground-soft'
    case Style.Bold:
      return 'text-foreground'
    case Style.Italic:
      return 'text-foreground-soft italic'
    default:
      return assertNever(style)
  }
}

const icon = (target: Target): string | null => props.iconFor?.(target) ?? null
</script>

<template>
  <!-- Four natures that must tell apart at a glance (Chrome e Stati.dc.html, "Token inline
       della wiki"): a solid underline opens, a dotted one only reads, a reference with no
       sprite leaves no icon hole. Vue condenses whitespace between tags on separate lines,
       so no space lands before a comma. -->
  <template v-for="(token, index) in inline" :key="index">
    <span v-if="token.kind === 'text'" :class="textClass(token.style)">{{
      token.text
    }}</span>
    <Button
      v-else-if="token.kind === 'ref'"
      :variant="ButtonVariant.Ref"
      :size="ButtonSize.Inline"
      @click="emit('navigate', token.target)"
    >
      <img
        v-if="icon(token.target)"
        :src="icon(token.target) ?? undefined"
        alt=""
        class="size-4 pixelated"
      />{{ token.label }}
    </Button>
    <span
      v-else-if="token.kind === 'concept'"
      class="border-b border-dotted border-secondary-edge text-subtle-foreground"
      >{{ token.label }}</span
    >
    <template v-else-if="token.kind === 'edition'">
      <span
        v-if="token.only.length"
        class="border border-secondary-edge px-1.25 text-label text-highlight"
        >{{ editionLabel(token.only) }}</span
      >
      <WikiInline
        :inline="token.inline"
        :icon-for="iconFor"
        @navigate="emit('navigate', $event)"
      />
    </template>
    <span v-else>{{ assertNever(token) }}</span>
  </template>
</template>
```

- [ ] **Step 7: Rewrite `ui/src/components/wiki/WikiBlocks.vue`**

```vue
<script setup lang="ts">
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { assertNever } from '@/lib/assertNever'
import { cn } from '@/lib/cn'
import type { Block, Target } from '@/lib/ipc/types'
import WikiInline from './WikiInline.vue'

defineProps<{
  blocks: Block[]
  iconFor?: (target: Target) => string | null
}>()
const emit = defineEmits<{ navigate: [target: Target] }>()
</script>

<template>
  <div class="flex flex-col gap-2 text-row">
    <template v-for="(block, index) in blocks" :key="index">
      <p v-if="block.kind === 'paragraph'">
        <WikiInline
          :inline="block.inline"
          :icon-for="iconFor"
          @navigate="emit('navigate', $event)"
        />
      </p>
      <component
        :is="block.ordered ? 'ol' : 'ul'"
        v-else-if="block.kind === 'list'"
        :class="
          cn(
            'flex flex-col gap-1 pl-4 text-foreground-soft',
            block.ordered ? 'list-decimal' : 'list-disc',
          )
        "
      >
        <li v-for="(item, i) in block.items" :key="i">
          <WikiInline
            :inline="item.inline"
            :icon-for="iconFor"
            @navigate="emit('navigate', $event)"
          />
          <WikiBlocks
            v-if="item.children.length"
            :blocks="item.children"
            :icon-for="iconFor"
            class="mt-1"
            @navigate="emit('navigate', $event)"
          />
        </li>
      </component>
      <Table v-else-if="block.kind === 'table'">
        <TableHeader>
          <TableRow>
            <TableHead v-for="(cell, i) in block.header" :key="i">
              <WikiInline
                :inline="cell"
                :icon-for="iconFor"
                @navigate="emit('navigate', $event)"
              />
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-for="(row, i) in block.rows" :key="i">
            <TableCell v-for="(cell, j) in row" :key="j">
              <WikiInline
                :inline="cell"
                :icon-for="iconFor"
                @navigate="emit('navigate', $event)"
              />
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
      <h3
        v-else-if="block.kind === 'heading' && block.level <= 3"
        class="text-control text-highlight"
      >
        <WikiInline
          :inline="block.inline"
          :icon-for="iconFor"
          @navigate="emit('navigate', $event)"
        />
      </h3>
      <h4 v-else-if="block.kind === 'heading'" class="text-caption text-highlight">
        <WikiInline
          :inline="block.inline"
          :icon-for="iconFor"
          @navigate="emit('navigate', $event)"
        />
      </h4>
      <p v-else>{{ assertNever(block) }}</p>
    </template>
  </div>
</template>
```

- [ ] **Step 8: Point `App.vue` at the new place** — replace

```ts
import WikiBlocks from './components/WikiBlocks.vue'
```

with

```ts
import WikiBlocks from './components/wiki/WikiBlocks.vue'
```

- [ ] **Step 9: Drop the scanner's WikiInline exemption** — in `ui/scripts/scan-conventions.mjs` delete the object

```js
  {
    file: 'src/components/WikiInline.vue',
    check: 'raw primitive <button>/<input>',
    reason:
      'verification render of the wiki dataset: the link to another target will become a primitive',
  },
```

- [ ] **Step 10: Verify** — Run: `pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test`. Expected: all green, scan reporting `0 violations, 1 declared exemptions`.

- [ ] **Step 11: Commit**

```bash
git add ui/src/components/wiki ui/src/components/WikiInline.vue ui/src/components/WikiBlocks.vue ui/src/App.vue ui/scripts/scan-conventions.mjs
git commit -m "feat(ui): the wiki's inline tokens and blocks, dressed"
```

---

### Task 8: Three data states

**Files:**
- Create: `ui/src/components/data-state/EmptyValue.vue`, `UnreadableValue.vue`, `EmptyCategory.vue`

**Interfaces:**
- Consumes: Task 5's `MarkCell`.
- Produces: three components with a default slot each.

- [ ] **Step 1: `ui/src/components/data-state/EmptyValue.vue`**

```vue
<template>
  <!-- Read, and the value is empty: the contract carries the field, the dataset doesn't
       fill it (Chrome e Stati.dc.html, "Stati del dato"). There's the box, not the content. -->
  <span class="text-row text-faint-foreground italic"><slot /></span>
</template>
```

- [ ] **Step 2: `ui/src/components/data-state/UnreadableValue.vue`**

```vue
<script setup lang="ts">
import MarkCell from '@/components/marks/MarkCell.vue'
</script>

<template>
  <!-- Not "never done": outside the denominator of every percentage. The cell is decoration
       here, the text beside it says what it is. -->
  <span class="inline-flex items-center gap-2.5">
    <MarkCell :cell="{ kind: 'unknown' }" :art="null" />
    <span class="text-caption text-muted-foreground"><slot /></span>
  </span>
</template>
```

- [ ] **Step 3: `ui/src/components/data-state/EmptyCategory.vue`**

```vue
<template>
  <!-- A category with no pages: a dashed frame, never a fallback to another page, or the tab
       would say one thing and the content another. -->
  <div
    class="border border-dashed border-border px-3 py-2.5 text-caption text-subtle-foreground"
  >
    <slot />
  </div>
</template>
```

- [ ] **Step 4: Verify** — Run: `pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan`. Expected: all green.

- [ ] **Step 5: Commit**

```bash
git add ui/src/components/data-state
git commit -m "feat(ui): three data states that must not look alike"
```

---

### Task 9: The Kit page shows the app components

**Files:**
- Create: `ui/src/kit/markArt.ts`
- Create: `ui/src/kit/sections/app/TitleBarSection.vue`, `NavBarSection.vue`, `SectionSidebarSection.vue`, `KpiTileSection.vue`, `MarkCellSection.vue`, `WikiSection.vue`, `DataStateSection.vue`, `CollapsibleCardSection.vue`
- Modify: `ui/src/kit/KitPage.vue`

**Interfaces:**
- Consumes: every component of Tasks 2 and 4–8.
- Produces: `kitMarkArt: { heart, polaroid, star, delirium, knife: MarkArt | null }`.

- [ ] **Step 1: `ui/src/kit/markArt.ts`**

```ts
import type { MarkArt } from '@/components/marks/markVisual'

// Development only: the mark symbols from the design pack under design-export/. The Kit
// page is behind import.meta.env.DEV, so the production build never sees these files; a
// clone without the pack gets an empty glob, null art and the fallback outfit.
const widget = import.meta.glob<string>(
  '../../../design-export/isaacdome-design-pack/images/sheets/completion_widget/*_0[02].png',
  { eager: true, query: '?url', import: 'default' },
)
const lobby = import.meta.glob<string>(
  '../../../design-export/isaacdome-design-pack/images/sheets/onlinelobby/background_completion_delirium_0[02].png',
  { eager: true, query: '?url', import: 'default' },
)

const file = (
  files: Record<string, string>,
  stem: string,
  tier: string,
): string | undefined =>
  Object.entries(files).find(([path]) =>
    path.endsWith(`/${stem}_${tier}.png`),
  )?.[1]

const art = (files: Record<string, string>, stem: string): MarkArt | null => {
  const normal = file(files, stem, '00')
  const hard = file(files, stem, '02')
  return normal && hard ? { normal, hard } : null
}

// The five columns the Kit shows: Mom's Heart, Isaac, Boss Rush, Delirium, Mother.
export const kitMarkArt = {
  heart: art(widget, 'heart'),
  polaroid: art(widget, 'polaroid'),
  star: art(widget, 'star'),
  delirium: art(lobby, 'background_completion_delirium'),
  knife: art(widget, 'knife'),
}
```

- [ ] **Step 2: `ui/src/kit/sections/app/TitleBarSection.vue`**

```vue
<script setup lang="ts">
import { ref } from 'vue'
import TitleBar from '@/components/shell/TitleBar.vue'
import type { TabView } from '@/components/shell/tabs'
import { TabOrigin } from '@/components/shell/tabs'
import KitSection from '../../KitSection.vue'

const tabs = ref<TabView[]>([
  { id: 'd6', label: 'The D6', origin: TabOrigin.Wiki },
  { id: 'steps', label: 'Prossimi passi', origin: TabOrigin.Progress },
  { id: 'profile', label: 'Profilo di gioco', origin: TabOrigin.Settings },
])
const active = ref<string | null>('d6')
let added = 0

const crowded: TabView[] = [
  'The D6',
  'False PHD',
  'Isaac',
  'Monstro',
  'Passi',
  'Unlock',
  'Run',
  'Profilo',
  'Tab',
].map((label, i) => ({
  id: `crowded-${i}`,
  label,
  origin: i < 4 ? TabOrigin.Wiki : TabOrigin.Progress,
}))

const move = (from: number, to: number) => {
  const next = [...tabs.value]
  const [tab] = next.splice(from, 1)
  if (tab) next.splice(to, 0, tab)
  tabs.value = next
}

const close = (id: string) => {
  tabs.value = tabs.value.filter((tab) => tab.id !== id)
}

const add = () => {
  added += 1
  const id = `new-${added}`
  tabs.value = [
    ...tabs.value,
    { id, label: `Nuova ${added}`, origin: TabOrigin.About },
  ]
  active.value = id
}
</script>

<template>
  <KitSection title="TitleBar" class="col-span-3">
    <TitleBar
      :tabs="tabs"
      :active-id="active"
      :focused="true"
      @select="active = $event"
      @close="close"
      @move="move"
      @add="add"
    />
    <TitleBar :tabs="tabs" :active-id="active" :focused="false" />
    <TitleBar
      :tabs="crowded"
      active-id="crowded-0"
      :focused="true"
      class="w-160"
    />
  </KitSection>
</template>
```

- [ ] **Step 3: `ui/src/kit/sections/app/NavBarSection.vue`**

```vue
<script setup lang="ts">
import { ref } from 'vue'
import NavBar from '@/components/shell/NavBar.vue'
import { NavSection } from '@/components/shell/navSection'
import KitSection from '../../KitSection.vue'

const section = ref<NavSection>(NavSection.Wiki)
</script>

<template>
  <KitSection title="NavBar" class="col-span-3">
    <NavBar v-model:section="section" :focused="true" />
    <NavBar :section="NavSection.Progress" :focused="false" />
  </KitSection>
</template>
```

- [ ] **Step 4: `ui/src/kit/sections/app/SectionSidebarSection.vue`**

```vue
<script setup lang="ts">
import {
  CogIcon,
  Grid2x2Icon,
  ListXIcon,
  SaveIcon,
  StarIcon,
} from '@lucide/vue'
import { ref } from 'vue'
import SectionSidebar from '@/components/shell/SectionSidebar.vue'
import SidebarItem from '@/components/shell/SidebarItem.vue'
import { SidebarWidth } from '@/components/shell/sidebarWidth'
import KitSection from '../../KitSection.vue'

const progressWidth = ref<number>(SidebarWidth.Default)
const settingsWidth = ref<number>(SidebarWidth.Default)
const current = ref('completion')
const progressItems = [
  { id: 'steps', label: 'Prossimi passi', icon: ListXIcon },
  { id: 'completion', label: 'Completamento', icon: Grid2x2Icon },
  { id: 'unlock', label: 'Unlock', icon: StarIcon },
]
</script>

<template>
  <KitSection title="SectionSidebar" class="col-span-2">
    <div class="flex items-start gap-4">
      <SectionSidebar
        v-model:width="progressWidth"
        title="Progressi"
        hint="Dipende dal profilo attivo."
      >
        <template #icon><ListXIcon /></template>
        <SidebarItem
          v-for="item in progressItems"
          :key="item.id"
          :active="item.id === current"
          @click="current = item.id"
        >
          <template #icon><component :is="item.icon" /></template>
          {{ item.label }}
        </SidebarItem>
      </SectionSidebar>
      <SectionSidebar
        v-model:width="settingsWidth"
        title="Impostazioni"
        hint="Come l'app trova gioco e salvataggi."
      >
        <template #icon><CogIcon /></template>
        <SidebarItem :active="false">
          <template #icon><SaveIcon /></template>
          Profilo di gioco
        </SidebarItem>
        <SidebarItem :active="true">
          <template #icon><Grid2x2Icon /></template>
          Tab
        </SidebarItem>
      </SectionSidebar>
    </div>
  </KitSection>
</template>
```

- [ ] **Step 5: `ui/src/kit/sections/app/KpiTileSection.vue`**

```vue
<script setup lang="ts">
import KpiTile from '@/components/kpi/KpiTile.vue'
import { KpiTone } from '@/components/kpi/kpiTone'
import KitSection from '../../KitSection.vue'
</script>

<template>
  <KitSection title="KpiTile">
    <div class="grid grid-cols-2 gap-2.5">
      <KpiTile :value="166" :denominator="368" label="marchi iniziati">
        <template #explain>
          Celle con almeno un segno, fra quelle che il salvataggio lascia leggere.
        </template>
      </KpiTile>
      <KpiTile :value="120" unit="celle" label="normale + hard" />
      <KpiTile
        :value="3"
        :denominator="34"
        label="personaggi completi"
        :tone="KpiTone.Done"
      />
      <KpiTile
        :value="40"
        :denominator="408"
        unit="celle"
        label="non leggibili"
        :tone="KpiTone.Unknown"
      />
    </div>
  </KitSection>
</template>
```

- [ ] **Step 6: `ui/src/kit/sections/app/MarkCellSection.vue`**

```vue
<script setup lang="ts">
import MarkCell from '@/components/marks/MarkCell.vue'
import type { Cell } from '@/lib/ipc/types'
import KitSection from '../../KitSection.vue'
import { kitMarkArt } from '../../markArt'

const known = (bits: number): Cell => ({ kind: 'known', bits })
const unknown: Cell = { kind: 'unknown' }

const states: { label: string; cell: Cell }[] = [
  { label: 'mai fatto', cell: known(0) },
  { label: 'normale', cell: known(1) },
  { label: 'hard', cell: known(3) },
  { label: 'hard + bit 2', cell: known(7) },
  { label: 'solo bit 2', cell: known(4) },
  { label: 'non leggibile', cell: unknown },
  { label: 'anomalo', cell: { kind: 'unexpected', value: 9 } },
]

const columns = [
  kitMarkArt.heart,
  kitMarkArt.polaroid,
  kitMarkArt.star,
  kitMarkArt.delirium,
  kitMarkArt.knife,
]

// The reference profile's shape: a dense top, an empty bottom, an unreadable corner.
const rows: { name: string; cells: Cell[] }[] = [
  { name: 'Isaac', cells: [known(3), known(7), known(3), known(2), known(1)] },
  { name: 'Cain', cells: [known(3), known(3), known(0), known(1), known(0)] },
  {
    name: 'The Forgotten',
    cells: [known(2), known(0), known(0), known(0), unknown],
  },
  {
    name: 'Tainted Lost',
    cells: [known(0), known(5), known(0), known(0), unknown],
  },
]
const outfits = [true, false]
</script>

<template>
  <KitSection title="MarkCell" class="col-span-2">
    <div
      v-for="withArt in outfits"
      :key="String(withArt)"
      class="flex flex-wrap gap-4"
    >
      <div class="flex flex-wrap gap-2">
        <div
          v-for="state in states"
          :key="state.label"
          class="flex w-14 flex-col items-center gap-1.5"
        >
          <MarkCell
            :cell="state.cell"
            :art="withArt ? kitMarkArt.heart : null"
            :label="state.label"
          />
          <span class="text-center text-micro text-subtle-foreground">{{
            state.label
          }}</span>
        </div>
      </div>
      <div class="flex flex-col gap-0.5">
        <div
          v-for="row in rows"
          :key="row.name"
          class="flex items-center gap-0.5"
        >
          <span class="w-24 truncate text-label text-foreground-soft">{{
            row.name
          }}</span>
          <MarkCell
            v-for="(cell, i) in row.cells"
            :key="i"
            :cell="cell"
            :art="withArt ? (columns[i] ?? null) : null"
            :label="`${row.name} ${i}`"
          />
        </div>
      </div>
    </div>
  </KitSection>
</template>
```

- [ ] **Step 7: `ui/src/kit/sections/app/WikiSection.vue`**

```vue
<script setup lang="ts">
import WikiBlocks from '@/components/wiki/WikiBlocks.vue'
import WikiInline from '@/components/wiki/WikiInline.vue'
import type { Block, Inline, Target } from '@/lib/ipc/types'
import { Dlc, Style } from '@/lib/ipc/types'
import KitSection from '../../KitSection.vue'
import { kitMarkArt } from '../../markArt'

const d6: Target = { kind: 'item', id: 105 }
const plain = (text: string): Inline => ({ kind: 'text', text, style: Style.Plain })

const natures: Inline[][] = [
  [plain('testo semplice '), { kind: 'text', text: 'e in grassetto', style: Style.Bold }],
  [{ kind: 'ref', target: d6, label: 'The D6' }],
  [{ kind: 'ref', target: { kind: 'pickup', name: 'Chest' }, label: 'Chest' }],
  [{ kind: 'concept', page: 'Devil Room', label: 'Devil Room' }],
  [{ kind: 'edition', only: [Dlc.Repentance], inline: [] }],
]

const paragraph: Inline[] = [
  plain("Al momento dell'uso, ripiazza tutti gli oggetti nella stanza. "),
  { kind: 'ref', target: d6, label: 'The D6' },
  plain(' si ricarica in '),
  { kind: 'text', text: '6 stanze', style: Style.Bold },
  plain('. Nella '),
  { kind: 'concept', page: 'Devil Room', label: 'Devil Room' },
  plain(" l'effetto è identico. "),
  {
    kind: 'edition',
    only: [Dlc.RepentancePlus, Dlc.Repentance],
    inline: [plain(' cambia il conteggio.')],
  },
]

const blocks: Block[] = [
  { kind: 'heading', level: 3, inline: [plain('Note')] },
  { kind: 'paragraph', inline: paragraph },
  {
    kind: 'list',
    ordered: false,
    items: [
      { inline: [plain('Non ripiazza gli oggetti del negozio.')], children: [] },
      {
        inline: [
          { kind: 'ref', target: d6, label: 'The D6' },
          plain(' con Car Battery: due lanci.'),
        ],
        children: [],
      },
    ],
  },
  {
    kind: 'table',
    header: [[plain('Edizione')], [plain('Carica')]],
    rows: [
      [[plain('Normale')], [plain('6')]],
      [[{ kind: 'edition', only: [Dlc.RepentancePlus], inline: [] }], [plain('4')]],
    ],
  },
]

// A stand-in for cycle 3's icon protocol: items get a sprite, everything else none, so a
// reference with an icon and one without sit side by side.
const iconFor = (target: Target): string | null =>
  target.kind === 'item' ? (kitMarkArt.heart?.normal ?? null) : null
</script>

<template>
  <KitSection title="Wiki" class="col-span-2">
    <div class="flex flex-col gap-2 text-row">
      <p v-for="(line, i) in natures" :key="i">
        <WikiInline :inline="line" :icon-for="iconFor" />
      </p>
    </div>
    <p class="text-row"><WikiInline :inline="paragraph" :icon-for="iconFor" /></p>
    <WikiBlocks :blocks="blocks" :icon-for="iconFor" />
  </KitSection>
</template>
```

- [ ] **Step 8: `ui/src/kit/sections/app/DataStateSection.vue`**

```vue
<script setup lang="ts">
import EmptyCategory from '@/components/data-state/EmptyCategory.vue'
import EmptyValue from '@/components/data-state/EmptyValue.vue'
import UnreadableValue from '@/components/data-state/UnreadableValue.vue'
import KitSection from '../../KitSection.vue'
</script>

<template>
  <KitSection title="Stati del dato">
    <span class="text-row text-foreground-soft">6 stanze</span>
    <EmptyValue>non nel dataset</EmptyValue>
    <UnreadableValue>il salvataggio non ce lo dice</UnreadableValue>
    <EmptyCategory>Nessuna pagina di Trinket fra i campioni.</EmptyCategory>
  </KitSection>
</template>
```

- [ ] **Step 9: `ui/src/kit/sections/app/CollapsibleCardSection.vue`**

```vue
<script setup lang="ts">
import {
  CardCollapsible,
  CardCollapsibleContent,
  CardCollapsibleTrigger,
} from '@/components/ui/card'
import KitSection from '../../KitSection.vue'
</script>

<template>
  <KitSection title="Card comprimibile">
    <CardCollapsible :default-open="true">
      <CardCollapsibleTrigger>
        Cambia profilo
        <template #summary>slot 1</template>
      </CardCollapsibleTrigger>
      <CardCollapsibleContent>
        Aperta: il contenuto sta dentro il bordo della card, con lo stesso passo di
        12px della testata.
      </CardCollapsibleContent>
    </CardCollapsible>
    <CardCollapsible>
      <CardCollapsibleTrigger>
        Rileggi il file
        <template #summary>2 giorni fa</template>
      </CardCollapsibleTrigger>
      <CardCollapsibleContent>
        Il salvataggio viene riletto da capo.
      </CardCollapsibleContent>
    </CardCollapsible>
  </KitSection>
</template>
```

- [ ] **Step 10: `ui/src/kit/KitPage.vue`** — add the imports after `import ProgressSection …`:

```ts
import TitleBarSection from './sections/app/TitleBarSection.vue'
import NavBarSection from './sections/app/NavBarSection.vue'
import SectionSidebarSection from './sections/app/SectionSidebarSection.vue'
import KpiTileSection from './sections/app/KpiTileSection.vue'
import MarkCellSection from './sections/app/MarkCellSection.vue'
import WikiSection from './sections/app/WikiSection.vue'
import DataStateSection from './sections/app/DataStateSection.vue'
import CollapsibleCardSection from './sections/app/CollapsibleCardSection.vue'
```

and after `<ProgressSection />`:

```vue
      <h1 class="col-span-3 mt-6 text-heading text-highlight">
        Componenti app
      </h1>
      <TitleBarSection />
      <NavBarSection />
      <SectionSidebarSection />
      <KpiTileSection />
      <MarkCellSection />
      <DataStateSection />
      <WikiSection />
      <CollapsibleCardSection />
```

- [ ] **Step 11: Verify statically** — Run: `pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test`. Expected: all green.

- [ ] **Step 12: Verify visually** — `pnpm ui:dev`, open `http://localhost:1420/#kit` (or the port Vite prints), compare against `design-export/design-system/Chrome e Stati.dc.html`:
  - tab strip: active tab's red top edge, a live drag shows the red drop edge on the landing side and reorders on release, a click without movement selects; nine tabs truncate and the drag region stays;
  - unfocused title bar and navbar: dimmed window glyphs and brand, tabs unchanged;
  - sidebar: dragging the handle resizes between 168 and 420, double click returns to 212, arrow keys step 16;
  - mark cells: sprite outfit shows symbols at 32px on the paper, fallback shows bars; unknown hatch with `?`; bit 2 square bottom-right on paper, top-right on bars;
  - wiki: no space before punctuation after a reference; concept dotted, reference solid; edition tag reads `Repentance · Repentance+`;
  - KPI tooltip opens on the first tile; the second has no bar.
  Fix what differs from the spec before committing.

- [ ] **Step 13: Commit**

```bash
git add ui/src/kit
git commit -m "feat(ui): the Kit page shows the app components"
```

---

### Task 10: Documents, full check, production build

**Files:**
- Modify: `docs/frontend-conventions.md`, `docs/STATUS.md`, `CLAUDE.md`, `docs/BACKLOG.md`

- [ ] **Step 1: `docs/frontend-conventions.md`** — in "Structure of `ui/`" replace

```
      <domain>/    app components, named for WHAT THEY ARE
```

with

```
      shell/       title bar, tabs, window controls, navbar, section sidebar
      marks/       the completion-matrix cell and its bit reading
      kpi/         the KPI tile
      wiki/        the wiki's inline tokens and blocks
      data-state/  read-but-empty, unreadable, empty category
      <domain>/    further app components, named for WHAT THEY ARE
```

In "UI primitives", after "Practical rule: …" paragraph add:

```markdown
**The extensions that exist** (cycle 2): `ButtonSize.Micro` (a tab's close), `Row` (sidebar
items), `Inline` (a wiki reference in running text), `Window` (window controls), `Compact`
(the search trigger); `ButtonVariant.Nav`, `Section`, `Ref`, `Chrome`, `ChromeDanger`,
`Field`. A size only sizes and a variant only colours: cva writes size classes after variant
classes, so a height inside a variant loses. The collapsible card is a set of Card parts
(`CardCollapsible`, `CardCollapsibleTrigger`, `CardCollapsibleContent`), because a prop
can't turn a `div` into Reka's collapsible root.
```

In "Tokens and Tailwind v4", after the Spacing bullet add:

```markdown
- **Letter spacing:** `tracking-*` is reset and holds two tokens, `tracking-nav` and
  `tracking-caps`; `cn()` knows them through `ThemeNamespace.Tracking`.
```

- [ ] **Step 2: `docs/STATUS.md`** — replace

```
      - [ ] 2. App components — tab strip, navbar, section sidebar, KPI tile, matrix cell,
            wiki inline tokens, data states
```

with

```
      - [x] **2. App components** (2026-09-10) — title bar and tabs, navbar, section
            sidebar, KPI tile, matrix cell in sprites or bars, wiki tokens and blocks, data
            states, collapsible card; presentational, on the Kit page. Spec
            `docs/superpowers/specs/2026-09-10-design-system-components-design.md`, plan
            `docs/superpowers/plans/archive/2026-09-10-design-system-components.md`
```

and in the 2026-09-10 session log replace `- [ ] Cycle 2 — app components.` with:

```markdown
- [x] **Cycle 2 — app components.** Scope, cell encoding, bit 2, edition tag, folders and
      shell agreed in conversation; the cell's scale chosen on a comparison page with the
      real sprites (40px, flat paper `#E9DADF` sampled from `paper_00.png`, symbol at 2×)
      and the rest delegated, marked **(delegated)** in the spec to revisit on first
      launch. Thirteen off-palette colours mapped, three tokens added.
- [ ] First launch: look at the delegated choices in the real window.
```

- [ ] **Step 3: `CLAUDE.md`** — in "State", replace

```
**The design export arrived on 2026-09-10, and cycle 1 of the design
system — tokens, font, `cn()`, i18n, 23 primitives on the development-only Kit page —
landed the same day.**
```

with

```
**The design export arrived on 2026-09-10, and cycles 1 and 2 of the design
system — tokens, font, `cn()`, i18n, 23 primitives, then the app components (title bar,
navbar, sidebar, KPI, matrix cell, wiki tokens, data states) on the development-only Kit
page — landed the same day.**
```

and replace

```
**`ui/` is still the verification page, plus the Kit page.** App components are cycle 2,
screens are cycle 3.
```

with

```
**`ui/` is still the verification page, plus the Kit page.** The app components exist
(cycle 2); screens and the shell's wiring are cycle 3.
```

- [ ] **Step 4: `docs/BACKLOG.md`** — append:

```markdown

---

## B13 — The marks map moves to `ipc` with the Completion screen (implementation, cycle 3)

Logged 2026-09-10, from cycle 2: `MarkCell` takes each column's symbol URLs as a prop, and
nothing in the app serves them yet. `crates/design-export`'s `marks.json` holds the column →
symbol map (with Delirium on the online-lobby sheet and its `symbolFallback`); DESIGN-BRIEF
§5.6 already says it belongs beside `BOSSES` in `ipc`. Needed: the map in `ipc`, the icon
protocol extended to the completion-widget sprites, and a test that every one of the twelve
columns resolves to two tiers.
```

- [ ] **Step 5: Full check** — Run: `pnpm check`. Expected: `all green`, skips only for missing samples.

- [ ] **Step 6: Production build** — Run:

```bash
pnpm --filter ui build
grep -rl "KitPage\|completion_widget\|background_completion_delirium" ui/dist || echo "no kit, no marks"
ls ui/dist/assets | grep -i "\.png$" || echo "no png"
```

Expected: `no kit, no marks` and `no png`.

- [ ] **Step 7: Commit**

```bash
git add docs/frontend-conventions.md docs/STATUS.md CLAUDE.md docs/BACKLOG.md
git commit -m "docs: the design system's second cycle in the conventions, status and backlog"
```
