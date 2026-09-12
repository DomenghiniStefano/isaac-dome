# 3.5d — "Bloccato" says where to go: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Every place the app says *bloccato* turns the blocker's name into a menu entry that
opens that thing's wiki page.

**Architecture:** Rust resolves the page — the frontend holds `{ kind, id }` and a page is keyed
`entity:20.0.0`, so the mapping stays where the catalog is. The mapping already exists inside
search's `documents()`; it moves into one module and both callers use it. Each requirement, and
each collection lock, then carries `page: Option<Target>`, `Some` only when the embedded dataset
really has that page. On the frontend a pure function turns a node (or a lock) into menu groups
carrying a `TabLocation`, a new `dropdown-menu` primitive draws them, and opening one is the
app's existing gesture: click navigates, Ctrl+click opens beside.

**Tech Stack:** Rust (`crates/ipc`, `crates/app`), Vue 3 + TypeScript, Reka UI 2.10.4 via
shadcn-vue, Tailwind v4, Vitest.

**Spec:** `docs/superpowers/specs/2026-09-12-blocked-menu-design.md`

## Global Constraints

- **Branch:** `feature/blocked-menu`, cut from `develop`. One sub-project, one branch.
- **Commits:** Conventional Commits, `type(scope): subject`, English, atomic. **Never** a
  `Co-Authored-By` trailer or any reference to Claude.
- **Every struct crossing the IPC** has `#[serde(rename_all = "camelCase")]`; every enum with
  struct variants also has `rename_all_fields = "camelCase"` — without it `page` is fine but any
  two-word field silently arrives `undefined`.
- **Exhaustiveness is mandatory:** no `_ =>` arm on a closed enum, in Rust or in a TypeScript
  `switch` (use `assertNever`).
- **No string unions in TypeScript:** `const X = { … } as const`. The only exception is a tagged
  union's tag.
- **Frontend:** no `<style>` in SFCs, no hardcoded visual constants (every value a token in
  `@theme`), no `invoke()` in components, no raw `<button>`, no visible string in a template that
  isn't a message. `pnpm scan` enforces all of it.
- **Degrade, never fail:** no dataset, no catalog, no page ⇒ the menu still lists what is
  missing, with every entry disabled. Never a link that leads nowhere.
- **Before declaring anything done:** `pnpm check` (`scripts/check`) green, and read its skip
  lines — `cargo test` hides them, the script runs `--nocapture` for that reason.

---

### Task 0: The branch

- [ ] **Step 1: Cut the branch from `develop`**

```bash
git switch develop
git switch -c feature/blocked-menu
```

- [ ] **Step 2: Confirm the tree is clean and the suite is green before touching anything**

Run: `pnpm check`
Expected: green. If it isn't, stop — this plan must not start on a red suite.

---

### Task 1: One mapping, catalog → wiki `Target`

Today `documents()` in `crates/ipc/src/search.rs` is the only place that knows a boss's page is
identified by its **portrait's file name** and not by its `BossId`. Two more callers are about to
need it. It moves out, and search starts calling it — no behaviour changes in this task.

**Files:**
- Create: `crates/ipc/src/wiki_target.rs`
- Modify: `crates/ipc/src/lib.rs` (declare the module)
- Modify: `crates/ipc/src/search.rs:170-210` (`documents()` calls the new functions)
- Test: `crates/ipc/tests/wiki_target.rs`

**Interfaces:**
- Consumes: `catalog::{Achievement, Boss, Challenge, Character, Item, ItemKind, AchievementId}`,
  `wiki::Target`, `crate::target_sprite::entity_key`.
- Produces: `pub(crate) fn item(&Item) -> Target`, `character(&Character) -> Target`,
  `challenge(&Challenge) -> Target`, `boss(&Boss) -> Option<Target>`,
  `achievement(AchievementId) -> Target`, all in module `crate::wiki_target`. Tasks 2 and 3 call
  them. Also `pub fn wiki_target_for_tests(...)` is **not** added: the test reaches the mapping
  through `documents_for_tests`, which is already public.

- [ ] **Step 1: Write the failing test**

Create `crates/ipc/tests/wiki_target.rs`:

```rust
//! The catalog → wiki page mapping. It is one function because two readers need the same
//! answer: search keys its documents by it, and a requirement links to it. The property below
//! is what stops the two from drifting — a boss is found by its portrait's file name, and a
//! second copy of that rule would be wrong within a release.

use catalog::Catalog;
use ipc::{documents_for_tests, SearchIndex};
use wiki::{Dataset, Target};

const ITEMS: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"105\" gfx=\"d6.png\" name=\"The D6\" /><trinket id=\"97\" gfx=\"t.png\" name=\"Tonsil\" /></items>";
const BOSSES: &[u8] = b"<bossportraits gfxroot=\"gfx/ui/boss/\"><boss id=\"1\" name=\"Monstro\" portrait=\"Portrait_20.0_Monstro.png\" /><boss id=\"2\" name=\"Nameless\" portrait=\"no_key.png\" /></bossportraits>";
const CHALLENGES: &[u8] =
    b"<challenges><challenge id=\"36\" name=\"Scat Man\" startingitems=\"\" /></challenges>";
const PLAYERS: &[u8] =
    b"<players><player id=\"0\" name=\"Isaac\" nameimage=\"i.png\" portrait=\"p.png\" /></players>";

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS.to_vec()),
        "bossportraits.xml" => Some(BOSSES.to_vec()),
        "challenges.xml" => Some(CHALLENGES.to_vec()),
        "players.xml" => Some(PLAYERS.to_vec()),
        _ => None,
    })
}

/// Search keys every catalog entity by the mapping. Whatever the mapping answers, these are
/// the keys that must come out — the test states them, it does not read them back from the
/// code that produced them.
#[test]
fn the_catalog_entities_are_keyed_by_their_wiki_target() {
    let ds = Dataset::empty_for_tests();
    let docs = documents_for_tests(&SearchIndex::build(Ok(&ds)), Some(&catalog()));
    let keys: Vec<&Target> = docs.keys().collect();

    assert!(keys.contains(&&Target::Item { id: 105 }), "a collectible is an item page");
    assert!(keys.contains(&&Target::Trinket { id: 97 }), "a trinket is a trinket page");
    assert!(keys.contains(&&Target::Challenge { number: 36 }), "the wiki calls the field number");
    assert!(keys.contains(&&Target::Character { id: 0 }));
    assert!(
        keys.contains(&&Target::Entity { id: 20, variant: 0, subtype: 0 }),
        "the entity key is read from the portrait's file name, not from the BossId"
    );
    assert_eq!(
        keys.iter().filter(|t| matches!(t, Target::Entity { .. })).count(),
        1,
        "a portrait that declares no key names no page, and is left out rather than guessed"
    );
}
```

- [ ] **Step 2: Run it and watch it pass**

Run: `cargo test -p ipc --test wiki_target`
Expected: PASS. This test pins the *current* behaviour before the refactor — that is its job.
If it fails now, the mapping is not what the plan says it is: stop and re-read `documents()`.

- [ ] **Step 3: Create the module**

Create `crates/ipc/src/wiki_target.rs`:

```rust
//! Catalog record → wiki `Target`: the one place that knows how the game's ids map onto the
//! dataset's page keys. None of it is obvious — a boss is identified by its portrait's file
//! name, a trinket lives on a different page kind from a collectible, and the wiki calls a
//! challenge's number what the catalog calls its id. Search keys its documents by this, and a
//! requirement links by it; a second copy would be wrong within a release.

use catalog::{AchievementId, Boss, Challenge, Character, Item, ItemKind};
use wiki::Target;

use crate::target_sprite::entity_key;

/// `items.xml` keeps collectibles and trinkets in one file; the dataset gives them two page
/// kinds.
pub(crate) fn item(i: &Item) -> Target {
    match i.kind {
        ItemKind::Trinket => Target::Trinket { id: i.id.0 },
        ItemKind::Passive | ItemKind::Active | ItemKind::Familiar => Target::Item { id: i.id.0 },
    }
}

/// The catalog's id already tells the base and Tainted forms apart (B28): the page is the
/// id's, and the name never enters into it.
pub(crate) fn character(c: &Character) -> Target {
    Target::Character { id: c.id.0 }
}

/// The wiki names the field `number`, the catalog names it `id`. Same number.
pub(crate) fn challenge(c: &Challenge) -> Target {
    Target::Challenge { number: c.id.0 }
}

/// The entity key lives in the portrait's file name (`Portrait_20.0_Monstro.png`), exactly as
/// `target_sprite` reads it. A portrait that declares none names no page: `None`, never a
/// guessed variant.
pub(crate) fn boss(b: &Boss) -> Option<Target> {
    entity_key(&b.portrait.path).map(|(id, variant)| Target::Entity {
        id,
        variant,
        subtype: 0,
    })
}

pub(crate) fn achievement(id: AchievementId) -> Target {
    Target::Achievement { id: id.0 }
}
```

- [ ] **Step 4: Declare the module**

In `crates/ipc/src/lib.rs`, beside the other `mod` lines, add:

```rust
mod wiki_target;
```

- [ ] **Step 5: Make `documents()` call it**

In `crates/ipc/src/search.rs`, inside `documents()`, replace the four inline constructions
with calls. The loops keep their shape; only the target expression changes:

```rust
    for i in c.items() {
        join(wiki_target::item(i), c.text(&i.name, en).to_string(), None);
    }
    for p in c.characters() {
        join(
            wiki_target::character(p),
            c.text(&p.name, en).to_string(),
            None,
        );
    }
    for b in c.bosses() {
        // The portrait's file name carries the entity key; a portrait that doesn't declare
        // one names no target and is left out.
        if let Some(target) = wiki_target::boss(b) {
            join(target, b.name.clone(), None);
        }
    }
    for ch in c.challenges() {
        join(wiki_target::challenge(ch), ch.name.clone(), None);
    }
    for a in c.achievements() {
        join(
            wiki_target::achievement(a.id),
            a.text.clone(),
            a.unlock_condition.clone(),
        );
    }
```

Add `use crate::wiki_target;` to the imports and drop the now-unused
`use crate::target_sprite::entity_key` **only if** nothing else in the file uses it — `cargo
clippy -D warnings` will say.

- [ ] **Step 6: Run the tests**

Run: `cargo test -p ipc --test wiki_target --test search && cargo clippy -p ipc --all-targets -- -D warnings`
Expected: PASS, no warnings. The refactor changed no behaviour, so `search`'s own tests are the
proof.

- [ ] **Step 7: Commit**

```bash
git add crates/ipc/src/wiki_target.rs crates/ipc/src/lib.rs crates/ipc/src/search.rs crates/ipc/tests/wiki_target.rs
git commit -m "refactor(ipc): one mapping from a catalog record to its wiki page"
```

---

### Task 2: A requirement carries its page

**Files:**
- Modify: `crates/ipc/src/graph.rs` (`RequirementView`, `missing_view`, `unlock_view`)
- Modify: `crates/ipc/src/queue.rs:145` (`queue_view` passes the dataset through)
- Modify: `crates/app/src/lib.rs` (the `unlock` command and `queue_view_now` hand over the
  embedded dataset)
- Test: `crates/ipc/tests/graph.rs` (the shape tests and one new behaviour test)
- Also modify, mechanically: `crates/ipc/tests/graph_real.rs`, `crates/ipc/tests/unlock_size.rs`
  — every `unlock_view(` call gains the new argument.

**Interfaces:**
- Consumes: `crate::wiki_target::{boss, challenge, character, item}` from Task 1.
- Produces:
  - `RequirementView::{Character, Boss, Challenge, Item}` each gain `page: Option<Target>`;
    `Gate` and `Unknown` do not.
  - `pub fn unlock_view(catalog: Option<&Catalog>, dataset: Option<&Dataset>, flags:
    Option<&[bool]>, graph: Option<&graph::Graph>, eval: Option<&graph::evaluate::Eval>, icon:
    impl FnMut(&IconRef) -> Option<String>) -> UnlockView` — `dataset` is the **second**
    parameter, beside the other source.
  - `pub fn queue_view(...)` gains `dataset: Option<&Dataset>` in the same position, after its
    `catalog`.
  - `fn page_of(dataset: Option<&Dataset>, target: Option<Target>) -> Option<Target>` — private
    to `graph.rs`; Task 3 writes its own, three lines, in `collection.rs`.

- [ ] **Step 1: Write the failing tests**

In `crates/ipc/tests/graph.rs`, extend `requirement_view_shapes` and add one behaviour test.
Replace the existing `requirement_view_shapes` body's first two assertions and add the boss one:

```rust
#[test]
fn requirement_view_shapes() {
    use ipc::RequirementView;
    assert_eq!(
        to_value(RequirementView::Character {
            id: 1,
            name: "Magdalene".into(),
            tainted: false,
            page: Some(Target::Character { id: 1 }),
        })
        .unwrap(),
        json!({
            "kind": "character", "id": 1, "name": "Magdalene", "tainted": false,
            "page": { "kind": "character", "id": 1 }
        })
    );
    assert_eq!(
        to_value(RequirementView::Item {
            item_kind: ItemKindView::Passive,
            id: 35,
            name: "The Bible".into(),
            page: None,
        })
        .unwrap(),
        json!({ "kind": "item", "itemKind": "passive", "id": 35, "name": "The Bible", "page": null }),
        "`kind` is the tag: the item's own kind is `itemKind`, and a fieldless enum is a \
         bare string"
    );
    assert_eq!(
        to_value(RequirementView::Gate {
            label: "The Void".into()
        })
        .unwrap(),
        json!({ "kind": "gate", "label": "The Void" }),
        "a gate is a curated label: it cannot have a page, so it carries no empty one"
    );
    assert_eq!(
        to_value(RequirementView::Unknown {
            label: "Guppy".into()
        })
        .unwrap(),
        json!({ "kind": "unknown", "label": "Guppy" })
    );
}
```

Add `use wiki::Target;` to the file's imports.

A node's `missing` is only filled when a real `graph::Graph` is present, and the embedded rules
are keyed by the game's own achievement ids — a synthetic three-achievement catalog would
produce requirements about entities it doesn't contain. So the *behaviour* is tested where the
ids are real, in `crates/ipc/tests/graph_real.rs`, which skips without `samples/`:

```rust
/// A requirement links to the page the dataset really has, and to nothing else. Real catalog,
/// real graph, real dataset: the mapping is only interesting where the ids are the game's.
#[test]
fn a_blocked_node_links_to_the_pages_the_dataset_has() {
    let Some((c, _, s)) = real() else { return };
    let flags = s.flags(Kind::Achievements).expect("section 1");
    let g = graph::Graph::build(&c, graph::rules::embedded().expect("embedded rules"));
    let e = g.evaluate(Some(&flags));
    let ds = wiki::Dataset::embedded().expect("the dataset is embedded at build time");

    let page_of = |r: &ipc::RequirementView| match r {
        ipc::RequirementView::Character { page, .. }
        | ipc::RequirementView::Boss { page, .. }
        | ipc::RequirementView::Challenge { page, .. }
        | ipc::RequirementView::Item { page, .. } => page.clone(),
        ipc::RequirementView::Gate { .. } | ipc::RequirementView::Unknown { .. } => None,
    };

    let v = unlock_view(Some(&c), Some(ds), Some(&flags), Some(&g), Some(&e), |_| None);
    let linked = v
        .nodes
        .iter()
        .flat_map(|n| n.missing.iter())
        .filter_map(&page_of)
        .inspect(|t| assert!(ds.entry(t).is_some(), "a page that goes out has to exist"))
        .count();
    assert!(linked > 0, "the real profile has blockers the wiki documents");

    // No dataset: the names still come out, and nothing links.
    let without = unlock_view(Some(&c), None, Some(&flags), Some(&g), Some(&e), |_| None);
    assert!(without
        .nodes
        .iter()
        .flat_map(|n| n.missing.iter())
        .all(|r| page_of(r).is_none()));
    assert_eq!(without.totals, v.totals, "only the pages changed");
    assert_eq!(
        without.nodes.iter().map(|n| n.missing.len()).sum::<usize>(),
        v.nodes.iter().map(|n| n.missing.len()).sum::<usize>(),
        "a missing page never removes a requirement"
    );
}
```

The deterministic half of the rule — *`page` is `Some` only when `Dataset::entry` answers* — is
tested without samples in Task 3, where no graph is involved and a hand-built dataset settles it.

- [ ] **Step 2: Run the tests and watch them fail**

Run: `cargo test -p ipc --test graph`
Expected: FAIL — `struct variant ipc::RequirementView::Character has no field named page`, and
`unlock_view` takes 5 arguments.

- [ ] **Step 3: Add the field**

In `crates/ipc/src/graph.rs`, add `use wiki::{Dataset, Target};` and put `page` on the four
variants:

```rust
pub enum RequirementView {
    Character {
        id: u32,
        /// Shared by the base and Tainted forms, like `UnlockTarget::Character`: the flag
        /// beside it is what tells the two apart.
        name: String,
        tainted: bool,
        /// The wiki page that says how *this* is unlocked. `None` is "the dataset has no
        /// page": the name shows, and does not link. Never "no requirement".
        page: Option<Target>,
    },
    Boss {
        id: u32,
        name: String,
        page: Option<Target>,
    },
    Challenge {
        id: u32,
        name: String,
        page: Option<Target>,
    },
    /// `itemKind` and not `kind`: the tag already took that name.
    Item {
        item_kind: ItemKindView,
        id: u32,
        name: String,
        page: Option<Target>,
    },
    /// A curated gate — stage, room, mode. The label is what the wiki calls it. No page by
    /// construction: it is a condition we chose not to resolve to an entity.
    Gate {
        label: String,
    },
    /// Not interpreted. A node carrying one cannot claim "available now".
    Unknown {
        label: String,
    },
}
```

- [ ] **Step 4: Fill it in**

Still in `graph.rs`, above `missing_view`:

```rust
/// A page, only when the dataset really has one. `Some(target)` is a link the screen can
/// follow; `None` is a name it draws without one — never a link that leads nowhere.
fn page_of(dataset: Option<&Dataset>, target: Option<Target>) -> Option<Target> {
    let (ds, t) = (dataset?, target?);
    ds.entry(&t).is_some().then_some(t)
}
```

Give `missing_view` the dataset (`fn missing_view(c: &Catalog, dataset: Option<&Dataset>, node:
&graph::build::Node, flags: &[bool]) -> Vec<RequirementView>`) and fill the four pushes:

```rust
                    out.push(RequirementView::Character {
                        id: id.0,
                        name: c.text(&ch.name, en).to_string(),
                        tainted: ch.tainted,
                        page: page_of(dataset, Some(wiki_target::character(ch))),
                    });
```
```rust
                    out.push(RequirementView::Boss {
                        id: id.0,
                        name: b.name.clone(),
                        page: page_of(dataset, wiki_target::boss(b)),
                    });
```
```rust
                    out.push(RequirementView::Challenge {
                        id: id.0,
                        name: ch.name.clone(),
                        page: page_of(dataset, Some(wiki_target::challenge(ch))),
                    });
```
```rust
                    out.push(RequirementView::Item {
                        item_kind: kind_view(*kind),
                        id: id.0,
                        name: c.text(&i.name, en).to_string(),
                        page: page_of(dataset, Some(wiki_target::item(i))),
                    });
```

Add `use crate::wiki_target;` at the top of the file.

- [ ] **Step 5: Thread the dataset through the signatures**

`unlock_view` gains `dataset: Option<&Dataset>` as its second parameter and passes it to
`missing_view`. `queue_view` in `crates/ipc/src/queue.rs` gains the same parameter, in the same
position after its catalog, and passes it at line ~145.

Run: `cargo check --workspace`
The compiler now names every remaining call site. Update each one:
- `crates/app/src/lib.rs`, the `unlock` command and `queue_view_now`: pass
  `wiki::Dataset::embedded().ok()`. An absent dataset is expected and is **not** cached as a
  failure — `embedded()` is already a `OnceLock` behind the scenes.
- `crates/ipc/tests/graph.rs`, `graph_real.rs`, `unlock_size.rs`: pass `None` unless the test is
  about pages, in which case pass its dataset.

- [ ] **Step 6: Run the tests**

Run: `cargo test -p ipc && cargo clippy --workspace --all-targets -- -D warnings`
Expected: PASS, no warnings.

- [ ] **Step 7: Commit**

```bash
git add crates/ipc/src/graph.rs crates/ipc/src/queue.rs crates/app/src/lib.rs crates/ipc/tests/
git commit -m "feat(ipc): a requirement carries the wiki page that explains it"
```

---

### Task 3: The lock carries its page

**Files:**
- Modify: `crates/ipc/src/collection.rs` (`LockView`, `lock_of`, `collection_view`)
- Modify: `crates/app/src/lib.rs` (the `collection` command)
- Test: `crates/ipc/tests/collection.rs`

**Interfaces:**
- Consumes: `crate::wiki_target::achievement` from Task 1.
- Produces: `LockView::{Unlocked, Locked, Unknown}` each gain `page: Option<Target>`; `Free` does
  not. `pub fn collection_view(catalog: Option<&Catalog>, dataset: Option<&Dataset>, items:
  Option<&[bool]>, achievements: Option<&[bool]>, icon: impl FnMut(&IconRef) -> Option<String>)
  -> CollectionView`.

- [ ] **Step 1: Write the failing test**

Add to `crates/ipc/tests/collection.rs`:

The file's existing fixtures already give what is needed: item 1 points at achievement 1 (done),
item 5 at achievement 2 (not done), and `DONE` is `[false, true, false]`.

```rust
/// The lock links to the achievement's page, and only when the dataset has it. `free` carries
/// no page because nothing unlocks the item — a different sentence from "no page".
#[test]
fn a_lock_carries_the_achievement_page_only_when_the_dataset_has_it() {
    let mut ds = Dataset::empty_for_tests();
    // Achievement 2 has a page, achievement 1 has none: one of each, in one view.
    ds.achievements.insert(
        2,
        Entry {
            title: "t2".into(),
            revid: 1,
            infobox: Infobox::Achievement {
                description: String::new(),
                requirements: vec![],
                unlocks: None,
            },
            sections: vec![],
        },
    );

    let v = collection_view(
        Some(&catalog()),
        Some(&ds),
        Some(&SLOTS),
        Some(&DONE),
        |_| None,
    );
    let lock = |id: u32| {
        to_value(
            &v.items
                .iter()
                .find(|i| i.id == id)
                .expect("the item is in the catalog")
                .lock,
        )
        .unwrap()
    };
    assert_eq!(lock(5)["kind"], "locked");
    assert_eq!(
        lock(5)["page"],
        json!({ "kind": "achievement", "id": 2 }),
        "the badge's menu opens the achievement that unlocks the item"
    );
    assert_eq!(
        lock(1)["page"],
        json!(null),
        "the dataset has no page for achievement 1: the name shows, and does not link"
    );
    assert_eq!(lock(2)["kind"], "free", "nothing unlocks item 2");
    assert_eq!(
        lock(2).get("page"),
        None,
        "a free item carries no page key at all: there is nothing to open"
    );

    // No dataset at all: every lock reads the same, and nothing links.
    let without = collection_view(
        Some(&catalog()),
        None,
        Some(&SLOTS),
        Some(&DONE),
        |_| None,
    );
    assert!(without.items.iter().all(|i| match &i.lock {
        LockView::Free => true,
        LockView::Unlocked { page, .. }
        | LockView::Locked { page, .. }
        | LockView::Unknown { page, .. } => page.is_none(),
    }));
}
```

Add `use wiki::{Dataset, Entry, Infobox};` to the file's imports. The existing `view()` helper
calls `collection_view` with four arguments: give it the new parameter as `None` so every other
test in the file keeps meaning what it meant.

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p ipc --test collection`
Expected: FAIL — `collection_view` takes 4 arguments, and `LockView::Locked` has no field `page`.

- [ ] **Step 3: Add the field and fill it in**

In `crates/ipc/src/collection.rs`, add `use wiki::{Dataset, Target};` and `use
crate::wiki_target;`, then:

```rust
pub enum LockView {
    /// Nothing unlocks it: it is in the game from the start, so there is no page to open.
    Free,
    Unlocked {
        achievement: u32,
        text: Option<String>,
        page: Option<Target>,
    },
    /// Its achievement isn't done: the item can't appear in a run yet.
    Locked {
        achievement: u32,
        text: Option<String>,
        page: Option<Target>,
    },
    /// Section 1 wasn't read: whether the achievement is done isn't known.
    Unknown {
        achievement: u32,
        text: Option<String>,
        page: Option<Target>,
    },
}
```

```rust
/// A page, only when the dataset really has one — the same rule as a requirement's.
fn page_of(dataset: Option<&Dataset>, target: Target) -> Option<Target> {
    dataset?.entry(&target).is_some().then_some(target)
}

fn lock_of(
    c: &Catalog,
    dataset: Option<&Dataset>,
    unlocked_by: Option<AchievementId>,
    achievements: Option<&[bool]>,
) -> LockView {
    let Some(a) = unlocked_by else {
        return LockView::Free;
    };
    let achievement = a.0;
    let text = c.achievement(a).map(|x| x.text.clone());
    let page = page_of(dataset, wiki_target::achievement(a));
    match achievements.map(|f| f.get(achievement as usize).copied().unwrap_or(false)) {
        None => LockView::Unknown { achievement, text, page },
        Some(true) => LockView::Unlocked { achievement, text, page },
        Some(false) => LockView::Locked { achievement, text, page },
    }
}
```

Give `collection_view` the `dataset: Option<&Dataset>` parameter (second, after the catalog) and
pass it to `lock_of`.

- [ ] **Step 4: Update the call sites**

Run: `cargo check --workspace` and fix what it names: the `collection` command in
`crates/app/src/lib.rs` passes `wiki::Dataset::embedded().ok()`; the other tests in
`crates/ipc/tests/collection.rs` and `collection_real.rs` pass `None`.

- [ ] **Step 5: Run the tests**

Run: `cargo test -p ipc && cargo clippy --workspace --all-targets -- -D warnings`
Expected: PASS, no warnings.

- [ ] **Step 6: Commit**

```bash
git add crates/ipc/src/collection.rs crates/app/src/lib.rs crates/ipc/tests/
git commit -m "feat(ipc): a collection lock carries its achievement's wiki page"
```

---

### Task 4: The TypeScript mirror, and the fixtures that predate it

The design pack's `unlock.json` and `collection.json` were exported before this field existed.
They must not become a second source of truth: the fixture layer fills `page: null` and warns
once, exactly as it already does for a character's `tainted` flag.

**Files:**
- Modify: `ui/src/lib/ipc/types.ts:244-252` (`RequirementView`), `:580-584` (`LockView`)
- Modify: `ui/src/lib/ipc/fixtures/graph.ts`, `ui/src/lib/ipc/fixtures/collection.ts`
- Test: `ui/src/lib/ipc/fixtures/graph.test.ts`, `ui/src/lib/ipc/fixtures/collection.test.ts`

**Interfaces:**
- Produces: `RequirementView` variants `character | boss | challenge | item` each with
  `page: Target | null`; `LockView` variants `unlocked | locked | unknown` each with
  `page: Target | null`. Tasks 5 and 7 read them.

- [ ] **Step 1: Mirror the wire types by hand**

In `ui/src/lib/ipc/types.ts`:

```ts
export type RequirementView =
  // `tainted` is part of the identity, not decoration: the two forms of a character share
  // the game's name, so the name alone names both (`docs/BACKLOG.md` B28).
  // `page` is the wiki page that says how this is unlocked; `null` means the dataset has no
  // page for it — the name shows, and does not link. Never "no requirement".
  | {
      kind: 'character'
      id: number
      name: string
      tainted: boolean
      page: Target | null
    }
  | { kind: 'boss'; id: number; name: string; page: Target | null }
  | { kind: 'challenge'; id: number; name: string; page: Target | null }
  | {
      kind: 'item'
      itemKind: ItemKindView
      id: number
      name: string
      page: Target | null
    }
  // A gate and an unknown condition carry no page by construction: they are labels we chose
  // not to resolve to an entity.
  | { kind: 'gate'; label: string }
  | { kind: 'unknown'; label: string }
```

```ts
export type LockView =
  | { kind: 'free' }
  | {
      kind: 'unlocked'
      achievement: number
      text: string | null
      page: Target | null
    }
  | {
      kind: 'locked'
      achievement: number
      text: string | null
      page: Target | null
    }
  | {
      kind: 'unknown'
      achievement: number
      text: string | null
      page: Target | null
    }
```

- [ ] **Step 2: Write the failing fixture test**

In `ui/src/lib/ipc/fixtures/graph.test.ts`:

```ts
it('fills the page a pack exported before the field existed', () => {
  const { unlock } = graphAnswers({ withArt: false, withCatalog: true })
  const missing = unlock.nodes.flatMap((n) => n.missing)
  expect(missing.length).toBeGreaterThan(0)
  // An absent key and an explicit null read the same in a template and type-check
  // differently: the fixture has to answer null, not nothing.
  missing.forEach((r) => {
    if (r.kind === 'gate' || r.kind === 'unknown') return
    expect(r.page === null || typeof r.page === 'object').toBe(true)
  })
})
```

Put it in the existing `describe('graphAnswers with the game installed')` block, which already
calls `graphAnswers({ withArt: false, withCatalog: true })`.

- [ ] **Step 3: Run it and watch it fail**

Run: `pnpm ui:test -- graph.test`
Expected: FAIL — `page` is `undefined` on the pack's requirements.

- [ ] **Step 4: Fill it in the fixture layer**

In `ui/src/lib/ipc/fixtures/graph.ts`, beside `withForm`:

```ts
// The design pack predates the requirement's page (spec 3.5d): a pack exported before it
// links nothing, and says so once rather than reading as "the dataset has no page".
let warnedAboutPages = false
const withPage = (r: RequirementView): RequirementView => {
  if (r.kind === 'gate' || r.kind === 'unknown') return r
  if (r.page !== undefined) return r
  if (!warnedAboutPages) {
    warnedAboutPages = true
    console.warn(
      "graph fixture: the design pack's unlock.json predates the requirement's page; nothing links until the next pnpm design:export on a machine with the game",
    )
  }
  return { ...r, page: null }
}
```

and apply it where `withForm` is applied: `missing: node.missing.map((r) => withPage(withForm(r)))`.

Do the same in `ui/src/lib/ipc/fixtures/collection.ts` for `lock`, with its own one-shot warning
and `free` left alone.

- [ ] **Step 5: Run the tests**

Run: `pnpm ui:test && pnpm typecheck`
Expected: PASS. `typecheck` is the real gate here: it is what proves the mirror matches every
reader.

- [ ] **Step 6: Commit**

```bash
git add ui/src/lib/ipc/types.ts ui/src/lib/ipc/fixtures/
git commit -m "feat(ui): mirror the requirement and lock page on the wire"
```

---

### Task 5: The menu's model, as a pure function

The repo tests logic, not components (there is no `mount()` anywhere in `ui/src`): presentation
is checked on the Kit page. So everything the menu decides lives in a pure function, and the
component that draws it holds nothing worth a test.

**Files:**
- Create: `ui/src/lib/graph/whyMenu.ts`
- Create: `ui/src/lib/graph/whyMenu.test.ts`
- Modify: `ui/src/lib/graph/nodeState.ts` (`missingGroups` returns entries, not bare names)
- Modify: `ui/src/lib/graph/nodeState.test.ts` (the two assertions that read `names`)

**Interfaces:**
- Consumes: `RequirementView`, `LockView`, `UnlockNode`, `CollectionItem` from
  `@/lib/ipc/types`; `pageLocation` from `@/lib/wiki/category`; `characterLabel` from
  `@/lib/graph/characterName`; `MessageKey`/`MessageSchema` from `@/i18n`.
- Produces:
  ```ts
  export interface WhyEntry { key: string; name: string; location: TabLocation | null }
  export interface WhyGroup { label: MessageKey<MessageSchema>; entries: WhyEntry[] }
  export const nodeWhy: (node: UnlockNode, t: Translate) => WhyGroup[]
  export const lockWhy: (lock: LockView, t: Translate) => WhyGroup[]
  ```
  Task 7's component consumes exactly these.

- [ ] **Step 1: Write the failing test**

Create `ui/src/lib/graph/whyMenu.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import type { LockView, UnlockNode } from '@/lib/ipc/types'
import { RouteName } from '@/router/routeTable'
import { lockWhy, nodeWhy } from './whyMenu'

const t = (key: string) => key

const node = (missing: UnlockNode['missing']): UnlockNode => ({
  achievement: { kind: 'known', id: 1, text: 'Dad’s Note', hint: null, iconUrl: null },
  done: false,
  unlocks: [],
  origin: null,
  missing,
  graph: { kind: 'computed', availableNow: false, blockedBy: missing.length, fanOut: 0, stepsMissing: 1 },
})

describe('nodeWhy', () => {
  it('groups by kind, in the order the why is told', () => {
    const groups = nodeWhy(
      node([
        { kind: 'boss', id: 19, name: 'Gish', page: null },
        { kind: 'character', id: 1, name: 'Magdalene', tainted: false, page: null },
      ]),
      t,
    )
    expect(groups.map((g) => g.label)).toEqual(['graph.why.character', 'graph.why.boss'])
  })

  it('an entry with a page carries the location that opens it', () => {
    const [group] = nodeWhy(
      node([{ kind: 'item', itemKind: 'passive', id: 105, name: 'The D6', page: { kind: 'item', id: 105 } }]),
      t,
    )
    expect(group?.entries[0]?.location).toEqual({
      name: RouteName.Wiki,
      query: { category: 'items', page: 'item:105' },
    })
  })

  it('an entry with no page is still an entry, with nowhere to go', () => {
    const [group] = nodeWhy(node([{ kind: 'gate', label: 'greedmode' }]), t)
    expect(group?.entries).toEqual([
      { key: 'gate-greedmode', name: 'greedmode', location: null },
    ])
  })

  it('a node with nothing missing has no groups: the badge is not a trigger', () => {
    expect(nodeWhy(node([]), t)).toEqual([])
  })

  it('a partial node keeps its why: the state does not decide whether there is a menu', () => {
    const partial: UnlockNode = {
      ...node([{ kind: 'unknown', label: 'Guppy' }]),
      graph: { kind: 'partial', blockedBy: 0, fanOut: 0, unknown: 1 },
    }
    expect(nodeWhy(partial, t)).toEqual([
      {
        label: 'graph.why.unknown',
        entries: [{ key: 'unknown-Guppy', name: 'Guppy', location: null }],
      },
    ])
  })
})

describe('lockWhy', () => {
  it('one group, one entry: the achievement that opens the item', () => {
    const lock: LockView = { kind: 'locked', achievement: 1, text: 'Dad’s Note', page: { kind: 'achievement', id: 1 } }
    expect(lockWhy(lock, t)).toEqual([
      {
        label: 'collection.lockedBy',
        entries: [
          {
            key: 'achievement-1',
            name: 'Dad’s Note',
            location: { name: RouteName.Wiki, query: { category: 'achievements', page: 'achievement:1' } },
          },
        ],
      },
    ])
  })

  it('nothing unlocks a free item: no menu at all', () => {
    expect(lockWhy({ kind: 'free' }, t)).toEqual([])
  })

  it('names the achievement by its number when the catalog has no text for it', () => {
    const lock: LockView = { kind: 'unknown', achievement: 7, text: null, page: null }
    expect(lockWhy(lock, t)[0]?.entries[0]?.name).toBe('collection.achievement 7')
  })
})
```

- [ ] **Step 2: Run it and watch it fail**

Run: `pnpm ui:test -- whyMenu`
Expected: FAIL — `whyMenu.ts` does not exist.

- [ ] **Step 3: Make `missingGroups` carry entries**

In `ui/src/lib/graph/nodeState.ts`, add the location beside the name and change the group's
shape:

```ts
import { pageLocation } from '@/lib/wiki/category'
import type { TabLocation } from '@/router/routeTable'

// What stands in the way, and where to read about it: `location` is `null` when the dataset
// has no page — a name the screen draws without a link, never a link that leads nowhere.
export interface RequirementEntry {
  key: string
  name: string
  location: TabLocation | null
}

export interface RequirementGroup {
  kind: RequirementKind
  entries: RequirementEntry[]
}

// A key that is stable per row and unique in the list: the kind and what identifies it. A
// gate and an unknown have only their label, and two of them never repeat inside one node.
const requirementKey = (requirement: RequirementView): string => {
  switch (requirement.kind) {
    case 'character':
    case 'boss':
    case 'challenge':
    case 'item':
      return `${requirement.kind}-${requirement.id}`
    case 'gate':
    case 'unknown':
      return `${requirement.kind}-${requirement.label}`
    default:
      return assertNever(requirement)
  }
}

const requirementLocation = (requirement: RequirementView): TabLocation | null => {
  switch (requirement.kind) {
    case 'character':
    case 'boss':
    case 'challenge':
    case 'item':
      return requirement.page ? pageLocation(requirement.page) : null
    case 'gate':
    case 'unknown':
      return null
    default:
      return assertNever(requirement)
  }
}
```

and in `missingGroups`, replace `names: entries.map((r) => requirementName(r, t))` with:

```ts
      ? [
          {
            kind,
            entries: entries.map((r) => ({
              key: requirementKey(r),
              name: requirementName(r, t),
              location: requirementLocation(r),
            })),
          },
        ]
```

Update the assertions in `ui/src/lib/graph/nodeState.test.ts` that read `names`: they now read
`entries.map((e) => e.name)`. **Do not** weaken what they assert.

- [ ] **Step 4: Write `whyMenu.ts`**

Create `ui/src/lib/graph/whyMenu.ts`:

```ts
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { assertNever } from '@/lib/assertNever'
import type { LockView, UnlockNode } from '@/lib/ipc/types'
import { pageLocation } from '@/lib/wiki/category'
import type { TabLocation } from '@/router/routeTable'
import { RequirementKind, missingGroups } from './nodeState'

type Translate = (
  key: MessageKey<MessageSchema>,
  params?: Record<string, unknown>,
) => string

// The menu behind a badge, as data: one group per kind, one entry per thing in the way, and
// for each the place that says how *it* is unlocked. Pure, so the model is what gets tested
// and the component that draws it holds no decision (DESIGN-BRIEF.md §7.1).
export interface WhyEntry {
  key: string
  name: string
  /** `null` = nowhere to go: the entry shows, disabled. Never a link that leads nowhere. */
  location: TabLocation | null
}

export interface WhyGroup {
  label: MessageKey<MessageSchema>
  entries: WhyEntry[]
}

const kindLabel: Record<RequirementKind, MessageKey<MessageSchema>> = {
  [RequirementKind.Character]: 'graph.why.character',
  [RequirementKind.Boss]: 'graph.why.boss',
  [RequirementKind.Challenge]: 'graph.why.challenge',
  [RequirementKind.Item]: 'graph.why.item',
  [RequirementKind.Gate]: 'graph.why.gate',
  [RequirementKind.Unknown]: 'graph.why.unknown',
}

export const nodeWhy = (node: UnlockNode, t: Translate): WhyGroup[] =>
  missingGroups(node, t).map((group) => ({
    label: kindLabel[group.kind],
    entries: group.entries,
  }))

// The Collection's lock: one group, one entry — the achievement that opens the item. `free`
// has nothing to say, and says nothing.
export const lockWhy = (lock: LockView, t: Translate): WhyGroup[] => {
  switch (lock.kind) {
    case 'free':
      return []
    case 'unlocked':
    case 'locked':
    case 'unknown':
      return [
        {
          label: 'collection.lockedBy',
          entries: [
            {
              key: `achievement-${lock.achievement}`,
              name:
                lock.text ?? `${t('collection.achievement')} ${lock.achievement}`,
              location: lock.page ? pageLocation(lock.page) : null,
            },
          ],
        },
      ]
    default:
      return assertNever(lock)
  }
}
```

- [ ] **Step 5: Run the tests**

Run: `pnpm ui:test && pnpm typecheck && pnpm lint`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add ui/src/lib/graph/whyMenu.ts ui/src/lib/graph/whyMenu.test.ts ui/src/lib/graph/nodeState.ts ui/src/lib/graph/nodeState.test.ts
git commit -m "feat(ui): the why of a blocked node, as a menu model"
```

---

### Task 6: The `dropdown-menu` primitive

**Files:**
- Create: `ui/src/components/ui/dropdown-menu/DropdownMenu.vue`, `DropdownMenuTrigger.vue`,
  `DropdownMenuContent.vue`, `DropdownMenuLabel.vue`, `DropdownMenuItem.vue`, `index.ts`
- Create: `ui/src/kit/sections/DropdownMenuSection.vue`
- Modify: `ui/src/kit/KitPage.vue`

**Interfaces:**
- Produces: `DropdownMenu`, `DropdownMenuTrigger`, `DropdownMenuContent`, `DropdownMenuLabel`,
  `DropdownMenuItem` exported from `@/components/ui/dropdown-menu`. `DropdownMenuItem` forwards
  Reka's `select` event and its `disabled` prop. Task 7 uses them.

- [ ] **Step 1: Write the five components**

Follow `ui/src/components/ui/popover/` exactly — same `data-slot` attributes, same
`useForwardPropsEmits`, same `reactiveOmit(props, 'class')`, tokens only.

`DropdownMenu.vue`:

```vue
<script setup lang="ts">
import type { DropdownMenuRootEmits, DropdownMenuRootProps } from 'reka-ui'
import { DropdownMenuRoot, useForwardPropsEmits } from 'reka-ui'

const props = defineProps<DropdownMenuRootProps>()
const emits = defineEmits<DropdownMenuRootEmits>()

const forwarded = useForwardPropsEmits(props, emits)
</script>

<template>
  <DropdownMenuRoot v-slot="slotProps" data-slot="dropdown-menu" v-bind="forwarded">
    <slot v-bind="slotProps" />
  </DropdownMenuRoot>
</template>
```

`DropdownMenuTrigger.vue`:

```vue
<script setup lang="ts">
import type { DropdownMenuTriggerProps } from 'reka-ui'
import { DropdownMenuTrigger } from 'reka-ui'

const props = defineProps<DropdownMenuTriggerProps>()
</script>

<template>
  <DropdownMenuTrigger data-slot="dropdown-menu-trigger" v-bind="props">
    <slot />
  </DropdownMenuTrigger>
</template>
```

`DropdownMenuContent.vue`: same shape as `PopoverContent.vue`, with
`DropdownMenuContent`/`DropdownMenuPortal` and the same class list minus the fixed width — a
menu is as wide as its longest entry, bounded:

```vue
<script setup lang="ts">
import type { DropdownMenuContentEmits, DropdownMenuContentProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import {
  DropdownMenuContent,
  DropdownMenuPortal,
  useForwardPropsEmits,
} from 'reka-ui'
import { cn } from '@/lib/cn'
import { Align } from '@/lib/constants/placement'

defineOptions({ inheritAttrs: false })

const props = withDefaults(
  defineProps<DropdownMenuContentProps & { class?: HTMLAttributes['class'] }>(),
  { align: Align.Start, sideOffset: 4 },
)
const emits = defineEmits<DropdownMenuContentEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <DropdownMenuPortal>
    <DropdownMenuContent
      data-slot="dropdown-menu-content"
      v-bind="{ ...$attrs, ...forwarded }"
      :class="
        cn(
          'z-50 flex max-w-(--reka-dropdown-menu-content-available-width) min-w-48 animate-panel-rise flex-col border border-input bg-popover py-1 text-popover-foreground',
          props.class,
        )
      "
    >
      <slot />
    </DropdownMenuContent>
  </DropdownMenuPortal>
</template>
```

`DropdownMenuItem.vue` — the classes are `SelectItem.vue`'s, which is the repo's existing answer
for "a row in an overlay that can be highlighted or disabled":

```vue
<script setup lang="ts">
import type { DropdownMenuItemEmits, DropdownMenuItemProps } from 'reka-ui'
import type { HTMLAttributes } from 'vue'
import { reactiveOmit } from '@vueuse/core'
import { DropdownMenuItem, useForwardPropsEmits } from 'reka-ui'
import { cn } from '@/lib/cn'

const props = defineProps<
  DropdownMenuItemProps & { class?: HTMLAttributes['class'] }
>()
const emits = defineEmits<DropdownMenuItemEmits>()

const delegatedProps = reactiveOmit(props, 'class')
const forwarded = useForwardPropsEmits(delegatedProps, emits)
</script>

<template>
  <!-- A disabled entry stays readable and stops taking the gesture: it says "we can't take
       you there", which is not the same as not being in the way. -->
  <DropdownMenuItem
    data-slot="dropdown-menu-item"
    v-bind="forwarded"
    :class="
      cn(
        'flex w-full cursor-default items-center gap-2 py-1.5 pr-6 pl-3 text-row text-foreground outline-none select-none data-[disabled]:pointer-events-none data-[disabled]:text-faint-foreground data-[highlighted]:bg-secondary',
        props.class,
      )
    "
  >
    <slot />
  </DropdownMenuItem>
</template>
```

`DropdownMenuLabel.vue` — the same shape, wrapping Reka's `DropdownMenuLabel`, with
`SelectLabel.vue`'s classes (read that file and copy them; it is the group heading of an overlay
list and this is the same thing). Both files use only tokens that already exist: `pnpm scan`
rejects a raw value, and the fix is a token in `@theme`, never an exemption.

`index.ts` exports the five, in the shape `popover/index.ts` uses.

- [ ] **Step 2: Add the Kit section**

Create `ui/src/kit/sections/DropdownMenuSection.vue` on the model of `PopoverSection.vue`: a
trigger badge, two groups with labels, one disabled entry among them. Register it in
`ui/src/kit/KitPage.vue` beside `PopoverSection`.

- [ ] **Step 3: Check the rules and the types**

Run: `pnpm typecheck && pnpm lint && pnpm scan`
Expected: `0 violations`. A violation here means a raw value slipped into a class: turn it into a
token in `@theme`, do **not** add an exemption.

- [ ] **Step 4: Look at it**

Run: `pnpm ui:dev`, open `#kit`, find the section.
Check: the menu opens on click and on Enter, arrows move between entries, Esc closes and focus
returns to the trigger, the disabled entry can't be chosen and doesn't take focus, and it reads
correctly in both themes.

- [ ] **Step 5: Commit**

```bash
git add ui/src/components/ui/dropdown-menu/ ui/src/kit/
git commit -m "feat(ui): a dropdown-menu primitive, on Reka"
```

---

### Task 7: The badge opens the menu, on both screens

**Files:**
- Create: `ui/src/components/graph/WhyMenu.vue`
- Modify: `ui/src/components/graph/NodeStateBadge.vue`
- Modify: `ui/src/screens/collection/CollectionRow.vue`

**Interfaces:**
- Consumes: `WhyGroup`/`nodeWhy`/`lockWhy` (Task 5), the `dropdown-menu` primitive (Task 6),
  `useTabsStore` and `useGestureModifiers`.
- Produces: `WhyMenu.vue` with props `{ groups: WhyGroup[]; label: string }` and a default slot
  for the trigger. Nothing else consumes it.

**Why there is no unit test here.** `ui/` mounts no component anywhere (`mount(` appears only in
`main.ts`): the repo tests logic and looks at presentation on the Kit page. Everything this
component decides was already decided in Task 5's pure functions; what is left is one `if` on
`ctrl`, copied from `SearchPalette.vue`, which has no test there either. It is checked by eye in
Task 8 — deliberately, not by omission.

- [ ] **Step 1: Write the component**

Create `ui/src/components/graph/WhyMenu.vue`:

```vue
<script setup lang="ts">
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu'
import { useGestureModifiers } from '@/composables/useGestureModifiers'
import { useMessages } from '@/i18n'
import type { WhyGroup } from '@/lib/graph/whyMenu'
import type { TabLocation } from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'

const props = defineProps<{ groups: WhyGroup[]; label: string }>()
const { t } = useMessages()
const tabs = useTabsStore()

// Reka answers with `select`, which says *that* an entry was chosen and not *how*: reading
// the click instead fires twice, because Reka replays it on the item. So the modifier comes
// from the window, exactly as the search palette reads it.
const { ctrl } = useGestureModifiers()

// The app's one gesture: a click navigates the active tab, Ctrl opens the page beside it.
const open = (location: TabLocation | null) => {
  if (!location) return
  if (ctrl.value) tabs.open(location)
  else tabs.navigate(location)
}
</script>

<template>
  <!-- The badge says the state, its menu says why and where to read about it: one overlay,
       not a tooltip to read and a menu to use (spec 3.5d, Decision 4). -->
  <DropdownMenu v-if="props.groups.length > 0">
    <DropdownMenuTrigger as-child>
      <slot />
    </DropdownMenuTrigger>
    <DropdownMenuContent :aria-label="props.label">
      <template v-for="group in props.groups" :key="group.label">
        <DropdownMenuLabel>{{ t(group.label) }}</DropdownMenuLabel>
        <DropdownMenuItem
          v-for="entry in group.entries"
          :key="entry.key"
          :disabled="entry.location === null"
          @select="open(entry.location)"
          >{{ entry.name }}</DropdownMenuItem
        >
      </template>
    </DropdownMenuContent>
  </DropdownMenu>
  <slot v-else />
</template>
```

- [ ] **Step 2: Rewrite `NodeStateBadge.vue`**

Drop the `Tooltip` import and its markup, drop the local `kindText` map (it moved into
`whyMenu.ts`), and wrap the badge:

```vue
<template>
  <WhyMenu :groups="groups" :label="t('graph.why.title')">
    <Badge :variant="variant[state]" :tabindex="groups.length > 0 ? 0 : undefined">{{
      label
    }}</Badge>
  </WhyMenu>
</template>
```

with `const groups = computed(() => nodeWhy(props.node, t))`. Keep the `label` computed and the
`variant`/`stateText` maps exactly as they are.

- [ ] **Step 3: Rewrite the Collection's state cell**

In `ui/src/screens/collection/CollectionRow.vue`, replace the `Tooltip` block with the same
wrapper, `const groups = computed(() => lockWhy(props.item.lock, t))`, and delete the now-unused
`lockText` computed and the `Tooltip` imports. The badge and its `variant` map stay.

- [ ] **Step 4: Check the rules and the types**

Run: `pnpm typecheck && pnpm lint && pnpm scan && pnpm ui:test`
Expected: PASS, `0 violations`. If `scan` flags a visible string, it is one that should be a
message — fix it there, not in the exemptions.

- [ ] **Step 5: Commit**

```bash
git add ui/src/components/graph/ ui/src/screens/collection/CollectionRow.vue
git commit -m "feat(ui): a blocked badge opens the menu of what is in the way"
```

---

### Task 8: The two risks, checked on the real screen

Not a formality: the Plan's rows are draggable, and the badge lives inside a virtualized table.
Both were written into the spec as things to verify, not assume.

**Files:** whichever the checks turn out to need — expected candidates are
`ui/src/screens/plan/` (the drag handle's surface) and `ui/src/components/graph/WhyMenu.vue`.

- [ ] **Step 1: Run the app on fixtures**

Run: `pnpm ui:dev` and open the Unlock screen with `?fixture=active`.
Check: a blocked badge opens its menu; an entry with a page opens the wiki page in the active
tab; Ctrl+click opens it beside; a disabled entry does nothing. A `grafo parziale` badge opens a
menu too, mostly disabled.

- [ ] **Step 2: The Plan's drag**

Open the Plan with a queue that has rows (`?queue=` with rows, per the fixture flags in
`CLAUDE.md`).
Check: opening a badge's menu does **not** start a drag, and dragging a row still works when the
gesture starts on the badge's row. If the two fight, fix it by narrowing the drag handle's
surface — never by making the badge behave differently on this one screen. Write down what you
found either way: it goes in the report.

- [ ] **Step 3: The virtualized table**

On Unlock, open a menu, then scroll the table hard enough to recycle that row.
Check: the menu closes (or stays anchored to a row that still exists). A menu left floating over
a different row is a bug — fix it by closing on the row's unmount.

- [ ] **Step 4: The whole app, with the backend**

Run: `pnpm dev`.
Check: on a real profile the entries link — a blocked node's character opens that character's
wiki page. This is the first moment the dataset is real, so it is the first proof that `page` is
`Some` where it should be.

- [ ] **Step 5: Commit whatever the checks changed**

```bash
git add -A
git commit -m "fix(ui): <what the check found>"
```

If nothing needed fixing, skip the commit and record it in the report instead.

---

### Task 9: Hand on the contract, and close the sub-project

**Files:**
- Modify: `DESIGN-BRIEF.md` (§7.1)
- Modify: `docs/STATUS.md` (the 3.5d line and the session log)
- Modify: `docs/BACKLOG.md` (the entry this sub-project deliberately did not do)
- Create: `docs/superpowers/plans/2026-09-12-blocked-menu-report.md`

- [ ] **Step 1: The contract**

In `DESIGN-BRIEF.md` §7.1, add `page` to the requirement type and to the lock, with the rule in
one line: *`null` means the dataset has no page — the name shows and does not link; it never
means "no requirement".*

- [ ] **Step 2: The backlog entry for what was left out**

Add a new numbered entry (the next free number after B33) to `docs/BACKLOG.md`: *Unlock's
"sblocca" column and the Plan's queue rows link to their pages too* — same idea as 3.5d, a
different surface (`UnlockTarget` instead of `RequirementView`), needing the same `page` field on
`UnlockTarget`.

- [ ] **Step 3: The report**

Write `docs/superpowers/plans/2026-09-12-blocked-menu-report.md`: what landed, what Task 8 found
(especially the Plan's drag), and the one open question for the first look — whether a disabled
entry should say *why* it can't be followed, which this sub-project deliberately left as
silent greying.

- [ ] **Step 4: STATUS**

Tick 3.5d in `docs/STATUS.md` **only** once the work is committed, with the date and a pointer to
the report, and add the session-log paragraph in the style of the entries above it.

- [ ] **Step 5: The whole suite**

Run: `pnpm check`
Expected: green. Read the skip lines it prints — a suite that skipped the real-data tests has not
tested the real data.

- [ ] **Step 6: Commit**

```bash
git add DESIGN-BRIEF.md docs/
git commit -m "docs: 3.5d lands, a blocked name says where to read about it"
```

- [ ] **Step 7: Merge**

Follow `superpowers:finishing-a-development-branch`: merge `feature/blocked-menu` into `develop`
with a merge commit, run `pnpm check` on the merge result, and only then push.
