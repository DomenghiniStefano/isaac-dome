# core-save Parser Implementation Report

**Date:** 2026-09-01  
**Reference plan:** `docs/superpowers/plans/2026-09-01-core-save-parser.md`  
**Status:** DONE_WITH_CONCERNS

---

## Commits

| Hash | Message |
|------|-----------|
| `d1beb81` | core-save: scheletro del crate e workspace |
| `f9eb7be` | core-save: parser completo, accessori, diff, integration test (verde e clippy-clean) |

---

## Output of the three gates

### `cargo test -p core-save`

```
running 6 tests (unit)
test parse::tests::reads_unknown_0x10 ... ok
test section::tests::bytes_per_entry_matches_format ... ok
test parse::tests::rejects_too_short ... ok
test parse::tests::rejects_bad_magic ... ok
test section::tests::number_and_from_number_are_inverse ... ok
test smoke::crate_compiles ... ok

test result: ok. 6 passed; 0 failed; 0 ignored

running 8 tests (degradation)
test bestiary_accessor_returns_raw_bytes ... ok
test bestiary_extends_to_end_ignoring_header_count ... ok
test accessors_expose_typed_views ... ok
test flags_an_unexpected_kind_but_keeps_going ... ok
test flags_a_section_that_overruns_the_buffer ... ok
test diff_reports_newly_set_flags_and_changed_counters ... ok
test flags_trailing_bytes_before_checksum ... ok
test parses_well_formed_sections ... ok

test result: ok. 8 passed; 0 failed; 0 ignored

running 5 tests (real_saves)
test achievement_count_is_read_from_file_not_hardcoded ... ok
test bestiary_length_follows_bytes_not_header_count ... ok
test diff_2025_to_live_is_coherent ... ok
test real_save_has_ten_sections_in_order ... ok
test stable_section_counts_match_the_format ... ok

test result: ok. 5 passed; 0 failed; 0 ignored

Doc-tests: 0 tests, 0 failures
```

**Total: 19/19 PASS, 0 ignored, 0 skipped.**

### `cargo fmt --check`

Exit code 0 — no differences.

### `cargo clippy -p core-save --all-targets -- -D warnings`

```
Checking core-save v0.1.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.31s
```

Exit code 0 — no warnings.

---

## Real tests against the samples (PASS, not skipped)

| Test | Result | Notes |
|------|-----------|------|
| `real_save_has_ten_sections_in_order` | PASS | 10 sections in order, `diagnostics` empty |
| `achievement_count_is_read_from_file_not_hardcoded` | PASS | 2025: count=641, live: count=642 |
| `stable_section_counts_match_the_format` | PASS | All counts verified: Counters=523, Items=733, etc. |
| `bestiary_length_follows_bytes_not_header_count` | PASS | Bestiary length taken from bytes up to `fine-4`, not from `count`; aligned to 8 bytes |
| `diff_2025_to_live_is_coherent` | PASS | achievement 641 present in the diff; counters changed (list non-empty) |

No test printed "skip" — both samples were present in `samples/`.

---

## Deviations from the plan

### 1. `Diagnostic` serde tag: `tag = "kind"` → `tag = "type"` (fix for an error in the plan)

**Problem:** The plan wrote `#[serde(rename_all = "snake_case", tag = "kind")]` on the `Diagnostic` enum, but one of the variants already has a field named `kind` (`SectionOverrun { kind: u32, … }`). Serde rejects this combination with a compile error:

```
error: variant field name `kind` conflicts with internal tag
```

**Fix applied:** Changed the serde tag from `"kind"` to `"type"`. The public signature of the Rust enum did not change; only the key in the serialized JSON changes (irrelevant until there is a consumer of the JSON).

**Rationale:** A minimal, idiomatic choice with no impact on tests or the Rust API. The `"type"` key is conventional for tagged JSON unions.

### 2. `UnexpectedKind` logic for the bestiary (bug fix in the plan)

**Problem:** The plan implements the check `if found_kind != expected { diagnostics.push(UnexpectedKind) }` before resolving the kind. The test `bestiary_extends_to_end_ignoring_header_count` builds a file with only the bestiary section (kind=10) when `expected=1`, and then asserts `diagnostics.is_empty()`. The two are contradictory: the plan's code would have produced an `UnexpectedKind` for the bestiary, making the test fail.

**Fix applied:** The `UnexpectedKind` check is exempted for the bestiary (the kind whose `bytes_per_entry() == None`). The condition becomes:

```rust
if found_kind != expected && kind.bytes_per_entry().is_some() {
    diagnostics.push(Diagnostic::UnexpectedKind { … });
}
```

**Rationale:** The bestiary is always the last section of the file; its position is unambiguous regardless of the value of `expected`. The exemption is semantically correct and doesn't affect any other test (the test `flags_an_unexpected_kind_but_keeps_going` uses kind=5, which has `bytes_per_entry() == Some(1)`, so it still produces diagnostics).

### 3. Clippy `manual_is_multiple_of` (lint fix)

The plan wrote `bestiary.bytes.len() % 8 == 0` in the real-data test. Clippy ≥1.95 rejects this under `-D warnings`. Fixed to `bestiary.bytes.len().is_multiple_of(8)`.

### 4. `cargo fmt` — module ordering in `lib.rs`

`cargo fmt` reordered the `mod` declarations alphabetically (`diff`, `parse`, `section`) and the `pub use` statements accordingly. No functional impact.

---

## Non-negotiable constraints — verification

| Constraint | Status |
|---------|-------|
| No write API in the crate | ✓ — no function or method modifies bytes |
| No hardcoded counts | ✓ — only the `bytes_per_entry` table is static; every count is read from the header |
| No `unwrap`/`expect`/`panic!` on untrusted input in `src/` | ✓ — all byte accesses stay within the bounds checked by the loop; tests legitimately use `unwrap` |
| `Save::open` = read-only (`std::fs::read`) | ✓ |
| `serde::Serialize` on every model type, no Tauri | ✓ |
| `OpenError` — no `Serialize` | ✓ |
| Bestiary: length = `fine-4 - offset`, not `count × 8` | ✓ — verified both by the synthetic test and the real-data test |
