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
  // Card #82 (2026-09-25) split two ipc files into modules and removed dead code; the dated
  // specs name the files they were written against.
  {
    path: 'screens/PlaceholderScreen.vue',
    why: 'the router placeholder machinery was removed once every route had a screen; the 2026-09-11 shell spec designed it',
  },
  {
    path: 'crates/ipc/src/search.rs',
    why: 'became crates/ipc/src/search/{mod,rank,text}.rs; the 2026-09-12 search and blocked-menu specs name the file they changed',
  },
  {
    path: 'crates/ipc/examples/dlc_mask.rs',
    why: 'a finished probe, deleted; the 2026-09-13 infobox spec records what it measured',
  },
  {
    path: 'target/release/latest.json',
    why: 'the updater manifest `pnpm release:manifest` writes; a build output under target/, never tracked, and the release design names where it lands',
  },
  // A document recording a rename has to name the file that went away. Most of these left with
  // the journal and the closed backlog entries on 2026-09-16.
  {
    path: 'docs/IMPROVEMENTS.md',
    why: 'moved to docs/completed/quality-review.md; a dated spec names it as it was',
  },
  {
    path: 'docs/progetto.html',
    why: 'the original Italian project document, rewritten as docs/PROJECT.md on 2026-09-07; the status names it as what master used to show, which is history and not a promise',
  },
  // Obiettivi and the Plan merged into one screen on 2026-09-22, and three files went with the
  // merge. The documents naming them are recording the going, which is the case this report
  // cannot tell apart from a promise on its own.
  {
    path: 'ui/src/screens/plan/QueueRow.vue',
    why: 'the queue row became the shared ui/src/components/plan/GoalRow.vue; the 2026-09-13 drag spec names the file it was written against',
  },
  {
    path: 'ui/src/screens/goals/GoalCard.vue',
    why: 'replaced by that same shared row; the merge spec lists it under what stops existing, which has to name it',
  },
  {
    path: 'ui/src/screens/plan/ProposalAside.vue',
    why: 'same: it was the recommendations drawn a second time, and the left pane is that, done once',
  },
  {
    path: 'ui/src/lib/window/session.ts',
    why: 'became ui/src/composables/useWindowSession.ts on 2026-09-24 (card #81, V7: a module that imports a store is a composable); the 2026-09-15 tabs spec names where it was written',
  },
  {
    path: 'components/shell/sectionNav.ts',
    why: 'moved to lib/shell/sectionNav.ts on 2026-09-24 (card #81, V7); the 2026-09-12 scale spec names where it was',
  },
  {
    path: 'components/shell/sidebarWidth.ts',
    why: 'moved to lib/shell/sidebarWidth.ts on 2026-09-24 (card #81, V7); the 2026-09-20 responsive spec names where it was',
  },
  {
    path: 'components/wiki/dlcNames.ts',
    why: 'moved to lib/wiki/dlcNames.ts on 2026-09-24 (card #81, C4: lib/ imports no component); the 2026-09-12 wiki spec names where it was written',
  },
  { path: 'stores/graph.ts', why: 'three view stores became one stores/views.ts with N8' },
  { path: 'stores/completion.ts', why: 'same' },
  { path: 'stores/collection.ts', why: 'same' },
  // The design pack: its export was retired on 2026-09-15 and the pack itself left the
  // repository, working tree and history both, on 2026-09-20. `docs/completed/` records what
  // it was and what it carried, which is history and not a promise.
  { path: 'contracts/payload/collection.json', why: 'design pack removed 2026-09-20' },
  { path: 'contracts/payload/wiki_index.json', why: 'same' },
  { path: 'contracts/types.ts', why: 'same' },
  { path: 'data/marks.json', why: 'design pack data/, removed with the export' },
  { path: 'images/INDEX.json', why: 'moved to ui/fixtures/index.json with the removal' },
  {
    path: 'contracts/payload/unlock.json',
    why: 'moved to ui/fixtures/payload/unlock.json on 2026-09-20; a dated spec names where it was',
  },
  {
    path: 'contracts/payload/queue.with_rows.json',
    why: 'a pack payload no fixture read; removed 2026-09-20',
  },
  {
    path: 'design-export/src/sheets.rs',
    why: 'the exporter, removed 2026-09-20; a dated spec names it as it was',
  },
  { path: 'crates/design-export/src/atlas.rs', why: 'same' },
  { path: 'crates/design-export/src/payload.rs', why: 'same' },
  // The three modules that served the pack's images. They went with it on 2026-09-20 — every
  // URL they answered was already null — and the documents that name them are recording that,
  // which is history and not a promise.
  { path: 'fixtures/art.ts', why: 'art fixtures removed with the design pack, 2026-09-20' },
  { path: 'lib/ipc/fixtures/art.ts', why: 'same' },
  { path: 'fixtures/graphArt.ts', why: 'same' },
  { path: 'kit/markArt.ts', why: 'same' },
  { path: 'src/kit/markArt.ts', why: 'same' },
  // Generated, and git-ignored on purpose: real on a built checkout, absent from `git ls-files`.
  {
    path: 'crates/app/gen/schemas/desktop-schema.json',
    why: 'written by tauri-build on every compile; .gitignore keeps it out',
  },
  // Paths **inside the built installer**, not inside the repo: B11 names where the licences land
  // once the app is packaged, which is a fact about an artefact and can never be a file git
  // tracks. The report reads paths, not what a path is about.
  {
    path: 'licenses/determination/license.txt',
    why: 'a path inside the installer, verified by extracting the MSI (B11)',
  },
  { path: 'licenses/determination/readme.txt', why: 'same' },
  { path: 'licenses/wiki/ATTRIBUTION.md', why: 'same' },
  // A document naming the file it is telling you went away.
  {
    path: 'screens/unlock/unlockLayout.test.ts',
    why: 'named inside the sentence that says the row height moved to lib/scale/rows.test.ts',
  },
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
  { path: 'components/facets/FacetDrawer.vue', why: 'the drawer 3.10 replaced with FilterBar.vue; B29 records where it went' },
  { path: 'auto-launch-0.5.0/src/windows.rs', why: "a dependency's own source, not this repo's" },
]

const DOCS = [
  'CLAUDE.md',
  'DESIGN-BRIEF.md',
  'docs/PROJECT.md',
  'docs/architecture.md',
  'docs/STATUS.md',
  'docs/BACKLOG.md',
  'docs/frontend-conventions.md',
  ...readdirSync('docs/superpowers/specs').map((f) => 'docs/superpowers/specs/' + f),
]

// **Tracked files, not the working tree.** Walking the disk made this answer differently on
// the machine that wrote it: `data/marks.json` resolved to a file under the design pack's
// git-ignored half, which existed only there, so a fresh clone would have reported a miss the
// author never saw. That is `samples/`'s lesson — present and looking right is worse than
// absent and noticed — and it applies to the checker too.
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
