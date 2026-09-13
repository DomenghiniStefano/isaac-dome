# Transformations, and the rest of B34 — implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:executing-plans` to implement
> this plan task by task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring the sixteen transformations into the wiki dataset, give `graph` a threshold
requirement — *N of these items* — and stop the generator from turning ordinary words into
pickup targets.

**Architecture:** The wiki side gains a seventh `PageKind` whose infobox variant carries the
item set and the count. The set reaches `graph` through `requirements.json`, the generated
rules file, because `Graph::build` never sees a `Dataset`. At runtime a threshold behaves
like `Mark` and `Counter`: it never draws a prerequisite edge, and evaluation against the
profile decides whether it is met.

**Tech Stack:** Rust 2021, `serde`/`serde_json`, Vue 3 + TypeScript for the two mirror
changes. No new dependency.

**Spec:** `docs/superpowers/specs/2026-09-13-transformations-design.md` — read it first; the
amendments in §2.2, §3.0 and §3.2 are the parts this plan implements, and they overrule the
paragraphs above them.

## Global Constraints

- **Precondition:** `feature/wiki-infobox` is merged into `develop`. Work on
  `feature/wiki-transformations`, cut from `develop`. Do not start otherwise.
- **No `_ =>` arm on a closed enum.** Adding `PageKind::Transformation`,
  `InfoboxKind::Transformation`, `Infobox::Transformation`, `Requirement::Threshold` and
  `RequirementView::Threshold` must break the build everywhere they are matched; fix each
  site explicitly.
- **Every struct crossing the IPC** has `#[serde(rename_all = "camelCase")]`, and every enum
  with struct variants also `rename_all_fields = "camelCase"`.
- **Never hardcode a count** of transformations, items, achievements or challenges. Sixteen
  is what the wiki has today, not a constant.
- **Test-first.** The expected value comes from the spec, never from the code's output. A
  failing test is first a hypothesis of a bug in the code.
- **Tests on real data skip with a note** (`skip: …` on stderr) and open `samples/` only
  through `test-support`.
- **Never `unwrap()` or `panic!`** outside tests on data read from disk.
- **`pnpm check`** must be green before anything is called done. Commit messages are
  Conventional Commits, `type(scope): subject`, English, no attribution trailer.
- Committed artefacts — `dataset/raw/`, `dataset/wiki.json`, `crates/graph/rules/*.json` —
  are regenerated in **their own commit**, never folded into the code change that caused it.
- **Anything that changes `dataset/raw/` or the wiki parser leaves `derived` red until
  `pnpm wiki:build` runs.** `crates/wiki/tests/derived.rs` asserts
  `wiki.json == build(raw, corrections)`, so a fetch or a new template breaks it immediately.
  Rebuild and commit the artefact **in the same sitting as the change that caused it**, not
  at the end: this plan originally scheduled the only rebuild at Task 6 and would have left
  the suite red across four tasks. Added on 2026-09-13, after doing exactly that.

---

## File structure

| file | responsibility after this plan |
|---|---|
| `crates/wiki-snapshot/src/api.rs` | `TABLES`: five more fields on the `transformation` query |
| `crates/wiki/src/page.rs` | `PageKind::Transformation`, `EntryKey::Transformation`, wiring in `parse_page` |
| `crates/wiki/src/inline.rs` | `{{collectible table}}` and `{{trinket table}}` → one `Inline::Ref` per name |
| `crates/wiki/src/transformation.rs` | **new**: the count read from the body sentence, the contributors union, the disagreement flag |
| `crates/wiki/src/infobox.rs` | `InfoboxKind::Transformation`, the new arm of `infobox_from` |
| `crates/wiki/src/model.rs` | `Infobox::Transformation` |
| `crates/wiki/src/dataset.rs` | `transformations` map, `Counts.transformations`, the `entry()` arm |
| `crates/wiki/src/diagnostics.rs` | `transformation_sources_disagree` |
| `crates/graph/src/rules.rs` | `TransformationRow`, `Requirements.transformations`, schema 2 |
| `crates/graph/src/generate.rs` | fills that map; the pickup promotion rule |
| `crates/graph/src/model.rs` | `Requirement::Threshold` |
| `crates/graph/src/resolve.rs` | `Target::Transformation` → `Threshold` |
| `crates/graph/src/build.rs` | `Threshold` draws no edge; `GraphDiagnostic::ThresholdUnmet` |
| `crates/graph/src/evaluate.rs` | the three outcomes, against the profile |
| `crates/ipc/src/graph.rs` | `RequirementView::Threshold` |
| `crates/ipc/src/wiki.rs` | `WikiCounts.transformations` |
| `ui/src/lib/ipc/types.ts` | the two mirrors |
| `ui/src/components/graph/WhyMenu.vue`, `ui/src/i18n/{it,en}.ts` | the row and its strings |

---

## Task 1: the five Cargo fields, and the snapshot refreshed

> **Done on 2026-09-13, and it taught something the spec had wrong.** Once downloaded, two of
> the five fields turned out to carry nothing: `requirement` is the identical template
> default on all sixteen rows, and `items` is rendered HTML, empty on ten of them. See the
> amendment in the spec's §0.2. The fields stay in the query and **nothing reads them**; the
> count comes from the page body instead (spec §2.4, Task 4 below). The steps are left as
> they were run.
>
> The refresh also caught wiki churn worth naming: collectible **378 was renamed from
> "Number Two" to "No. 2"**, same id, so a page was deleted and another written. Nothing was
> lost, and the rename is what the deletion in `fetch`'s output means.

**Files:**
- Modify: `crates/wiki-snapshot/src/api.rs` (the `transformation` row of `TABLES`)
- Test: `crates/wiki-snapshot/src/api.rs` (inline `mod tests`)
- Artefact: `dataset/raw/cargo/transformation.json`, and whatever else the fetch refreshes

**Interfaces:**
- Consumes: nothing.
- Produces: `dataset/raw/cargo/transformation.json` rows carrying `requirement`, `items`,
  `description`, `target`, `appearance` besides `_pageName`, `id`, `alias`, `dlc`.

- [ ] **Step 1: Write the failing test**

In `crates/wiki-snapshot/src/api.rs`, inside `mod tests`:

```rust
/// The `transformation` table declares eight fields (`Special:CargoTables/transformation`,
/// read 2026-09-13) and the page's own content lives in five of them. Asking for four was
/// enough while a transformation was only a name; it is not enough to state a threshold.
#[test]
fn the_transformation_query_asks_for_the_fields_that_carry_the_content() {
    let (_, fields) = TABLES
        .iter()
        .find(|(t, _)| *t == "transformation")
        .expect("the transformation table is downloaded");
    for f in ["requirement", "items", "description", "target", "appearance"] {
        assert!(fields.contains(f), "missing field {f} in {fields}");
    }
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p wiki-snapshot the_transformation_query -- --nocapture`
Expected: FAIL, `missing field requirement in _pageName,id,alias,dlc`

- [ ] **Step 3: Make it pass**

```rust
    (
        "transformation",
        "_pageName,id,alias,dlc,requirement,items,description,target,appearance",
    ),
```

- [ ] **Step 4: Run it and watch it pass**

Run: `cargo test -p wiki-snapshot the_transformation_query`
Expected: PASS

- [ ] **Step 5: Commit the code change on its own**

```bash
git add crates/wiki-snapshot/src/api.rs
git commit -m "feat(wiki-snapshot): the transformation query asks for the content fields"
```

- [ ] **Step 6: Refresh the snapshot, and read the diff**

Run: `pnpm wiki:fetch`

This is the once-per-release pass and it re-reads every page, so expect churn unrelated to
this work: pages edited on the wiki since 2026-09-06. `write_if_changed` means only real
changes appear. Read `git status` and skim `git diff --stat` before staging. Confirm
`dataset/raw/cargo/transformation.json` now carries the five fields on all sixteen rows.

- [ ] **Step 7: Commit the artefact separately**

```bash
git add dataset/raw
git commit -m "chore(dataset): refresh the snapshot with the transformation content fields"
```

---

## Task 2: `{{collectible table}}` and `{{trinket table}}`

**Files:**
- Modify: `crates/wiki/src/inline.rs` (the template `match`, beside `"achievement text"`)
- Test: `crates/wiki/src/inline.rs` (inline `mod tests`)

**Interfaces:**
- Consumes: `Resolver::resolve("i" | "t", name) -> Resolution`.
- Produces: for a page's body, one `Inline::Ref { target: Target::Item | Target::Trinket }`
  per comma-separated name. Task 4 reads these back out of the wikitext.

- [ ] **Step 1: Write the failing test**

```rust
/// A transformation page lists what counts toward it as two tables, each one template with
/// a comma-separated argument — the same shape as `achievement text`, which is why it
/// cannot go through `resolve`: that answers with one target and this needs one per name.
#[test]
fn the_two_tables_push_one_reference_per_name() {
    let (v, d) = p("{{collectible table | Dead Cat, Guppy's Head }} {{trinket table | Kid's Drawing }}");
    let targets: Vec<_> = v
        .iter()
        .filter_map(|i| match i {
            Inline::Ref { target, .. } => Some(target.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(targets.len(), 3, "{v:?}");
    assert!(matches!(targets[2], Target::Trinket { .. }));
    assert!(d.unknown_templates.is_empty(), "{:?}", d.unknown_templates);
}

/// A name the resolver does not know stays on the page as text: the reader still sees what
/// the wiki said, and the miss is counted rather than swallowed.
#[test]
fn an_unknown_name_in_a_table_stays_as_text() {
    let (v, _) = p("{{collectible table | Nope }}");
    assert!(v.iter().all(|i| !matches!(i, Inline::Ref { .. })), "{v:?}");
}
```

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p wiki the_two_tables_push -- --nocapture`
Expected: FAIL — the template is unknown, so no `Inline::Ref` is produced and
`unknown_templates` holds `collectible table`.

- [ ] **Step 3: Make them pass**

In the template `match` of `inline.rs`, next to the `"achievement text"` arm:

```rust
        // Two more list templates, same reason as `achievement text`: the argument is a
        // comma-separated list of names and `resolve` answers with a single target. They
        // are the transformation pages' own statement of what counts toward them.
        "collectible table" | "trinket table" => {
            let kind = if name == "collectible table" { "i" } else { "t" };
            for (n, item) in arg.split(',').enumerate() {
                let item = item.trim();
                if item.is_empty() {
                    continue;
                }
                if n > 0 {
                    out.buf.push_str(", ");
                }
                match r.resolve(kind, item) {
                    Resolution::Target(target) => out.push(Inline::Ref {
                        target,
                        label: item.to_string(),
                    }),
                    // Not dropped: a name we cannot resolve is still what the page says.
                    Resolution::Concept
                    | Resolution::Unresolved
                    | Resolution::Ignore
                    | Resolution::Unknown => {
                        d.unresolved(kind);
                        out.buf.push_str(item);
                    }
                }
            }
        }
```

- [ ] **Step 4: Run them and watch them pass**

Run: `cargo test -p wiki tables`
Expected: PASS, both.

- [ ] **Step 5: Commit**

```bash
git add crates/wiki/src/inline.rs
git commit -m "feat(wiki): the two table templates resolve one reference per name"
```

---

## Task 3: `PageKind::Transformation`

**Files:**
- Modify: `crates/wiki/src/page.rs` (`PageKind`, `ALL`, `dir`, `template`)
- Test: `crates/wiki/src/page.rs`

**Interfaces:**
- Produces: `PageKind::Transformation`, `dir()` = `"transformation"`, `template()` =
  `"Template:Infobox transformation"`. `wiki-snapshot`'s fetch walks `PageKind::ALL`, so no
  change is needed there.

- [ ] **Step 1: Write the failing test**

```rust
/// Measured on 2026-09-13: `Template:Infobox transformation` exists and is transcluded by
/// exactly the sixteen pages the Cargo table has rows for. The kind is listed like the
/// other six so that `fetch`, which walks `PageKind::ALL`, picks it up without a special
/// case.
#[test]
fn the_transformation_kind_names_its_template_and_its_folder() {
    assert_eq!(PageKind::Transformation.dir(), "transformation");
    assert_eq!(
        PageKind::Transformation.template(),
        "Template:Infobox transformation"
    );
    assert!(PageKind::ALL.contains(&PageKind::Transformation));
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p wiki the_transformation_kind_names -- --nocapture`
Expected: FAIL to compile — no variant `Transformation`.

- [ ] **Step 3: Make it pass**

Add the variant to `PageKind`, extend `ALL` to `[PageKind; 7]`, and add the two arms to
`dir()` and `template()`. The compiler names every other match site; there should be none
outside this file.

- [ ] **Step 4: Run the crate's tests**

Run: `cargo test -p wiki`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/wiki/src/page.rs
git commit -m "feat(wiki): a seventh page kind for the transformations"
```

- [ ] **Step 6: Now fetch the pages — Task 1's run could not**

`fetch` walks `PageKind::ALL`, and until this task the kind did not exist, so Task 1's pass
downloaded the Cargo fields and **no transformation page at all**. Run it again:

Run: `pnpm wiki:fetch`
Expected: a new `transformation:` line reporting sixteen pages written, and
`dataset/raw/pages/transformation/` with sixteen `.wikitext` files. Everything else should be
unchanged, since Task 1 just refreshed it.

- [ ] **Step 7: Commit the artefact separately**

```bash
git add dataset/raw
git commit -m "chore(dataset): the sixteen transformation pages"
```

---

## Task 4: the count, the contributors, and the disagreement

**Files:**
- Create: `crates/wiki/src/transformation.rs`
- Modify: `crates/wiki/src/lib.rs` (`mod transformation;`), `crates/wiki/src/diagnostics.rs`
- Test: `crates/wiki/src/transformation.rs`

**Interfaces:**
- Consumes: `RawInfobox`, the page text, `Resolver`, `Diagnostics`.
- Produces:
  ```rust
  pub fn requires(text: &str) -> Option<u32>
  pub struct Contributors { pub targets: Vec<Target>, pub disagree: bool }
  pub fn contributors(ib: &RawInfobox, text: &str, r: &Resolver, d: &mut Diagnostics) -> Contributors
  ```
  Task 5 calls both from `infobox_from`.

- [ ] **Step 1: Write the failing tests**

```rust
use super::*;
use crate::resolver::fixtures::test_resolver;

fn ib(pairs: &[(&str, &str)]) -> RawInfobox {
    RawInfobox {
        name: "infobox transformation".into(),
        params: pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
    }
}

/// Guppy's real sentence. The count lives in the page body, not in the infobox's
/// `requirement`, which holds the same template default on all sixteen rows (spec §0.2).
#[test]
fn the_count_is_read_from_the_body_sentence() {
    assert_eq!(
        requires("Pick up 3 [[item]]s or [[trinket]]s from the following list"),
        Some(3)
    );
    assert_eq!(requires("Pick up 3 [[item]]s from the following list."), Some(3));
}

/// Adult's real sentence, which is about pickups and not about a set. It stays unread, and
/// the count must never fall back to three — three is exactly what a default would produce,
/// so a wrong default here would be invisible.
#[test]
fn a_sentence_that_is_not_the_pattern_is_none_and_never_three() {
    assert_eq!(requires("turns Isaac into an adult upon taking three [[Puberty]] pills"), None);
    assert_eq!(requires(""), None);
    assert_eq!(requires("three items from this set"), None);
}

/// The union of the two sources on the same page. The body's tables carry the trinket the
/// infobox omits (Guppy's Kid's Drawing, measured 2026-09-13); the infobox carries pages
/// whose body has no table at all. Neither alone is the set.
#[test]
fn contributors_are_the_union_of_the_infobox_and_the_body_tables() {
    let r = test_resolver();
    let mut d = Diagnostics::default();
    let c = contributors(
        &ib(&[("items", "{{i|Dead Cat}}")]),
        "== Collectibles ==\n{{collectible table | Dead Cat }}\n{{trinket table | Kid's Drawing }}",
        &r,
        &mut d,
    );
    assert_eq!(c.targets.len(), 2, "deduplicated, and the trinket kept: {:?}", c.targets);
    assert!(c.disagree, "the infobox is short by one: that is a disagreement");
    assert_eq!(d.transformation_sources_disagree, 1);
}

/// Same set on both sides is not a disagreement, and the diagnostic stays at zero — the
/// instrument has to be able to say "nothing to report" as well as "something".
#[test]
fn agreeing_sources_are_not_a_disagreement() {
    let r = test_resolver();
    let mut d = Diagnostics::default();
    let c = contributors(
        &ib(&[("items", "{{i|Dead Cat}}")]),
        "{{collectible table | Dead Cat }}",
        &r,
        &mut d,
    );
    assert_eq!(c.targets.len(), 1);
    assert!(!c.disagree);
    assert_eq!(d.transformation_sources_disagree, 0);
}
```

Use whatever item and trinket names `test_resolver()` already knows; if `Dead Cat` and
`Kid's Drawing` are not among them, add them to the fixture in
`crates/wiki/src/resolver.rs` rather than changing the assertions — the point of the test
is the union, and the names should be the real ones.

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p wiki transformation:: -- --nocapture`
Expected: FAIL to compile — the module does not exist.

- [ ] **Step 3: Write the module**

```rust
//! The two facts a transformation page states beyond its prose: how many items it takes,
//! and which items count. Both are read from the page itself; neither is inferred from the
//! item side, which since `429c04f` cannot tell a contribution from a mention.

use crate::infobox::RawInfobox;
use crate::inline::parse_inline;
use crate::resolver::Resolver;
use crate::{Diagnostics, Inline, Target};

/// The sentence that states the count, and the only one read. The infobox's `requirement`
/// is **not** a source: it holds `three items from this set` on all sixteen rows, a template
/// default that would hand every transformation the same number (spec §0.2).
const PICK_UP: &str = "Pick up ";

/// `"Pick up 3 [[item]]s or [[trinket]]s from the following list"` → 3. Narrow on purpose:
/// Adult's page says "upon taking three [[Puberty]] pills", which is a different claim about
/// a different kind of thing, and widening the pattern until it matched would mean inventing
/// a set of collectibles for an item that has none.
pub fn requires(text: &str) -> Option<u32> {
    let after = text.find(PICK_UP).map(|i| &text[i + PICK_UP.len()..])?;
    let digits: String = after.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

pub struct Contributors {
    pub targets: Vec<Target>,
    /// The two sources named different sets. Recorded, not resolved: on the live wiki today
    /// Guppy's infobox omits a trinket its body lists.
    pub disagree: bool,
}

/// Every item and trinket target of an inline run, in order.
fn refs(inline: &[Inline], out: &mut Vec<Target>) {
    for i in inline {
        match i {
            Inline::Ref { target, .. } => match target {
                Target::Item { .. } | Target::Trinket { .. } => out.push(target.clone()),
                Target::Character { .. }
                | Target::Achievement { .. }
                | Target::Challenge { .. }
                | Target::Entity { .. }
                | Target::Transformation { .. }
                | Target::Stage { .. }
                | Target::Room { .. }
                | Target::Pickup { .. } => {}
            },
            Inline::Edition { inline, .. } => refs(inline, out),
            Inline::Text { .. } | Inline::Concept { .. } => {}
        }
    }
}

/// The union of the infobox's `items` and the body's `{{collectible table}}` /
/// `{{trinket table}}`, deduplicated, body first because that is the more complete source.
pub fn contributors(
    ib: &RawInfobox,
    text: &str,
    r: &Resolver,
    d: &mut Diagnostics,
) -> Contributors {
    let mut body = Vec::new();
    for line in text.lines().filter(|l| {
        l.contains("{{collectible table") || l.contains("{{trinket table")
    }) {
        refs(&parse_inline(line, r, d), &mut body);
    }
    let mut from_infobox = Vec::new();
    refs(
        &parse_inline(ib.params.get("items").map(String::as_str).unwrap_or(""), r, d),
        &mut from_infobox,
    );
    let disagree = body
        .iter()
        .any(|t| !from_infobox.contains(t))
        || from_infobox.iter().any(|t| !body.contains(t));
    if disagree {
        d.transformation_sources_disagree += 1;
    }
    let mut targets = Vec::new();
    for t in body.into_iter().chain(from_infobox) {
        if !targets.contains(&t) {
            targets.push(t);
        }
    }
    Contributors { targets, disagree }
}
```

Add `pub transformation_sources_disagree: u32` to `Diagnostics` (it derives `Default`,
`Serialize`, `Deserialize` and has a `merge`: add the field to `merge` too, summing).
Declare `mod transformation;` in `lib.rs` and re-export what §5 needs.

- [ ] **Step 4: Run them and watch them pass**

Run: `cargo test -p wiki transformation::`
Expected: PASS, all four.

- [ ] **Step 5: Commit**

```bash
git add crates/wiki/src/transformation.rs crates/wiki/src/lib.rs crates/wiki/src/diagnostics.rs
git commit -m "feat(wiki): a transformation's count and item set, read from its own page"
```

---

## Task 5: the infobox variant

**Files:**
- Modify: `crates/wiki/src/model.rs` (`Infobox`), `crates/wiki/src/infobox.rs`
  (`InfoboxKind`, `infobox_from`, `IGNORED_PARAMS`), `crates/wiki/src/page.rs`
  (`entry_key`, `entry_title`, `parse_page`, the `sections` match)
- Test: `crates/wiki/src/infobox.rs`

**Interfaces:**
- Consumes: `transformation::{requires, contributors}` from Task 4.
- Produces: `Infobox::Transformation { requires: Option<u32>, contributors: Vec<Target>,
  target: Vec<Inline> }`, and `infobox_from(kind, ib, text, r, d)` — note the new `text`
  parameter, which Task 5 threads from `parse_page`.

- [ ] **Step 1: Write the failing test**

```rust
/// Super Bum's real infobox, read on 2026-09-13: `id = n/a`, no `requirement`, and an item
/// list written as positional arguments so `items` is absent. It is the degradation case,
/// and it degrades to "no count, no contributors" rather than to a panic or a default.
#[test]
fn a_malformed_transformation_infobox_degrades_to_unknown() {
    let r = test_resolver();
    let mut d = Diagnostics::default();
    let ib = raw("infobox transformation", &[("dlc", "a"), ("target", "Isaac's bums")]);
    let Infobox::Transformation { requires, contributors, target } =
        infobox_from(InfoboxKind::Transformation, &ib, "", &r, &mut d)
    else {
        panic!()
    };
    assert_eq!(requires, None);
    assert!(contributors.is_empty());
    assert!(matches!(target.first(), Some(Inline::Text { text, .. }) if text == "Isaac's bums"));
}

#[test]
fn the_transformation_template_is_a_kind() {
    assert_eq!(
        InfoboxKind::of("infobox transformation"),
        Some(InfoboxKind::Transformation)
    );
}
```

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p wiki transformation_infobox -- --nocapture`
Expected: FAIL to compile — no `InfoboxKind::Transformation`, no `Infobox::Transformation`,
and `infobox_from` takes five arguments, not six.

- [ ] **Step 3: Make them pass**

1. `Infobox` gains the variant from the spec's §2.2.
2. `InfoboxKind` gains `Transformation`, and `of()` maps `"infobox transformation"`.
3. `infobox_from` gains `text: &str` — documented as "only the transformation completes its
   infobox from the page body" — and the arm:
   ```rust
   InfoboxKind::Transformation => {
       let c = crate::transformation::contributors(ib, text, r, d);
       Infobox::Transformation {
           requires: crate::transformation::requires(text),
           contributors: c.targets,
           target: inline(ib, "target", r, d),
       }
   }
   ```
4. `IGNORED_PARAMS` already covers `id`, `alias`, `name` and `appearance`; `dlc` and
   `description` are `EntryFacts`. `requirement`, `items` and `target` are now fields, so
   `no_silent_parameter` stays green. Run it and confirm rather than assume.
5. `page.rs`: `entry_key` returns `EntryKey::Transformation(number(ib, "id")?)`;
   `entry_title` takes the page title (add `Transformation` to that match arm);
   `parse_page` passes `text` to `infobox_from` and gives the kind the page's sections, like
   every kind except `Achievement`.

- [ ] **Step 4: Run the crate's tests**

Run: `cargo test -p wiki -- --nocapture`
Expected: PASS, `no_silent_parameter` included.

- [ ] **Step 5: Commit**

```bash
git add crates/wiki/src/model.rs crates/wiki/src/infobox.rs crates/wiki/src/page.rs
git commit -m "feat(wiki): the transformation infobox, completed from the page body"
```

---

## Task 6: the dataset map, and the rebuilt artefact

**Files:**
- Modify: `crates/wiki/src/dataset.rs` (`Dataset`, `Counts`, `empty`, `entry`),
  `crates/wiki/src/build.rs` (the `EntryKey` match, `counts`)
- Test: `crates/wiki/src/build.rs`
- Artefact: `dataset/wiki.json`

**Interfaces:**
- Produces: `Dataset.transformations: BTreeMap<u32, Entry>`, `Counts.transformations`, and
  `Dataset::entry(&Target::Transformation { id })` answering `Some`.

- [ ] **Step 1: Write the failing test**

Extend the `raw()` fixture in `crates/wiki/src/build.rs` with a transformation page, then:

```rust
/// The kind that used to answer `None` by construction now answers, and the count travels
/// in `meta` like the other six. `Stage` stays `None`: it has no page.
#[test]
fn a_transformation_has_an_entry_and_a_count() {
    let ds = build(&raw(), &Corrections::default());
    assert_eq!(ds.meta.counts.transformations, 1);
    let e = ds.entry(&Target::Transformation { id: 0 }).unwrap();
    assert_eq!(e.title, "Guppy");
    assert!(ds.entry(&Target::Stage { name: "x".into() }).is_none());
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p wiki a_transformation_has_an_entry -- --nocapture`
Expected: FAIL to compile — no field `transformations` on `Counts`.

- [ ] **Step 3: Make it pass**

Add the map to `Dataset`, the field to `Counts` and to `Dataset::empty()`, the arm to
`Dataset::entry()` — removing `Transformation` from the "`None` by construction" group and
correcting that doc comment — and the `EntryKey::Transformation` arm plus the count in
`build()`. Bump `dataset::SCHEMA_VERSION` to 2: the JSON shape changed, and a file from the
old version must be rejected rather than read half-broken.

- [ ] **Step 4: Run the crate's tests**

Run: `cargo test -p wiki`
Expected: PASS, `json_roundtrip_and_schema_check` included.

- [ ] **Step 5: Commit the code**

```bash
git add crates/wiki/src/dataset.rs crates/wiki/src/build.rs
git commit -m "feat(wiki): transformations are entries of the dataset"
```

- [ ] **Step 6: Rebuild the artefact and check the real numbers**

Run: `pnpm wiki:build`

Then read the summary it prints and confirm against the wiki, not against the code: the
transformation count equals the number of rows in `dataset/raw/cargo/transformation.json`,
`collectible table` has left the unknown-template list, and the disagreement count is at
least one. Run `cargo test -p wiki derived` to prove `wiki.json == build(raw, corrections)`.

- [ ] **Step 7: Commit the artefact separately**

```bash
git add dataset/wiki.json
git commit -m "chore(dataset): rebuild with the transformation entries"
```

---

## Task 7: real-data properties on the dataset

**Files:**
- Create or modify: `crates/wiki/tests/transformations_real.rs`

**Interfaces:**
- Consumes: `wiki::Dataset::embedded()`.
- Produces: nothing; this is the guard that the union and the numeral map did their job on
  the real snapshot rather than only on fixtures.

- [ ] **Step 1: Write the tests**

```rust
//! Properties over the embedded snapshot. Values that belong to the wiki (how many
//! transformations, which items) are read from the dataset; only the shape is asserted.

use wiki::{Dataset, Infobox, Target};

fn transformation(ds: &Dataset, id: u32) -> &Infobox {
    &ds.entry(&Target::Transformation { id }).expect("entry").infobox
}

/// As many entries as the Cargo table has rows — never the literal sixteen, which is what
/// the wiki has today and not a constant. Read from `dataset/raw/` the way the `derived`
/// test does, because the embedded dataset does not carry the table it came from.
#[test]
fn every_row_of_the_table_became_an_entry() {
    let raw = wiki::Raw::load(std::path::Path::new("../../dataset/raw")).expect("raw");
    let ds = Dataset::embedded().expect("embedded dataset");
    assert_eq!(ds.transformations.len(), raw.tables.transformation.len());
}

/// Guppy's set contains a trinket. This is the non-vacuity guard for the whole union: the
/// infobox alone loses exactly this element (measured 2026-09-13), so a union that quietly
/// stopped working would fail here and nowhere else.
#[test]
fn guppys_set_contains_a_trinket() {
    let ds = Dataset::embedded().expect("embedded dataset");
    let Infobox::Transformation { contributors, .. } = transformation(&ds, 0) else {
        panic!("Guppy is a transformation")
    };
    assert!(
        contributors.iter().any(|t| matches!(t, Target::Trinket { .. })),
        "{contributors:?}"
    );
}

/// The cross-check has to be able to speak: at least one page disagrees with itself today.
#[test]
fn the_source_disagreement_is_not_a_dead_counter() {
    let ds = Dataset::embedded().expect("embedded dataset");
    assert!(ds.meta.diagnostics.transformation_sources_disagree > 0);
}

/// A count read for most, unread for some, and never invented. Both halves are asserted:
/// a run where every count came back `None` would otherwise look like a pass.
#[test]
fn some_counts_are_read_and_some_are_honestly_unread() {
    let ds = Dataset::embedded().expect("embedded dataset");
    let (mut read, mut unread) = (0, 0);
    for e in ds.transformations.values() {
        let Infobox::Transformation { requires, .. } = &e.infobox else {
            panic!("{} is not a transformation", e.title)
        };
        match requires {
            Some(n) => {
                assert!(*n > 0, "{}: a threshold of zero is met by nothing", e.title);
                read += 1;
            }
            None => unread += 1,
        }
    }
    assert!(read > 0 && unread > 0, "read {read}, unread {unread}");
}
```

- [ ] **Step 2: Run them**

Run: `cargo test -p wiki --test transformations_real -- --nocapture`
Expected: PASS. If `some_counts_are_read_and_some_are_honestly_unread` fails because
`unread` is zero, do **not** relax it: it means the numeral map matched something it should
not have, and that is the bug it was written to find.

Note what that third test does *not* do: it never names Adult or Super Bum. The spec's §6.4
named them because they are today's two irregular pages, but a test that names a page pins a
fixture of an era — the wiki can fix an infobox tomorrow. Asserting that both outcomes occur
is the same guard and survives the edit; the report (Task 13) is where the current names
belong.

- [ ] **Step 3: Commit**

```bash
git add crates/wiki/tests/transformations_real.rs
git commit -m "test(wiki): the transformation set and count, on the real snapshot"
```

---

## Task 8: the threshold in the rules file

**Files:**
- Modify: `crates/graph/src/rules.rs` (`TransformationRow`, `Requirements`,
  `SCHEMA_VERSION`), `crates/graph/src/generate.rs`
- Test: `crates/graph/tests/generate.rs` (or the module's inline tests, matching the crate's
  existing layout)

**Interfaces:**
- Consumes: `Dataset.transformations`, `Infobox::Transformation`.
- Produces:
  ```rust
  pub struct TransformationRow { pub label: String, pub at_least: Option<u32>, pub items: Vec<Target> }
  // Requirements.transformations: BTreeMap<u32, TransformationRow>
  // Rules::transformation(&self, id: u32) -> Option<&TransformationRow>
  ```
  Task 9 resolves against it.

- [ ] **Step 1: Write the failing test**

```rust
/// `Graph::build` never sees a `Dataset`: the wiki reaches this crate only through the
/// generated rules file. A threshold needs its number and its items at runtime, so they
/// travel there — a row per transformation, emitted even when the count could not be read,
/// because "we have it and cannot read its count" has to stay visible.
#[test]
fn the_generator_emits_a_row_per_transformation() {
    let ds = dataset_with_one_transformation();  // fixture: Guppy, requires 3, two items
    let r = generate(&ds);
    let row = r.transformations.get(&0).expect("Guppy");
    assert_eq!(row.label, "Guppy");
    assert_eq!(row.at_least, Some(3));
    assert_eq!(row.items.len(), 2);
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p graph the_generator_emits_a_row -- --nocapture`
Expected: FAIL to compile — no field `transformations` on `Requirements`.

- [ ] **Step 3: Make it pass**

Add `TransformationRow` and the map to `Requirements` (both `#[serde(rename_all =
"camelCase")]`), bump `rules::SCHEMA_VERSION` to 2, add `Rules::transformation`, and in
`generate()` walk `d.transformations` filling one row each from
`Infobox::Transformation`.

- [ ] **Step 4: Run the crate's tests**

Run: `cargo test -p graph`
Expected: PASS to compile and run; the on-disk `requirements.json` is still schema 1, so
whatever test loads it will fail until Task 11 regenerates it. If that blocks, do Task 11's
regeneration now and re-read its diff there.

- [ ] **Step 5: Commit**

```bash
git add crates/graph/src/rules.rs crates/graph/src/generate.rs
git commit -m "feat(graph): the rules file carries each transformation's set and count"
```

---

## Task 9: `Requirement::Threshold`

**Files:**
- Modify: `crates/graph/src/model.rs`, `crates/graph/src/resolve.rs`
- Test: `crates/graph/tests/resolve.rs`

**Interfaces:**
- Consumes: `Rules::transformation`.
- Produces:
  ```rust
  Requirement::Threshold { transformation: u32, label: String, at_least: u32,
                           of: Vec<ThresholdItem>, unresolved: u32 }
  pub struct ThresholdItem { pub kind: ItemKind, pub id: ItemId }
  ```

- [ ] **Step 1: Write the failing tests**

```rust
/// A transformation resolves to a threshold over the items the rules file names, each one
/// looked up in the catalog the user actually has.
#[test]
fn a_transformation_resolves_to_a_threshold() {
    let (c, rules) = fixture_with_guppy();
    let row = RefRow { target: Target::Transformation { id: 0 }, label: "Guppy".into() };
    let Requirement::Threshold { at_least, of, unresolved, .. } =
        requirement(&c, &rules, &row, None)
    else {
        panic!("expected a threshold")
    };
    assert_eq!(at_least, 3);
    assert_eq!(of.len(), 2);
    assert_eq!(unresolved, 0);
}

/// A count the wiki did not state is not a threshold of zero, which everything meets. The
/// requirement is `Unknown` and the node it holds drops to `Partial`.
#[test]
fn a_transformation_without_a_count_is_unknown_not_a_threshold_of_zero() {
    let (c, rules) = fixture_with_a_countless_transformation();
    let row = RefRow { target: Target::Transformation { id: 11 }, label: "Adult".into() };
    assert!(matches!(
        requirement(&c, &rules, &row, None),
        Requirement::Unknown { .. }
    ));
}

/// An item the catalog does not have is counted, not dropped: Task 10's evaluation needs to
/// know that the tally it has is incomplete.
#[test]
fn contributors_outside_the_catalog_are_counted() {
    let (c, rules) = fixture_with_guppy_and_one_unknown_item();
    let row = RefRow { target: Target::Transformation { id: 0 }, label: "Guppy".into() };
    let Requirement::Threshold { of, unresolved, .. } = requirement(&c, &rules, &row, None)
    else {
        panic!()
    };
    assert_eq!(of.len(), 2);
    assert_eq!(unresolved, 1);
}
```

The three fixtures, all built with `graph::for_tests` and a catalog stub, no file touched:

| fixture | rules file holds | catalog holds |
|---|---|---|
| `fixture_with_guppy` | transformation 0, `atLeast: 3`, two item targets | both items |
| `fixture_with_a_countless_transformation` | transformation 11, `atLeast: None`, one item | that item |
| `fixture_with_guppy_and_one_unknown_item` | transformation 0, `atLeast: 3`, three item targets | two of the three |

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p graph threshold -- --nocapture`
Expected: FAIL to compile — no variant `Threshold`.

- [ ] **Step 3: Make them pass**

Add the variant to `Requirement`, and in `requirement_with` replace the
`Target::Transformation` arm — it currently falls into `from_verdict` with `Stage`, `Room`
and `Pickup`, and must now stand alone:

```rust
        Target::Transformation { id } => match rules.transformation(*id) {
            Some(row) => match row.at_least {
                Some(at_least) => {
                    let mut of = Vec::new();
                    let mut unresolved = 0;
                    for t in &row.items {
                        match catalog_item(c, index, t) {
                            Some((kind, id)) => of.push(ThresholdItem { kind, id }),
                            None => unresolved += 1,
                        }
                    }
                    Requirement::Threshold {
                        transformation: *id,
                        label: label.clone(),
                        at_least,
                        of,
                        unresolved,
                    }
                }
                // No count is not a count of zero.
                None => unknown(),
            },
            None => unknown(),
        },
        Target::Stage { .. } | Target::Room { .. } | Target::Pickup { .. } => {
            from_verdict(rules, &verdict_key, character, unknown)
        }
```

`catalog_item` is the lookup the `Target::Item | Target::Trinket` arm already performs,
lifted into a function so the two arms share it rather than repeat it.

- [ ] **Step 4: Run them and watch them pass**

Run: `cargo test -p graph threshold`
Expected: PASS, all three.

- [ ] **Step 5: Commit**

```bash
git add crates/graph/src/model.rs crates/graph/src/resolve.rs
git commit -m "feat(graph): a transformation resolves to a threshold over its items"
```

---

## Task 10: no edge, and the answer at evaluation

**Files:**
- Modify: `crates/graph/src/build.rs` (the `Requirement` match, `GraphDiagnostic`),
  `crates/graph/src/evaluate.rs` (the `unanswerable` closure)
- Test: `crates/graph/tests/evaluate.rs`

**Interfaces:**
- Consumes: `Requirement::Threshold`.
- Produces: `GraphDiagnostic::ThresholdUnmet { node: u32, label: String, current: u32,
  at_least: u32 }`.

- [ ] **Step 1: Write the failing tests**

```rust
/// Above the line the threshold costs nothing: no edge, no unknown. A node held by nothing
/// else is available now — the same reading a mark and a counter already have.
#[test]
fn a_met_threshold_leaves_the_node_available_now() {
    let (g, profile) = graph_where_three_guppy_items_are_unlocked();
    assert!(matches!(
        g.evaluate(&profile).infos[&65],
        NodeInfo::Computed { available_now: true, blocked_by: 0, .. }
    ));
}

/// Below the line the node cannot be done, and the graph still draws no edge: the
/// prerequisites of "any three of these eight" are a disjunction, and this repo answers a
/// disjunction with an unknown, never with an invented conjunction.
#[test]
fn an_unmet_threshold_makes_the_node_partial_and_says_why() {
    let (g, profile) = graph_where_two_guppy_items_are_unlocked();
    let e = g.evaluate(&profile);
    assert!(matches!(e.infos[&65], NodeInfo::Partial { blocked_by: 0, .. }));
    assert!(e.diagnostics.iter().any(|d| matches!(
        d,
        GraphDiagnostic::ThresholdUnmet { node: 65, current: 2, at_least: 3, .. }
    )));
}

/// Monotonicity, and it is the property the ordering exists for: an unresolved contributor
/// can only ever add to the count, so a threshold already met stays met when one of the
/// items it did not need turns out to be outside the catalog.
#[test]
fn an_unresolved_contributor_never_unmeets_a_met_threshold() {
    let (g, profile) = graph_where_three_guppy_items_are_unlocked_and_one_is_unknown();
    assert!(matches!(
        g.evaluate(&profile).infos[&65],
        NodeInfo::Computed { available_now: true, .. }
    ));
}
```

The three fixtures share one shape — node 65, whose only requirement is Guppy's threshold of
3 over four item targets — and differ in the profile and the catalog:

| fixture | unlocked contributors | outside the catalog |
|---|---|---|
| `graph_where_three_guppy_items_are_unlocked` | 3 | 0 |
| `graph_where_two_guppy_items_are_unlocked` | 2 | 0 |
| `graph_where_three_guppy_items_are_unlocked_and_one_is_unknown` | 3 | 1 |

"Unlocked" means the item's gating achievement is done in the profile, or it has none — the
same predicate `Requirement::Item` already uses; take it from there rather than writing a
second one.

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p graph threshold -- --nocapture`
Expected: FAIL to compile — the `Requirement` match in `build.rs` is not exhaustive.

- [ ] **Step 3: Make them pass**

In `build.rs`, put `Requirement::Threshold { .. }` beside `Mark` and `Counter` in the arm
that pushes neither an edge nor an unknown, with the comment saying why: the edge would be a
disjunction. Add the `ThresholdUnmet` variant to `GraphDiagnostic`.

In `evaluate.rs`, a threshold is judged in one place and the closure only reads the verdict,
so that the diagnostic is produced once per node rather than as a side effect inside a
filter:

```rust
/// Met, unmet, or unanswerable. Satisfaction is tested **first**, and that order is the
/// design: an unresolved contributor can only ever add to the tally, so asking about it
/// first would turn "you can do this" into "we cannot say" for a profile that already can.
enum ThresholdState { Met, Unmet { current: u32 }, Unanswerable }

fn threshold_state(c: &Catalog, done: &dyn Fn(u32) -> bool, r: &Requirement) -> ThresholdState {
    let Requirement::Threshold { at_least, of, unresolved, .. } = r else {
        return ThresholdState::Met; // not a threshold: nothing to judge
    };
    let current = of
        .iter()
        .filter(|i| match c.item(i.kind, i.id).and_then(|it| it.unlocked_by) {
            Some(a) => done(a.0),
            None => true, // nothing gates it
        })
        .count() as u32;
    if current >= *at_least {
        ThresholdState::Met
    } else if *unresolved > 0 {
        ThresholdState::Unanswerable
    } else {
        ThresholdState::Unmet { current }
    }
}
```

The node loop calls it once per `Threshold` requirement: `Met` contributes nothing,
`Unanswerable` and `Unmet` both add one to `unanswerable`, and `Unmet` additionally pushes
`GraphDiagnostic::ThresholdUnmet { node, label, current, at_least }`. Push it the way the
existing `GateSatisfiedByEvidence` loop does, into the diagnostics the evaluation returns —
not into the graph's own, which are built without a profile.

- [ ] **Step 4: Run them and watch them pass**

Run: `cargo test -p graph`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/graph/src/build.rs crates/graph/src/evaluate.rs
git commit -m "feat(graph): a threshold answers at evaluation and never draws an edge"
```

---

## Task 11: the pickup rule — **abandoned on 2026-09-13, do not implement**

> The rule was written, and then checked against the sentences the thirteen references come
> from. They are not words: node 69 reads "unlock all non-DLC secrets and **endings**", 324
> "Collect every entry in the **Bestiary**", 276 "as every character (**tainted
> character**)". Dropping them removes each node's only uninterpreted requirement, which
> stops it being `Partial` and makes it read **available now** — a silent optimistic
> regression that the shrinking count of uninterpreted references would have looked like
> progress. The rule also dropped 45 of 49 pickup targets, `Hard mode` (38 uses) among them.
>
> Spec §4 carries the measurement and what B34 needs instead. The steps below are left
> unexecuted on purpose; the perimeter of this sub-project is now the transformations alone.

### The steps as they were written

**Files:**
- Modify: `crates/graph/src/generate.rs` (`collect_refs`)
- Test: `crates/graph/tests/generate.rs`
- Artefacts: `crates/graph/rules/requirements.json`, `crates/graph/rules/corrections.json`

**Interfaces:**
- Consumes: the `pickup` Cargo table, through the dataset's resolver.
- Produces: a `requirements.json` with no pickup target the wiki does not call a pickup.

- [ ] **Step 1: Write the failing test**

```rust
/// `Inline::Concept` promoted *any* linked concept page to a pickup target, which is right
/// for Red Heart and wrong for `collect`, `ending` and `Bestiary`. The rule is the wiki's
/// own answer to "is this a pickup": the Cargo table it declares them in.
#[test]
fn only_a_concept_the_pickup_table_knows_becomes_a_pickup() {
    let mut out = Vec::new();
    collect_refs(
        &[
            Inline::Concept { page: "Red Heart".into(), label: "Red Heart".into() },
            Inline::Concept { page: "collect".into(), label: "collect".into() },
        ],
        &pickup_names(),   // the set the dataset was built with
        &mut out,
    );
    assert_eq!(out.len(), 1);
    assert!(matches!(out[0].target, Target::Pickup { .. }));
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p graph only_a_concept -- --nocapture`
Expected: FAIL — two rows come back, or the signature does not take the set.

- [ ] **Step 3: Make it pass**

Give `collect_refs` the set of pickup names and drop a concept that is not in it. The names
have to reach the generator: the dataset does not currently carry the pickup table, so
either `Dataset` exposes it or `generate` takes it as a second argument from
`graph-rules.rs`, which reads `dataset/raw/` anyway. Prefer the argument: the dataset is
the app's payload and a lookup table for the generator does not belong in the shipped
binary.

- [ ] **Step 4: Run the crate's tests**

Run: `cargo test -p graph`
Expected: PASS

- [ ] **Step 5: Regenerate, and read the diff**

Run: `pnpm graph:rules`

Then, before staging: `git diff --stat crates/graph/rules/requirements.json`, and read the
diff itself. Check three things — the thirteen pickup targets listed in B34 are gone; **no
target that used to resolve stopped resolving**; the `transformations` map is present with a
row per transformation. `cargo test -p graph --test coverage` is the guard that goes red if
the filter ate a real target: run it and believe it over the diff.

- [ ] **Step 6: Pin the result on the artefact itself**

Reading a diff once is not a guard. Add to `crates/graph/tests/coverage.rs`:

```rust
/// Every pickup target that survived generation is one the wiki declares as a pickup. The
/// diff was read once, on one day; this is what keeps it true. It reads the committed
/// rules file and the committed snapshot, so it needs no sample and never skips.
#[test]
fn no_surviving_pickup_target_is_unknown_to_the_table() {
    let rules = Requirements::load_embedded().expect("requirements.json");
    let raw = wiki::Raw::load(std::path::Path::new("../../dataset/raw")).expect("raw");
    let known: std::collections::BTreeSet<String> = raw
        .tables
        .pickup
        .iter()
        .filter_map(|r| r.get("alias").or_else(|| r.get("_pageName")))
        .map(|s| s.trim().to_lowercase())
        .collect();
    for refs in rules.achievements.values() {
        for row in &refs.refs {
            if let Target::Pickup { name } = &row.target {
                assert!(known.contains(&name.trim().to_lowercase()), "{name}");
            }
        }
    }
}
```

`pickup_names()` in step 1 is that same set, built from the same table: write it once as a
helper the two tests share.

Run: `cargo test -p graph no_surviving_pickup_target`
Expected: PASS

- [ ] **Step 7: Drop the orphaned verdicts**

Remove from `crates/graph/rules/corrections.json` the `pickup:` verdicts whose target no
longer exists and the two `transformation:` ones, which now resolve. Leave every other
verdict alone.

- [ ] **Step 8: Commit code and artefact separately**

```bash
git add crates/graph/src/generate.rs crates/graph/tests/generate.rs
git commit -m "fix(graph): a concept becomes a pickup only if the wiki calls it one"
git add crates/graph/rules/requirements.json crates/graph/rules/corrections.json
git commit -m "chore(graph): regenerate the rules without the invented pickup targets"
```

---

## Task 12: the IPC view and the screen

**Files:**
- Modify: `crates/ipc/src/graph.rs` (`RequirementView`, the mapping), `crates/ipc/src/wiki.rs`
  (`WikiCounts`), `ui/src/lib/ipc/types.ts`, `ui/src/i18n/it.ts`, `ui/src/i18n/en.ts`,
  `ui/src/components/graph/WhyMenu.vue`
- Test: `crates/ipc/tests/graph_shape.rs`, `ui/src/components/graph/__tests__/`

**Interfaces:**
- Consumes: `Requirement::Threshold`.
- Produces: `RequirementView::Threshold { label, current, at_least, of, unresolved, page }`
  with `of: Vec<ThresholdItemView> { item_kind, id, name, unlocked, page }`, and
  `WikiCounts.transformations`.

- [ ] **Step 1: Write the failing tests**

```rust
/// `rename_all` on the enum renames the variants, not the fields inside them: without
/// `rename_all_fields` the wire says `at_least` and `item_kind`, TypeScript reads
/// `undefined`, and nothing fails. Pinned on the JSON, like every other variant here.
#[test]
fn the_threshold_view_is_camel_case_on_the_wire() {
    let v = RequirementView::Threshold {
        label: "Guppy".into(),
        current: 2,
        at_least: 3,
        of: vec![ThresholdItemView {
            item_kind: ItemKindView::Passive,
            id: 211,
            name: "Guppy's Head".into(),
            unlocked: true,
            page: None,
        }],
        unresolved: 0,
        page: None,
    };
    let s = serde_json::to_string(&v).unwrap();
    assert!(s.contains(r#""kind":"threshold""#), "{s}");
    assert!(s.contains(r#""atLeast":3"#), "{s}");
    assert!(s.contains(r#""itemKind":"passive""#), "{s}");
}

/// A met threshold is not a blocker and does not appear among the missing requirements —
/// the same rule `Counter` follows: the view lists what stands in the way, and a threshold
/// already reached does not.
#[test]
fn a_met_threshold_is_not_listed_as_missing() {
    let (c, graph, profile, dataset) = node_whose_only_requirement_is_a_met_threshold();
    let missing = missing_requirements(&c, &graph, &profile, dataset.ok(), 65);
    assert!(
        !missing
            .iter()
            .any(|r| matches!(r, RequirementView::Threshold { .. })),
        "{missing:?}"
    );
}
```

`node_whose_only_requirement_is_a_met_threshold` is Task 10's
`graph_where_three_guppy_items_are_unlocked` plus the catalog and dataset the view builder
needs; `missing_requirements` is whatever the existing mapping function is called in
`crates/ipc/src/graph.rs` — use it, do not write a second one.

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p ipc threshold -- --nocapture`
Expected: FAIL to compile.

- [ ] **Step 3: Make them pass**

Add the variant and `ThresholdItemView` (both `#[serde(rename_all = "camelCase")]`; the enum
already carries `rename_all_fields`), map it in the requirement loop pushing only when
`current < at_least`, and add `transformations` to `WikiCounts` and to `wiki_info`.

- [ ] **Step 4: Mirror it in TypeScript**

In `ui/src/lib/ipc/types.ts`, beside the other members of `RequirementView`:

```ts
  // A transformation: N of a set of items. Like a counter, informative rather than a wall —
  // it only appears while `current < atLeast`. `unresolved` is how many of the wiki's items
  // this catalog does not have; they can only ever add to `current`, never subtract.
  | {
      kind: 'threshold'
      label: string
      current: number
      atLeast: number
      of: ThresholdItemView[]
      unresolved: number
      page: Target | null
    }
```

and add `transformations: number` to the `WikiCounts` interface. No string union: keep
`ItemKindView` as it already is.

- [ ] **Step 5: Draw it**

Add the row to `WhyMenu.vue` — the label, `current` of `atLeast`, and the contributors with
the locked ones marked, each a link like the existing requirement rows. Strings go in
`it.ts` and `en.ts`; no visible string in the template, no hardcoded size or spacing, and
`pnpm scan` is what checks both.

- [ ] **Step 6: Run everything**

Run: `pnpm check`
Expected: green — `cargo fmt`, clippy with `-D warnings`, the workspace tests, `typecheck`,
`ui:test`, `lint`, `format:check`, `scan`. Read the skip lines: a run where the real-data
tests all skipped has not tested Task 7 or Task 10.

- [ ] **Step 7: Commit**

```bash
git add crates/ipc ui/src/lib/ipc/types.ts ui/src/i18n ui/src/components/graph
git commit -m "feat(ipc): a threshold requirement crosses the boundary and draws"
```

---

## Task 13: close the books

- [ ] **Step 1: Confirm the four nodes, with a test and not a grep**

Create `crates/ipc/tests/threshold_real.rs`:

```rust
//! The four nodes B34 left unanswerable. On the real save and the real catalog, through
//! `test-support` — it declares on stderr which file it used, or why it skipped.

/// 65 and 161 are held by Guppy, 178 and 352 by Beelzebub. Before this work each carried a
/// hand-written `Verdict::Unknown` and was `Partial` for that reason alone. The assertion
/// is not "they are available": whether they are depends on the profile. It is that no
/// unanswered *transformation* is left on them.
#[test]
fn the_four_transformation_nodes_are_no_longer_unanswered_for_that_reason() {
    let Some(fixture) = test_support::real_profile_and_catalog() else {
        return; // test-support has already printed `skip: …`
    };
    for id in [65, 161, 178, 352] {
        let missing = missing_requirements(&fixture, id);
        assert!(
            !missing.iter().any(|r| matches!(r, RequirementView::Unknown { label }
                if label.eq_ignore_ascii_case("guppy") || label.eq_ignore_ascii_case("beelzebub"))),
            "node {id} still carries an uninterpreted transformation: {missing:?}"
        );
    }
}
```

Use whatever `test-support` already exposes for "a real save plus the installed catalog" —
do not add a new accessor if one exists, and never open `samples/` by hand.

Run: `cargo test -p ipc --test threshold_real -- --nocapture`
Expected: PASS, or a `skip:` line. **A skip proves nothing**: if it skips, say so in the
report rather than reporting the task as verified.

- [ ] **Step 2: Update the status and the backlog**

`docs/STATUS.md`: a checked box means committed work, never a note. `docs/BACKLOG.md`: B34
closes; add an entry for `player.json` and `stage.json`, downloaded and read by nothing,
with `player.parent` named as the Tainted→base relation.

- [ ] **Step 3: Write the report**

`docs/superpowers/reports/2026-09-13-transformations-report.md`: what was measured, what
changed, and what is still unread — the transformations whose count the numeral map could
not read, and the disagreements the cross-check recorded.

- [ ] **Step 4: Commit**

```bash
git add docs/STATUS.md docs/BACKLOG.md docs/superpowers/reports/2026-09-13-transformations-report.md
git commit -m "docs: the transformations report, and B34 closed"
```
