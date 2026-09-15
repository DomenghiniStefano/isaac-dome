// Every file path the living documents name, against the files that exist.
//
// **A report, never a gate.** It always exits 0. Two things it cannot do, and both are the
// reason it reports instead of failing: it cannot tell a name written as *history* from one
// written as a *promise* — `docs/STATO.md` → `docs/STATUS.md` in a rename record is correct,
// and the same string in an open backlog entry is a defect — and it cannot read a condition,
// so a spec naming a file "only if the spike says so" looks identical to one that lied.
//
// It exists because the same defect turned up in every document on 2026-09-15: a document
// names a file, a refactor renames the file, and nothing notices. `frontend-conventions` had
// been citing a deleted test as the thing that pins a row height; three open backlog entries
// pointed at files N8 had renamed under them. The precedent is `ui/scripts/scan-conventions.mjs`,
// which exists for the same reason — no linter enforces what a document promises.
//
// **What it checks**: the seven documents that describe the project as it is, plus the specs,
// which this project calls the durable half. Not `plans/`, `reports/` or `docs/completed/`:
// those are the record of a day and say so, and a name inside them is history by construction.
// Splitting the journal out of `STATUS.md` on 2026-09-16 moved five exemptions' worth of
// references into it at once, and the GONE lines are what said so — which is the tool working,
// not a problem with it.
//
// **The signal is the delta.** Everything below in EXEMPTIONS was read on 2026-09-15 and found
// legitimate; anything else is printed as new. An exemption that no longer resolves to a
// missing file is printed too — a rename that fixed itself still means this list drifted.

import { readFileSync, readdirSync, existsSync } from 'fs'
import { execFileSync } from 'child_process'

// Known-legitimate unresolved references: `path` as the document writes it, and why it is not
// a defect. A name that stops appearing here has either been fixed or the document changed.
const EXEMPTIONS = [
  // A document recording a rename has to name the file that went away. Most of these left with
  // the journal on 2026-09-16; `progetto.html` is named by an open backlog entry as well.
  { path: 'docs/progetto.html', why: 'rewritten as docs/PROJECT.md; a backlog entry records it' },
  // Entries quoting their own earlier text, marked as such in the document.
  {
    path: 'ui/src/lib/graph/unlockFilter.ts',
    why: 'quoted under "the original entry, for the record"; became unlockFacets.ts with N8',
  },
  {
    path: 'ui/src/lib/collection/collectionFilter.ts',
    why: 'same entry; became collectionFacets.ts with N8',
  },
  { path: 'stores/graph.ts', why: 'three view stores became one stores/views.ts with N8' },
  { path: 'stores/completion.ts', why: 'same' },
  { path: 'stores/collection.ts', why: 'same' },
  // The design pack: its export was retired on 2026-09-15 and its payloads are named
  // conditionally ("once `pnpm design:export` has run"), which is now never.
  { path: 'contracts/payload/collection.json', why: 'design export retired 2026-09-15' },
  { path: 'contracts/payload/wiki_index.json', why: 'same' },
  { path: 'data/marks.json', why: 'design pack data/, removed with the export' },
  // Generated, and git-ignored on purpose: real on a built checkout, absent from `git ls-files`.
  {
    path: 'crates/app/gen/schemas/desktop-schema.json',
    why: 'written by tauri-build on every compile; .gitignore keeps it out',
  },
  // A document naming the file it is telling you went away.
  {
    path: 'screens/unlock/unlockLayout.test.ts',
    why: 'named inside the sentence that says the row height moved to lib/scale/rows.test.ts',
  },
  // 3.7's spec, naming what it will create. `useTabView.ts` left this list on 2026-09-16, when
  // 3.7a built it; the other two become GONE when 3.7b lands, which is the line that says to
  // delete them.
  {
    path: 'ui/src/lib/window/sessionWriter.ts',
    why: '3.7b tabs-session spec, not built yet',
  },
  { path: 'ui/src/lib/window/monitorClamp.ts', why: 'same' },
  // Files a spec named before creating, under a name that later changed — or never created,
  // because the spec made them conditional.
  {
    path: 'crates/app/src/cursor.rs',
    why: 'the spec says "only if the spike says so"; the spike did not',
  },
  { path: 'crates/catalog/examples/probe_graph.rs', why: 'a probe the graph spec planned' },
  { path: 'raw/characters.json', why: 'a dataset file the wiki spec planned' },
  { path: 'components/marks/CharacterHead.vue', why: 'planned name; the component landed elsewhere' },
  { path: 'screens/profile/profileView.ts', why: 'landed as lib/profile/profileView.ts' },
  { path: 'screens/collection/CollectionFacetDrawer.vue', why: 'became components/facets/FacetDrawer.vue' },
  { path: 'screens/NextStepsScreen.vue', why: 'became GoalsScreen.vue with B32' },
  { path: 'nextsteps/StepCard.vue', why: 'became screens/goals/GoalCard.vue with B32' },
  { path: 'auto-launch-0.5.0/src/windows.rs', why: "a dependency's own source, not this repo's" },
  { path: 'samples/filelist.txt', why: 'samples/ is git-ignored and not walked' },
]

const DOCS = [
  'CLAUDE.md',
  'DESIGN-BRIEF.md',
  'docs/PROJECT.md',
  'docs/STATUS.md',
  'docs/BACKLOG.md',
  'docs/IMPROVEMENTS.md',
  'docs/frontend-conventions.md',
  ...readdirSync('docs/superpowers/specs').map((f) => 'docs/superpowers/specs/' + f),
]

// **Tracked files, not the working tree.** Walking the disk made this answer differently on
// the machine that wrote it: `data/marks.json` resolved to a file under
// `design-export/design-system/`, which is git-ignored and exists only there, so a fresh clone
// would have reported a miss the author never saw. That is `samples/`'s lesson — present and
// looking right is worse than absent and noticed — and it applies to the checker too.
const files = execFileSync('git', ['ls-files'], { encoding: 'utf8' })
  .split('\n')
  .filter(Boolean)

const byBase = new Map()
for (const f of files) {
  const b = f.slice(f.lastIndexOf('/') + 1)
  if (!byBase.has(b)) byBase.set(b, [])
  byBase.get(b).push(f)
}

// Resolves as a path suffix, or as the shorthand the documents use — `ipc/graph_real.rs` for
// `crates/ipc/tests/graph_real.rs` — the basename plus every other segment appearing in order.
const resolves = (ref) => {
  if (files.some((f) => f === ref || f.endsWith('/' + ref))) return true
  const segs = ref.split('/')
  const base = segs.pop()
  return (byBase.get(base) ?? []).some((f) => {
    const parts = f.split('/')
    let at = 0
    for (const s of segs) {
      const i = parts.indexOf(s, at)
      if (i < 0) return false
      at = i + 1
    }
    return true
  })
}

const PATH =
  /`([A-Za-z0-9_@.\-/+]*\/[A-Za-z0-9_@.\-/+]*\.(rs|ts|vue|json|md|mjs|toml|css|py|txt|html))`/g

const exempt = new Map(EXEMPTIONS.map((e) => [e.path, e]))
const seenExempt = new Set()
const fresh = []

for (const doc of DOCS) {
  if (!existsSync(doc)) {
    fresh.push([doc, '(the document itself is missing)', 0])
    continue
  }
  const at = new Map()
  readFileSync(doc, 'utf8')
    .split('\n')
    .forEach((line, i) => {
      for (const m of line.matchAll(PATH)) {
        const p = m[1].replace(/^…\//, '')
        if (!p.startsWith('http') && !p.includes('*') && !at.has(p)) at.set(p, i + 1)
      }
    })
  for (const [p, line] of at) {
    if (resolves(p)) continue
    if (exempt.has(p)) {
      seenExempt.add(p)
      continue
    }
    fresh.push([doc, p, line])
  }
}

const stale = EXEMPTIONS.filter((e) => !seenExempt.has(e.path))

console.log(
  `doc references: ${DOCS.length} documents, ${files.length} files, ` +
    `${seenExempt.size} known-legitimate, ${fresh.length} new`,
)
for (const [doc, p, line] of fresh) console.log(`  NEW  ${doc}:${line}  ${p}`)
for (const e of stale) console.log(`  GONE ${e.path}  (exempted: ${e.why})`)
if (fresh.length === 0 && stale.length === 0) console.log('  nothing new, nothing stale')
