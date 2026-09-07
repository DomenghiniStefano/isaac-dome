# catalog — plan B, implementation report

**Date:** 2026-09-04/05
**Plan:** `docs/superpowers/plans/2026-09-04-catalog-b.md`
**Spec:** `docs/superpowers/specs/2026-09-03-catalog-design.md`
**Outcome:** 7 tasks out of 7 completed, all passed review.

## What was built

| Task | What | Commit |
|---|---|---|
| 1 | `SOURCES` with nine entries (`Source` from 4 to 9 variants: `Metadata`, `Achievements`, `ItemPools`, `Challenges`, `BossPortraits`), `ChallengeId`/`BossId`, `path_of` with an exhaustive match, `fetch(Source, ..)` as the single read entry point. | `e4866a7` |
| 2 | `metadata.rs`: `items_metadata.xml` is the real source of quality and tags, not `items.xml`. `Item.quality` changed from `Option<u8>` to `Option<i8>` (the real scale runs from −1 to 4); `<item>` metadata indexed under `ItemKind::Passive` with an Active/Familiar → Passive lookup. | `2b35c1d` |
| 3 | `achievements.rs`: `Achievement { id, text, unlock_condition, sprite }`, unlock condition read from the element's `comment_before` (raw, for M2); removed the `#[allow(dead_code)]` on `Element::comment_before` in `xml.rs`, which now has a production reader. | `14466ee` |
| 4 | `itempools.rs`: `Pool`, `PoolEntry`, `PoolMembership`; `Item.pools: Vec<PoolMembership>`, written against the first `ItemKind` (Passive/Active/Familiar) in which the id exists. `Item` loses `Eq` (the weights are `f32`). | `abf9eb3` |
| 5 | `challenges.rs` (challenges with an achievement list and starting items) and `bossportraits.rs` (portrait + unlock link); removed the leftover placeholder loop, all nine sources are now read explicitly. | `0f14849` |
| 6 | `origin.rs`: `Edition`, `edition_of(kind, id)`, id thresholds for Rebirth/Afterbirth/Afterbirth+/Repentance verified by hand against the winning file before being written down. | `5b6337a` |
| 7 | Eight tests on real data in `real_data.rs`: cross-check against the save, achievements, bosses, sprites, origin, quality, pools, challenges. Found and fixed a real bug in the challenge parser; corrected four wrong measurement numbers in the brief and propagated the fix to the spec. | `c0582ef`, `6ba74d3`, `3506cdd`, `36b395a` (+ `7c0798a`, `b534bd7` spec-only) |

**Final gate:** `cargo test --workspace` → **196 tests, 0 failed** (of which 81 in
the `catalog` crate: 51 unit tests, 15 in `tests/build.rs`, 15 in `tests/real_data.rs`).
`cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` clean.

### The result that matters

The catalog now covers **all** the static data that can be derived from the game files: 909
items with name, sprite, quality, tags, pools and DLC of origin; 637 achievements (283 with
a readable unlock condition) linked to 370 items via `achievement="N"`; 45 challenges with
the full list of linked achievements; 103 boss portraits (101 resolvable in the archives).
The cross-check with `core-save` confirms that the catalog's ids are a subset of the save's
slots: 733 item slots, 721 collectibles in the catalog, 11 unused ids identified one by one.

## What review caught

| Task | Finding | Outcome |
|---|---|---|
| 2 | Minor: `#[derive(Default)]` on `Metadata` with no use anywhere in the crate. | Deferred. |
| 3 | Minor: an inline comment is missing on `text: unwrap_or("")` (a deliberate choice, not `MissingName`). | Deferred. |
| 3 | Minor: missing a test for "a comment before the root does not reach the first achievement". | Deferred. |
| 4 | Minor, plan-mandated: an id present in two `ItemKind`s would take the pool only from the first — doesn't happen in the real file, but the behavior isn't tested in the negative. | Deferred, behavior intended by the plan. |
| 4 | Minor, plan-mandated: `Weight="abc"` silently falls back to the default instead of a diagnostic — covers only the missing-attribute case, not a malformed value. | Deferred, intended by the brief ("like the game"). |
| 7 | **Real bug in the challenge parser.** `id_list` only did `split(',')`: challenge 44 (`achievements="490 415"`, space separator) and challenges 37/38 (`startingitems="-584,..."`, negative id) were being discarded as `MalformedList`, bringing the count of challenges read down to 42 instead of 45. | Fixed: `id_list` now accepts comma **or** whitespace; a new `item_id_list`, used only for `startingitems`, discards ids `<= 0` without invalidating the challenge (a negative id is a signal from the game, not an item). Three new unit tests, real-data test back to 45/34/14. |
| 7 | **Four wrong numbers in the brief**, all from measurements made against the text instead of the parser (see "The methodological lesson" below). | Fixed in the real-data tests and in the spec: see the dedicated table. |
| 7 | Minor, deferred: `item_id_list` has no direct test for `"abc"` → `None` and for `"0,34"` → `[34]` (correct by construction, but not covered by a test). | Deferred. |
| 7 | Minor, deferred: `id_list` and `item_id_list` duplicate the same split/trim/filter pipeline. | Deferred. |
| 7 | Minor, deferred: `n as u32` silently truncates an `i64` beyond `u32::MAX` instead of rejecting it — unrealistic in this domain (no game id comes anywhere near that threshold), but it isn't an explicit `TryFrom`. | Deferred. |

The four numbers fixed by Task 7, with their exact cause:

| Number | Brief | Real | Cause |
|---|---|---|---|
| Quality −1 | 2 cases | 0 cases | The grep searched for `quality="-1"` and matched `craftquality="-1"` as a substring (ids 422 and 710): a different attribute, the same text. |
| "Orphan" items (in no pool) | 26 | 24 | The count was derived from 723 collectibles, which includes the two **commented-out** elements of `items.xml` (id 43 `PILLS_HERE`, id 61 `TAROT_CARD`) as if they existed. |
| Gaps in ids 1..=732 | 9 | 11 | Same cause: the "723 collectibles" baseline was wrong (the real figure is 721); with the two commented-out elements counted as extra gaps, 733 − 721 − 1 = 11, not 9. |
| Challenges with more than one achievement | 13 | 14 | The grep only counted comma separators in the `achievements` list, missing challenge 44 (`"490 415"`, space separator), which still has two achievements. |

Tasks 1, 5 and 6 passed review with no findings at all.

## Final review

A second review — over the entire plan-B code and the documentation task that goes with
it — produced three Important findings, all closed in a single pass:

- **`crates/catalog/src/itempools.rs:2-3`**: the module doc said "26 items are in no
  pool"; the real number, after the Task 7 fix, is **24 collectibles** (trinkets aren't in
  pools by design, and aren't counted as "orphans"). Comment fixed.
- **Test promised by the spec but missing**: "the winning `items.xml` contains every id of
  the base-game `config.a`" was in the list of real-data tests but didn't exist. Added
  `the_winning_items_xml_contains_every_id_of_the_base_game` in `tests/real_data.rs`: it
  opens `samples/packed/config.a` with `unpack::Archive::open`, reads `resources/items.xml`,
  passes it to `Catalog::build` in isolation, and checks that the base game's set of
  `(ItemKind, id)` is included in the real winning file's set, with a guardrail (> 300
  items) so the test can't pass vacuously against an almost-empty file.
- **`catalog::Edition` clashed with `discovery::Edition`**: two types with the same name and
  different meanings (one the DLC an item belongs to, the other the edition of the installed
  game, already on the IPC boundary with different casing) are a conflict waiting to confuse
  anyone importing both crates. Renamed to `catalog::Origin` (and `edition_of` to
  `origin_of`), with a doc line in the module explaining why.

## Decisions made during execution

- **`Item` loses `Eq`.** `pools: Vec<PoolMembership>` carries a `weight: f32`, which
  doesn't implement `Eq`. `PartialEq` is kept; confirmed that no test uses `Item` as a
  `HashMap`/`HashSet` key.
- **`<item>` metadata indexed under `ItemKind::Passive`, with an explicit lookup.**
  `items_metadata.xml` doesn't distinguish passives, actives and familiars: a single
  `<item id>` element covers all three. Indexed under `Passive`, with an
  Active/Familiar → Passive translation at lookup time instead of triplicating the entry.
- **`quality: Option<i8>` instead of `u8`.** The real scale runs from −1 to 4 (not just
  0..4 as first assumed); `None` when the file is missing the line.
- **`Challenge.unlocked_by: Vec<AchievementId>`** (plural, since the file can link more
  than one achievement to a challenge) and **`Boss.unlocked_by` added** (the file carries
  it on 27 out of 103 bosses, not anticipated in the brief's first draft).
- **Negative ids in `startingitems` are discarded without invalidating the challenge.**
  `-584` is the game's own convention for "this starting item is actually a trinket", not
  a collectible item: outside the catalog's model, but not a format error that would
  justify discarding the whole challenge.
- **Expected numbers align to the parser, never the other way around.** Every time a
  real-data test failed, the rule was: investigate the code with a throwaway example
  first, change the expected value only after understanding where the discrepancy comes
  from — never blindly. In all four Task 7 cases the investigation showed a measurement
  error, not a parser bug (except for the challenges, where there really was a bug: see
  above). The explanation for each error is written in the corresponding test's comment.
- **The historical plan (`docs/superpowers/plans/2026-09-04-catalog-b.md`) is left
  untouched.** It's the document describing what was decided before the real numbers were
  discovered; the corrections live in the spec (the authority on conflicts) and in this
  report.

## The methodological lesson

For the second time in a row — plan A and plan B — numbers measured with a `grep` over
the raw XML text were wrong, and the real parser (`quick-xml` through the crate) was
right. The causes differ each time, but the shape of the error is the same: the text
seems to say something that the XML, read as structure, doesn't actually say.

- **Plan A:** a `grep` counted `PILLS_HERE_NAME` and `TAROT_CARD_NAME` as real items in
  `items.xml`; they were inside a `<!-- ... -->`, which `quick-xml` rightly ignores
  (911 instead of 909).
- **Plan B:** the same comment propagated the error downstream (723 instead of 721
  collectibles, hence 9 instead of 11 gaps and 26 instead of 24 pool orphans); an
  attribute with a matching substring (`craftquality="-1"` read as `quality="-1"`)
  invented a quality that doesn't exist; a separator different from the one assumed
  (space instead of comma in just one challenge out of 45) made a grep that assumed a
  single format miss a count.

From now on, numbers used as an expected value in a fixture **are measured with the
crate's own parser**, not with a `grep` over the text: a small throwaway example that
opens the file and prints the real count, as done in Task 7. A `grep` is still useful for
*orienting yourself*, but never as the source of a number that ends up in an
`assert_eq!`.

## What's left

- **The Minor items still deferred**: the test in `xml.rs` for the comment before the
  root (Task 3), the untested-in-the-negative behavior for duplicate ids across
  `ItemKind`s and for a malformed `Weight` (Task 4), the duplication between `id_list` and
  `item_id_list` (Task 7). The others — `Default` on `Metadata`, the comment on `text`,
  the `item_id_list` tests for `"abc"`/`"0,34"`, `n as u32` — were closed by the final
  review (see above).
- **`Diagnostic::UnresolvedKey` doesn't extend to achievements and challenges.** The spec
  anticipated this only if those files used keys the way `items.xml`/`players.xml` do;
  they don't: `achievements.xml` and `challenges.xml` carry literal English text, not a
  key into the string table. There's nothing to resolve, so nothing to extend.
- **Plan-A leftovers that remain open:** `SpriteView.kind` (no screen consumes it yet),
  the JSON shape of `ExtractionReport` (not yet pinned down by a test), `Archive::open`
  loading the whole archive (~1.3 GB) into memory instead of reading entry by entry.
- **The IPC contracts for the graph screens (M2).** Step 2 of "Handoff to design" in
  `docs/STATO.md`: types for a graph node (done · unlockable now · blocked by N),
  fan-out, goal and expanded plan, with fake data behind a command that declares itself
  as such. Three decisions remain before those types can be written:
  - a **reverse index `AchievementId → what it unlocks`**: today `catalog` only carries
    `unlocked_by` (item/character/challenge/boss → achievement), the graph also needs the
    opposite direction;
  - the **join between achievements and the save's section-1 slots**, to be pinned down
    with a real-data test (637 achievements in the file, 642 slots in the save: the
    correspondence isn't an identity and must be verified against data, not assumed);
  - **`unlock_condition` as displayable text**, with `None` treated as a normal state (354
    out of 637 achievements have no readable condition: it isn't missing data, it's a
    fact of the file).
