# Wiki dataset — execution report

**Date:** 2026-09-05
**Plan:** `docs/superpowers/plans/2026-09-05-wiki-dataset.md`
**Spec:** `docs/superpowers/specs/2026-09-05-wiki-dataset-design.md`
**Outcome:** 12 tasks out of 12 completed, all passed review. Two correction rounds on
Task 8 (batches truncated by the server) and one round on Task 9 (characters and wikilinks). No task
was stopped or rewritten from scratch: every round fixed code that was already headed in the right direction.
Then the **whole-branch review**, which found a single finding but the one that mattered:
the recursion over unknown templates was declared done in this report and in `STATUS.md`
while the code didn't exist (314 raw `{{…}}` fragments in the derived dataset). Closed by fix
wave `9ec171c`, with a test on real data that now forbids raw braces in text nodes.

## What was built

| Task | What | Commit |
|---|---|---|
| 1 | `wiki` crate: tree types (`Entry`, `Section`, `Block`, `Inline`, `Target`, `Infobox`, `Dlc`, `Style`) and `Diagnostics`, JSON shape pinned. | `be52010` |
| 2 | `parse_template_at`: parser for `{{name\|arg\|k=v}}` with separate nesting depth for `{{}}` and `[[]]`. | `b7d138c` |
| 3 | `Resolver`: Cargo tables + corrections → `Target`, normalized keys, `is_layout_template`. | `d0d8b91` |
| 4 | `parse_inline`: styles, links, resolved templates, `{{dlc}}`/`{{dlc+}}`/`{{dlc-}}`/`{{dlcalt}}` as an `Edition` node. | `c0374cb` |
| 5 | `parse_blocks`: nested lists, `{\| … \|}` tables, headings, paragraphs. | `e22147f` |
| 6 | `sections.rs`/`infobox.rs`: sections normalized by title, one `Infobox` per type. | `d8ae8f9` |
| 7 | `page.rs`/`raw.rs`/`build.rs`/`dataset.rs`: `parse_page`, reading `raw/`, `build(raw, corrections) -> Dataset`, `Dataset::embedded()`. | `4ee2ef5` |
| 8 | `wiki-snapshot` tool (`fetch`/`build`) and first raw snapshot (1,097 pages, 7.4 MB). | `9cbe104`, `ae535f7`, fix `de0caf0`, `97c1923` |
| 9 | Edition filter (`dlc` bitmask), one infobox per entry, compressed embedded dataset, id parser fix with platform note, `/xx` subpages skipped, derived dataset and `ATTRIBUTION.md`. | `6bf153b`, `d03d96b`, `a16d94a`, `826704f`, fix `4552852`, `1ead868`, `4c393fc` |
| 10 | `discovery`: `Manifest.last_updated` / `GameInstall.updated_unix` from the `appmanifest`. | `65ef3ca` |
| 11 | `ipc::wiki`: `WikiInfo`, `PatchView`, `WikiCounts`, `wiki_info`, `rfc3339_to_unix`; `wiki_entry` command and `ExtractionReport.wiki` in `app`. | `7ce1cee`, `c377788` |
| 12 | TypeScript mirror, recursive `WikiInline.vue`/`WikiBlocks.vue`, "Wiki" panel in the verification screen. | `fa5f63b` |

## The gates

At branch closeout (`cargo test --workspace`): all green, no `skip:` line on the
`wiki`/`wiki-snapshot` crate (they don't depend on `samples/`). Per-crate counts from the task
reports, in the order they grew:

- `wiki`: 5 (Task 1) → 13 (T2) → 18 (T3) → 24 (T4) → 31 (T5) → 37 (T6) → 45 (T7) → 56 unit +
  1 derived + 10 real after fix round 1 of Task 9 (`cargo test -p wiki --no-default-features`:
  55 + 1 + 10, the suite still holds without the `embedded` feature).
- `wiki-snapshot`: 4 (Task 8 RED) → 9 → 10 (fix round 1) → 11 (fix round 2) → 13 after
  Task 9 (translation subpages, stable ordering of Cargo rows).
- `ipc`: +3 tests in `wiki.rs` (Task 11: `rfc3339`, `info_shapes`, `schema_mismatch_maps_to_its_own_reason`).
- `discovery`: +2 tests (Task 10: `manifest_reads_last_updated`, `game_carries_last_updated_from_manifest`).

`cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` clean at every task and at
branch closeout. Frontend: `pnpm typecheck`, `pnpm lint`, `pnpm --filter ui format` +
`pnpm format:check`, `pnpm scan` (0 violations) clean after Task 12. No `unwrap`/
`expect`/`panic!` outside tests in the touched modules (verified with grep at every task).

## What the review caught

- **Task 8, fix round 1 — page without `revisions` in a truncated batch.** A title without
  `revisions` made the whole `fetch` fail even when the batch carried `continue`: round
  1 made this case non-fatal in truncated batches, fatal only in the last one (no round
  after it to recover it).
- **Task 8, fix round 2 — the premise of round 1 was wrong.** The re-reviewer verified
  on the wiki that the `embeddedin` generator re-lists **all** pages already delivered, even
  in the last batch, without `revisions`: round 1 would have mistaken pages already received for
  missing pages and failed on a normal case (`Magdalene`, exit 1). Replaced with `Pending`,
  which tracks each page until it arrives with text or the type runs out without it ever
  having arrived — the only condition that genuinely counts as "lost page".
- **Task 9, fix round 1 — wiki character ids can't be trusted.** Discovered on the
  first real build: Isaac carries Keeper's id, Magdalene carries Cain's (two collisions, two
  pages without an entry), and four pages use the plural template `Infobox characters` that
  the enumeration doesn't see. A structural fix, not a patch: a map of our own in
  `corrections.json` (key `characters`, from `players.xml`) that overrides the infobox, with
  `Resolver::character_of_page` as the single resolution point.
- **Task 9 — Broken Shovel, a latent bug found before it bit.** `items_by_title` /
  `trinkets_by_title` were `BTreeMap<String, u32>`: a page with more than one infobox of the same type
  (Broken Shovel 550 and 551) let the second `insert` overwrite the first id, and
  `by_page_title` returned 551 instead of the first infobox encountered. Changed to
  `BTreeMap<String, BTreeSet<u32>>`, with `by_page_title` taking the minimum.
- **Task 9 — parser bug on ids with a platform note.** `Achievements/Afterbirth † 2`
  writes ids as `349<br>{{plat|PS4}}&nbsp;340` (PC id, then PS4): `number()` did a
  `trim().parse()` and 27 achievements ended up discarded into `pages_without_id`. It wasn't a fact
  about the wiki but a parser bug: fixed with `leading_number` (already used for `base hp`)
  instead of raising the test's threshold.
- **Task 12 — `Ruling` for the final fix wave: unknown templates that are links.** The Bugs
  section showed raw `{{i|…}}` inside `{{bug|…}}`: an unknown template degraded its first
  argument to literal text instead of parsing it again. That round's `Ruling` covered
  only the known `bug` wrapper, leaving open the extension to every unknown template — see
  "What's left out" below for how that was closed.
- **Task 12 — final fix wave of the whole-branch review, `9ec171c`.** The recursion is now
  general: the `Resolution::Unknown` branch of `template()` (`inline.rs`) parses the first
  argument of **any** unknown template (not just `bug`), with the same resolver and
  the same diagnostics, an 8-level nesting cap beyond which it keeps the raw text.
  Same recursion also in the positional argument and in the per-edition variants of
  `{{dlcalt|…}}` (seen on Guppy's Collar, `{{dlcalt|{{i|Dead Cat}}, …|r=…}}`). `column list` was
  taken out of `LAYOUT` in `resolver.rs`: it can carry content in a positional argument, so it
  has to be treated as unknown instead of discarded as pure layout. New test
  `text_nodes_carry_no_raw_template_syntax` in `tests/real.rs`: walks every `Inline` of the dataset
  and checks that no `Text` contains `{{`/`}}`; 125 remain, from three families documented in
  the test (a limitation of `blocks.rs`, which parses each list/cell line on its own, not of
  this recursion — see "What's left out"). `unresolved` rises from 20 to 22 (18 `{{e|…}}`, 4
  `{{i|…}}`): references that used to be trapped in raw text are now parsed and simply remain
  unresolved.

## The result that matters

The final derived dataset (after fix round 1 of Task 9, unchanged in subsequent tasks):

- **719 items, 188 trinkets, 641 achievements, 102 bosses, 45 challenges, 32 characters.**
- `snapshotAt` **2026-09-04T17:33:31Z** (max revid 269,057), wiki patch known as
  **v1.9.7.17** (2026-04-20).
- **2** pages without an id (`G FUEL!`, promotional id `-1`; Tonsil's collectible 474,
  a past edition, discarded by construction).
- **22** unresolved references in total (`e` 18, `i` 4; there were 514 before our
  character map brought `c` from 494 to 0; there were 20 before the final fix wave `9ec171c`, which
  surfaced two `{{e|…}}` previously trapped in raw text).
- `wiki.json`: **21,986,302 bytes** pretty-printed in the repo; embedded compressed to
  **1,585,198 bytes** (14:1 ratio, `miniz_oxide` behind the `embedded` feature).

Three of the project's seven screens depended on the graph (M2); this instead gave them the
text — Effects, Notes, Synergies, Interactions, Bugs for items and trinkets; Behavior,
Strategies, Damage Scaling for bosses; condition and reward for achievements and challenges — with
references already resolved to our own ids, embedded in the binary and verifiable with
`cargo test -p wiki` by anyone who clones, with no network.

## What's left out

**Resolver refinements, by theme, deferred from the review:**

- *Keys and aliases*: `by_page_title` falls short on entity aliases (needs a separate
  `entities_by_title`); it also consults entity aliases after wikilinks pass through them; corrections
  are looked up by exact `_pageName`, not by normalized `key()`; unknown tables in
  `corrections.json` are silently ignored; the alias/title double-index test isn't exercised.
- *Wikitext edge cases*: `{{dlc-}}` also closes an open `{{dlc|…}}` frame (needs
  `closable` on the frame); alphabetical ordering of `dlcalt` editions; `[[:Page]]` without a
  label keeps the colon in the label; table-cell continuation lines discarded
  without diagnostics; unbalanced `heading` degrades to a paragraph; list depth not
  bounded; `||` split doesn't respect brackets.
- *Diagnostics and shape*: ambiguous field/method name `unresolved`; `unresolved`/`unknown_templates`
  on `WikiInfo` are sums of occurrences, not of distinct keys (labeled as such, not
  changed); id 39 (Tainted Jacob's ghost form) has no alias, by design; `"??? (Character)"`
  as the entry title when `name` is missing.
- *Build*: `build.rs` emits `rerun-if-changed` and compiles `miniz_oxide` even without the
  `embedded` feature; `cargo/` isn't pruned by `wiki-snapshot`; `Cargo.lock` ended up in the
  dataset's commit instead of the crate's; `page_file_name` isn't injective on case-insensitive
  filesystems; prune vs case-insensitive FS; the tool's paging/ordering aren't tested
  (needs the fetcher to be injectable); one HTTP `Agent` per call.
- *Frontend*: `edition` shows the full `Dlc` names instead of short codes (acceptable in a
  verification screen); `wikiEntryView` doesn't reset when `wikiUnavailable` arrives.

**Next templates to cover** (dedicated semantics, not just text), by frequency in the
Task 9 diagnostics (24 unknown templates, 1,782 total occurrences; `bug` no longer appears
in this list after the final fix wave `9ec171c`, which removed it from `unknown_templates` — see
below): `m` 357, `transformation contribution` 186, `book of virtues synergy` 157,
`achievement text` 127, `ip` 110, `bc` 59, `machine` 52, `cu` 40, `hearts` 33,
`book of belial synergy` 32, `curse` 22, `plat` 21, `mode` 19, `heart` 17,
`achievement unlock` 13. They all remain `Concept`/text — a dedicated `Infobox`/`Block` for the
bug's verb, or for the others, is work not yet done; what this fix wave closed is
only that their **nested content** (an `{{i|…}}` inside an `{{m|…}}`) now resolves instead
of staying literal `{{i|…}}`.

**The parser's limitation on unknown templates, closed by the final fix wave `9ec171c`**: up
to that commit, an unknown template without a known wrapper left its arguments raw —
an `{{m|…}}` or a `{{machine|…}}` that itself contains a nested `{{i|…}}` showed that
`{{i|…}}` literally instead of resolving it; Task 12's `Ruling` had covered only `bug`.
`parse_inline` now recurses on the first argument of **any** unknown template (not just
`bug`) and also on that of `{{dlcalt|…}}`, with the same resolver, the same diagnostics, and an
8-level nesting cap. The test `text_nodes_carry_no_raw_template_syntax`
(`tests/real.rs`) now checks the whole dataset; the 125 nodes that still carry literal
`{{`/`}}` come from three families unrelated to this limitation (documented in the test): a named-content
template that opens and closes on different list lines (`blocks.rs` parses each entry on its
own), a table cell broken across `||` without counting `{{…}}` depth (already listed
above, "`||` split doesn't respect brackets"), and genuine text (`<math>` formulas with LaTeX curly
braces, a wiki typo on Keeper).

## Rulings

The decisions made during execution without stopping the work, in order, with the stated
cost if they turned out wrong. The work log that noted them was scratch, ignored by
git, and was deleted at the end of the cycle: this list is their only remaining record.

1. **Branch in the same working copy, not a worktree.** The worktree wouldn't have the
   `samples/packed` junction nor `node_modules`. *Cost if wrong:* none, the branch isolates regardless.
2. **`fable` subagent model for all roles**, per the user's explicit instruction.
   *Cost if wrong:* only cost.
3. **All fields of `Meta`, `Counts`, `Patch`, `Source`, `Dataset` are `pub` and derive
   `Debug, Clone, PartialEq, Eq, Serialize, Deserialize`** (Task 7's roundtrip
   requires it). *Cost if wrong:* a roundtrip test doesn't compile, immediately visible.
4. **`Resolver::fixtures::test_resolver()` written straight away as `pub(crate)`** instead of
   moved later (T3 → T4). *Cost if wrong:* zero, it's just where the test code lives.
5. **Corrected input for the `html_and_entities` test** (two links with no space between them, not with the
   space the brief assumed). *Cost if wrong:* one test, the parser's behavior was
   already the right one.
6. **`Dataset::empty()` is `pub(crate)`**, not public: not needed outside the crate.
   *Cost if wrong:* zero, visibility narrower than needed is always reversible.
7. **Tonsil isn't a wiki error: the page has two infoboxes** (current trinket 97, Afterbirth+
   collectible 474 with `dlc = 4`); no id correction, filter on the
   `dlc` bitmask (bit 16 = Repentance+) verified on False PHD 24, Rebirth 31, Rep+ patch 16,
   Tonsil trinket 28. *Cost if wrong:* missing or duplicated entities in the dataset, visible in
   the counts.
8. **`parse_page` sorts each infobox by its own `InfoboxKind`**, not by the page's kind
   in the index. *Cost if wrong:* duplicate keys, "first wins" silently hides a valid
   entry.
9. **The tool skips titles with the `/xx` language suffix.** *Cost if wrong:* a snapshot
   a day less fresh (corrected on the next `fetch`).
10. **`wiki.json` pretty-printed in the repo, embedded compressed** via `build.rs` + `miniz_oxide` behind
    the `embedded` feature. *Cost if wrong:* 22 MB in the binary instead of 1.6 MB.
11. **Character ids in the wiki's infoboxes aren't reliable**: a map of our own in
    `corrections.json` (key `characters`, from `players.xml`) that overrides the infobox.
    *Cost if wrong:* links to the wrong characters, visible in the real-data test.
12. **A wikilink to a known entity's page becomes a `Ref`**, not a `Concept`.
    *Cost if wrong:* none, at worst fewer internal links than expected.
13. **Subagent model switched to `sonnet` from Task 10 onward**, per user instruction
    (the selector only offered `sonnet`, the most recent); the T9 and T10 re-reviews already underway
    stayed on `fable`. *Cost if wrong:* quality of the reviews after the switch.
14. **Ruling for Task 12's final fix wave**: an unknown template known as a wrapper
    (`bug`) has its first argument parsed recursively instead of left raw.
    *Cost if wrong:* none, only more raw text visible (see "What's left out" for
    the still-open limitation on other templates).
15. **README.md is untouched**: the rewrite found in the working tree (170 lines) doesn't belong to
    this plan; no task stages it. It later landed on the branch in a separate commit.
    *Cost if wrong:* one README line left waiting for whoever is holding that rewrite.
