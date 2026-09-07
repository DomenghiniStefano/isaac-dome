# Tauri shell and IPC boundary — implementation report

**Date:** 2026-09-03
**Plan:** `docs/superpowers/plans/2026-09-02-app-shell-ipc.md`
**Spec:** `docs/superpowers/specs/2026-09-02-app-shell-ipc-design.md`
**Outcome:** 12 of 12 tasks completed, all passed review.

## What exists now

| Crate / folder | Contents | Tests |
|---|---|---|
| `crates/ipc` | Pure view-models: opaque profile id, candidates, active-profile resolution, mark matrix, save summary, settings. No I/O, no dependency on Tauri. | 36 |
| `crates/app` | Tauri crate: four commands, tagged `IpcError`, persistence of the choice to `settings.json`. | — (wiring) |
| `ui/` | Vue 3 + TypeScript, Vite on port 1420, Tailwind v4, typed IPC layer, verification screen, conventions scanner. | — (configuration) |

**Final gate:** 87 Rust tests passed, 0 failed. `cargo fmt --check` and
`cargo clippy --all-targets -- -D warnings` clean. `pnpm typecheck`, `pnpm lint`,
`pnpm format:check` and `pnpm scan` green, with the scanner at `0 violations`.

## The result that matters

The cross-check against the Python reference (`reference/isaac_counters.py`) compares the
matrix built in Rust with the one from the original implementation, **cell by cell**, on the
real save from January 2025:

```
321 cells compared, all matching
 93 marks started
  0 suspect values
 19 cells not located (Delirium column for the 19 characters of the third group)
```

The tables carried over by hand from the Python are correct. The implementer verified that the
test can actually fail by temporarily corrupting an index, and the reviewer confirmed on the
diff that it had been restored.

## What review caught

Three rounds of correction across twelve tasks, **all of them plan defects**, and all of the
same family: a piece of data crossing the IPC boundary without anyone having explicitly decided
what it should carry with it.

| Task | Defect | How it would have failed |
|---|---|---|
| 2 | `pathHint` contained the Steam account id, because with Steam Cloud the id is a folder segment. The test called for by the plan used a fixture without that segment: it verified the absence of something it never had. | Silent: personal data in the payload, no error |
| 3 | `rename_all` on an enum renames the variants, not the fields of struct variants. `autoSelected` would have arrived as `auto_selected`. | Silent: `undefined` on the TypeScript side |
| 7 | `format!("{:?}")` on diagnostics printed the raw `PathBuf`s from `discovery` — account id and Windows username — and the offsets from `core-save`. | Silent: forbidden data in the app's very first command |
| 11 | Nested ternaries on `Cell` instead of an exhaustive `switch`, on the type the conventions cite by name. | Silent: a future variant labeled as "suspect" |
| 12 | The exception for `<style>` blocks only needed to be named in a comment to be granted. | Silent: rule bypassable by mistake |

**None of these produced an error, a crash, or a red test.** That is why the review gate after
each task earned its cost: an implementer following the brief to the letter could not have seen
them, and neither could the green suite.

The rules born from this now live in `CLAUDE.md` — never `format!("{:?}")` across the boundary,
`rename_all_fields` on enums with struct variants, mandatory exhaustiveness — because a defect
that fails silently is worth more as a written rule than as a one-off fix.

## Decisions made during execution

- **`pathHint` gets redacted, not truncated.** The field exists to tell the user *"found here"*;
  truncating it would destroy that use in the manual-fallback case. Only the account-id segment
  is replaced, which `SaveSource::SteamCloud` already carries.
- **Diagnostics carry only the last path component** (`remote`,
  `appmanifest_250900.acf`): they say what failed to read without any identifying segment.
- **Domain-crate enums stay `snake_case`.** Wrapping them in view-models just to unify casing
  would be duplication with no payoff; the rule is written down and pinned by a test.
- **Exceptions to `<style>` blocks are declared with a marker**, rather than being recognized:
  recognizing them would require a CSS parser, and the approximation was easy to bypass.
- **Placeholder icons** in a neutral tint: `tauri-build` on Windows requires them, and a project
  constraint forbids shipping game assets in the package.
- **`.gitattributes` with `eol=lf`**: the repo had `core.autocrlf=true` and no
  `.gitattributes`, so git checked out CRLF for files whose blobs are LF, leaving
  `format:check` red on otherwise-correct files.

## What's missing

- **The application has never been launched.** No agent could open a window without getting
  stuck on it. It's the one verification from the plan still pending, and it has to be done by
  hand.
- The real Completion screen, which will come with the design system.
- `catalog`, still blocked on the absence of the game's XML files.
