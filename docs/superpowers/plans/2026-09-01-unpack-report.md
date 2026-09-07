# unpack — Implementation report

**Date:** 2026-09-01
**Status:** DONE
**Plan:** `docs/superpowers/plans/2026-09-01-unpack.md`

---

## Commits

| Hash | Message |
|------|-----------|
| `ad6444c` | unpack: scaffold del crate, tipi pubblici e hash djb2+FNV |
| `b677f70` | unpack: test suite (unit hash, synthetic ARCH000, archivi reali) |

Note: tasks 1-6 of the plan were merged into two cohesive commits — sources and tests — because all the code (hash, arch, lzw, extract) was written and verified in a single session. `Cargo.lock` is included in the first commit.

---

## Output of the three gates

```
cargo test -p unpack
  unit:         3/3 ok
  synthetic:    3/3 ok
  real_archive: 5/5 ok
  TOTALE: 11 passed, 0 failed

cargo fmt --check
  exit 0 (nessuna diff)

cargo clippy -p unpack --all-targets -- -D warnings
  exit 0 (nessun warning)
```

---

## LZW implementation

### Reference used

**Gibbed.Rebirth** by Rick Gibbed (zlib license):
- `projects/Gibbed.Rebirth.FileFormats/LZW.cs`
- `projects/Gibbed.Rebirth.FileFormats/BitReader.cs`

### Exact variant

| Parameter | Value |
|-----------|--------|
| Bit ordering | **MSB-first** (contrary to what the plan stated as "LSB-first") |
| Initial width | **8 bit** (not 9) |
| Dictionary | 256 literals, capacity 4096 |
| Growth | `update_code_length`: increments when `count >= 1 << width` |
| Reset | Automatic when `count + 1 >= 4096`, no explicit clear-code |
| Dictionary shared across chunks | **Yes**: created once, outside the chunk loop (Gibbed) |
| Per-chunk reset | **No**: the dictionary (and `prev_idx`, `last_value`) persist across chunks |

### Deviations from the plan regarding the LZW description

The plan stated "LSB-first" and "starts at 9 bits". Both were wrong:

1. **MSB-first, not LSB-first.** Manual verification confirmed that with LSB-first at 9 bits the second code is 24 ('0'), while with MSB-first at 8 bits (then 9) it's 97 ('a'). The MSB-first sequence produces `<achiev...`, which matches the correct start of the XML file.

2. **8 initial bits, not 9.** Gibbed's `CodeDictionary` starts with `_CodeLength = 8`. The first code is read at 8 bits (a pure literal), then `UpdateCodeLength` brings the width to 9 before the second code.

3. **Dictionary shared across chunks.** The plan described "dictionary reset on every chunk?". The answer is **no**: in Gibbed the `dictionary` is allocated once, outside the block loop, and persists for the whole entry. This was the main bug found during implementation: the first chunk decompressed correctly to 1024 bytes, but the second chunk always failed because the dictionary was being reset.

### How the bug was found

1. MSB vs LSB: manual verification of the codes on the first chunk — with LSB-9bit the second code is '0', with MSB-8bit it's 'a', which matches the expected XML content.
2. Dictionary sharing: the `decompresses_achievements_xml_entry` test failed with "decompress returned None". A binary example confirmed that chunk 0 on its own decompresses correctly to 1024 bytes, but chunk 1 in isolation never decompresses. Re-reading the Gibbed code revealed that `dictionary` is created outside the `while(remaining > 0)` loop.

---

## Real-data test results

| Test | Result |
|------|-----------|
| `opens_real_config_archive` | PASS — 24 entries, `contains("resources/achievements.xml")` ✓ |
| `decompresses_achievements_xml_entry` | PASS — **25894 bytes**, first byte `b'<'` (0x3C) ✓ |
| `reads_and_decompresses_by_path` | PASS — 25894 bytes via `Archive::read`, uppercase/backslash normalization ✓ |
| `extract_subset_writes_wanted_files_and_reports_missing` | PASS — file written, `missing` correct ✓ |
| `extract_subset_refuses_paths_escaping_cache` | PASS — paths with `..` rejected ✓ |

---

## Deviations from the plan

| Point | Plan | Implemented | Reason |
|-------|-------|--------------|-------------|
| Commit structure | ~6 commits (one per task) | 2 commits (sources + tests) | Tasks 1-6 were implemented correctly in sequence but committed in cohesive blocks |
| LZW bit ordering | "LSB-first" | MSB-first | The plan was wrong; empirical verification against real data established the correct variant |
| LZW initial width | "starts at 9 bits" | 8 bits | The plan was wrong; Gibbed starts at 8 |
| Per-chunk dictionary | Ambiguous ("reset per chunk?") | Shared across chunks | Gibbed allocates the dict outside the block loop |
| File `examples/debug_lzw.rs` | Not planned | Created and removed before the commit | Used temporarily for debugging; not included in the commits |

---

## Post-review fixes

### FIX 1 (Critical) — absolute-path guard in `extract_subset`

**Problem:** the previous guard (`split(['/', '\\']).any(|c| c == "..")`) only rejected
`..` components, but an absolute Windows path (`C:\Windows\evasione.xml`), POSIX
(`/etc/evasione.xml`), or UNC path passed to `cache_dir.join(path)` would cause it to
escape `cache_dir` and write outside the cache.

**Fix:** introduced the private function `safe_join(cache_dir, rel)` in `extract.rs`
which iterates over the path's components via `std::path::Path::components()` and only
accepts `Component::Normal`. Any other component (`RootDir`, `Prefix`, `ParentDir`,
`CurDir`) causes an immediate `None` return. The function fully replaces the old `..`
check and the `cache_dir.join(path)` call.

**Unchanged behaviour:** valid paths (containing only `Normal` components) follow the
same path as before — `contains` → `read` → `create_dir_all` → `write`.

**File changed:** `crates/unpack/src/extract.rs`

### FIX 2 (Important) — no-panic tests on corrupted input

**Added 4 tests in `crates/unpack/tests/synthetic.rs`:**

| Test | Scenario | Expected outcome |
|------|----------|--------------|
| `lzw_garbage_bytes_returns_none` | stream of random bytes | `None`, no panic |
| `lzw_chunk_len_overflows_slice_returns_none` | `chunkLen = u32::MAX` with 3 bytes of data | `None`, no panic |
| `lzw_truncated_data_returns_none` | `decompressed_len = 1_000_000` on 8 bytes | `None`, no panic |
| `archive_read_with_invalid_lzw_data_returns_none` | synthetic archive with correct hashes but non-LZW data | `contains` → `true`, `read` → `None`, no panic |

**Updated** `extract_subset_refuses_paths_escaping_cache` in
`crates/unpack/tests/real_archive.rs`: now also passes `C:\Windows\evasione.xml` and
`/etc/evasione.xml` alongside `../evasione.xml`, asserts that all of them end up in
`missing`, and checks that the absolute paths were not created on the filesystem.

### FIX 3 (Important) — spec/code reconciliation on `EntryOutOfBounds`

**Decision:** align the spec with the code (not the other way around). The code silently
discards malformed or out-of-bounds index records in `Archive::open`, which is correct
(defensive degradation at open time, where there is no diagnostic channel).

**Changes to `docs/superpowers/specs/2026-09-01-unpack-design.md`:**
- Removed the `EntryOutOfBounds` variant from the `Diagnostic` enum in the public API
  block; added an explicit comment clarifying why it's absent.
- Updated the `open` Algorithm section: replaced the mention of `EntryOutOfBounds` with
  the correct description ("silently discards, no Diagnostic emitted").
- Updated the Synthetic Tests section: removed the reference to `EntryOutOfBounds`.
- Added a docstring to `Archive::open` in the public API documenting the degradation
  behaviour.

### FIX 4 (Minor) — use of `self.entries.get(i)` in `arch.rs`

**Problem:** `contains` and `read` indexed `self.entries[i]` after obtaining `i` from the
HashMap, which could cause a latent panic in case of inconsistency between the HashMap
and the Vec (theoretically impossible, but not guaranteed by the type).

**Fix:** replaced `self.entries[i]` with `self.entries.get(i)`:
- In `contains`: the chain becomes `.and_then(|&i| self.entries.get(i)).map(|e| ...)`.
- In `read`: `self.entries.get(i)?` returns `None` instead of panicking.

Behaviour unchanged for all valid cases.

**File changed:** `crates/unpack/src/arch.rs`

### Output of the three gates (post-fix)

```
cargo test -p unpack
  unit:         3/3 ok
  synthetic:    7/7 ok   (+4 nuovi test no-panic)
  real_archive: 5/5 ok   (extract_subset_refuses_paths_escaping_cache aggiornato)
  TOTALE: 15 passed, 0 failed

cargo fmt --check
  exit 0 (nessuna diff)

cargo clippy -p unpack --all-targets -- -D warnings
  exit 0 (nessun warning)
```
</content>
