#!/usr/bin/env node
// **`samples/` is opened from `crates/test-support/` and nowhere else** (CLAUDE.md, *Tests*), and
// since 2026-09-24 this gate is what says so (card #81, C2). The rule exists because a test on
// real data has to declare which file it ran on or why it skipped — `test-support` does that on
// every call, a hand-built path does not, and on 2026-09-24 nine examples and three tests were
// building their own.
//
// Over every tracked `.rs` outside `crates/test-support/`, with `//` comments blanked (the repo
// explains this rule in comments everywhere, and a check that trips on its own explanation
// trains people to reword it — B59), it flags two things:
//
// - a string literal that **is** a path into `samples/`: `"samples"`, `"samples/logs/x.tsv"`,
//   `"../../samples/packed"`. A literal with a space in it is a sentence — the skip messages
//   say `"samples/packed missing: …"` on purpose — and is left alone;
// - a string literal starting `live.`: a sample is named by its date (CLAUDE.md, *Test data*),
//   and a file called "live" invites being overwritten.
//
// What it cannot see, said rather than found later: a path assembled from pieces
// (`"sam" + "ples"`, a `const` in another file), and `/* … */` comments, which it reads as code.
// Neither exists today.

import { execFileSync } from 'node:child_process'
import { readFileSync } from 'node:fs'

// Permanent exceptions, a reason each. Never a violation waiting for a fix.
const EXEMPTIONS = []

// A literal that is nothing but a path into samples/: any run of `/`, `./` and `../` in front
// (`"/../../samples"` is what `concat!(env!("CARGO_MANIFEST_DIR"), …)` needs), and `\` read as `/`.
const SAMPLES_PATH = /^(?:\.{0,2}[/\\])*samples(?:[/\\][^\s"]*)?$/
const LIVE_NAME = /^live\./

// A raw string's opening at `i`: `r"`, `r#"`, `br##"` and so on, not preceded by an identifier
// character (`for"` is not a raw string). Answers the number of `#` and where the text starts.
const RAW_OPEN = /^b?r(#*)"/
const rawAt = (line, i) => {
  if (i > 0 && /\w/.test(line[i - 1])) return null
  const m = RAW_OPEN.exec(line.slice(i))
  return m ? { hashes: m[1].length, start: i + m[0].length } : null
}

// The string literals of one line, with `//` comments outside strings ignored. Not a parser: it
// tracks `"…"` with backslash escapes, raw strings (`r"…"`, `r#"…"#`) without them, and stops at
// `//` outside one, which is what `.rs` lines in this repo need.
export const literalsOf = (line) => {
  const out = []
  let i = 0
  while (i < line.length) {
    const c = line[i]
    if (c === '/' && line[i + 1] === '/') break
    const raw = rawAt(line, i)
    if (raw) {
      const close = '"' + '#'.repeat(raw.hashes)
      const end = line.indexOf(close, raw.start)
      out.push(line.slice(raw.start, end === -1 ? line.length : end))
      i = end === -1 ? line.length : end + close.length
      continue
    }
    if (c === "'" && line[i + 2] === "'") {
      i += 3 // a char literal like '"' must not open a string
      continue
    }
    if (c === "'" && line[i + 1] === '\\' && line[i + 3] === "'") {
      i += 4 // nor an escaped one like '\"'
      continue
    }
    if (c === '"') {
      let j = i + 1
      let text = ''
      while (j < line.length && line[j] !== '"') {
        if (line[j] === '\\') j += 1
        text += line[j] ?? ''
        j += 1
      }
      out.push(text)
      i = j + 1
      continue
    }
    i += 1
  }
  return out
}

export const findingsOf = (source) =>
  source.split('\n').flatMap((line, n) =>
    literalsOf(line).flatMap((lit) => {
      if (SAMPLES_PATH.test(lit))
        return [{ line: n + 1, what: `path into samples/: "${lit}"` }]
      if (LIVE_NAME.test(lit))
        return [{ line: n + 1, what: `sample named "live": "${lit}"` }]
      return []
    }),
  )

// Every run proves the check can still speak before it is believed to have nothing to say.
const FIXTURES = [
  // The shapes the 2026-09-24 review found getting through the first version of this gate.
  { src: 'concat!(env!("CARGO_MANIFEST_DIR"), "/../../samples/packed")', found: 1 },
  { src: 'Path::new("./samples/packed")', found: 1 },
  { src: 'Path::new("..\\\\..\\\\samples")', found: 1 },
  { src: 'Path::new("samples\\\\packed")', found: 1 },
  { src: 'let q = \'\\"\'; let s = "samples";', found: 1 },
  { src: 'let p = Path::new("samples");', found: 1 },
  { src: 'x.join("../../samples/packed")', found: 1 },
  { src: 'PathBuf::from("samples/logs/probe.tsv")', found: 1 },
  { src: 'test_support::sample("live.rep+persistentgamedata1.dat")', found: 1 },
  { src: 'skip("samples/packed missing: no game")', found: 0 },
  { src: '// Path::new("samples") in a comment', found: 0 },
  { src: 'let url = "https://x"; // "samples"', found: 0 },
  { src: 'let q = \'"\'; let s = "samples";', found: 1 },
  { src: 'test_support::sample("20260831.rep+persistentgamedata1.dat")', found: 0 },
  // Raw strings keep `\` as it is: a Windows path written the way Windows writes it.
  { src: 'Path::new(r"..\\..\\samples")', found: 1 },
  { src: 'Path::new(r#"..\\samples\\packed"#)', found: 1 },
  { src: 'let d = r"C:\\"; let s = "samples";', found: 1 },
  { src: 'let d = br"x\\"; let s = "samples";', found: 1 },
  { src: 'let r = "a"; let s = "samples";', found: 1 },
]

const broken = FIXTURES.filter((f) => findingsOf(f.src).length !== f.found)
if (broken.length > 0) {
  for (const f of broken)
    console.log(`fixture broke: ${f.src} (expected ${f.found}, got ${findingsOf(f.src).length})`)
  process.exit(1)
}

const files = execFileSync('git', ['ls-files', '-z', '--', 'crates/*.rs'], {
  encoding: 'utf8',
})
  .split('\0')
  .filter((f) => f && !f.startsWith('crates/test-support/'))

const exempt = (file, what) =>
  EXEMPTIONS.some((e) => e.file === file && what.includes(e.literal))

const violations = files.flatMap((file) =>
  findingsOf(readFileSync(file, 'utf8'))
    .filter((f) => !exempt(file, f.what))
    .map((f) => `${file}:${f.line}: ${f.what}`),
)

for (const v of violations) console.log(v)
console.log(
  `${violations.length} violations, ${EXEMPTIONS.length} declared exemptions, ${FIXTURES.length} fixtures`,
)
process.exit(violations.length > 0 ? 1 : 0)
