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
const EXEMPTIONS = []

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
