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
