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
const DEV_ONLY_DIRS = [join('src', 'kit'), join('src', 'verify')]
const WINDOW_DIR = join('src', 'lib', 'window')
const SCREENS_DIR = join('src', 'screens')

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

// **A screen is what the router mounts**, not everything under `src/screens/` — that folder holds
// "one screen per route, **and the parts only it uses**" (the conventions), so `UnlockRow.vue` and
// `SaveCard.vue` live there and are not screens. Read from `routes.ts` rather than guessed from
// the path, so it cannot rot: a screen added to the router is a screen here the same minute.
// `WelcomeScreen.vue` is added by hand because it is mounted by `App.vue` as a takeover above the
// router (3.8), which is the one screen the router never names.
//
// What this cannot see, said rather than discovered: `WikiScreen.vue` delegates to four bodies
// that are the real roots, and they are not router-imported, so the shape is not *required* of
// them. They carry it anyway; nothing would notice if a fifth one did not.
const SCREEN_FILES = new Set(
  [
    ...readFileSync(join(SRC, 'router', 'routes.ts'), 'utf8').matchAll(
      /from '@\/screens\/([^']+)'/g,
    ),
  ]
    .map(([, path]) => join('src', 'screens', ...path.split('/')))
    .concat(join('src', 'screens', 'welcome', 'WelcomeScreen.vue')),
)
const isScreen = (file) => SCREEN_FILES.has(relative(ROOT, file))

// A screen's root: the first tag inside its `<template>`, and its class list. Both responsive
// rules below are facts about that one line and nothing deeper — a `max-w-*` on a card inside a
// screen is the card's business, and a scroll on a panel is the panel's (spec 3.13a §4).
const ROOT_TAG = /<template>\s*(?:<!--[\s\S]*?-->\s*)*<([a-zA-Z][\w-]*)([^>]*)>/
const rootClasses = (body) => {
  const template = body.match(TEMPLATE_BLOCK)
  if (!template) return ''
  const root = `<template>${template[1]}`.match(ROOT_TAG)
  const attrs = root ? (root[2] ?? '') : ''
  return attrs.match(/\bclass="([^"]*)"/)?.[1] ?? ''
}

// **A comment is not code** (`docs/BACKLOG.md` B59). Every rule below is a regex against the raw
// file, so a comment saying *"a screen never calls `invoke()`"* tripped the rule forbidding the
// call — found on 2026-09-16 while writing 3.7a. The comment was reworded, which is the reflex
// this pre-pass exists to stop training: **a rule that cannot tell a mention from a call will
// eventually refuse a correct explanation of itself.**
//
// Not a parser — these are regexes over `.ts` and `.vue`, and the cheap version is enough:
// `//…`, `/*…*/` and `<!--…-->` become spaces, keeping every newline so line numbers and offsets
// still mean what they meant.
//
// **Strings are deliberately left alone**, per the entry: a rule matching inside a string literal
// is usually matching a real thing, a class name or a route. They are tracked only so that a
// `//` inside `'https://…'` is not read as a comment.
//
// **A backslash takes the next character with it.** What that is for is an escaped quote:
// without it `'it\'s // not one'` closes at `\'`, the rest of the string becomes code, and the
// `//` inside it blanks the rest of a real line. It is applied outside strings too, which costs
// nothing and keeps the rule one sentence.
//
// What it is **not** for, written down because the first version of this comment claimed it and
// was wrong: `/^isaac:\/\/achievement\/(\d+)$/`, which is real and lives in
// `lib/ipc/fixtures/graphArt.ts`, needs no help at all — `\/\/` is backslash, slash, backslash,
// slash, so there are never two slashes adjacent and nothing ever looked like a comment. The
// fixture for it stayed, as a regression guard rather than as the reason.
//
// What it cannot do, said here rather than found later: an apostrophe in prose that is *not* in a
// comment — `Isaac's` as template text — opens a quote that never closes, and from there the file
// is read as it was before B59. That degrades to the old behaviour, never to a false accusation,
// and visible prose in a template is forbidden by another rule anyway.
const blankComments = (body) => {
  const out = []
  let quote = null
  let i = 0
  const blank = (from, to) => {
    for (let n = from; n < to; n += 1) out.push(body[n] === '\n' ? '\n' : ' ')
  }
  while (i < body.length) {
    const c = body[i]
    if (c === '\\') {
      out.push(c, body[i + 1] ?? '')
      i += 2
      continue
    }
    if (quote) {
      if (c === quote) quote = null
      out.push(c)
      i += 1
      continue
    }
    if (c === "'" || c === '"' || c === '`') {
      quote = c
      out.push(c)
      i += 1
      continue
    }
    if (c === '/' && body[i + 1] === '/') {
      const end = body.indexOf('\n', i)
      const to = end === -1 ? body.length : end
      blank(i, to)
      i = to
      continue
    }
    if (c === '/' && body[i + 1] === '*') {
      const end = body.indexOf('*/', i + 2)
      const to = end === -1 ? body.length : end + 2
      blank(i, to)
      i = to
      continue
    }
    if (body.startsWith('<!--', i)) {
      const end = body.indexOf('-->', i + 4)
      const to = end === -1 ? body.length : end + 3
      blank(i, to)
      i = to
      continue
    }
    out.push(c)
    i += 1
  }
  return out.join('')
}

const isUnder = (file, dir) => relative(ROOT, file).startsWith(dir)

// Exceptions are declared here, per file and per check, with a reason. An exception
// with no reason is an untracked violation; an empty list is the goal.
const EXEMPTIONS = [
  {
    file: 'src/screens/welcome/WelcomeScreen.vue',
    check: 'screen root is neither flowing nor filling',
    reason:
      'the welcome is a takeover above the router (3.8), drawn by App.vue outside <main>: there is no page box for it to fill, and it sizes itself against the window',
  },
  {
    file: 'src/screens/WikiScreen.vue',
    check: 'screen root is neither flowing nor filling',
    reason:
      'it has no root of its own: it picks one of four bodies from the query, and each of those carries the shape',
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
    // **The one rule that reads a comment on purpose**: the exemption *is* a comment
    // (`/* exception allowed: … */`), so blanking comments for this one would turn every
    // declared exemption into a violation. Nothing in `src/` has a `<style>` block today, so
    // that breakage would have been silent — which is why the flag is here and not discovered
    // the first time somebody needs the exemption.
    raw: true,
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
    // Three namespaces, not one: `window` was the whole story while there was one window.
    // A tab torn off into a window of its own brought `webviewWindow` (creating them) and
    // `event` (what they say to each other), and the rule that keeps all of it in one module
    // is worth nothing if it only knows the first name.
    name: 'window API outside src/lib/window/',
    test: (file, body) =>
      /@tauri-apps\/api\/(window|webviewWindow|event)/.test(body) &&
      !isUnder(file, WINDOW_DIR),
  },
  {
    // Spec 3.13a §2 and §9. The first half is the rejected approach: a media query measures the
    // window, which is not what the content has — the sidebar alone is 252px the window cannot
    // account for — and its `rem` cannot see the interface's scale at all.
    //
    // The second half is the hole the first one leaves: `containers.css` does not reset Tailwind's
    // own `--container-*` scale, so `@max-md/page:` is writable today and would be a fourth
    // threshold nobody declared, measured against a number nobody chose.
    //
    // **A Vue event binding has the same shape as a variant** — `@update:open="x"` — so the two
    // are told apart the only way that is honest here: a container variant names its container
    // with a `/`, and an event binding never does. That naming is §5's rule, which is what makes
    // this rule possible at all.
    name: 'media query variant, or a container size that is not ours',
    test: (_f, body) => {
      const media = /(^|[\s"'`])(?:[a-z0-9-]+:)*(?:sm|md|lg|xl|2xl):/m.test(
        body,
      )
      const ours = new Set([
        'compact',
        'regular',
        'wide',
        'tab-narrow',
        'sidebar-room',
      ])
      const named = [...body.matchAll(/@(?:max-)?([a-z0-9-]+)\/[a-z-]+:/g)]
      return media || named.some(([, size]) => !ours.has(size))
    },
  },
  {
    // Spec 3.13a §4. Two shapes and not twenty: a screen either flows and scrolls, or fills and
    // hands the height that is left to one region inside it. A root that is neither is a screen
    // whose height nobody decided — which fails nothing, and looks like a bug in the list inside
    // it rather than in the screen around it.
    name: 'screen root is neither flowing nor filling',
    test: (file, body) => {
      if (!isScreen(file)) return false
      const cls = rootClasses(body)
      const flowing = /\boverflow-y-auto\b/.test(cls)
      const filling = /\boverflow-hidden\b/.test(cls) && /\bmin-h-0\b/.test(cls)
      return !/\bh-full\b/.test(cls) || !(flowing || filling)
    },
  },
  {
    // Spec 3.13a §3: the cap left by decision, on every screen at once. This is what stops it
    // coming back by habit from the screen next door — which is exactly how all of them came to
    // carry the same one.
    name: 'width cap on a screen root',
    test: (file, body) =>
      file.endsWith('.vue') &&
      isUnder(file, SCREENS_DIR) &&
      /\bmax-w-/.test(rootClasses(body)),
  },
  {
    // Spec 3.13a §7 and §9. It does not prove the *right* columns fell — nothing in a script can.
    // It proves both edits were made: a narrow template with no hidden cell is a grid that dropped
    // a track while every cell stayed, which slides the rest into the wrong columns. The header
    // and the row live in two files, so each has to pass on its own.
    name: 'narrow grid template with no column hidden',
    test: (_f, body) =>
      /\bgrid-cols-[a-z-]+-narrow\b/.test(body) &&
      !/@max-compact\/page:hidden/.test(body),
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
      !DEV_ONLY_DIRS.some((dir) => isUnder(file, dir)) &&
      /\p{L}{2,}/u.test(visibleText(body)),
  },
  {
    // One theme. Without `@custom-variant dark`, Tailwind's built-in `dark:` compiles to
    // `prefers-color-scheme`, so a leftover class would switch on with the OS setting.
    // The variant chain in front of it is part of the class: `hover:dark:bg-x` is the same
    // leftover, and the first version of this rule could not see it.
    name: 'dark: variant in a one-theme app',
    test: (_f, body) => /(^|[\s"'`])(?:[a-z0-9-]+:)*dark:[a-z[*]/m.test(body),
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
    // Every one of these names a constant: `ButtonVariant`, `ProgressTone`, `AlertLive`, and
    // `Orientation`, `Align`, `Side` in `lib/constants/placement.ts`. Measured when the last
    // five were added (2026-09-18): no file outside `components/ui/` wrote any of them as a
    // literal, so this is a guard and not a cleanup.
    name: "literal variant on a primitive: use the component's constant",
    test: (file, body) =>
      file.endsWith('.vue') &&
      !isUnder(file, UI_DIR) &&
      /\s(variant|size|density|orientation|position|align|side|tone|live)="[a-z]/.test(
        body,
      ),
  },
  {
    // Cyan means focus, and it is the only thing that says where the keyboard is. A primitive
    // may take the outline off because it puts something in its place — Reka moves real DOM
    // focus through a listbox and draws a highlight instead, which is what the four in
    // `components/ui/` do. A screen has nothing to put in its place.
    name: 'outline-none outside a primitive',
    test: (file, body) =>
      /\boutline-none\b/.test(body) && !isUnder(file, UI_DIR),
  },
  {
    // `@theme` resets `--text-*`, `--font-weight-*`, `--radius-*` and `--shadow-*` to
    // `initial` (typography.css, radius.css, shadow.css), so these classes generate no CSS at
    // all: the element keeps whatever it inherited and nothing says otherwise. `rounded-full`
    // and `rounded-none` are not theme values and still work. Found two on the day it was
    // written, both `text-sm`, both inert since the scale was reset.
    name: 'class of a reset default scale: it generates nothing',
    test: (_f, body) =>
      /\b(text-(xs|sm|base|lg|[2-9]xl|xl)|font-(thin|extralight|light|normal|medium|semibold|bold|extrabold|black)|rounded-(xs|sm|md|lg|[2-4]xl|xl)|shadow-(2xs|xs|sm|md|lg|[2-4]xl|xl|inner))\b/.test(
        body,
      ),
  },
  {
    name: 'glyph missing from Determination: use an icon',
    test: (_f, body) => MISSING_GLYPHS.test(body),
  },
]

// The interface's size is the root's font size and every token is in rem (cycle 3.5c), so a
// token left in px stays its own size while the rest of the app moves — which fails nothing
// and looks like a bug in one component. Some values do keep px on purpose (a hairline, a
// radius, a sprite's whole multiple): the rule is that each one says why, on the spot.
const PX_TOKEN = /^\s*--[a-z0-9-]+\s*:\s*[^;]*\d+px/
const COMMENT = /(^\s*\/\*)|(^\s*\*)|(\*\/\s*$)/
const COMMENT_REACH = 5

const pxWithoutReason = (body) => {
  const lines = body.split('\n')
  let lastComment = -COMMENT_REACH - 1
  return lines.flatMap((line, index) => {
    if (COMMENT.test(line)) lastComment = index
    if (!PX_TOKEN.test(line)) return []
    const reasoned = index - lastComment <= COMMENT_REACH || line.includes('/*')
    return reasoned ? [] : [line.trim()]
  })
}

// One file against every rule. A rule sees the code unless it says it wants the raw text, and
// only one does.
const breaches = (file, raw) => {
  const code = blankComments(raw)
  return checks
    .filter((c) => c.test(file, c.raw ? raw : code) && !isExempt(file, c.name))
    .map((c) => c.name)
}

// **The scanner's own fixtures** (B59). They exist because the pre-pass above changes no answer
// on the repository as it stands — it could not, since the only comment that ever tripped a rule
// was reworded the day it was found — so nothing else would say whether it works. Each is a file
// that never existed, and what it must and must not be accused of.
const FIXTURES = [
  {
    name: 'a comment naming invoke() is not a call',
    file: 'src/screens/Fixture.vue',
    body: '<script setup lang="ts">\n// A screen never calls invoke() directly: the wrappers in lib/ipc/ do.\nconst a = 1\n</script>\n',
    expect: [],
  },
  {
    name: 'a real invoke() is still a call',
    file: 'src/screens/Fixture.vue',
    body: '<script setup lang="ts">\nconst a = await invoke("thing")\n</script>\n',
    expect: ['invoke() outside src/lib/ipc/'],
  },
  {
    name: 'a block comment naming the window API is not an import',
    file: 'src/screens/Fixture.ts',
    body: '/*\n * Nothing outside lib/window/ imports @tauri-apps/api/event.\n */\nexport const a = 1\n',
    expect: [],
  },
  {
    name: 'an HTML comment naming a pixel class is not one',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <!-- never w-[48px]: every size is a token -->\n  <div class="w-tab-min" />\n</template>\n',
    expect: [],
  },
  {
    name: 'a pixel class in the markup is still one',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <div class="w-[48px]" />\n</template>\n',
    expect: ['arbitrary pixel value in a class'],
  },
  {
    // The blanker's own guard: read as a comment, the `//` would blank the rest of the line and
    // the class after it would go unseen.
    name: 'a // inside a string does not blank the rest of the line',
    file: 'src/screens/Fixture.ts',
    body: "export const a = { url: 'https://example.com/a//b', gap: 'w-[12px]' }\n",
    expect: ['arbitrary pixel value in a class'],
  },
  {
    // The backslash branch's real job: without it the string closes at `\'`, what follows
    // becomes code, and the `//` inside the string blanks the rest of the line.
    name: 'an escaped quote does not end the string it is in',
    file: 'src/screens/Fixture.ts',
    body: "export const a = 'it\\'s // not a comment', gap = 'w-[12px]'\n",
    expect: ['arbitrary pixel value in a class'],
  },
  {
    // A regression guard and not a reason: `\/\/` never puts two slashes side by side, so this
    // passes even with the backslash branch removed. Pinned so a "simpler" blanker that strips
    // backslashes first is caught. `/^isaac:\/\/item\/…$/` is real, in lib/ipc/fixtures/.
    name: 'a regex full of escaped slashes is not a comment',
    file: 'src/screens/Fixture.ts',
    body: 'export const L = /^isaac:\\/\\/item\\/(\\d+)$/, gap = "w-[12px]"\n',
    expect: ['arbitrary pixel value in a class'],
  },
  {
    name: 'a stacked dark: variant is still a dark: variant',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <div class="hover:dark:bg-data" />\n</template>\n',
    expect: ['dark: variant in a one-theme app'],
  },
  {
    // The guard on the chain: `dark` has to be the whole variant, not the tail of a word.
    name: 'a class ending in dark is not the dark: variant',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <div class="bg-sky-dark" />\n</template>\n',
    expect: [],
  },
  {
    name: 'a literal side on a primitive is caught like a literal variant',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <TooltipContent side="top" />\n</template>\n',
    expect: ["literal variant on a primitive: use the component's constant"],
  },
  {
    name: 'outline-none in a screen has nothing to replace the ring',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <div class="outline-none" />\n</template>\n',
    expect: ['outline-none outside a primitive'],
  },
  {
    name: 'outline-none in a primitive is allowed: a highlight replaces the ring',
    file: 'src/components/ui/command/Fixture.vue',
    body: '<template>\n  <div class="outline-none" />\n</template>\n',
    expect: [],
  },
  {
    name: 'a class of a reset scale generates nothing and is caught',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <p class="text-sm font-bold rounded-md" />\n</template>\n',
    expect: ['class of a reset default scale: it generates nothing'],
  },
  {
    name: 'rounded-full is not a theme value and still generates',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <span class="rounded-full rounded-none" />\n</template>\n',
    expect: [],
  },
  {
    // The guard on that rule's own names: our tokens share the namespaces and must not be
    // caught by it. `text-body` is ours, `text-base` is the one that is gone.
    name: 'a token of ours in the same namespace is not a reset scale',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <p class="text-body rounded-input" />\n</template>\n',
    expect: [],
  },
  {
    name: 'a declared style exemption is still read from its comment',
    file: 'src/screens/Fixture.vue',
    body: '<style>\n/* exception allowed: -webkit-app-region */\n.a { -webkit-app-region: drag; }\n</style>\n',
    expect: [],
  },
  {
    name: 'a style block with no exemption is still caught',
    file: 'src/screens/Fixture.vue',
    body: '<style>\n.a { color: red; }\n</style>\n',
    expect: ['style block with no declared exemption'],
  },
  // The four below name a **real** screen, because the rule's scope is read from the router and a
  // file the router never heard of is not a screen (see `SCREEN_FILES`). The body is still
  // invented; only the path has to be one the rule applies to.
  {
    name: 'a flowing screen root is allowed',
    file: 'src/screens/GoalsScreen.vue',
    body: '<template>\n  <div class="flex h-full flex-col gap-4 overflow-y-auto pt-5 pb-15" />\n</template>\n',
    expect: [],
  },
  {
    name: 'a filling screen root is allowed',
    file: 'src/screens/GoalsScreen.vue',
    body: '<template>\n  <div class="flex h-full min-h-0 flex-col gap-4 overflow-hidden pt-5 pb-5" />\n</template>\n',
    expect: [],
  },
  {
    name: 'a screen root with no shape at all is caught',
    file: 'src/screens/GoalsScreen.vue',
    body: '<template>\n  <div class="flex flex-col gap-4" />\n</template>\n',
    expect: ['screen root is neither flowing nor filling'],
  },
  {
    // A part that lives under `screens/` is not a screen and owes no shape. This is the guard on
    // the scope: the first version of the rule accused twenty-seven of them.
    name: 'a part under screens/ is not a screen',
    file: 'src/screens/unlock/UnlockRow.vue',
    body: '<template>\n  <span class="px-2" />\n</template>\n',
    expect: [],
  },
  {
    name: 'a media query variant is caught',
    file: 'src/screens/GoalsScreen.vue',
    body: '<template>\n  <div class="grid grid-cols-2 sm:grid-cols-4" />\n</template>\n',
    expect: [
      'media query variant, or a container size that is not ours',
      'screen root is neither flowing nor filling',
    ],
  },
  {
    name: 'one of our container variants is allowed',
    file: 'src/screens/unlock/UnlockRow.vue',
    body: '<template>\n  <span class="flex flex-col @wide/page:flex-row @max-compact/page:hidden" />\n</template>\n',
    expect: [],
  },
  {
    name: 'a container size that is not ours is caught',
    file: 'src/screens/unlock/UnlockRow.vue',
    body: '<template>\n  <span class="@max-md/page:hidden" />\n</template>\n',
    expect: ['media query variant, or a container size that is not ours'],
  },
  {
    // The guard on the half that could not tell them apart: an event binding wears the same shape
    // as a variant, and only the container's name separates them.
    name: 'a Vue event binding is not a container variant',
    file: 'src/screens/unlock/UnlockRow.vue',
    body: '<template>\n  <Thing @update:open="go" @update:model-value="go" />\n</template>\n',
    expect: [],
  },
  {
    name: 'a narrow template with its hidden cells is allowed',
    file: 'src/screens/unlock/UnlockRow.vue',
    body: '<template>\n  <div class="grid grid-cols-unlock @max-compact/page:grid-cols-unlock-narrow">\n    <span class="@max-compact/page:hidden" />\n  </div>\n</template>\n',
    expect: [],
  },
  {
    name: 'a narrow template with no hidden cell is half the work',
    file: 'src/screens/unlock/UnlockRow.vue',
    body: '<template>\n  <div class="grid grid-cols-unlock @max-compact/page:grid-cols-unlock-narrow" />\n</template>\n',
    expect: ['narrow grid template with no column hidden'],
  },
  {
    name: 'a width cap on a screen root is caught',
    file: 'src/screens/GoalsScreen.vue',
    body: '<template>\n  <div class="flex h-full max-w-250 flex-col overflow-y-auto" />\n</template>\n',
    expect: ['width cap on a screen root'],
  },
  {
    // The guard on the rule's own reach: the root is the first tag and nothing deeper, so a cap
    // on a card inside a screen is the card's business and must not be accused.
    name: 'a width cap below the root is not the root',
    file: 'src/screens/Fixture.vue',
    body: '<template>\n  <div class="flex h-full flex-col overflow-y-auto">\n    <p class="max-w-80" />\n  </div>\n</template>\n',
    expect: [],
  },
]

const fixtureFailures = FIXTURES.flatMap((f) => {
  const got = breaches(join(ROOT, ...f.file.split('/')), f.body).sort()
  const want = [...f.expect].sort()
  return got.join(' | ') === want.join(' | ')
    ? []
    : [
        `fixture "${f.name}": expected [${want.join(', ')}], got [${got.join(', ')}]`,
      ]
})

const violations = walk(SRC)
  .filter((f) => /\.(vue|ts)$/.test(f))
  .flatMap((file) =>
    breaches(file, readFileSync(file, 'utf8')).map(
      (name) => `${relative(ROOT, file)}: ${name}`,
    ),
  )
  .concat(
    walk(join(SRC, 'assets'))
      .filter((f) => f.endsWith('.css'))
      .flatMap((file) =>
        pxWithoutReason(readFileSync(file, 'utf8')).map(
          (line) =>
            `${relative(ROOT, file)}: px token with no reason beside it — ${line}`,
        ),
      ),
  )

fixtureFailures.forEach((f) => console.error(f))
violations.forEach((v) => console.error(v))
// The fixture count is printed even when they all pass, because a self-test nobody can see the
// size of is a self-test that can quietly go to zero — which is B63's lesson one file over.
console.log(
  `${violations.length} violations, ${EXEMPTIONS.length} declared exemptions, ${FIXTURES.length} fixtures`,
)
process.exit(violations.length + fixtureFailures.length === 0 ? 0 : 1)
