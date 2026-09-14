# M4 sub-project 2b — Live: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this
> plan task by task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** while you play, the app shows the run it is watching and **what finishing it would
open** — and says plainly when it cannot tell which character you are.

**Spec:** `docs/superpowers/specs/2026-09-14-m4-run-screens-design.md` §3 and §4.

## The decision 2a's plan deferred: where the join happens

**In the backend, in one command.** The alternative was a second command the frontend asks
once it knows the character, and N8 closed the evening by undoing exactly that shape
everywhere else: two commands cannot promise that two answers describe the same profile,
because a save written between them makes them describe two. Live joins the archive's open
run with the graph's marks, and both come from one reading.

## The rule, and why it is narrow

A run **would open** an achievement when **every** requirement still missing from it is a mark
for the character being played. One other missing requirement — another character, an item —
and finishing this run does not open it, so it is not offered. The screen would rather say
less than promise something the graph does not.

Grouped by the cell it needs: *beat Mom's Heart with Cain → these three*. A flat list would
make the player read the same boss five times.

## The character, which is the hard part (spec §3)

The archive has the **name** the log printed, and the game gives a Tainted character the base
form's name. So a name can answer to two ids, and the screen says **both** — with the fact
that it is both, not a guess. Resolving it from the starting items is refused here: it is an
inference, and inferences that name things are what this repo spends its corrections on.

## File structure

| File | Responsibility |
|---|---|
| `crates/ipc/src/live.rs` (create) | `LiveView`, `LiveOpen`, `LiveDiagnostic`, and the pure join. |
| `crates/ipc/src/lib.rs` (modify) | Exports. |
| `crates/ipc/src/contract.rs` (modify) | The new types into the generated contract. |
| `crates/ipc/tests/live.rs` (create) | The rule, the ambiguity, and every diagnostic. |
| `crates/app/src/commands/runs.rs` (modify) | The `live` command: the open run and the unlock view, joined once. |
| `crates/app/src/lib.rs` (modify) | Registers it. |
| `ui/src/lib/ipc/runs.ts` (modify) | The typed wrapper. |
| `ui/src/screens/LiveScreen.vue` (create) | The run, and what it would open. |
| `ui/src/router/routes.ts` (modify) | `RouteName.Live` stops falling through. |
| `ui/src/i18n/messages/*.ts` (modify) | Its strings, in parity. |
| `ui/src/lib/ipc/fixtures/*` (modify) | A live answer, including the ambiguous case. |

## Tasks

- [ ] **1. The pure join, test first.** Every missing requirement is a mark for this
      character → the node is offered; one other missing requirement → it is not. Grouping by
      `(character, column, level)`. A run with no character named, a name that answers to two
      ids, a name that answers to none, no graph, no open run: each its own diagnostic, none of
      them silence.
- [ ] **2. The contract**: types declared, `pnpm ipc:types` regenerated, never hand-edited.
- [ ] **3. The command**, gathering the archive's open run and the unlock view in one call.
- [ ] **4. The screen**: the run (character, floor count, what it holds), then what it would
      open, grouped by cell, with the ambiguity said out loud when it is there.
- [ ] **5. The route and the strings.**
- [ ] **6. `pnpm check`, and the window.** The fixtures must carry the ambiguous case, because
      it is the one a real machine shows only while a Tainted character is being played.

## Out of scope

- Resolving the Tainted ambiguity from the starting items. It returns with a measurement.
- Anything about the run that the archive does not already carry.
