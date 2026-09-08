# catalog — plan A, implementation report

**Date:** 2026-09-03
**Plan:** `docs/superpowers/plans/2026-09-03-catalog-a.md`
**Spec:** `docs/superpowers/specs/2026-09-03-catalog-design.md`
**Outcome:** 10 tasks out of 10 completed, all passed review.

## What was built

| Task | What | Commit |
|---|---|---|
| 1 | Scaffold of the `catalog` crate: id newtypes (`ItemId`, `CharacterId`, `AchievementId`), typed `Diagnostic`/`Source`/`SkipReason`, `Catalog::build(read)` which queries every source and never fails. | `854aa5f` |
| 2 | `xml.rs`, the single layer over `quick-xml`: `Element { name, attrs, text, comment_before, depth }`, `elements(bytes) -> Result<Vec<Element>, XmlError>`. | `854aa5f..233e8dd` |
| 3 | `text.rs` (`Text::Key`/`Literal`, `Language`, 8 languages) and `strings.rs` (stringtable as XML); `Catalog::text` which never returns `None` (requested language → English → key). | `233e8dd..482df86` |
| 4 | `sprite.rs` (`Rect`, `SpriteRef`) and `items.rs`: `items.xml` with type, sprite from `gfxroot`, quality, tags and link to the achievement. Map key `(ItemKind, ItemId)`. | `482df86..cbe1f99` |
| 5 | `players.rs`: `players.xml` with portrait from `portraitroot` and Tainted-form recognition. | `cbe1f99..cf076ec` |
| 6 | `heads.rs` and the anm2 spike: frame → head sprite map, written and verified by eye (37 out of 41 characters). | `cf076ec..7817e87` |
| 7 | Real-data tests (`crates/catalog/tests/real_data.rs`): found and fixed a real Tainted-recognition bug, and found wrong measured numbers in the brief (911 instead of 909). | `7817e87..948be98` |
| 8 | `ipc::catalog_view`: catalog view-model; removed `ipc::resources::catalog_peek` and its text parser; bridge from matrix row ↔ `players.xml`. | `948be98..f2185e7` |
| 9 | Catalog built once in `CatalogState` (Tauri managed state); verification screen with real names instead of keys. | `f2185e7..dbb5245` |
| 10 | This report; aligned `docs/STATUS.md`, `DESIGN-BRIEF.md` and the spec with the real numbers that came out of Task 7; renamed the `all_item_names_resolve` test. | `dbb5245..a83fff8` |

**Final gate:** `cargo test --workspace` → **156 tests, 0 failed** (42 of them in the
`catalog` crate: 28 unit, 7 on `build.rs`, 7 on real data). `cargo fmt --check` and
`cargo clippy --all-targets -- -D warnings` clean.

### The result that matters

The catalog read from the file that wins precedence (`items.xml` from `repentance.a`)
gives **909 objects — 425 passive, 170 active, 126 familiars, 188 trinkets** — and every
sprite resolves in the archives. All name keys resolve in the stringtable, with no
exceptions. 41 characters, 37 of which have a head cropped from the `coop menu.png` sheet
(map: id 0-19 → frame id+1, id 21-37 → frame id); Esau and the three formless forms remain
`head: None`, diagnosed, not an error.

## What review caught

| Task | Finding | Fate |
|---|---|---|
| 2 | Important: no tests on numeric references in text, unknown entity, whitespace between children. | Fix (round 1): 4 tests added, no code change — the behaviour was already correct, just unprotected. |
| 2 | Minor: `Text`/`GeneralRef` outside the root silently discarded instead of `Err`. | Deferred: no game XML exercises this. |
| 2 | Minor: mixed text+children content does not collapse internal whitespace. | Deferred: no game XML uses this form. |
| 3 | `Strings::len`/`is_empty` with no callers in any plan-A or plan-B task. | Removed (YAGNI): the test that used `len() == 2` now instead asserts that a missing key gives `None`. |
| 3 | Minor: no test with an unknown language IN THE MIDDLE of the indices. | Deferred: code correct on inspection (lookup by value, not by offset), gap inherited from the brief. |
| 3 | Minor: `unwrap_or(0)` on `position` in `strings.rs`. | Deferred: rationale confirmed by the reviewer, stays as is. |
| 4 | Important: `gfxroot=""` (attribute present but empty) produced a broken path (`/collectibles/a.png`), violating the "logical path" contract; untested. | Fix: an empty root after normalization counts as absent → default `gfx/items`, with a test on `normalize_root`. |
| 4 | Minor: no direct test on the `resources/` prefix and on backslashes in `normalize_root`. | **Resolved** (final review): `normalize_root_strips_the_archive_prefix_backslashes_and_trailing_slashes`. |
| 4 | Minor: no test on the ordering of `items()`. | Deferred: guaranteed by construction via `BTreeMap`, verified by hand by the reviewer. |
| 6 | Minor: `dump_heads` prints the frame count to stdout before the PowerShell script. | Deferred: throwaway spike, line to be removed by hand from the `.ps1`. |
| 7 | Real bug: character 38 (revived Tainted Lazarus) has portrait `PlayerPortrait_Lazarus_b_dead.png`; the `ends_with("_b.png")` rule missed it. | Fix: recognition on the `_b` token in the portrait name, not just on the suffix. |
| 7 | The brief's expected numbers (426/171/126/188, two unresolved keys) were a measurement error: a `grep` on the text counted two elements inside an XML comment that `quick-xml` correctly ignores. | Fix: values corrected in the test (425/170/126/188 = 909, 0 unresolved), with a comment explaining the discrepancy; propagated to the documents in Task 10. |
| 7 | Minor: the `names_resolve_except_two_pickup_placeholders` test no longer has any exceptions to name. | Renamed to `all_item_names_resolve` (Task 10). |
| 9 | Minor: `CatalogState` caches `None` for the lifetime of the process if the game is missing on the first attempt. | Deferred: plan-mandated, commented in the code; to revisit once an "install and recheck" flow exists. |

Tasks 1, 5 and 8 passed review with no findings at all.

## Decisions made during execution

- **`#[allow(dead_code)]` as a bridge, later narrowed.** In Task 2, `xml.rs` had no
  consumers yet: a module-level `allow` kept `-D warnings` green for a single task. Task
  3 (the first consumer) removed it; a targeted `allow` remains only on the
  `Element::comment_before` field, which in production is read only by `achievements.rs`
  in plan B, with a comment saying so.
- **Item map key `(ItemKind, ItemId)`, not just `ItemId`.** Trinkets and passives can
  share the same numeric id in the file; the composite key makes this explicit instead of
  silently colliding entries.
- **An empty root counts as absent.** `gfxroot=""`, once normalized, gives an empty
  string, which Task 4 treats as "absent → default `gfx/items`": a single rule, simpler
  than the conditional branch on the slash proposed during review, and it also covers
  `gfxroot="/"` and `gfxroot="resources/"`.
- **Minimal bridge in `app` between Task 8 and 9.** Task 8 changes the signature of
  `ipc::extraction_report` and removes `catalog_peek`, but the plan didn't anticipate
  `app` failing to compile in the middle: the minimal change (`None` as the catalog, no
  sprite) kept `cargo test --workspace` green for a commit that was never pushed on its
  own.
- **Tainted from the `_b` token, not from the suffix.** Found by the real-data test (Task
  7): `PlayerPortrait_Lazarus_b_dead.png` has `_b` followed by more text, not at the end
  of the name. The rule becomes "the portrait name contains the `_b` token", not "ends
  with `_b.png`".
- **Numbers corrected from the parser, not from the brief.** When an expected number in a
  real-data test failed, the dispatch rule was: investigate the code, never blindly
  change the value. The investigation (Task 7) showed that the brief had a measurement
  error — a `grep` that counted elements inside an XML comment — not the parser. The
  expected values were corrected to what the parser actually produces (425/170/126/188 =
  909 objects, 0 unresolved keys), with the explanation written into the test; fixing the
  documents that cited 911 is this task's job.
- **No worktree for the plan.** The real-data tests depend on `samples/packed`, a
  git-ignored junction that a worktree would not have: work was done in the main checkout
  on `develop`, with all intermediate commits reversible and no push until the end of the
  whole branch's review.

## What's missing

- **Plan B** (`docs/superpowers/specs/2026-09-03-catalog-design.md`, "Plans" section):
  `achievements.rs` with comments and unlock links, `itempools.rs`, `challenges.rs`,
  `bossportraits.rs`, `origin` (source DLC by id thresholds), cross-check between the
  catalog and section 4 of the save file.
- **The deferred Minor items** listed above: `Text`/`GeneralRef` outside the root, internal
  whitespace in mixed content, the stdout line in `dump_heads`, the missing test on the
  ordering of `items()`. The test on `normalize_root`, the unknown language in the middle
  of the indices, and `unwrap_or(0)` on `position` were closed in the final review (see
  below); `CatalogState` doesn't rebuild the catalog if the game changes after the first
  successful start (no longer "caches `None` forever": now the constructor never fails,
  but it remains a `OnceLock` that doesn't retry).
- **The `Archive::open` bottleneck**, inherited from `unpack`: it reads the whole archive
  into memory, and `extraction_report` does so on every call. The catalog is now built
  only once, but sprites are still extracted by reopening everything.

## Final review

A review of the whole branch (not just plan A) produced four Important findings, all
closed in a single pass:

| # | Finding | Resolution |
|---|---|---|
| 1 | `extraction_report` opened two `ResourceSet`s (~1.3 GB in memory each): one in the command, one inside `CatalogState::get_or_build`, which discovered and opened the game on its own — and its internal `discover()` was unreachable anyway, because the command bails out early if the game is missing. | `CatalogState::get_or_build(&self, rs: &ResourceSet)` now builds with `Catalog::build(\|p\| rs.read(p))` from the set the command has already opened; no `discover()` in the state, a single `ResourceSet::open` per call. `crates/app/src/lib.rs`. |
| 2 | `Diagnostic::UnresolvedKey` existed in the enum but no code ever produced it: keys with no string stayed invisible to diagnostics. | `Catalog::build` calls `diagnose_unresolved_keys` after reading items and players: one `UnresolvedKey` per distinct key (not per element) among names, descriptions and character names that don't resolve in any stringtable language. `ipc::catalog_view` counts diagnostics instead of recomputing with a local `is_unresolved`, which was removed. Three new tests in `crates/catalog/tests/build.rs`; the `crates/ipc/tests/catalog_view.rs` fixture was completed with the missing descriptions so the `unresolved_names == 1` assertion would still hold once descriptions are counted too. |
| 3 | `crates/ipc/tests/character_bridge.rs` only checked "34 rows, 34 distinct characters": a shift of the two tables (`CHARACTERS`, `CHARACTER_KEYS`) relative to each other would have passed all the same, because the rows stay distinct even if shifted. | New test `matrix_row_anchors_point_at_the_expected_character_ids`: five anchors verified against the real catalog (row 0 "Isaac" → id 0, row 14 "The Forgotten" → 16, row 16 "Jacob & Esau" → 19, row 17 "T. Isaac" → 21, row 33 "T. Jacob & Esau" → 37). |
| 4 | The spec (`docs/superpowers/specs/2026-09-03-catalog-design.md`) still described the superseded model: `items: BTreeMap<ItemId, Item>` instead of the composite key, `Character` without the `tainted` field, `Text` as a struct instead of the `Key`/`Literal` enum, and a test on "the `items.xml` winner contains all base ids" presented as existing when it doesn't. | Spec updated on all four points, leaving the rest untouched (the plan-B sections stay as described, since they haven't been built yet). |

Verification after closing these out: `cargo test --workspace` green (see the separate
report `final-fix-report.md`), `cargo fmt --check` and `cargo clippy --all-targets -- -D
warnings` clean, real-data tests run with no `skip:`.
</content>
