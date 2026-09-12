# Graph — requirements the profile answers with a mark or a counter (design)

**Date:** 2026-09-12
**Milestone:** M2's graph, reopened (`docs/STATUS.md`)
**Branch:** `feature/graph-mark-requirements`, cut from `develop`
**Depends on:** M2 (`crates/graph`, the two rules files); the completion matrix of cycle 3.2
(`crates/ipc/src/marks.rs`, 34 × 12, verified 2026-09-08); `core_save::Save::u32s` on
section 2; `crates/ipc/tests/marks_real.rs`
**Status:** the design below was approved section by section in conversation on 2026-09-12.
The measurement in §2 was taken during that conversation and is new to the repository.

## 1. What this sub-project is

The graph can express exactly one kind of prerequisite: **"behind another achievement"**.
Five of the most-referenced prerequisites in the wiki are not behind an achievement at all —
they are behind *having played something* — so they resolve to `Requirement::Unknown` and
drag their nodes down to `Partial`.

That is **178 of the 195 uninterpreted references** in the whole graph:

| target | nodes it blocks |
|---|---|
| `entity:Mother` | 38 |
| `entity:Delirium` | 36 |
| `entity:Hush` | 36 |
| `entity:The Beast` | 34 |
| `entity:Ultra Greedier` | 34 |

**Not one of them is a curation gap.** A probe over the embedded rules found **zero**
references that reach `Unknown` for want of a verdict: every one is a `Verdict::Unknown`
written by hand, with its reason. `corrections.json` is complete; the *model* is what is
missing a case.

This sub-project adds that case, and the save already holds the data to answer it.

## 2. The measurement this design rests on

Taken 2026-09-12 on the dated series in `samples/` (36 saves, 2026-06-29 → 2026-09-12), by the
method the mark bases were pinned with on 2026-09-08: look at **the day a cell changed** and
require three independent facts to agree.

### 2.1 The shape of the requirement — it is per character

170 of the 178 references **carry a character alongside the boss**. The wiki does not say
"defeat Mother", it says "defeat Mother *as Magdalene*":

```
297  entity:Ultra Greedier | character:Magdalene
299  entity:Ultra Greedier | character:Judas
308  entity:Ultra Greedier | character:Keeper
```

So the answer is **one cell of the 34 × 12 completion matrix**, not a global tally. Only 8
nodes name the boss alone (2 `Hush`, 2 `Delirium`, 4 `Mother`), and those are the ones a
kill counter answers.

### 2.2 New: in the Greed column, bit 1 is Ultra Greedier

Three days, three different characters, each time **that character's** Greed cell
(block base 130) gains bit 1:

| day | achievement | character from the wiki | cell | value |
|---|---|---|---|---|
| 20260824 | 308 | Keeper (row 12) | 142 | `0 → 3` |
| 20260829 | 299 | Judas (row 3) | 133 | `1 → 3` |
| 20260831 | 297 | Magdalene (row 1) | 131 | `0 → 3` |

**Scope of the claim:** measured on the Greed column only. Whether bit 1 means "the harder
variant" in the other eleven columns is **not** measured here and stays open.

This lands against the reading `CLAUDE.md` records the same day: bit 2 is **"won online"**,
and bits 0 and 1 are the mark's two levels, with *which level is which* left as the unmeasured
half. §2.2 measures that half for one column — in Greed, the level on bit 1 is Ultra Greedier —
and claims nothing about the other eleven. The two measurements do not overlap: all three days
above set bit 1 with bit 2 clear (`0 → 3`, `1 → 3`), so none of them is an online clear and
none is evidence about bit 2.

### 2.3 The method reproduces what is already known

Run against cells that were pinned independently, the same probe returns them:

- `442` (Mother as Magdalene, row 1) moved cell **424** → base **423**, the documented Mother base.
- `445` (The Beast as Cain, row 2) moved cell **459** → base **457**, the documented Beast base.
- `586` (Delirium as T. Cain, +4) moved **408**; `591` (Delirium as T. Azazel, +9) moved **413**
  — both against the 19-block Delirium base **404**, and matching the note in
  `reference/isaac_counters.py` that four characters agree on that base.

### 2.4 The 40 unlocated cells stay unlocated — now verified, not inherited

`docs/STATUS.md` records that Mother and The Beast for The Forgotten and the 19 later
characters sit in 423–490 by spacing but are zero in every save. The probe searched the whole
series for **a single Mother or The Beast completion by any of those 20 characters** and found
none. The claim is now measured rather than assumed: the blocker is real, and one run of
Mother with a Tainted character closes it.

## 3. Applicable constraints

1. **A name is structural or measured, never plausible.** §2.2 is a measurement with three
   independent agreements. Everything this design cannot measure stays `Unknown` and says so.
2. **`graph` stays pure and dependency-light**: no `core-save`, no new I/O. It already depends
   only on `catalog` and `wiki`, and keeps to it.
3. **No offsets in a rules file, and none across the IPC.** `corrections.json` names
   `"hushKills"`; the number 158 lives with the code that knows the file's layout. Same reason
   the boundary refuses paths and offsets.
4. **Degrade, never fail.** A missing or short section 2 sends every new requirement back to
   `Unknown` — never to "satisfied".
5. **Exhaustiveness.** Two new `Requirement` variants and two new `RequirementView` variants;
   no `_ =>` arm anywhere, so every match that needs updating breaks the build.
6. **The IPC contract is live.** `RequirementView` is material the design system is built on:
   this change is handed on, not merely committed.

## 4. Design

### 4.1 Domain — two variants, not one

```rust
pub enum Requirement {
    // … existing …
    /// One cell of the completion matrix: "beat <column> as <character>, at <level>".
    Mark { character: CharacterId, column: MarkColumn, level: MarkLevel },
    /// A section-2 tally, named rather than indexed.
    Counter { name: CounterName, at_least: u32 },
}
```

Two variants and not one `Progress`, because they are two different things to the player — a
cell is "you, with this character" and a counter is "anyone, ever" — and fusing them would
erase exactly the distinction the screen draws.

`MarkColumn` is the twelve columns. `MarkLevel` is **named structurally, not semantically**:
`Base` for bit 0 and `Second` for bit 1. Calling it `Hard` would smuggle in exactly the claim
§2.2 refuses to make — that bit 1 means "hard mode" in the other eleven columns — and this
repository has already paid once for a plausible label. Both enums are fieldless.

### 4.2 Curation — five rows, one new verdict

The boss's name is not the column's name (`Ultra Greedier` lives in the `Greed` column, at
level 1), so the mapping is curation, in the file curation already lives in:

```json
"entity:Ultra Greedier": { "mark": { "column": "greed", "level": "second" } },
"entity:Hush":           { "mark": { "column": "hush",  "level": "base" },
                           "counter": { "name": "hushKills", "atLeast": 1 } }
```

A target may carry both: the `mark` answers a reference that comes with a character, the
`counter` answers the 8 that do not. `Mother` and `The Beast` get the same pair; `Delirium`
too. Where a target has only one of the two and the reference needs the other, the outcome is
`Unknown` — declared, as always.

### 4.3 Evaluation — `graph` never learns the layout

`Graph::evaluate(flags: Option<&[bool]>)` becomes `Graph::evaluate(profile: &dyn Profile)`,
where `Profile` is the achievement flags plus two resolved lookups supplied by the caller:

```rust
pub trait Profile {
    fn done(&self) -> Option<&[bool]>;
    fn mark(&self, character: CharacterId, column: MarkColumn) -> Option<MarkLevel>;
    fn counter(&self, name: CounterName) -> Option<u32>;
}
```

`Graph::missing_chain(achievement, flags)` takes the same parameter and changes with it: the
two are the crate's only entry points that read a profile, and they must not disagree about
what a profile is.

`graph` stays symbolic end to end. The implementation — `CharacterId` → matrix row, column →
block base, `"hushKills"` → 158 — lives with the code that knows the save's layout.

### 4.4 A structural fix carried along

The mark layout (`BOSSES`, `CHARACTERS`, the block bases, `counter_index`) is **knowledge of
the save file's shape**, and it currently sits in `crates/ipc/src/marks.rs`, a view-model
module. It moves to a module whose subject it actually is, and `ipc::marks` becomes a view
over it — which is also what makes §4.3's implementation have an obvious home instead of a
convenient one. Targeted, not a refactor of the crate.

### 4.5 The IPC contract

```rust
pub enum RequirementView {
    // … existing …
    Mark { character: u32, character_name: String, column: MarkColumn, level: MarkLevel },
    Counter { label: String, current: u32, at_least: u32 },
}
```

- `Counter` carries the progress (`current` / `atLeast`): a threshold above 1 can be curated
  later without reopening the type.
- `Mark` carries no progress: for one cell the state is binary, and an invented percentage
  would be a number nobody measured.
- `MarkColumn` and `MarkLevel` are fieldless enums, so on the wire they are **bare camelCase
  strings** and the TypeScript is a union of values — the repo's rule, zero exceptions.
- `rename_all_fields = "camelCase"` on both variants, pinned by a test on the JSON shape.

### 4.6 What changes on screen

A node held back only by a mark or a counter is **`availableNow: true`**: nothing is locked,
the content is reachable, it only has to be played. It keeps listing what to go do in
`missing[]`.

Those nodes are `Partial` today, so **Unlock's "unlockable now" count rises** and such nodes
become eligible as Next steps — which is the question that screen exists to answer. This is
the part to hand to the design system.

## 5. What it closes

| | nodes |
|---|---|
| answered by `Mark` | **130** (Hush 34, Delirium 34, Ultra Greedier 34, Mother 14, The Beast 14) |
| answered by `Counter` | **8** |
| still `Partial`, declared | **40** (Mother and The Beast × The Forgotten and the 19) |

**138 of 178.** The 40 have one cause, named in §2.4, and one run closes them.

## 6. Out of scope

- The 17 remaining uninterpreted references (13 `pickup:`, 4 `transformation:`). The `pickup:`
  ones are generator noise — `collect`, `ending`, `collection`, `Bestiary` are words the wiki
  sentence turned into `Target::Pickup` via `Inline::Concept`, not targets. Filtering them
  means regenerating `requirements.json`, which is its own verification: separate task.
- Bit 1 in the other eleven columns, and bit 2 anywhere (B21).
- Locating the 40 cells. It needs a run, not code.

## 7. Testing

Test-first, and on the series **properties rather than pinned values**:

1. **The measurement becomes its guard.** On every day of the series where an achievement
   whose references are (`entity:Ultra Greedier`, character *X*) flipped, *X*'s Greed cell
   gained bit 1. Holds for any profile and any era; it is what would catch the column moving.
2. **The method still reproduces the known bases** (§2.3): Mother 423 and The Beast 457 for
   the originals, Delirium 404 for the 19.
3. **Degradation**: a truncated or absent section 2 yields `Unknown` for every `Mark` and
   `Counter`, and never "satisfied" — asserted on the value, not on the absence of a panic.
4. **An unlocated cell degrades the same way**: the 40 stay `Partial` and name themselves.
5. **JSON shape** of both new variants, `rename_all_fields` included.
6. Real-data tests skip with a note when `samples/` is absent, as every other one does.
