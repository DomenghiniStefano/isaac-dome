# A contract that is generated, not mirrored

**Status:** design agreed in conversation on 2026-09-13. This is N7 in `docs/STATUS.md`'s
*Next up* section and **B2** in `docs/IMPROVEMENTS.md`, whose spike — run on 2026-09-13 with
`ts-rs` 12.0.1 — is the input to this document and is not repeated here.

**Scope.** `ui/src/lib/ipc/types.ts` stops being written by hand. A binary in `crates/ipc`
emits it from the Rust types themselves, a post-step inside that binary repairs the one case
the generator cannot express in this repo's style, and `scripts/check` fails when the
committed file and the Rust types disagree. 865 lines and 81 exported types leave the set of
things a human can get wrong.

**Not** in scope: the command wrappers in `ui/src/lib/ipc/*.ts` (they are call shapes, not
types), the shape tests in `crates/ipc/tests/`, and anything about the Tauri command names or
their arguments. §8 says why each is left out.

---

## 0. What was verified for this design, and how

Read on 2026-09-13 from the vendored sources in `~/.cargo/registry`, not from memory. The
spike had left nothing behind in the repo — no `ts-rs` in any manifest, none in `Cargo.lock`.

1. **A declaration can be had without a file and without imports.** `TS::decl(&Config)`
   returns `type X = …` on its own (`ts-rs-12.0.1/src/lib.rs:420`), and `TS::docs()` returns
   the JSDoc block built from the Rust `///` comments (`:398`). `export.rs:314-322` shows the
   crate's own assembly order — docs first, then the declaration — but `generate_decl` is
   private, so the caller does those two calls itself. **This is what makes one file possible
   without an import graph**, and it is why §1 does not use the export path at all.
2. **`Config` is the whole of the generator's behaviour, and it is ours to set.**
   `lib.rs:581-601`: `export_dir` (default `./bindings`), `large_int_type` (default
   `"bigint"`), `import_extension`, `array_tuple_limit`. Env variables feed it
   (`TS_RS_EXPORT_DIR`, `TS_RS_LARGE_INT`), `Config::new()` takes the defaults, and
   `with_large_int` overrides one (`:648`).
3. **`u64` and `i64` would arrive as `bigint`, and that would be a lie about the wire.**
   `usize`/`isize`/`f32`/`f64` map to `number` (`lib.rs:1139`), but `impl_large_integers!`
   (`:738-743`, applied at `:1146`) gives `u64`/`i64` the configured `large_int_type`, which
   defaults to `bigint`. The boundary has `i64` fields — `created_unix` in `goals.rs:194` and
   `graph.rs:287`, `modified_unix` in the profile — and **JSON has no bigint**: `serde_json`
   writes a number and the frontend reads a number. So the binary sets
   `with_large_int("number")`, and the reason is the wire, not convenience.
4. **serde's attributes are read by default.** `default = ["serde-compat"]` in
   `ts-rs-12.0.1/Cargo.toml:48`. That feature is what makes `rename_all`,
   `rename_all_fields`, `tag`, `rename` and `transparent` — every attribute this repo's IPC
   rules are written in terms of — the generator's input rather than something restated.
5. **The hand-written file and the generator already agree on the convention that could have
   diverged.** `Option<T>` is spelled `T | null` 47 times in `types.ts` and `?:` appears
   **zero** times, which is exactly what `ts-rs` emits for `Option<T>`. The one systematic
   difference that would have touched every nullable field on the boundary does not exist.
6. **Five types cross the boundary from crates that are not `ipc`**, and they are where the
   documented incident lives: `core_save::Kind` (`crates/core-save/src/section.rs:18`) —
   the one `CLAUDE.md` records as having changed the wire with the whole suite green —
   `wiki::Target` (`crates/wiki/src/model.rs:147`), and `discovery`'s `Edition`
   (`src/lib.rs:61`), `Dlc` (`:71`) and `SavePrefix` (`:98`).

---

## 1. The shape: a binary, an explicit list, one file

`crates/ipc/src/bin/ipc-types.rs`, run as `pnpm ipc:types`. It holds an **ordered list of the
types that cross the boundary** and writes `ui/src/lib/ipc/types.ts` by concatenating
`T::docs()` and `T::decl(&cfg)` for each one.

`ipc` is described as a pure crate and this binary writes a file. That is not a new exception:
`graph` carries the same description and has `src/bin/graph-rules.rs`, which writes
`rules/requirements.json`. The shape already exists in the repo and is already argued.

**Why a binary and not `#[ts(export)]`.** `ts-rs`'s native mode writes the bindings as a side
effect of `cargo test`. That would make the suite modify the working tree, and it would leave
`scripts/check` unable to name where the file comes from. A generator is a command you run —
`pnpm graph:rules` and `pnpm wiki:build` already set that precedent, and `scripts/check` is the
one place the project's commands are allowed to live.

**Why an explicit list and not `export_all`.** `export_all` walks dependencies and decides the
file layout itself. The list gives three things instead: the order of the file is ours, so a
diff stays readable; the list **is** the registry of what crosses the boundary, which is a fact
worth stating in one place; and a type reachable only as a dependency of another cannot appear
by accident. The failure mode of an incomplete list is a `types.ts` that does not compile, and
`pnpm typecheck` is already in `scripts/check` — the net is structural and already there.

## 2. Where the derive goes

`#[derive(TS)]` on the types of `ipc`, and on the five of §0.6 in `core-save`, `wiki` and
`discovery`. `ts-rs` becomes an ordinary dependency of those four crates.

**Not behind a feature gate**, and the reason is worth writing down because the opposite reads
as the tidier choice. "Pure" in this repo means *does not touch the disk* — `wiki` is pure
because it only reads `dataset/raw/` — and a derive is compile-time and touches nothing. The
gate would buy keeping a small crate out of the release binary's dependency graph, and it would
cost four manifests with optional dependencies plus a `required-features` on the binary, all of
which have to stay aligned with each other forever. Five types do not justify that.

**Declaring the five by hand in the post-step was considered and rejected.** It is the one
outcome that makes this whole item pointless: `core_save::Kind` is the documented incident, and
leaving it hand-written leaves the incident possible.

## 3. The one gap: fieldless enums

`ts-rs` renders a fieldless enum as a string union — `"steamNotFound" | "gameNotFound" |
"noSaves"`. Frontend rule 5 forbids literal unions, `pnpm scan` enforces it, and every component
compares against a symbol (`MissingReason.SteamNotFound`), so the union breaks the rule and the
call sites at once. There are 16 such enums in the file today.

The binary rewrites them, **by form and not by list**: any declaration whose right-hand side is
a union of string literals becomes the `const … as const` pair plus its `type` alias, with the
key the PascalCase of the value (`steamNotFound` → `SteamNotFound`, `rep_plus` → `RepPlus`,
matching what the file already spells today).

By form, because rule 5 admits no exceptions: there is no string union that should survive, so
there is nothing for a list to protect and nothing for a list to forget when the seventeenth
enum arrives. `pnpm scan` stays the net underneath.

**This is not a different crate's job.** Both `ts-rs` and `specta` target idiomatic TypeScript,
and `const … as const` is this repo's convention, not a generator's. The gap needs a post-step;
evaluating a second generator would end at the same union.

## 4. Formatting

`pnpm ipc:types` is two commands: the binary, then `prettier --write` on the file it wrote.
`pnpm format:check` is in `scripts/check` and has to stay green, and hand-assembled output
cannot promise prettier's exact shape — so it is not asked to.

## 5. The staleness gate

`scripts/check` regenerates into a temporary directory and fails if the committed `types.ts`
differs, naming `pnpm ipc:types` in the failure. It does not write into the working tree: a
check that repairs what it is checking cannot report on it.

**Not in the pre-commit hook.** The hook runs the two fast checks by design; this one is a
`cargo run` plus prettier, and the rule about what the hook is for does not change for it.

## 6. The comments

The 865 lines carry real knowledge — "`miniZ` is not a typo" is there because B9 turned it into
a rule. Every comment that carries a fact becomes a `///` on the Rust type, where it is next to
what it describes and can no longer drift from it; the ones that only say *Mirrors
`crates/ipc/src/reasons.rs`* die with the file they were navigating.

This is the slowest part of the work and the one with no technical risk. It is also the part
that turns the item from a deduplication into a move: after it, the knowledge lives once.

## 7. What is tested

- **The generated file equals the committed one.** This is the first test and the spine of the
  work. It is not "testing against the code's current output": `types.ts` *is* the live
  contract, so every difference is a change to the wire and has to be explained one by one
  before it is accepted. §0.5 says the expected count of systematic differences is zero.
- **A renamed Rust variant makes the check fail.** A test that renames a variant of the type
  the incident was about — `core_save::Kind` — and asserts the generated declaration changes.
  Without it, the item's whole claim rests on inspection.
- **The rewriter is a pure function with its own tests**: a union of string literals becomes the
  pair; a union with a non-string member is left alone; the PascalCase of `rep_plus` is
  `RepPlus`.
- **The existing shape tests stay untouched and stay green.** They are the control: if the
  generated contract differs from what they assert, the generator is wrong.

## 8. Out of scope, and why

- **The shape tests in `crates/ipc/tests/`.** They say *what* must appear on the wire —
  `summary_shape.rs` exists because a rename slipped through once. The generator answers
  *that* the file matches the types, never *that the types are right*. Deleting them would
  remove the only statement of intent.
- **The wrappers in `ui/src/lib/ipc/*.ts`.** They are call shapes — which command, which
  argument names — not types. Generating them is a different item with a different risk.
- **The Tauri command surface.** Same reason, one level up: nothing here reads
  `#[tauri::command]`.
- **`ui/src/lib/ipc/fixtures/`.** They are test data shaped like the types, and they fail
  loudly through `pnpm typecheck` when a type moves. No generation needed.

## 9. Done when

1. `pnpm ipc:types` regenerates `ui/src/lib/ipc/types.ts` and `git diff` is empty.
2. Changing an enum in Rust without regenerating makes `scripts/check` fail.
3. `pnpm check` is green, with the skip count unchanged from before the work.
4. `types.ts` takes no further hand edits in its subsequent history — at which point it stops
   being a file anybody opens.
