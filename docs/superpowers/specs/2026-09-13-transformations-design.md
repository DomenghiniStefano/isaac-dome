# The transformations, and the other half of B34

**Status:** design agreed in conversation on 2026-09-13. Three decisions were taken there —
the perimeter (transformations *and* the pickup noise, i.e. all of B34), where the item set
comes from (the pages, not the inverted index), and what "satisfied" means for a threshold.
Everything else below follows from those three plus the measurements in §0; it was written
after the agreement rather than walked through line by line, so it is the part most worth
disagreeing with.

**Scope.** Bring the game's sixteen transformations into the dataset, teach `graph` the one
shape of prerequisite it has never been able to say — *N of these items* — and stop the
generator from inventing pickup targets out of ordinary words. Closes **B34** whole.

**Not** in scope: every other family of reference without a page (`stage`, `room`,
`pickup`), the non-boss entities, and the two Cargo tables the snapshot downloads and never
reads. §7 lists them with the reason each is left out.

---

## 0. What was measured for this design, and how

Six facts. Four were read from the live wiki through its public, credential-free API as a
design probe — a developer reading a page, not code: `wiki-snapshot` remains the only thing
in the repo that talks to the network, and nothing here changes that. Two were counted on
the committed dataset. All on 2026-09-13.

1. **`Template:Infobox transformation` exists and is transcluded by exactly sixteen pages**
   (`action=query&list=embeddedin`), and they are the same sixteen rows as
   `dataset/raw/cargo/transformation.json`. The mechanism this repo already uses to list a
   page kind — `PageKind::template()` fed to `embeddedin` — fits without inventing anything.
2. **The `transformation` Cargo table declares eight fields; the snapshot downloads four.**
   `Special:CargoTables/transformation` lists `id`, `alias`, `dlc`, `requirement`, `items`,
   `description`, `target`, `appearance`. `crates/wiki-snapshot/src/api.rs` asks for
   `_pageName,id,alias,dlc`. The five it does not ask for are most of the content.

   > **Amended on 2026-09-13, after downloading them.** Two of those five carry nothing
   > usable, and this paragraph was wrong to count them as content.
   >
   > - **`requirement` is a template default.** All sixteen rows read the identical string
   >   `three items from this set` — Adult, whose infobox has no `requirement` parameter at
   >   all, included. A field with one value across the whole table states nothing about any
   >   row, and a numeral read from it would hand every transformation a 3 that looks
   >   measured and is not.
   > - **`items` is rendered HTML, and mostly absent.** Ten of the sixteen are empty; of the
   >   rest, four are markup Cargo produced rather than wikitext an editor wrote
   >   (`<span class="tooltip" data-tooltip="i%2Ftooltip…">`,
   >   `[[File:Dlc a indicator.png|link=|class=dlc…]]`). Only Adult's `[[Puberty]] pill` and
   >   Stompy's `Leo, Magic Mushroom` are plain, and neither is a complete set.
   >
   > `description`, `target` and `appearance` are plain text and are kept. The two dead
   > fields stay in the query — sixteen rows cost nothing, and dropping them would leave the
   > next person to rediscover why — but **nothing reads them**. §2.4 says where the count
   > comes from instead.
3. **The infobox's `items` is not the whole set.** Guppy's, verbatim, is seven `{{i|…}}` —
   Dead Cat, Guppy's Collar, Guppy's Tail, Guppy's Hairball, Guppy's Eye, Guppy's Head,
   Guppy's Paw. The page's body additionally carries `{{trinket table | Kid's Drawing }}`,
   and Kid's Drawing does count toward Guppy in the game. A set built from the infobox alone
   is short by a trinket, and short in the direction that makes the app tell someone they
   are one item away when they are not.
4. **Two pages are the degradation cases, and they are not hypothetical.** *Adult* has no
   `requirement` parameter at all and `items = [[Puberty]] pill` — free prose naming a
   pickup, not a collectible. *Super Bum* has `id = n/a` in the infobox and its item list
   written as three positional arguments (`| Bum Friend | Dark Bum | Key Bum`) with no
   `items =` key at all.
5. **`1000` is a sentinel, not a mystery.** Super Bum's infobox says `id = n/a` while its
   Cargo row says `1000`: the wiki's own template maps "no game id" onto that number. It is
   therefore *not* a `PlayerForm` id sitting next to 0–15, and nothing should treat it as
   one. Which id, if any, the game gives Super Bum is not answered here and is not needed.
6. **The inverted item→transformation index cannot be the source of the set.** Counted over
   `feature/wiki-infobox`'s `wiki.json`, distinct item and trinket pages carrying a
   reference to each transformation: Guppy **28**, Conjoined **44**, Yes Mother? **26**,
   against real sets of roughly seven. Since `429c04f` made
   `{{transformation contribution|X}}` resolve exactly like `{{tf|X}}`, the item side can no
   longer tell "counts toward Guppy" from "mentions Guppy" — Dr. Fetus and Brimstone are in
   Guppy's 28. **Adult sits at 0**, because a pill is not an item page. The set has to come
   from the transformation's side.

---

## 1. Boundaries

Nothing here needs a new crate. The work lands in the four places that already own these
concerns, and the split is the repo's usual one.

| crate | gains |
|---|---|
| `wiki-snapshot` | five field names in one Cargo query, and a seventh page kind to walk |
| `wiki` | `PageKind::Transformation`, the `Transformation` record, two template parsers |
| `graph` | `Requirement::Threshold`, its evaluation, and the generator's pickup rule |
| `ipc` / `ui` | one `RequirementView` variant, one count, the mirror types and the strings |

---

## 2. `wiki` — the seventh page kind

### 2.1 Fetching

`PageKind` gains `Transformation`, with `template()` = `Template:Infobox transformation`
and `dir()` = `transformation`. `PageKind::ALL` grows to seven and `dataset/raw/pages/`
gains a folder of sixteen files. The `transformation` entry in `TABLES` becomes
`_pageName,id,alias,dlc,requirement,items,description,target,appearance`.

### 2.2 The record

**Amended on 2026-09-13 while writing the plan**, against the code rather than against a
sketch of it. The first version of this section invented a `Transformation { page, requires,
contributors }` struct and a map of it. It is not needed: `Entry` already carries an
`infobox: Infobox`, an enum with one variant per kind, and that is precisely where
infobox-derived facts live for the other six kinds. So the transformation gets a variant
like everyone else —

```rust
Transformation {
    /// How many of `contributors` are needed. `None` when the page does not say it in a
    /// form we can read — NEVER defaulted to three.
    requires: Option<u32>,
    /// The items and trinkets that count, in page order, deduplicated.
    contributors: Vec<Target>,
    /// The infobox's `target`: what the transformation acts on ("Isaac's bums"). Kept
    /// because the Cargo table declares it, and `no_silent_parameter` fails on a
    /// parameter that is neither a field nor deliberately ignored.
    target: Vec<Inline>,
},
```

— and `Dataset` gains `transformations: BTreeMap<u32, Entry>`, shaped like its six
siblings. Page and set are then the same value by construction rather than by discipline,
which is what the original wording was reaching for. `Dataset::entry()` answers
`Target::Transformation` from that map; `Stage`, `Room` and `Pickup` stay in the "`None` by
construction" group and the comment has to stop claiming transformations belong there.

One consequence to accept with open eyes: `infobox_from` currently sees only the infobox,
and `contributors` needs the page body too (§2.3). It gains a `text: &str` parameter that
six of the seven arms ignore. The alternative — building this one variant outside
`infobox_from` — would break the property that one function builds every infobox, which is
worth more than the unused parameter.

### 2.3 Where `contributors` comes from

The **union** of two sources on the same page, deduplicated, in the order the page presents
them:

- the body's `{{collectible table | A, B }}` and `{{trinket table | C }}` — new templates,
  each resolving its comma-separated names through the existing `Resolver` into
  `Target::Item` / `Target::Trinket`. `collectible table` is currently among the dataset's
  unknown templates (5 occurrences), so this also shortens that list;
- the infobox's `items`, parsed as inline like any other wikitext, keeping the `Target`s.

A union rather than a choice because §0.3 and §0.4 show each source failing on a different
page, and in opposite directions. **Where the two disagree, the disagreement is recorded as
a diagnostic**: not resolved silently, and not treated as an error. Guppy is a real
disagreement on the live wiki today, and the count of them is the instrument that says the
cross-check is awake.

An item the resolver cannot turn into a `Target` is counted in the existing `unresolved`
diagnostic, as everywhere else.

### 2.4 Reading the count

**Rewritten on 2026-09-13** after §0.2's amendment. The count was to be read from
`requirement` through a closed map of English numerals; that field turns out to hold one
constant across all sixteen rows, so the map would have returned 3 for everything and the
`None` branch would never have run. A rule that cannot fail is not a rule.

The per-page statement is in the **body**, and it is a digit: Guppy's page reads *"Pick up 3
[[item]]s or [[trinket]]s from the following list"*, Super Bum's *"Pick up 3 [[item]]s from
the following list."* So `requires` is the number in that sentence, matched on the phrase
that introduces it, and **nothing else is tried**.

Adult stays unread on purpose, and is the reason the rule stays narrow: its page says
*"turns Isaac into an adult upon taking three [[Puberty]] pills"* — a different sentence
about a different kind of thing, a pickup rather than a set of collectibles. Widening the
pattern until Adult matches would mean inventing a set for it too. `requires: None` is the
honest answer there, it costs nothing (no achievement references Adult), and it keeps the
"some read, some unread" property of §6.4 from becoming vacuous.

There is still no fallback to three, for the reason the original paragraph gave and the
amendment strengthens: three is exactly the number a default would produce, so a wrong
default here is invisible.

---

## 3. `graph` — the threshold

### 3.0 How the data reaches `graph`, which is not through the dataset

**Amended on 2026-09-13 while writing the plan.** `Graph::build(c: &Catalog, rules: &Rules)`
never sees a `Dataset`: the wiki reaches this crate only through `requirements.json`,
generated offline by `graph::generate` and read at runtime by `rules.rs`. A threshold needs
its number and its item list at runtime, so they travel there, as a third top-level map
beside `achievements` and `targets`:

```rust
pub struct TransformationRow {
    pub label: String,
    /// `None` when the wiki page did not say. A row with no number is still emitted, so
    /// that "we have the transformation and cannot read its count" stays visible.
    pub at_least: Option<u32>,
    pub items: Vec<Target>,
}
```

`Requirements` gains `transformations: BTreeMap<u32, TransformationRow>`, and the generator
fills it from `Dataset.transformations`. This is a schema change to a committed artefact, so
`rules::SCHEMA_VERSION` goes to 2 and the regeneration is its own commit.

### 3.1 The variant

```rust
/// N of a set of items, in any combination: a transformation. Not a wall like the
/// others — the items may all be unlocked and the run still has to be played.
Threshold {
    transformation: u32,
    label: String,
    at_least: u32,
    of: Vec<ThresholdItem>,   // { kind: ItemKind, id: ItemId }
    /// Contributors the catalog could not resolve. Carried as a number rather than
    /// dropped, because §3.2 needs to know they exist: they can only ever help.
    unresolved: u32,
},
```

`resolve.rs` stops sending `Target::Transformation` to `from_verdict` and builds this
instead, resolving each contributor against the catalog the way `Requirement::Item` already
does.

**A set smaller than its own count is not a threshold either.** Found on 2026-09-13, on the
first real build: Stompy's page says "Pick up 3 items or pills from the following list" and
then lists two collectibles and **a pill**, as a bullet rather than in a table — and a pill
is not something this model has. Two contributors with a count of three is a requirement
nothing can ever satisfy, and evaluating it would report "you are one item away" forever. So
`graph` refuses to build a `Threshold` when `items.len() < at_least` and produces `Unknown`
instead. The dataset keeps both numbers, because both are what the page says; it is the
crate that has to *answer* that declines to answer with a number it knows is unreachable.
No achievement references Stompy today, so this is a trap disarmed rather than a bug fixed.

`at_least` is a `u32` and not an `Option`, so **a transformation whose `requires` is `None`
never becomes a `Threshold` at all**: it resolves straight to `Requirement::Unknown` with
the transformation's label. A variant that can be constructed without the number it is
about would let "unknown threshold" be mistaken for "threshold of zero", which is satisfied
by everything.

### 3.2 Evaluation, and why there is never an edge

**Amended on 2026-09-13 while writing the plan.** The first version said the third outcome
was "blocked, naming which contributors are still locked". Against `build.rs` that is not
expressible and it would have been wrong: being blocked is carried by `prerequisites`, and
the prerequisites of *any three of these eight items* are a disjunction of subsets. This
repo has already met that shape once — a challenge unlocked by several achievements — and
its answer is `GraphDiagnostic::Disjunction` plus an unknown, never an invented conjunction.
A `Threshold` therefore **never produces a prerequisite edge**, exactly like `Mark` and
`Counter`.

Which leaves the answer to evaluation, where the profile is. In `Graph::evaluate`, a
`Threshold` is one more requirement the `unanswerable` closure judges. Count the
contributors the profile has unlocked, then, in this order:

1. **at or above `at_least` → answered and met.** It contributes nothing: no edge, not
   unknown, not unanswerable. A node held by nothing else is then `availableNow` — the same
   reading `Mark` and `Counter` already have, "nothing is locked, the content only has to be
   played".
2. **otherwise, if `unresolved` is not zero, or `at_least` is `None` → unanswerable.** The
   node drops to `Partial`.
3. **otherwise → unanswerable as well, and declared**: a new
   `GraphDiagnostic::ThresholdUnmet { node, label, current, at_least }`, so that "this node
   is `Partial` because you have two of the three Guppy items" is on the record rather than
   folded into a generic count of unknowns. The view still draws `2 di 3` and the missing
   items (§5): what the graph refuses to invent is the *edge*, not the information.

The order matters and it is the whole design. An unresolved contributor can only ever *add*
to the count, so testing satisfaction first means an incomplete catalog can never turn a
"you can do this" into a "you cannot". Testing the unknown first would have produced exactly
the pessimistic wrong answer the repo's honesty rules exist to prevent, and the property in
§6 pins the order rather than trusting the reading.

### 3.3 What stops being hand-written

`transformation:Guppy` and `transformation:Beelzebub` lose their `Verdict::Unknown` entries
in `crates/graph/rules/corrections.json`, and `transformation` stops being one of the kinds
`verdict_required` covers, because it now resolves the way an item does. Four nodes — 65,
161, 178, 352 — get a real answer.

---

## 4. The other half of B34 — the pickup noise

`generate.rs` promotes an `Inline::Concept` to `Target::Pickup`, which is right for *Red
Heart* and wrong for *collect*, *ending*, *Bestiary*, *collection* and *tainted character*.
The rule replacing it is stated in code and not as a list of words:

> A concept page becomes `Target::Pickup` only if the `pickup` Cargo table knows that title.

The table is already downloaded, already loaded into `Tables`, and already used by the
`{{p|…}}` resolution — it is the wiki's own answer to "is this a pickup", and it separates
the thirteen noise targets from the real ones without anybody curating a stop-list.

`requirements.json` is a committed artefact, so this means regenerating it with
`pnpm graph:rules`, **reading the diff rather than trusting it**, keeping
`crates/graph/tests/coverage.rs` green — it goes red if the filter is too wide — and
deleting from `corrections.json` the verdicts that no longer have a target.

---

## 5. `ipc` and `ui`

```rust
/// A transformation: N of a set of items. Like Counter, not a wall when it is met.
Threshold {
    label: String,
    current: u32,
    at_least: u32,
    of: Vec<ThresholdItemView>,   // { itemKind, id, name, unlocked, page }
    unresolved: u32,
    /// The transformation's own wiki page, which now exists.
    page: Option<Target>,
},
```

`WikiCounts` gains `transformations`, which the verification screen shows beside the other
six. Both are changes to the live IPC contract, so they are **handed on, not merely
committed**, along with the mirror in `ui/src/lib/ipc/types.ts`.

On screen, the blocked badge of 3.5d draws `Guppy — 2 di 3` and its menu lists the
contributors, the locked ones marked, each opening its own page; the transformation's name
opens the transformation's page. No new primitive: `WhyMenu.vue` already draws a list of
requirements with links, and the strings go in `it.ts` / `en.ts` like every other one.

`TargetSprite` for a transformation stays `NoArt`. Whether the game ships artwork for a
transformation at all is unmeasured, and finding out is a measurement in `unpack`, not part
of this.

---

## 6. Testing

Test-first, and on the dataset properties rather than pinned numbers.

1. **The entries are as many as the Cargo table has rows** — never the literal sixteen. The
   count of transformations is the game's to change, and a hardcoded one breaks itself, as
   the 641-to-642 slot jump already taught this repo once.
2. **Guppy's `contributors` contains Kid's Drawing.** This is the non-vacuity guard for the
   whole union: it is precisely the element the infobox alone loses, so a test that passed
   with the union broken would be reporting coverage that is not there.
3. **At least one transformation records a source disagreement.** Same reason: a cross-check
   that has never spoken has not been shown able to.
4. **`requires` is `None` for Adult and for Super Bum, and `Some` for Guppy.** The
   degradation is asserted on the value, not on the absence of a panic.
5. **Monotonicity.** A profile whose unlocked contributors already reach the threshold stays
   satisfied when one of the remaining contributors is made unresolvable. This is §3.2's
   order, pinned.
6. **`Partial` is reachable and honest**: `requires: None` yields `Unknown`, never
   satisfied, and never blocked.
7. **No surviving pickup target is unknown to the Cargo table**, asserted over the
   regenerated `requirements.json`; and `coverage.rs` stays green, which is the guard
   against a filter that eats a real target.
8. **JSON shape** of `RequirementView::Threshold` and of the new count, `rename_all_fields`
   included — the silent failure this repo has already been bitten by.
9. **On real data**: nodes 65, 161, 178 and 352 stop being `Partial`. Skips with a note when
   `samples/` is absent, through `test-support` like every other real-data test.
10. `crates/wiki`'s `derived` test keeps `wiki.json == build(raw, corrections)`, so the
    re-snapshot and the parser cannot drift apart.

---

## 7. Out of scope, with the reason

- **`stage` (1444 references), `room` (1420), `pickup` (668).** Three more page kinds, none
  of which has a threshold to make it useful; `room` has no Cargo table at all. They are the
  next question, not this one.
- **The non-boss entities.** The `entity` table covers everything, the pages cover 102
  bosses. Hundreds of pages for the bestiary is its own sub-project.
- **`player.json` and `stage.json`** — downloaded, committed, and read by nothing, because
  `Raw::load` never puts them in `Tables`. `player.parent` is the Tainted→base relation,
  which is exactly what the two character-identity bugs of M2 were about, so it deserves a
  backlog entry of its own rather than a silent fix here.
- **B36**, the marks and counters that name a boss without linking to it: a different
  measurement.
- **Reopening `429c04f`.** The item side can no longer distinguish a contribution from a
  mention, and §0.6 is why that stopped mattering: the set comes from the transformation's
  side, and the cross-check lives between two sources on the *same* page.

---

## 8. Order

Precondition: **`feature/wiki-infobox` merged into `develop`.** It is reshaping `Entry` and
the template handling that this design builds on, and N7 is already waiting on it; two
branches rewriting `wiki.json` in parallel is a merge nobody wants to read.

Then `feature/wiki-transformations`, cut from `develop`, in this order — each step leaving
the suite green:

1. the five Cargo fields, and the snapshot re-fetched;
2. `PageKind::Transformation`, the two table templates, the `Transformation` record;
3. `Requirement::Threshold` and its evaluation;
4. the generator's pickup rule, `requirements.json` regenerated, orphaned verdicts removed;
5. `ipc`, the mirror types, the screen.

Steps 1–2 and step 4 each rewrite a committed artefact, so each is its own commit with its
diff read, not folded into the code change that motivated it.
