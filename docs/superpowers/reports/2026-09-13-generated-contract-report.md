# A contract that is generated, not mirrored — report

**Branch** `feature/generated-contract`, cut from `develop` on 2026-09-13 after
`feature/background-and-tray` merged. Spec
`docs/superpowers/specs/2026-09-13-generated-contract-design.md`, plan
`docs/superpowers/plans/archive/2026-09-13-generated-contract.md`. This is **N7** in
`docs/STATUS.md` and **B2** in `docs/IMPROVEMENTS.md`. `scripts/check` green on the merged
result: 1281 real files touched, 7 skips, all named and all for samples this machine does not
have.

**What it does.** `pnpm ipc:types` writes `ui/src/lib/ipc/types.ts` from the Rust types, and
`scripts/check` fails when the committed file and those types disagree. 865 hand-written lines
that mirrored the `#[serde]` attributes are gone; what is left is an ordered list of the types
that cross the boundary, which is now the only place that says what crosses.

**What it does not do.** It does not generate the command wrappers in `ui/src/lib/ipc/*.ts`,
which are call shapes rather than types, and it does not touch the shape tests in
`crates/ipc/tests/`: they say *what* must appear on the wire, which is a question the generator
never answers.

---

## 1. The shape, and the two parts of it that were decided rather than defaulted

A binary, `crates/ipc/src/bin/ipc-types.rs`, thin like `graph-rules.rs`; the logic in
`ipc::contract`, where it can be tested. `ts-rs`'s native mode — bindings written as a side
effect of `cargo test` — was rejected so the suite never writes into the working tree.

Two decisions came from reading the vendored source rather than the README:

- **`u64` and `i64` would have arrived as `bigint`.** `impl_large_integers!` gives them the
  configured `large_int_type`, whose default is `bigint`, while `usize` maps to `number`. The
  boundary carries `i64` timestamps, and **JSON has no bigint**: `serde_json` writes a number.
  `Config::with_large_int("number")` is therefore a statement about the wire, not a preference,
  and `a_unix_timestamp_is_a_number_and_never_a_bigint` keeps it.
- **One file, assembled by the caller.** `TS::decl()` returns a declaration without imports and
  `TS::docs()` the JSDoc block; the crate's own `generate_decl` concatenates them and is
  private, so the binary does those two calls itself. That is what makes the file's order ours,
  and a readable diff is the only reason the reconciliation below could be done at all.

## 2. What the reconciliation found

The plan said to go through every difference between the generated file and the committed one,
accept three named kinds and stop on anything else. Four things were none of the three, and each
is a defect that no test could see — which is the argument B2 had been making since 2026-09-05,
arriving as evidence.

| | |
|---|---|
| `SectionCount.kind` | typed `string`; it is `Kind`. The incident `CLAUDE.md` records, closed by construction |
| `GameView.edition` | typed `string`; it is `Edition` |
| `GameView.dlcs` | typed `string[]`; it is `InstalledDlc[]` |
| `Infobox` | **had no `transformation` variant at all** |
| `ProgressMark` | exported only through `pub mod for_tests`, while being a field of `SearchHit` |

The fourth is the one with a user in front of it. `wiki::Infobox` has had a `Transformation`
variant since the transformations landed earlier the same day; the hand-written mirror never
learned about it, so `WikiInfobox.vue`'s `switch` was exhaustive **only because the type lied**,
and a transformation page's infobox reached `assertNever`, which throws. The screen now draws no
card for that kind and says so on the spot; the rows are **B40**.

The fifth is a correction to N1, which had moved `ProgressMark` behind `for_tests` reading it as
the return type of a test-only call. A type reachable only from `for_tests` cannot be named by
the list of what crosses — that is how it surfaced.

## 3. The item's count of foreign types was wrong, in the direction that mattered

B2 named five types crossing from crates that are not `ipc`. There are **nine**: `wiki`
contributes `SectionKind`, `CollectibleTemplate` and `Style` beside `Target`, and
`core_save::marks::CharacterGroup` is the ninth.

This is the same shape as N2's "the item named `StoreReason`; there were four" and N1's "the
survey found four entry points; there were seven". The difference is that here it cost nothing:
having chosen to derive on the real types, the compiler named the missing four in one pass. The
alternative the spike had weighed — declaring the foreign types by hand in the post-step — would
have left four of them hand-written and silent instead of one.

## 4. One name collision, and one decision it forced

`discovery::Dlc` and `wiki::Dlc` are different enums with the same name: the DLCs found in the
install, `snake_case`, against where a piece of content comes from, `camelCase` and including
`Rebirth`. Two crates may both call a type `Dlc`; one TypeScript file may not declare it twice.

The installed one is `InstalledDlc` in TypeScript, through `#[ts(rename)]`, which does not touch
serde and therefore does not touch the wire. The frontend's existing `Dlc` — the one
`dlcNames.ts` and the facet labels compare against — keeps its name and its meaning.

## 5. Two things the plan had wrong, corrected in flight

- **The equality of generated and committed cannot be a Rust test.** The committed file is
  prettier's output and a Rust test cannot run prettier. The gate in `scripts/check` is where
  that assertion can live; the spec was amended before the plan was written, rather than left
  to disagree with it.
- **The scratch copy cannot live in the system temp.** Prettier resolves its configuration by
  walking up from the file, so a copy anywhere outside `ui/` is formatted by other rules and
  reads as stale forever — and on Windows prettier cannot open an MSYS `/tmp` path at all. The
  scratch is `ui/.ipc-types-check/`, gitignored, removed by the script's `trap`.

## 6. The gate was made to speak before its silence was trusted

Checked with `#[serde(rename = "bestiaryy")]` on one variant of `core_save::Kind`, **not** with a
renamed variant: a renamed variant fails to compile, which proves nothing about a check. A serde
rename changes the wire and leaves every Rust call site compiling — which is exactly the shape of
the incident this item exists for — and it failed `cargo-test` and `ipc-types` together, with the
diff naming the line.

## 7. Two couplings to live with

- **A `///` on a wire type is UI source.** It becomes JSDoc in a file under `ui/src`, so it
  obeys `pnpm scan`: `MissingReason` lost an arrow, because the app's font has no glyph for it
  and the rule does not read comments differently from anything else.
- **`ts-rs` prints one permanent warning.** It does not parse `#[serde(transparent)]` and says
  so on every build. Left standing rather than silenced with the `no-serde-warnings` feature:
  the attribute is a no-op on a newtype in serde too, the generated output is right and pinned
  by a test, and the feature that hides this line would hide the next one as well.

## 8. What this unblocks

**M4 sub-project 1**, which is the next item in *Next up*'s order and was the whole argument for
running N7 before it: every view-model M4 adds to the contract would otherwise have been
hand-mirrored into `types.ts` and then regenerated.
