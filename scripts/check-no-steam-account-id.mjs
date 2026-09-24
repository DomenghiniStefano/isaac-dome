#!/usr/bin/env node
// Fails if a tracked file carries a Steam account id that is not one of the declared fakes.
//
// **Why this is a gate and not a report.** A save under Steam Cloud lives in
// `userdata\<account id>\250900\remote`, and the account id is the owner's: added to
// 76561197960265728 it is the SteamID64, the address of a public Steam profile. It is not a
// credential, it is a link between two identities — the same reason CLAUDE.md forbids a
// `PathBuf` under `userdata\` from crossing the IPC. The app kept that rule; the number came in
// through a hand-written probe path and a test fixture instead, stayed in the public repository
// for three weeks, and left the history on 2026-09-24 (card #83). Nothing looked, so this does.
//
// Two shapes are checked: a `userdata/<digits>` path segment (either slash), and an
// `account_id: "<digits>"` literal. Any id in them must be a fake declared below. The real one
// is never named here: a gate that spelled it out would put it back in the tree.

import { execFileSync } from 'node:child_process'

// Each entry says where it is used, the way check-no-game-assets.mjs keeps its allowlist.
const FAKES = [
  { id: '1', why: '`ipc` profile-id tests and their archived plan, `userdata\\1\\`' },
  { id: '111', why: '`discovery` scan tests: the account with two slots' },
  { id: '222', why: '`discovery` scan tests: the account with the Repentance save' },
  { id: '333', why: '`discovery` scan tests: the account under another AppID' },
  { id: '123456789', why: 'the stand-in for a real account id in `ipc` and its plan' },
]

const PATTERNS = [/userdata[\\/]+(\d+)/g, /account_?id:\s*"(\d+)"/gi]

// `git grep` exits 1 when nothing matches, which is the answer we want and not an error.
function trackedLines() {
  try {
    return execFileSync(
      'git',
      ['grep', '-n', '-I', '-E', '-i', String.raw`userdata[\\/]+[0-9]|account_?id: *"[0-9]`],
      { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] },
    )
  } catch (error) {
    if (error.status === 1) return ''
    throw error
  }
}

const grep = trackedLines()

const allowed = new Set(FAKES.map((f) => f.id))
const offenders = []
for (const line of grep.split('\n').filter(Boolean)) {
  for (const pattern of PATTERNS) {
    for (const match of line.matchAll(pattern)) {
      if (!allowed.has(match[1])) offenders.push(line.slice(0, line.indexOf(':', line.indexOf(':') + 1)))
    }
  }
}

if (offenders.length === 0) {
  console.log('no Steam account id tracked beyond the declared fakes')
  process.exit(0)
}

console.error(
  `${offenders.length} place(s) carry a Steam account id that is not a declared fake:`,
)
for (const where of offenders) console.error(`  ${where}`)
console.error(
  '\nA real account id links the repository to a Steam profile (card #83). Use a fake one;',
)
console.error('if a new fake is genuinely needed, add it to FAKES in this script with a reason.')
process.exit(1)
