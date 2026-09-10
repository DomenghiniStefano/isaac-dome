# Design system cycle 1 — foundations and primitives Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the Claude Design export of 2026-09-10 into the token layer and the 22 shadcn-vue primitives of `ui/`, each dressed in the IsaacDome skin and shown on a development-only Kit page.

**Architecture:** Tokens live in `ui/src/assets/theme/*.css` inside `@theme`, with Tailwind's default scales reset so off-system classes generate nothing. Primitives are shadcn-vue 2.8.2 (`reka-vega`) components written directly in their dressed form under `ui/src/components/ui/`, variants as `as const` objects in `variants.ts`. `cn()` learns the token names from the theme CSS; strings go through a typed `useMessages()`; the conventions scanner enforces what the type system can't.

**Tech Stack:** Vue 3.5, TypeScript 6, Vite 8, Tailwind CSS 4.3.3, Reka UI 2.10.4, @vueuse/core 14.4.0, @lucide/vue 1.44.0, class-variance-authority 0.7.1, clsx 2.1.1, tailwind-merge 3.6.0, lodash-es 4.18.1, vue-i18n 11.4.10, Vitest 5.0.0.

**Spec:** `docs/superpowers/specs/2026-09-10-design-system-foundations-design.md` — read it before Task 1; every "Decision N" below refers to it.

## Global Constraints

- Run commands from the repo root `C:\PersonalProjects\isaac-dome` (Git Bash). The frontend is the pnpm workspace `ui`: `pnpm --filter ui <cmd>`. Node 22, pnpm 10.33.0.
- **One theme, dark.** No `.dark` class, no `dark:` variant, no `@custom-variant dark`.
- **No visual literal in a class**: no `[12px]`, no `[#hex]`, no `opacity-50` (only `opacity-muted`/`opacity-disabled`, `opacity-0`, `opacity-100`), no `duration-150`, no colour alpha (`bg-x/50`), no `tw-animate-css` classes (`animate-in`, `fade-in`, `zoom-in`, `slide-in-from-*`), no `rounded-md`/`shadow-*`/`text-sm`/`font-bold` (they generate nothing once the scales are reset).
- **No glyphs `→ ← ↑ ↓ ⏎ ⌘ ✓` in source**: use `ArrowRightIcon`, `ArrowUpIcon`, `ArrowDownIcon`, `CornerDownLeftIcon`, `CheckIcon` from `@lucide/vue`.
- **Closed sets are `as const` objects** (`export const X = { A: 'a' } as const` plus `export type X = (typeof X)[keyof typeof X]`). Consumers pass `:variant="ButtonVariant.Outline"`, never `variant="outline"`. Never a hand-written `'a' | 'b'` union.
- **Variants live in `variants.ts`** inside the primitive's folder; `.vue` files import from `./variants`, never from `.`; `index.ts` re-exports by name. Never `export *`.
- `provide`/`inject` only with a typed `InjectionKey` symbol.
- No `<style>` blocks in SFCs. No `let`/`while` to accumulate; functional style, lodash-es named imports where more expressive.
- Visible text in templates goes through `useMessages().t(...)`, except under `ui/src/kit/` (development-only, Decision 11).
- Icons from `@lucide/vue`, sized with `size-*` classes, never `:size`.
- Primitives must not set `outline-none`/`outline-hidden` on focusable elements (the focus ring comes from `base.css`), except text inputs inside `Command`, where the caret is the focus.
- Commits: Conventional Commits in English, scope `ui` for `ui/`, `docs` for documents. **Never a `Co-Authored-By`, `Generated-By` or any AI trailer.** Stage explicit paths only (never `git add -A`: `IsaacDome design system.zip` in the root must stay untracked). The pre-commit hook runs `cargo fmt --check` and `pnpm scan`; never `--no-verify`.
- LF line endings; don't touch `.gitattributes`.
- Formatting: run `pnpm --filter ui format` before checking; the Tailwind Prettier plugin reorders classes, that's expected.
- **Every task ends green** on: `pnpm typecheck`, `pnpm lint`, `pnpm format:check`, `pnpm scan`, and — from Task 2 — `pnpm ui:test`. Tasks that touch CSS or components also run `pnpm --filter ui build`.
- Primitive source of truth for anything not spelled out here: `pnpm dlx shadcn-vue@2.8.2 add <name> --view` (registry version), styled per the spec's catalogue.

---

### Task 1: A typecheck that checks

`vue-tsc --noEmit` on `ui/tsconfig.json` (a solution file with `"files": []` and `references`) checks no file (spec Decision 12). Build mode does.

**Files:**
- Modify: `ui/package.json` (scripts `build`, `typecheck`)

**Interfaces:**
- Produces: `pnpm typecheck` = `vue-tsc --build --force` over `tsconfig.app.json` and `tsconfig.node.json`.

- [ ] **Step 1: Prove the current script misses a type error**

Create `ui/src/typecheckProbe.ts` (temporary, never committed):

```ts
export const deliberate: number = 'not a number'
```

Run: `pnpm typecheck; echo "exit=$?"`
Expected: `exit=0` — the error is not reported. This is the bug.

- [ ] **Step 2: Switch both scripts to build mode**

In `ui/package.json`, replace the two script lines:

```json
    "build": "vue-tsc --noEmit && vite build",
```
```json
    "typecheck": "vue-tsc --noEmit",
```

with:

```json
    "build": "vue-tsc --build --force && vite build",
```
```json
    "typecheck": "vue-tsc --build --force",
```

- [ ] **Step 3: Verify the probe is now caught**

Run: `pnpm typecheck; echo "exit=$?"`
Expected: non-zero exit, output contains `src/typecheckProbe.ts(1,14): error TS2322: Type 'string' is not assignable to type 'number'.`

- [ ] **Step 4: Remove the probe and verify the repo is clean**

Run: `rm ui/src/typecheckProbe.ts && pnpm typecheck; echo "exit=$?"`
Expected: `exit=0`, no `error TS` lines (measured during planning: 0 errors on the current code).

- [ ] **Step 5: Lint, format, scan**

Run: `pnpm lint && pnpm format:check && pnpm scan`
Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add ui/package.json
git commit -m "fix(ui): typecheck checks the project, not an empty solution file" -m "ui/tsconfig.json is a solution file (files: [] plus references), and vue-tsc --noEmit on it checks nothing: a deliberate type error passed with exit 0. Build mode walks the references and reports it. The current code has no errors in build mode."
```

---

### Task 2: `@/` alias, Vitest, and the theme key reader

**Files:**
- Modify: `ui/tsconfig.app.json`, `ui/tsconfig.json`, `ui/vite.config.ts`, `ui/package.json`, `package.json`, `scripts/check`
- Create: `ui/src/lib/design/themeKeys.ts`
- Test: `ui/src/lib/design/themeKeys.test.ts`

**Interfaces:**
- Produces: `ThemeNamespace` (`Text: 'text'`, `Font: 'font'`, `Spacing: 'spacing'`, `Radius: 'radius'`, `Ease: 'ease'`, `Animate: 'animate'`, `TransitionDuration: 'transition-duration'`) and `themeKeys(css: string, namespace: ThemeNamespace): string[]` from `@/lib/design/themeKeys`. `pnpm ui:test` at the root.

- [ ] **Step 1: Install**

Run:
```bash
pnpm --filter ui add lodash-es@4.18.1
pnpm --filter ui add -D vitest@5.0.0 @types/lodash-es@4.17.12
```
Expected: both succeed (a pnpm warning about ignored build scripts is fine).

- [ ] **Step 2: Alias in `ui/tsconfig.app.json`**

Replace the whole file with:

```jsonc
{
  "extends": "@vue/tsconfig/tsconfig.dom.json",
  "compilerOptions": {
    "tsBuildInfoFile": "./node_modules/.tmp/tsconfig.app.tsbuildinfo",
    "types": ["vite/client"],
    "allowArbitraryExtensions": true,
    "paths": {
      "@/*": ["./src/*"]
    },

    /* Linting */
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "erasableSyntaxOnly": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src/**/*.ts", "src/**/*.tsx", "src/**/*.vue"]
}
```

- [ ] **Step 3: Alias in `ui/tsconfig.json`**

Replace the whole file with:

```jsonc
{
  "files": [],
  // Repeated here only because the shadcn-vue CLI looks for the alias in this file.
  "compilerOptions": {
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "references": [
    { "path": "./tsconfig.app.json" },
    { "path": "./tsconfig.node.json" }
  ]
}
```

- [ ] **Step 4: Alias and Vitest in `ui/vite.config.ts`**

Replace the whole file with:

```ts
/// <reference types="vitest/config" />
import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  resolve: {
    alias: { '@': fileURLToPath(new URL('./src', import.meta.url)) },
  },
  server: { port: 1420, strictPort: true },
  test: {
    include: ['src/**/*.test.ts'],
    // Vitest doesn't load CSS by default and hands a `?raw` import an empty string.
    // `cn()` reads its token names from the theme files, so those load as source. No `$`
    // anchor: the module id ends in `?raw`.
    css: { include: [/src\/assets\/theme\/.+\.css/] },
  },
})
```

- [ ] **Step 5: Scripts**

In `ui/package.json` add to `scripts`, after `"scan"`:

```json
    "test": "vitest run",
```

In the root `package.json` add to `scripts`, after `"ui:dev"`:

```json
    "ui:test": "pnpm --filter ui test",
```

In `scripts/check`, after the line `run 'pnpm typecheck' pnpm typecheck` add:

```sh
run 'pnpm ui:test' pnpm ui:test
```

- [ ] **Step 6: Write the failing test**

Create `ui/src/lib/design/themeKeys.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { ThemeNamespace, themeKeys } from './themeKeys'

describe('themeKeys', () => {
  it('returns every name declared in the namespace, once', () => {
    const css = [
      '@theme {',
      '  --text-body: 14px;',
      '  --text-body--line-height: 1.5;',
      '  --text-caption: 12px;',
      '}',
    ].join('\n')
    expect(themeKeys(css, ThemeNamespace.Text)).toEqual(['body', 'caption'])
  })

  it('skips the namespace reset', () => {
    const css = '--radius-*: initial;\n--radius-input: 4px;'
    expect(themeKeys(css, ThemeNamespace.Radius)).toEqual(['input'])
  })

  it('keeps a multi-word name whole', () => {
    const css = '--spacing-row-compact: 24px;'
    expect(themeKeys(css, ThemeNamespace.Spacing)).toEqual(['row-compact'])
  })

  it('reads a hyphenated namespace', () => {
    const css = '--transition-duration-tap: 80ms;'
    expect(themeKeys(css, ThemeNamespace.TransitionDuration)).toEqual(['tap'])
  })

  it('ignores references, other namespaces and look-alike variables', () => {
    const css = [
      '--color-text-muted: #ffffff;',
      '--default-font-family: var(--font-pixel);',
      '--default-transition-duration: 0ms;',
      '--animate-tap-in: rise-in var(--transition-duration-tap) var(--ease-tap);',
      'font-size: var(--text-body);',
    ].join('\n')
    expect(themeKeys(css, ThemeNamespace.Text)).toEqual([])
    expect(themeKeys(css, ThemeNamespace.Font)).toEqual([])
    expect(themeKeys(css, ThemeNamespace.TransitionDuration)).toEqual([])
    expect(themeKeys(css, ThemeNamespace.Ease)).toEqual([])
  })
})
```

- [ ] **Step 7: Run it to verify it fails**

Run: `pnpm ui:test`
Expected: FAIL — `Failed to resolve import "./themeKeys"` (or "Cannot find module").

- [ ] **Step 8: Implement**

Create `ui/src/lib/design/themeKeys.ts`:

```ts
import { uniq } from 'lodash-es'

// The Tailwind namespaces whose custom names `cn()` has to know. A namespace is the
// prefix of a theme variable: `--text-body` belongs to `text`.
export const ThemeNamespace = {
  Text: 'text',
  Font: 'font',
  Spacing: 'spacing',
  Radius: 'radius',
  Ease: 'ease',
  Animate: 'animate',
  TransitionDuration: 'transition-duration',
} as const
export type ThemeNamespace = (typeof ThemeNamespace)[keyof typeof ThemeNamespace]

// `--<namespace>-<name>:` at a declaration. The name stops before a `--` sub-property, so
// `--text-body--line-height` still names `body`; the `--text-*: initial` reset has no name
// and doesn't match; `var(--text-body)` isn't followed by a colon and doesn't either.
const declaration = (namespace: ThemeNamespace) =>
  new RegExp(`--${namespace}-([a-z0-9]+(?:-[a-z0-9]+)*)(?:--[a-z0-9-]+)?\\s*:`, 'g')

export const themeKeys = (css: string, namespace: ThemeNamespace): string[] =>
  uniq([...css.matchAll(declaration(namespace))].map(([, name]) => name ?? ''))
```

- [ ] **Step 9: Run the tests**

Run: `pnpm ui:test`
Expected: PASS, 5 tests.

- [ ] **Step 10: Checks**

Run: `pnpm --filter ui format && pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan`
Expected: all pass.

- [ ] **Step 11: Commit**

```bash
git add ui/package.json ui/pnpm-lock.yaml pnpm-lock.yaml package.json scripts/check ui/tsconfig.json ui/tsconfig.app.json ui/vite.config.ts ui/src/lib/design/themeKeys.ts ui/src/lib/design/themeKeys.test.ts
git commit -m "build(ui): the @/ alias, Vitest, and a reader for the theme's token names" -m "themeKeys reads the custom names of a Tailwind namespace out of a theme file, so cn() can learn them without a second copy in TypeScript. Vitest loads the theme CSS as source: by default it hands a ?raw CSS import an empty string. pnpm ui:test joins scripts/check."
```

(If `ui/pnpm-lock.yaml` doesn't exist, drop it from `git add`: the workspace lockfile is the root one.)

---

### Task 3: The tokens, the font, and what already used the old ones

Spec Decisions 1, 3, 4, 5, 6, 12. After this task Tailwind's default colours, sizes, weights, radii, shadows, curves and animations generate nothing, so `App.vue`, `WikiInline.vue` and `WikiBlocks.vue` migrate in the same commit.

**Files:**
- Create: `ui/src/assets/theme/colors.css`, `typography.css`, `spacing.css`, `radius.css`, `shadow.css`, `opacity.css`, `motion.css`; `ui/src/assets/base.css`; `ui/src/assets/utilities.css`; `ui/src/assets/fonts/determination/{determination.ttf,license.txt,readme.txt}`
- Modify: `ui/src/assets/main.css`, `ui/src/App.vue`, `ui/src/components/WikiInline.vue`, `ui/src/components/WikiBlocks.vue`
- Test: `ui/src/lib/design/themeKeys.test.ts`

**Interfaces:**
- Consumes: `themeKeys`, `ThemeNamespace` (Task 2).
- Produces: every class named in the spec's token tables: colours (`bg-primary`, `text-foreground-soft`, `border-hairline`, `bg-state-done-surface`, …), `text-title|heading|body|control|row|caption|label|micro`, `h-row`, `h-row-compact`, `h-row-wide`, `h-control`/`size-control`, `w-scrollbar`, `rounded-cell`, `rounded-input`, `duration-tap|panel|sheet|loop`, `ease-tap|panel|sheet|frame`, `animate-tap-in|panel-rise|panel-drop|panel-open|sheet-rise|skeleton`, `opacity-muted|disabled`, `pixelated`, `hatch-placeholder`, `hatch-unknown`, `font-pixel`.

- [ ] **Step 1: Write the failing test on the real theme files**

Replace `ui/src/lib/design/themeKeys.test.ts` with the Task 2 file plus two imports and one test. Full file:

```ts
import { describe, expect, it } from 'vitest'
import motion from '@/assets/theme/motion.css?raw'
import typography from '@/assets/theme/typography.css?raw'
import { ThemeNamespace, themeKeys } from './themeKeys'

describe('themeKeys', () => {
  it('returns every name declared in the namespace, once', () => {
    const css = [
      '@theme {',
      '  --text-body: 14px;',
      '  --text-body--line-height: 1.5;',
      '  --text-caption: 12px;',
      '}',
    ].join('\n')
    expect(themeKeys(css, ThemeNamespace.Text)).toEqual(['body', 'caption'])
  })

  it('skips the namespace reset', () => {
    const css = '--radius-*: initial;\n--radius-input: 4px;'
    expect(themeKeys(css, ThemeNamespace.Radius)).toEqual(['input'])
  })

  it('keeps a multi-word name whole', () => {
    const css = '--spacing-row-compact: 24px;'
    expect(themeKeys(css, ThemeNamespace.Spacing)).toEqual(['row-compact'])
  })

  it('reads a hyphenated namespace', () => {
    const css = '--transition-duration-tap: 80ms;'
    expect(themeKeys(css, ThemeNamespace.TransitionDuration)).toEqual(['tap'])
  })

  it('ignores references, other namespaces and look-alike variables', () => {
    const css = [
      '--color-text-muted: #ffffff;',
      '--default-font-family: var(--font-pixel);',
      '--default-transition-duration: 0ms;',
      '--animate-tap-in: rise-in var(--transition-duration-tap) var(--ease-tap);',
      'font-size: var(--text-body);',
    ].join('\n')
    expect(themeKeys(css, ThemeNamespace.Text)).toEqual([])
    expect(themeKeys(css, ThemeNamespace.Font)).toEqual([])
    expect(themeKeys(css, ThemeNamespace.TransitionDuration)).toEqual([])
    expect(themeKeys(css, ThemeNamespace.Ease)).toEqual([])
  })

  it('reads the real theme files as source, not as compiled CSS', () => {
    expect(typography).toContain('@theme')
    expect(themeKeys(typography, ThemeNamespace.Text)).toEqual([
      'title',
      'heading',
      'body',
      'control',
      'row',
      'caption',
      'label',
      'micro',
    ])
    expect(themeKeys(motion, ThemeNamespace.TransitionDuration)).toEqual([
      'tap',
      'panel',
      'sheet',
      'loop',
    ])
  })
})
```

- [ ] **Step 2: Run it to verify it fails**

Run: `pnpm ui:test`
Expected: FAIL — cannot resolve `@/assets/theme/motion.css?raw`.

- [ ] **Step 3: The font**

Run:
```bash
mkdir -p ui/src/assets/fonts/determination
unzip -j -o "IsaacDome design system.zip" "uploads/determination/determination.ttf" "uploads/determination/license.txt" "uploads/determination/readme.txt" -d ui/src/assets/fonts/determination
ls ui/src/assets/fonts/determination
```
Expected: `determination.ttf  license.txt  readme.txt`. (The zip is in the repo root, untracked. `license.txt` must say CC BY 3.0.)

- [ ] **Step 4: `ui/src/assets/theme/colors.css`**

```css
/* Colour tokens. One theme, the dark one ("il seminterrato"): values live directly in
   @theme, with no :root/.dark indirection to switch (spec Decision 1).
   Source: Shadcn Kit.dc.html (dark), Claude Design export of 2026-09-10. */
@theme {
  /* Tailwind's palette goes: bg-red-500 and text-white generate nothing. */
  --color-*: initial;

  /* shadcn roles */
  --color-background: #150e0d;
  --color-foreground: #f2ecec;
  --color-card: #1b120e;
  --color-card-foreground: #f2ecec;
  --color-popover: #1b120e;
  --color-popover-foreground: #f2ecec;
  --color-primary: #901800;
  --color-primary-foreground: #f2ecec;
  --color-secondary: #3a251d;
  --color-secondary-foreground: #f2ecec;
  --color-muted: #241812;
  --color-muted-foreground: #bfb2ac;
  --color-accent: #3a251d;
  --color-accent-foreground: #f2ecec;
  --color-destructive: #fc0000;
  --color-destructive-foreground: #e8c4bf;
  --color-border: #5a3a2c;
  --color-input: #8a5a46;
  /* Focus, and active filters from cycle 2. Nothing else is cyan. */
  --color-ring: #00a6cc;

  /* Domain roles */
  --color-sheet: #2a1c17;
  --color-band: #6c4437;
  --color-band-foreground: #f2ecec;
  --color-data: #1b120e;
  --color-row-alt: #20140f;
  --color-row-hover: #241812;
  --color-hairline: #2e1e17;
  --color-primary-hover: #a81c00;
  --color-primary-active: #6c0c00;
  --color-primary-edge: #4a0a00;
  --color-secondary-hover: #492e20;
  --color-secondary-edge: #6c4437;
  --color-selection-edge: #fc0000;
  --color-destructive-surface: #2a0b08;
  --color-tooltip: #492e20;
  /* The alpha lives here, never in a class. */
  --color-overlay: rgb(12 7 6 / 0.8);
  --color-highlight: #ebd9c4;
  --color-foreground-soft: #d5c8c2;
  --color-subtle-foreground: #a69690;
  --color-faint-foreground: #8a7a72;
  /* "Image not yet extracted": temporary, it will arrive. */
  --color-placeholder-stripe-a: #3a251d;
  --color-placeholder-stripe-b: #241812;

  /* States — the brief's, never reused for anything else. Gold means "unlockable now". */
  --color-state-done: #2cc53f;
  --color-state-done-foreground: #8fe79b;
  --color-state-done-surface: #132916;
  --color-state-now: #be8c32;
  --color-state-now-foreground: #e8be6e;
  --color-state-now-surface: #2b2010;
  --color-state-blocked: #6c4437;
  --color-state-blocked-foreground: #bfb2ac;
  --color-state-blocked-surface: #241812;
  /* "We can't read it", which is not zero. */
  --color-state-unknown: #8a7a72;
  --color-state-unknown-foreground: #bfb2ac;
  --color-state-unknown-stripe-a: #2e1e17;
  --color-state-unknown-stripe-b: #241812;
  --color-state-unexpected: #fc0000;
  --color-state-unexpected-foreground: #e8c4bf;
  --color-state-unexpected-surface: #2a0b08;
  --color-challenge: #a855a8;
  --color-challenge-foreground: #dda8dd;
  --color-challenge-surface: #25102a;
}
```

- [ ] **Step 5: `ui/src/assets/theme/typography.css`**

```css
/* Determination — FontStruct, CC BY 3.0 (license.txt beside the file), one weight.
   Served by the app: no CDN, the app works offline. */
@font-face {
  font-family: 'Determination';
  src: url('../fonts/determination/determination.ttf') format('truetype');
  font-weight: 400;
  font-display: block;
}

@theme {
  --font-*: initial;
  /* One weight: font-bold would be a smeared fake. Emphasis is colour. */
  --font-weight-*: initial;
  --font-pixel: 'Determination', monospace;
  /* What preflight reads for the document's font. */
  --default-font-family: var(--font-pixel);

  /* The kit's scale (Shadcn Kit.dc.html), in px: a pixel font is crisp on whole pixels. */
  --text-*: initial;
  --text-title: 30px;
  --text-title--line-height: 1.3;
  --text-heading: 21px;
  --text-heading--line-height: 1.35;
  --text-body: 14px;
  --text-body--line-height: 1.5;
  --text-control: 13px;
  --text-control--line-height: 1;
  --text-row: 13px;
  --text-row--line-height: 1.35;
  --text-caption: 12px;
  --text-caption--line-height: 1.45;
  --text-label: 11px;
  --text-label--line-height: 1.3;
  --text-micro: 10px;
  --text-micro--line-height: 1.2;
}
```

- [ ] **Step 6: `ui/src/assets/theme/spacing.css`**

```css
/* Padding and gaps use Tailwind's 4px grid. Named tokens exist only for dimensions that
   mean something. */
@theme {
  /* Table rows, Chrome e Stati.dc.html "Densità": normal is the kit's default. */
  --spacing-row: 30px;
  --spacing-row-compact: 24px;
  --spacing-row-wide: 40px;
  /* Height of button, input, select; side of an icon button. */
  --spacing-control: 34px;
  --spacing-scrollbar: 12px;
  /* The game's icons are 32x32 native: doubled, they stay sharp. */
  --spacing-sprite: 4rem;
  /* Achievement icons are painted 263x176 images, not pixel art:
     at half their native size they stay readable without going mushy. */
  --spacing-achievement: 5.5rem;
}
```

- [ ] **Step 7: `radius.css`, `shadow.css`, `opacity.css`**

`ui/src/assets/theme/radius.css`:
```css
/* Paper, cards, buttons and panels have no radius at all. rounded-full stays for chips:
   it isn't a theme value. */
@theme {
  --radius-*: initial;
  --radius-cell: 2px;
  --radius-input: 4px;
}
```

`ui/src/assets/theme/shadow.css`:
```css
/* The skin is flat ("bordo pixel piatto"): no shadow token exists. */
@theme {
  --shadow-*: initial;
}
```

`ui/src/assets/theme/opacity.css`:
```css
@theme {
  --opacity-muted: 0.6;
  --opacity-disabled: 0.4;
}
```

- [ ] **Step 8: `ui/src/assets/theme/motion.css`**

```css
/* Motion.dc.html: the game doesn't interpolate, so neither do we. Everything moves on
   steps(); data never moves. */
@theme {
  /* A bare transition-* class is instant and never a curve: hover and active can't lag. */
  --default-transition-duration: 0ms;
  --default-transition-timing-function: steps(1);

  --transition-duration-tap: 80ms;
  --transition-duration-panel: 120ms;
  --transition-duration-sheet: 200ms;
  --transition-duration-loop: 640ms;

  --ease-*: initial;
  --ease-tap: steps(2);
  --ease-panel: steps(3);
  --ease-sheet: steps(5);
  /* Loops are frame-by-frame keyframes: each segment jumps. */
  --ease-frame: steps(1);

  --animate-*: initial;
  --animate-tap-in: rise-in var(--transition-duration-tap) var(--ease-tap);
  --animate-panel-rise: rise-in var(--transition-duration-panel) var(--ease-panel);
  --animate-panel-drop: drop-in var(--transition-duration-panel) var(--ease-panel);
  --animate-panel-open: open-panel var(--transition-duration-panel) var(--ease-panel);
  --animate-sheet-rise: rise-in var(--transition-duration-sheet) var(--ease-sheet);
  --animate-skeleton: skeleton-sweep var(--transition-duration-loop) var(--ease-frame)
    infinite;

  /* Translations are multiples of 4px. transform, not translate, so they compose with
     the translate-* utilities a centred dialog uses. */
  @keyframes rise-in {
    0% {
      transform: translateY(8px);
      opacity: 0;
    }
    50% {
      transform: translateY(4px);
      opacity: 1;
    }
    100% {
      transform: translateY(0);
      opacity: 1;
    }
  }
  @keyframes drop-in {
    0% {
      transform: translateY(-8px);
      opacity: 0;
    }
    50% {
      transform: translateY(-4px);
      opacity: 1;
    }
    100% {
      transform: translateY(0);
      opacity: 1;
    }
  }
  @keyframes open-panel {
    0% {
      transform: translateY(-4px);
      opacity: 0;
    }
    100% {
      transform: translateY(0);
      opacity: 1;
    }
  }
  @keyframes skeleton-sweep {
    0%,
    100% {
      background-color: var(--color-secondary);
    }
    50% {
      background-color: var(--color-secondary-hover);
    }
  }
}
```

- [ ] **Step 9: `ui/src/assets/base.css`**

```css
@layer base {
  html {
    /* One weight: never a synthesised bold. Italic synthesis stays: the kit uses it. */
    font-synthesis-weight: none;
    background-color: var(--color-background);
    color: var(--color-foreground);
    font-size: var(--text-body);
    line-height: var(--text-body--line-height);
  }

  /* One focus ring for the whole app (Chrome e Stati.dc.html): never blurred, never
     rounded. Rows move it inside with -outline-offset-2. */
  :focus-visible {
    outline: 2px solid var(--color-ring);
    outline-offset: 2px;
  }

  /* Always visible, 12px: in a long table the position is information. Only the webkit
     pseudo-elements: in Chromium (WebView2) a non-initial scrollbar-width or
     scrollbar-color makes these rules be ignored. */
  ::-webkit-scrollbar {
    width: var(--spacing-scrollbar);
    height: var(--spacing-scrollbar);
  }
  ::-webkit-scrollbar-track {
    background-color: var(--color-data);
    border-left: 1px solid var(--color-secondary);
  }
  ::-webkit-scrollbar-thumb {
    background-color: var(--color-band);
    border: 1px solid var(--color-input);
  }
  ::-webkit-scrollbar-thumb:hover {
    background-color: var(--color-input);
  }
  ::-webkit-scrollbar-corner {
    background-color: var(--color-data);
  }
}

/* Motion.dc.html: loops become static, entrances instant. */
@media (prefers-reduced-motion: reduce) {
  *,
  ::before,
  ::after {
    animation-duration: 0ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0ms !important;
  }
}
```

- [ ] **Step 10: `ui/src/assets/utilities.css`**

```css
/* Sprites are pixel art: the browser's interpolation smears them. */
@utility pixelated {
  image-rendering: pixelated;
}

/* Two gaps that must never look alike (DESIGN-BRIEF.md §5.6): an image that isn't there
   yet, and a value we can't read. Same geometry on the 4px grid, different stripes; where
   hatch-unknown stands for data it also gets a dashed edge and a "?". */
@utility hatch-placeholder {
  background-image: repeating-linear-gradient(
    45deg,
    var(--color-placeholder-stripe-a) 0 var(--spacing),
    var(--color-placeholder-stripe-b) var(--spacing) calc(var(--spacing) * 2)
  );
}

@utility hatch-unknown {
  background-image: repeating-linear-gradient(
    45deg,
    var(--color-state-unknown-stripe-a) 0 var(--spacing),
    var(--color-state-unknown-stripe-b) var(--spacing) calc(var(--spacing) * 2)
  );
}
```

- [ ] **Step 11: `ui/src/assets/main.css`**

Replace the whole file with:

```css
@import 'tailwindcss';
@import './theme/colors.css';
@import './theme/typography.css';
@import './theme/spacing.css';
@import './theme/radius.css';
@import './theme/shadow.css';
@import './theme/opacity.css';
@import './theme/motion.css';
@import './base.css';
@import './utilities.css';
```

- [ ] **Step 12: Run the tests**

Run: `pnpm ui:test`
Expected: PASS, 6 tests.

- [ ] **Step 13: Migrate `App.vue`**

In `ui/src/App.vue`:

1. Replace `      return c.bits === 0 ? 'text-mark-none' : 'text-mark-done'` with `      return c.bits === 0 ? 'text-faint-foreground' : 'text-state-done-foreground'`.
2. Replace `      return 'text-mark-unknown italic'` with `      return 'text-state-unknown-foreground italic'`.
3. Replace `      return 'text-mark-partial font-bold'` with `      return 'text-state-unexpected-foreground'`.
4. Replace `  <main class="flex flex-col gap-6 p-6 font-mono text-sm">` with `  <main class="flex flex-col gap-6 p-6 text-foreground-soft">`.
5. Replace `    <p v-if="error" class="text-mark-partial">` with `    <p v-if="error" class="text-destructive">`.
6. Replace `      <h1 class="text-lg font-bold">Status</h1>` with `      <h1 class="text-heading text-foreground">Status</h1>`.
7. Replace **every** `class="font-bold"` with `class="text-foreground"` (Edit with `replace_all: true`).
8. Replace `          class="border"` with `          class="border border-input bg-data"`.

- [ ] **Step 14: Migrate the wiki verification components**

In `ui/src/components/WikiInline.vue` replace `      return 'font-bold'` with `      return 'text-foreground'`.
In `ui/src/components/WikiBlocks.vue` replace every `class="font-bold"` with `class="text-foreground"` (`replace_all: true`).

- [ ] **Step 15: Verify nothing still uses a removed class**

Run: `grep -rnE "mark-(none|done|partial|unknown)|font-(bold|mono|medium|semibold)|text-(xs|sm|base|lg|xl)\b|rounded-(sm|md|lg|xl)|shadow-(xs|sm|md)" ui/src --include=*.vue --include=*.ts`
Expected: no output.

- [ ] **Step 16: Build and inspect the CSS**

Run:
```bash
pnpm --filter ui build
CSS=$(ls ui/dist/assets/*.css)
grep -ci "determination" $CSS
ls ui/dist/assets | grep -c determination
grep -o "\.text-heading" $CSS | head -1
grep -o -- "--color-state-done-foreground:[^;]*" $CSS | head -1
grep -o "::-webkit-scrollbar-thumb" $CSS | head -1
grep -cE "\.font-mono|\.text-sm|\.font-bold|\.text-lg" $CSS
```
Expected, in order: a number ≥ 1; `1`; `.text-heading`; `--color-state-done-foreground:#8fe79b`; `::-webkit-scrollbar-thumb`; `0`.

- [ ] **Step 17: Checks**

Run: `pnpm --filter ui format && pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test`
Expected: all pass.

- [ ] **Step 18: Commit**

```bash
git add ui/src/assets ui/src/App.vue ui/src/components/WikiInline.vue ui/src/components/WikiBlocks.vue ui/src/lib/design/themeKeys.test.ts
git commit -m "feat(ui): the design tokens, one dark theme, and the Determination font" -m "Tokens from the Claude Design export in @theme, one file per family. Tailwind's default colours, sizes, weights, radii, shadows, curves and animations are reset, so an off-system class generates nothing. Motion runs on steps(); one focus ring; an always-visible 12px scrollbar. The verification pages move to the new tokens in the same commit, since their old classes stop generating."
```

---

### Task 4: `cn()` that knows the tokens

Spec Decision 9. Default tailwind-merge reads `text-body` as a colour and drops it next to `text-foreground`.

**Files:**
- Create: `ui/src/lib/cn.ts`
- Test: `ui/src/lib/cn.test.ts`

**Interfaces:**
- Consumes: `themeKeys`, `ThemeNamespace` (Task 2); theme files (Task 3).
- Produces: `cn(...inputs: ClassValue[]): string` from `@/lib/cn`, used by every primitive.

- [ ] **Step 1: Install**

Run: `pnpm --filter ui add clsx@2.1.1 tailwind-merge@3.6.0`

- [ ] **Step 2: Write the failing test**

Create `ui/src/lib/cn.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { cn } from './cn'

describe('cn', () => {
  it('keeps a font size beside a text colour', () => {
    expect(cn('text-body', 'text-foreground')).toBe('text-body text-foreground')
  })

  it('lets a later font size replace an earlier one', () => {
    expect(cn('text-body', 'text-caption')).toBe('text-caption')
  })

  it('lets a later spacing token replace an earlier one', () => {
    expect(cn('h-row', 'h-control')).toBe('h-control')
  })

  it('lets a later radius replace an earlier one', () => {
    expect(cn('rounded-input', 'rounded-cell')).toBe('rounded-cell')
  })

  it('lets a later colour replace an earlier one', () => {
    expect(cn('bg-primary', 'bg-secondary')).toBe('bg-secondary')
  })

  it('lets a later duration and easing replace earlier ones', () => {
    expect(cn('duration-tap ease-tap', 'duration-panel ease-panel')).toBe(
      'duration-panel ease-panel',
    )
  })

  it('lets a later animation replace an earlier one', () => {
    expect(cn('animate-panel-rise', 'animate-panel-drop')).toBe('animate-panel-drop')
  })

  it('drops falsy inputs', () => {
    expect(cn('text-body', false, undefined, 'text-foreground')).toBe(
      'text-body text-foreground',
    )
  })
})
```

- [ ] **Step 3: Run it to verify it fails**

Run: `pnpm ui:test`
Expected: FAIL — cannot resolve `./cn`.

- [ ] **Step 4: Implement**

Create `ui/src/lib/cn.ts`:

```ts
import type { ClassValue } from 'clsx'
import { clsx } from 'clsx'
import { extendTailwindMerge } from 'tailwind-merge'
import motion from '@/assets/theme/motion.css?raw'
import radius from '@/assets/theme/radius.css?raw'
import spacing from '@/assets/theme/spacing.css?raw'
import typography from '@/assets/theme/typography.css?raw'
import { ThemeNamespace, themeKeys } from '@/lib/design/themeKeys'

// shadcn's cn() is twMerge(clsx(...)) with no configuration, and with our tokens that
// drops classes silently: tailwind-merge only knows t-shirt font sizes, reads `text-body`
// as a colour, and cn('text-body', 'text-foreground') keeps only the second. The names
// come from the theme files themselves, so a token is still declared in one place.
// tailwind-merge has no theme key for durations: they extend the class group instead.
const merge = extendTailwindMerge({
  extend: {
    theme: {
      text: themeKeys(typography, ThemeNamespace.Text),
      font: themeKeys(typography, ThemeNamespace.Font),
      spacing: themeKeys(spacing, ThemeNamespace.Spacing),
      radius: themeKeys(radius, ThemeNamespace.Radius),
      ease: themeKeys(motion, ThemeNamespace.Ease),
      animate: themeKeys(motion, ThemeNamespace.Animate),
    },
    classGroups: {
      duration: [{ duration: themeKeys(motion, ThemeNamespace.TransitionDuration) }],
    },
  },
})

export const cn = (...inputs: ClassValue[]) => merge(clsx(inputs))
```

- [ ] **Step 5: Run the tests**

Run: `pnpm ui:test`
Expected: PASS, 14 tests (6 themeKeys + 8 cn).

- [ ] **Step 6: Checks**

Run: `pnpm --filter ui format && pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan`

- [ ] **Step 7: Commit**

```bash
git add ui/package.json pnpm-lock.yaml ui/src/lib/cn.ts ui/src/lib/cn.test.ts
git commit -m "feat(ui): cn() learns the theme's token names from the CSS" -m "Unconfigured, tailwind-merge reads text-body as a colour and drops it beside text-foreground. The merger is extended with every custom name of the text, font, spacing, radius, ease and animate namespaces and with the duration class group, read from the theme files rather than copied into TypeScript."
```

---

### Task 5: Typed i18n

Spec Decision 10. vue-i18n's own `t()` accepts any string, so components use `useMessages()`, whose `t` only takes keys that exist.

**Files:**
- Create: `ui/src/i18n/locale.ts`, `ui/src/i18n/messageKey.ts`, `ui/src/i18n/messageKey.typecheck.ts`, `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts`, `ui/src/i18n/index.ts`
- Modify: `ui/src/main.ts`
- Test: `ui/src/i18n/locale.test.ts`

**Interfaces:**
- Produces: `Locale` (`It: 'it'`, `En: 'en'`), `resolveLocale(languages: readonly string[]): Locale` from `@/i18n/locale`; `MessageKey<T>` from `@/i18n/messageKey`; `MessageSchema` from `@/i18n/messages/it`; `i18n` (the plugin) and `useMessages(): { t: (key: MessageKey<MessageSchema>) => string }` from `@/i18n`. Message keys available after this task: `ui.close`.

- [ ] **Step 1: Install**

Run: `pnpm --filter ui add vue-i18n@11.4.10`

- [ ] **Step 2: Write the failing test**

Create `ui/src/i18n/locale.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { Locale, resolveLocale } from './locale'

describe('resolveLocale', () => {
  it('picks Italian from a regional tag', () => {
    expect(resolveLocale(['it-IT', 'en'])).toBe(Locale.It)
  })

  it('picks the first language the app has', () => {
    expect(resolveLocale(['de-DE', 'en-GB'])).toBe(Locale.En)
  })

  it('ignores case', () => {
    expect(resolveLocale(['IT'])).toBe(Locale.It)
  })

  it('falls back to English when no language matches', () => {
    expect(resolveLocale(['de'])).toBe(Locale.En)
  })

  it('falls back to English with no languages at all', () => {
    expect(resolveLocale([])).toBe(Locale.En)
  })
})
```

- [ ] **Step 3: Run it to verify it fails**

Run: `pnpm ui:test`
Expected: FAIL — cannot resolve `./locale`.

- [ ] **Step 4: Implement `locale.ts`**

Create `ui/src/i18n/locale.ts`:

```ts
export const Locale = { It: 'it', En: 'en' } as const
export type Locale = (typeof Locale)[keyof typeof Locale]

const locales: readonly string[] = Object.values(Locale)

const isLocale = (tag: string): tag is Locale => locales.includes(tag)

// `it-IT` becomes `it`: the app speaks one Italian and one English, not regional variants.
const primarySubtag = (language: string): string =>
  (language.split('-')[0] ?? '').toLowerCase()

// The first of the user's languages the app has, English otherwise. The preference in
// Settings arrives with that screen and will take precedence over this.
export const resolveLocale = (languages: readonly string[]): Locale =>
  languages.map(primarySubtag).find(isLocale) ?? Locale.En
```

- [ ] **Step 5: Run the tests**

Run: `pnpm ui:test`
Expected: PASS (5 new tests).

- [ ] **Step 6: Messages**

Create `ui/src/i18n/messages/it.ts`:

```ts
// Italian is the schema: en.ts is typed against it, so a key missing from English is a
// compile error, not a review note. Item, character and boss names are data, not
// messages: they stay in English and never appear here.
export const it = {
  ui: {
    close: 'Chiudi',
  },
}

export type MessageSchema = typeof it
```

Create `ui/src/i18n/messages/en.ts`:

```ts
import type { MessageSchema } from './it'

export const en: MessageSchema = {
  ui: {
    close: 'Close',
  },
}
```

- [ ] **Step 7: The key type and its compile-time probe**

Create `ui/src/i18n/messageKey.ts`:

```ts
// The dotted path of every string in a message tree: { ui: { close } } gives 'ui.close'.
export type MessageKey<T> = {
  [K in keyof T & string]: T[K] extends string ? K : `${K}.${MessageKey<T[K]>}`
}[keyof T & string]
```

Create `ui/src/i18n/messageKey.typecheck.ts`:

```ts
import type { MessageKey } from './messageKey'
import type { MessageSchema } from './messages/it'

// Checked by `pnpm typecheck`, not by a test runner: an existing key is accepted and a
// missing one is rejected. If MessageKey ever widened to `string`, the directive below
// would go unused and the typecheck would fail.
export const existingKey: MessageKey<MessageSchema> = 'ui.close'
// @ts-expect-error the key does not exist in the schema
export const missingKey: MessageKey<MessageSchema> = 'ui.nope'
```

- [ ] **Step 8: The plugin and `useMessages()`**

Create `ui/src/i18n/index.ts`:

```ts
import { createI18n, useI18n } from 'vue-i18n'
import { Locale, resolveLocale } from './locale'
import type { MessageKey } from './messageKey'
import { en } from './messages/en'
import { it, type MessageSchema } from './messages/it'

export const i18n = createI18n<[MessageSchema], Locale, false>({
  legacy: false,
  locale: resolveLocale(navigator.languages),
  fallbackLocale: Locale.En,
  messages: { [Locale.It]: it, [Locale.En]: en },
})

// vue-i18n's own t() accepts any string, so a misspelled key would compile and render the
// key itself. Components call this instead: the key is narrowed to the paths that exist.
// Global scope, so no component needs a local i18n instance.
export const useMessages = () => {
  const { t } = useI18n({ useScope: 'global' })
  return { t: (key: MessageKey<MessageSchema>): string => t(key) }
}
```

- [ ] **Step 9: Install the plugin**

Replace `ui/src/main.ts` with:

```ts
import { createApp } from 'vue'
import './assets/main.css'
import App from './App.vue'
import { i18n } from './i18n'

createApp(App).use(i18n).mount('#app')
```

- [ ] **Step 10: Prove the probe bites**

Temporarily change `'ui.nope'` in `messageKey.typecheck.ts` to `'ui.close'`, run `pnpm typecheck`.
Expected: fails with `error TS2578: Unused '@ts-expect-error' directive.`
Restore `'ui.nope'`, run `pnpm typecheck`. Expected: passes.

- [ ] **Step 11: Checks**

Run: `pnpm --filter ui format && pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test`

- [ ] **Step 12: Commit**

```bash
git add ui/package.json pnpm-lock.yaml ui/src/i18n ui/src/main.ts
git commit -m "feat(ui): i18n in Italian and English, with keys the compiler checks" -m "vue-i18n's t() accepts any string, so useMessages() narrows it to the paths of the Italian schema; English is typed against that schema. A committed type probe fails the typecheck if the key type ever widens. The initial locale is the first of the user's languages the app has, English otherwise."
```

---

### Task 6: The scanner learns the design system's rules

Spec Decision 13.

**Files:**
- Modify: `ui/scripts/scan-conventions.mjs`, `docs/frontend-conventions.md`

**Interfaces:**
- Produces: six new checks, the attribute-aware visible-text heuristic, and `src/kit/` as the development-only directory (excused from the visible-string check only).

- [ ] **Step 1: Write the probes that must be caught** (temporary, never committed)

Create `ui/src/ScanProbe.vue`:

```vue
<script setup lang="ts"></script>

<template>
  <div class="bg-primary/50 dark:bg-primary animate-in fade-in bg-[#ff0000]">→</div>
  <Button variant="outline" />
</template>
```

Create `ui/src/ScanProbeAttribute.vue` (must NOT be reported):

```vue
<script setup lang="ts"></script>

<template>
  <div class="has-[>svg]:grid-cols-2 *:[svg]:size-4"></div>
</template>
```

Create `ui/src/kit/ScanProbeKit.vue` (must NOT be reported):

```vue
<script setup lang="ts"></script>

<template>
  <p>Testo visibile nella pagina Kit</p>
</template>
```

- [ ] **Step 2: Run the scanner to see today's behaviour**

Run: `pnpm scan`
Expected (the bug and the gaps): `ScanProbeAttribute.vue: visible string in the template` and `kit/ScanProbeKit.vue: visible string in the template` are reported; none of the six new rules is.

- [ ] **Step 3: Rewrite the scanner**

Replace `ui/scripts/scan-conventions.mjs` with:

```js
import { readdirSync, readFileSync, statSync } from 'node:fs'
import { join, relative } from 'node:path'
import { fileURLToPath } from 'node:url'

const ROOT = fileURLToPath(new URL('..', import.meta.url))
const SRC = join(ROOT, 'src')
const IPC_DIR = join('src', 'lib', 'ipc')
const UI_DIR = join('src', 'components', 'ui')
// Development-only pages: main.ts imports them behind `import.meta.env.DEV`, so they never
// reach the production build and their text is never user-facing. The visible-string check
// is the one check they are excused from.
const DEV_ONLY_DIR = join('src', 'kit')

const STYLE_BLOCK = /<style[^>]*>([\s\S]*?)<\/style>/g
const STYLE_EXEMPTION = /^\s*\/\*\s*exception allowed:/
const SCRIPT_TAG = /<script([^>]*)>/g
const SETUP_ATTR = /\bsetup\b/
const TEMPLATE_BLOCK = /<template>([\s\S]*)<\/template>/
// A tag with its attributes. Quoted values are skipped whole, because a class can contain
// `>` (`has-[>svg]:grid-cols-2`), and ending the tag there would leave the rest of the class
// list behind as "visible text".
const TAG = /<(?:[^>"']|"[^"]*"|'[^']*')*>/g
// Measured on determination.ttf (2026-09-10): glyphs the font doesn't have. Written in
// source they fall back to whatever system font the machine has.
const MISSING_GLYPHS = /[→←↑↓⏎⌘✓]/

const hasScriptTagWithoutSetup = (body) =>
  [...body.matchAll(SCRIPT_TAG)].some(([, attrs]) => !SETUP_ATTR.test(attrs))

const walk = (dir) =>
  readdirSync(dir).flatMap((name) => {
    const full = join(dir, name)
    return statSync(full).isDirectory() ? walk(full) : [full]
  })

const hasUnexemptedStyleBlock = (body) =>
  [...body.matchAll(STYLE_BLOCK)].some(
    ([, content]) => !STYLE_EXEMPTION.test(content),
  )

// "Internationalization" section of the conventions: no visible string in the
// template, because the two languages start from day one of the frontend. The heuristic
// is stated rather than guessed: from the <template> block, comments, `{{ … }}`
// interpolations, and every tag with its attributes are stripped; what's left is text
// the user reads. Two letters in a row are enough to flag it — a `·` or a `[` aren't,
// because separators and punctuation aren't translated.
const visibleText = (body) => {
  const template = body.match(TEMPLATE_BLOCK)
  if (!template) return ''
  return template[1]
    .replace(/<!--[\s\S]*?-->/g, '')
    .replace(/\{\{[\s\S]*?\}\}/g, '')
    .replace(TAG, ' ')
}

const isUnder = (file, dir) => relative(ROOT, file).startsWith(dir)

// Exceptions are declared here, per file and per check, with a reason. An exception
// with no reason is an untracked violation; an empty list is the goal.
const EXEMPTIONS = [
  {
    file: 'src/App.vue',
    check: 'raw primitive <button>/<input>',
    reason:
      "declared verification page, to be replaced by the design system: the primitives don't exist yet",
  },
  {
    file: 'src/App.vue',
    check: 'visible string in the template',
    reason: 'same verification page: i18n arrives with the real frontend',
  },
  {
    file: 'src/components/WikiInline.vue',
    check: 'raw primitive <button>/<input>',
    reason:
      'verification render of the wiki dataset: the link to another target will become a primitive',
  },
]

const isExempt = (file, check) =>
  EXEMPTIONS.some(
    (e) =>
      e.check === check && relative(ROOT, file) === join(...e.file.split('/')),
  )

const checks = [
  {
    name: 'style block with no declared exemption',
    test: (file, body) =>
      file.endsWith('.vue') && hasUnexemptedStyleBlock(body),
  },
  {
    name: '<script> without setup',
    test: (file, body) =>
      file.endsWith('.vue') && hasScriptTagWithoutSetup(body),
  },
  {
    name: 'invoke() outside src/lib/ipc/',
    test: (file, body) => /\binvoke\s*\(/.test(body) && !isUnder(file, IPC_DIR),
  },
  {
    name: 'arbitrary pixel value in a class',
    test: (_f, body) => /\[\d+px\]/.test(body),
  },
  {
    name: 'hardcoded opacity',
    test: (_f, body) => /\bopacity-(?!0\b|100\b)\d+/.test(body),
  },
  {
    name: 'hardcoded duration',
    test: (_f, body) => /\bduration-\d+/.test(body),
  },
  {
    name: 'size prop on an icon: use size-*',
    test: (_f, body) => /:size="\d+"/.test(body),
  },
  {
    name: 'raw primitive <button>/<input>',
    test: (file, body) =>
      file.endsWith('.vue') &&
      !isUnder(file, UI_DIR) &&
      /<(button|input)[\s>/]/.test(body),
  },
  {
    // Rule 5: two string literals joined by `|` are a hand-written union, whether
    // in `type X = 'a' | 'b'` or inline on a prop or in a generic.
    name: "string literal union: use an 'as const' object",
    test: (_f, body) => /'[^'\n]*'\s*\|\s*'[^'\n]*'/.test(body),
  },
  {
    name: 'visible string in the template',
    test: (file, body) =>
      file.endsWith('.vue') &&
      !isUnder(file, DEV_ONLY_DIR) &&
      /\p{L}{2,}/u.test(visibleText(body)),
  },
  {
    // One theme. Without `@custom-variant dark`, Tailwind's built-in `dark:` compiles to
    // `prefers-color-scheme`, so a leftover class would switch on with the OS setting.
    name: 'dark: variant in a one-theme app',
    test: (_f, body) => /(^|[\s"'`])dark:[a-z[*]/m.test(body),
  },
  {
    name: 'literal colour in a class: colours are tokens',
    test: (_f, body) =>
      /\[(#[0-9a-fA-F]{3,8}|(rgb|rgba|hsl|hsla|oklch|oklab)\()/.test(body),
  },
  {
    name: 'colour alpha modifier: the alpha belongs in a token',
    test: (_f, body) =>
      /\b(bg|text|border|ring|outline|fill|stroke|divide|placeholder|decoration|caret|accent|shadow|from|via|to)-[a-z][a-z0-9-]*\/\d+/.test(
        body,
      ),
  },
  {
    // The package isn't installed: these classes would generate nothing, silently.
    name: 'tw-animate-css class',
    test: (_f, body) =>
      /\b(animate-(in|out)|(fade|zoom|spin)-(in|out)|slide-(in|out)-from)\b/.test(
        body,
      ),
  },
  {
    name: "literal variant on a primitive: use the component's constant",
    test: (file, body) =>
      file.endsWith('.vue') &&
      !isUnder(file, UI_DIR) &&
      /\s(variant|size|density|orientation)="[a-z]/.test(body),
  },
  {
    name: 'glyph missing from Determination: use an icon',
    test: (_f, body) => MISSING_GLYPHS.test(body),
  },
]

const violations = walk(SRC)
  .filter((f) => /\.(vue|ts)$/.test(f))
  .flatMap((file) => {
    const body = readFileSync(file, 'utf8')
    return checks
      .filter((c) => c.test(file, body) && !isExempt(file, c.name))
      .map((c) => `${relative(ROOT, file)}: ${c.name}`)
  })

violations.forEach((v) => console.error(v))
console.log(
  `${violations.length} violations, ${EXEMPTIONS.length} declared exemptions`,
)
process.exit(violations.length === 0 ? 0 : 1)
```

- [ ] **Step 4: Run the scanner on the probes**

Run: `pnpm scan`
Expected: exactly these six lines (order may differ), then `6 violations, 3 declared exemptions`:
```
src\ScanProbe.vue: dark: variant in a one-theme app
src\ScanProbe.vue: literal colour in a class: colours are tokens
src\ScanProbe.vue: colour alpha modifier: the alpha belongs in a token
src\ScanProbe.vue: tw-animate-css class
src\ScanProbe.vue: literal variant on a primitive: use the component's constant
src\ScanProbe.vue: glyph missing from Determination: use an icon
```
Neither `ScanProbeAttribute.vue` nor `kit\ScanProbeKit.vue` appears. If any other file of the repo appears, the pattern has a false positive: narrow it, don't exempt the file.

- [ ] **Step 5: Remove the probes**

Run: `rm ui/src/ScanProbe.vue ui/src/ScanProbeAttribute.vue && rm -r ui/src/kit && pnpm scan`
Expected: `0 violations, 3 declared exemptions`.

- [ ] **Step 6: The conventions table follows the script**

In `docs/frontend-conventions.md`, replace the row

```
| **Visible strings in the template** | `ui/scripts/scan-conventions.mjs` |
```

with

```
| **Visible strings in the template** (skipping `src/kit/`, development-only) | `ui/scripts/scan-conventions.mjs` |
| **`dark:` variant** (one theme) | `ui/scripts/scan-conventions.mjs` |
| **Literal colour in a class** | `ui/scripts/scan-conventions.mjs` |
| **Colour alpha modifier (`bg-x/50`)** | `ui/scripts/scan-conventions.mjs` |
| **`tw-animate-css` class** (not installed) | `ui/scripts/scan-conventions.mjs` |
| **Literal `variant`/`size`/`density`/`orientation` on a primitive** | `ui/scripts/scan-conventions.mjs` |
| **Glyph missing from Determination** | `ui/scripts/scan-conventions.mjs` |
```

and replace the sentence

```
The last three rows arrived on 2026-09-06: before that, the document declared five rules
and the script checked three.
```

with

```
Three rows arrived on 2026-09-06 — before that, the document declared five rules and the
script checked three — and six more on 2026-09-10, with the design system. On the same day
the visible-string heuristic learned to skip quoted attribute values: a class such as
`has-[>svg]:grid-cols-2` used to end the tag early and leave half a class list behind as
"visible text".
```

- [ ] **Step 7: Checks**

Run: `pnpm --filter ui format && pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test`

- [ ] **Step 8: Commit**

```bash
git add ui/scripts/scan-conventions.mjs docs/frontend-conventions.md
git commit -m "build(ui): the conventions scanner learns the design system's rules" -m "Six checks: dark: variants, literal colours, colour alpha modifiers, tw-animate-css classes, literal variants on primitives, glyphs Determination lacks. The visible-string heuristic skips quoted attribute values, which a class containing > used to cut short, and src/kit/ is declared development-only for that check alone."
```

---

### Task 7: Primitive groundwork — Button, Kbd, Separator, Skeleton, and the Kit page

Spec Decisions 7, 8, 11. From here on, every primitive is written in its dressed form. The "Visual check" step of each primitive task is done in a browser; if the executor has none, it says so in its report and the controller does it before accepting the task.

**Files:**
- Create: `ui/components.json`
- Modify: `ui/eslint.config.js`, `ui/src/main.ts`
- Create: `ui/src/lib/constants/placement.ts`, `ui/src/lib/constants/devRoutes.ts`, `ui/src/lib/constants/keyNames.ts`
- Create: `ui/src/components/ui/button/{variants.ts,Button.vue,index.ts}`, `ui/src/components/ui/kbd/{Kbd.vue,KbdGroup.vue,index.ts}`, `ui/src/components/ui/separator/{Separator.vue,index.ts}`, `ui/src/components/ui/skeleton/{Skeleton.vue,index.ts}`
- Create: `ui/src/kit/KitPage.vue`, `ui/src/kit/KitSection.vue`, `ui/src/kit/sections/{ButtonSection,KbdSection,SeparatorSection,SkeletonSection}.vue`

**Interfaces:**
- Consumes: `cn` (Task 4), `i18n` (Task 5).
- Produces:
  - `Orientation` (`Horizontal`, `Vertical`), `Align` (`Start`, `Center`, `End`), `Side` (`Top`, `Right`, `Bottom`, `Left`) from `@/lib/constants/placement`; `DevRoute.Kit = '#kit'` from `@/lib/constants/devRoutes`; `KeyName` (`Ctrl`, `K`, `Esc`) from `@/lib/constants/keyNames`.
  - `Button`, `ButtonVariant` (`Default`, `Secondary`, `Outline`, `Ghost`, `Link`), `ButtonSize` (`Default`, `Icon`), `buttonVariants` from `@/components/ui/button`.
  - `Kbd`, `KbdGroup`; `Separator` (prop `orientation?: Orientation`); `Skeleton`.
  - `KitSection` (prop `title: string`, default slot) and `KitPage`, which later tasks extend by adding one import and one `<XSection />` line.

- [ ] **Step 1: Install**

Run: `pnpm --filter ui add reka-ui@2.10.4 @vueuse/core@14.4.0 @lucide/vue@1.44.0 class-variance-authority@0.7.1`

- [ ] **Step 2: `ui/components.json`**

Written by hand: `shadcn-vue init` would rewrite `main.css` (spec Decision 7).

```json
{
  "$schema": "https://shadcn-vue.com/schema.json",
  "style": "reka-vega",
  "typescript": true,
  "tailwind": {
    "config": "",
    "css": "src/assets/main.css",
    "baseColor": "neutral",
    "cssVariables": true,
    "prefix": ""
  },
  "iconLibrary": "lucide",
  "rtl": false,
  "pointer": false,
  "aliases": {
    "components": "@/components",
    "utils": "@/lib/cn",
    "ui": "@/components/ui",
    "lib": "@/lib",
    "composables": "@/composables"
  },
  "registries": {}
}
```

- [ ] **Step 3: ESLint override**

Replace `ui/eslint.config.js` with:

```js
import pluginVue from 'eslint-plugin-vue'
import { withVueTs, vueTsConfigs } from '@vue/eslint-config-typescript'

export default withVueTs(
  { ignores: ['dist', 'node_modules'] },
  pluginVue.configs['flat/essential'],
  vueTsConfigs.recommended,
  {
    // shadcn-vue primitives are named for the element they stand for — Button.vue,
    // Badge.vue. The rule exists to keep app components from clashing with HTML elements;
    // a folder of primitives imported by name is exactly where single words belong.
    files: ['src/components/ui/**/*.vue'],
    rules: { 'vue/multi-word-component-names': 'off' },
  },
)
```

- [ ] **Step 4: Constants**

`ui/src/lib/constants/placement.ts`:
```ts
// Reka UI's orientation and floating placement, as values templates and defaults can name.
export const Orientation = { Horizontal: 'horizontal', Vertical: 'vertical' } as const
export type Orientation = (typeof Orientation)[keyof typeof Orientation]

export const Align = { Start: 'start', Center: 'center', End: 'end' } as const
export type Align = (typeof Align)[keyof typeof Align]

export const Side = { Top: 'top', Right: 'right', Bottom: 'bottom', Left: 'left' } as const
export type Side = (typeof Side)[keyof typeof Side]
```

`ui/src/lib/constants/devRoutes.ts`:
```ts
// Hash routes that only exist under `pnpm ui:dev`: main.ts checks import.meta.env.DEV
// before honouring them, and the production build drops what they import.
export const DevRoute = { Kit: '#kit' } as const
export type DevRoute = (typeof DevRoute)[keyof typeof DevRoute]
```

`ui/src/lib/constants/keyNames.ts`:
```ts
// Key caps as Windows prints them. They read the same in every language the app speaks,
// so they're data, not translations. Arrows and Enter are icons: Determination has no
// glyph for them.
export const KeyName = { Ctrl: 'Ctrl', K: 'K', Esc: 'Esc' } as const
export type KeyName = (typeof KeyName)[keyof typeof KeyName]
```

- [ ] **Step 5: Button**

`ui/src/components/ui/button/variants.ts`:
```ts
import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const ButtonVariant = {
  Default: 'default',
  Secondary: 'secondary',
  Outline: 'outline',
  Ghost: 'ghost',
  Link: 'link',
} as const
export type ButtonVariant = (typeof ButtonVariant)[keyof typeof ButtonVariant]

export const ButtonSize = { Default: 'default', Icon: 'icon' } as const
export type ButtonSize = (typeof ButtonSize)[keyof typeof ButtonSize]

// Shadcn Kit.dc.html, "Button": a flat edge, no radius, and no transition — hover and
// active change colour at once (Motion.dc.html, 0ms). Disabled is its own surface, not an
// opacity.
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
      },
      size: {
        [ButtonSize.Default]: 'h-control px-4',
        [ButtonSize.Icon]: 'size-control',
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

`ui/src/components/ui/button/Button.vue`:
```vue
<script setup lang="ts">
import type { PrimitiveProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { Primitive } from 'reka-ui'
import { cn } from '@/lib/cn'
import type { ButtonSize, ButtonVariant } from './variants'
import { buttonVariants } from './variants'

interface Props extends PrimitiveProps {
  variant?: ButtonVariant
  size?: ButtonSize
  class?: HTMLAttributes['class']
}

const props = withDefaults(defineProps<Props>(), {
  as: 'button',
})
</script>

<template>
  <Primitive
    data-slot="button"
    :data-variant="variant"
    :data-size="size"
    :as="as"
    :as-child="asChild"
    :class="cn(buttonVariants({ variant, size }), props.class)"
  >
    <slot />
  </Primitive>
</template>
```

`ui/src/components/ui/button/index.ts`:
```ts
export { default as Button } from './Button.vue'
export { ButtonSize, ButtonVariant, buttonVariants } from './variants'
export type { ButtonVariants } from './variants'
```

- [ ] **Step 6: Kbd**

`ui/src/components/ui/kbd/Kbd.vue`:
```vue
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <kbd
    data-slot="kbd"
    :class="
      cn(
        'pointer-events-none inline-flex h-5 min-w-5 items-center justify-center gap-1 border border-secondary-edge px-1.5 text-label text-foreground select-none [&_svg:not([class*=size-])]:size-3',
        props.class,
      )
    "
  >
    <slot />
  </kbd>
</template>
```

`ui/src/components/ui/kbd/KbdGroup.vue`:
```vue
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <kbd
    data-slot="kbd-group"
    :class="cn('inline-flex items-center gap-1', props.class)"
  >
    <slot />
  </kbd>
</template>
```

`ui/src/components/ui/kbd/index.ts`:
```ts
export { default as Kbd } from './Kbd.vue'
export { default as KbdGroup } from './KbdGroup.vue'
```

- [ ] **Step 7: Separator**

`ui/src/components/ui/separator/Separator.vue`:
```vue
<script setup lang="ts">
import type { SeparatorProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { Separator } from 'reka-ui'
import { cn } from '@/lib/cn'
import { Orientation } from '@/lib/constants/placement'

const props = withDefaults(
  defineProps<SeparatorProps & { class?: HTMLAttributes['class'] }>(),
  {
    orientation: Orientation.Horizontal,
    decorative: true,
  },
)

const delegatedProps = reactiveOmit(props, 'class')
</script>

<template>
  <Separator
    data-slot="separator"
    v-bind="delegatedProps"
    :class="
      cn(
        'shrink-0 bg-border data-[orientation=horizontal]:h-px data-[orientation=horizontal]:w-full data-[orientation=vertical]:w-px data-[orientation=vertical]:self-stretch',
        props.class,
      )
    "
  />
</template>
```

`ui/src/components/ui/separator/index.ts`:
```ts
export { default as Separator } from './Separator.vue'
```

- [ ] **Step 8: Skeleton**

`ui/src/components/ui/skeleton/Skeleton.vue`:
```vue
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <!-- A sweep between two flat surfaces on steps, not a pulse: the loading strip of
       Motion.dc.html. The consumer gives the size, known ahead of time (brief §10). -->
  <div
    data-slot="skeleton"
    :class="cn('animate-skeleton bg-secondary', props.class)"
  />
</template>
```

`ui/src/components/ui/skeleton/index.ts`:
```ts
export { default as Skeleton } from './Skeleton.vue'
```

- [ ] **Step 9: The Kit page frame**

`ui/src/kit/KitSection.vue`:
```vue
<script setup lang="ts">
defineProps<{ title: string }>()
</script>

<template>
  <section class="flex flex-col gap-3 border border-border bg-sheet p-4">
    <h2 class="text-control text-highlight">{{ title }}</h2>
    <slot />
  </section>
</template>
```

`ui/src/kit/KitPage.vue`:
```vue
<script setup lang="ts">
import ButtonSection from './sections/ButtonSection.vue'
import KbdSection from './sections/KbdSection.vue'
import SeparatorSection from './sections/SeparatorSection.vue'
import SkeletonSection from './sections/SkeletonSection.vue'
</script>

<template>
  <main
    class="grid min-h-screen grid-cols-3 items-start gap-4 bg-background p-8 text-foreground"
  >
    <ButtonSection />
    <KbdSection />
    <SeparatorSection />
    <SkeletonSection />
  </main>
</template>
```

- [ ] **Step 10: The first sections**

`ui/src/kit/sections/ButtonSection.vue`:
```vue
<script setup lang="ts">
import { PlusIcon } from '@lucide/vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Button">
    <div class="flex flex-wrap items-center gap-2">
      <Button>Default</Button>
      <Button :variant="ButtonVariant.Secondary">Secondary</Button>
      <Button :variant="ButtonVariant.Outline">Outline</Button>
      <Button :variant="ButtonVariant.Ghost">Ghost</Button>
      <Button :variant="ButtonVariant.Link">Link</Button>
      <Button disabled>Disabled</Button>
      <Button
        :variant="ButtonVariant.Secondary"
        :size="ButtonSize.Icon"
        aria-label="Aggiungi"
      >
        <PlusIcon />
      </Button>
    </div>
  </KitSection>
</template>
```

`ui/src/kit/sections/KbdSection.vue`:
```vue
<script setup lang="ts">
import { ArrowDownIcon, ArrowUpIcon, CornerDownLeftIcon } from '@lucide/vue'
import { Kbd, KbdGroup } from '@/components/ui/kbd'
import { KeyName } from '@/lib/constants/keyNames'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Kbd">
    <div
      class="flex flex-wrap items-center gap-3 text-caption text-muted-foreground"
    >
      <KbdGroup>
        <Kbd>{{ KeyName.Ctrl }}</Kbd>
        <Kbd>{{ KeyName.K }}</Kbd>
      </KbdGroup>
      <span class="flex items-center gap-1">
        <Kbd><ArrowUpIcon /></Kbd>
        <Kbd><ArrowDownIcon /></Kbd>
        naviga
      </span>
      <span class="flex items-center gap-1">
        <Kbd><CornerDownLeftIcon /></Kbd>
        apri
      </span>
      <Kbd>{{ KeyName.Esc }}</Kbd>
    </div>
  </KitSection>
</template>
```

`ui/src/kit/sections/SeparatorSection.vue`:
```vue
<script setup lang="ts">
import { Separator } from '@/components/ui/separator'
import { Orientation } from '@/lib/constants/placement'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Separator">
    <p class="text-body text-foreground-soft">Sopra il separatore</p>
    <Separator />
    <div class="flex h-6 items-center gap-3 text-body text-foreground-soft">
      <span>Sinistra</span>
      <Separator :orientation="Orientation.Vertical" />
      <span>Destra</span>
    </div>
  </KitSection>
</template>
```

`ui/src/kit/sections/SkeletonSection.vue`:
```vue
<script setup lang="ts">
import { Skeleton } from '@/components/ui/skeleton'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Skeleton">
    <div class="flex items-center gap-3">
      <Skeleton class="size-10" />
      <div class="flex flex-1 flex-col gap-2">
        <Skeleton class="h-3 w-3/4" />
        <Skeleton class="h-3 w-1/2" />
      </div>
    </div>
  </KitSection>
</template>
```

- [ ] **Step 11: Mount the Kit page in development**

Replace `ui/src/main.ts` with:

```ts
import { createApp } from 'vue'
import './assets/main.css'
import App from './App.vue'
import { i18n } from './i18n'
import { DevRoute } from './lib/constants/devRoutes'

// The Kit page shows every primitive in every state, to compare with the design export.
// Behind import.meta.env.DEV the production build drops the import entirely.
const mountKit = async () => {
  const { default: KitPage } = await import('./kit/KitPage.vue')
  createApp(KitPage).use(i18n).mount('#app')
}

if (import.meta.env.DEV && window.location.hash === DevRoute.Kit) {
  void mountKit()
} else {
  createApp(App).use(i18n).mount('#app')
}
```

- [ ] **Step 12: Checks and build**

Run:
```bash
pnpm --filter ui format && pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test
pnpm --filter ui build
grep -l "KitSection\|Sopra il separatore" ui/dist/assets/*.js; echo "kit in dist: $?"
grep -o "\.animate-skeleton" ui/dist/assets/*.css | head -1
grep -o "@keyframes skeleton-sweep" ui/dist/assets/*.css | head -1
```
Expected: all green; `kit in dist: 1` (grep found nothing); `.animate-skeleton`; `@keyframes skeleton-sweep` (Tailwind scans the Kit files even though the bundle drops them).

- [ ] **Step 13: Visual check**

Run `pnpm ui:dev`, open `http://localhost:1420/#kit`. Check against the spec catalogue: square buttons 34px high, red Default, leather Secondary edge, cream underlined Link, disabled on a dark flat surface; hovering changes colour instantly; keyboard Tab shows a 2px cyan ring with a gap; kbd caps with icon arrows; skeleton bars stepping between two browns. Stop the server.

- [ ] **Step 14: Commit**

```bash
git add ui/package.json pnpm-lock.yaml ui/components.json ui/eslint.config.js ui/src/main.ts ui/src/lib/constants ui/src/components/ui/button ui/src/components/ui/kbd ui/src/components/ui/separator ui/src/components/ui/skeleton ui/src/kit
git commit -m "feat(ui): Button, Kbd, Separator and Skeleton, and a development-only Kit page" -m "The first shadcn-vue primitives (reka-vega), written already dressed: tokens instead of shadcn's classes, variants as constants in variants.ts. components.json is written by hand, since init would rewrite main.css. The Kit page mounts only under pnpm ui:dev at #kit and is absent from the production build."
```

---

### Task 8: Badge — states that carry their shape

**Files:**
- Create: `ui/src/components/ui/badge/{variants.ts,icons.ts,Badge.vue,index.ts}`, `ui/src/kit/sections/BadgeSection.vue`
- Modify: `ui/src/kit/KitPage.vue`

**Interfaces:**
- Consumes: `cn`; `KitSection`.
- Produces: `Badge` (props `variant?: BadgeVariant` default `Tag`, `as`, `asChild`, `class`), `BadgeVariant` (`Done`, `Now`, `Blocked`, `Unknown`, `Unexpected`, `Tag`, `Challenge`), `badgeVariants`, `badgeIcons: Record<BadgeVariant, Component | null>`.

- [ ] **Step 1: Variants**

`ui/src/components/ui/badge/variants.ts`:
```ts
import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const BadgeVariant = {
  Done: 'done',
  Now: 'now',
  Blocked: 'blocked',
  Unknown: 'unknown',
  Unexpected: 'unexpected',
  Tag: 'tag',
  Challenge: 'challenge',
} as const
export type BadgeVariant = (typeof BadgeVariant)[keyof typeof BadgeVariant]

// State variants are pills with a mark (icons.ts); tag variants are square and bare
// (Shadcn Kit.dc.html, "Badge").
const statePill = 'rounded-full px-2.5 py-0.5 text-caption'
const squareTag = 'px-2 py-1 text-control'

export const badgeVariants = cva(
  'inline-flex w-fit shrink-0 items-center gap-1 border whitespace-nowrap [&_svg]:pointer-events-none [&_svg]:size-3',
  {
    variants: {
      variant: {
        [BadgeVariant.Done]: `${statePill} border-state-done bg-state-done-surface text-state-done-foreground`,
        [BadgeVariant.Now]: `${statePill} border-state-now bg-state-now-surface text-state-now-foreground`,
        [BadgeVariant.Blocked]: `${statePill} border-state-blocked bg-state-blocked-surface text-state-blocked-foreground`,
        [BadgeVariant.Unknown]: `${statePill} hatch-unknown border-dashed border-state-unknown text-state-unknown-foreground`,
        [BadgeVariant.Unexpected]: `${statePill} border-state-unexpected bg-state-unexpected-surface text-state-unexpected-foreground`,
        [BadgeVariant.Tag]: `${squareTag} border-input bg-data text-foreground`,
        [BadgeVariant.Challenge]: `${squareTag} border-challenge bg-challenge-surface text-challenge-foreground`,
      },
    },
    defaultVariants: {
      variant: BadgeVariant.Tag,
    },
  },
)
export type BadgeVariants = VariantProps<typeof badgeVariants>
```

(The Tailwind scanner sees class names inside template literals: the two constants are whole classes, not fragments.)

- [ ] **Step 2: Marks**

`ui/src/components/ui/badge/icons.ts`:
```ts
import type { Component, FunctionalComponent } from 'vue'
import { CheckIcon, LockIcon, StarIcon, TriangleAlertIcon } from '@lucide/vue'
import { h } from 'vue'
import { BadgeVariant } from './variants'

// Unknown is drawn with a "?", as in the kit: a question mark reads as "we can't tell",
// where one more icon would read as one more state.
const QuestionMark: FunctionalComponent = () =>
  h('span', { 'aria-hidden': 'true' }, '?')

// A state is never colour alone (Tokens.dc.html: every state is told apart by shape), so
// each state variant carries its mark; tags have none. A record rather than a switch: a
// variant added without a mark doesn't compile.
export const badgeIcons: Record<BadgeVariant, Component | null> = {
  [BadgeVariant.Done]: CheckIcon,
  [BadgeVariant.Now]: StarIcon,
  [BadgeVariant.Blocked]: LockIcon,
  [BadgeVariant.Unknown]: QuestionMark,
  [BadgeVariant.Unexpected]: TriangleAlertIcon,
  [BadgeVariant.Tag]: null,
  [BadgeVariant.Challenge]: null,
}
```

- [ ] **Step 3: Component and index**

`ui/src/components/ui/badge/Badge.vue`:
```vue
<script setup lang="ts">
import type { PrimitiveProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { Primitive } from 'reka-ui'
import { computed } from 'vue'
import { cn } from '@/lib/cn'
import { badgeIcons } from './icons'
import { BadgeVariant, badgeVariants } from './variants'

const props = withDefaults(
  defineProps<
    PrimitiveProps & {
      variant?: BadgeVariant
      class?: HTMLAttributes['class']
    }
  >(),
  {
    as: 'span',
    variant: BadgeVariant.Tag,
  },
)

const delegatedProps = reactiveOmit(props, 'class', 'variant')
const icon = computed(() => badgeIcons[props.variant])
</script>

<template>
  <Primitive
    data-slot="badge"
    :data-variant="variant"
    v-bind="delegatedProps"
    :class="cn(badgeVariants({ variant }), props.class)"
  >
    <component :is="icon" v-if="icon" />
    <slot />
  </Primitive>
</template>
```

`ui/src/components/ui/badge/index.ts`:
```ts
export { default as Badge } from './Badge.vue'
export { badgeIcons } from './icons'
export { BadgeVariant, badgeVariants } from './variants'
export type { BadgeVariants } from './variants'
```

- [ ] **Step 4: Kit section**

`ui/src/kit/sections/BadgeSection.vue`:
```vue
<script setup lang="ts">
import { Badge, BadgeVariant } from '@/components/ui/badge'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Badge">
    <div class="flex flex-wrap items-center gap-2">
      <Badge :variant="BadgeVariant.Done">Fatto</Badge>
      <Badge :variant="BadgeVariant.Now">Ora</Badge>
      <Badge :variant="BadgeVariant.Blocked">Da 2</Badge>
      <Badge :variant="BadgeVariant.Unknown">Sconosciuto</Badge>
      <Badge :variant="BadgeVariant.Unexpected">Anomalo</Badge>
      <Badge :variant="BadgeVariant.Tag">Oggetto</Badge>
      <Badge :variant="BadgeVariant.Challenge">Sfida</Badge>
    </div>
  </KitSection>
</template>
```

In `ui/src/kit/KitPage.vue` add `import BadgeSection from './sections/BadgeSection.vue'` after the `ButtonSection` import, and `<BadgeSection />` after `<ButtonSection />`.

- [ ] **Step 5: Checks, build, visual check**

Run: `pnpm --filter ui format && pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test && pnpm --filter ui build`
Then `pnpm ui:dev`, `#kit`: five pills each with its mark (green check, gold star, lock, "?" on dashed stripes, red triangle), two square tags.

- [ ] **Step 6: Commit**

```bash
git add ui/src/components/ui/badge ui/src/kit
git commit -m "feat(ui): Badge, whose states carry their shape" -m "Done, unlockable now, blocked, unknown and unexpected are pills with a mark each, from an exhaustive record, so a state is never told by colour alone. Gold is the unlockable-now state only. Unknown is a question mark on the unknown hatch; unexpected, which the export didn't draw, takes the error tone with a warning triangle."
```

---

### Task 9: Card, Alert, Empty

**Files:**
- Create: `ui/src/components/ui/card/{Card,CardHeader,CardTitle,CardAction,CardContent,CardFooter}.vue` + `index.ts`
- Create: `ui/src/components/ui/alert/{variants.ts,Alert.vue,AlertTitle.vue,AlertDescription.vue,index.ts}`
- Create: `ui/src/components/ui/empty/{Empty,EmptyMedia,EmptyTitle,EmptyDescription,EmptyContent}.vue` + `index.ts`
- Create: `ui/src/kit/sections/{CardSection,AlertSection,EmptySection}.vue`
- Modify: `ui/src/kit/KitPage.vue`

**Interfaces:**
- Consumes: `cn`, `Button`, `ButtonVariant`, `KitSection`.
- Produces: `Card`, `CardHeader`, `CardTitle`, `CardAction`, `CardContent`, `CardFooter`; `Alert` (prop `variant?: AlertVariant`), `AlertTitle`, `AlertDescription`, `AlertVariant` (`Default`, `Destructive`), `alertVariants`; `Empty`, `EmptyMedia`, `EmptyTitle`, `EmptyDescription`, `EmptyContent`. Every part takes `class?`.

- [ ] **Step 1: Card parts**

Each file below has the same script block; only `data-slot` and the base classes differ.

```vue
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>
```

`ui/src/components/ui/card/Card.vue` template:
```vue
<template>
  <div
    data-slot="card"
    :class="
      cn(
        'flex flex-col border border-border bg-card text-card-foreground',
        props.class,
      )
    "
  >
    <slot />
  </div>
</template>
```

`CardHeader.vue` template — the leather band:
```vue
<template>
  <div
    data-slot="card-header"
    :class="
      cn(
        'flex items-center justify-between gap-2 bg-band px-3 py-2 text-band-foreground',
        props.class,
      )
    "
  >
    <slot />
  </div>
</template>
```

`CardTitle.vue` template:
```vue
<template>
  <div data-slot="card-title" :class="cn('text-control', props.class)">
    <slot />
  </div>
</template>
```

`CardAction.vue` template:
```vue
<template>
  <div data-slot="card-action" :class="cn('text-label', props.class)">
    <slot />
  </div>
</template>
```

`CardContent.vue` template:
```vue
<template>
  <div data-slot="card-content" :class="cn('p-3 text-body', props.class)">
    <slot />
  </div>
</template>
```

`CardFooter.vue` template:
```vue
<template>
  <div
    data-slot="card-footer"
    :class="
      cn(
        'flex items-center gap-2 border-t border-hairline px-3 py-2.5',
        props.class,
      )
    "
  >
    <slot />
  </div>
</template>
```

`ui/src/components/ui/card/index.ts`:
```ts
export { default as Card } from './Card.vue'
export { default as CardAction } from './CardAction.vue'
export { default as CardContent } from './CardContent.vue'
export { default as CardFooter } from './CardFooter.vue'
export { default as CardHeader } from './CardHeader.vue'
export { default as CardTitle } from './CardTitle.vue'
```

- [ ] **Step 2: Alert**

`ui/src/components/ui/alert/variants.ts`:
```ts
import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const AlertVariant = { Default: 'default', Destructive: 'destructive' } as const
export type AlertVariant = (typeof AlertVariant)[keyof typeof AlertVariant]

// A diagnostic, not a notification: no dismiss control, it goes when the cause does. The
// default icon is cream, not cyan: cyan is reserved to focus.
export const alertVariants = cva(
  'relative grid w-full gap-0.5 border p-3 text-left text-body has-[>svg]:grid-cols-[auto_1fr] has-[>svg]:gap-x-2.5 *:[svg]:row-span-2 *:[svg]:mt-0.5 *:[svg:not([class*=size-])]:size-4',
  {
    variants: {
      variant: {
        [AlertVariant.Default]:
          'border-border bg-data text-foreground *:[svg]:text-highlight',
        [AlertVariant.Destructive]:
          'border-destructive bg-destructive-surface text-foreground *:[svg]:text-destructive *:data-[slot=alert-description]:text-destructive-foreground',
      },
    },
    defaultVariants: {
      variant: AlertVariant.Default,
    },
  },
)
export type AlertVariants = VariantProps<typeof alertVariants>
```

`ui/src/components/ui/alert/Alert.vue`:
```vue
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'
import type { AlertVariant } from './variants'
import { alertVariants } from './variants'

const props = defineProps<{
  class?: HTMLAttributes['class']
  variant?: AlertVariant
}>()
</script>

<template>
  <div
    data-slot="alert"
    role="alert"
    :class="cn(alertVariants({ variant }), props.class)"
  >
    <slot />
  </div>
</template>
```

`AlertTitle.vue` (script as in Step 1):
```vue
<template>
  <div
    data-slot="alert-title"
    :class="cn('col-start-2 text-body text-foreground', props.class)"
  >
    <slot />
  </div>
</template>
```

`AlertDescription.vue` (script as in Step 1):
```vue
<template>
  <div
    data-slot="alert-description"
    :class="cn('col-start-2 text-body text-foreground-soft', props.class)"
  >
    <slot />
  </div>
</template>
```

`ui/src/components/ui/alert/index.ts`:
```ts
export { default as Alert } from './Alert.vue'
export { default as AlertDescription } from './AlertDescription.vue'
export { default as AlertTitle } from './AlertTitle.vue'
export { AlertVariant, alertVariants } from './variants'
export type { AlertVariants } from './variants'
```

- [ ] **Step 3: Empty**

All five files use the script block of Step 1.

`Empty.vue`:
```vue
<template>
  <div
    data-slot="empty"
    :class="
      cn(
        'flex w-full min-w-0 flex-col items-center justify-center gap-3 border border-dashed border-secondary-edge bg-data px-4.5 py-6.5 text-center',
        props.class,
      )
    "
  >
    <slot />
  </div>
</template>
```

`EmptyMedia.vue`:
```vue
<template>
  <div
    data-slot="empty-media"
    :class="
      cn(
        'flex shrink-0 items-center justify-center text-faint-foreground [&_svg:not([class*=size-])]:size-8.5',
        props.class,
      )
    "
  >
    <slot />
  </div>
</template>
```

`EmptyTitle.vue`:
```vue
<template>
  <div
    data-slot="empty-title"
    :class="cn('text-heading text-foreground', props.class)"
  >
    <slot />
  </div>
</template>
```

`EmptyDescription.vue`:
```vue
<template>
  <p
    data-slot="empty-description"
    :class="cn('text-row text-muted-foreground', props.class)"
  >
    <slot />
  </p>
</template>
```

`EmptyContent.vue`:
```vue
<template>
  <div
    data-slot="empty-content"
    :class="cn('mt-1 flex flex-col items-center gap-2', props.class)"
  >
    <slot />
  </div>
</template>
```

`ui/src/components/ui/empty/index.ts`:
```ts
export { default as Empty } from './Empty.vue'
export { default as EmptyContent } from './EmptyContent.vue'
export { default as EmptyDescription } from './EmptyDescription.vue'
export { default as EmptyMedia } from './EmptyMedia.vue'
export { default as EmptyTitle } from './EmptyTitle.vue'
```

- [ ] **Step 4: Kit sections**

`ui/src/kit/sections/CardSection.vue`:
```vue
<script setup lang="ts">
import { Button, ButtonVariant } from '@/components/ui/button'
import {
  Card,
  CardAction,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/components/ui/card'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Card">
    <Card>
      <CardHeader>
        <CardTitle>01 · Priorità</CardTitle>
        <CardAction>sblocca 14</CardAction>
      </CardHeader>
      <CardContent class="flex gap-3">
        <div class="size-10 shrink-0 border border-input hatch-placeholder" />
        <div class="flex flex-col gap-1">
          <span class="text-heading">The Lost</span>
          <span class="text-caption text-muted-foreground">Mother · Hard</span>
        </div>
      </CardContent>
      <CardFooter>
        <Button class="flex-1">Segna come obiettivo</Button>
        <Button :variant="ButtonVariant.Outline">Dettagli</Button>
      </CardFooter>
    </Card>
  </KitSection>
</template>
```

`ui/src/kit/sections/AlertSection.vue`:
```vue
<script setup lang="ts">
import { InfoIcon, TriangleAlertIcon } from '@lucide/vue'
import {
  Alert,
  AlertDescription,
  AlertTitle,
  AlertVariant,
} from '@/components/ui/alert'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Alert">
    <Alert>
      <InfoIcon />
      <AlertTitle>Salvataggio riletto</AlertTitle>
      <AlertDescription>Rilette le voci dello slot Rep+ 1.</AlertDescription>
    </Alert>
    <Alert :variant="AlertVariant.Destructive">
      <TriangleAlertIcon />
      <AlertTitle>Catalogo assente</AlertTitle>
      <AlertDescription>
        Senza il gioco installato non si sa cosa sblocca cosa.
      </AlertDescription>
    </Alert>
  </KitSection>
</template>
```

`ui/src/kit/sections/EmptySection.vue`:
```vue
<script setup lang="ts">
import { SkullIcon } from '@lucide/vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyMedia,
  EmptyTitle,
} from '@/components/ui/empty'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Empty">
    <Empty>
      <EmptyMedia><SkullIcon /></EmptyMedia>
      <EmptyTitle>Niente qui</EmptyTitle>
      <EmptyDescription>Nessuna voce corrisponde ai filtri attivi.</EmptyDescription>
      <EmptyContent>
        <Button :variant="ButtonVariant.Outline">Azzera filtri</Button>
      </EmptyContent>
    </Empty>
  </KitSection>
</template>
```

In `ui/src/kit/KitPage.vue` add the three imports (`CardSection`, `AlertSection`, `EmptySection`) and, after `<BadgeSection />`, the lines `<CardSection />`, `<AlertSection />`, `<EmptySection />`.

- [ ] **Step 5: Checks, build, visual check**

Run: `pnpm --filter ui format && pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test && pnpm --filter ui build`
Visual (`#kit`): card with a leather header strip, placeholder square with warm stripes, hairline above the footer; default alert with a cream icon, destructive alert with red edge and dark red ground; empty state with a dashed leather edge.

- [ ] **Step 6: Commit**

```bash
git add ui/src/components/ui/card ui/src/components/ui/alert ui/src/components/ui/empty ui/src/kit
git commit -m "feat(ui): Card, Alert and Empty" -m "Card has the leather header band and a hairline footer. Alert is a diagnostic without a dismiss control, its default icon cream because cyan is reserved to focus. Empty has the dashed leather edge the kit draws."
```

---

### Task 10: Input, Label, Field

**Files:**
- Create: `ui/src/components/ui/input/{Input.vue,index.ts}`, `ui/src/components/ui/label/{Label.vue,index.ts}`, `ui/src/components/ui/field/{variants.ts,Field.vue,FieldLabel.vue,FieldDescription.vue,FieldError.vue,index.ts}`, `ui/src/kit/sections/FieldSection.vue`
- Modify: `ui/src/kit/KitPage.vue`

**Interfaces:**
- Consumes: `cn`, `KitSection`.
- Produces: `Input` (`v-model: string | number`, `defaultValue?`, attributes such as `placeholder`, `aria-invalid`, `disabled` fall through to the `<input>`); `Label` (Reka `LabelProps`); `Field` (prop `orientation?: FieldOrientation`), `FieldLabel`, `FieldDescription`, `FieldError` (prop `errors?: Array<string | { message: string | undefined } | undefined>` or a default slot), `FieldOrientation` (`Vertical`, `Horizontal`).

- [ ] **Step 1: Input**

`ui/src/components/ui/input/Input.vue`:
```vue
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { useVModel } from '@vueuse/core'
import { cn } from '@/lib/cn'

const props = defineProps<{
  defaultValue?: string | number
  modelValue?: string | number
  class?: HTMLAttributes['class']
}>()

const emits = defineEmits<{
  (e: 'update:modelValue', payload: string | number): void
}>()

const modelValue = useVModel(props, 'modelValue', emits, {
  passive: true,
  defaultValue: props.defaultValue,
})
</script>

<template>
  <input
    v-model="modelValue"
    data-slot="input"
    :class="
      cn(
        'h-control w-full min-w-0 rounded-input border border-input bg-data px-3 text-body text-foreground placeholder:text-faint-foreground aria-invalid:border-destructive disabled:pointer-events-none disabled:border-secondary disabled:bg-muted disabled:text-faint-foreground',
        props.class,
      )
    "
  />
</template>
```

`ui/src/components/ui/input/index.ts`:
```ts
export { default as Input } from './Input.vue'
```

- [ ] **Step 2: Label**

`ui/src/components/ui/label/Label.vue`:
```vue
<script setup lang="ts">
import type { LabelProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { Label } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<LabelProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = reactiveOmit(props, 'class')
</script>

<template>
  <Label
    data-slot="label"
    v-bind="delegatedProps"
    :class="
      cn(
        'flex items-center gap-2 text-caption text-foreground-soft select-none peer-disabled:cursor-not-allowed peer-disabled:text-faint-foreground',
        props.class,
      )
    "
  >
    <slot />
  </Label>
</template>
```

`ui/src/components/ui/label/index.ts`:
```ts
export { default as Label } from './Label.vue'
```

- [ ] **Step 3: Field**

`ui/src/components/ui/field/variants.ts`:
```ts
import type { VariantProps } from 'class-variance-authority'
import { cva } from 'class-variance-authority'

export const FieldOrientation = { Vertical: 'vertical', Horizontal: 'horizontal' } as const
export type FieldOrientation = (typeof FieldOrientation)[keyof typeof FieldOrientation]

export const fieldVariants = cva('flex w-full', {
  variants: {
    orientation: {
      [FieldOrientation.Vertical]: 'flex-col gap-1.5',
      [FieldOrientation.Horizontal]: 'flex-row items-center gap-2',
    },
  },
  defaultVariants: {
    orientation: FieldOrientation.Vertical,
  },
})
export type FieldVariants = VariantProps<typeof fieldVariants>
```

`ui/src/components/ui/field/Field.vue`:
```vue
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'
import { FieldOrientation, fieldVariants } from './variants'

const props = withDefaults(
  defineProps<{
    class?: HTMLAttributes['class']
    orientation?: FieldOrientation
  }>(),
  {
    orientation: FieldOrientation.Vertical,
  },
)
</script>

<template>
  <div
    role="group"
    data-slot="field"
    :data-orientation="orientation"
    :class="cn(fieldVariants({ orientation }), props.class)"
  >
    <slot />
  </div>
</template>
```

`ui/src/components/ui/field/FieldLabel.vue`:
```vue
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { Label } from '@/components/ui/label'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <Label data-slot="field-label" :class="cn('w-fit', props.class)">
    <slot />
  </Label>
</template>
```

`ui/src/components/ui/field/FieldDescription.vue`:
```vue
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <p
    data-slot="field-description"
    :class="cn('text-caption text-muted-foreground', props.class)"
  >
    <slot />
  </p>
</template>
```

`ui/src/components/ui/field/FieldError.vue`:
```vue
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { uniq } from 'lodash-es'
import { computed } from 'vue'
import { cn } from '@/lib/cn'

type FieldErrorEntry = string | { message: string | undefined } | undefined

const props = defineProps<{
  class?: HTMLAttributes['class']
  errors?: FieldErrorEntry[]
}>()

const messageOf = (entry: FieldErrorEntry): string | undefined =>
  typeof entry === 'string' ? entry : entry?.message

// The distinct messages, in order; the field's red edge says where, this says what.
const messages = computed(() =>
  uniq((props.errors ?? []).map(messageOf).filter((m): m is string => !!m)),
)
</script>

<template>
  <div
    v-if="$slots.default || messages.length"
    role="alert"
    data-slot="field-error"
    :class="cn('text-caption text-foreground', props.class)"
  >
    <slot v-if="$slots.default" />
    <template v-else-if="messages.length === 1">{{ messages[0] }}</template>
    <ul v-else class="ml-4 flex list-disc flex-col gap-1">
      <li v-for="message in messages" :key="message">{{ message }}</li>
    </ul>
  </div>
</template>
```

`ui/src/components/ui/field/index.ts`:
```ts
export { default as Field } from './Field.vue'
export { default as FieldDescription } from './FieldDescription.vue'
export { default as FieldError } from './FieldError.vue'
export { default as FieldLabel } from './FieldLabel.vue'
export { FieldOrientation, fieldVariants } from './variants'
export type { FieldVariants } from './variants'
```

- [ ] **Step 4: Kit section**

`ui/src/kit/sections/FieldSection.vue`:
```vue
<script setup lang="ts">
import {
  Field,
  FieldDescription,
  FieldError,
  FieldLabel,
} from '@/components/ui/field'
import { Input } from '@/components/ui/input'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Input · Label · Field">
    <Field>
      <FieldLabel for="kit-game-folder">Cartella del gioco</FieldLabel>
      <Input id="kit-game-folder" placeholder="Scegli una cartella…" />
      <FieldDescription>Trovata automaticamente, se Steam c'è.</FieldDescription>
    </Field>
    <Field>
      <FieldLabel for="kit-save-folder">Cartella dei salvataggi</FieldLabel>
      <Input id="kit-save-folder" aria-invalid="true" default-value="D:\Giochi" />
      <FieldError :errors="['Nessun salvataggio in questa cartella.']" />
    </Field>
    <Field>
      <FieldLabel for="kit-disabled">Disabilitato</FieldLabel>
      <Input id="kit-disabled" disabled default-value="—" />
    </Field>
  </KitSection>
</template>
```

In `ui/src/kit/KitPage.vue` add `import FieldSection from './sections/FieldSection.vue'` and `<FieldSection />` after `<EmptySection />`.

- [ ] **Step 5: Checks, build, visual check**

Run: `pnpm --filter ui format && pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test && pnpm --filter ui build`
Visual (`#kit`): inputs 34px high with a 4px radius on the data surface; the invalid one with a red edge and a message under it; the disabled one flat and faint; focus ring on Tab.

- [ ] **Step 6: Commit**

```bash
git add ui/src/components/ui/input ui/src/components/ui/label ui/src/components/ui/field ui/src/kit
git commit -m "feat(ui): Input, Label and Field" -m "Fields sit on the data surface with the 4px input radius; aria-invalid turns the edge red and FieldError says what is wrong. Only the field parts a screen uses are written."
```

---

### Task 11: Checkbox, Switch, Toggle group, Tabs

**Files:**
- Create: `ui/src/components/ui/checkbox/{state.ts,Checkbox.vue,index.ts}`, `ui/src/components/ui/switch/{Switch.vue,index.ts}`, `ui/src/components/ui/toggle-group/{variants.ts,ToggleGroup.vue,ToggleGroupItem.vue,index.ts}`, `ui/src/components/ui/tabs/{Tabs.vue,TabsList.vue,TabsTrigger.vue,TabsContent.vue,index.ts}`
- Create: `ui/src/kit/sections/{CheckboxSection,SwitchSection,ToggleGroupSection,TabsSection}.vue`
- Modify: `ui/src/kit/KitPage.vue`

**Interfaces:**
- Consumes: `cn`, `Label`, `KitSection`.
- Produces: `Checkbox` (Reka `CheckboxRootProps` + `class`), `CheckboxState` (`Checked: true`, `Unchecked: false`, `Indeterminate: 'indeterminate'`); `Switch` (Reka `SwitchRootProps` + `class`); `ToggleGroup` (Reka `ToggleGroupRootProps` + `size?: ToggleSize` + `class`), `ToggleGroupItem` (Reka `ToggleGroupItemProps` + `size?` + `class`), `ToggleSize` (`Default`, `Icon`), `ToggleGroupType` (`Single`, `Multiple`), `toggleGroupItemVariants`, `toggleGroupSizeKey`; `Tabs`, `TabsList`, `TabsTrigger`, `TabsContent`.

- [ ] **Step 1: Checkbox**

`ui/src/components/ui/checkbox/state.ts`:
```ts
import type { CheckboxCheckedState } from 'reka-ui'

// Reka's checked state is a boolean or 'indeterminate': named here so no template writes
// the string.
export const CheckboxState = {
  Checked: true,
  Unchecked: false,
  Indeterminate: 'indeterminate',
} as const satisfies Record<string, CheckboxCheckedState>
export type CheckboxState = (typeof CheckboxState)[keyof typeof CheckboxState]
```

`ui/src/components/ui/checkbox/Checkbox.vue`:
```vue
<script setup lang="ts">
import type { CheckboxRootEmits, CheckboxRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { CheckIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import { CheckboxIndicator, CheckboxRoot, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  CheckboxRootProps & { class?: HTMLAttributes['class'] }
>()
const emits = defineEmits<CheckboxRootEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <!-- Checked and indeterminate share the red fill and the lit edge; the indicator's own
       data-state picks the tick or the bar, in CSS, with no string compared in script. -->
  <CheckboxRoot
    v-slot="slotProps"
    data-slot="checkbox"
    v-bind="forwarded"
    :class="
      cn(
        'peer relative flex size-4 shrink-0 cursor-pointer items-center justify-center border border-input bg-data text-foreground disabled:cursor-not-allowed disabled:border-secondary disabled:bg-muted data-[state=checked]:border-selection-edge data-[state=checked]:bg-primary data-[state=indeterminate]:border-selection-edge data-[state=indeterminate]:bg-primary',
        props.class,
      )
    "
  >
    <CheckboxIndicator
      data-slot="checkbox-indicator"
      class="group grid place-content-center"
    >
      <slot v-bind="slotProps">
        <CheckIcon
          class="size-3 animate-tap-in group-data-[state=indeterminate]:hidden"
        />
        <span
          class="hidden h-0.5 w-2.5 bg-foreground group-data-[state=indeterminate]:block"
        />
      </slot>
    </CheckboxIndicator>
  </CheckboxRoot>
</template>
```

`ui/src/components/ui/checkbox/index.ts`:
```ts
export { default as Checkbox } from './Checkbox.vue'
export { CheckboxState } from './state'
```

- [ ] **Step 2: Switch**

`ui/src/components/ui/switch/Switch.vue`:
```vue
<script setup lang="ts">
import type { SwitchRootEmits, SwitchRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { SwitchRoot, SwitchThumb, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  SwitchRootProps & { class?: HTMLAttributes['class'] }
>()
const emits = defineEmits<SwitchRootEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <!-- Square, 34x18 with a 12px thumb (Shadcn Kit.dc.html). Inner width 28px, so the thumb
       travels 16px: translate-x-4, in two steps. -->
  <SwitchRoot
    v-slot="slotProps"
    data-slot="switch"
    v-bind="forwarded"
    :class="
      cn(
        'peer inline-flex h-4.5 w-8.5 shrink-0 cursor-pointer items-center border border-input bg-data p-0.5 disabled:cursor-not-allowed disabled:border-secondary disabled:bg-muted data-[state=checked]:border-selection-edge data-[state=checked]:bg-primary',
        props.class,
      )
    "
  >
    <SwitchThumb
      data-slot="switch-thumb"
      class="pointer-events-none block size-3 bg-faint-foreground transition-transform duration-tap ease-tap data-[state=checked]:translate-x-4 data-[state=checked]:bg-foreground"
    >
      <slot name="thumb" v-bind="slotProps" />
    </SwitchThumb>
  </SwitchRoot>
</template>
```

`ui/src/components/ui/switch/index.ts`:
```ts
export { default as Switch } from './Switch.vue'
```

- [ ] **Step 3: Toggle group**

`ui/src/components/ui/toggle-group/variants.ts`:
```ts
import type { VariantProps } from 'class-variance-authority'
import type { InjectionKey, Ref } from 'vue'
import { cva } from 'class-variance-authority'

export const ToggleSize = { Default: 'default', Icon: 'icon' } as const
export type ToggleSize = (typeof ToggleSize)[keyof typeof ToggleSize]

export const ToggleGroupType = { Single: 'single', Multiple: 'multiple' } as const
export type ToggleGroupType = (typeof ToggleGroupType)[keyof typeof ToggleGroupType]

// One edge around the group and a divider between items; the chosen item is red, like a
// selected tab (Shadcn Kit.dc.html, "Tabs · Toggle group").
export const toggleGroupItemVariants = cva(
  'inline-flex h-8 shrink-0 cursor-pointer items-center justify-center gap-1 border-l border-secondary-edge text-control text-foreground first:border-l-0 hover:bg-secondary disabled:pointer-events-none disabled:text-faint-foreground data-[state=on]:bg-primary data-[state=on]:text-primary-foreground [&_svg]:pointer-events-none [&_svg:not([class*=size-])]:size-4',
  {
    variants: {
      size: {
        [ToggleSize.Default]: 'px-3',
        [ToggleSize.Icon]: 'w-8.5',
      },
    },
    defaultVariants: {
      size: ToggleSize.Default,
    },
  },
)
export type ToggleGroupItemVariants = VariantProps<typeof toggleGroupItemVariants>

// The group's size, for items that don't set their own.
export const toggleGroupSizeKey: InjectionKey<Readonly<Ref<ToggleSize>>> =
  Symbol('ToggleGroupSize')
```

`ui/src/components/ui/toggle-group/ToggleGroup.vue`:
```vue
<script setup lang="ts">
import type { ToggleGroupRootEmits, ToggleGroupRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { ToggleGroupRoot, useForwardPropsEmits } from 'reka-ui'
import { provide, toRef } from 'vue'
import { cn } from '@/lib/cn'
import { ToggleSize, toggleGroupSizeKey } from './variants'

const props = withDefaults(
  defineProps<
    ToggleGroupRootProps & {
      class?: HTMLAttributes['class']
      size?: ToggleSize
    }
  >(),
  {
    size: ToggleSize.Default,
  },
)
const emits = defineEmits<ToggleGroupRootEmits>()

provide(toggleGroupSizeKey, toRef(props, 'size'))

const delegatedProps = reactiveOmit(props, 'class', 'size')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <ToggleGroupRoot
    v-slot="slotProps"
    data-slot="toggle-group"
    v-bind="forwarded"
    :class="
      cn(
        'flex w-fit items-center border border-secondary-edge bg-data',
        props.class,
      )
    "
  >
    <slot v-bind="slotProps" />
  </ToggleGroupRoot>
</template>
```

`ui/src/components/ui/toggle-group/ToggleGroupItem.vue`:
```vue
<script setup lang="ts">
import type { ToggleGroupItemProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { ToggleGroupItem, useForwardProps } from 'reka-ui'
import { computed, inject } from 'vue'
import { cn } from '@/lib/cn'
import {
  ToggleSize,
  toggleGroupItemVariants,
  toggleGroupSizeKey,
} from './variants'

const props = defineProps<
  ToggleGroupItemProps & {
    class?: HTMLAttributes['class']
    size?: ToggleSize
  }
>()

const groupSize = inject(toggleGroupSizeKey, undefined)
const size = computed(
  () => props.size ?? groupSize?.value ?? ToggleSize.Default,
)

const delegatedProps = reactiveOmit(props, 'class', 'size')
const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <ToggleGroupItem
    v-slot="slotProps"
    data-slot="toggle-group-item"
    :data-size="size"
    v-bind="forwardedProps"
    :class="cn(toggleGroupItemVariants({ size }), props.class)"
  >
    <slot v-bind="slotProps" />
  </ToggleGroupItem>
</template>
```

`ui/src/components/ui/toggle-group/index.ts`:
```ts
export { default as ToggleGroup } from './ToggleGroup.vue'
export { default as ToggleGroupItem } from './ToggleGroupItem.vue'
export {
  ToggleGroupType,
  ToggleSize,
  toggleGroupItemVariants,
  toggleGroupSizeKey,
} from './variants'
export type { ToggleGroupItemVariants } from './variants'
```

- [ ] **Step 4: Tabs**

`ui/src/components/ui/tabs/Tabs.vue`:
```vue
<script setup lang="ts">
import type { TabsRootEmits, TabsRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { TabsRoot, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<TabsRootProps & { class?: HTMLAttributes['class'] }>()
const emits = defineEmits<TabsRootEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <TabsRoot
    v-slot="slotProps"
    data-slot="tabs"
    v-bind="forwarded"
    :class="cn('flex flex-col', props.class)"
  >
    <slot v-bind="slotProps" />
  </TabsRoot>
</template>
```

`ui/src/components/ui/tabs/TabsList.vue`:
```vue
<script setup lang="ts">
import type { TabsListProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { TabsList } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<TabsListProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = reactiveOmit(props, 'class')
</script>

<template>
  <TabsList
    data-slot="tabs-list"
    v-bind="delegatedProps"
    :class="cn('inline-flex w-full items-stretch border-b border-border', props.class)"
  >
    <slot />
  </TabsList>
</template>
```

`ui/src/components/ui/tabs/TabsTrigger.vue`:
```vue
<script setup lang="ts">
import type { TabsTriggerProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { TabsTrigger, useForwardProps } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  TabsTriggerProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <TabsTrigger
    data-slot="tabs-trigger"
    v-bind="forwardedProps"
    :class="
      cn(
        'inline-flex cursor-pointer items-center justify-center gap-1.5 px-3.5 py-2.5 text-control whitespace-nowrap text-muted-foreground hover:text-foreground disabled:pointer-events-none disabled:text-faint-foreground data-[state=active]:bg-primary data-[state=active]:text-primary-foreground [&_svg:not([class*=size-])]:size-4',
        props.class,
      )
    "
  >
    <slot />
  </TabsTrigger>
</template>
```

`ui/src/components/ui/tabs/TabsContent.vue`:
```vue
<script setup lang="ts">
import type { TabsContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { TabsContent } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  TabsContentProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
</script>

<template>
  <!-- The panel takes the sheet's edge, never a radius, and has no top edge of its own:
       the list's underline is it. -->
  <TabsContent
    data-slot="tabs-content"
    v-bind="delegatedProps"
    :class="
      cn(
        'border border-t-0 border-border bg-data p-3 text-body text-foreground-soft',
        props.class,
      )
    "
  >
    <slot />
  </TabsContent>
</template>
```

`ui/src/components/ui/tabs/index.ts`:
```ts
export { default as Tabs } from './Tabs.vue'
export { default as TabsContent } from './TabsContent.vue'
export { default as TabsList } from './TabsList.vue'
export { default as TabsTrigger } from './TabsTrigger.vue'
```

- [ ] **Step 5: Kit sections**

`ui/src/kit/sections/CheckboxSection.vue`:
```vue
<script setup lang="ts">
import { Checkbox, CheckboxState } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Checkbox">
    <div class="flex flex-wrap items-center gap-4">
      <Label><Checkbox :default-value="CheckboxState.Checked" /> mancanti</Label>
      <Label><Checkbox /> spoiler</Label>
      <Label>
        <Checkbox :model-value="CheckboxState.Indeterminate" /> parziale
      </Label>
      <Label><Checkbox disabled /> bloccato</Label>
    </div>
  </KitSection>
</template>
```

`ui/src/kit/sections/SwitchSection.vue`:
```vue
<script setup lang="ts">
import { Label } from '@/components/ui/label'
import { Switch } from '@/components/ui/switch'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Switch">
    <div class="flex flex-col gap-3">
      <Label><Switch :default-value="true" /> Riapri le tab all'avvio</Label>
      <Label><Switch /> Apri in secondo piano</Label>
      <Label><Switch disabled /> Non disponibile</Label>
    </div>
  </KitSection>
</template>
```

`ui/src/kit/sections/ToggleGroupSection.vue`:
```vue
<script setup lang="ts">
import { CheckIcon, LockIcon, StarIcon } from '@lucide/vue'
import {
  ToggleGroup,
  ToggleGroupItem,
  ToggleGroupType,
  ToggleSize,
} from '@/components/ui/toggle-group'
import KitSection from '../KitSection.vue'

const Filter = { All: 'all', Missing: 'missing', Done: 'done' } as const
const Status = { Done: 'done', Now: 'now', Blocked: 'blocked' } as const
</script>

<template>
  <KitSection title="Toggle group">
    <ToggleGroup :type="ToggleGroupType.Single" :default-value="Filter.All">
      <ToggleGroupItem :value="Filter.All">Tutti</ToggleGroupItem>
      <ToggleGroupItem :value="Filter.Missing">Manca</ToggleGroupItem>
      <ToggleGroupItem :value="Filter.Done">Fatto</ToggleGroupItem>
    </ToggleGroup>
    <ToggleGroup
      :type="ToggleGroupType.Single"
      :size="ToggleSize.Icon"
      :default-value="Status.Done"
    >
      <ToggleGroupItem :value="Status.Done" aria-label="Fatto">
        <CheckIcon />
      </ToggleGroupItem>
      <ToggleGroupItem :value="Status.Now" aria-label="Sbloccabile ora">
        <StarIcon />
      </ToggleGroupItem>
      <ToggleGroupItem :value="Status.Blocked" aria-label="Bloccato">
        <LockIcon />
      </ToggleGroupItem>
    </ToggleGroup>
  </KitSection>
</template>
```

`ui/src/kit/sections/TabsSection.vue`:
```vue
<script setup lang="ts">
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import KitSection from '../KitSection.vue'

const Tab = { Items: 'items', Challenges: 'challenges', Secrets: 'secrets' } as const
</script>

<template>
  <KitSection title="Tabs">
    <Tabs :default-value="Tab.Items">
      <TabsList>
        <TabsTrigger :value="Tab.Items">Oggetti</TabsTrigger>
        <TabsTrigger :value="Tab.Challenges">Sfide</TabsTrigger>
        <TabsTrigger :value="Tab.Secrets">Segreti</TabsTrigger>
      </TabsList>
      <TabsContent :value="Tab.Items">118 oggetti mancanti.</TabsContent>
      <TabsContent :value="Tab.Challenges">9 sfide ancora aperte.</TabsContent>
      <TabsContent :value="Tab.Secrets">22 segreti da trovare.</TabsContent>
    </Tabs>
  </KitSection>
</template>
```

In `ui/src/kit/KitPage.vue` add the four imports and, after `<FieldSection />`, `<CheckboxSection />`, `<SwitchSection />`, `<ToggleGroupSection />`, `<TabsSection />`.

- [ ] **Step 6: Checks, build, visual check**

Run: `pnpm --filter ui format && pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test && pnpm --filter ui build`
If `vue-tsc` rejects a `:default-value`/`:model-value` binding type, fix the binding in the Kit section to the exact Reka prop type — never widen the primitive.
Visual (`#kit`): 16px square checkboxes, red with a lit edge when checked, a bar when indeterminate, the tick appearing in two steps; square switches whose thumb jumps; toggle groups with one edge and red selection; tabs with a red active trigger over a panel without top edge.

- [ ] **Step 7: Commit**

```bash
git add ui/src/components/ui/checkbox ui/src/components/ui/switch ui/src/components/ui/toggle-group ui/src/components/ui/tabs ui/src/kit
git commit -m "feat(ui): Checkbox, Switch, Toggle group and Tabs" -m "Selection is red with a lit edge, everywhere. The indeterminate bar is switched by the indicator's data-state in CSS. Toggle group carries its own variants and shares its size through a typed injection key, not the registry's string key."
```

---

### Task 12: Select

**Files:**
- Create: `ui/src/components/ui/select/{variants.ts,Select.vue,SelectTrigger.vue,SelectValue.vue,SelectContent.vue,SelectGroup.vue,SelectLabel.vue,SelectItem.vue,SelectScrollUpButton.vue,SelectScrollDownButton.vue,index.ts}`, `ui/src/kit/sections/SelectSection.vue`
- Modify: `ui/src/kit/KitPage.vue`

**Interfaces:**
- Consumes: `cn`, `Align` (Task 7), `KitSection`.
- Produces: `Select`, `SelectTrigger`, `SelectValue`, `SelectContent` (defaults `position: SelectPosition.Popper`, `align: Align.Start`, `sideOffset: 4`), `SelectGroup`, `SelectLabel`, `SelectItem`, `SelectScrollUpButton`, `SelectScrollDownButton`, `SelectPosition` (`Popper`, `ItemAligned`).

- [ ] **Step 1: Position constant**

`ui/src/components/ui/select/variants.ts`:
```ts
// Reka's two placements for the list. The kit draws it as a panel below the trigger:
// popper. Item-aligned would cover the trigger with the list.
export const SelectPosition = { Popper: 'popper', ItemAligned: 'item-aligned' } as const
export type SelectPosition = (typeof SelectPosition)[keyof typeof SelectPosition]
```

- [ ] **Step 2: Root, value, group, label**

`ui/src/components/ui/select/Select.vue`:
```vue
<script setup lang="ts">
import type { SelectRootEmits, SelectRootProps } from 'reka-ui'
import { SelectRoot, useForwardPropsEmits } from 'reka-ui'

const props = defineProps<SelectRootProps>()
const emits = defineEmits<SelectRootEmits>()

const forwarded = useForwardPropsEmits(props, emits)
</script>

<template>
  <SelectRoot v-slot="slotProps" data-slot="select" v-bind="forwarded">
    <slot v-bind="slotProps" />
  </SelectRoot>
</template>
```

`ui/src/components/ui/select/SelectValue.vue`:
```vue
<script setup lang="ts">
import type { SelectValueProps } from 'reka-ui'
import { SelectValue } from 'reka-ui'

const props = defineProps<SelectValueProps>()
</script>

<template>
  <SelectValue data-slot="select-value" v-bind="props">
    <slot />
  </SelectValue>
</template>
```

`ui/src/components/ui/select/SelectGroup.vue`:
```vue
<script setup lang="ts">
import type { SelectGroupProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { SelectGroup } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  SelectGroupProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
</script>

<template>
  <SelectGroup
    data-slot="select-group"
    v-bind="delegatedProps"
    :class="cn('py-1', props.class)"
  >
    <slot />
  </SelectGroup>
</template>
```

`ui/src/components/ui/select/SelectLabel.vue`:
```vue
<script setup lang="ts">
import type { SelectLabelProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { SelectLabel } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  SelectLabelProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
</script>

<template>
  <SelectLabel
    data-slot="select-label"
    v-bind="delegatedProps"
    :class="cn('px-3 pt-1.5 pb-1 text-label text-subtle-foreground', props.class)"
  >
    <slot />
  </SelectLabel>
</template>
```

- [ ] **Step 3: Trigger**

`ui/src/components/ui/select/SelectTrigger.vue`:
```vue
<script setup lang="ts">
import type { SelectTriggerProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { ChevronDownIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import { SelectIcon, SelectTrigger, useForwardProps } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  SelectTriggerProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <!-- One control height for every field. The chevron turns in three steps when the list
       opens: the trigger's own data-state drives it. -->
  <SelectTrigger
    data-slot="select-trigger"
    v-bind="forwardedProps"
    :class="
      cn(
        'group flex h-control w-fit min-w-40 cursor-pointer items-center justify-between gap-2 rounded-input border border-input bg-data px-3 text-body whitespace-nowrap text-foreground hover:bg-row-hover disabled:cursor-not-allowed disabled:border-secondary disabled:bg-muted data-[placeholder]:text-faint-foreground *:data-[slot=select-value]:line-clamp-1 [&_svg]:pointer-events-none [&_svg]:shrink-0',
        props.class,
      )
    "
  >
    <slot />
    <SelectIcon as-child>
      <ChevronDownIcon
        class="size-3.5 text-muted-foreground transition-transform duration-panel ease-panel group-data-[state=open]:rotate-180"
      />
    </SelectIcon>
  </SelectTrigger>
</template>
```

- [ ] **Step 4: Content, item, scroll buttons**

`ui/src/components/ui/select/SelectContent.vue`:
```vue
<script setup lang="ts">
import type { SelectContentEmits, SelectContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import {
  SelectContent,
  SelectPortal,
  SelectViewport,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '@/lib/cn'
import { Align } from '@/lib/constants/placement'
import SelectScrollDownButton from './SelectScrollDownButton.vue'
import SelectScrollUpButton from './SelectScrollUpButton.vue'
import { SelectPosition } from './variants'

defineOptions({
  inheritAttrs: false,
})

const props = withDefaults(
  defineProps<SelectContentProps & { class?: HTMLAttributes['class'] }>(),
  {
    position: SelectPosition.Popper,
    align: Align.Start,
    sideOffset: 4,
  },
)
const emits = defineEmits<SelectContentEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <SelectPortal>
    <SelectContent
      data-slot="select-content"
      v-bind="{ ...$attrs, ...forwarded }"
      :class="
        cn(
          'relative z-50 max-h-(--reka-select-content-available-height) min-w-(--reka-select-trigger-width) animate-panel-drop overflow-x-hidden overflow-y-auto border border-input bg-popover text-popover-foreground',
          props.class,
        )
      "
    >
      <SelectScrollUpButton />
      <SelectViewport
        :class="
          cn(
            'data-[position=popper]:h-(--reka-select-trigger-height) data-[position=popper]:w-full data-[position=popper]:min-w-(--reka-select-trigger-width)',
          )
        "
      >
        <slot />
      </SelectViewport>
      <SelectScrollDownButton />
    </SelectContent>
  </SelectPortal>
</template>
```

`ui/src/components/ui/select/SelectItem.vue`:
```vue
<script setup lang="ts">
import type { SelectItemProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { CheckIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import {
  SelectItem,
  SelectItemIndicator,
  SelectItemText,
  useForwardProps,
} from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  SelectItemProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <!-- The tick is foreground, not the done green the kit draws: selection is not a state
       of the data. -->
  <SelectItem
    data-slot="select-item"
    v-bind="forwardedProps"
    :class="
      cn(
        'relative flex w-full cursor-default items-center gap-2 py-2 pr-8 pl-3 text-row text-foreground outline-none select-none data-[disabled]:pointer-events-none data-[disabled]:text-faint-foreground data-[highlighted]:bg-secondary [&_svg:not([class*=size-])]:size-3.5',
        props.class,
      )
    "
  >
    <span
      class="pointer-events-none absolute right-3 flex size-3.5 items-center justify-center"
    >
      <SelectItemIndicator>
        <slot name="indicator-icon">
          <CheckIcon />
        </slot>
      </SelectItemIndicator>
    </span>
    <SelectItemText>
      <slot />
    </SelectItemText>
  </SelectItem>
</template>
```

(`outline-none` on the item is correct here: Reka moves a highlight, not the focus ring, through the list, and `data-[highlighted]` draws it.)

`ui/src/components/ui/select/SelectScrollUpButton.vue`:
```vue
<script setup lang="ts">
import type { SelectScrollUpButtonProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { ChevronUpIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import { SelectScrollUpButton, useForwardProps } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  SelectScrollUpButtonProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <SelectScrollUpButton
    data-slot="select-scroll-up-button"
    v-bind="forwardedProps"
    :class="
      cn(
        'flex cursor-default items-center justify-center bg-popover py-1 text-muted-foreground [&_svg:not([class*=size-])]:size-3.5',
        props.class,
      )
    "
  >
    <slot>
      <ChevronUpIcon />
    </slot>
  </SelectScrollUpButton>
</template>
```

`ui/src/components/ui/select/SelectScrollDownButton.vue`:
```vue
<script setup lang="ts">
import type { SelectScrollDownButtonProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { ChevronDownIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import { SelectScrollDownButton, useForwardProps } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  SelectScrollDownButtonProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <SelectScrollDownButton
    data-slot="select-scroll-down-button"
    v-bind="forwardedProps"
    :class="
      cn(
        'flex cursor-default items-center justify-center bg-popover py-1 text-muted-foreground [&_svg:not([class*=size-])]:size-3.5',
        props.class,
      )
    "
  >
    <slot>
      <ChevronDownIcon />
    </slot>
  </SelectScrollDownButton>
</template>
```

`ui/src/components/ui/select/index.ts`:
```ts
export { default as Select } from './Select.vue'
export { default as SelectContent } from './SelectContent.vue'
export { default as SelectGroup } from './SelectGroup.vue'
export { default as SelectItem } from './SelectItem.vue'
export { default as SelectLabel } from './SelectLabel.vue'
export { default as SelectScrollDownButton } from './SelectScrollDownButton.vue'
export { default as SelectScrollUpButton } from './SelectScrollUpButton.vue'
export { default as SelectTrigger } from './SelectTrigger.vue'
export { default as SelectValue } from './SelectValue.vue'
export { SelectPosition } from './variants'
```

- [ ] **Step 5: Kit section**

`ui/src/kit/sections/SelectSection.vue`:
```vue
<script setup lang="ts">
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import KitSection from '../KitSection.vue'

const SortOrder = {
  MissingFirst: 'missingFirst',
  DoneFirst: 'doneFirst',
  Alphabetical: 'alphabetical',
} as const
</script>

<template>
  <KitSection title="Select">
    <Select :default-value="SortOrder.MissingFirst">
      <SelectTrigger>
        <SelectValue placeholder="Ordina per…" />
      </SelectTrigger>
      <SelectContent>
        <SelectGroup>
          <SelectLabel>Ordina per</SelectLabel>
          <SelectItem :value="SortOrder.MissingFirst">Mancanti prima</SelectItem>
          <SelectItem :value="SortOrder.DoneFirst">Fatti prima</SelectItem>
          <SelectItem :value="SortOrder.Alphabetical">Alfabetico</SelectItem>
        </SelectGroup>
      </SelectContent>
    </Select>
    <Select>
      <SelectTrigger>
        <SelectValue placeholder="Nessuna scelta" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem :value="SortOrder.Alphabetical">Alfabetico</SelectItem>
      </SelectContent>
    </Select>
  </KitSection>
</template>
```

In `ui/src/kit/KitPage.vue` add the import and `<SelectSection />` after `<TabsSection />`.

- [ ] **Step 6: Checks, build, visual check**

Run: `pnpm --filter ui format && pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test && pnpm --filter ui build`
Visual (`#kit`): trigger 34px with a 4px radius; the list drops below it in steps, as wide as the trigger; highlighted row on the raised surface; the tick in the foreground colour; the placeholder faint.

- [ ] **Step 7: Commit**

```bash
git add ui/src/components/ui/select ui/src/kit
git commit -m "feat(ui): Select" -m "The list opens below the trigger, as wide as it, dropping in three steps. The selected tick is the foreground colour, not the done green the export drew: selection isn't a state of the data."
```

---

### Task 13: Tooltip, Popover, Collapsible

**Files:**
- Create: `ui/src/components/ui/tooltip/{Tooltip,TooltipTrigger,TooltipContent,TooltipProvider}.vue` + `index.ts`, `ui/src/components/ui/popover/{Popover,PopoverTrigger,PopoverContent,PopoverTitle}.vue` + `index.ts`, `ui/src/components/ui/collapsible/{Collapsible,CollapsibleTrigger,CollapsibleContent}.vue` + `index.ts`
- Create: `ui/src/kit/sections/{TooltipSection,PopoverSection,CollapsibleSection}.vue`
- Modify: `ui/src/kit/KitPage.vue`

**Interfaces:**
- Consumes: `cn`, `Align`, `Button`, `ButtonVariant`, `Checkbox`, `Label`, `KitSection`.
- Produces: `Tooltip`, `TooltipTrigger`, `TooltipContent` (default `sideOffset: 4`), `TooltipProvider` (default `delayDuration: 0`, required once above any tooltip); `Popover`, `PopoverTrigger`, `PopoverContent` (defaults `align: Align.Start`, `sideOffset: 4`), `PopoverTitle`; `Collapsible`, `CollapsibleTrigger`, `CollapsibleContent` (accepts `class`).

- [ ] **Step 1: Tooltip**

`ui/src/components/ui/tooltip/Tooltip.vue`:
```vue
<script setup lang="ts">
import type { TooltipRootEmits, TooltipRootProps } from 'reka-ui'
import { TooltipRoot, useForwardPropsEmits } from 'reka-ui'

const props = defineProps<TooltipRootProps>()
const emits = defineEmits<TooltipRootEmits>()

const forwarded = useForwardPropsEmits(props, emits)
</script>

<template>
  <TooltipRoot v-slot="slotProps" data-slot="tooltip" v-bind="forwarded">
    <slot v-bind="slotProps" />
  </TooltipRoot>
</template>
```

`ui/src/components/ui/tooltip/TooltipTrigger.vue`:
```vue
<script setup lang="ts">
import type { TooltipTriggerProps } from 'reka-ui'
import { TooltipTrigger } from 'reka-ui'

const props = defineProps<TooltipTriggerProps>()
</script>

<template>
  <TooltipTrigger data-slot="tooltip-trigger" v-bind="props">
    <slot />
  </TooltipTrigger>
</template>
```

`ui/src/components/ui/tooltip/TooltipProvider.vue`:
```vue
<script setup lang="ts">
import type { TooltipProviderProps } from 'reka-ui'
import { TooltipProvider } from 'reka-ui'

const props = withDefaults(defineProps<TooltipProviderProps>(), {
  delayDuration: 0,
})
</script>

<template>
  <TooltipProvider v-bind="props">
    <slot />
  </TooltipProvider>
</template>
```

`ui/src/components/ui/tooltip/TooltipContent.vue`:
```vue
<script setup lang="ts">
import type { TooltipContentEmits, TooltipContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { TooltipContent, TooltipPortal, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/cn'

defineOptions({
  inheritAttrs: false,
})

const props = withDefaults(
  defineProps<TooltipContentProps & { class?: HTMLAttributes['class'] }>(),
  {
    sideOffset: 4,
  },
)
const emits = defineEmits<TooltipContentEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <!-- No arrow: a flat edge, like every panel. -->
  <TooltipPortal>
    <TooltipContent
      data-slot="tooltip-content"
      v-bind="{ ...forwarded, ...$attrs }"
      :class="
        cn(
          'z-50 w-fit max-w-xs animate-panel-rise border border-input bg-tooltip px-2 py-1.5 text-caption text-foreground',
          props.class,
        )
      "
    >
      <slot />
    </TooltipContent>
  </TooltipPortal>
</template>
```

`ui/src/components/ui/tooltip/index.ts`:
```ts
export { default as Tooltip } from './Tooltip.vue'
export { default as TooltipContent } from './TooltipContent.vue'
export { default as TooltipProvider } from './TooltipProvider.vue'
export { default as TooltipTrigger } from './TooltipTrigger.vue'
```

- [ ] **Step 2: Popover**

`ui/src/components/ui/popover/Popover.vue`:
```vue
<script setup lang="ts">
import type { PopoverRootEmits, PopoverRootProps } from 'reka-ui'
import { PopoverRoot, useForwardPropsEmits } from 'reka-ui'

const props = defineProps<PopoverRootProps>()
const emits = defineEmits<PopoverRootEmits>()

const forwarded = useForwardPropsEmits(props, emits)
</script>

<template>
  <PopoverRoot v-slot="slotProps" data-slot="popover" v-bind="forwarded">
    <slot v-bind="slotProps" />
  </PopoverRoot>
</template>
```

`ui/src/components/ui/popover/PopoverTrigger.vue`:
```vue
<script setup lang="ts">
import type { PopoverTriggerProps } from 'reka-ui'
import { PopoverTrigger } from 'reka-ui'

const props = defineProps<PopoverTriggerProps>()
</script>

<template>
  <PopoverTrigger data-slot="popover-trigger" v-bind="props">
    <slot />
  </PopoverTrigger>
</template>
```

`ui/src/components/ui/popover/PopoverContent.vue`:
```vue
<script setup lang="ts">
import type { PopoverContentEmits, PopoverContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { PopoverContent, PopoverPortal, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/cn'
import { Align } from '@/lib/constants/placement'

defineOptions({
  inheritAttrs: false,
})

const props = withDefaults(
  defineProps<PopoverContentProps & { class?: HTMLAttributes['class'] }>(),
  {
    align: Align.Start,
    sideOffset: 4,
  },
)
const emits = defineEmits<PopoverContentEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <PopoverPortal>
    <PopoverContent
      data-slot="popover-content"
      v-bind="{ ...$attrs, ...forwarded }"
      :class="
        cn(
          'z-50 flex w-72 max-w-(--reka-popover-content-available-width) animate-panel-rise flex-col border border-input bg-popover text-popover-foreground',
          props.class,
        )
      "
    >
      <slot />
    </PopoverContent>
  </PopoverPortal>
</template>
```

`ui/src/components/ui/popover/PopoverTitle.vue`:
```vue
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <div
    data-slot="popover-title"
    :class="
      cn('bg-band px-3 py-1.5 text-control text-band-foreground', props.class)
    "
  >
    <slot />
  </div>
</template>
```

`ui/src/components/ui/popover/index.ts`:
```ts
export { default as Popover } from './Popover.vue'
export { default as PopoverContent } from './PopoverContent.vue'
export { default as PopoverTitle } from './PopoverTitle.vue'
export { default as PopoverTrigger } from './PopoverTrigger.vue'
```

- [ ] **Step 3: Collapsible**

`ui/src/components/ui/collapsible/Collapsible.vue`:
```vue
<script setup lang="ts">
import type { CollapsibleRootEmits, CollapsibleRootProps } from 'reka-ui'
import { CollapsibleRoot, useForwardPropsEmits } from 'reka-ui'

const props = defineProps<CollapsibleRootProps>()
const emits = defineEmits<CollapsibleRootEmits>()

const forwarded = useForwardPropsEmits(props, emits)
</script>

<template>
  <CollapsibleRoot v-slot="slotProps" data-slot="collapsible" v-bind="forwarded">
    <slot v-bind="slotProps" />
  </CollapsibleRoot>
</template>
```

`ui/src/components/ui/collapsible/CollapsibleTrigger.vue`:
```vue
<script setup lang="ts">
import type { CollapsibleTriggerProps } from 'reka-ui'
import { CollapsibleTrigger } from 'reka-ui'

const props = defineProps<CollapsibleTriggerProps>()
</script>

<template>
  <CollapsibleTrigger data-slot="collapsible-trigger" v-bind="props">
    <slot />
  </CollapsibleTrigger>
</template>
```

`ui/src/components/ui/collapsible/CollapsibleContent.vue`:
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
    data-slot="collapsible-content"
    v-bind="delegatedProps"
    :class="cn('animate-panel-open', props.class)"
  >
    <slot />
  </CollapsibleContent>
</template>
```

`ui/src/components/ui/collapsible/index.ts`:
```ts
export { default as Collapsible } from './Collapsible.vue'
export { default as CollapsibleContent } from './CollapsibleContent.vue'
export { default as CollapsibleTrigger } from './CollapsibleTrigger.vue'
```

- [ ] **Step 4: Kit sections, and the tooltip provider**

`ui/src/kit/sections/TooltipSection.vue`:
```vue
<script setup lang="ts">
import { Button, ButtonVariant } from '@/components/ui/button'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/components/ui/tooltip'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Tooltip">
    <Tooltip>
      <TooltipTrigger as-child>
        <Button :variant="ButtonVariant.Outline" class="w-fit">Passa qui</Button>
      </TooltipTrigger>
      <TooltipContent>Bloccato da 2 voci</TooltipContent>
    </Tooltip>
  </KitSection>
</template>
```

`ui/src/kit/sections/PopoverSection.vue`:
```vue
<script setup lang="ts">
import { ChevronDownIcon } from '@lucide/vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import { Checkbox, CheckboxState } from '@/components/ui/checkbox'
import { Label } from '@/components/ui/label'
import {
  Popover,
  PopoverContent,
  PopoverTitle,
  PopoverTrigger,
} from '@/components/ui/popover'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Popover">
    <Popover>
      <PopoverTrigger as-child>
        <Button :variant="ButtonVariant.Secondary" class="w-fit">
          Filtri
          <ChevronDownIcon class="size-3" />
        </Button>
      </PopoverTrigger>
      <PopoverContent>
        <PopoverTitle>Filtri</PopoverTitle>
        <div class="flex flex-col gap-2.5 p-3">
          <Label>
            <Checkbox :default-value="CheckboxState.Checked" /> Sbloccabili ora
          </Label>
          <Label><Checkbox /> Solo Hard mode</Label>
        </div>
      </PopoverContent>
    </Popover>
  </KitSection>
</template>
```

`ui/src/kit/sections/CollapsibleSection.vue`:
```vue
<script setup lang="ts">
import { ChevronRightIcon } from '@lucide/vue'
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from '@/components/ui/collapsible'
import KitSection from '../KitSection.vue'

const blocking = ['Sacred Orb', 'Death Certificate', "Dad's Note"]
</script>

<template>
  <KitSection title="Collapsible">
    <Collapsible>
      <CollapsibleTrigger
        class="group flex cursor-pointer items-center gap-2 text-row text-foreground"
      >
        <ChevronRightIcon
          class="size-3.5 text-muted-foreground transition-transform duration-panel ease-panel group-data-[state=open]:rotate-90"
        />
        3 voci bloccanti
      </CollapsibleTrigger>
      <CollapsibleContent class="mt-2 border border-border bg-data">
        <p
          v-for="name in blocking"
          :key="name"
          class="border-b border-hairline px-2.5 py-1.5 text-row last:border-b-0"
        >
          {{ name }}
        </p>
      </CollapsibleContent>
    </Collapsible>
  </KitSection>
</template>
```

Replace `ui/src/kit/KitPage.vue`'s template so the whole page sits inside one `TooltipProvider` (add `import { TooltipProvider } from '@/components/ui/tooltip'` and the three section imports):

```vue
<template>
  <TooltipProvider>
    <main
      class="grid min-h-screen grid-cols-3 items-start gap-4 bg-background p-8 text-foreground"
    >
      <ButtonSection />
      <BadgeSection />
      <KbdSection />
      <SeparatorSection />
      <SkeletonSection />
      <CardSection />
      <AlertSection />
      <EmptySection />
      <FieldSection />
      <CheckboxSection />
      <SwitchSection />
      <ToggleGroupSection />
      <TabsSection />
      <SelectSection />
      <TooltipSection />
      <PopoverSection />
      <CollapsibleSection />
    </main>
  </TooltipProvider>
</template>
```

- [ ] **Step 5: Checks, build, visual check**

Run: `pnpm --filter ui format && pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test && pnpm --filter ui build`
Visual (`#kit`): tooltip on the brown tooltip surface, no arrow, rising in steps; popover with a leather title band, aligned to the trigger's left edge; collapsible chevron turning 90° in three steps, content dropping in.

- [ ] **Step 6: Commit**

```bash
git add ui/src/components/ui/tooltip ui/src/components/ui/popover ui/src/components/ui/collapsible ui/src/kit
git commit -m "feat(ui): Tooltip, Popover and Collapsible" -m "Floating panels rise in three steps with a flat edge and no arrow. The popover opens aligned to its trigger's start, with the leather title band. Collapsible content drops in; the consumer's chevron turns on the trigger's data-state."
```

---

### Task 14: Dialog and Command

**Files:**
- Create: `ui/src/components/ui/dialog/{Dialog,DialogTrigger,DialogClose,DialogOverlay,DialogContent,DialogHeader,DialogTitle,DialogDescription,DialogFooter}.vue` + `index.ts`
- Create: `ui/src/components/ui/command/{context.ts,Command.vue,CommandDialog.vue,CommandInput.vue,CommandList.vue,CommandGroup.vue,CommandItem.vue,CommandEmpty.vue,CommandFooter.vue,index.ts}`
- Create: `ui/src/kit/sections/{DialogSection,CommandSection}.vue`
- Modify: `ui/src/kit/KitPage.vue`

**Interfaces:**
- Consumes: `cn`, `useMessages` (key `ui.close`), `Button`, `ButtonVariant`, `Kbd`, `KbdGroup`, `KeyName`, `KitSection`.
- Produces: `Dialog`, `DialogTrigger`, `DialogClose`, `DialogOverlay`, `DialogContent` (prop `showCloseButton?: boolean`, default `true`), `DialogHeader`, `DialogTitle`, `DialogDescription`, `DialogFooter`; `Command` (Reka `ListboxRootProps`), `CommandDialog` (Reka `DialogRootProps` + required `title: string`, `description: string`), `CommandInput`, `CommandList`, `CommandGroup` (prop `heading?: string`), `CommandItem` (requires `value`), `CommandEmpty`, `CommandFooter`, `useCommand`, `provideCommandContext`, `useCommandGroup`, `provideCommandGroupContext`.

- [ ] **Step 1: Dialog pass-through parts**

`ui/src/components/ui/dialog/Dialog.vue`:
```vue
<script setup lang="ts">
import type { DialogRootEmits, DialogRootProps } from 'reka-ui'
import { DialogRoot, useForwardPropsEmits } from 'reka-ui'

const props = defineProps<DialogRootProps>()
const emits = defineEmits<DialogRootEmits>()

const forwarded = useForwardPropsEmits(props, emits)
</script>

<template>
  <DialogRoot v-slot="slotProps" data-slot="dialog" v-bind="forwarded">
    <slot v-bind="slotProps" />
  </DialogRoot>
</template>
```

`ui/src/components/ui/dialog/DialogTrigger.vue`:
```vue
<script setup lang="ts">
import type { DialogTriggerProps } from 'reka-ui'
import { DialogTrigger } from 'reka-ui'

const props = defineProps<DialogTriggerProps>()
</script>

<template>
  <DialogTrigger data-slot="dialog-trigger" v-bind="props">
    <slot />
  </DialogTrigger>
</template>
```

`ui/src/components/ui/dialog/DialogClose.vue`:
```vue
<script setup lang="ts">
import type { DialogCloseProps } from 'reka-ui'
import { DialogClose } from 'reka-ui'

const props = defineProps<DialogCloseProps>()
</script>

<template>
  <DialogClose data-slot="dialog-close" v-bind="props">
    <slot />
  </DialogClose>
</template>
```

- [ ] **Step 2: Overlay and content**

`ui/src/components/ui/dialog/DialogOverlay.vue`:
```vue
<script setup lang="ts">
import type { DialogOverlayProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { DialogOverlay } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  DialogOverlayProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
</script>

<template>
  <!-- Solid dim, no blur: the alpha lives in the overlay token. -->
  <DialogOverlay
    data-slot="dialog-overlay"
    v-bind="delegatedProps"
    :class="cn('fixed inset-0 z-50 bg-overlay', props.class)"
  >
    <slot />
  </DialogOverlay>
</template>
```

`ui/src/components/ui/dialog/DialogContent.vue`:
```vue
<script setup lang="ts">
import type { DialogContentEmits, DialogContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { XIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import {
  DialogClose,
  DialogContent,
  DialogPortal,
  useForwardPropsEmits,
} from 'reka-ui'
import { useMessages } from '@/i18n'
import { cn } from '@/lib/cn'
import DialogOverlay from './DialogOverlay.vue'

defineOptions({
  inheritAttrs: false,
})

const props = withDefaults(
  defineProps<
    DialogContentProps & {
      class?: HTMLAttributes['class']
      showCloseButton?: boolean
    }
  >(),
  {
    showCloseButton: true,
  },
)
const emits = defineEmits<DialogContentEmits>()

const delegatedProps = reactiveOmit(props, 'class', 'showCloseButton')
const forwarded = useForwardPropsEmits(delegatedProps, emits)

const { t } = useMessages()
</script>

<template>
  <!-- The sheet rises in five steps (Motion.dc.html, --m-sheet). The close control sits on
       the header band, so its colour is the band's text. -->
  <DialogPortal>
    <DialogOverlay />
    <DialogContent
      data-slot="dialog-content"
      v-bind="{ ...$attrs, ...forwarded }"
      :class="
        cn(
          'fixed top-1/2 left-1/2 z-50 flex w-full max-w-md -translate-x-1/2 -translate-y-1/2 animate-sheet-rise flex-col border border-input bg-sheet text-foreground',
          props.class,
        )
      "
    >
      <slot />
      <DialogClose
        v-if="showCloseButton"
        data-slot="dialog-close"
        class="absolute top-1.5 right-2 grid size-5 cursor-pointer place-items-center text-band-foreground [&_svg]:size-3.5"
      >
        <XIcon />
        <span class="sr-only">{{ t('ui.close') }}</span>
      </DialogClose>
    </DialogContent>
  </DialogPortal>
</template>
```

- [ ] **Step 3: Header, title, description, footer**

`ui/src/components/ui/dialog/DialogHeader.vue`:
```vue
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <!-- The leather band; the right padding leaves room for the close control. -->
  <div
    data-slot="dialog-header"
    :class="
      cn(
        'flex items-center gap-2 bg-band py-2 pr-9 pl-3 text-band-foreground',
        props.class,
      )
    "
  >
    <slot />
  </div>
</template>
```

`ui/src/components/ui/dialog/DialogTitle.vue`:
```vue
<script setup lang="ts">
import type { DialogTitleProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { DialogTitle, useForwardProps } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  DialogTitleProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <DialogTitle
    data-slot="dialog-title"
    v-bind="forwardedProps"
    :class="cn('text-control', props.class)"
  >
    <slot />
  </DialogTitle>
</template>
```

`ui/src/components/ui/dialog/DialogDescription.vue`:
```vue
<script setup lang="ts">
import type { DialogDescriptionProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { DialogDescription, useForwardProps } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  DialogDescriptionProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
const forwardedProps = useForwardProps(delegatedProps)
</script>

<template>
  <DialogDescription
    data-slot="dialog-description"
    v-bind="forwardedProps"
    :class="cn('px-3 py-3.5 text-body text-foreground-soft', props.class)"
  >
    <slot />
  </DialogDescription>
</template>
```

`ui/src/components/ui/dialog/DialogFooter.vue`:
```vue
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <div
    data-slot="dialog-footer"
    :class="
      cn('flex justify-end gap-2 border-t border-border px-3 py-3', props.class)
    "
  >
    <slot />
  </div>
</template>
```

`ui/src/components/ui/dialog/index.ts`:
```ts
export { default as Dialog } from './Dialog.vue'
export { default as DialogClose } from './DialogClose.vue'
export { default as DialogContent } from './DialogContent.vue'
export { default as DialogDescription } from './DialogDescription.vue'
export { default as DialogFooter } from './DialogFooter.vue'
export { default as DialogHeader } from './DialogHeader.vue'
export { default as DialogOverlay } from './DialogOverlay.vue'
export { default as DialogTitle } from './DialogTitle.vue'
export { default as DialogTrigger } from './DialogTrigger.vue'
```

- [ ] **Step 4: Command context and root**

`ui/src/components/ui/command/context.ts`:
```ts
import type { Ref } from 'vue'
import { createContext } from 'reka-ui'

export interface CommandFilterState {
  search: string
  filtered: {
    /** How many items match the search. */
    count: number
    /** Item id to 1 when it matches, 0 when it doesn't. */
    items: Map<string, number>
    /** Groups with at least one matching item. */
    groups: Set<string>
  }
}

export const [useCommand, provideCommandContext] = createContext<{
  allItems: Ref<Map<string, string>>
  allGroups: Ref<Map<string, Set<string>>>
  filterState: CommandFilterState
}>('Command')

export const [useCommandGroup, provideCommandGroupContext] = createContext<{
  id?: string
}>('CommandGroup')
```

`ui/src/components/ui/command/Command.vue`:
```vue
<script setup lang="ts">
import type { ListboxRootEmits, ListboxRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { ListboxRoot, useFilter, useForwardPropsEmits } from 'reka-ui'
import { reactive, ref, watch } from 'vue'
import { cn } from '@/lib/cn'
import type { CommandFilterState } from './context'
import { provideCommandContext } from './context'

const props = withDefaults(
  defineProps<ListboxRootProps & { class?: HTMLAttributes['class'] }>(),
  {
    modelValue: '',
    highlightOnHover: true,
  },
)
const emits = defineEmits<ListboxRootEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)

const allItems = ref<Map<string, string>>(new Map())
const allGroups = ref<Map<string, Set<string>>>(new Map())

const { contains } = useFilter({ sensitivity: 'base' })
const filterState = reactive<CommandFilterState>({
  search: '',
  filtered: { count: 0, items: new Map(), groups: new Set() },
})

// With an empty search every item shows itself; otherwise score each item once, then
// derive the visible groups and the count from the scores.
const filterItems = () => {
  if (!filterState.search) {
    filterState.filtered.count = allItems.value.size
    return
  }
  const scores = new Map(
    [...allItems.value].map(([id, text]): [string, number] => [
      id,
      contains(text, filterState.search) ? 1 : 0,
    ]),
  )
  filterState.filtered.items = scores
  filterState.filtered.groups = new Set(
    [...allGroups.value]
      .filter(([, itemIds]) =>
        [...itemIds].some((itemId) => (scores.get(itemId) ?? 0) > 0),
      )
      .map(([groupId]) => groupId),
  )
  filterState.filtered.count = [...scores.values()].filter(
    (score) => score > 0,
  ).length
}

watch(() => filterState.search, filterItems)

provideCommandContext({ allItems, allGroups, filterState })
</script>

<template>
  <ListboxRoot
    data-slot="command"
    v-bind="forwarded"
    :class="
      cn(
        'flex size-full flex-col overflow-hidden border border-input bg-popover text-popover-foreground',
        props.class,
      )
    "
  >
    <slot />
  </ListboxRoot>
</template>
```

- [ ] **Step 5: Command dialog, input, list**

`ui/src/components/ui/command/CommandDialog.vue`:
```vue
<script setup lang="ts">
import type { DialogRootEmits, DialogRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { useForwardPropsEmits } from 'reka-ui'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { cn } from '@/lib/cn'
import Command from './Command.vue'

// Title and description are required: a screen reader announces them, and a default
// would be an English string nobody translated.
const props = defineProps<
  DialogRootProps & {
    title: string
    description: string
    class?: HTMLAttributes['class']
  }
>()
const emits = defineEmits<DialogRootEmits>()

const delegatedProps = reactiveOmit(props, 'title', 'description', 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <Dialog v-slot="slotProps" v-bind="forwarded">
    <DialogContent
      :show-close-button="false"
      :class="cn('top-1/3 max-w-lg translate-y-0', props.class)"
    >
      <DialogHeader class="sr-only">
        <DialogTitle>{{ title }}</DialogTitle>
        <DialogDescription>{{ description }}</DialogDescription>
      </DialogHeader>
      <Command class="border-0">
        <slot v-bind="slotProps" />
      </Command>
    </DialogContent>
  </Dialog>
</template>
```

`ui/src/components/ui/command/CommandInput.vue`:
```vue
<script setup lang="ts">
import type { ListboxFilterProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { SearchIcon } from '@lucide/vue'
import { reactiveOmit } from '@vueuse/core'
import { ListboxFilter, useForwardProps } from 'reka-ui'
import { cn } from '@/lib/cn'
import { useCommand } from './context'

defineOptions({
  inheritAttrs: false,
})

const props = defineProps<
  ListboxFilterProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
const forwardedProps = useForwardProps(delegatedProps)

const { filterState } = useCommand()
</script>

<template>
  <!-- A plain row, not the registry's InputGroup: the caret is the focus here, so the
       input itself carries no ring. -->
  <div
    data-slot="command-input-wrapper"
    class="flex items-center gap-2 border-b border-hairline px-3 py-2"
  >
    <SearchIcon class="size-3.5 shrink-0 text-muted-foreground" />
    <ListboxFilter
      v-bind="{ ...forwardedProps, ...$attrs }"
      v-model="filterState.search"
      data-slot="command-input"
      auto-focus
      :class="
        cn(
          'w-full bg-transparent text-body text-foreground outline-none placeholder:text-faint-foreground',
          props.class,
        )
      "
    />
  </div>
</template>
```

`ui/src/components/ui/command/CommandList.vue`:
```vue
<script setup lang="ts">
import type { ListboxContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { ListboxContent, useForwardProps } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  ListboxContentProps & { class?: HTMLAttributes['class'] }
>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardProps(delegatedProps)
</script>

<template>
  <ListboxContent
    data-slot="command-list"
    v-bind="forwarded"
    :class="
      cn('max-h-72 scroll-py-1 overflow-x-hidden overflow-y-auto', props.class)
    "
  >
    <div role="presentation">
      <slot />
    </div>
  </ListboxContent>
</template>
```

- [ ] **Step 6: Group, item, empty, footer, index**

`ui/src/components/ui/command/CommandGroup.vue`:
```vue
<script setup lang="ts">
import type { ListboxGroupProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { ListboxGroup, ListboxGroupLabel, useId } from 'reka-ui'
import { computed, onMounted, onUnmounted } from 'vue'
import { cn } from '@/lib/cn'
import { provideCommandGroupContext, useCommand } from './context'

const props = defineProps<
  ListboxGroupProps & {
    class?: HTMLAttributes['class']
    heading?: string
  }
>()

const delegatedProps = reactiveOmit(props, 'class', 'heading')

const { allGroups, filterState } = useCommand()
const id = useId()

const isRender = computed(
  () => !filterState.search || filterState.filtered.groups.has(id),
)

provideCommandGroupContext({ id })
onMounted(() => {
  if (!allGroups.value.has(id)) allGroups.value.set(id, new Set())
})
onUnmounted(() => {
  allGroups.value.delete(id)
})
</script>

<template>
  <ListboxGroup
    v-bind="delegatedProps"
    :id="id"
    data-slot="command-group"
    :class="cn('overflow-hidden py-1 text-foreground', props.class)"
    :hidden="isRender ? undefined : true"
  >
    <ListboxGroupLabel
      v-if="heading"
      data-slot="command-group-heading"
      class="px-3 pt-1.5 pb-1 text-label text-subtle-foreground"
    >
      {{ heading }}
    </ListboxGroupLabel>
    <slot />
  </ListboxGroup>
</template>
```

`ui/src/components/ui/command/CommandItem.vue`:
```vue
<script setup lang="ts">
import type { ListboxItemEmits, ListboxItemProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit, useCurrentElement } from '@vueuse/core'
import { ListboxItem, useForwardPropsEmits, useId } from 'reka-ui'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { cn } from '@/lib/cn'
import { useCommand, useCommandGroup } from './context'

const props = defineProps<
  ListboxItemProps & { class?: HTMLAttributes['class'] }
>()
const emits = defineEmits<ListboxItemEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)

const id = useId()
const { filterState, allItems, allGroups } = useCommand()
// null: an item outside any group is allowed.
const groupContext = useCommandGroup(null)

// Before its first score an item isn't in the map yet: it renders once to register.
const isRender = computed(
  () =>
    !filterState.search || (filterState.filtered.items.get(id) ?? 1) > 0,
)

const itemRef = ref()
const currentElement = useCurrentElement(itemRef)

const registerInGroup = (groupId: string) => {
  const group = allGroups.value.get(groupId)
  if (group) {
    group.add(id)
    return
  }
  allGroups.value.set(groupId, new Set([id]))
}

onMounted(() => {
  if (!(currentElement.value instanceof HTMLElement)) return
  allItems.value.set(
    id,
    currentElement.value.textContent ?? props.value?.toString() ?? '',
  )
  if (groupContext?.id) registerInGroup(groupContext.id)
})
onUnmounted(() => {
  allItems.value.delete(id)
})
</script>

<template>
  <!-- Reka moves a highlight through the list rather than the focus ring. -->
  <ListboxItem
    v-if="isRender"
    v-bind="forwarded"
    :id="id"
    ref="itemRef"
    data-slot="command-item"
    :class="
      cn(
        'relative flex cursor-default items-center gap-2 px-3 py-2 text-row outline-none select-none data-[disabled]:pointer-events-none data-[disabled]:text-faint-foreground data-[highlighted]:bg-secondary [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*=size-])]:size-3.5',
        props.class,
      )
    "
    @select="() => (filterState.search = '')"
  >
    <slot />
  </ListboxItem>
</template>
```

`ui/src/components/ui/command/CommandEmpty.vue`:
```vue
<script setup lang="ts">
import type { PrimitiveProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { Primitive } from 'reka-ui'
import { computed } from 'vue'
import { cn } from '@/lib/cn'
import { useCommand } from './context'

const props = defineProps<PrimitiveProps & { class?: HTMLAttributes['class'] }>()

const delegatedProps = reactiveOmit(props, 'class')

const { filterState } = useCommand()
const isRender = computed(
  () => !!filterState.search && filterState.filtered.count === 0,
)
</script>

<template>
  <Primitive
    v-if="isRender"
    data-slot="command-empty"
    v-bind="delegatedProps"
    :class="cn('py-6 text-center text-row text-muted-foreground', props.class)"
  >
    <slot />
  </Primitive>
</template>
```

`ui/src/components/ui/command/CommandFooter.vue`:
```vue
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <!-- Ours, not the registry's: the palette's count and key hints (Shadcn Kit.dc.html). -->
  <div
    data-slot="command-footer"
    :class="
      cn(
        'flex items-center justify-between gap-2 border-t border-hairline px-3 py-2 text-label text-subtle-foreground',
        props.class,
      )
    "
  >
    <slot />
  </div>
</template>
```

`ui/src/components/ui/command/index.ts`:
```ts
export { default as Command } from './Command.vue'
export { default as CommandDialog } from './CommandDialog.vue'
export { default as CommandEmpty } from './CommandEmpty.vue'
export { default as CommandFooter } from './CommandFooter.vue'
export { default as CommandGroup } from './CommandGroup.vue'
export { default as CommandInput } from './CommandInput.vue'
export { default as CommandItem } from './CommandItem.vue'
export { default as CommandList } from './CommandList.vue'
export {
  provideCommandContext,
  provideCommandGroupContext,
  useCommand,
  useCommandGroup,
} from './context'
export type { CommandFilterState } from './context'
```

- [ ] **Step 7: Kit sections**

`ui/src/kit/sections/DialogSection.vue`:
```vue
<script setup lang="ts">
import { Button, ButtonVariant } from '@/components/ui/button'
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Dialog">
    <Dialog>
      <DialogTrigger as-child>
        <Button class="w-fit">Apri dialog</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Rileggi il salvataggio</DialogTitle>
        </DialogHeader>
        <DialogDescription>
          Le schermate mostreranno quello che il file dichiara adesso.
        </DialogDescription>
        <DialogFooter>
          <DialogClose as-child>
            <Button :variant="ButtonVariant.Outline">Annulla</Button>
          </DialogClose>
          <DialogClose as-child>
            <Button>Rileggi</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  </KitSection>
</template>
```

`ui/src/kit/sections/CommandSection.vue`:
```vue
<script setup lang="ts">
import { ArrowDownIcon, ArrowUpIcon, CornerDownLeftIcon } from '@lucide/vue'
import { ref } from 'vue'
import { Button, ButtonVariant } from '@/components/ui/button'
import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandFooter,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import { Kbd, KbdGroup } from '@/components/ui/kbd'
import { KeyName } from '@/lib/constants/keyNames'
import KitSection from '../KitSection.vue'

const items = ['Brimstone', 'Brimstone Bombs', 'Godhead']
const screens = ['Unlock', 'Completamento']
const paletteOpen = ref(false)
</script>

<template>
  <KitSection title="Command">
    <Command>
      <CommandInput placeholder="Cerca in tutto…" />
      <CommandList>
        <CommandEmpty>Nessun risultato.</CommandEmpty>
        <CommandGroup heading="Oggetti">
          <CommandItem v-for="name in items" :key="name" :value="name">
            {{ name }}
          </CommandItem>
        </CommandGroup>
        <CommandGroup heading="Schermate">
          <CommandItem v-for="name in screens" :key="name" :value="name">
            {{ name }}
          </CommandItem>
        </CommandGroup>
      </CommandList>
      <CommandFooter>
        <span class="flex items-center gap-1">
          <Kbd><ArrowUpIcon /></Kbd><Kbd><ArrowDownIcon /></Kbd> naviga
        </span>
        <span class="flex items-center gap-1">
          <Kbd><CornerDownLeftIcon /></Kbd> apri
        </span>
      </CommandFooter>
    </Command>
    <Button
      :variant="ButtonVariant.Secondary"
      class="w-fit"
      @click="paletteOpen = true"
    >
      <KbdGroup>
        <Kbd>{{ KeyName.Ctrl }}</Kbd>
        <Kbd>{{ KeyName.K }}</Kbd>
      </KbdGroup>
      Palette
    </Button>
    <CommandDialog
      v-model:open="paletteOpen"
      title="Ricerca globale"
      description="Cerca oggetti, schermate e pagine della wiki."
    >
      <CommandInput placeholder="Cerca in tutto…" />
      <CommandList>
        <CommandEmpty>Nessun risultato.</CommandEmpty>
        <CommandGroup heading="Oggetti">
          <CommandItem v-for="name in items" :key="name" :value="name">
            {{ name }}
          </CommandItem>
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  </KitSection>
</template>
```

In `ui/src/kit/KitPage.vue` add the two imports and, after `<CollapsibleSection />`, `<DialogSection />` and `<CommandSection />`.

- [ ] **Step 8: Checks, build, visual check**

Run: `pnpm --filter ui format && pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test && pnpm --filter ui build`
If `useCommandGroup(null)` doesn't type-check against Reka's `createContext`, read `node_modules/reka-ui/dist/index.d.ts` for the injector's signature and pass the fallback it accepts; the requirement is that an item outside a group doesn't throw.
Visual (`#kit`): the dialog rises in steps over a solid dim, with the leather header and a close cross whose label is "Chiudi" in Italian (inspect the `sr-only` span); typing "bri" in the inline command hides Godhead and the Schermate group; "zzz" shows "Nessun risultato."; the palette button opens the dialog version one third down the window.

- [ ] **Step 9: Commit**

```bash
git add ui/src/components/ui/dialog ui/src/components/ui/command ui/src/kit
git commit -m "feat(ui): Dialog and Command, the parts of the Ctrl+K palette" -m "The dialog rises in five steps over a solid dim, with the leather header and a close control labelled through i18n. Command filters its items functionally, keeps its contexts in their own module, and has a plain input row instead of the registry's InputGroup. CommandDialog requires its title and description rather than defaulting to English."
```

---

### Task 15: Table, with three densities

**Files:**
- Create: `ui/src/components/ui/table/{variants.ts,Table.vue,TableHeader.vue,TableBody.vue,TableRow.vue,TableHead.vue,TableCell.vue,TableFooter.vue,index.ts}`, `ui/src/kit/sections/TableSection.vue`
- Modify: `ui/src/kit/KitPage.vue`

**Interfaces:**
- Consumes: `cn`, `Badge`, `BadgeVariant`, `Checkbox`, `KitSection`.
- Produces: `Table` (prop `density?: TableDensity`, default `Normal`), `TableHeader`, `TableBody`, `TableRow`, `TableHead`, `TableCell`, `TableFooter`, `TableDensity` (`Compact`, `Normal`, `Wide`), `tableBodyVariants`, `tableDensityKey`.

- [ ] **Step 1: Variants**

`ui/src/components/ui/table/variants.ts`:
```ts
import type { VariantProps } from 'class-variance-authority'
import type { InjectionKey, Ref } from 'vue'
import { cva } from 'class-variance-authority'

export const TableDensity = { Compact: 'compact', Normal: 'normal', Wide: 'wide' } as const
export type TableDensity = (typeof TableDensity)[keyof typeof TableDensity]

// Chrome e Stati.dc.html, "Densità": only the body rows' height changes; text stays
// text-row and the header keeps its own height.
export const tableBodyVariants = cva('[&>tr:last-child]:border-b-0', {
  variants: {
    density: {
      [TableDensity.Compact]: '[&>tr]:h-row-compact',
      [TableDensity.Normal]: '[&>tr]:h-row',
      [TableDensity.Wide]: '[&>tr]:h-row-wide',
    },
  },
  defaultVariants: {
    density: TableDensity.Normal,
  },
})
export type TableBodyVariants = VariantProps<typeof tableBodyVariants>

// Set on Table, read by TableBody.
export const tableDensityKey: InjectionKey<Readonly<Ref<TableDensity>>> =
  Symbol('TableDensity')
```

- [ ] **Step 2: Table and body**

`ui/src/components/ui/table/Table.vue`:
```vue
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { provide, toRef } from 'vue'
import { cn } from '@/lib/cn'
import { TableDensity, tableDensityKey } from './variants'

const props = withDefaults(
  defineProps<{
    class?: HTMLAttributes['class']
    density?: TableDensity
  }>(),
  {
    density: TableDensity.Normal,
  },
)

provide(tableDensityKey, toRef(props, 'density'))
</script>

<template>
  <div
    data-slot="table-container"
    class="relative w-full overflow-x-auto border border-border bg-data"
  >
    <table
      data-slot="table"
      :class="cn('w-full text-row text-foreground', props.class)"
    >
      <slot />
    </table>
  </div>
</template>
```

`ui/src/components/ui/table/TableBody.vue`:
```vue
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { computed, inject } from 'vue'
import { cn } from '@/lib/cn'
import { TableDensity, tableBodyVariants, tableDensityKey } from './variants'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()

const tableDensity = inject(tableDensityKey, undefined)
const density = computed(() => tableDensity?.value ?? TableDensity.Normal)
</script>

<template>
  <tbody
    data-slot="table-body"
    :class="cn(tableBodyVariants({ density }), props.class)"
  >
    <slot />
  </tbody>
</template>
```

- [ ] **Step 3: Header, row, cells, footer**

All four files below use this script block:

```vue
<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/cn'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>
```

`TableHeader.vue` template:
```vue
<template>
  <!-- The leather band. Its row opts out of the body's stripes and hover: the thead
       selector outranks the row's own classes. -->
  <thead
    data-slot="table-header"
    :class="
      cn(
        'text-band-foreground [&>tr]:border-b-0 [&>tr]:bg-band [&>tr:hover]:bg-band',
        props.class,
      )
    "
  >
    <slot />
  </thead>
</template>
```

`TableRow.vue` template:
```vue
<template>
  <!-- A focused row keeps the ring inside, so it doesn't push its neighbours. -->
  <tr
    data-slot="table-row"
    :class="
      cn(
        'border-b border-hairline even:bg-row-alt hover:bg-row-hover focus-visible:-outline-offset-2 data-[state=selected]:bg-row-hover',
        props.class,
      )
    "
  >
    <slot />
  </tr>
</template>
```

`TableHead.vue` template:
```vue
<template>
  <th
    data-slot="table-head"
    :class="
      cn(
        'h-7 px-2 text-left align-middle text-label whitespace-nowrap',
        props.class,
      )
    "
  >
    <slot />
  </th>
</template>
```

`TableCell.vue` template:
```vue
<template>
  <td
    data-slot="table-cell"
    :class="cn('px-2 align-middle whitespace-nowrap', props.class)"
  >
    <slot />
  </td>
</template>
```

`TableFooter.vue` template:
```vue
<template>
  <tfoot
    data-slot="table-footer"
    :class="
      cn(
        'border-t border-border text-label text-subtle-foreground [&>tr]:border-b-0 [&>tr:hover]:bg-transparent',
        props.class,
      )
    "
  >
    <slot />
  </tfoot>
</template>
```

`ui/src/components/ui/table/index.ts`:
```ts
export { default as Table } from './Table.vue'
export { default as TableBody } from './TableBody.vue'
export { default as TableCell } from './TableCell.vue'
export { default as TableFooter } from './TableFooter.vue'
export { default as TableHead } from './TableHead.vue'
export { default as TableHeader } from './TableHeader.vue'
export { default as TableRow } from './TableRow.vue'
export { TableDensity, tableBodyVariants, tableDensityKey } from './variants'
export type { TableBodyVariants } from './variants'
```

- [ ] **Step 4: Kit section**

`ui/src/kit/sections/TableSection.vue`:
```vue
<script setup lang="ts">
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { Checkbox } from '@/components/ui/checkbox'
import {
  Table,
  TableBody,
  TableCell,
  TableDensity,
  TableFooter,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import KitSection from '../KitSection.vue'

const rows = [
  {
    name: 'Godhead',
    condition: 'Sfida 33',
    status: BadgeVariant.Now,
    label: 'Ora',
    quality: '4',
  },
  {
    name: 'Sacred Orb',
    condition: 'The Lost, Mother (Hard)',
    status: BadgeVariant.Blocked,
    label: 'Da 2',
    quality: '4',
  },
  {
    name: "Dad's Note",
    condition: 'Colonna non ancora localizzata',
    status: BadgeVariant.Unknown,
    label: '',
    quality: '—',
  },
  {
    name: 'Brimstone',
    condition: "10 uccisioni di Mom's Heart",
    status: BadgeVariant.Done,
    label: 'Fatto',
    quality: '4',
  },
]
</script>

<template>
  <KitSection title="Table" class="col-span-2">
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead class="w-9"><Checkbox /></TableHead>
          <TableHead class="w-9" />
          <TableHead>Sblocca</TableHead>
          <TableHead>Condizione</TableHead>
          <TableHead>Stato</TableHead>
          <TableHead class="text-right">Q</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow v-for="row in rows" :key="row.name" tabindex="0">
          <TableCell><Checkbox /></TableCell>
          <TableCell>
            <div class="size-5 border border-border hatch-placeholder" />
          </TableCell>
          <TableCell>{{ row.name }}</TableCell>
          <TableCell class="text-foreground-soft">{{ row.condition }}</TableCell>
          <TableCell>
            <Badge :variant="row.status">{{ row.label }}</Badge>
          </TableCell>
          <TableCell class="text-right">{{ row.quality }}</TableCell>
        </TableRow>
      </TableBody>
      <TableFooter>
        <TableRow>
          <TableCell :colspan="6" class="py-2">0 di 4 selezionate</TableCell>
        </TableRow>
      </TableFooter>
    </Table>
    <Table :density="TableDensity.Compact">
      <TableBody>
        <TableRow v-for="row in rows" :key="row.name">
          <TableCell>{{ row.name }}</TableCell>
          <TableCell class="text-right">{{ row.quality }}</TableCell>
        </TableRow>
      </TableBody>
    </Table>
  </KitSection>
</template>
```

In `ui/src/kit/KitPage.vue` add the import and `<TableSection />` after `<CommandSection />`. (`class="col-span-2"` falls through to the section root.)

- [ ] **Step 5: Checks, build, visual check**

Run: `pnpm --filter ui format && pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test && pnpm --filter ui build`
Visual (`#kit`): a leather header row that doesn't change on hover; 30px body rows with alternate stripes, hover on every row including striped ones; Tab onto a row shows the ring inside its edges; the compact table's rows are 24px.

- [ ] **Step 6: Commit**

```bash
git add ui/src/components/ui/table ui/src/kit
git commit -m "feat(ui): Table, with three densities" -m "Body rows are 24, 30 or 40px, set on Table and read by TableBody through a typed injection key; text and header don't change with density. The header is the leather band and opts out of stripes and hover; a focused row keeps its ring inside."
```

---

### Task 16: Progress, with the unreadable segment

Spec catalogue, Progress. The logic is `progressShares`, test-first.

**Files:**
- Create: `ui/src/components/ui/progress/progressShares.ts`, `ui/src/components/ui/progress/Progress.vue`, `ui/src/components/ui/progress/index.ts`, `ui/src/kit/sections/ProgressSection.vue`
- Test: `ui/src/components/ui/progress/progressShares.test.ts`
- Modify: `ui/src/kit/KitPage.vue`

**Interfaces:**
- Consumes: `cn`, `Label`, `KitSection`.
- Produces: `progressShares(value: number, unknown: number, max: number): ProgressShares` with `ProgressShares = { value: number; unknown: number }` (percentages); `Progress` (Reka `ProgressRootProps` — `modelValue`, `max` — plus `unknown?: number`, `class?`).

- [ ] **Step 1: Write the failing test**

`ui/src/components/ui/progress/progressShares.test.ts`:
```ts
import { describe, expect, it } from 'vitest'
import { progressShares } from './progressShares'

describe('progressShares', () => {
  it('turns value and unknown into shares of max', () => {
    expect(progressShares(25, 10, 100)).toEqual({ value: 25, unknown: 10 })
  })

  it('scales to a max other than 100', () => {
    expect(progressShares(1, 1, 4)).toEqual({ value: 25, unknown: 25 })
  })

  it('clamps negative inputs to zero', () => {
    expect(progressShares(-5, -3, 100)).toEqual({ value: 0, unknown: 0 })
  })

  it('clamps a value beyond max and leaves no room for unknown', () => {
    expect(progressShares(150, 10, 100)).toEqual({ value: 100, unknown: 0 })
  })

  it('gives unknown only the room the value leaves', () => {
    expect(progressShares(80, 40, 100)).toEqual({ value: 80, unknown: 20 })
  })

  it('is empty, not NaN, when there is nothing to measure', () => {
    expect(progressShares(5, 5, 0)).toEqual({ value: 0, unknown: 0 })
  })
})
```

- [ ] **Step 2: Run it to verify it fails**

Run: `pnpm ui:test`
Expected: FAIL — cannot resolve `./progressShares`.

- [ ] **Step 3: Implement**

`ui/src/components/ui/progress/progressShares.ts`:
```ts
import { clamp } from 'lodash-es'

export interface ProgressShares {
  /** Percent of the bar that is done. */
  value: number
  /** Percent of the bar that can't be read: never "not done", never part of `value`. */
  unknown: number
}

// The bar's two filled segments as percentages of max. Together they never pass 100, and
// a bar with nothing to measure is empty rather than NaN.
export const progressShares = (
  value: number,
  unknown: number,
  max: number,
): ProgressShares => {
  if (max <= 0) return { value: 0, unknown: 0 }
  const done = clamp(value, 0, max)
  const unreadable = clamp(unknown, 0, max - done)
  return { value: (done / max) * 100, unknown: (unreadable / max) * 100 }
}
```

- [ ] **Step 4: Run the tests**

Run: `pnpm ui:test`
Expected: PASS (6 new tests).

- [ ] **Step 5: Component and index**

`ui/src/components/ui/progress/Progress.vue`:
```vue
<script setup lang="ts">
import type { ProgressRootProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { ProgressIndicator, ProgressRoot } from 'reka-ui'
import { computed } from 'vue'
import { cn } from '@/lib/cn'
import { progressShares } from './progressShares'

const props = withDefaults(
  defineProps<
    ProgressRootProps & {
      unknown?: number
      class?: HTMLAttributes['class']
    }
  >(),
  {
    modelValue: 0,
    max: 100,
    unknown: 0,
  },
)

const delegatedProps = reactiveOmit(props, 'class', 'unknown')

const shares = computed(() =>
  progressShares(props.modelValue ?? 0, props.unknown, props.max),
)
// The widths are computed, the vocabulary isn't: CSS variables bound here, consumed by
// w-(--progress-value) and w-(--progress-unknown) in the classes.
const shareVariables = computed(() => ({
  '--progress-value': `${shares.value.value}%`,
  '--progress-unknown': `${shares.value.unknown}%`,
}))
</script>

<template>
  <ProgressRoot
    data-slot="progress"
    v-bind="delegatedProps"
    :style="shareVariables"
    :class="
      cn(
        'flex h-3.5 w-full overflow-hidden border border-input bg-data',
        props.class,
      )
    "
  >
    <ProgressIndicator
      data-slot="progress-indicator"
      class="h-full w-(--progress-value) bg-primary"
    />
    <div
      data-slot="progress-unknown"
      class="h-full w-(--progress-unknown) hatch-unknown"
    />
  </ProgressRoot>
</template>
```

`ui/src/components/ui/progress/index.ts`:
```ts
export { default as Progress } from './Progress.vue'
export { progressShares } from './progressShares'
export type { ProgressShares } from './progressShares'
```

- [ ] **Step 6: Kit section**

`ui/src/kit/sections/ProgressSection.vue`:
```vue
<script setup lang="ts">
import { Progress } from '@/components/ui/progress'
import KitSection from '../KitSection.vue'
</script>

<template>
  <KitSection title="Progress">
    <div class="flex items-baseline justify-between">
      <span class="text-caption text-foreground-soft">Marchi almeno iniziati</span>
      <span class="text-control">166 / 368 leggibili</span>
    </div>
    <Progress :model-value="166" :unknown="40" :max="408" />
    <p class="text-label text-muted-foreground">
      40 celle su 408 non sono leggibili · nessuna percentuale totale
    </p>
  </KitSection>
</template>
```

In `ui/src/kit/KitPage.vue` add the import and `<ProgressSection />` after `<TableSection />`.

- [ ] **Step 7: Checks, build, visual check**

Run: `pnpm --filter ui format && pnpm typecheck && pnpm lint && pnpm format:check && pnpm scan && pnpm ui:test && pnpm --filter ui build`
Visual (`#kit`): a red segment of about 41%, then a striped segment of about 10%, then the empty data surface.

- [ ] **Step 8: Commit**

```bash
git add ui/src/components/ui/progress ui/src/kit
git commit -m "feat(ui): Progress, with a segment for what can't be read" -m "The bar shows what is done and, beside it, what the save doesn't let us read, on the unknown hatch: never folded into done or not done. progressShares keeps both inside max and returns zeros instead of NaN for an empty measure; the widths reach the template through CSS variables."
```

---

### Task 17: The verification page uses the primitives

Spec Decision 12: `App.vue`'s "primitives don't exist yet" exemption stops being true.

**Files:**
- Modify: `ui/src/App.vue`, `ui/scripts/scan-conventions.mjs`

**Interfaces:**
- Consumes: `Button`, `ButtonVariant`, `Input`, `Label`.

- [ ] **Step 1: Remove the exemption first, and watch the scanner fail**

In `ui/scripts/scan-conventions.mjs`, delete this entry from `EXEMPTIONS`:

```js
  {
    file: 'src/App.vue',
    check: 'raw primitive <button>/<input>',
    reason:
      "declared verification page, to be replaced by the design system: the primitives don't exist yet",
  },
```

and replace the remaining `App.vue` entry's reason

```js
    reason: 'same verification page: i18n arrives with the real frontend',
```

with

```js
    reason:
      'declared verification page, replaced by the shell in cycle 3 of the design system: its text is deliberately left untranslated',
```

Run: `pnpm scan`
Expected: FAIL — `src\App.vue: raw primitive <button>/<input>`.

- [ ] **Step 2: Imports and model type**

In `ui/src/App.vue`, after `import WikiBlocks from './components/WikiBlocks.vue'` add:

```ts
import { Button, ButtonVariant } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
```

Replace `const wikiId = ref('664')` with `const wikiId = ref<string | number>('664')` (`Input`'s model is `string | number`).

- [ ] **Step 3: The candidate buttons**

Replace

```vue
          <button class="underline" @click="choose(c.id)">
            {{ c.prefix }} slot {{ c.slot }} — {{ c.sizeBytes }} bytes
            <span v-if="c.suggested">(suggested)</span>
          </button>
```

with

```vue
          <Button :variant="ButtonVariant.Link" @click="choose(c.id)">
            {{ c.prefix }} slot {{ c.slot }} — {{ c.sizeBytes }} bytes
            <span v-if="c.suggested">(suggested)</span>
          </Button>
```

- [ ] **Step 4: The item id field**

Replace the whole `<label class="flex gap-2"> … </label>` block that contains `v-model="wikiId"` (after Task 3 its input carries `class="border border-input bg-data"`; Prettier may have wrapped it) with:

```vue
      <Label class="w-fit">
        item id
        <Input
          v-model="wikiId"
          class="w-40"
          @keyup.enter="
            Number.isFinite(Number(wikiId)) &&
            loadWiki({ kind: 'item', id: Number(wikiId) })
          "
        />
      </Label>
```

- [ ] **Step 5: Scanner and checks**

Run: `pnpm --filter ui format && pnpm scan`
Expected: `0 violations, 2 declared exemptions`.
Run: `pnpm typecheck && pnpm lint && pnpm format:check && pnpm ui:test && pnpm --filter ui build`

- [ ] **Step 6: Visual check**

`pnpm ui:dev`, open `http://localhost:1420/` (no hash). In the browser the Tauri commands fail, so the page shows the error line in the destructive colour and little else; that's expected outside Tauri. With `pnpm dev` (Tauri) the candidate list renders as cream underlined links and the item id field as a 34px input; Enter on a number loads that wiki entry.

- [ ] **Step 7: Commit**

```bash
git add ui/src/App.vue ui/scripts/scan-conventions.mjs
git commit -m "refactor(ui): the verification page uses Button, Input and Label" -m "Its exemption said the primitives didn't exist yet; they do now, so the exemption goes and the raw elements with it. The page's untranslated-text exemption stays, reworded: the shell replaces the page in cycle 3."
```

---

### Task 18: Documents, and the whole suite

**Files:**
- Modify: `docs/frontend-conventions.md`, `CLAUDE.md`, `docs/STATUS.md`, `docs/BACKLOG.md`, and `README.md` if it enumerates the checks.

- [ ] **Step 1: `docs/frontend-conventions.md` — structure**

Replace the code block under "## Structure of `ui/`":

```
ui/
  src/
    components/
      ui/          shadcn-vue primitives: they live in the repo and get modified
      <domain>/    app components, named for WHAT THEY ARE
    composables/
    stores/        Pinia, setup syntax
    lib/
      ipc/         typed wrappers around Tauri commands — the only place with invoke()
      constants/   magic strings: command names, storage keys
    i18n/          it, en
    assets/
    App.vue
    main.ts
```

with:

```
ui/
  components.json  shadcn-vue registry settings, written by hand (never `init`)
  src/
    components/
      ui/          shadcn-vue primitives (reka-vega), dressed: they live in the repo
      <domain>/    app components, named for WHAT THEY ARE
    composables/
    stores/        Pinia, setup syntax
    kit/           development-only Kit page: every primitive in every state (`#kit`)
    lib/
      ipc/         typed wrappers around Tauri commands — the only place with invoke()
      constants/   magic strings: command names, dev routes, key names, placement
      design/      themeKeys: the token names cn() reads from the theme CSS
      cn.ts        class merging that knows our tokens
    i18n/          it (the schema), en, locale, useMessages()
    assets/
      main.css     imports only
      theme/       one file per token family
      base.css     document defaults, focus ring, scrollbar, reduced motion
      utilities.css
      fonts/
    App.vue
    main.ts
```

- [ ] **Step 2: `docs/frontend-conventions.md` — tokens**

Replace

```
As long as the tokens fit on half a screen, they stay in `main.css`. The split happens
when it helps find them, not before.
```

with

```
The split happened with the design system (2026-09-10): `theme/colors.css`,
`typography.css`, `spacing.css`, `radius.css`, `shadow.css`, `opacity.css`, `motion.css`.
The values and the reasoning behind each are in
`docs/superpowers/specs/2026-09-10-design-system-foundations-design.md`.
```

Replace

```
Dark mode in v4 is no longer enabled with `darkMode: 'class'` in a config file: it
requires an explicit variant in the CSS (`@custom-variant dark …`). The dark theme is the
app's true default, not a variant: tokens are defined for dark and adapted for light.
```

with

```
**One theme, the dark one.** Values live directly in `@theme`; there is no `.dark` class and
no `@custom-variant dark`, and a `dark:` class is a scanner violation — without the custom
variant, Tailwind's built-in `dark:` follows `prefers-color-scheme`, so a leftover would
switch on with the Windows setting. A light theme later means moving the values of
`theme/colors.css` to selectors and mapping them with `@theme inline`; no component class
changes.

**The default scales are off.** Every namespace we define is reset first
(`--color-*: initial`, `--text-*`, `--font-*`, `--font-weight-*`, `--radius-*`, `--shadow-*`,
`--ease-*`, `--animate-*`), so `bg-red-500`, `text-sm`, `rounded-md`, `font-bold`,
`ease-in-out` and `animate-pulse` generate nothing. The static utilities survive
(`bg-transparent`, `text-current`, `rounded-full`). Spacing keeps the default grid. The trap
this creates is the old one in a new place: a class that doesn't exist emits nothing, so
check the Kit page, not only the typecheck.

**Motion runs on `steps()`.** `duration-tap|panel|sheet|loop` with `ease-tap|panel|sheet|frame`,
and `animate-*` tokens for entrances; the default transition is `0ms` on `steps(1)`, so hover
and active never lag. No exit animations. `prefers-reduced-motion` collapses everything to
0ms in `base.css`.
```

- [ ] **Step 3: `docs/frontend-conventions.md` — icons, primitives, i18n, tests, TypeScript**

After the paragraph ending `a disproportionate stroke.` in "## Icons", add:

```

**Determination has no `→ ← ↑ ↓ ⏎ ⌘ ✓`** (measured on the font file). Written in source they
fall back to whatever system font the machine has, so they are Lucide icons
(`ArrowRightIcon`, `ArrowUpIcon`, `ArrowDownIcon`, `CornerDownLeftIcon`, `CheckIcon`), inside a
`Kbd` for keys; the scanner rejects the glyphs. The font has one weight: emphasis is colour
(`text-foreground` against `text-foreground-soft`), never `font-bold`.
```

After `Practical rule: if you're writing \`<button class="… hover:bg-…">\`, stop and look for the
primitive.` add:

```

### How a primitive is written

- **From the registry, already dressed.** shadcn-vue 2.8.2, style `reka-vega`. Never
  `shadcn-vue init` (it rewrites `main.css`); `pnpm dlx shadcn-vue@2.8.2 add <name> --view`
  shows the registry version, `--diff` compares it with ours.
- **Variants in `variants.ts`**, as `as const` objects keying the `cva` config; `index.ts`
  re-exports them by name. Not in `index.ts`: a component that uses a constant as a prop
  default would read it before `index.ts` has initialised it.
- **State styling with `data-[state=…]`**: Reka sets `data-state="open"`, `"checked"`,
  `"active"`, `"on"` and `data-highlighted`; the registry's `data-open:` classes need a
  stylesheet we don't import.
- **No `tw-animate-css`, no `opacity-50`, no `outline-none` on focusable elements** (the ring
  comes from `base.css`), no `shadow-*`, no radius except `rounded-input`, `rounded-cell` and
  `rounded-full`.
- **`cn()` from `@/lib/cn`**: it knows our token names. Plain `twMerge` would read `text-body`
  as a colour and drop it next to `text-foreground`.
- Shared state between parts goes through a typed `InjectionKey`, never a string key.
```

Replace

```
- `useI18n()` must also be called in the root component, otherwise children emit the
  "Not found parent scope" warning.
```

with

```
- **Components call `useMessages()`** from `@/i18n`, never `useI18n()` directly: vue-i18n's
  own `t()` accepts any string, while `useMessages().t` only takes a key that exists in the
  Italian schema (`i18n/messages/it.ts`). `en.ts` is typed against that schema, so a missing
  English key is a compile error. `useMessages()` uses the global scope: no component needs a
  local instance.
```

After the paragraph ending `not where there's a\ngrid.` in "## Tests", add:

```

**Vitest** (`pnpm ui:test`, part of `scripts/check`) runs the frontend's logic: pure functions
beside their module (`*.test.ts`). Type-level guarantees are probes checked by
`pnpm typecheck` (`i18n/messageKey.typecheck.ts`). Presentation is checked on the Kit page
(`pnpm ui:dev`, then `#kit`). Vitest doesn't load CSS unless `test.css.include` matches it:
the theme files are listed there because `cn()` reads them.
```

In "## TypeScript and Vue", after `- **TypeScript strict**, always.` add:

```
- **`pnpm typecheck` is `vue-tsc --build --force`.** `ui/tsconfig.json` is a solution file;
  `vue-tsc --noEmit` on it checks no file at all, and did so until 2026-09-10.
```

- [ ] **Step 4: `CLAUDE.md`**

Replace

```
  **Target stack, not today's**: `ui/` currently has only Vue, Vite, Tailwind and
  `@tauri-apps/api` installed. The rest arrives with the design system, on purpose: adding
  it earlier would mean guessing the tokens.
```

with

```
  **Target stack, not all of it today's**: since the design system's first cycle
  (2026-09-10) `ui/` has Vue, Vite, Tailwind, `@tauri-apps/api`, shadcn-vue on Reka UI,
  Lucide, vue-i18n and Vitest. Pinia, Vue Router and TanStack arrive with the screens.
```

Replace `` `lint`, `scan`, `format:check` are pass-throughs to `ui/`. `` with `` `lint`, `scan`, `format:check`, `ui:test` are pass-throughs to `ui/`. ``

Replace

```
4. **No raw `<button>` / `<input>`** — use the primitives in `ui/src/components/ui/`
   (folder still to be created, arrives with shadcn-vue), and extend them with a prop
   instead of styling by hand.
```

with

```
4. **No raw `<button>` / `<input>`** — use the primitives in `ui/src/components/ui/`, and
   extend them with a prop instead of styling by hand.
```

Replace `` `cargo test --workspace`, `pnpm typecheck`, `pnpm lint`, `pnpm format:check`, `pnpm scan`. `` with `` `cargo test --workspace`, `pnpm typecheck`, `pnpm ui:test`, `pnpm lint`, `pnpm format:check`, `pnpm scan`. `` (the sentence may wrap across lines: edit the wrapped text, keep the wrapping).

Replace `- Frontend: no test runner installed. Test-first rules apply once there's logic.` with

```
- Frontend: Vitest (`pnpm ui:test`) for the logic in `ui/`, test-first like the Rust side;
  presentation is checked on the development-only Kit page (`pnpm ui:dev`, `#kit`).
```

- [ ] **Step 5: `docs/STATUS.md`**

Replace `**Last update:** 2026-09-09` with `**Last update:** 2026-09-10`.

Replace

```
- [ ] shadcn-vue, Reka UI, Pinia, vue-i18n, TanStack — deliberately out of the first step:
      they arrive with the design system, and getting ahead of it would mean guessing the
      tokens
- [ ] Design system (in progress on Claude Design, outside this repo)
```

with

```
- [x] shadcn-vue, Reka UI, vue-i18n — arrived with the design system's first cycle
      (2026-09-10); Pinia, Vue Router and TanStack arrive with the screens
- [ ] Design system — the Claude Design export arrived on 2026-09-10, built in three cycles:
      - [x] **1. Foundations and primitives** (2026-09-10) — tokens, font, motion, i18n,
            `cn()`, 22 primitives on a development-only Kit page. Spec
            `docs/superpowers/specs/2026-09-10-design-system-foundations-design.md`, plan
            `docs/superpowers/plans/2026-09-10-design-system-foundations.md`
      - [ ] 2. App components — tab strip, navbar, section sidebar, KPI tile, matrix cell,
            wiki inline tokens, data states
      - [ ] 3. Screens — shell, Pinia, Vue Router, TanStack
```

Directly under `## Session log` (before `### 2026-09-09 (last)`), insert:

```

### 2026-09-10 — the design system's first cycle, and a typecheck that checked nothing

The Claude Design export arrived as a zip of six pages. Read against the brief, it
contradicted itself in thirteen places — two light palettes, three button heights, two
encodings of the matrix cell, a "multiplayer" meaning for the one bit the brief calls
unconfirmed, gold for both "unlockable now" and every heading — and the spec resolves each
and hands the list back to design. One theme, the dark one; Tailwind's default scales reset,
so an off-system class generates nothing; motion on `steps()`; Determination only.

- [x] **`pnpm typecheck` checked no file.** `ui/tsconfig.json` is a solution file, and
      `vue-tsc --noEmit` on it exits 0 with a deliberate type error. Now
      `vue-tsc --build --force`; the existing code had 0 errors in build mode, so nothing
      had been hiding — but nothing would have been caught either.
- [x] **Found while planning, each by making the tool answer**: `shadcn-vue init` rewrites
      `main.css` (so `components.json` is hand-written); the registry's `data-open:` classes
      don't match Reka's `data-state`; vue-i18n's `t()` accepts any string (so
      `useMessages()` narrows the key type); Vitest hands a `?raw` CSS import an empty
      string unless `test.css.include` lists it; unconfigured tailwind-merge drops
      `text-body` beside `text-foreground`; the scanner's visible-text heuristic ended a tag
      at the `>` inside `has-[>svg]:`.
- [x] 22 primitives, dressed, on the Kit page (`pnpm ui:dev`, `#kit`); `App.vue` uses
      them and loses its exemption.
- [ ] Cycle 2 — app components.
```

- [ ] **Step 6: `docs/BACKLOG.md`**

Append at the end of the file:

```

---

## B10 — The design export pack: what the design tool had to measure by hand (implementation, `design-export`)

Logged on 2026-09-10, from `design-export.md` inside the Claude Design export: the places
where the pack `pnpm design:export` produces forced the design tool to measure, crop or
guess. They aren't bugs in the app, but cycle 2's matrix cell leans on the first four, and
every later export repeats the work until the pack says what it knows.

1. **Sprites aren't trimmed**: `completion_widget/paper_00.png` is 96×96 with the drawing at
   `x 0–84, y 3–82`, so centring the frame centres empty pixels. Export trimmed frames, or a
   `trim: [x, y, w, h]` and `pivot: [x, y]` per frame in `sheets.json`.
2. **`sheets.json` has no content rectangle or pivot** (same fix).
3. **Delirium's mark isn't among the marks**: it lives in `onlinelobby/background_completion_delirium_*`,
   under another naming.
4. **The `_00`/`_02` tier is guessed from layer names**: an explicit `mark`, `tier` field.
5. **No usable card or card back**: `ui_cardfronts/outline.png` is a 16×24 outline.
6. **Papers come paired in one image** (`pausescreen_mystuff/paper.png`, `deedsmenu/paper.png`,
   `scoremenu/smallpaper_00.png`): one file per sheet, or declared 9-slice cuts.
7. **`sheet` paths contain spaces** (`gfx/ui/seed paper.png`).
8. **The co-op sheet's holes are undeclared**: a `characters.json` mapping character id → cell,
   with an explicit `null`.
9. **40 completion cells are unreadable** (already a real data gap, modelled as `unknown`).
10. **`unlock.json` derives the target from text**: 30 of 72 sampled rows match nothing.
11. **Boss portraits are indexed by sheet position**, the entity key only inside the `source`
    file name: a `target: { kind: 'entity', id, variant }` field.
12. **A typed target doesn't imply an image**: the list of holes of `target_sprite`, not only
    its aggregate coverage.
13. **`unlock.illustrated.json` inlines icons as base64**: references to paths (the C2 flaw).
```

- [ ] **Step 7: `README.md`**

Run: `grep -n "pnpm typecheck\|no test runner\|format:check" README.md`
Where README enumerates the commands `scripts/check` runs, add `pnpm ui:test` after `pnpm typecheck`. Where it says the frontend has no test runner, replace that with the Vitest sentence of Step 4. If neither appears, leave README untouched.

- [ ] **Step 8: The whole suite**

Run: `pnpm check`
Expected: ends with `all green`. Read the summary's skip count: skips on real data are expected on a machine without `samples/`; a new failure is not.

- [ ] **Step 9: Visual pass of the whole Kit page**

Extract the export outside the repo (`unzip -q "IsaacDome design system.zip" -d "$TEMP/isaacdome-export"`), open `Shadcn Kit.dc.html` from there in the browser, and `pnpm ui:dev` → `http://localhost:1420/#kit` beside it. Expected differences are the ones the spec decided (gold "Ora", cream alert icon, foreground select tick, 34px buttons, no Calendar/Chart/Stepper…). Anything else that differs is either a dressing bug to fix in the primitive, or a new contradiction to add to the spec's "Handed back to design" list. Then close the tab and delete the extracted folder.

- [ ] **Step 10: Commit**

```bash
git add docs/frontend-conventions.md CLAUDE.md docs/STATUS.md docs/BACKLOG.md
git commit -m "docs: the design system's first cycle in the conventions, status and backlog" -m "One theme, the reset scales, motion on steps, the font's missing glyphs, how a primitive is written, useMessages, Vitest and the Kit page. STATUS records the typecheck that checked nothing; B10 logs the thirteen problems of the design export pack."
```

(Add `README.md` to `git add` if Step 7 changed it.)

