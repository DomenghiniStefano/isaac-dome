# Cycle 3.5b — search: Implementation Plan

> **For agentic workers:** executed inline with superpowers:executing-plans, on the owner's
> delegation ("prosegui coi lavori"). Steps use checkbox (`- [ ]`) syntax. Every commit is
> pushed as it lands (the owner's rule: the remote stays aligned). Branch
> `feature/screens-search`, cut from `develop`: `feature/screens-wiki-search` carried half A
> and is fully merged, so half B gets its own branch under the one-branch-per-sub-project
> rule.

**Goal:** one index over everything the app knows by name or by text — catalog names,
achievement conditions, wiki titles, the body of wiki sections — queried by a command, shown
in the `Ctrl+K` palette from any tab and, in full, on a Search screen. A result is a `Target`
plus why it matched, and opening it is the same action as following a wiki link.

**Architecture:** a pure `crates/ipc/src/search.rs`. The **wiki side** is built once from the
embedded dataset (a page's title, and each section flattened to one string) and kept in a
managed `SearchState`; the **catalog side** is read live at every query (names, and the
achievements' `text` and `unlock_condition`). The two are joined per `Target` into one
document, matched field by field — title, alias, condition, then each section — ranked by six
tiers and then by the profile, and cut to a limit. The frontend never re-ranks: the palette
turns each hit into the **destinations** it can open (Wiki, Unlock, Collezione, plus the
Schermate rows it makes itself), the Search screen shows the same rows uncapped, and both
navigate by tab location.

**Tech Stack:** Rust (`crates/ipc`, `crates/app`, `crates/design-export`), Vue 3.5,
TypeScript, Pinia, Vue Router, Tailwind v4, Reka UI, vue-i18n, `@tanstack/vue-virtual`,
Vitest.

**Spec:** `docs/superpowers/specs/2026-09-12-screens-wiki-search-design.md` (Decisions 5–10;
1–4 landed as 3.5a)

## Global Constraints

- Only resolved view-models cross the IPC: a hit is a wiki `Target` plus text; icons are `isaac://` links; no paths, no offsets.
- Degrade, never fail: no profile → every mark `unknown` and a diagnostic, never an `Err`; no game → wiki titles and text only, no Unlock or Collection rows; no dataset → catalog names only, `hasPage` false everywhere.
- IPC: `#[serde(rename_all = "camelCase")]`; enums with data tagged (`tag = "kind"`, `rename_all_fields = "camelCase"`); **an all-fieldless enum is a bare camelCase string** (`ProgressMark`, `SearchDiagnostic`) — this corrects the spec's Decision 5, which drew `SearchDiagnostic` as `{ kind: … }`: the repo has zero tagged unit enums, and a second convention for the same thing makes a TypeScript `switch` fall into no branch.
- Exhaustive matches, no `_ =>` on a closed enum; no `unwrap`/`panic` outside tests.
- Frontend: the five rules (no `<style>`, no hardcoded visual constants, no `invoke()` in components, no raw `<button>`/`<input>`, no string unions), `assertNever`, every visible string through `useMessages()`; page and item titles are data, in English.
- Fixtures only under `import.meta.env.DEV`; pack files through `import.meta.glob`.
- The frontend has no component test runner (no jsdom, no `@vue/test-utils`): what is worth checking lives in a pure module and is tested there; rendering is checked by eye.
- Checks judged by exit code; Cargo with `CARGO_BUILD_JOBS=4` on this machine.
- Commits: Conventional Commits, English, no `Co-Authored-By` or Claude reference, **pushed after each task**.

---

### Task 1: the wiki side of the index

**Files:**
- Create: `crates/ipc/src/search.rs`
- Modify: `crates/ipc/src/lib.rs`, `crates/ipc/src/wiki.rs` (`boss_target` becomes `pub(crate)`)
- Test: `crates/ipc/tests/search.rs` (new)

**Interfaces:**
- Produces: `SearchIndex::build(dataset: Result<&Dataset, &DatasetError>) -> SearchIndex`; `SearchIndex::len()`, `is_empty()`, `is_loaded()`, `title(&Target) -> Option<&str>`, `section_text(&Target, SectionKind) -> Option<&str>`.

- [ ] **Step 1: Write the failing test** — `crates/ipc/tests/search.rs`:

```rust
//! Search: one index over the wiki's text and the catalog's names. The tests state what a
//! page reads as, which field a query hits, and how the hits are ordered — never what the
//! current code happens to answer.

use ipc::{SearchIndex, Target};
use wiki::{
    Block, Dataset, DatasetError, Dlc, Entry, Infobox, Inline, ListItem, Section, SectionKind,
    Style,
};

fn text(s: &str) -> Inline {
    Inline::Text {
        text: s.into(),
        style: Style::Plain,
    }
}

fn entry_with(title: &str, infobox: Infobox, sections: Vec<Section>) -> Entry {
    Entry {
        title: title.into(),
        revid: 1,
        infobox,
        sections,
    }
}

/// The D6's Effects section as the dataset holds it: a heading, a paragraph with a `ref` and
/// an `edition` inline, a list with a nested block, and a one-row table. The flattening is
/// what a query is matched against, so the test states the string it becomes.
fn d6_effects() -> Section {
    Section {
        kind: SectionKind::Effects,
        blocks: vec![
            Block::Heading {
                level: 2,
                inline: vec![text("Effects")],
            },
            Block::Paragraph {
                inline: vec![
                    text("Rerolls the items in the"),
                    Inline::Ref {
                        target: Target::Room {
                            name: "Treasure Room".into(),
                        },
                        label: "Treasure Room".into(),
                    },
                    Inline::Edition {
                        only: vec![Dlc::Repentance],
                        inline: vec![text("only in Repentance")],
                    },
                ],
            },
            Block::List {
                ordered: false,
                items: vec![ListItem {
                    inline: vec![text("Damage up")],
                    children: vec![Block::Paragraph {
                        inline: vec![text("stacks")],
                    }],
                }],
            },
            Block::Table {
                header: vec![vec![text("Quality")]],
                rows: vec![vec![vec![text("4")]]],
            },
        ],
    }
}

fn dataset() -> Dataset {
    let mut ds = Dataset::empty_for_tests();
    ds.items
        .insert(105, entry_with("The D6", Infobox::Item, vec![d6_effects()]));
    ds.trinkets
        .insert(97, entry_with("Tonsil", Infobox::Trinket, vec![]));
    ds.bosses.insert(
        Dataset::boss_key(20, 0, 0),
        entry_with(
            "Monstro",
            Infobox::Boss {
                base_hp: None,
                environment: vec![],
                pool: vec![],
                unlocked_by: None,
            },
            vec![Section {
                kind: SectionKind::Behavior,
                blocks: vec![Block::Paragraph {
                    inline: vec![text("Jumps at the player and spits blood")],
                }],
            }],
        ),
    );
    ds
}

#[test]
fn a_section_reads_as_its_text_labels_and_cells_in_order() {
    let ds = dataset();
    let index = SearchIndex::build(Ok(&ds));
    assert!(index.is_loaded());
    assert_eq!(index.len(), 3);
    assert_eq!(index.title(&Target::Item { id: 105 }), Some("The D6"));
    // Headings, ref labels, nested list blocks and table cells are all text to search; an
    // `edition` inline is unwrapped, because the page shows its words.
    assert_eq!(
        index.section_text(&Target::Item { id: 105 }, SectionKind::Effects),
        Some(
            "Effects Rerolls the items in the Treasure Room only in Repentance Damage up stacks Quality 4"
        )
    );
    assert_eq!(
        index.section_text(&Target::Item { id: 105 }, SectionKind::Notes),
        None
    );
    assert_eq!(
        index.section_text(
            &Target::Entity {
                id: 20,
                variant: 0,
                subtype: 0
            },
            SectionKind::Behavior
        ),
        Some("Jumps at the player and spits blood")
    );
}

#[test]
fn a_dataset_that_did_not_load_is_an_empty_index_that_says_so() {
    let e = DatasetError::Malformed { reason: "x".into() };
    let index = SearchIndex::build(Err(&e));
    assert!(!index.is_loaded());
    assert!(index.is_empty());
    assert_eq!(index.title(&Target::Item { id: 105 }), None);
}
```

- [ ] **Step 2: Run the test to see it fail**

Run: `cargo test -p ipc --test search`
Expected: FAIL — `SearchIndex` doesn't exist.

- [ ] **Step 3: Write the index** — `crates/ipc/src/search.rs`:

```rust
//! Search: one index over everything the app knows by name or by text (spec 3.5, Decisions
//! 5 and 6). Pure, like the rest of `ipc`.
//!
//! Two sources. The **wiki side** is built once — the dataset never changes — and each page
//! becomes its title plus one flattened string per section. The **catalog side** is read at
//! every query: 2,000 names cost nothing, and "the game isn't installed" is never cached.

use std::collections::BTreeMap;

use serde::Serialize;
use wiki::{Block, Dataset, DatasetError, Entry, Inline, SectionKind, Target};

use crate::wiki::boss_target;

/// One page as the search reads it: the title, and the text of each section in the order the
/// page has them.
struct WikiDoc {
    title: String,
    sections: Vec<(SectionKind, String)>,
}

/// The wiki side of the index, built once from the embedded dataset.
pub struct SearchIndex {
    loaded: bool,
    pages: BTreeMap<Target, WikiDoc>,
}

impl SearchIndex {
    pub fn build(dataset: Result<&Dataset, &DatasetError>) -> SearchIndex {
        let Ok(ds) = dataset else {
            return SearchIndex {
                loaded: false,
                pages: BTreeMap::new(),
            };
        };
        let mut pages = BTreeMap::new();
        let mut add = |target: Target, entry: &Entry| {
            pages.insert(target, doc(entry));
        };
        for (id, e) in &ds.items {
            add(Target::Item { id: *id }, e);
        }
        for (id, e) in &ds.trinkets {
            add(Target::Trinket { id: *id }, e);
        }
        for (id, e) in &ds.achievements {
            add(Target::Achievement { id: *id }, e);
        }
        for (key, e) in &ds.bosses {
            if let Some(t) = boss_target(key) {
                add(t, e);
            }
        }
        for (n, e) in &ds.challenges {
            add(Target::Challenge { number: *n }, e);
        }
        for (id, e) in &ds.characters {
            add(Target::Character { id: *id }, e);
        }
        SearchIndex {
            loaded: true,
            pages,
        }
    }

    /// Whether the dataset loaded at all: `false` is the `noWiki` diagnostic.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn len(&self) -> usize {
        self.pages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }

    pub fn title(&self, target: &Target) -> Option<&str> {
        self.pages.get(target).map(|d| d.title.as_str())
    }

    /// The flattened text of one page's section: the measurable half of the index, so a test
    /// can state what a page reads as without going through a query.
    pub fn section_text(&self, target: &Target, kind: SectionKind) -> Option<&str> {
        self.pages
            .get(target)?
            .sections
            .iter()
            .find(|(k, _)| *k == kind)
            .map(|(_, text)| text.as_str())
    }
}

fn doc(entry: &Entry) -> WikiDoc {
    WikiDoc {
        title: entry.title.clone(),
        sections: entry
            .sections
            .iter()
            .map(|s| {
                let mut text = String::new();
                for block in &s.blocks {
                    flatten_block(block, &mut text);
                }
                (s.kind, text)
            })
            .collect(),
    }
}

fn push(out: &mut String, piece: &str) {
    let piece = piece.trim();
    if piece.is_empty() {
        return;
    }
    if !out.is_empty() {
        out.push(' ');
    }
    out.push_str(piece);
}

fn flatten_inline(inline: &[Inline], out: &mut String) {
    for i in inline {
        match i {
            Inline::Text { text, .. } => push(out, text),
            Inline::Ref { label, .. } | Inline::Concept { label, .. } => push(out, label),
            // The words are on the page: an edition inline is unwrapped, not skipped.
            Inline::Edition { inline, .. } => flatten_inline(inline, out),
        }
    }
}

fn flatten_block(block: &Block, out: &mut String) {
    match block {
        Block::Paragraph { inline } | Block::Heading { inline, .. } => flatten_inline(inline, out),
        Block::List { items, .. } => {
            for item in items {
                flatten_inline(&item.inline, out);
                for child in &item.children {
                    flatten_block(child, out);
                }
            }
        }
        Block::Table { header, rows } => {
            for cell in header {
                flatten_inline(cell, out);
            }
            for row in rows {
                for cell in row {
                    flatten_inline(cell, out);
                }
            }
        }
    }
}
```

In `crates/ipc/src/wiki.rs`, `fn boss_target` becomes `pub(crate) fn boss_target`. In
`crates/ipc/src/lib.rs`: `mod search;` and `pub use search::SearchIndex;`.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p ipc --test search`
Expected: PASS (2 tests).

- [ ] **Step 5: Commit and push**

```bash
git add crates/ipc/src/search.rs crates/ipc/src/lib.rs crates/ipc/src/wiki.rs crates/ipc/tests/search.rs
git commit -m "feat(ipc): the wiki side of the search index, one string per section"
git push
```

---

### Task 2: the query — words, fields, fragments

**Files:**
- Modify: `crates/ipc/src/search.rs` (private functions and their unit tests)

**Interfaces:**
- Produces (crate-private): `fold(&str) -> String`, `words(&str) -> Vec<String>`, `contains_all(&str, &[String]) -> bool`, `first_at(&str, &[String]) -> Option<(usize, usize)>` (byte offset, byte length), `fragment(&str, &[String]) -> Option<(String, String, String)>` — before, matched, after.

- [ ] **Step 1: Write the failing tests** — append to `crates/ipc/src/search.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folding_keeps_byte_offsets_aligned() {
        // ASCII fold only: a non-ASCII letter matches itself, and every byte keeps its
        // place, so an offset found in the folded string cuts the original correctly.
        let source = "Brimstone è Rosso";
        let folded = fold(source);
        assert_eq!(folded, "brimstone è rosso");
        assert_eq!(folded.len(), source.len());
    }

    #[test]
    fn every_word_must_be_in_the_same_field() {
        let w = words("  blood  BOMB ");
        assert_eq!(w, vec!["blood".to_string(), "bomb".to_string()]);
        assert!(contains_all("a bomb full of blood", &w));
        assert!(!contains_all("a bomb", &w));
        assert!(words("   ").is_empty());
        // An empty query matches nothing at all, rather than everything.
        assert!(!contains_all("anything", &words("")));
    }

    #[test]
    fn a_fragment_is_cut_around_the_earliest_word_on_character_boundaries() {
        let text = "The D6 rerolls the items in the Treasure Room, and Blood Bombs stay";
        let (before, matched, after) =
            fragment(text, &words("bombs treasure")).expect("the text holds both words");
        // The earliest of the two words names the place, whatever order they were typed in.
        assert_eq!(matched, "Treasure");
        assert!(text.starts_with(&before), "before: {before:?}");
        assert!(text.ends_with(&after), "after: {after:?}");
        assert!(before.chars().count() <= 60);
        assert!(after.chars().count() <= 90);
        assert_eq!(fragment(text, &words("nothing")), None);
    }

    #[test]
    fn a_fragment_never_splits_a_character() {
        // Eighty accented characters, two bytes each: a cut counted in bytes would land
        // inside one and panic.
        let text = format!("{} bomba", "è".repeat(80));
        let (before, matched, _) = fragment(&text, &words("bomba")).expect("matches");
        assert_eq!(matched, "bomba");
        assert!(before.chars().count() <= 60);
    }
}
```

- [ ] **Step 2: Run to see them fail**

Run: `cargo test -p ipc --lib search`
Expected: FAIL — `fold`, `words`, `contains_all`, `fragment` don't exist.

- [ ] **Step 3: Write the matcher** — in `crates/ipc/src/search.rs`:

```rust
/// The characters kept before and after a match in a section fragment: enough to read the
/// sentence, counted in **characters**, so a cut never lands inside one.
const BEFORE: usize = 60;
const AFTER: usize = 90;

/// ASCII fold. Every ASCII byte maps to one byte and everything else is left alone, so an
/// offset in the folded string is an offset in the original: the fragment cuts the page's
/// own text without a second search.
fn fold(s: &str) -> String {
    s.to_ascii_lowercase()
}

fn words(query: &str) -> Vec<String> {
    fold(query).split_whitespace().map(str::to_string).collect()
}

fn contains_all(folded: &str, words: &[String]) -> bool {
    !words.is_empty() && words.iter().all(|w| folded.contains(w.as_str()))
}

/// The earliest occurrence among the words: where the fragment is cut, and how long the
/// matched text is.
fn first_at(folded: &str, words: &[String]) -> Option<(usize, usize)> {
    words
        .iter()
        .filter_map(|w| folded.find(w.as_str()).map(|at| (at, w.len())))
        .min_by_key(|(at, _)| *at)
}

fn fragment(text: &str, words: &[String]) -> Option<(String, String, String)> {
    let folded = fold(text);
    if !contains_all(&folded, words) {
        return None;
    }
    let (at, len) = first_at(&folded, words)?;
    let head = text.get(..at)?;
    let start = head
        .char_indices()
        .rev()
        .nth(BEFORE - 1)
        .map_or(0, |(i, _)| i);
    let matched = text.get(at..at + len)?;
    let tail = text.get(at + len..)?;
    let end = tail.char_indices().nth(AFTER).map_or(tail.len(), |(i, _)| i);
    Some((
        head[start..].to_string(),
        matched.to_string(),
        tail[..end].to_string(),
    ))
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p ipc --lib search`
Expected: PASS (4 tests).

- [ ] **Step 5: Commit and push**

```bash
git add crates/ipc/src/search.rs
git commit -m "feat(ipc): the search query, its words and the fragment around a match"
git push
```

---

### Task 3: the catalog side, the join, and the marks

**Files:**
- Modify: `crates/ipc/src/search.rs`, `crates/ipc/src/lib.rs`, `crates/ipc/src/target_sprite.rs` (`entity_key` becomes `pub(crate)`)
- Test: `crates/ipc/tests/search.rs`

**Interfaces:**
- Consumes: `SearchIndex` (Task 1), the matcher (Task 2).
- Produces: `pub struct SaveFlags<'a> { pub achievements: Option<&'a [bool]>, pub items: Option<&'a [bool]> }`; `pub enum ProgressMark { Done, Pending, Unknown, None }` (bare camelCase string); `pub struct Doc { pub title: String, pub alias: Option<String>, pub condition: Option<String>, pub has_page: bool }`; `documents_for_tests(&SearchIndex, Option<&Catalog>) -> BTreeMap<Target, Doc>`; `progress_for_tests(&Target, Option<SaveFlags>) -> ProgressMark`.

- [ ] **Step 1: Write the failing test** — append to `crates/ipc/tests/search.rs`:

```rust
use catalog::Catalog;
use ipc::{documents_for_tests, progress_for_tests, ProgressMark, SaveFlags};

const ITEMS: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"105\" gfx=\"d6.png\" name=\"The D6\" achievement=\"1\" /><trinket id=\"97\" gfx=\"t.png\" name=\"Tonsil\" /></items>";
const ACH: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\"><!-- Defeat Mom's Heart 10 times --><achievement id=\"1\" text=\"You unlocked The D6\" gfx=\"1.png\" /></achievements>";
const BOSSES: &[u8] = b"<bossportraits gfxroot=\"gfx/ui/boss/\"><boss id=\"1\" name=\"Monstro\" portrait=\"Portrait_20.0_Monstro.png\" /></bossportraits>";

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS.to_vec()),
        "achievements.xml" => Some(ACH.to_vec()),
        "bossportraits.xml" => Some(BOSSES.to_vec()),
        _ => None,
    })
}

#[test]
fn a_target_both_sides_know_is_one_document_with_the_catalog_name_as_its_title() {
    let ds = dataset();
    let index = SearchIndex::build(Ok(&ds));
    let docs = documents_for_tests(&index, Some(&catalog()));
    let d6 = docs
        .get(&Target::Item { id: 105 })
        .expect("the item is on both sides");
    assert_eq!(d6.title, "The D6");
    // The wiki title equals the catalog name here, so there is no second name to carry.
    assert_eq!(d6.alias, None);
    assert!(d6.has_page);
    // The achievement is in the catalog only: still a document, with no page to open.
    let a = docs
        .get(&Target::Achievement { id: 1 })
        .expect("the achievement is in the catalog");
    assert_eq!(a.title, "You unlocked The D6");
    assert_eq!(a.condition.as_deref(), Some("Defeat Mom's Heart 10 times"));
    assert!(!a.has_page);
    // The boss's entity key comes from the portrait's file name, the way the icon does.
    assert!(docs.contains_key(&Target::Entity {
        id: 20,
        variant: 0,
        subtype: 0
    }));
}

#[test]
fn without_a_catalog_the_documents_are_the_wiki_pages_alone() {
    let ds = dataset();
    let index = SearchIndex::build(Ok(&ds));
    let docs = documents_for_tests(&index, None);
    assert_eq!(docs.len(), index.len());
    assert_eq!(
        docs.get(&Target::Trinket { id: 97 }).map(|d| d.title.as_str()),
        Some("Tonsil")
    );
}

#[test]
fn a_mark_is_read_from_the_section_that_holds_it() {
    // Achievement 1 done, achievement 2 not; no collectible slot is set.
    let done = [false, true, false];
    let owned = [false; 106];
    let flags = SaveFlags {
        achievements: Some(&done),
        items: Some(&owned),
    };
    assert_eq!(
        progress_for_tests(&Target::Achievement { id: 1 }, Some(flags)),
        ProgressMark::Done
    );
    assert_eq!(
        progress_for_tests(&Target::Achievement { id: 2 }, Some(flags)),
        ProgressMark::Pending
    );
    assert_eq!(
        progress_for_tests(&Target::Item { id: 105 }, Some(flags)),
        ProgressMark::Pending
    );
    // A trinket has no slot in section 4, and a boss none anywhere: no mark, not "unknown".
    assert_eq!(
        progress_for_tests(&Target::Trinket { id: 97 }, Some(flags)),
        ProgressMark::None
    );
    // No profile, and a profile whose section didn't read, are both "unknown".
    assert_eq!(
        progress_for_tests(&Target::Achievement { id: 1 }, None),
        ProgressMark::Unknown
    );
    let unread = SaveFlags {
        achievements: None,
        items: None,
    };
    assert_eq!(
        progress_for_tests(&Target::Item { id: 105 }, Some(unread)),
        ProgressMark::Unknown
    );
}
```

- [ ] **Step 2: Run to see it fail**

Run: `cargo test -p ipc --test search`
Expected: FAIL — `documents_for_tests`, `progress_for_tests`, `SaveFlags`, `ProgressMark` don't exist.

- [ ] **Step 3: Write the join** — in `crates/ipc/src/search.rs`:

```rust
use catalog::{Catalog, ItemKind, Language};

use crate::target_sprite::entity_key;

/// The two flag sections search reads. `None` for a section is "it didn't read", which is
/// not "nothing is done": the mark says `unknown` and a diagnostic says which section.
#[derive(Debug, Clone, Copy)]
pub struct SaveFlags<'a> {
    pub achievements: Option<&'a [bool]>,
    pub items: Option<&'a [bool]>,
}

/// Where a target stands in the profile. Fieldless: a bare string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProgressMark {
    Done,
    Pending,
    Unknown,
    None,
}

/// One searchable thing, with whatever the two sides know about it.
#[derive(Debug, Clone)]
pub struct Doc {
    pub title: String,
    /// The other name, when the two sides disagree: the wiki's title beside the game's.
    pub alias: Option<String>,
    /// An achievement's own wording of what to do.
    pub condition: Option<String>,
    pub has_page: bool,
}

/// Every document, keyed by target so the order is the kind's and then the id's, and a
/// target the two sides share is one row.
fn documents(index: &SearchIndex, catalog: Option<&Catalog>) -> BTreeMap<Target, Doc> {
    let mut docs: BTreeMap<Target, Doc> = index
        .pages
        .iter()
        .map(|(target, page)| {
            (
                target.clone(),
                Doc {
                    title: page.title.clone(),
                    alias: None,
                    condition: None,
                    has_page: true,
                },
            )
        })
        .collect();
    let Some(c) = catalog else {
        return docs;
    };
    let en = Language::English;
    let mut join = |target: Target, title: String, condition: Option<String>| {
        match docs.get_mut(&target) {
            // The catalog's name wins: it is what the game itself calls the thing. The wiki
            // title stays as an alias when it says something else.
            Some(doc) => {
                if doc.title != title {
                    doc.alias = Some(std::mem::replace(&mut doc.title, title));
                }
                doc.condition = condition;
            }
            None => {
                docs.insert(
                    target,
                    Doc {
                        title,
                        alias: None,
                        condition,
                        has_page: false,
                    },
                );
            }
        }
    };
    for i in c.items() {
        let target = match i.kind {
            ItemKind::Trinket => Target::Trinket { id: i.id.0 },
            ItemKind::Passive | ItemKind::Active | ItemKind::Familiar => {
                Target::Item { id: i.id.0 }
            }
        };
        join(target, c.text(&i.name, en).to_string(), None);
    }
    for p in c.characters() {
        join(
            Target::Character { id: p.id.0 },
            c.text(&p.name, en).to_string(),
            None,
        );
    }
    for b in c.bosses() {
        // The portrait's file name carries the entity key, exactly as `target_sprite` reads
        // it; a portrait that doesn't declare one names no target and is left out.
        if let Some((id, variant)) = entity_key(&b.portrait.path) {
            join(
                Target::Entity {
                    id,
                    variant,
                    subtype: 0,
                },
                b.name.clone(),
                None,
            );
        }
    }
    for ch in c.challenges() {
        join(Target::Challenge { number: ch.id.0 }, ch.name.clone(), None);
    }
    for a in c.achievements() {
        join(
            Target::Achievement { id: a.id.0 },
            a.text.clone(),
            a.unlock_condition.clone(),
        );
    }
    docs
}

/// Where the target stands in the profile: section 1 for an achievement, section 4 for a
/// collectible, nothing for anything else. A slot past a section's end reads as not done —
/// the save has no record of it — while a section that didn't read is `Unknown`.
fn progress(target: &Target, flags: Option<SaveFlags<'_>>) -> ProgressMark {
    let mark = |slots: Option<&[bool]>, id: u32| match slots {
        None => ProgressMark::Unknown,
        Some(f) => {
            if f.get(id as usize).copied().unwrap_or(false) {
                ProgressMark::Done
            } else {
                ProgressMark::Pending
            }
        }
    };
    match target {
        Target::Achievement { id } => match flags {
            None => ProgressMark::Unknown,
            Some(f) => mark(f.achievements, *id),
        },
        Target::Item { id } => match flags {
            None => ProgressMark::Unknown,
            Some(f) => mark(f.items, *id),
        },
        // A trinket has no slot in section 4, and a boss, a character or a challenge has no
        // slot to read at all: no mark is not an unknown one.
        Target::Trinket { .. }
        | Target::Character { .. }
        | Target::Challenge { .. }
        | Target::Entity { .. }
        | Target::Transformation { .. }
        | Target::Stage { .. }
        | Target::Room { .. }
        | Target::Pickup { .. } => ProgressMark::None,
    }
}

/// The two halves above, reachable from the integration test: they are what a query is built
/// on, and a test that could only see the ranked answer would say nothing about them.
#[doc(hidden)]
pub fn documents_for_tests(index: &SearchIndex, catalog: Option<&Catalog>) -> BTreeMap<Target, Doc> {
    documents(index, catalog)
}

#[doc(hidden)]
pub fn progress_for_tests(target: &Target, flags: Option<SaveFlags<'_>>) -> ProgressMark {
    progress(target, flags)
}
```

`crates/ipc/src/target_sprite.rs`: `fn entity_key` becomes `pub(crate) fn entity_key`.
`crates/ipc/src/lib.rs` exports `Doc`, `ProgressMark`, `SaveFlags`, `documents_for_tests`,
`progress_for_tests` beside `SearchIndex`.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p ipc --test search`
Expected: PASS (5 tests).

- [ ] **Step 5: Commit and push**

```bash
git add crates/ipc/src/search.rs crates/ipc/src/lib.rs crates/ipc/src/target_sprite.rs crates/ipc/tests/search.rs
git commit -m "feat(ipc): the catalog side of the search index, joined per target"
git push
```

---

### Task 4: ranking, the limit, the diagnostics, and the public `search()`

**Files:**
- Modify: `crates/ipc/src/search.rs`, `crates/ipc/src/lib.rs`
- Test: `crates/ipc/tests/search.rs`

**Interfaces:**
- Consumes: `documents`, `progress`, the matcher.
- Produces: `pub fn search(index: &SearchIndex, catalog: Option<&Catalog>, flags: Option<SaveFlags<'_>>, query: &str, limit: usize, icon: impl FnMut(&IconRef) -> Option<String>) -> SearchView`; `SearchView { query, hits, total, diagnostics }`; `SearchHit { target, title, icon_url, has_page, match, progress }`; `SearchMatch::{Title, Condition { text }, Section { section, before, matched, after }}`; `SearchDiagnostic::{NoProfile, NoCatalog, NoWiki, NoAchievementSection, NoCollectionSection}` (bare string).

- [ ] **Step 1: Write the failing test** — append to `crates/ipc/tests/search.rs`:

```rust
use ipc::{search, IconRef, SearchDiagnostic, SearchMatch, SearchView};
use serde_json::{json, to_value};

fn link(r: &IconRef) -> Option<String> {
    Some(format!("isaac://{}", r.to_path()))
}

fn view(query: &str, limit: usize, with_catalog: bool, flags: Option<SaveFlags<'_>>) -> SearchView {
    let ds = dataset();
    let index = SearchIndex::build(Ok(&ds));
    let c = catalog();
    search(
        &index,
        with_catalog.then_some(&c),
        flags,
        query,
        limit,
        link,
    )
}

#[test]
fn the_shapes_are_pinned() {
    assert_eq!(to_value(SearchMatch::Title).unwrap(), json!({"kind":"title"}));
    assert_eq!(
        to_value(SearchMatch::Condition {
            text: "Defeat Mom".into()
        })
        .unwrap(),
        json!({"kind":"condition","text":"Defeat Mom"})
    );
    assert_eq!(
        to_value(SearchMatch::Section {
            section: SectionKind::Effects,
            before: "a ".into(),
            matched: "b".into(),
            after: " c".into()
        })
        .unwrap(),
        json!({"kind":"section","section":"effects","before":"a ","matched":"b","after":" c"})
    );
    // Fieldless enums are bare strings, not tagged objects: the repo has no other kind.
    assert_eq!(to_value(SearchDiagnostic::NoProfile).unwrap(), json!("noProfile"));
    assert_eq!(to_value(ProgressMark::Pending).unwrap(), json!("pending"));

    let v = to_value(view("d6", 10, true, None)).unwrap();
    assert_eq!(v["query"], json!("d6"));
    assert_eq!(v["hits"][0]["target"], json!({"kind":"item","id":105}));
    assert_eq!(v["hits"][0]["title"], json!("The D6"));
    assert_eq!(v["hits"][0]["hasPage"], json!(true));
    assert_eq!(v["hits"][0]["iconUrl"], json!("isaac://page/item/105"));
    assert_eq!(v["hits"][0]["match"], json!({"kind":"title"}));
    assert_eq!(v["hits"][0]["progress"], json!("unknown"));
}

#[test]
fn one_hit_per_target_named_by_the_first_field_that_matches() {
    // "d6" is in the item's title and in the achievement's text: two targets, two hits, and
    // neither is listed twice for matching in a section as well.
    let v = view("d6", 10, true, None);
    assert_eq!(v.hits.len(), 2);
    assert_eq!(v.total, 2);
    // The achievement matches by title too ("You unlocked The D6"), never by its condition.
    let achievement = v
        .hits
        .iter()
        .find(|h| h.target == Target::Achievement { id: 1 })
        .expect("the achievement is a hit");
    assert_eq!(achievement.matched, SearchMatch::Title);

    // A word only the condition has names the condition; one only a section has names the
    // section, with the page's own text around it.
    let v = view("heart", 10, true, None);
    assert!(matches!(
        v.hits.first().map(|h| &h.matched),
        Some(SearchMatch::Condition { .. })
    ));
    let v = view("spits", 10, true, None);
    let SearchMatch::Section {
        section, matched, ..
    } = v.hits.first().map(|h| h.matched.clone()).expect("a hit")
    else {
        panic!("a word only a section has must name that section")
    };
    assert_eq!(section, SectionKind::Behavior);
    assert_eq!(matched, "spits");
}

#[test]
fn every_word_of_the_query_must_be_in_one_field() {
    // "monstro" is a title and "spits" is in its Behavior section: no field holds both, so
    // the page is not a hit.
    assert_eq!(view("monstro spits", 10, true, None).hits.len(), 0);
    assert_eq!(view("jumps blood", 10, true, None).hits.len(), 1);
}

#[test]
fn the_six_tiers_order_the_answer() {
    // Four titles that all contain "the": the whole title, its start, the start of a word in
    // it, and — "Mother" — the query buried inside a word, which is the weakest of the four.
    let mut ds = Dataset::empty_for_tests();
    for (id, title) in [(1, "Mother"), (2, "The"), (3, "The Bible"), (4, "Of the")] {
        ds.items.insert(id, entry_with(title, Infobox::Item, vec![]));
    }
    let index = SearchIndex::build(Ok(&ds));
    let v = search(&index, None, None, "the", 10, link);
    let titles: Vec<&str> = v.hits.iter().map(|h| h.title.as_str()).collect();
    assert_eq!(titles, vec!["The", "The Bible", "Of the", "Mother"]);
}

#[test]
fn not_done_comes_before_done_inside_a_tier() {
    let mut ds = Dataset::empty_for_tests();
    ds.items.insert(1, entry_with("Bomb One", Infobox::Item, vec![]));
    ds.items.insert(2, entry_with("Bomb Two", Infobox::Item, vec![]));
    let index = SearchIndex::build(Ok(&ds));
    // Item 1 is in the collection, item 2 isn't: the one still to find is listed first.
    let owned = [false, true, false];
    let flags = SaveFlags {
        achievements: Some(&[]),
        items: Some(&owned),
    };
    let v = search(&index, None, Some(flags), "bomb", 10, link);
    let titles: Vec<&str> = v.hits.iter().map(|h| h.title.as_str()).collect();
    assert_eq!(titles, vec!["Bomb Two", "Bomb One"]);
}

#[test]
fn the_limit_cuts_the_hits_and_total_says_how_many_there_were() {
    let mut ds = Dataset::empty_for_tests();
    for id in 1..=5 {
        ds.items
            .insert(id, entry_with(&format!("Bomb {id}"), Infobox::Item, vec![]));
    }
    let index = SearchIndex::build(Ok(&ds));
    let v = search(&index, None, None, "bomb", 2, link);
    assert_eq!(v.hits.len(), 2);
    assert_eq!(v.total, 5);
}

#[test]
fn the_five_diagnostics_say_what_is_missing() {
    // No profile: one diagnostic, not one per section.
    let v = view("d6", 10, true, None);
    assert_eq!(v.diagnostics, vec![SearchDiagnostic::NoProfile]);
    // A profile whose two sections didn't read says so, once each.
    let unread = SaveFlags {
        achievements: None,
        items: None,
    };
    let v = view("d6", 10, true, Some(unread));
    assert_eq!(
        v.diagnostics,
        vec![
            SearchDiagnostic::NoAchievementSection,
            SearchDiagnostic::NoCollectionSection
        ]
    );
    // No game: wiki titles only, no icons, and no condition to match.
    let v = view("d6", 10, false, None);
    assert_eq!(
        v.diagnostics,
        vec![SearchDiagnostic::NoCatalog, SearchDiagnostic::NoProfile]
    );
    assert!(v.hits.iter().all(|h| h.icon_url.is_none()));
    // No dataset: catalog names only, and nothing has a page.
    let e = DatasetError::Malformed { reason: "x".into() };
    let index = SearchIndex::build(Err(&e));
    let c = catalog();
    let v = search(&index, Some(&c), None, "d6", 10, link);
    assert!(v.diagnostics.contains(&SearchDiagnostic::NoWiki));
    assert!(v.hits.iter().all(|h| !h.has_page));
    assert!(!v.hits.is_empty(), "the catalog still answers by name");
}

#[test]
fn an_empty_query_answers_nothing_and_says_nothing() {
    let v = view("   ", 10, true, None);
    assert!(v.hits.is_empty());
    assert_eq!(v.total, 0);
    assert!(v.diagnostics.is_empty());
}
```

- [ ] **Step 2: Run to see it fail**

Run: `cargo test -p ipc --test search`
Expected: FAIL — `search`, `SearchView`, `SearchMatch`, `SearchDiagnostic` don't exist.

- [ ] **Step 3: Write the ranking and the view** — in `crates/ipc/src/search.rs`:

```rust
use crate::icon::IconRef;
use crate::target_sprite::{target_sprite, TargetSprite};

/// A ranked answer. `total` is how many documents matched before the limit: the screen says
/// "300 of N" from it, and the palette's last row counts with it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchView {
    pub query: String,
    pub hits: Vec<SearchHit>,
    pub total: u32,
    pub diagnostics: Vec<SearchDiagnostic>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub target: Target,
    pub title: String,
    pub icon_url: Option<String>,
    /// The dataset has this page: a Wiki destination exists for the hit.
    pub has_page: bool,
    #[serde(rename = "match")]
    pub matched: SearchMatch,
    pub progress: ProgressMark,
}

/// Why the hit matched, in the words the row shows. Tagged: two of the three carry data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SearchMatch {
    Title,
    /// The achievement's own wording of what to do.
    Condition { text: String },
    Section {
        section: SectionKind,
        before: String,
        matched: String,
        after: String,
    },
}

/// What the answer couldn't take into account. Fieldless: a bare string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SearchDiagnostic {
    /// No profile chosen: every mark is unknown. Expected before a save is picked, not a
    /// failure.
    NoProfile,
    NoCatalog,
    NoWiki,
    NoAchievementSection,
    NoCollectionSection,
}

/// Tier 0 to 3, on a name: equal, prefix, a word's prefix, or merely containing the words.
fn title_tier(folded_title: &str, folded_query: &str) -> u8 {
    if folded_title == folded_query {
        0
    } else if folded_title.starts_with(folded_query) {
        1
    } else if folded_title
        .split_whitespace()
        .any(|w| w.starts_with(folded_query))
    {
        2
    } else {
        3
    }
}

/// Not done before done: that is what makes the profile part of the ranking (B5).
fn progress_rank(p: ProgressMark) -> u8 {
    match p {
        ProgressMark::Pending => 0,
        ProgressMark::Unknown => 1,
        ProgressMark::None => 2,
        ProgressMark::Done => 3,
    }
}

/// The first field of this document that holds every word, and the tier it earns. Fields in
/// order: title, alias, condition, then each section — so one target is one hit, named by
/// the strongest reason it matched.
fn best_field(
    index: &SearchIndex,
    target: &Target,
    doc: &Doc,
    words: &[String],
    folded_query: &str,
) -> Option<(u8, SearchMatch)> {
    let title = fold(&doc.title);
    if contains_all(&title, words) {
        return Some((title_tier(&title, folded_query), SearchMatch::Title));
    }
    if let Some(alias) = &doc.alias {
        let alias = fold(alias);
        if contains_all(&alias, words) {
            return Some((title_tier(&alias, folded_query), SearchMatch::Title));
        }
    }
    if let Some(condition) = &doc.condition {
        if contains_all(&fold(condition), words) {
            return Some((
                4,
                SearchMatch::Condition {
                    text: condition.clone(),
                },
            ));
        }
    }
    for (kind, text) in index.pages.get(target).map(|p| &p.sections)? {
        if let Some((before, matched, after)) = fragment(text, words) {
            return Some((
                5,
                SearchMatch::Section {
                    section: *kind,
                    before,
                    matched,
                    after,
                },
            ));
        }
    }
    None
}

/// The ranked answer to one query. Pure: the caller supplies the index, the catalog, the
/// profile's two sections and the icon link.
pub fn search(
    index: &SearchIndex,
    catalog: Option<&Catalog>,
    flags: Option<SaveFlags<'_>>,
    query: &str,
    limit: usize,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> SearchView {
    let words = words(query);
    if words.is_empty() {
        return SearchView {
            query: query.to_string(),
            hits: Vec::new(),
            total: 0,
            diagnostics: Vec::new(),
        };
    }
    let folded_query = fold(query.trim());
    let docs = documents(index, catalog);
    let mut ranked: Vec<(u8, u8, String, Target, Doc, SearchMatch, ProgressMark)> = docs
        .into_iter()
        .filter_map(|(target, doc)| {
            let (tier, matched) = best_field(index, &target, &doc, &words, &folded_query)?;
            let progress = progress(&target, flags);
            Some((
                tier,
                progress_rank(progress),
                fold(&doc.title),
                target,
                doc,
                matched,
                progress,
            ))
        })
        .collect();
    // Total and then the name, and last the target itself: the order is total, so a test can
    // pin it and two runs can never disagree.
    ranked.sort_by(|a, b| {
        (a.0, a.1, &a.2, &a.3)
            .partial_cmp(&(b.0, b.1, &b.2, &b.3))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let total = ranked.len() as u32;
    ranked.truncate(limit);
    let hits = ranked
        .into_iter()
        .map(|(_, _, _, target, doc, matched, progress)| SearchHit {
            icon_url: catalog.and_then(|c| match target_sprite(c, &target) {
                TargetSprite::Found(_) => icon(&IconRef::Page {
                    target: target.clone(),
                }),
                TargetSprite::NoArt | TargetSprite::Unknown => None,
            }),
            has_page: doc.has_page,
            title: doc.title,
            target,
            matched,
            progress,
        })
        .collect();
    SearchView {
        query: query.to_string(),
        hits,
        total,
        diagnostics: diagnostics(index, catalog, flags),
    }
}

fn diagnostics(
    index: &SearchIndex,
    catalog: Option<&Catalog>,
    flags: Option<SaveFlags<'_>>,
) -> Vec<SearchDiagnostic> {
    let mut out = Vec::new();
    if catalog.is_none() {
        out.push(SearchDiagnostic::NoCatalog);
    }
    if !index.is_loaded() {
        out.push(SearchDiagnostic::NoWiki);
    }
    match flags {
        // One word, not two: without a profile there is no section to miss.
        None => out.push(SearchDiagnostic::NoProfile),
        Some(f) => {
            if f.achievements.is_none() {
                out.push(SearchDiagnostic::NoAchievementSection);
            }
            if f.items.is_none() {
                out.push(SearchDiagnostic::NoCollectionSection);
            }
        }
    }
    out
}
```

`sort_by` on tuples of `(u8, u8, &String, &Target)`: `Target` derives `Ord`, so
`Ord::cmp` is available — use `a_key.cmp(&b_key)` rather than `partial_cmp` if clippy asks.
`crates/ipc/src/lib.rs` exports `search`, `SearchDiagnostic`, `SearchHit`, `SearchMatch`,
`SearchView`.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p ipc --test search` then `cargo clippy -p ipc --all-targets -- -D warnings`
Expected: PASS (13 tests), no warnings.

- [ ] **Step 5: Commit and push**

```bash
git add crates/ipc/src/search.rs crates/ipc/src/lib.rs crates/ipc/tests/search.rs
git commit -m "feat(ipc): the search view, six tiers and the profile in the ranking"
git push
```

---

### Task 5: measured on the real dataset

**Files:**
- Create: `crates/ipc/tests/search_real.rs`
- Modify: `crates/ipc/tests/unlock_size.rs`

**Interfaces:**
- Consumes: `SearchIndex::build`, `search`, `wiki::Dataset::embedded()`, `test_support`.

- [ ] **Step 1: Write the failing test** — `crates/ipc/tests/search_real.rs`:

```rust
//! Search on the real dataset. B5 asks for the scan to be **measured, not assumed**: the
//! timings are printed, never pinned — a number that depends on the machine would fail on
//! someone else's. What is pinned is the answer: the best-known query names its own page.

use std::time::Instant;

use catalog::Catalog;
use ipc::{search, IconRef, SearchIndex, Target};
use unpack::ResourceSet;
use wiki::Dataset;

fn link(r: &IconRef) -> Option<String> {
    Some(format!("isaac://{}", r.to_path()))
}

#[test]
fn brimstone_finds_its_own_page_first() {
    let Ok(ds) = Dataset::embedded() else {
        test_support::skip("the embedded dataset didn't load");
        return;
    };
    let built = Instant::now();
    let index = SearchIndex::build(Ok(ds));
    eprintln!(
        "search: index of {} pages built in {} ms",
        index.len(),
        built.elapsed().as_millis()
    );
    // The catalog is a bonus here: without the game the titles still answer.
    let catalog = test_support::packed_dir().map(|p| {
        let rs = ResourceSet::open(&p);
        Catalog::build(|f| rs.read(f))
    });
    for query in ["brimstone", "the lost", "mom's heart"] {
        let at = Instant::now();
        let view = search(&index, catalog.as_ref(), None, query, 300, link);
        eprintln!(
            "search: {query:?} → {} hits of {} in {} ms",
            view.hits.len(),
            view.total,
            at.elapsed().as_millis()
        );
    }
    let view = search(&index, catalog.as_ref(), None, "brimstone", 30, link);
    assert_eq!(
        view.hits.first().map(|h| h.target.clone()),
        Some(Target::Item { id: 118 }),
        "the item's own page must rank first for its exact name"
    );
}
```

Append to `crates/ipc/tests/unlock_size.rs`:

```rust
/// The wiki index is answered once per window and carries every page's title: it has to stay
/// a payload, not a download. Titles only — the icons are links — so 256 KB is generous and
/// a regression here means something started travelling that shouldn't.
const INDEX_CEILING: usize = 256_000;

#[test]
fn the_wiki_index_stays_a_payload() {
    let Ok(ds) = wiki::Dataset::embedded() else {
        test_support::skip("the embedded dataset didn't load");
        return;
    };
    let index = ipc::wiki_index(Ok(ds), None, None, |_| None);
    let json = serde_json::to_string(&index).expect("serializes");
    assert!(
        json.len() < INDEX_CEILING,
        "wiki index is {} bytes, over the {INDEX_CEILING} ceiling",
        json.len()
    );
}
```

- [ ] **Step 2: Run to see it fail (or skip loudly)**

Run: `cargo test -p ipc --test search_real -- --nocapture`
Expected: FAIL — the file doesn't compile until `search_real.rs` exists; once it does, either
PASS with the timings on stderr, or a `skip:` line naming what was missing.

- [ ] **Step 3: Adjust nothing but the test**

There is no implementation step here: if `brimstone` doesn't rank first, that is a **bug in
the ranking** (Task 4), not an expectation to relax. Fix `title_tier` or `best_field` and say
so in the report.

- [ ] **Step 4: Run both**

Run: `cargo test -p ipc -- --nocapture`
Expected: PASS, with `search:` lines and any `skip:` lines visible.

- [ ] **Step 5: Commit and push**

```bash
git add crates/ipc/tests/search_real.rs crates/ipc/tests/unlock_size.rs
git commit -m "test(ipc): search on the embedded dataset, timed and pinned on brimstone"
git push
```

---

### Task 6: the command, its state, and the design payload

**Files:**
- Modify: `crates/app/src/lib.rs`, `crates/design-export/src/payload.rs`

**Interfaces:**
- Consumes: `ipc::search`, `ipc::SearchIndex`, `ipc::SaveFlags`, `active_save`, `CatalogState`, `ResourcesState`.
- Produces: the Tauri command `search(query: String, limit: usize) -> Result<ipc::SearchView, IpcError>`; `contracts/payload/search.json`.

- [ ] **Step 1: Write the wiring** — in `crates/app/src/lib.rs`:

```rust
/// The wiki side of the search index, built once: the dataset is compiled into the binary
/// and never changes, so the flattening is paid for on the first query and never again. The
/// catalog side is *not* cached here — it is read per query, like everywhere else, because
/// "the game isn't installed" is never a cached answer.
#[derive(Default)]
struct SearchState(OnceLock<ipc::SearchIndex>);

impl SearchState {
    fn get(&self) -> &ipc::SearchIndex {
        self.0
            .get_or_init(|| ipc::SearchIndex::build(wiki::Dataset::embedded()))
    }
}

/// One query over the wiki's text and the catalog's names. No profile is a **diagnostic**,
/// not an error: search answers before a save is chosen, and says the marks are unknown.
#[tauri::command]
fn search(
    app: AppHandle,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    index: tauri::State<'_, SearchState>,
    query: String,
    limit: usize,
) -> Result<ipc::SearchView, IpcError> {
    // Game not installed is expected: the answer goes out with wiki titles alone.
    let catalog = resources.get().and_then(|rs| state.get_or_build(rs));
    let sections = active_save(&app)
        .ok()
        .map(|(_, s)| (s.flags(Kind::Achievements), s.flags(Kind::Items)));
    let flags = sections.as_ref().map(|(a, i)| ipc::SaveFlags {
        achievements: a.as_deref(),
        items: i.as_deref(),
    });
    Ok(ipc::search(
        index.get(),
        catalog,
        flags,
        &query,
        limit,
        icon_url,
    ))
}
```

Register it: `.manage(SearchState::default())` beside the other managed states, and `search`
in the `tauri::generate_handler![…]` list.

In `crates/design-export/src/payload.rs`, after the `wiki_index.json` block:

```rust
    // Three queries as the app answers them, so a design can be drawn on real hits: a name,
    // a condition, and a word that only a section has.
    let search_index = ipc::SearchIndex::build(wiki::Dataset::embedded());
    let searches: Vec<ipc::SearchView> = ["brimstone", "mom's heart", "rerolls"]
        .iter()
        .map(|q| {
            ipc::search(
                &search_index,
                Some(c),
                None,
                q,
                60,
                canonical,
            )
        })
        .collect();
    record(
        "contracts/payload/search.json",
        write_json(root, "contracts/payload/search.json", &searches)?,
    );
```

- [ ] **Step 2: Build and check the command list**

Run: `cargo build -p app` then `cargo clippy --all-targets -- -D warnings`
Expected: both succeed; `search` appears in the handler list.

- [ ] **Step 3: Verify the whole suite still passes**

Run: `cargo test --workspace -- --nocapture`
Expected: PASS, no new skips.

- [ ] **Step 4: Commit and push**

```bash
git add crates/app/src/lib.rs crates/design-export/src/payload.rs
git commit -m "feat(app): the search command, its index built once per window"
git push
```

---

### Task 7: the wire — types, wrapper, fixture

**Files:**
- Modify: `ui/src/lib/ipc/types.ts`, `ui/src/lib/constants/commands.ts`, `ui/src/lib/ipc/fixtures/index.ts`
- Create: `ui/src/lib/ipc/search.ts`, `ui/src/lib/ipc/fixtures/search.ts`
- Test: `ui/src/lib/ipc/fixtures/search.test.ts`

**Interfaces:**
- Consumes: the Rust shapes of Task 4.
- Produces: `ProgressMark`, `SearchMatch`, `SearchDiagnostic`, `SearchHit`, `SearchView` in `types.ts`; `search(query: string, limit: number): Promise<SearchView>` and `SearchLimit = { Palette: 40, Screen: 300 }` in `lib/ipc/search.ts`; `rankFixture(docs: FixtureDoc[], query: string, limit: number): SearchView` and `searchAnswer(query, limit, opts)` in the fixture.

- [ ] **Step 1: Write the failing test** — `ui/src/lib/ipc/fixtures/search.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { rankFixture } from './search'
import type { FixtureDoc } from './search'

const doc = (
  id: number,
  title: string,
  extra: Partial<FixtureDoc> = {},
): FixtureDoc => ({
  target: { kind: 'item', id },
  title,
  condition: null,
  sections: [],
  hasPage: true,
  iconUrl: null,
  ...extra,
})

describe('the fixture search', () => {
  it('orders by the same six tiers the backend uses', () => {
    const docs = [
      doc(1, 'Contains the word'),
      doc(2, 'The'),
      doc(3, 'The Bible'),
      doc(4, 'Of the'),
    ]
    const view = rankFixture(docs, 'the', 10)
    expect(view.hits.map((h) => h.title)).toEqual([
      'The',
      'The Bible',
      'Of the',
      'Contains the word',
    ])
    expect(view.total).toBe(4)
  })

  it('names the first field that holds every word, once per target', () => {
    const docs = [
      doc(1, 'Brimstone', {
        sections: [{ section: 'effects', text: 'Fires a blood laser' }],
      }),
      doc(2, 'Blood Bomb', { condition: 'Blow up 10 tinted rocks' }),
    ]
    expect(rankFixture(docs, 'brimstone laser', 10).hits).toHaveLength(0)
    expect(rankFixture(docs, 'blood laser', 10).hits[0]?.match).toEqual({
      kind: 'section',
      section: 'effects',
      before: 'Fires a ',
      matched: 'blood',
      after: ' laser',
    })
    expect(rankFixture(docs, 'tinted', 10).hits[0]?.match).toEqual({
      kind: 'condition',
      text: 'Blow up 10 tinted rocks',
    })
  })

  it('cuts to the limit and still says how many matched', () => {
    const docs = [1, 2, 3, 4, 5].map((id) => doc(id, `Bomb ${id}`))
    const view = rankFixture(docs, 'bomb', 2)
    expect(view.hits).toHaveLength(2)
    expect(view.total).toBe(5)
  })

  it('answers an empty query with nothing at all', () => {
    const view = rankFixture([doc(1, 'Bomb')], '   ', 10)
    expect(view.hits).toEqual([])
    expect(view.total).toBe(0)
    expect(view.diagnostics).toEqual([])
  })
})
```

- [ ] **Step 2: Run to see it fail**

Run: `pnpm ui:test -- search`
Expected: FAIL — `./search` doesn't exist.

- [ ] **Step 3: Write the wire**

`ui/src/lib/ipc/types.ts`, a new `// --- search ---` block after the wiki one:

```ts
// Mirrors crates/ipc/src/search.rs. `ProgressMark` and `SearchDiagnostic` are fieldless in
// Rust, so they travel as bare strings: values, not discriminators.
export const ProgressMark = {
  Done: 'done',
  Pending: 'pending',
  Unknown: 'unknown',
  None: 'none',
} as const
export type ProgressMark = (typeof ProgressMark)[keyof typeof ProgressMark]

export const SearchDiagnostic = {
  NoProfile: 'noProfile',
  NoCatalog: 'noCatalog',
  NoWiki: 'noWiki',
  NoAchievementSection: 'noAchievementSection',
  NoCollectionSection: 'noCollectionSection',
} as const
export type SearchDiagnostic =
  (typeof SearchDiagnostic)[keyof typeof SearchDiagnostic]

// Why the hit matched, in the words its row shows.
export type SearchMatch =
  | { kind: 'title' }
  | { kind: 'condition'; text: string }
  | {
      kind: 'section'
      section: SectionKind
      before: string
      matched: string
      after: string
    }

export interface SearchHit {
  target: Target
  title: string
  iconUrl: string | null
  hasPage: boolean
  match: SearchMatch
  progress: ProgressMark
}

export interface SearchView {
  query: string
  hits: SearchHit[]
  total: number
  diagnostics: SearchDiagnostic[]
}
```

`ui/src/lib/constants/commands.ts`: `Search: 'search',`.

`ui/src/lib/ipc/search.ts`:

```ts
import { Command } from '../constants/commands'
import { call } from './transport'
import type { SearchView } from './types'

// How many hits each caller asks for. The palette shows five rows per group and needs enough
// to fill four of them; the screen asks for everything up to a line that says it stopped.
export const SearchLimit = { Palette: 40, Screen: 300 } as const

export const search = (query: string, limit: number): Promise<SearchView> =>
  call(Command.Search, { query, limit })
```

`ui/src/lib/ipc/fixtures/search.ts` — the pack's index and sample pages, scanned with the
tiers of Decision 6. Declared synthetic in the console, like the Collection's flags: the
ranking that counts is Rust's, and this only lets the palette be looked at.

```ts
import type {
  SearchDiagnostic,
  SearchHit,
  SearchMatch,
  SearchView,
  SectionKind,
  Target,
} from '../types'

export interface FixtureSection {
  section: SectionKind
  text: string
}
export interface FixtureDoc {
  target: Target
  title: string
  condition: string | null
  sections: FixtureSection[]
  hasPage: boolean
  iconUrl: string | null
}

const fold = (s: string): string => s.toLowerCase()
const words = (query: string): string[] => fold(query).trim().split(/\s+/).filter(Boolean)
const containsAll = (folded: string, all: string[]): boolean =>
  all.length > 0 && all.every((w) => folded.includes(w))

const Before = 60
const After = 90

const fragment = (text: string, all: string[]): SearchMatch | null => {
  const folded = fold(text)
  if (!containsAll(folded, all)) return null
  const at = Math.min(...all.map((w) => folded.indexOf(w)).filter((i) => i >= 0))
  const word = all.find((w) => folded.indexOf(w) === at) ?? ''
  return {
    kind: 'section',
    section: 'effects',
    before: text.slice(Math.max(0, at - Before), at),
    matched: text.slice(at, at + word.length),
    after: text.slice(at + word.length, at + word.length + After),
  }
}

const titleTier = (folded: string, query: string): number => {
  if (folded === query) return 0
  if (folded.startsWith(query)) return 1
  if (folded.split(/\s+/).some((w) => w.startsWith(query))) return 2
  return 3
}

// Exported for its test: the ranking is the only part of the fixture worth checking.
export const rankFixture = (
  docs: FixtureDoc[],
  query: string,
  limit: number,
  diagnostics: SearchDiagnostic[] = [],
): SearchView => {
  const all = words(query)
  if (all.length === 0) return { query, hits: [], total: 0, diagnostics: [] }
  const folded = fold(query.trim())
  const ranked = docs.flatMap((doc) => {
    const title = fold(doc.title)
    if (containsAll(title, all))
      return [{ tier: titleTier(title, folded), doc, match: { kind: 'title' } as SearchMatch }]
    if (doc.condition !== null && containsAll(fold(doc.condition), all))
      return [
        { tier: 4, doc, match: { kind: 'condition', text: doc.condition } as SearchMatch },
      ]
    for (const s of doc.sections) {
      const match = fragment(s.text, all)
      if (match) return [{ tier: 5, doc, match: { ...match, section: s.section } }]
    }
    return []
  })
  ranked.sort((a, b) => a.tier - b.tier || a.doc.title.localeCompare(b.doc.title))
  const hits: SearchHit[] = ranked.slice(0, limit).map(({ doc, match }) => ({
    target: doc.target,
    title: doc.title,
    iconUrl: doc.iconUrl,
    hasPage: doc.hasPage,
    match,
    progress: 'unknown',
  }))
  return { query, hits, total: ranked.length, diagnostics }
}
```

and the answer the fixture handler calls, in the same file. `fixtures/wiki.ts` exports its
`packPages` and its `pages` map (renamed `samplePages`) for it:

```ts
import { packPages, samplePages } from './wiki'

export interface SearchAnswerOptions {
  withArt: boolean
  withCatalog: boolean
  withWiki: boolean
}

let warned = false

// Development only, and **synthetic**: the ranking that counts is Rust's. This exists so the
// palette and the Search screen can be looked at without the backend.
export const searchAnswer = (
  { withArt, withCatalog, withWiki }: SearchAnswerOptions,
  query: string,
  limit: number,
): SearchView => {
  if (!warned) {
    warned = true
    console.warn('search fixture: the ranking is synthetic; the real one is in crates/ipc')
  }
  const diagnostics: SearchDiagnostic[] = [
    ...(withCatalog ? [] : (['noCatalog'] as const)),
    ...(withWiki ? [] : (['noWiki'] as const)),
    'noProfile' as const,
  ]
  const docs: FixtureDoc[] = withWiki
    ? packPages().map((page) => {
        const key = pageKey(page.target)
        const sample = key === null ? undefined : samplePages.get(key)
        return {
          target: page.target,
          title: sample?.title ?? page.title,
          condition: conditionOf(page.target),
          sections: (sample?.sections ?? []).map((s) => ({
            section: s.kind,
            text: sectionText(s),
          })),
          hasPage: true,
          iconUrl: withArt && withCatalog ? iconOf(page.target, page.bossId) : null,
        }
      })
    : []
  return rankFixture(docs, query, limit, diagnostics)
}
```

with `conditionOf` reading `unlock.json`'s nodes for an achievement's `hint`, `sectionText`
flattening a sample's blocks the way `crates/ipc` does (text, `ref`/`concept` labels, editions
unwrapped), and `iconOf` the one `fixtures/wiki.ts` already has, exported beside `packPages`.

`ui/src/lib/ipc/fixtures/index.ts`: a handler for `Command.Search` that awaits the module and
answers `searchAnswer(...)` with `artShown()`, `catalogShown()` and `wikiShown()`.

- [ ] **Step 4: Run the tests**

Run: `pnpm ui:test -- search` then `pnpm typecheck`
Expected: PASS (4 tests), no type errors.

- [ ] **Step 5: Commit and push**

```bash
git add ui/src/lib/ipc/types.ts ui/src/lib/ipc/search.ts ui/src/lib/constants/commands.ts ui/src/lib/ipc/fixtures/search.ts ui/src/lib/ipc/fixtures/search.test.ts ui/src/lib/ipc/fixtures/wiki.ts ui/src/lib/ipc/fixtures/index.ts
git commit -m "feat(ui): the search command's types, wrapper and development fixture"
git push
```

---

### Task 8: the route, the origin and the `q` in a location

**Files:**
- Modify: `ui/src/router/routeTable.ts`, `ui/src/router/routes.ts`, `ui/src/components/shell/tabs.ts`, `ui/src/components/shell/tabOriginIcon.ts`, `ui/src/components/shell/sectionNav.ts`, `ui/src/App.vue`, `ui/src/i18n/messages/{it,en}.ts`
- Create: `ui/src/screens/SearchScreen.vue` (header and query field only; the results are Task 12)
- Test: `ui/src/router/routeTable.test.ts`, `ui/src/components/shell/sectionNav.test.ts`

**Interfaces:**
- Produces: `RouteName.Search = 'search'` at `/search`; `TabOrigin.Search = 'search'`; `TabLocation.query.q?: string`; `sectionOfOrigin(origin): SidebarSection | null`.

- [ ] **Step 1: Write the failing tests** — append to `ui/src/router/routeTable.test.ts`:

```ts
it('search is a location of its own, with the query in it', () => {
  expect(routePath[RouteName.Search]).toBe('/search')
  expect(routeOrigin[RouteName.Search]).toBe(TabOrigin.Search)
  // It needs no profile: the gate is only for the Progress origin.
  expect(routeOrigin[RouteName.Search]).not.toBe(TabOrigin.Progress)
  const location: TabLocation = {
    name: RouteName.Search,
    query: { q: 'brimstone' },
  }
  expect(locationTitle(location)).toBe('routes.search')
})
```

and to `ui/src/components/shell/sectionNav.test.ts`:

```ts
it('search belongs to no sidebar section', () => {
  // It sits above the two sections (DESIGN-BRIEF.md §4.2): the sidebar keeps showing
  // whatever it was showing, rather than jumping to one of them.
  expect(sectionOfOrigin(TabOrigin.Search)).toBeNull()
  expect(sectionOfOrigin(TabOrigin.Wiki)).toBe(SidebarSection.Wiki)
})
```

- [ ] **Step 2: Run to see them fail**

Run: `pnpm ui:test -- routeTable sectionNav`
Expected: FAIL — `RouteName.Search` and `TabOrigin.Search` don't exist.

- [ ] **Step 3: Add the route**

- `tabs.ts`: `Search: 'search'` in `TabOrigin`.
- `tabOriginIcon.ts`: `[TabOrigin.Search]: SearchIcon` (from `@lucide/vue`).
- `routeTable.ts`: `Search: 'search'` in `RouteName`; `q?: string` in `TabLocation['query']`;
  `/search` in `routePath`; `'routes.search'` in `routeTitle`; `TabOrigin.Search` in
  `routeOrigin`; `tabOriginIcon[TabOrigin.Search]` in `routeIcon`.
- `sectionNav.ts`: `sectionOfOrigin` returns `SidebarSection | null`, with
  `case TabOrigin.Search: return null` and the `assertNever` default kept.
- `App.vue`: the watch that follows the active tab keeps the current section when the origin
  has none:

```ts
watch(
  () => tabs.active?.location.name,
  (name) => {
    if (!name) return
    // Search belongs to neither section: the sidebar stays where the user left it.
    const section = sectionOfOrigin(routeOrigin[name])
    if (section) browsing.value = section
  },
  { immediate: true },
)
```

- `routes.ts`: `[RouteName.Search]: SearchScreen` in the `screens` map.
- `SearchScreen.vue`, this task's half: `ScreenHeader` with the search icon and
  `t('routes.search')`, an `Input` bound to the location's `q` — typing navigates the active
  tab (debounced with `Timing.SearchDebounce`), so **the tab is the search** and survives as
  one — and nothing else yet.
- i18n: `routes.search` ("Cerca" / "Search"), `search.placeholder`
  ("Cerca schermate, achievement, oggetti, pagine wiki…"), `search.intro`.

- [ ] **Step 4: Run the tests**

Run: `pnpm ui:test -- routeTable sectionNav` then `pnpm typecheck && pnpm scan`
Expected: PASS; the compiler lists every exhaustive `switch` that needs the new origin.

- [ ] **Step 5: Commit and push**

```bash
git add ui/src/router ui/src/components/shell ui/src/App.vue ui/src/screens/SearchScreen.vue ui/src/i18n
git commit -m "feat(ui): search is a location of its own, above the two sections"
git push
```

---

### Task 9: rows are destinations, and the stale answer is dropped

**Files:**
- Create: `ui/src/lib/search/rows.ts`, `ui/src/lib/search/latest.ts`, `ui/src/lib/constants/timing.ts`
- Test: `ui/src/lib/search/rows.test.ts`, `ui/src/lib/search/latest.test.ts`

**Interfaces:**
- Consumes: `SearchHit` (Task 7), `pageLocation`/`categoryOf` (`lib/wiki/category.ts`), `RouteName`, `TabLocation` (Task 8).
- Produces: `RowGroup`, `rowGroupOrder`, `ScreenEntry`, `SearchRow`, `screenEntries(t)`, `matchingScreens(entries, query)`, `searchRows(hits, screens, options)`, `groupCounts(rows)`, `latest()`.

- [ ] **Step 1: Write the failing tests** — `ui/src/lib/search/rows.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import type { SearchHit } from '@/lib/ipc/types'
import { RouteName } from '@/router/routeTable'
import { RowGroup, groupCounts, searchRows } from './rows'

const hit = (over: Partial<SearchHit>): SearchHit => ({
  target: { kind: 'item', id: 105 },
  title: 'The D6',
  iconUrl: null,
  hasPage: true,
  match: { kind: 'title' },
  progress: 'unknown',
  ...over,
})

const screens = [
  {
    key: 'route-collection',
    label: 'routes.collection' as const,
    text: 'Collezione',
    location: { name: RouteName.Collection },
  },
]

describe('searchRows', () => {
  it('turns one hit into every destination it can open', () => {
    const rows = searchRows([hit({})], [], { catalog: true, cap: null })
    expect(rows.map((r) => r.group)).toEqual([RowGroup.Wiki, RowGroup.Collection])
    // The Collection row opens the list already filtered on the item's name (B3).
    const collection = rows.find((r) => r.group === RowGroup.Collection)
    expect(collection?.location).toEqual({
      name: RouteName.Collection,
      query: { q: 'The D6' },
    })
    // The Wiki row is the page itself, with its category so the sidebar stays lit.
    expect(rows[0]?.location).toEqual({
      name: RouteName.Wiki,
      query: { category: 'items', page: 'item:105' },
    })
  })

  it('gives an achievement an Unlock row, and a page-less hit no Wiki row', () => {
    const rows = searchRows(
      [hit({ target: { kind: 'achievement', id: 3 }, title: 'You unlocked X', hasPage: false })],
      [],
      { catalog: true, cap: null },
    )
    expect(rows.map((r) => r.group)).toEqual([RowGroup.Unlock])
  })

  it('drops the Unlock and Collection rows without a catalog', () => {
    // Those two lists have no names to filter by when the game isn't installed.
    const rows = searchRows([hit({})], [], { catalog: false, cap: null })
    expect(rows.map((r) => r.group)).toEqual([RowGroup.Wiki])
  })

  it('lists the screens first and caps each group in the palette', () => {
    const many = [1, 2, 3, 4, 5, 6].map((id) =>
      hit({ target: { kind: 'item', id }, title: `Bomb ${id}` }),
    )
    const rows = searchRows(many, screens, { catalog: true, cap: 5 })
    expect(rows[0]?.group).toBe(RowGroup.Screens)
    const counts = groupCounts(rows)
    expect(counts[RowGroup.Wiki]).toBe(5)
    expect(counts[RowGroup.Collection]).toBe(5)
    // Uncapped, the screen shows all six of each.
    const all = groupCounts(searchRows(many, screens, { catalog: true, cap: null }))
    expect(all[RowGroup.Wiki]).toBe(6)
  })
})
```

`ui/src/lib/search/latest.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { latest } from './latest'

describe('latest', () => {
  it('keeps only the answer to the newest question', () => {
    // A slow scan must never overwrite a fresher result.
    const guard = latest()
    const first = guard.next()
    const second = guard.next()
    expect(guard.isCurrent(second)).toBe(true)
    expect(guard.isCurrent(first)).toBe(false)
  })
})
```

- [ ] **Step 2: Run to see them fail**

Run: `pnpm ui:test -- rows latest`
Expected: FAIL — the modules don't exist.

- [ ] **Step 3: Write the two modules**

`ui/src/lib/search/latest.ts`:

```ts
// The palette asks again on every keystroke and the answers come back out of order: each
// question takes a number, and only the newest one's answer is shown.
export const latest = (): {
  next: () => number
  isCurrent: (token: number) => boolean
} => {
  let seq = 0
  return {
    next: () => ++seq,
    isCurrent: (token: number) => token === seq,
  }
}
```

`ui/src/lib/search/rows.ts`:

```ts
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import type { SearchHit } from '@/lib/ipc/types'
import { pageLocation } from '@/lib/wiki/category'
import {
  RouteName,
  WikiCategory,
  routeTitle,
  wikiCategoryTitle,
} from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'

type Message = MessageKey<MessageSchema>

// A result is not a row: it is the **destinations** it can open. "Brimstone" is a wiki page
// and a Collection row, and the two are different places (DESIGN-BRIEF.md §4.2).
export const RowGroup = {
  Screens: 'screens',
  Wiki: 'wiki',
  Unlock: 'unlock',
  Collection: 'collection',
} as const
export type RowGroup = (typeof RowGroup)[keyof typeof RowGroup]

export const rowGroupOrder: RowGroup[] = [
  RowGroup.Screens,
  RowGroup.Wiki,
  RowGroup.Unlock,
  RowGroup.Collection,
]

export interface ScreenEntry {
  key: string
  label: Message
  /** The label as the user reads it: what the query is matched against. */
  text: string
  location: TabLocation
}

export type SearchRow =
  | { kind: 'screen'; key: string; group: RowGroup; entry: ScreenEntry }
  | { kind: 'hit'; key: string; group: RowGroup; hit: SearchHit; location: TabLocation }

// Every screen and wiki category, named as the user sees them: the frontend answers these
// itself, because the backend knows nothing about the app's own pages.
export const screenEntries = (t: (m: Message) => string): ScreenEntry[] => [
  ...Object.values(RouteName)
    .filter((name) => name !== RouteName.Search)
    .map((name) => ({
      key: `route-${name}`,
      label: routeTitle[name],
      text: t(routeTitle[name]),
      location: { name },
    })),
  ...Object.values(WikiCategory).map((category) => ({
    key: `wiki-${category}`,
    label: wikiCategoryTitle[category],
    text: t(wikiCategoryTitle[category]),
    location: { name: RouteName.Wiki, query: { category } },
  })),
]

export const matchingScreens = (
  entries: ScreenEntry[],
  query: string,
): ScreenEntry[] => {
  const wanted = query.trim().toLowerCase()
  return wanted === ''
    ? []
    : entries.filter((e) => e.text.toLowerCase().includes(wanted))
}

export interface RowOptions {
  /** Without the game there are no names for Unlock and the Collection to filter by. */
  catalog: boolean
  /** How many rows a group shows: five in the palette, all of them on the screen. */
  cap: number | null
}

const destinations = (hit: SearchHit, catalog: boolean): SearchRow[] => {
  const rows: SearchRow[] = []
  const page = pageLocation(hit.target)
  if (hit.hasPage && page)
    rows.push({ kind: 'hit', key: `wiki-${hit.title}`, group: RowGroup.Wiki, hit, location: page })
  if (!catalog) return rows
  if (hit.target.kind === 'achievement')
    rows.push({
      kind: 'hit',
      key: `unlock-${hit.title}`,
      group: RowGroup.Unlock,
      hit,
      location: { name: RouteName.Unlock, query: { q: hit.title } },
    })
  if (hit.target.kind === 'item')
    rows.push({
      kind: 'hit',
      key: `collection-${hit.title}`,
      group: RowGroup.Collection,
      hit,
      location: { name: RouteName.Collection, query: { q: hit.title } },
    })
  return rows
}

export const searchRows = (
  hits: SearchHit[],
  screens: ScreenEntry[],
  { catalog, cap }: RowOptions,
): SearchRow[] => {
  const rows: SearchRow[] = [
    ...screens.map((entry) => ({
      kind: 'screen' as const,
      key: entry.key,
      group: RowGroup.Screens,
      entry,
    })),
    ...hits.flatMap((hit) => destinations(hit, catalog)),
  ]
  // The backend's order is kept inside each group: the frontend never re-ranks.
  return rowGroupOrder.flatMap((group) => {
    const inGroup = rows.filter((r) => r.group === group)
    return cap === null ? inGroup : inGroup.slice(0, cap)
  })
}

export const groupCounts = (rows: SearchRow[]): Record<RowGroup, number> =>
  Object.fromEntries(
    rowGroupOrder.map((group) => [group, rows.filter((r) => r.group === group).length]),
  ) as Record<RowGroup, number>
```

`ui/src/lib/constants/timing.ts`:

```ts
// Delays measured in milliseconds that live in TypeScript, not in a class: a debounce is not
// a visual constant, and the value is written once here.
export const Timing = { SearchDebounce: 120 } as const
```

- [ ] **Step 4: Run the tests**

Run: `pnpm ui:test -- rows latest`
Expected: PASS (5 tests).

- [ ] **Step 5: Commit and push**

```bash
git add ui/src/lib/search ui/src/lib/constants/timing.ts
git commit -m "feat(ui): a search result becomes the destinations it can open"
git push
```

---

### Task 10: the `Command` primitive stops scoring when the backend already did

**Files:**
- Create: `ui/src/components/ui/command/filter.ts`
- Modify: `ui/src/components/ui/command/Command.vue`, `CommandDialog.vue`, `index.ts`
- Test: `ui/src/components/ui/command/filter.test.ts`

**Interfaces:**
- Produces: `scoreItems(items, groups, search, filter, contains): Filtered`; `Command` props `filter?: boolean` (default `true`) and `search?: string` with `update:search`; both forwarded by `CommandDialog`.

- [ ] **Step 1: Write the failing test** — `ui/src/components/ui/command/filter.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { scoreItems } from './filter'

const items = new Map([
  ['a', 'Brimstone'],
  ['b', 'Blood Bomb'],
])
const groups = new Map([['g1', new Set(['a'])], ['g2', new Set(['b'])]])
const contains = (text: string, query: string) =>
  text.toLowerCase().includes(query.toLowerCase())

describe('scoreItems', () => {
  it('scores nothing when the list is already filtered', () => {
    // The palette's rows come from the backend: scoring them again would hide rows whose
    // text doesn't repeat the query — a section fragment, a screen's name.
    const filtered = scoreItems(items, groups, 'zzz', false, contains)
    expect(filtered.count).toBe(2)
    expect(filtered.items.size).toBe(0)
    expect([...filtered.groups]).toEqual(['g1', 'g2'])
  })

  it('scores every item when it does filter', () => {
    const filtered = scoreItems(items, groups, 'blood', true, contains)
    expect(filtered.count).toBe(1)
    expect(filtered.items.get('a')).toBe(0)
    expect(filtered.items.get('b')).toBe(1)
    expect([...filtered.groups]).toEqual(['g2'])
  })

  it('shows everything on an empty search', () => {
    const filtered = scoreItems(items, groups, '', true, contains)
    expect(filtered.count).toBe(2)
    expect(filtered.items.size).toBe(0)
  })
})
```

- [ ] **Step 2: Run to see it fail**

Run: `pnpm ui:test -- filter`
Expected: FAIL — `./filter` doesn't exist.

- [ ] **Step 3: Extract the scoring and add the two props**

`ui/src/components/ui/command/filter.ts`:

```ts
import type { CommandFilterState } from './context'

type Filtered = CommandFilterState['filtered']

// The scoring, out of the component so it can be stated in a test. `filter: false` is the one
// change B12 item 4 asks for that the palette can't do without: the rows are already the
// backend's answer, so every mounted item shows and every group stays visible.
export const scoreItems = (
  items: Map<string, string>,
  groups: Map<string, Set<string>>,
  search: string,
  filter: boolean,
  contains: (text: string, query: string) => boolean,
): Filtered => {
  if (!filter || !search)
    return { count: items.size, items: new Map(), groups: new Set(groups.keys()) }
  const scores = new Map(
    [...items].map(([id, text]): [string, number] => [id, contains(text, search) ? 1 : 0]),
  )
  return {
    count: [...scores.values()].filter((score) => score > 0).length,
    items: scores,
    groups: new Set(
      [...groups]
        .filter(([, ids]) => [...ids].some((id) => (scores.get(id) ?? 0) > 0))
        .map(([id]) => id),
    ),
  }
}
```

`Command.vue`: the props gain `filter?: boolean` (default `true`) and `search?: string`;
`reactiveOmit(props, 'class', 'filter', 'search')` keeps both off `ListboxRoot`; the emits
gain `'update:search': [value: string]`; two watches keep `filterState.search` and the prop in
step, so the palette owns the typed text and can debounce it; and `filterItems` becomes

```ts
const filterItems = () => {
  Object.assign(
    filterState.filtered,
    scoreItems(allItems.value, allGroups.value, filterState.search, props.filter, contains),
  )
}
watch(
  [() => filterState.search, () => props.filter, () => allItems.value.size],
  filterItems,
)
```

`CommandDialog.vue` declares the same two and forwards them to `Command` (`:filter`,
`v-model:search`), because the palette never mounts `Command` itself; its
`reactiveOmit(props, 'title', 'description', 'class')` grows `'filter'` and `'search'`, so
neither reaches `DialogRoot`.

- [ ] **Step 4: Run the tests**

Run: `pnpm ui:test -- filter` then `pnpm typecheck`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit and push**

```bash
git add ui/src/components/ui/command
git commit -m "feat(ui): Command can stop scoring a list the backend already filtered"
git push
```

---

### Task 11: the palette, from any tab

**Files:**
- Create: `ui/src/composables/useSearch.ts`, `ui/src/components/search/SearchPalette.vue`, `ui/src/components/search/SearchRow.vue`
- Modify: `ui/src/lib/constants/eventKeys.ts`, `ui/src/App.vue`, `ui/src/i18n/messages/{it,en}.ts`, `ui/src/kit/KitPage.vue` + a Kit section for the palette's row
- Test: `ui/src/composables/useSearch.test.ts`

**Interfaces:**
- Consumes: `search`, `SearchLimit` (Task 7), `searchRows`, `screenEntries`, `matchingScreens`, `latest`, `Timing` (Task 9), `Command`'s `filter`/`search` (Task 10).
- Produces: `useSearch(limit: number): { view: Ref<SearchView | null>; error: Ref<IpcError | null>; ask: (query: string) => void }`; `<SearchPalette v-model:open>`; `EventKey.K = 'k'`.

- [ ] **Step 1: Write the failing test** — `ui/src/composables/useSearch.test.ts`:

```ts
import { describe, expect, it, vi } from 'vitest'
import { searchFrom } from './useSearch'
import type { SearchView } from '@/lib/ipc/types'

const view = (query: string): SearchView => ({
  query,
  hits: [],
  total: 0,
  diagnostics: [],
})

describe('searchFrom', () => {
  it('keeps the answer to the last question, whatever order they come back in', async () => {
    // The slow first answer must not overwrite the fast second one.
    let release: (v: SearchView) => void = () => {}
    const pending = new Promise<SearchView>((resolve) => {
      release = resolve
    })
    const call = vi
      .fn()
      .mockImplementationOnce(() => pending)
      .mockImplementationOnce(() => Promise.resolve(view('br')))
    const state = searchFrom(call, 10)
    const slow = state.run('b')
    await state.run('br')
    expect(state.view.value?.query).toBe('br')
    release(view('b'))
    await slow
    expect(state.view.value?.query).toBe('br')
  })

  it('an empty query clears the answer without asking', async () => {
    const call = vi.fn()
    const state = searchFrom(call, 10)
    await state.run('   ')
    expect(call).not.toHaveBeenCalled()
    expect(state.view.value).toBeNull()
  })
})
```

- [ ] **Step 2: Run to see it fail**

Run: `pnpm ui:test -- useSearch`
Expected: FAIL — the composable doesn't exist.

- [ ] **Step 3: Write the composable and the palette**

`ui/src/composables/useSearch.ts`:

```ts
import { useDebounceFn } from '@vueuse/core'
import { ref } from 'vue'
import type { Ref } from 'vue'
import { Timing } from '@/lib/constants/timing'
import { isIpcError } from '@/lib/ipc/errors'
import { search } from '@/lib/ipc/search'
import type { IpcError, SearchView } from '@/lib/ipc/types'
import { latest } from '@/lib/search/latest'

// One search in flight per caller. The palette and the Search screen each have their own:
// they ask different questions, and a shared store would make one overwrite the other.
export const searchFrom = (
  call: (query: string, limit: number) => Promise<SearchView>,
  limit: number,
): {
  view: Ref<SearchView | null>
  error: Ref<IpcError | null>
  run: (query: string) => Promise<void>
} => {
  const view = ref<SearchView | null>(null)
  const error = ref<IpcError | null>(null)
  const guard = latest()
  const run = async (query: string): Promise<void> => {
    const token = guard.next()
    if (query.trim() === '') {
      view.value = null
      return
    }
    try {
      const answer = await call(query, limit)
      // An answer to an older question than the one typed is dropped.
      if (guard.isCurrent(token)) view.value = answer
    } catch (e) {
      if (guard.isCurrent(token)) error.value = isIpcError(e) ? e : null
    }
  }
  return { view, error, run }
}

export const useSearch = (limit: number) => {
  const state = searchFrom(search, limit)
  return { ...state, ask: useDebounceFn(state.run, Timing.SearchDebounce) }
}
```

`ui/src/lib/constants/eventKeys.ts`: `K: 'k'` — `KeyboardEvent.key` is lowercase for an
unshifted letter, and the palette compares `event.key.toLowerCase()`.

`ui/src/components/search/SearchPalette.vue`:
- `const open = defineModel<boolean>('open', { required: true })`.
- `useShortcut((event) => { if (!event.ctrlKey || event.key.toLowerCase() !== EventKey.K) return false; open.value = true; return true })` — mounted once in `App.vue`, so the shortcut works from any tab.
- `CommandDialog` with `v-model:open`, `:filter="false"`, `v-model:search="typed"`, the title and description from `search.placeholder` / `search.intro`, and `CommandInput` inside it.
- `watch(typed, (q) => void ask(q))`; rows are
  `searchRows(view?.hits ?? [], matchingScreens(screenEntries(t), typed), { catalog: !view?.diagnostics.includes(SearchDiagnostic.NoCatalog), cap: 5 })`.
- One `CommandGroup` per `rowGroupOrder` entry that has rows, headed by `search.groups.*`, a `SearchRow` per row, and a last `CommandItem` reading `t('search.allResults', { count: view?.total ?? 0 })` that navigates to `{ name: RouteName.Search, query: { q: typed } }`.
- Opening: `@select` and a click call `go(row, event)` — `tabs.navigate(row.location)`, or `tabs.open(row.location)` when `event.ctrlKey`; `Ctrl+Enter` is the keyboard half, read from the same handler. The palette closes either way.
- `CommandFooter` with the `Kbd` hints, as the Kit's Command section already draws them.

`ui/src/components/search/SearchRow.vue`: the icon (`PixelSprite` on `hit.iconUrl`, the group's
Lucide icon for a screen row), the title, and one second line — the fragment with `matched`
emphasised, the condition, or nothing for a title match — plus a `Badge` with the progress
mark when it isn't `none`.

`ui/src/App.vue`: `<SearchPalette v-model:open="paletteOpen" />` beside `AboutDialog`, and
`@search="paletteOpen = true"` on `NavBar` (the emit exists since 3.1 and opened nothing).

i18n, Italian first and English mirrored: `search.groups.{screens,wiki,unlock,collection}`,
`search.allResults` ("Tutti i risultati ({count})"), `search.empty`, `search.hint.{open,newTab}`,
`search.progress.{done,pending,unknown}`.

A Kit section (`kit/sections/app/SearchRowSection.vue`) draws the four row shapes — title
match, condition, section fragment, screen — because the palette itself can't be opened on
the Kit page.

- [ ] **Step 4: Run the tests and look at it**

Run: `pnpm ui:test -- useSearch` then `pnpm typecheck && pnpm lint && pnpm scan`
Expected: PASS, no violations.

- [ ] **Step 5: Commit and push**

```bash
git add ui/src/composables ui/src/components/search ui/src/lib/constants/eventKeys.ts ui/src/App.vue ui/src/i18n ui/src/kit
git commit -m "feat(ui): the Ctrl+K palette, whose rows are destinations"
git push
```

---

### Task 12: the Search screen

**Files:**
- Modify: `ui/src/screens/SearchScreen.vue`, `ui/src/lib/search/rows.ts`, `ui/src/lib/scale/rows.ts`, `ui/src/assets/theme/spacing.css`, `ui/src/i18n/messages/{it,en}.ts`
- Create: `ui/src/screens/search/SearchToolbar.vue`, `ui/src/screens/search/SearchResults.vue`, `ui/src/screens/search/SearchDiagnostics.vue`
- Test: `ui/src/lib/search/rows.test.ts` (the group filter), `ui/src/lib/scale/rows.test.ts`

**Interfaces:**
- Consumes: `useSearch(SearchLimit.Screen)`, `searchRows(..., { cap: null })`, `groupCounts`.
- Produces: `filterGroups(rows: SearchRow[], picked: RowGroup[]): SearchRow[]`; `rowResultPx(percent: number): number`; the token `--spacing-row-result`.

- [ ] **Step 1: Write the failing tests** — append to `ui/src/lib/search/rows.test.ts`:

```ts
it('shows every group until one is picked', () => {
  const rows = searchRows([hit({})], screens, { catalog: true, cap: null })
  // No pick is "all of them": an empty toggle group never means an empty screen.
  expect(filterGroups(rows, [])).toHaveLength(rows.length)
  expect(filterGroups(rows, [RowGroup.Wiki]).map((r) => r.group)).toEqual([
    RowGroup.Wiki,
  ])
})
```

and to `ui/src/lib/scale/rows.test.ts`:

```ts
it('a result row is measured from its own token', () => {
  // Two lines, so it is taller than a table row: the virtualizer positions it with the same
  // number the class draws it with, at every scale (the defect 3.5c found).
  expect(rowResultPx(100)).toBe(56)
  expect(rowResultPx(200)).toBe(112)
})
```

- [ ] **Step 2: Run to see them fail**

Run: `pnpm ui:test -- rows`
Expected: FAIL — `filterGroups` and `rowResultPx` don't exist.

- [ ] **Step 3: Write the screen**

- `ui/src/assets/theme/spacing.css`: `--spacing-row-result: 3.5rem;` beside the other row
  tokens (a two-line row: the icon, the title and the fragment).
- `ui/src/lib/scale/rows.ts`: `export const RowResultRem = 3.5` and
  `export const rowResultPx = (percent: number): number => remToPx(RowResultRem, percent)`.
- `ui/src/lib/search/rows.ts`: `filterGroups`, exactly as the test states — an empty pick is
  every group.
- `SearchToolbar.vue`: a multiple `ToggleGroup` over Wiki, Unlock, Collezione with their
  counts from `groupCounts`, and the shown/total line.
- `SearchDiagnostics.vue`: one `Alert` per diagnostic, `noProfile` as a note rather than a
  warning — it is the expected state before a save is chosen.
- `SearchResults.vue`: the rows, virtualized with `@tanstack/vue-virtual` past sixty
  (`estimateSize: () => rowResultPx(settings.scale)`, as `UnlockTable` does), each drawn by
  the same `SearchRow` the palette uses, with the same click gestures (`Ctrl` opens a tab).
- `SearchScreen.vue` completes: the query field bound to the tab's `q` (Task 8), the toolbar,
  the diagnostics, the results, and the line `search.limit` ("mostrati {shown} di {total}:
  affina la ricerca") when `total` is above what came back.

- [ ] **Step 4: Run the tests**

Run: `pnpm ui:test` then `pnpm typecheck && pnpm lint && pnpm scan`
Expected: PASS.

- [ ] **Step 5: Commit and push**

```bash
git add ui/src/screens ui/src/lib/search ui/src/lib/scale ui/src/assets/theme/spacing.css ui/src/i18n
git commit -m "feat(ui): the Search screen, its toggles and its virtualized rows"
git push
```

---

### Task 13: Unlock and the Collection open already filtered

**Files:**
- Create: `ui/src/lib/search/queryParam.ts`
- Modify: `ui/src/screens/UnlockScreen.vue`, `ui/src/screens/CollectionScreen.vue`, `ui/src/screens/WikiScreen.vue` (its local copy of the same helper)
- Test: `ui/src/lib/search/queryParam.test.ts`

**Interfaces:**
- Produces: `singleQuery(value: unknown): string | null`.

- [ ] **Step 1: Write the failing test** — `ui/src/lib/search/queryParam.test.ts`:

```ts
import { describe, expect, it } from 'vitest'
import { singleQuery } from './queryParam'

describe('singleQuery', () => {
  it('takes one string and nothing else', () => {
    expect(singleQuery('brimstone')).toBe('brimstone')
    // The router hands an array when a key repeats, and null when it has no value.
    expect(singleQuery(['a', 'b'])).toBeNull()
    expect(singleQuery(null)).toBeNull()
    expect(singleQuery(undefined)).toBeNull()
  })
})
```

- [ ] **Step 2: Run to see it fail**

Run: `pnpm ui:test -- queryParam`
Expected: FAIL — the module doesn't exist.

- [ ] **Step 3: Write it and use it in three screens**

```ts
// A route query value is a string, an array of them, or nothing: only one string is a query.
export const singleQuery = (value: unknown): string | null =>
  typeof value === 'string' ? value : null
```

In `UnlockScreen.vue` and `CollectionScreen.vue`, the filter starts from the location's `q`:

```ts
// A Search row opens the list already filtered on the name it found (B3, spec Decision 8).
const route = useRoute()
watch(
  () => route.query.q,
  (value) => {
    const q = singleQuery(value)
    if (q !== null) filter.value = { ...filter.value, query: q }
  },
  { immediate: true },
)
```

`WikiScreen.vue` drops its local `single` for the shared helper. Nothing else on those screens
changes.

- [ ] **Step 4: Run the tests**

Run: `pnpm ui:test` then `pnpm typecheck`
Expected: PASS.

- [ ] **Step 5: Commit and push**

```bash
git add ui/src/lib/search/queryParam.ts ui/src/lib/search/queryParam.test.ts ui/src/screens/UnlockScreen.vue ui/src/screens/CollectionScreen.vue ui/src/screens/WikiScreen.vue
git commit -m "feat(ui): a search result opens Unlock and the Collection already filtered"
git push
```

---

### Task 14: seen by eye, written down, merged

**Files:**
- Modify: `DESIGN-BRIEF.md` (§8 gains the search view), `docs/STATUS.md`, `docs/BACKLOG.md` (B5), `docs/superpowers/specs/2026-09-12-screens-wiki-search-design.md` (the two corrections below)
- Create: `docs/superpowers/plans/2026-09-12-screens-search-report.md`

- [ ] **Step 1: The whole check, with the skips visible**

Run: `scripts/check`
Expected: green, and the `skip:` lines counted rather than hidden.

- [ ] **Step 2: Look at it through Playwright**, on `pnpm ui:dev`:

- the palette on "brim": four groups, five rows each, the "Tutti i risultati" row;
- `Enter` opens the Wiki page in the active tab, `Ctrl+Enter` opens it beside;
- an Unlock row: the list opens with its filter already carrying the name;
- the Search screen with the toggles and their counts, and the limit line on a query with more than 300 hits;
- `?catalog=none`: no Unlock or Collection rows, and the `noCatalog` note;
- `?wiki=none`: catalog names only, nothing offering a page;
- `?fixture=none`: the `noProfile` note, and no mark on any row;
- 200% scale: the palette and the result rows still whole (the defect 3.5c found).

- [ ] **Step 3: Write the report and update the documents**

- `DESIGN-BRIEF.md` §8: the `SearchView` types and the two commands, handed on as a contract
  change (the IPC contract is live).
- `docs/STATUS.md`: 3.5b ticked under cycle 3, a session-log entry saying what was measured
  (the index's build time and the three queries from `search_real.rs`), and what was **not**
  seen — a real Tauri window.
- `docs/BACKLOG.md`: B5 closed, naming the fallback that stays available (FTS5 in `store`) and
  the timings that made it unnecessary.
- The spec's three corrections, written into the spec itself so the two never disagree:
  `SearchDiagnostic` is a bare string, not a tagged object (the IPC rule on fieldless enums);
  the palette's query needs `Command` to carry `v-model:search`, so the primitive gains two
  props rather than one; and the search state is a **composable**, not `stores/search.ts` —
  the palette and the screen ask different questions at the same time, and one Pinia store
  would have each overwrite the other's answer.

- [ ] **Step 4: Merge into `develop`**

```bash
git checkout develop
git merge --no-ff feature/screens-search -m "merge: screens 3.5b, search, into develop"
scripts/check
git push
```

- [ ] **Step 5: Commit the documents**

```bash
git add DESIGN-BRIEF.md docs/STATUS.md docs/BACKLOG.md docs/superpowers
git commit -m "docs: cycle 3.5b lands, one index and two ways to ask it"
git push
```

---

## Out of scope for this plan

- Tabs that survive a restart (3.7): a Search tab is a location like any other, ready to be saved, but nothing saves it.
- Search over the plan queue, the run archive (M4) and the options.
- The rest of B12 item 4: the `Command` filter's own tests beyond `scoreItems`, `autoFocus` as a prop, pruning group ids on unmount.
- FTS5 in `store`: only if the timings printed by `search_real.rs` turn out to be felt. Nothing in the contract changes if they are.
