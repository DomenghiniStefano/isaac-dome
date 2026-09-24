// Whether a branch has fewer tests than the point it left `develop` at.
//
// **Why it exists** (B63): on 2026-09-12 one commit deleted `crates/ipc/tests/profile.rs` — 647
// lines, 27 tests — in the same diff that added `progress.rs`, and every gate stayed green for four
// days. A test runner reports what ran and never what stopped existing: a deleted test is not a
// dimmed suite, it is a shorter one, and it makes no sound at all.
//
// **Why it reads git and keeps no number** (card #84, 2026-09-24). From 2026-09-16 to that day the
// same guard was two totals in `scripts/test-floor`, and every branch that added a test had to
// raise them. Two branches in parallel therefore always conflicted on that file, whatever they were
// about — six times in its own record, once auto-merged to the wrong number, twice in a row on the
// day it went — and the paragraph owed with each raise had stopped being written. The comparison
// this makes needs no stored number: the other side of it is the branch's own merge base, read
// with `git grep` at that revision, and nothing is compiled.
//
// **What it counts is declarations, not runs.** `#[test]` in Rust — 1254 of them, the same 1254
// `cargo test` runs, measured on the day this was written — and `it(`/`test(` in Vitest, where an
// `it.each` with four cases is one declaration and four runs. For the question it answers, "did
// somebody delete tests", a declaration is the right unit: a case taken out of an `it.each` table
// goes unnoticed, and that is the price of not running anything.
//
// **What fails and what is only said.** A suite whose total shrank fails the run. A file that lost
// tests is *named* whatever the totals do, because B63's shape was exactly a file leaving while
// another arrived — the total held and the file went. Naming it is what a person reads; failing on
// it would fail every file that is renamed or split.
//
// **A deliberate removal** is `ISAACDOME_ALLOW_FEWER_TESTS=1 pnpm check`, and the commit that makes
// it says why. It passes and still prints what went.

import { execFileSync } from 'node:child_process'

const SUITES = [
  { name: 'rust', pattern: '#\\[([a-z_]+::)?test\\]', paths: ['*.rs'] },
  {
    name: 'ui',
    pattern: '^[[:space:]]*(it|test)(\\.[a-zA-Z]+)*[[:space:]]*[(`]',
    paths: ['*.test.ts'],
  },
]

// `git grep -c` prints `path:count`, or `rev:path:count` at a revision. The count is after the last
// colon; the revision, when there is one, is before the first.
const parseCounts = (text, rev) =>
  new Map(
    text
      .split('\n')
      .filter((line) => line.length > 0)
      .map((line) => {
        const bare = rev === null ? line : line.slice(rev.length + 1)
        const at = bare.lastIndexOf(':')
        return [bare.slice(0, at), Number(bare.slice(at + 1))]
      }),
  )

const total = (counts) => [...counts.values()].reduce((sum, n) => sum + n, 0)

const compare = (before, after) => ({
  before: total(before),
  after: total(after),
  lost: [...before.entries()]
    .filter(([path, n]) => (after.get(path) ?? 0) < n)
    .map(([path, n]) => ({ path, before: n, after: after.get(path) ?? 0 })),
})

const verdict = (suites, allow) => {
  const shrank = suites.filter((s) => s.after < s.before).map((s) => s.name)
  return shrank.length === 0 || allow
    ? { ok: true, allowed: allow ? shrank : [] }
    : { ok: false, allowed: [] }
}

const FIXTURES = [
  {
    name: 'the working tree prints path:count',
    run: () => [
      ...parseCounts('crates/a/tests/x.rs:3\ncrates/b/src/lib.rs:12\n', null),
    ],
    want: [
      ['crates/a/tests/x.rs', 3],
      ['crates/b/src/lib.rs', 12],
    ],
  },
  {
    name: 'a revision prefixes every line with itself, and the prefix is not the path',
    run: () => [...parseCounts('abc123:ui/src/a.test.ts:7\n', 'abc123')],
    want: [['ui/src/a.test.ts', 7]],
  },
  {
    name: 'no match at all is an empty suite, not an error',
    run: () => [...parseCounts('', null)],
    want: [],
  },
  {
    // B63's shape: a file goes with its tests and another arrives. The total alone cannot see it,
    // so the file that went is named whatever the totals say.
    name: 'a file that left is named even when the total held',
    run: () =>
      compare(
        new Map([
          ['profile.rs', 27],
          ['other.rs', 4],
        ]),
        new Map([
          ['progress.rs', 27],
          ['other.rs', 4],
        ]),
      ),
    want: {
      before: 31,
      after: 31,
      lost: [{ path: 'profile.rs', before: 27, after: 0 }],
    },
  },
  {
    name: 'a file that kept some of its tests is named with what it has left',
    run: () => compare(new Map([['a.rs', 5]]), new Map([['a.rs', 3]])),
    want: { before: 5, after: 3, lost: [{ path: 'a.rs', before: 5, after: 3 }] },
  },
  {
    name: 'a suite that grew loses nothing',
    run: () => compare(new Map([['a.rs', 5]]), new Map([['a.rs', 6]])),
    want: { before: 5, after: 6, lost: [] },
  },
  {
    name: 'a suite that shrank fails',
    run: () => verdict([{ name: 'rust', before: 5, after: 4 }], false),
    want: { ok: false, allowed: [] },
  },
  {
    name: 'a suite that shrank on purpose passes, and still says so',
    run: () => verdict([{ name: 'rust', before: 5, after: 4 }], true),
    want: { ok: true, allowed: ['rust'] },
  },
  {
    name: 'a suite that held or grew passes',
    run: () =>
      verdict(
        [
          { name: 'rust', before: 5, after: 5 },
          { name: 'ui', before: 5, after: 9 },
        ],
        false,
      ),
    want: { ok: true, allowed: [] },
  },
]

const git = (args) =>
  execFileSync('git', args, { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim()

// Exit status 1 is `git grep`'s "no match", which is an empty suite and not a failure.
const grepCounts = (suite, rev) => {
  // The working tree with `--untracked`, so a test file not yet added still counts; an option,
  // so it goes before the pattern, where a revision cannot be mistaken for it.
  const args =
    rev === null
      ? ['grep', '-c', '--untracked', '-E', suite.pattern, '--', ...suite.paths]
      : ['grep', '-c', '-E', suite.pattern, rev, '--', ...suite.paths]
  try {
    return parseCounts(git(args), rev)
  } catch (e) {
    if (e.status === 1) return new Map()
    throw e
  }
}

const refExists = (ref) => {
  try {
    git(['rev-parse', '--verify', '--quiet', `${ref}^{commit}`])
    return true
  } catch {
    return false
  }
}

// The point this branch left `develop` at. On `develop` itself that is `HEAD`, and the comparison
// is the working tree against it — which is a merge in progress, the one moment on `develop` when
// tests can go. Local `develop` before `origin/develop`: it is the branch the merge lands on.
const findBase = () => {
  const ref = ['develop', 'origin/develop'].find(refExists)
  return ref === undefined ? null : { ref, rev: git(['merge-base', 'HEAD', ref]) }
}

const fixtureFailures = FIXTURES.flatMap((f) => {
  const got = JSON.stringify(f.run())
  const want = JSON.stringify(f.want)
  return got === want ? [] : [`fixture "${f.name}": expected ${want}, got ${got}`]
})
fixtureFailures.forEach((f) => console.error(f))

const base = findBase()
if (base === null) {
  console.log(`no develop to compare against: nothing checked (${FIXTURES.length} fixtures)`)
  process.exit(fixtureFailures.length === 0 ? 0 : 1)
}

const results = SUITES.map((suite) => ({
  name: suite.name,
  ...compare(grepCounts(suite, base.rev), grepCounts(suite, null)),
}))
const allow = process.env.ISAACDOME_ALLOW_FEWER_TESTS === '1'
const decided = verdict(results, allow)

console.log(`against ${base.ref} at ${base.rev.slice(0, 8)} (the merge base):`)
results.forEach((r) => {
  console.log(`  ${r.name}: ${r.before} -> ${r.after} declared`)
  r.lost.forEach((l) => console.log(`    fewer in ${l.path}: ${l.before} -> ${l.after}`))
})
if (!decided.ok)
  console.log(
    '  a suite is SHORTER than where this branch started. If tests were meant to go, rerun with ISAACDOME_ALLOW_FEWER_TESTS=1 and say why in the commit.',
  )
decided.allowed.forEach((name) =>
  console.log(`  ${name} is shorter, allowed by ISAACDOME_ALLOW_FEWER_TESTS`),
)
console.log(`${FIXTURES.length} fixtures`)
process.exit(decided.ok && fixtureFailures.length === 0 ? 0 : 1)
