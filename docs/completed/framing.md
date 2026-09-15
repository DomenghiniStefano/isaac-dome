# Completed — the framing that sat in CLAUDE.md

A narrative of what was built and why, which lived under `## State` in `CLAUDE.md` until
2026-09-16. It opened by saying the real state is in `docs/STATUS.md` and that what followed was
"just the framing" — and it was a third copy of the state, beside STATUS and PROJECT, which is
how two of them came to disagree.

**Searched, never read.** It is kept whole because parts of it are recorded nowhere else: the
design export arriving, the shell's six screens, Floor under Tool. Where it describes state it
is out of date by construction; where it describes *what was built*, it is the only account.


**The up-to-date state lives in `docs/STATUS.md`**, as checkboxes: milestones, modules,
open blockers, and a session log. What follows is just the framing.

M0 closed. M1 closed on the Rust side, structural base closed on 2026-09-05: static data
normalized and IPC contracts fixed. M2 (the graph) closed on 2026-09-07 and M3's plan
queue on 2026-09-08. **The design export arrived on 2026-09-10, and cycles 1 and 2 of the design
system — tokens, font, `cn()`, i18n, 23 primitives, then the app components (title bar,
navbar, sidebar, KPI, matrix cell, wiki tokens, data states) on the development-only Kit
page — landed the same day.**
**The export is retired since 2026-09-15**: the design is decided at runtime on the real
screens, `pnpm design:export` is not run again, and the pack's 6065 images left the repository
with it — 46 MB of a commercial game's sprites produced by a command nobody will run, against
this file's own first promise. What stays committed is 28 files and 2.8 MB, and it stays because
`ui/src/lib/ipc/fixtures/*` reads it: the real payloads for 720 items and 642 achievements, two
indexes and ten wiki pages. `?art=none` is the only art there is now.

**The IPC contract is live, and since 2026-09-13 it is generated.** `pnpm ipc:types` writes
`ui/src/lib/ipc/types.ts` from the Rust types; `scripts/check` regenerates into a scratch copy
and goes red when the committed file disagrees. **Never edit that file** — the edit is lost on
the next run and the check fails in between. What still has to be **handed on, not merely
committed**, is the change to the contract itself: a reshaped type is a change to material a
design is being built on, and that is a fact about the design, not about a file.

The silent case that made this worth doing is closed by construction: `core_save::Kind` crosses
inside `ipc::SectionCount`, the hand-written mirror typed that field as `string`, and renaming a
variant changed the wire with the whole suite green. It is `Kind` on both sides now, pinned by
`the_sections_of_the_save_are_a_closed_set_of_names` in `crates/ipc/tests/contract_foreign.rs`
alongside `crates/ipc/tests/summary_shape.rs`. Generation found two more of the same shape the
day it landed — `GameView.edition` was `string`, `dlcs` was `string[]` — and one worse: `Infobox`
had no `transformation` variant, so a transformation page reached `assertNever` and threw.

**A doc comment on a wire type is UI source now.** It becomes the JSDoc of the generated file,
so it lives under `pnpm scan`'s rules — an arrow or a check mark in a `///` fails the scan,
because the app's font has no glyph for it.

**`ui/` is the shell, with six real screens** (2026-09-11): tabs, router, the live window
chrome, the profile indicator; profile selection (3.1), Completion (3.2), whose mark symbols
and character heads are crops of the user's own sheets served through the icon protocol,
Next steps and Unlock (3.3a), which draw a node's state and its why one way, with Unlock's
facets, search and sort as pure functions over a virtualized table, the Plan (3.3b), the
first screen that writes: a queue you drag, whose drop names the row it lands under and whose
answer is the order drawn next, and the Collection (3.4), the save's item collection joined
with the catalog — collectibles only, since trinkets have no slot, and unread never shown as
"not found". Every other screen is a placeholder naming the sub-project that
brings it.

**Floor (F1, 2026-09-15) is the first screen that answers without reading anything**: you paint
the floor's minimap on a 13x13 grid and the empty cells light up where the game's own
documented rules allow a Secret, Super Secret or Ultra Secret room, each lit cell naming the
rule, quoting the sentence and giving the page — which is how the dataset's CC BY-SA
attribution reaches the person reading the screen. Spec
`docs/superpowers/specs/2026-09-15-floor-secret-rooms-design.md`, report
`docs/superpowers/reports/2026-09-15-floor-report.md`, the rules themselves and what could
**not** be sourced in `docs/superpowers/reports/2026-09-15-secret-room-rules.md`. It lives
under **Tool**, the section for screens that answer without the save.

The verification page lives behind `#verify` and the Kit behind `#kit`, both
development-only; `pnpm ui:dev` runs without the backend on fixtures
(`?fixture=none|pick|active`, `?art=none` for the first-launch outfit without sprites,
`?catalog=none` for a machine without the game, `?queue=empty|unavailable|unreadable`,
`?floor=empty` for a grid nobody has painted). Frontend conventions and their scanner exist already on
purpose: a rule introduced before the code is free.

The **wiki dataset** (crate `wiki`, tool `wiki-snapshot`, `dataset/`) is implemented,
passed whole-branch review, and **merged into `develop`** on 2026-09-06.

Other documents: `DESIGN-BRIEF.md` (the design system's contract, with the TypeScript
types); `docs/IMPROVEMENTS.md` (quality tasks with closing criteria); `docs/BACKLOG.md`
(registered, not-yet-started tasks).

**Every open backlog entry says what it needs before you can start it** — a `**Needs:**` line
under its heading, one of `nothing`, `a real save`, `the game`, `a measurement`, with
`, then a window` where finishing it means looking at the built app. The vocabulary, the reason
each value exists and the current counts are at the top of `docs/BACKLOG.md`; N8 in
`docs/STATUS.md` carries one too. **On a machine without the game, read those first**: fourteen
of the twenty-four open entries need nothing but a clone, and finding that out by trying is how
the session of 2026-09-14 started.

Every module follows spec → TDD plan → execution →
report, one folder per stage: `docs/superpowers/specs/`, `docs/superpowers/plans/`,
`docs/superpowers/reports/`. A plan is an execution script, single-use: **when its
sub-project merges into `develop`, the plan moves to `docs/superpowers/plans/archive/`**, so
what `plans/` lists is what's being executed right now. The spec stays where it is — it's the
durable half, and five doc comments under `crates/` and `ui/` point at it.

