#!/usr/bin/env node
// Fails if the repository tracks a picture, a sound or a game archive.
//
// **Why this is a gate and not a report.** CLAUDE.md's third promise is that no game asset
// ships in the package: images are extracted from the user's own copy at runtime. For two
// weeks that promise was false — `design-export/isaacdome-design-pack/images/` carried 6065
// PNGs cut out of a commercial game, 43 MB of them, on a public repository. They left the
// working tree on 2026-09-15 and the history on 2026-09-20, and what made it possible to
// miss for that long is that nothing looked.
//
// A fixture, a screenshot pasted in to illustrate a card, a sprite committed "just to try
// it": each arrives one file at a time and none of them looks like a decision. This is the
// thing that makes it one.
//
// The allowlist is the app's own icons, which we drew.

import { execFileSync } from 'node:child_process'

// Raster pictures, sound, and the game's own formats. SVG is deliberately not here: the game
// ships none, and ours (`ui/public/favicon.svg`, `ui/src/assets/logo.svg`) are drawn as text.
const BANNED = /\.(png|jpe?g|gif|webp|bmp|ico|tiff?|anm2|wav|ogg|mp3|a)$/i

// Each entry says what it is and why it may be here, the way scan-conventions.mjs does.
const ALLOWED = [
  { path: 'crates/app/icons/', why: "the app's own icon, drawn for it" },
]

const tracked = execFileSync('git', ['ls-files', '-z'], { encoding: 'utf8' })
  .split('\0')
  .filter(Boolean)

const offenders = tracked.filter(
  (file) =>
    BANNED.test(file) && !ALLOWED.some((entry) => file.startsWith(entry.path)),
)

if (offenders.length === 0) {
  console.log(`no game asset tracked (${tracked.length} files checked)`)
  process.exit(0)
}

console.error(
  `${offenders.length} tracked file(s) look like assets, which the repository does not ship:`,
)
for (const file of offenders) console.error(`  ${file}`)
console.error(
  '\nImages come from the user’s own copy of the game at runtime (CLAUDE.md, constraint 3).',
)
console.error(
  'If one of these genuinely belongs to us, add it to ALLOWED in this script with a reason.',
)
process.exit(1)
