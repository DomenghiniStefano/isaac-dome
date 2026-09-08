# Wiki dataset — implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** bring wiki.gg's text sections into the app for items, trinkets, achievements, bosses, challenges, and characters as a typed tree with references resolved to our own ids, built at build time and embedded in the binary.

**Architecture:** pure `wiki` crate (serde types, wikitext parser, resolver, `build`, `Dataset::embedded()`); `wiki-snapshot` tool (the only one talking to the network: `fetch` writes `dataset/raw/`, `build` writes `dataset/wiki.json`); `ipc::wiki` re-exposes the types plus a `WikiInfo`; `app` adds the `wiki_entry` command; the verification screen shows an `Entry`.

**Tech Stack:** Rust 2021, `serde`/`serde_json`, `ureq` (tool only), Tauri 2, Vue 3 + TypeScript.

**Spec:** `docs/superpowers/specs/2026-09-05-wiki-dataset-design.md`

**Deviations from the spec, decided while writing the plan** (the spec needs updating in Task 13):

1. No `raw/characters.json` and no tool dependency on `catalog`/`unpack`: characters are resolved from the infoboxes of `character` pages (title → `id`, and `id` is the one from `players.xml`: Cain has `id = 2`). The tool depends only on `wiki` and `ureq`.
2. No `WikiState` in `app`: `Dataset::embedded()` is already a `OnceLock` inside `wiki`, the app just calls it.
3. `SectionKind` is a fieldless enum → bare string (repo rule), as already written in the spec; same for `Style` and `Dlc`.

## Global Constraints

- Every struct crossing the IPC: `#[serde(rename_all = "camelCase")]`; enum with data: `#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]`; fieldless enum: bare camelCase string, never tagged.
- No `panic!`/`unwrap()` outside tests on data read from disk or network. Odd wikitext degrades, it doesn't terminate.
- No `_ =>` branch on our own closed enums.
- Tauri commands: `Result<T, IpcError>`.
- Never `format!("{:?}")` for IPC.
- Commits in Italian, module prefix (`wiki:`, `wiki-snapshot:`, `ipc:`, `app-shell:`, `discovery:`, `docs:`, `chore:`), **never** a `Co-Authored-By` trailer or references to Claude.
- Before saying "done": `cargo test --workspace`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `pnpm typecheck`, `pnpm lint`, `pnpm format:check`, `pnpm scan`.
- Frontend: no `<style>`, no hardcoded visual constants, no `invoke()` outside `ui/src/lib/ipc/`, `as const` for discriminators.
- `samples/` is gitignored; `dataset/` is **not**: `raw/` and `wiki.json` get committed.
- The tool talks to the wiki sequentially, with an identified `User-Agent` and `maxlag=5`. No test makes network calls.

## File structure

```
crates/wiki/
  Cargo.toml
  src/lib.rs            re-export
  src/model.rs          Entry, Section, SectionKind, Block, ListItem, Inline, Style, Target, Dlc, Infobox
  src/diagnostics.rs    Diagnostics (counters)
  src/template.rs       parser for {{name|arg|k=v}}
  src/resolver.rs       Tables, Corrections, Resolver, key()
  src/inline.rs         wikitext of a line → Vec<Inline>
  src/blocks.rs         lines → Vec<Block> (lists, tables, headings, paragraphs)
  src/sections.rs       page → preamble + sections; title → SectionKind
  src/infobox.rs        infobox extraction and conversion to Infobox
  src/page.rs           PageKind, parse_page → Vec<(EntryKey, Entry)>
  src/raw.rs            reading dataset/raw/
  src/build.rs          build(&Raw, &Corrections) → Dataset
  src/dataset.rs        Dataset, Meta, entry(), embedded(), DatasetError
  tests/real.rs         pass over dataset/raw/ and fixed points
  tests/derived.rs      wiki.json == build(raw)
crates/wiki-snapshot/
  Cargo.toml
  src/main.rs           arguments, orchestration
  src/api.rs            URLs and response parsing (pure)
  src/http.rs           ureq, User-Agent, retry
  src/store.rs          file name, write-if-changed, cleanup
dataset/
  raw/index.json, raw/pages/<kind>/*.wikitext, raw/cargo/*.json
  corrections.json, wiki.json, ATTRIBUTION.md
crates/discovery/src/game.rs, lib.rs      last_updated
crates/ipc/src/wiki.rs, resources.rs, lib.rs, Cargo.toml
crates/app/src/lib.rs, error.rs, Cargo.toml
ui/src/lib/ipc/types.ts, wiki.ts, constants/commands.ts
ui/src/components/WikiBlocks.vue, ui/src/App.vue
package.json (script wiki:fetch, wiki:build)
docs/STATUS.md, docs/BACKLOG.md, CLAUDE.md, README.md, DESIGN-BRIEF.md, spec
```

---

### Task 1: `wiki` crate, model types and JSON shape

**Files:**
- Create: `crates/wiki/Cargo.toml`, `crates/wiki/src/lib.rs`, `crates/wiki/src/model.rs`, `crates/wiki/src/diagnostics.rs`

**Interfaces:**
- Produces: all the types in `model.rs` (below) and `Diagnostics`; used by every task that follows.

- [ ] **Step 1: Cargo.toml and lib.rs**

`crates/wiki/Cargo.toml`:
```toml
[package]
name = "wiki"
version = "0.1.0"
edition = "2021"
description = "Wiki-derived dataset: wikitext parser and typed tree"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

`crates/wiki/src/lib.rs`:
```rust
//! wiki — the wiki's text sections as a typed tree. A pure crate except for one spot:
//! `raw` reads `dataset/raw/` from disk. No network, no game archive.

mod diagnostics;
mod model;

pub use diagnostics::Diagnostics;
pub use model::{
    Block, Dlc, Entry, Infobox, Inline, ListItem, Section, SectionKind, Style, Target,
};
```

- [ ] **Step 2: write the JSON shape tests (they fail: the types don't exist)**

In `crates/wiki/src/model.rs`, at the bottom:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, to_value};

    #[test]
    fn inline_shapes() {
        assert_eq!(
            to_value(Inline::Text { text: "x".into(), style: Style::Bold }).unwrap(),
            json!({"kind":"text","text":"x","style":"bold"})
        );
        assert_eq!(
            to_value(Inline::Ref { target: Target::Item { id: 25 }, label: "Breakfast".into() }).unwrap(),
            json!({"kind":"ref","target":{"kind":"item","id":25},"label":"Breakfast"})
        );
        assert_eq!(
            to_value(Inline::Concept { page: "Shot Speed".into(), label: "Shot Speed".into() }).unwrap(),
            json!({"kind":"concept","page":"Shot Speed","label":"Shot Speed"})
        );
        assert_eq!(
            to_value(Inline::Edition { only: vec![Dlc::Repentance, Dlc::RepentancePlus], inline: vec![] }).unwrap(),
            json!({"kind":"edition","only":["repentance","repentancePlus"],"inline":[]})
        );
    }

    #[test]
    fn target_shapes() {
        assert_eq!(
            to_value(Target::Entity { id: 407, variant: 0, subtype: 0 }).unwrap(),
            json!({"kind":"entity","id":407,"variant":0,"subtype":0})
        );
        assert_eq!(to_value(Target::Challenge { number: 19 }).unwrap(), json!({"kind":"challenge","number":19}));
        assert_eq!(to_value(Target::Stage { name: "Depths".into() }).unwrap(), json!({"kind":"stage","name":"Depths"}));
    }

    #[test]
    fn block_shapes() {
        assert_eq!(
            to_value(Block::List { ordered: false, items: vec![ListItem { inline: vec![], children: vec![] }] }).unwrap(),
            json!({"kind":"list","ordered":false,"items":[{"inline":[],"children":[]}]})
        );
        assert_eq!(
            to_value(Block::Heading { level: 3, inline: vec![] }).unwrap(),
            json!({"kind":"heading","level":3,"inline":[]})
        );
        assert_eq!(
            to_value(Block::Table { header: vec![], rows: vec![] }).unwrap(),
            json!({"kind":"table","header":[],"rows":[]})
        );
    }

    #[test]
    fn infobox_and_section_shapes() {
        assert_eq!(to_value(Infobox::Item).unwrap(), json!({"kind":"item"}));
        assert_eq!(
            to_value(Infobox::Boss { base_hp: Some(6666), environment: vec![], pool: vec![], unlocked_by: None }).unwrap(),
            json!({"kind":"boss","baseHp":6666,"environment":[],"pool":[],"unlockedBy":null})
        );
        assert_eq!(to_value(SectionKind::ChampionVersions).unwrap(), json!("championVersions"));
        let e = Entry { title: "Hush".into(), revid: 1, infobox: Infobox::Item, sections: vec![] };
        assert_eq!(to_value(e).unwrap(), json!({"title":"Hush","revid":1,"infobox":{"kind":"item"},"sections":[]}));
    }

    #[test]
    fn roundtrip() {
        let e = Entry {
            title: "X".into(), revid: 2, infobox: Infobox::Trinket,
            sections: vec![Section { kind: SectionKind::Effects, blocks: vec![Block::Paragraph { inline: vec![
                Inline::Text { text: "a".into(), style: Style::Plain },
                Inline::Ref { target: Target::Trinket { id: 97 }, label: "Tonsil".into() },
            ]}]}],
        };
        let s = serde_json::to_string(&e).unwrap();
        assert_eq!(serde_json::from_str::<Entry>(&s).unwrap(), e);
    }
}
```

- [ ] **Step 3: run, confirm it fails**

Run: `cargo test -p wiki`
Expected: compile error (types not defined).

- [ ] **Step 4: write `model.rs`**

```rust
//! The typed tree. Everything crosses the IPC as-is: camelCase, enums with data tagged by
//! `kind`, fieldless enums as a bare string.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub title: String,
    pub revid: u64,
    pub infobox: Infobox,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    pub kind: SectionKind,
    pub blocks: Vec<Block>,
}

/// Fieldless: a bare string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SectionKind {
    Effects, Notes, Synergies, Interactions, Bugs, Behavior, ChampionVersions,
    DamageScaling, Strategies, Difficulty, Reward, Unlockable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Block {
    Paragraph { inline: Vec<Inline> },
    List { ordered: bool, items: Vec<ListItem> },
    Table { header: Vec<Vec<Inline>>, rows: Vec<Vec<Vec<Inline>>> },
    Heading { level: u8, inline: Vec<Inline> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    pub inline: Vec<Inline>,
    pub children: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Inline {
    Text { text: String, style: Style },
    Ref { target: Target, label: String },
    Concept { page: String, label: String },
    Edition { only: Vec<Dlc>, inline: Vec<Inline> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Style { Plain, Bold, Italic }

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Target {
    Item { id: u32 },
    Trinket { id: u32 },
    Character { id: u32 },
    Achievement { id: u32 },
    Challenge { number: u32 },
    Entity { id: u32, variant: u32, subtype: u32 },
    Transformation { id: u32 },
    Stage { name: String },
    Room { name: String },
    Pickup { name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Dlc { Rebirth, Afterbirth, AfterbirthPlus, Repentance, RepentancePlus }

impl Dlc {
    /// The `{{dlc|…}}` template codes: `n`, `a`, `a+`, `r`, `r+`. Anything else → `None`.
    pub fn from_code(code: &str) -> Option<Dlc> {
        match code.trim() {
            "n" => Some(Dlc::Rebirth),
            "a" => Some(Dlc::Afterbirth),
            "a+" => Some(Dlc::AfterbirthPlus),
            "r" => Some(Dlc::Repentance),
            "r+" => Some(Dlc::RepentancePlus),
            _ => None, // allowed: the input is an open-ended string from the wiki
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Infobox {
    Item,
    Trinket,
    Achievement { description: String, requirements: Vec<Inline>, unlocks: Option<Target> },
    Boss { base_hp: Option<u32>, environment: Vec<Inline>, pool: Vec<Inline>, unlocked_by: Option<Target> },
    Challenge {
        blindfolded: bool, has_shops: bool, has_treasure_rooms: bool,
        items: Vec<Inline>, trinkets: Vec<Inline>, pickups: Vec<Inline>, health: Vec<Inline>,
        curse: Vec<Inline>, goal: Vec<Inline>, unlocks: Option<Target>, unlocked_by: Option<Target>,
    },
    Character {
        health: Vec<Inline>, damage: String, range: String, speed: String, luck: String,
        shot_speed: String, pickups: Vec<Inline>, collectibles: Vec<Inline>, unlocked_by: Option<Target>,
    },
}
```

`crates/wiki/src/diagnostics.rs`:
```rust
//! Parser counters: they end up in the dataset's `meta`, so a rebuild tells whether the
//! parser has lost ground. Never an error: everything degrades and is counted.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    /// Unresolved `Ref`s, by template name (`i`, `c`, `e`, …).
    pub unresolved: BTreeMap<String, u32>,
    /// Templates listed neither among the links nor among the layout ones, by name.
    pub unknown_templates: BTreeMap<String, u32>,
    /// Discarded sections, by original title.
    pub discarded_sections: BTreeMap<String, u32>,
    /// Pages with an infobox but no usable `id`.
    pub pages_without_id: u32,
}

impl Diagnostics {
    pub fn unresolved(&mut self, template: &str) {
        *self.unresolved.entry(template.to_string()).or_default() += 1;
    }
    pub fn unknown_template(&mut self, name: &str) {
        *self.unknown_templates.entry(name.to_string()).or_default() += 1;
    }
    pub fn discarded_section(&mut self, title: &str) {
        *self.discarded_sections.entry(title.to_string()).or_default() += 1;
    }
    /// Adds the counters of another pass (a page) into this one (the snapshot).
    pub fn merge(&mut self, other: &Diagnostics) {
        for (k, v) in &other.unresolved { *self.unresolved.entry(k.clone()).or_default() += v; }
        for (k, v) in &other.unknown_templates { *self.unknown_templates.entry(k.clone()).or_default() += v; }
        for (k, v) in &other.discarded_sections { *self.discarded_sections.entry(k.clone()).or_default() += v; }
        self.pages_without_id += other.pages_without_id;
    }
}
```

- [ ] **Step 5: run the tests**

Run: `cargo test -p wiki` → 5 tests pass. `cargo fmt` and `cargo clippy -p wiki --all-targets -- -D warnings` clean.

- [ ] **Step 6: commit**

```bash
git add crates/wiki
git commit -m "wiki: crate with the tree types and the JSON shape pinned"
```

---

### Task 2: `{{…}}` template parser

**Files:**
- Create: `crates/wiki/src/template.rs`
- Modify: `crates/wiki/src/lib.rs` (add `mod template;` and `pub use template::{Template, parse_template_at};`)

**Interfaces:**
- Produces:
  ```rust
  pub struct Template { pub name: String, pub args: Vec<String>, pub named: BTreeMap<String, String> }
  /// `s[at..]` starts with `{{`. Returns the template and the byte index after `}}`.
  pub fn parse_template_at(s: &str, at: usize) -> Option<(Template, usize)>;
  ```
  `name` is cleaned up (trim, lowercase). Arguments keep the raw text (they can contain nested `{{…}}` and `[[…]]`, not split on internal `|`). `k=v` ends up in `named` with a trimmed + lowercased key and a trimmed value; the rest goes into `args`, trimmed.

- [ ] **Step 1: test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> Template { parse_template_at(s, 0).unwrap().0 }

    #[test]
    fn name_and_positional_args() {
        let x = t("{{i|Little Baggy|Baggy}}");
        assert_eq!(x.name, "i");
        assert_eq!(x.args, vec!["Little Baggy", "Baggy"]);
        assert!(x.named.is_empty());
    }

    #[test]
    fn named_args_and_case() {
        let x = t("{{ I | Little Baggy | text = Baggy }}");
        assert_eq!(x.name, "i");
        assert_eq!(x.args, vec!["Little Baggy"]);
        assert_eq!(x.named.get("text").map(String::as_str), Some("Baggy"));
    }

    #[test]
    fn nested_templates_and_links_are_not_split() {
        let x = t("{{dlc+|a+}} Entrance to {{s|The Void}} [[a|b]] {{dlc-}}");
        // parse_template_at only reads the first template
        assert_eq!(x.name, "dlc+");
        assert_eq!(x.args, vec!["a+"]);
        let (y, end) = parse_template_at("{{bug|{{i|A|x=y}} and [[p|q]]|dlc=r}} tail", 0).unwrap();
        assert_eq!(y.args, vec!["{{i|A|x=y}} and [[p|q]]"]);
        assert_eq!(y.named.get("dlc").map(String::as_str), Some("r"));
        assert_eq!(&"{{bug|{{i|A|x=y}} and [[p|q]]|dlc=r}} tail"[end..], " tail");
    }

    #[test]
    fn multiline_infobox() {
        let src = "{{infobox boss\n | dlc = a\n | id = 407\n | base hp = 6666\n}}\n'''Hush'''";
        let (x, end) = parse_template_at(src, 0).unwrap();
        assert_eq!(x.name, "infobox boss");
        assert_eq!(x.named.get("base hp").map(String::as_str), Some("6666"));
        assert_eq!(&src[end..], "\n'''Hush'''");
    }

    #[test]
    fn unterminated_returns_none() {
        assert!(parse_template_at("{{i|Breakfast", 0).is_none());
        assert!(parse_template_at("no template", 0).is_none());
    }

    #[test]
    fn parser_functions_keep_the_hash_name() {
        let x = t("{{#ev:youtube|dFwYucBWQ9k}}");
        assert_eq!(x.name, "#ev:youtube");
    }
}
```

- [ ] **Step 2: run, it fails** (`cargo test -p wiki template`)

- [ ] **Step 3: implementation**

```rust
//! `{{name|arg|k=v}}` with nesting: arguments stay raw, `|` inside nested `{{…}}` and
//! `[[…]]` doesn't split.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub name: String,
    pub args: Vec<String>,
    pub named: BTreeMap<String, String>,
}

pub fn parse_template_at(s: &str, at: usize) -> Option<(Template, usize)> {
    let b = s.as_bytes();
    if !s[at..].starts_with("{{") { return None; }
    let mut i = at + 2;
    let mut depth_t = 0usize; // nested {{ }}
    let mut depth_l = 0usize; // nested [[ ]]
    let mut parts: Vec<String> = Vec::new();
    let mut cur = String::new();
    while i < b.len() {
        let rest = &s[i..];
        if rest.starts_with("{{") { depth_t += 1; cur.push_str("{{"); i += 2; continue; }
        if rest.starts_with("}}") {
            if depth_t == 0 {
                parts.push(std::mem::take(&mut cur));
                return Some((assemble(parts), i + 2));
            }
            depth_t -= 1; cur.push_str("}}"); i += 2; continue;
        }
        if rest.starts_with("[[") { depth_l += 1; cur.push_str("[["); i += 2; continue; }
        if rest.starts_with("]]") { depth_l = depth_l.saturating_sub(1); cur.push_str("]]"); i += 2; continue; }
        if b[i] == b'|' && depth_t == 0 && depth_l == 0 {
            parts.push(std::mem::take(&mut cur)); i += 1; continue;
        }
        let ch = rest.chars().next()?;
        cur.push(ch);
        i += ch.len_utf8();
    }
    None
}

fn assemble(parts: Vec<String>) -> Template {
    let mut it = parts.into_iter();
    let name = it.next().unwrap_or_default().trim().to_lowercase();
    let mut args = Vec::new();
    let mut named = BTreeMap::new();
    for p in it {
        // `k=v` only if `k` doesn't contain `{{`/`[[`: an `=` inside a link isn't a name.
        match p.split_once('=') {
            Some((k, v)) if !k.contains("{{") && !k.contains("[[") && !k.trim().is_empty() => {
                named.insert(k.trim().to_lowercase(), v.trim().to_string());
            }
            _ => args.push(p.trim().to_string()),
        }
    }
    Template { name, args, named }
}
```

Note on the `nested_templates_and_links_are_not_split` test: the argument `{{i|A|x=y}} and [[p|q]]` contains `=` inside `{{…}}`; the `!k.contains("{{")` criterion keeps it positional because the part before `=` is `{{i|A|x`, which contains `{{`.

- [ ] **Step 4: green tests, fmt, clippy**

- [ ] **Step 5: commit** — `git commit -m "wiki: template parser with nesting"`

---

### Task 3: reference resolver

**Files:**
- Create: `crates/wiki/src/resolver.rs`
- Modify: `crates/wiki/src/lib.rs` (`mod resolver; pub use resolver::{Corrections, Resolution, Resolver, Row, Tables, key};`)

**Interfaces:**
- Produces:
  ```rust
  pub type Row = BTreeMap<String, String>;          // a cargoquery row, keys as they arrive ("_pageName", "id", "alias", "name", "number", "variant", "subtype", "type")
  #[derive(Default)] pub struct Tables { pub collectible: Vec<Row>, pub trinket: Vec<Row>, pub achievement: Vec<Row>, pub entity: Vec<Row>, pub challenge: Vec<Row>, pub transformation: Vec<Row>, pub pickup: Vec<Row> }
  #[derive(Default, Serialize, Deserialize)] pub struct Corrections { pub page_id: BTreeMap<String, BTreeMap<String, u32>> }   // "collectible" → { "Tonsil": 97 }
  pub enum Resolution { Target(Target), Unresolved, Ignore, Unknown }
  pub struct Resolver { … }
  impl Resolver {
      pub fn new(tables: &Tables, characters: &BTreeMap<String, u32>, corrections: &Corrections) -> Resolver;
      /// `template` already lowercase; `arg` the raw first argument.
      pub fn resolve(&self, template: &str, arg: &str) -> Resolution;
      /// Page title → target, for the infoboxes' `link`/`unlocks`/`unlocked by`.
      pub fn by_page_title(&self, title: &str) -> Option<Target>;
      pub fn achievement_by_name(&self, name: &str) -> Option<Target>;
      pub fn boss_key(&self, page_title: &str) -> Option<(u32, u32, u32)>;
  }
  pub fn key(s: &str) -> String;  // lowercase, compressed spaces, "&" → "and", trim
  pub fn is_layout_template(name: &str) -> bool;  // cit, nav, #ev:youtube, disambig msg, header characters, storage page, unlockable, reflist, column list, clear, main, hatnote, see also
  ```
  `Resolution::Ignore` for layout templates; `Unknown` for a template that is neither a link nor layout; `Unresolved` when the template is a link but the name isn't found.

- [ ] **Step 1: test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::Target;

    fn row(pairs: &[(&str, &str)]) -> Row {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    fn resolver() -> Resolver {
        let tables = Tables {
            collectible: vec![
                row(&[("_pageName", "Breakfast"), ("id", "25"), ("alias", "Breakfast")]),
                row(&[("_pageName", "Tonsil"), ("id", "474"), ("alias", "Tonsil")]),
                row(&[("_pageName", "Jacob & Esau"), ("id", "0"), ("alias", "Jacob and Esau")]),
            ],
            trinket: vec![row(&[("_pageName", "Swallowed Penny"), ("id", "1"), ("alias", "Swallowed Penny")])],
            achievement: vec![row(&[("_pageName", "Achievements/Rebirth 1"), ("id", "62"), ("name", "Epic Fetus"), ("alias", "Epic Fetus")])],
            entity: vec![
                row(&[("_pageName", "Mom"), ("id", "45"), ("variant", "0"), ("subtype", "0"), ("type", "boss"), ("alias", "Mom")]),
                row(&[("_pageName", "Angel"), ("id", "271"), ("variant", "0"), ("subtype", "0"), ("type", "mini-boss"), ("alias", "Uriel")]),
            ],
            challenge: vec![row(&[("_pageName", "The Family Man"), ("number", "19"), ("alias", "The Family Man")])],
            transformation: vec![row(&[("_pageName", "Beelzebub"), ("id", "1"), ("alias", "Beelzebub")])],
            pickup: vec![row(&[("_pageName", "Cards"), ("alias", "The Fool")])],
        };
        let mut chars = BTreeMap::new();
        chars.insert("Tainted Isaac".to_string(), 21);
        chars.insert("Jacob & Esau".to_string(), 19);
        let corrections: Corrections = serde_json::from_str(r#"{"pageId":{"collectible":{"Tonsil":97}}}"#).unwrap();
        Resolver::new(&tables, &chars, &corrections)
    }

    #[test]
    fn key_normalizes() {
        assert_eq!(key("  Jacob   &  Esau "), "jacob and esau");
        assert_eq!(key("Tainted ???"), "tainted ???");
    }

    #[test]
    fn items_by_alias_and_page_name_with_correction() {
        let r = resolver();
        assert_eq!(r.resolve("i", "Breakfast"), Resolution::Target(Target::Item { id: 25 }));
        assert_eq!(r.resolve("I", "breakfast"), Resolution::Target(Target::Item { id: 25 }));
        assert_eq!(r.resolve("i", "Tonsil"), Resolution::Target(Target::Item { id: 97 }));
        assert_eq!(r.resolve("i", "Jacob and Esau"), Resolution::Target(Target::Item { id: 0 }));
        assert_eq!(r.resolve("i", "Nope"), Resolution::Unresolved);
    }

    #[test]
    fn other_link_templates() {
        let r = resolver();
        assert_eq!(r.resolve("t", "Swallowed Penny"), Resolution::Target(Target::Trinket { id: 1 }));
        assert_eq!(r.resolve("a", "Epic Fetus"), Resolution::Target(Target::Achievement { id: 62 }));
        assert_eq!(r.resolve("chal", "The Family Man"), Resolution::Target(Target::Challenge { number: 19 }));
        assert_eq!(r.resolve("chal", "19"), Resolution::Target(Target::Challenge { number: 19 }));
        assert_eq!(r.resolve("e", "Mom"), Resolution::Target(Target::Entity { id: 45, variant: 0, subtype: 0 }));
        assert_eq!(r.resolve("e", "Uriel"), Resolution::Target(Target::Entity { id: 271, variant: 0, subtype: 0 }));
        assert_eq!(r.resolve("tf", "Beelzebub"), Resolution::Target(Target::Transformation { id: 1 }));
        assert_eq!(r.resolve("p", "The Fool"), Resolution::Target(Target::Pickup { name: "The Fool".into() }));
        assert_eq!(r.resolve("c", "Tainted Isaac"), Resolution::Target(Target::Character { id: 21 }));
        assert_eq!(r.resolve("c", "Jacob and Esau"), Resolution::Target(Target::Character { id: 19 }));
        assert_eq!(r.resolve("s", "Depths"), Resolution::Target(Target::Stage { name: "Depths".into() }));
        assert_eq!(r.resolve("floor", "Depths"), Resolution::Target(Target::Stage { name: "Depths".into() }));
        assert_eq!(r.resolve("r", "Shop"), Resolution::Target(Target::Room { name: "Shop".into() }));
        assert_eq!(r.resolve("room", "boss rush"), Resolution::Target(Target::Room { name: "boss rush".into() }));
    }

    #[test]
    fn layout_and_unknown() {
        let r = resolver();
        assert_eq!(r.resolve("cit", "p"), Resolution::Ignore);
        assert_eq!(r.resolve("nav", ""), Resolution::Ignore);
        assert_eq!(r.resolve("#ev:youtube", "x"), Resolution::Ignore);
        assert_eq!(r.resolve("m", "Donation Machine"), Resolution::Unknown);
    }

    #[test]
    fn page_titles_and_boss_keys() {
        let r = resolver();
        assert_eq!(r.by_page_title("Breakfast"), Some(Target::Item { id: 25 }));
        assert_eq!(r.by_page_title("Swallowed Penny"), Some(Target::Trinket { id: 1 }));
        assert_eq!(r.by_page_title("Jacob & Esau"), Some(Target::Character { id: 19 }));
        assert_eq!(r.by_page_title("The Family Man"), Some(Target::Challenge { number: 19 }));
        assert_eq!(r.by_page_title("Mom"), Some(Target::Entity { id: 45, variant: 0, subtype: 0 }));
        assert_eq!(r.by_page_title("Nope"), None);
        assert_eq!(r.achievement_by_name("Epic Fetus"), Some(Target::Achievement { id: 62 }));
        assert_eq!(r.boss_key("Mom"), Some((45, 0, 0)));
        assert_eq!(r.boss_key("Angel"), None); // mini-boss, not a boss
    }
}
```

Precedence in `by_page_title`: character, then item, trinket, challenge, entity. (A title like "Isaac" is both a character and the boss "Isaac (Boss)": the exact title tells them apart.)

- [ ] **Step 2: run, it fails**

- [ ] **Step 3: implementation**

```rust
//! From a name (alias or page title) to a `Target`, using the same Cargo tables the wiki's
//! templates use to resolve links. `corrections.json` wins over the table.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::Target;

pub type Row = BTreeMap<String, String>;

#[derive(Debug, Default, Clone)]
pub struct Tables {
    pub collectible: Vec<Row>, pub trinket: Vec<Row>, pub achievement: Vec<Row>, pub entity: Vec<Row>,
    pub challenge: Vec<Row>, pub transformation: Vec<Row>, pub pickup: Vec<Row>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Corrections {
    #[serde(default)]
    pub page_id: BTreeMap<String, BTreeMap<String, u32>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resolution { Target(Target), Unresolved, Ignore, Unknown }

#[derive(Debug, Default)]
pub struct Resolver {
    items: BTreeMap<String, u32>, items_by_title: BTreeMap<String, u32>,
    trinkets: BTreeMap<String, u32>, trinkets_by_title: BTreeMap<String, u32>,
    achievements: BTreeMap<String, u32>,
    challenges: BTreeMap<String, u32>, challenges_by_title: BTreeMap<String, u32>,
    entities: BTreeMap<String, (u32, u32, u32)>, bosses_by_title: BTreeMap<String, (u32, u32, u32)>,
    transformations: BTreeMap<String, u32>,
    pickups: BTreeMap<String, String>,
    characters: BTreeMap<String, u32>,
}

pub fn key(s: &str) -> String {
    let lower = s.trim().to_lowercase().replace('&', " and ");
    lower.split_whitespace().collect::<Vec<_>>().join(" ")
}

const LAYOUT: &[&str] = &[
    "cit", "nav", "#ev:youtube", "disambig msg", "header characters", "storage page", "unlockable",
    "reflist", "column list", "clear", "main", "hatnote", "see also", "distinguish", "distinguish visual",
    "toc", "__toc__",
];

pub fn is_layout_template(name: &str) -> bool { LAYOUT.contains(&name) }

fn num(row: &Row, field: &str) -> Option<u32> { row.get(field)?.trim().parse().ok() }

fn get<'a>(row: &'a Row, field: &str) -> Option<&'a str> {
    row.get(field).map(|s| s.as_str()).filter(|s| !s.trim().is_empty())
}

impl Resolver {
    pub fn new(tables: &Tables, characters: &BTreeMap<String, u32>, corrections: &Corrections) -> Resolver {
        let mut r = Resolver::default();
        let fix = |table: &str, title: &str, id: u32| -> u32 {
            corrections.page_id.get(table).and_then(|m| m.get(title)).copied().unwrap_or(id)
        };
        for row in &tables.collectible {
            let (Some(title), Some(id)) = (get(row, "_pageName"), num(row, "id")) else { continue };
            let id = fix("collectible", title, id);
            r.items_by_title.insert(key(title), id);
            r.items.entry(key(title)).or_insert(id);
            if let Some(a) = get(row, "alias") { r.items.insert(key(a), id); }
        }
        for row in &tables.trinket {
            let (Some(title), Some(id)) = (get(row, "_pageName"), num(row, "id")) else { continue };
            let id = fix("trinket", title, id);
            r.trinkets_by_title.insert(key(title), id);
            r.trinkets.entry(key(title)).or_insert(id);
            if let Some(a) = get(row, "alias") { r.trinkets.insert(key(a), id); }
        }
        for row in &tables.achievement {
            let Some(id) = num(row, "id") else { continue };
            if let Some(n) = get(row, "name") { r.achievements.insert(key(n), id); }
            if let Some(a) = get(row, "alias") { r.achievements.entry(key(a)).or_insert(id); }
        }
        for row in &tables.challenge {
            let Some(n) = num(row, "number") else { continue };
            if let Some(t) = get(row, "_pageName") { r.challenges_by_title.insert(key(t), n); r.challenges.entry(key(t)).or_insert(n); }
            if let Some(a) = get(row, "alias") { r.challenges.insert(key(a), n); }
            r.challenges.insert(n.to_string(), n);
        }
        for row in &tables.entity {
            let (Some(id), Some(v), Some(s)) = (num(row, "id"), num(row, "variant"), num(row, "subtype")) else { continue };
            let title = get(row, "_pageName");
            if let Some(t) = title { r.entities.entry(key(t)).or_insert((id, v, s)); }
            if let Some(a) = get(row, "alias") { r.entities.insert(key(a), (id, v, s)); }
            if get(row, "type") == Some("boss") {
                if let Some(t) = title { r.bosses_by_title.entry(key(t)).or_insert((id, v, s)); }
            }
        }
        for row in &tables.transformation {
            let Some(id) = num(row, "id") else { continue };
            if let Some(t) = get(row, "_pageName") { r.transformations.entry(key(t)).or_insert(id); }
            if let Some(a) = get(row, "alias") { r.transformations.insert(key(a), id); }
        }
        for row in &tables.pickup {
            if let Some(a) = get(row, "alias") { r.pickups.insert(key(a), a.to_string()); }
        }
        for (name, id) in characters { r.characters.insert(key(name), *id); }
        r
    }

    pub fn resolve(&self, template: &str, arg: &str) -> Resolution {
        let t = template.trim().to_lowercase();
        if is_layout_template(&t) { return Resolution::Ignore; }
        let k = key(arg);
        let found = match t.as_str() {
            "i" => self.items.get(&k).map(|id| Target::Item { id: *id }),
            "t" => self.trinkets.get(&k).map(|id| Target::Trinket { id: *id }),
            "a" | "achievement" => self.achievements.get(&k).map(|id| Target::Achievement { id: *id }),
            "chal" => self.challenges.get(&k).map(|n| Target::Challenge { number: *n }),
            "e" => self.entities.get(&k).map(|(id, variant, subtype)| Target::Entity { id: *id, variant: *variant, subtype: *subtype }),
            "tf" => self.transformations.get(&k).map(|id| Target::Transformation { id: *id }),
            "p" => self.pickups.get(&k).map(|n| Target::Pickup { name: n.clone() }),
            "c" => self.characters.get(&k).map(|id| Target::Character { id: *id }),
            "s" | "floor" => return Resolution::Target(Target::Stage { name: arg.trim().to_string() }),
            "r" | "room" => return Resolution::Target(Target::Room { name: arg.trim().to_string() }),
            _ => return Resolution::Unknown, // allowed: template name, open-ended string
        };
        match found { Some(t) => Resolution::Target(t), None => Resolution::Unresolved }
    }

    pub fn by_page_title(&self, title: &str) -> Option<Target> {
        let k = key(title);
        if let Some(id) = self.characters.get(&k) { return Some(Target::Character { id: *id }); }
        if let Some(id) = self.items_by_title.get(&k) { return Some(Target::Item { id: *id }); }
        if let Some(id) = self.trinkets_by_title.get(&k) { return Some(Target::Trinket { id: *id }); }
        if let Some(n) = self.challenges_by_title.get(&k) { return Some(Target::Challenge { number: *n }); }
        if let Some((id, variant, subtype)) = self.entities.get(&k) { return Some(Target::Entity { id: *id, variant: *variant, subtype: *subtype }); }
        None
    }

    pub fn achievement_by_name(&self, name: &str) -> Option<Target> {
        self.achievements.get(&key(name)).map(|id| Target::Achievement { id: *id })
    }

    pub fn boss_key(&self, page_title: &str) -> Option<(u32, u32, u32)> {
        self.bosses_by_title.get(&key(page_title)).copied()
    }
}
```

- [ ] **Step 4: green tests, fmt, clippy**
- [ ] **Step 5: commit** — `wiki: reference resolver over the Cargo tables, with the corrections`

---

### Task 4: inline parser

**Files:**
- Create: `crates/wiki/src/inline.rs`
- Modify: `crates/wiki/src/lib.rs` (`mod inline; pub use inline::parse_inline;`)

**Interfaces:**
- Consumes: `parse_template_at`, `Resolver::resolve`, `Diagnostics`, `Dlc::from_code`.
- Produces: `pub fn parse_inline(src: &str, r: &Resolver, d: &mut Diagnostics) -> Vec<Inline>`

Rules (from spec §2 and §3):
- Plain text → `Text { style }`; `'''` toggles Bold, `''` toggles Italic (Bold wins if both are open); adjacent `Text` with the same style merge; empty text isn't emitted.
- `[[Page|label]]` / `[[Page]]` / `[[Page#anchor|x]]` → `Concept { page, label }` (`page` without `#…`, label = label or page). `[[File:…]]`, `[[Category:…]]`, `[[:…]]` → discarded (File/Category) or Concept (`:`).
- `{{x|arg|label}}` or `text=`: `resolve(name, arg)`: `Target` → `Ref { target, label }` with label = `named["text"]` or `args[1]` if present and it doesn't contain `=`, otherwise `arg`; `Unresolved` → `Concept { page: arg, label }` + `d.unresolved(name)`; `Unknown` → `Text` with the first argument (or nothing if absent) + `d.unknown_template(name)`; `Ignore` → nothing.
- `{{dlc+|codes}}` opens an `Edition` closed by `{{dlc-}}` or at end of line; `{{dlc|codes}}` opens an `Edition` that closes at end of line; `{{dlcalt|x|code=y}}` → `Text x` inside an `Edition` of every code not listed? No: too much. `{{dlcalt|a|r=b}}` → two `Edition`s: `only=[]` with `a`... Simplify: `{{dlcalt|a|r=b}}` → `Text a` followed by `Edition { only: [r], inline: [Text " b"] }`. Codes: separated by `+`? No, `a+` is a code: try the whole code first, then split on `,` and spaces. An unknown code → `Edition` with an empty `only`.
- HTML: `<br>`, `<br/>`, `<br />` → `Text " "`; `<ref …>…</ref>` and `<ref … />` → discarded; `<!-- … -->` discarded; every other tag (`<u>`, `<span …>`, `</span>`, `<small>`, `<sup>`, `<nowiki>`) → removed, content kept. Entities: `&nbsp;`→space, `&times;`→`×`, `&amp;`→`&`, `&lt;`→`<`, `&gt;`→`>`, `&quot;`→`"`, `&#32;`→space, `&ndash;`→`–`, `&mdash;`→`—`; others stay as-is.
- `{{!}}` → `|`.

- [ ] **Step 1: test** (uses the `resolver()` from Task 3: move it into a `#[cfg(test)] pub(crate) mod fixtures` inside `resolver.rs` and call it `test_resolver()`)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::fixtures::test_resolver;
    use crate::{Diagnostics, Dlc, Inline, Style, Target};

    fn p(s: &str) -> (Vec<Inline>, Diagnostics) {
        let r = test_resolver(); let mut d = Diagnostics::default();
        (parse_inline(s, &r, &mut d), d)
    }
    fn text(t: &str, style: Style) -> Inline { Inline::Text { text: t.into(), style } }

    #[test]
    fn plain_bold_italic() {
        let (v, _) = p("a '''b''' c ''d''");
        assert_eq!(v, vec![text("a ", Style::Plain), text("b", Style::Bold), text(" c ", Style::Plain), text("d", Style::Italic)]);
    }

    #[test]
    fn refs_concepts_and_labels() {
        let (v, d) = p("{{i|Breakfast}}: +0.2 [[Shot Speed]], see {{i|Tonsil|the tonsil}} and [[Pills#Horse Pills|horse pill]] {{I|Nope}}");
        assert_eq!(v[0], Inline::Ref { target: Target::Item { id: 25 }, label: "Breakfast".into() });
        assert_eq!(v[1], text(": +0.2 ", Style::Plain));
        assert_eq!(v[2], Inline::Concept { page: "Shot Speed".into(), label: "Shot Speed".into() });
        assert_eq!(v[4], Inline::Ref { target: Target::Item { id: 97 }, label: "the tonsil".into() });
        assert_eq!(v[6], Inline::Concept { page: "Pills".into(), label: "horse pill".into() });
        assert_eq!(v[8], Inline::Concept { page: "Nope".into(), label: "Nope".into() });
        assert_eq!(d.unresolved.get("i"), Some(&1));
    }

    #[test]
    fn unknown_and_layout_templates() {
        let (v, d) = p("{{cit|p|r}}x {{m|Donation Machine}} y");
        assert_eq!(v, vec![text("x Donation Machine y", Style::Plain)]);
        assert_eq!(d.unknown_templates.get("m"), Some(&1));
        assert!(d.unresolved.is_empty());
    }

    #[test]
    fn editions() {
        let (v, _) = p("A {{dlc+|r}}only rep {{i|Breakfast}}{{dlc-}} B");
        assert_eq!(v, vec![
            text("A ", Style::Plain),
            Inline::Edition { only: vec![Dlc::Repentance], inline: vec![
                text("only rep ", Style::Plain),
                Inline::Ref { target: Target::Item { id: 25 }, label: "Breakfast".into() },
            ]},
            text(" B", Style::Plain),
        ]);
        let (v, _) = p("{{dlc|a+}} Isaac also starts with X");
        assert_eq!(v, vec![Inline::Edition { only: vec![Dlc::AfterbirthPlus], inline: vec![text(" Isaac also starts with X", Style::Plain)] }]);
        let (v, _) = p("{{dlcalt|17.75|r=4.5}}");
        assert_eq!(v, vec![text("17.75", Style::Plain), Inline::Edition { only: vec![Dlc::Repentance], inline: vec![text(" 4.5", Style::Plain)] }]);
    }

    #[test]
    fn html_and_entities() {
        let (v, _) = p("2&times;2 <u>Boss</u><br>next<ref>cite</ref> <span class=\"x\">in</span> done<!-- c -->");
        assert_eq!(v, vec![text("2×2 Boss next in done", Style::Plain)]);
        let (v, _) = p("[[File:Boss Hush.png|x]] [[Category:Modified]] keep");
        assert_eq!(v, vec![text(" keep", Style::Plain)]);
    }
}
```

- [ ] **Step 2: run, it fails**

- [ ] **Step 3: implementation**

```rust
//! From a line of wikitext to `Vec<Inline>`. Single-pass scan with a stack of open
//! `Edition`s; text accumulates and is emitted on a style or node change.

use crate::resolver::{Resolution, Resolver};
use crate::template::parse_template_at;
use crate::{Diagnostics, Dlc, Inline, Style};

struct Out {
    frames: Vec<(Vec<Dlc>, Vec<Inline>)>, // bottom = top level
    buf: String,
    bold: bool,
    italic: bool,
}

impl Out {
    fn new() -> Out { Out { frames: vec![(Vec::new(), Vec::new())], buf: String::new(), bold: false, italic: false } }
    fn style(&self) -> Style { if self.bold { Style::Bold } else if self.italic { Style::Italic } else { Style::Plain } }
    fn flush(&mut self) {
        if self.buf.is_empty() { return; }
        let style = self.style();
        let text = std::mem::take(&mut self.buf);
        let top = &mut self.frames.last_mut().expect("at least one frame").1;
        match top.last_mut() {
            Some(Inline::Text { text: t, style: s }) if *s == style => t.push_str(&text),
            _ => top.push(Inline::Text { text, style }),
        }
    }
    fn push(&mut self, node: Inline) { self.flush(); self.frames.last_mut().expect("frame").1.push(node); }
    fn open(&mut self, only: Vec<Dlc>) { self.flush(); self.frames.push((only, Vec::new())); }
    fn close(&mut self) {
        self.flush();
        if self.frames.len() > 1 {
            let (only, inline) = self.frames.pop().expect("frame");
            if !inline.is_empty() { self.frames.last_mut().expect("frame").1.push(Inline::Edition { only, inline }); }
        }
    }
    fn finish(mut self) -> Vec<Inline> { while self.frames.len() > 1 { self.close(); } self.flush(); self.frames.pop().map(|f| f.1).unwrap_or_default() }
}

fn dlc_codes(s: &str) -> Vec<Dlc> {
    if let Some(d) = Dlc::from_code(s) { return vec![d]; }
    s.split(|c: char| c == ',' || c.is_whitespace()).filter_map(Dlc::from_code).collect()
}

fn entity(name: &str) -> Option<&'static str> {
    Some(match name {
        "nbsp" | "#32" => " ", "times" => "×", "amp" => "&", "lt" => "<", "gt" => ">", "quot" => "\"",
        "ndash" => "–", "mdash" => "—",
        _ => return None, // allowed: HTML entity, open-ended set
    })
}

pub fn parse_inline(src: &str, r: &Resolver, d: &mut Diagnostics) -> Vec<Inline> {
    let mut out = Out::new();
    let mut i = 0;
    let b = src.as_bytes();
    while i < b.len() {
        let rest = &src[i..];
        if let Some(end) = rest.strip_prefix("<!--").and_then(|t| t.find("-->")) { i += 4 + end + 3; continue; }
        if rest.starts_with("'''") { out.flush(); out.bold = !out.bold; i += 3; continue; }
        if rest.starts_with("''") { out.flush(); out.italic = !out.italic; i += 2; continue; }
        if rest.starts_with("{{") {
            if let Some((t, end)) = parse_template_at(src, i) {
                i = end;
                let arg = t.args.first().cloned().unwrap_or_default();
                match t.name.as_str() {
                    "!" => out.buf.push('|'),
                    "dlc+" => out.open(dlc_codes(&arg)),
                    "dlc-" => out.close(),
                    "dlc" => out.open(dlc_codes(&arg)),
                    "dlcalt" => {
                        out.buf.push_str(&arg);
                        for (code, text) in &t.named {
                            out.open(dlc_codes(code)); out.buf.push(' '); out.buf.push_str(text); out.close();
                        }
                    }
                    name => match r.resolve(name, &arg) {
                        Resolution::Target(target) => {
                            let label = t.named.get("text").cloned()
                                .or_else(|| t.args.get(1).filter(|a| !a.contains('=') && !a.is_empty()).cloned())
                                .unwrap_or_else(|| arg.clone());
                            out.push(Inline::Ref { target, label });
                        }
                        Resolution::Unresolved => {
                            d.unresolved(name);
                            let label = t.named.get("text").cloned().unwrap_or_else(|| arg.clone());
                            out.push(Inline::Concept { page: arg.clone(), label });
                        }
                        Resolution::Unknown => { d.unknown_template(name); out.buf.push_str(&arg); }
                        Resolution::Ignore => {}
                    },
                }
                continue;
            }
            out.buf.push_str("{{"); i += 2; continue;
        }
        if rest.starts_with("[[") {
            if let Some(end) = rest.find("]]") {
                let inner = &rest[2..end];
                i += end + 2;
                let (page, label) = match inner.split_once('|') { Some((p, l)) => (p, l), None => (inner, inner) };
                let lower = page.to_ascii_lowercase();
                if lower.starts_with("file:") || lower.starts_with("category:") || lower.starts_with("image:") { continue; }
                let page = page.trim_start_matches(':');
                let page = page.split('#').next().unwrap_or(page).trim().to_string();
                let label = label.split('|').next().unwrap_or(label).trim().to_string();
                if page.is_empty() { out.buf.push_str(&label); continue; }
                out.push(Inline::Concept { page, label });
                continue;
            }
            out.buf.push_str("[["); i += 2; continue;
        }
        if rest.starts_with('<') {
            if let Some(end) = rest.find('>') {
                let tag = &rest[1..end];
                let name = tag.trim_start_matches('/').split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or("").to_ascii_lowercase();
                i += end + 1;
                if name == "br" { out.buf.push(' '); continue; }
                if name == "ref" && !tag.ends_with('/') && !tag.starts_with('/') {
                    if let Some(close) = src[i..].find("</ref>") { i += close + 6; }
                    continue;
                }
                continue; // any other tag: gone, content stays
            }
        }
        if rest.starts_with('&') {
            if let Some(end) = rest[1..].find(';').filter(|e| *e <= 8) {
                if let Some(rep) = entity(&rest[1..1 + end]) { out.buf.push_str(rep); i += end + 2; continue; }
            }
        }
        let ch = rest.chars().next().expect("not empty");
        out.buf.push(ch);
        i += ch.len_utf8();
    }
    out.finish()
}
```

- [ ] **Step 4: green tests, fmt, clippy** (if clippy complains about `expect` outside tests: these are internal invariants of the builder, not disk data; if you prefer, replace with `if let`.)
- [ ] **Step 5: commit** — `wiki: inline parser with styles, links, resolved references and editions`

---

### Task 5: block parser

**Files:**
- Create: `crates/wiki/src/blocks.rs`
- Modify: `crates/wiki/src/lib.rs` (`mod blocks; pub use blocks::parse_blocks;`)

**Interfaces:**
- Consumes: `parse_inline`.
- Produces: `pub fn parse_blocks(body: &str, r: &Resolver, d: &mut Diagnostics) -> Vec<Block>`

Rules:
- A `=== T ===` line (3 or 4 `=`) → `Heading { level, inline }`; level 2 doesn't appear here (sections are already split in Task 6).
- Lines starting with `*` or `#`: the marker prefix gives depth and type (`#` → ordered). Consecutive entries form a `List`; an entry at depth n+1 goes into the `children` of the last entry at depth n (as a `List`). An entry at depth 2 with no preceding depth-1 entry gets appended to an empty entry.
- `{|` … `|}`: table. `|-` separates rows; `!` cells (header) and `|`, with `!!`/`||` as separators on the same line; an attribute before the cell (`colspan="2"|`) is stripped: if the cell contains a `|` and the part before it contains `=`, the part after is kept. The first row with `!` cells is `header`; subsequent rows are `rows`; a row made only of `!` after the first is added to `rows` (pill table subtitles).
- `:` or `;` at the start of a line → paragraph (after stripping the prefix).
- Blank lines separate paragraphs; consecutive non-blank lines join with a space.
- Lines that are only `__TOC__`, `{{clear}}` or similar give empty inline: a paragraph with empty inline **is not emitted**.

- [ ] **Step 1: test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::fixtures::test_resolver;
    use crate::{Block, Diagnostics, Inline, ListItem, Style};

    fn p(s: &str) -> Vec<Block> { parse_blocks(s, &test_resolver(), &mut Diagnostics::default()) }
    fn t(s: &str) -> Vec<Inline> { vec![Inline::Text { text: s.into(), style: Style::Plain }] }

    #[test]
    fn paragraphs_join_lines_and_split_on_blank() {
        assert_eq!(p("a\nb\n\nc\n"), vec![Block::Paragraph { inline: t("a b") }, Block::Paragraph { inline: t("c") }]);
    }

    #[test]
    fn nested_list() {
        let v = p("* one\n** two\n** three\n* four\n# n1\n");
        assert_eq!(v, vec![
            Block::List { ordered: false, items: vec![
                ListItem { inline: t("one"), children: vec![Block::List { ordered: false, items: vec![
                    ListItem { inline: t("two"), children: vec![] }, ListItem { inline: t("three"), children: vec![] }] }] },
                ListItem { inline: t("four"), children: vec![] },
            ]},
            Block::List { ordered: true, items: vec![ListItem { inline: t("n1"), children: vec![] }] },
        ]);
    }

    #[test]
    fn table() {
        let src = "{| class=\"wikitable\"\n ! Pill !! Changes Into\n |-\n | Stat up pill\n | Stat down pill\n |-\n ! colspan=\"2\"| Neutral pills\n |-\n |colspan=\"2\"| ''I Found Pills''\n|}\n";
        let v = p(src);
        let Block::Table { header, rows } = &v[0] else { panic!("table") };
        assert_eq!(header, &vec![t("Pill"), t("Changes Into")]);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0], vec![t("Stat up pill"), t("Stat down pill")]);
        assert_eq!(rows[1], vec![t("Neutral pills")]);
        assert_eq!(rows[2], vec![vec![Inline::Text { text: "I Found Pills".into(), style: Style::Italic }]]);
    }

    #[test]
    fn headings_and_definition_lines() {
        let v = p("=== Phase 1 ===\ntext\n==== Phase 2-1 (100-80% HP) ====\n: indented\n");
        assert_eq!(v[0], Block::Heading { level: 3, inline: t("Phase 1") });
        assert_eq!(v[1], Block::Paragraph { inline: t("text") });
        assert_eq!(v[2], Block::Heading { level: 4, inline: t("Phase 2-1 (100-80% HP)") });
        assert_eq!(v[3], Block::Paragraph { inline: t("indented") });
    }

    #[test]
    fn empty_paragraphs_are_dropped() {
        assert_eq!(p("__TOC__\n{{cit|p}}\n"), vec![]);
    }
}
```

- [ ] **Step 2: run, it fails**

- [ ] **Step 3: implementation**

```rust
//! From wikitext lines to blocks. One pass over the lines with three states in flight:
//! paragraph, list, table.

use crate::inline::parse_inline;
use crate::resolver::Resolver;
use crate::{Block, Diagnostics, Inline, ListItem};

pub fn parse_blocks(body: &str, r: &Resolver, d: &mut Diagnostics) -> Vec<Block> {
    let mut out = Vec::new();
    let mut para: Vec<String> = Vec::new();
    let mut list: Vec<(usize, bool, String)> = Vec::new(); // (depth, ordered, text)
    let mut table: Option<Vec<String>> = None;

    let flush_para = |para: &mut Vec<String>, out: &mut Vec<Block>, d: &mut Diagnostics| {
        if para.is_empty() { return; }
        let inline = parse_inline(&para.join(" "), r, d);
        para.clear();
        if !inline.is_empty() { out.push(Block::Paragraph { inline }); }
    };
    let flush_list = |list: &mut Vec<(usize, bool, String)>, out: &mut Vec<Block>, d: &mut Diagnostics| {
        if list.is_empty() { return; }
        let items = std::mem::take(list);
        out.extend(build_lists(&items, 1, r, d));
    };

    for raw in body.lines() {
        let line = raw.trim_end();
        if let Some(rows) = table.as_mut() {
            if line.trim_start().starts_with("|}") { let rows = table.take().unwrap_or_default(); out.push(build_table(&rows, r, d)); }
            else { rows.push(line.to_string()); }
            continue;
        }
        let trimmed = line.trim_start();
        if trimmed.starts_with("{|") { flush_para(&mut para, &mut out, d); flush_list(&mut list, &mut out, d); table = Some(Vec::new()); continue; }
        if let Some((level, text)) = heading(trimmed) {
            flush_para(&mut para, &mut out, d); flush_list(&mut list, &mut out, d);
            out.push(Block::Heading { level, inline: parse_inline(text, r, d) });
            continue;
        }
        let markers = trimmed.chars().take_while(|c| *c == '*' || *c == '#').count();
        if markers > 0 {
            flush_para(&mut para, &mut out, d);
            let ordered = trimmed.chars().nth(markers - 1) == Some('#');
            list.push((markers, ordered, trimmed[markers..].trim().to_string()));
            continue;
        }
        flush_list(&mut list, &mut out, d);
        if trimmed.is_empty() { flush_para(&mut para, &mut out, d); continue; }
        let text = trimmed.strip_prefix(':').or_else(|| trimmed.strip_prefix(';')).map(str::trim).unwrap_or(trimmed);
        if text == "__TOC__" { continue; }
        para.push(text.to_string());
    }
    flush_para(&mut para, &mut out, d);
    flush_list(&mut list, &mut out, d);
    if let Some(rows) = table { out.push(build_table(&rows, r, d)); }
    out
}

fn heading(line: &str) -> Option<(u8, &str)> {
    for level in [4u8, 3] {
        let marks = "=".repeat(level as usize);
        if let Some(inner) = line.strip_prefix(marks.as_str()).and_then(|s| s.strip_suffix(marks.as_str())) {
            if !inner.starts_with('=') { return Some((level, inner.trim())); }
        }
    }
    None
}

/// Items at depth `depth` become a list; deeper items that follow an item end up in its
/// `children`. Multiple lists appear when `ordered` changes at the same level.
fn build_lists(items: &[(usize, bool, String)], depth: usize, r: &Resolver, d: &mut Diagnostics) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut cur: Option<(bool, Vec<ListItem>)> = None;
    let mut i = 0;
    while i < items.len() {
        let (dep, ordered, text) = &items[i];
        if *dep < depth { break; }
        if *dep > depth {
            // deeper entries with no parent: empty parent
            let start = i;
            while i < items.len() && items[i].0 > depth { i += 1; }
            let children = build_lists(&items[start..i], depth + 1, r, d);
            let item = ListItem { inline: Vec::new(), children };
            match cur.as_mut() { Some((_, v)) => v.push(item), None => cur = Some((*ordered, vec![item])) }
            continue;
        }
        let inline = parse_inline(text, r, d);
        i += 1;
        let start = i;
        while i < items.len() && items[i].0 > depth { i += 1; }
        let children = build_lists(&items[start..i], depth + 1, r, d);
        let item = ListItem { inline, children };
        match cur.as_mut() {
            Some((o, v)) if *o == *ordered => v.push(item),
            _ => {
                if let Some((o, v)) = cur.take() { blocks.push(Block::List { ordered: o, items: v }); }
                cur = Some((*ordered, vec![item]));
            }
        }
    }
    if let Some((o, v)) = cur { blocks.push(Block::List { ordered: o, items: v }); }
    blocks
}

fn build_table(lines: &[String], r: &Resolver, d: &mut Diagnostics) -> Block {
    let mut header: Vec<Vec<Inline>> = Vec::new();
    let mut rows: Vec<Vec<Vec<Inline>>> = Vec::new();
    let mut row: Vec<Vec<Inline>> = Vec::new();
    let mut row_is_header = false;
    let mut first = true;
    let mut close_row = |row: &mut Vec<Vec<Inline>>, row_is_header: bool, header: &mut Vec<Vec<Inline>>, rows: &mut Vec<Vec<Vec<Inline>>>, first: &mut bool| {
        if row.is_empty() { return; }
        let cells = std::mem::take(row);
        if row_is_header && *first { *header = cells; } else { rows.push(cells); }
        *first = false;
    };
    for line in lines {
        let l = line.trim();
        if l.starts_with("|-") { close_row(&mut row, row_is_header, &mut header, &mut rows, &mut first); row_is_header = false; continue; }
        if l.starts_with("|+") { continue; } // caption
        let (is_h, body) = if let Some(b) = l.strip_prefix('!') { (true, b) } else if let Some(b) = l.strip_prefix('|') { (false, b) } else { continue };
        if is_h { row_is_header = true; }
        let sep = if is_h { "!!" } else { "||" };
        for cell in body.split(sep) {
            let cell = strip_attributes(cell.trim());
            let inline = parse_inline(cell, r, d);
            row.push(inline);
        }
    }
    close_row(&mut row, row_is_header, &mut header, &mut rows, &mut first);
    Block::Table { header, rows }
}

/// `colspan="2"| text` → `text`. A `|` inside `[[…]]`/`{{…}}` is not a separator, and a
/// `|` with no `=` before it is not an attribute: the cell stays whole.
fn strip_attributes(cell: &str) -> &str {
    let mut depth = 0i32;
    for (i, ch) in cell.char_indices() {
        match ch {
            '[' | '{' => depth += 1,
            ']' | '}' => depth -= 1,
            '|' if depth == 0 => {
                let before = &cell[..i];
                if before.contains('=') { return cell[i + 1..].trim(); }
                return cell;
            }
            _ => {}
        }
    }
    cell
}
```

- [ ] **Step 4: green tests, fmt, clippy** (clippy may ask to shrink the closures that take too many arguments: turning `close_row` into a free `fn` is fine).
- [ ] **Step 5: commit** — `wiki: block parser, nested lists, tables and headings`

---

### Task 6: sections and infobox

**Files:**
- Create: `crates/wiki/src/sections.rs`, `crates/wiki/src/infobox.rs`
- Modify: `crates/wiki/src/lib.rs` (`mod sections; mod infobox; pub use sections::{split_page, section_kind, RawSection}; pub use infobox::{extract_infoboxes, infobox_from, InfoboxKind, RawInfobox};`)

**Interfaces:**
- Produces:
  ```rust
  pub struct RawSection { pub title: String, pub body: String }   // level 2 only; the body includes the subsections
  pub fn split_page(text: &str) -> (String, Vec<RawSection>);      // (preamble, sections)
  pub fn section_kind(title: &str) -> Option<SectionKind>;
  pub struct RawInfobox { pub name: String, pub params: BTreeMap<String, String> }
  pub fn extract_infoboxes(text: &str) -> Vec<RawInfobox>;         // every top-level `{{infobox …}}`
  pub enum InfoboxKind { Collectible, Trinket, Achievement, Boss, Challenge, Character }
  impl InfoboxKind { pub fn of(name: &str) -> Option<InfoboxKind> }  // "infobox passive collectible"/"infobox activated collectible" → Collectible, etc.
  pub fn infobox_from(kind: InfoboxKind, ib: &RawInfobox, r: &Resolver, d: &mut Diagnostics) -> Infobox;
  ```

`section_kind` (normalized title: `[[`/`]]` stripped, lowercase, spaces compressed):
`effects|effect|no effect → Effects`; `notes → Notes`; `synergies → Synergies`; `interactions|item interactions → Interactions`; `bugs → Bugs`; `behavior → Behavior`; `champion versions → ChampionVersions`; `damage scaling → DamageScaling`; `strategies|strategy|tips → Strategies`; `difficulty → Difficulty`; `reward → Reward`; `unlockable achievements|unlockable achievement|unlockable starting items → Unlockable`. Titles with a template (`== Champion Versions {{unlockable}} ==`) are normalized by stripping `{{…}}`.

`infobox_from`, per kind:
- `Collectible` → `Infobox::Item`; `Trinket` → `Infobox::Trinket`.
- `Achievement` → `description` = the `description` param (text, entities decoded with `parse_inline` then concatenating the `Text`s; or raw trimmed: use raw), `requirements` = `parse_inline(params["requirements"])`, `unlocks` = `r.by_page_title(params["link"])`.
- `Boss` → `base_hp` = `params["base hp"]` parsed as `u32` after stripping everything before the first digit (`"6666"`, `"250 (x2)"` → 250); `environment` = parse_inline(`environment`); `pool` = parse_inline(`pool`); `unlocked_by` = `r.achievement_by_name(params["unlocked by"])`.
- `Challenge` → `blindfolded`/`has shops`/`has treasure rooms`: `yes` → true, otherwise → false; `items` = parse_inline(`item`), `trinkets` (`trinket`), `pickups` (`pickup`), `health`, `curse`, `goal`; `unlocks` = `r.by_page_title(unlocks)` then `r.achievement_by_name(unlocks)`; `unlocked_by` = `r.achievement_by_name(unlocked by)`.
- `Character` → `health` = parse_inline(`health`), `damage`/`range`/`speed`/`luck`/`shot speed` (raw strings, `""` if absent), `pickups`, `collectibles` inline, `unlocked_by` = achievement by name.

- [ ] **Step 1: test**

```rust
// sections.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::SectionKind;

    #[test]
    fn splits_level_two_and_keeps_subsections_inside() {
        let src = "{{infobox boss\n | id = 1\n}}\n'''X''' intro\n\n== Behavior ==\n=== Phase 1 ===\na\n== Notes ==\nb\n";
        let (pre, secs) = split_page(src);
        assert!(pre.contains("intro"));
        assert_eq!(secs.len(), 2);
        assert_eq!(secs[0].title, "Behavior");
        assert_eq!(secs[0].body.trim(), "=== Phase 1 ===\na");
        assert_eq!(secs[1].title, "Notes");
    }

    #[test]
    fn section_kinds() {
        assert_eq!(section_kind("Effects"), Some(SectionKind::Effects));
        assert_eq!(section_kind("No Effect"), Some(SectionKind::Effects));
        assert_eq!(section_kind("Unlockable [[Achievement]]s"), Some(SectionKind::Unlockable));
        assert_eq!(section_kind("Champion Versions {{unlockable}}"), Some(SectionKind::ChampionVersions));
        assert_eq!(section_kind("In-game Footage"), None);
        assert_eq!(section_kind("Trivia"), None);
    }
}

// infobox.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::fixtures::test_resolver;
    use crate::{Diagnostics, Infobox, Inline, Target};

    #[test]
    fn extracts_many_infoboxes_from_a_storage_page() {
        let src = "{{storage page}}\n{{infobox achievement\n | name = Epic Fetus\n | link = Epic Fetus\n | description = Unlocked a new item.\n | requirements = Complete {{chal|The Family Man}}\n | id = 62\n}} {{infobox achievement\n | name = Cain\n | id = 2\n}}";
        let v = extract_infoboxes(src);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].name, "infobox achievement");
        assert_eq!(v[0].params.get("id").map(String::as_str), Some("62"));
        assert_eq!(v[1].params.get("name").map(String::as_str), Some("Cain"));
    }

    #[test]
    fn kinds() {
        assert_eq!(InfoboxKind::of("infobox passive collectible"), Some(InfoboxKind::Collectible));
        assert_eq!(InfoboxKind::of("infobox activated collectible"), Some(InfoboxKind::Collectible));
        assert_eq!(InfoboxKind::of("infobox trinket"), Some(InfoboxKind::Trinket));
        assert_eq!(InfoboxKind::of("infobox boss"), Some(InfoboxKind::Boss));
        assert_eq!(InfoboxKind::of("infobox"), None);
    }

    #[test]
    fn achievement_and_boss() {
        let r = test_resolver(); let mut d = Diagnostics::default();
        let ib = RawInfobox { name: "infobox achievement".into(), params: [("description", "Unlocked a new item."), ("requirements", "Complete {{chal|The Family Man}}"), ("link", "Breakfast")].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect() };
        let Infobox::Achievement { description, requirements, unlocks } = infobox_from(InfoboxKind::Achievement, &ib, &r, &mut d) else { panic!() };
        assert_eq!(description, "Unlocked a new item.");
        assert!(requirements.iter().any(|i| matches!(i, Inline::Ref { target: Target::Challenge { number: 19 }, .. })));
        assert_eq!(unlocks, Some(Target::Item { id: 25 }));

        let ib = RawInfobox { name: "infobox boss".into(), params: [("base hp", "250 (x2)"), ("unlocked by", "Epic Fetus")].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect() };
        let Infobox::Boss { base_hp, unlocked_by, .. } = infobox_from(InfoboxKind::Boss, &ib, &r, &mut d) else { panic!() };
        assert_eq!(base_hp, Some(250));
        assert_eq!(unlocked_by, Some(Target::Achievement { id: 62 }));
    }

    #[test]
    fn challenge_flags() {
        let r = test_resolver(); let mut d = Diagnostics::default();
        let ib = RawInfobox { name: "infobox challenge".into(), params: [("blindfolded", "yes"), ("has shops", "no"), ("unlocks", "Epic Fetus")].iter().map(|(k, v)| (k.to_string(), v.to_string())).collect() };
        let Infobox::Challenge { blindfolded, has_shops, has_treasure_rooms, unlocks, .. } = infobox_from(InfoboxKind::Challenge, &ib, &r, &mut d) else { panic!() };
        assert!(blindfolded); assert!(!has_shops); assert!(!has_treasure_rooms);
        assert_eq!(unlocks, Some(Target::Achievement { id: 62 }));
    }
}
```

- [ ] **Step 2: run, it fails**

- [ ] **Step 3: implementation**

`sections.rs`:
```rust
//! A page = preamble + level-2 sections. Subsections stay in the body.

use crate::SectionKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSection { pub title: String, pub body: String }

pub fn split_page(text: &str) -> (String, Vec<RawSection>) {
    let mut pre = String::new();
    let mut secs: Vec<RawSection> = Vec::new();
    for line in text.lines() {
        let t = line.trim();
        let is_l2 = t.starts_with("==") && !t.starts_with("===") && t.ends_with("==") && t.len() > 4;
        if is_l2 {
            let title = t.trim_matches('=').trim().to_string();
            secs.push(RawSection { title, body: String::new() });
            continue;
        }
        match secs.last_mut() {
            Some(s) => { s.body.push_str(line); s.body.push('\n'); }
            None => { pre.push_str(line); pre.push('\n'); }
        }
    }
    (pre, secs)
}

fn normalize_title(title: &str) -> String {
    let mut s = String::new();
    let mut skip = 0;
    let mut chars = title.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'{') { skip += 1; chars.next(); continue; }
        if c == '}' && chars.peek() == Some(&'}') { skip = skip.saturating_sub(1); chars.next(); continue; }
        if skip > 0 { continue; }
        if c == '[' || c == ']' { continue; }
        s.push(c);
    }
    s.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn section_kind(title: &str) -> Option<SectionKind> {
    Some(match normalize_title(title).as_str() {
        "effects" | "effect" | "no effect" => SectionKind::Effects,
        "notes" => SectionKind::Notes,
        "synergies" => SectionKind::Synergies,
        "interactions" | "item interactions" => SectionKind::Interactions,
        "bugs" => SectionKind::Bugs,
        "behavior" => SectionKind::Behavior,
        "champion versions" => SectionKind::ChampionVersions,
        "damage scaling" => SectionKind::DamageScaling,
        "strategies" | "strategy" | "tips" => SectionKind::Strategies,
        "difficulty" => SectionKind::Difficulty,
        "reward" => SectionKind::Reward,
        "unlockable achievements" | "unlockable achievement" | "unlockable starting items" => SectionKind::Unlockable,
        _ => return None, // allowed: free-form wiki title
    })
}
```

`infobox.rs`:
```rust
//! Infoboxes: extracted as top-level templates, converted into the `Infobox` of their kind.

use std::collections::BTreeMap;

use crate::inline::parse_inline;
use crate::resolver::Resolver;
use crate::template::parse_template_at;
use crate::{Diagnostics, Infobox, Inline};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawInfobox { pub name: String, pub params: BTreeMap<String, String> }

pub fn extract_infoboxes(text: &str) -> Vec<RawInfobox> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(pos) = text[i..].find("{{") {
        let at = i + pos;
        match parse_template_at(text, at) {
            Some((t, end)) => {
                if t.name.starts_with("infobox") {
                    out.push(RawInfobox { name: t.name, params: t.named });
                }
                i = end;
            }
            None => i = at + 2,
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoboxKind { Collectible, Trinket, Achievement, Boss, Challenge, Character }

impl InfoboxKind {
    pub fn of(name: &str) -> Option<InfoboxKind> {
        Some(match name {
            "infobox passive collectible" | "infobox activated collectible" | "infobox collectible" => InfoboxKind::Collectible,
            "infobox trinket" => InfoboxKind::Trinket,
            "infobox achievement" => InfoboxKind::Achievement,
            "infobox boss" => InfoboxKind::Boss,
            "infobox challenge" => InfoboxKind::Challenge,
            "infobox character" => InfoboxKind::Character,
            _ => return None, // allowed: template name, open-ended string
        })
    }
}

fn param<'a>(ib: &'a RawInfobox, name: &str) -> &'a str { ib.params.get(name).map(String::as_str).unwrap_or("") }
fn inline(ib: &RawInfobox, name: &str, r: &Resolver, d: &mut Diagnostics) -> Vec<Inline> { parse_inline(param(ib, name), r, d) }
fn yes(ib: &RawInfobox, name: &str) -> bool { param(ib, name).trim().eq_ignore_ascii_case("yes") }
fn leading_number(s: &str) -> Option<u32> { s.trim().chars().take_while(char::is_ascii_digit).collect::<String>().parse().ok() }

pub fn infobox_from(kind: InfoboxKind, ib: &RawInfobox, r: &Resolver, d: &mut Diagnostics) -> Infobox {
    match kind {
        InfoboxKind::Collectible => Infobox::Item,
        InfoboxKind::Trinket => Infobox::Trinket,
        InfoboxKind::Achievement => Infobox::Achievement {
            description: param(ib, "description").trim().to_string(),
            requirements: inline(ib, "requirements", r, d),
            unlocks: r.by_page_title(param(ib, "link")),
        },
        InfoboxKind::Boss => Infobox::Boss {
            base_hp: leading_number(param(ib, "base hp")),
            environment: inline(ib, "environment", r, d),
            pool: inline(ib, "pool", r, d),
            unlocked_by: r.achievement_by_name(param(ib, "unlocked by")),
        },
        InfoboxKind::Challenge => Infobox::Challenge {
            blindfolded: yes(ib, "blindfolded"), has_shops: yes(ib, "has shops"), has_treasure_rooms: yes(ib, "has treasure rooms"),
            items: inline(ib, "item", r, d), trinkets: inline(ib, "trinket", r, d), pickups: inline(ib, "pickup", r, d),
            health: inline(ib, "health", r, d), curse: inline(ib, "curse", r, d), goal: inline(ib, "goal", r, d),
            unlocks: r.by_page_title(param(ib, "unlocks")).or_else(|| r.achievement_by_name(param(ib, "unlocks"))),
            unlocked_by: r.achievement_by_name(param(ib, "unlocked by")),
        },
        InfoboxKind::Character => Infobox::Character {
            health: inline(ib, "health", r, d),
            damage: param(ib, "damage").trim().to_string(), range: param(ib, "range").trim().to_string(),
            speed: param(ib, "speed").trim().to_string(), luck: param(ib, "luck").trim().to_string(),
            shot_speed: param(ib, "shot speed").trim().to_string(),
            pickups: inline(ib, "pickups", r, d), collectibles: inline(ib, "collectibles", r, d),
            unlocked_by: r.achievement_by_name(param(ib, "unlocked by")),
        },
    }
}
```

- [ ] **Step 4: green tests, fmt, clippy**
- [ ] **Step 5: commit** — `wiki: normalized sections and infobox per type`

---

### Task 7: page, raw, build and Dataset

**Files:**
- Create: `crates/wiki/src/page.rs`, `crates/wiki/src/raw.rs`, `crates/wiki/src/build.rs`, `crates/wiki/src/dataset.rs`
- Modify: `crates/wiki/src/lib.rs`

**Interfaces:**
- Produces:
  ```rust
  // page.rs
  #[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)] #[serde(rename_all = "lowercase")]
  pub enum PageKind { Collectible, Trinket, Achievement, Boss, Challenge, Character }
  impl PageKind { pub const ALL: [PageKind; 6]; pub fn dir(self) -> &'static str /* "collectible" … */; pub fn template(self) -> &'static str /* "Template:Infobox collectible" … */ }
  pub enum EntryKey { Item(u32), Trinket(u32), Achievement(u32), Boss(u32, u32, u32), Challenge(u32), Character(u32) }
  pub fn parse_page(kind: PageKind, title: &str, revid: u64, text: &str, r: &Resolver, d: &mut Diagnostics) -> Vec<(EntryKey, Entry)>;
  // raw.rs
  #[derive(Serialize, Deserialize)] pub struct IndexEntry { pub kind: PageKind, pub pageid: u64, pub revid: u64, pub timestamp: String }
  pub struct RawPage { pub title: String, pub index: IndexEntry, pub text: String }
  pub struct Raw { pub pages: Vec<RawPage>, pub tables: Tables, pub versions: Vec<Row> }
  pub fn page_file_name(title: &str) -> String;   // spaces → '_', characters ?:*"<>|/\% → %XX
  impl Raw { pub fn load(dir: &Path) -> Result<Raw, RawError>; }
  pub enum RawError { Missing(PathBuf), Unreadable(PathBuf, String), BadIndex(String) }
  // build.rs
  pub fn build(raw: &Raw, corrections: &Corrections) -> Dataset;
  // dataset.rs
  pub struct Meta { schema_version: u32, snapshot_at: String, max_revid: u64, last_known_patch: Option<Patch>, source: Source, counts: Counts, diagnostics: Diagnostics }
  pub struct Patch { number: String, date: String }
  pub struct Source { name: String, url: String, license: String }
  pub struct Counts { items, trinkets, achievements, bosses, challenges, characters: u32 }
  pub struct Dataset { pub meta: Meta, pub items: BTreeMap<u32, Entry>, pub trinkets: BTreeMap<u32, Entry>, pub achievements: BTreeMap<u32, Entry>, pub bosses: BTreeMap<String, Entry>, pub challenges: BTreeMap<u32, Entry>, pub characters: BTreeMap<u32, Entry> }
  pub const SCHEMA_VERSION: u32 = 1;
  impl Dataset { pub fn entry(&self, t: &Target) -> Option<&Entry>; pub fn embedded() -> Result<&'static Dataset, &'static DatasetError>; pub fn from_json(s: &str) -> Result<Dataset, DatasetError>; pub fn to_json(&self) -> String /* pretty */; pub fn boss_key(id: u32, variant: u32, subtype: u32) -> String }
  pub enum DatasetError { SchemaMismatch { found: u32, expected: u32 }, Malformed { reason: String } }
  ```

`parse_page`:
- `extract_infoboxes(text)`; for every infobox whose `InfoboxKind::of(name)` matches the page's `kind` (Collectible/Trinket/Boss/Challenge/Character: the first one; Achievement: all of them):
  - key: Collectible/Trinket/Character → `params["id"]` parsed as u32; Achievement → `params["id"]`; Challenge → `params["number"]`; Boss → `r.boss_key(title)` or `(params["id"], 0, 0)`. If missing → `d.pages_without_id += 1`, skip.
  - `Entry`'s `title`: the page title; for achievement pages, `params["name"]`.
  - sections: `split_page(text)`, for each `section_kind(title)` → `Section { kind, blocks: parse_blocks(body) }` (sections with empty `blocks` are still kept); `None` → `d.discarded_section(title)`. Achievement pages get no sections.
  - The preamble is discarded (it's the "X is a passive item…" sentence already covered by `catalog`).
- The `Resolver` is built in `build` from `raw.tables` and the characters (title → id map from the infoboxes of `character` pages: `parse` with `extract_infoboxes` + `params["id"]`), with `corrections`.

`Raw::load(dir)`: reads `index.json` (`BTreeMap<String, IndexEntry>`), for each entry reads `pages/<kind.dir()>/<page_file_name(title)>.wikitext` (missing → `RawError::Missing`), `cargo/<table>.json` for the seven tables of `Tables` plus `version` (a missing file → `Tables` with that vector empty, **not** an error: the wiki may not have a table; but a missing `collectible` → `Missing`). Every table JSON file is a `Vec<Row>`.

`build`: builds the resolver, iterates pages ordered by (kind, title), fills the maps (duplicate key → first one wins, the second counts as `pages_without_id`? No: counted in a separate counter? Keep it simple: first one wins, no counter), `meta` with `snapshot_at` = max `timestamp`, `max_revid`, `last_known_patch` = the `versions` row with the highest `date` (`number`, `date`), constant `source`, `counts`, summed `diagnostics`.

`Dataset::embedded()`:
```rust
static EMBEDDED: OnceLock<Result<Dataset, DatasetError>> = OnceLock::new();
pub fn embedded() -> Result<&'static Dataset, &'static DatasetError> {
    EMBEDDED.get_or_init(|| Dataset::from_json(include_str!("../../../dataset/wiki.json"))).as_ref()
}
```
`from_json`: first reads only `{"meta":{"schemaVersion":N}}` (partial struct) and compares against `SCHEMA_VERSION` → `SchemaMismatch`; then the whole thing → `Malformed { reason }`.

**Careful:** until `dataset/wiki.json` exists (Task 9), `include_str!` won't compile. In this task create a minimal valid `dataset/wiki.json`: `{"meta":{"schemaVersion":1,"snapshotAt":"","maxRevid":0,"lastKnownPatch":null,"source":{"name":"","url":"","license":""},"counts":{"items":0,"trinkets":0,"achievements":0,"bosses":0,"challenges":0,"characters":0},"diagnostics":{"unresolved":{},"unknownTemplates":{},"discardedSections":{},"pagesWithoutId":0}},"items":{},"trinkets":{},"achievements":{},"bosses":{},"challenges":{},"characters":{}}` and `dataset/corrections.json` = `{"pageId":{}}`.

- [ ] **Step 1: test**

```rust
// page.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::fixtures::test_resolver;
    use crate::{Block, Diagnostics, SectionKind};

    #[test]
    fn collectible_page_yields_one_entry_with_kept_sections() {
        let src = "{{infobox passive collectible\n | id = 25\n | quote = x\n}}\n{{cit|p|r}}\n\n== Effects ==\n* a\n== Trivia ==\n* t\n== Notes ==\nn\n";
        let mut d = Diagnostics::default();
        let v = parse_page(PageKind::Collectible, "Breakfast", 7, src, &test_resolver(), &mut d);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].0, EntryKey::Item(25));
        let e = &v[0].1;
        assert_eq!(e.title, "Breakfast"); assert_eq!(e.revid, 7);
        assert_eq!(e.sections.iter().map(|s| s.kind).collect::<Vec<_>>(), vec![SectionKind::Effects, SectionKind::Notes]);
        assert!(matches!(e.sections[0].blocks[0], Block::List { .. }));
        assert_eq!(d.discarded_sections.get("Trivia"), Some(&1));
    }

    #[test]
    fn achievement_storage_page_yields_many() {
        let src = "{{storage page}}\n{{infobox achievement\n | name = Epic Fetus\n | link = Breakfast\n | id = 62\n}} {{infobox achievement\n | name = Cain\n | id = 2\n}} {{infobox achievement\n | name = NoId\n}}";
        let mut d = Diagnostics::default();
        let v = parse_page(PageKind::Achievement, "Achievements/Rebirth 1", 1, src, &test_resolver(), &mut d);
        assert_eq!(v.iter().map(|(k, _)| k.clone()).collect::<Vec<_>>(), vec![EntryKey::Achievement(62), EntryKey::Achievement(2)]);
        assert_eq!(v[0].1.title, "Epic Fetus");
        assert_eq!(d.pages_without_id, 1);
    }

    #[test]
    fn boss_key_from_entity_table_or_infobox() {
        let src = "{{infobox boss\n | id = 45\n | base hp = 1\n}}\n== Behavior ==\nx\n";
        let v = parse_page(PageKind::Boss, "Mom", 1, src, &test_resolver(), &mut Diagnostics::default());
        assert_eq!(v[0].0, EntryKey::Boss(45, 0, 0));
        let v = parse_page(PageKind::Boss, "Unknown Boss", 1, "{{infobox boss\n | id = 999\n}}", &test_resolver(), &mut Diagnostics::default());
        assert_eq!(v[0].0, EntryKey::Boss(999, 0, 0));
    }
}

// raw.rs
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn file_names() {
        assert_eq!(page_file_name("False PHD"), "False_PHD");
        assert_eq!(page_file_name("??? (Boss)"), "%3F%3F%3F_(Boss)");
        assert_eq!(page_file_name("Mom's Knife"), "Mom's_Knife");
        assert_eq!(page_file_name("Achievements/Rebirth 1"), "Achievements%2FRebirth_1");
        assert_eq!(page_file_name("100% Fun"), "100%25_Fun");
    }
    #[test]
    fn load_reads_index_pages_and_tables() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        std::fs::create_dir_all(p.join("pages/collectible")).unwrap();
        std::fs::create_dir_all(p.join("cargo")).unwrap();
        std::fs::write(p.join("index.json"), r#"{"Breakfast":{"kind":"collectible","pageid":1,"revid":7,"timestamp":"2026-01-01T00:00:00Z"}}"#).unwrap();
        std::fs::write(p.join("pages/collectible/Breakfast.wikitext"), "{{infobox passive collectible|id=25}}").unwrap();
        std::fs::write(p.join("cargo/collectible.json"), r#"[{"_pageName":"Breakfast","id":"25","alias":"Breakfast"}]"#).unwrap();
        let raw = Raw::load(p).unwrap();
        assert_eq!(raw.pages.len(), 1);
        assert_eq!(raw.pages[0].title, "Breakfast");
        assert_eq!(raw.tables.collectible.len(), 1);
        assert!(raw.tables.trinket.is_empty());
        std::fs::remove_file(p.join("pages/collectible/Breakfast.wikitext")).unwrap();
        assert!(matches!(Raw::load(p), Err(RawError::Missing(_))));
    }
}

// build.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::raw::{IndexEntry, RawPage};
    use crate::{PageKind, Target};

    fn raw() -> Raw {
        let page = |kind, title: &str, ts: &str, revid, text: &str| RawPage { title: title.into(), index: IndexEntry { kind, pageid: 1, revid, timestamp: ts.into() }, text: text.into() };
        let row = |pairs: &[(&str, &str)]| pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<Row>();
        Raw {
            pages: vec![
                page(PageKind::Collectible, "Breakfast", "2026-02-01T00:00:00Z", 9, "{{infobox passive collectible|id=25}}\n== Effects ==\n* with {{c|Cain}} and {{i|Nope}}\n"),
                page(PageKind::Character, "Cain", "2026-01-01T00:00:00Z", 3, "{{infobox character|id=2}}\n== Notes ==\nn\n"),
            ],
            tables: Tables { collectible: vec![row(&[("_pageName", "Breakfast"), ("id", "25"), ("alias", "Breakfast")])], ..Tables::default() },
            versions: vec![row(&[("number", "v1.9.7.16"), ("date", "2026-04-11")]), row(&[("number", "v1.9.7.17"), ("date", "2026-04-20")])],
        }
    }

    #[test]
    fn builds_maps_meta_and_resolves_characters_from_their_pages() {
        let ds = build(&raw(), &Corrections::default());
        assert_eq!(ds.meta.schema_version, SCHEMA_VERSION);
        assert_eq!(ds.meta.snapshot_at, "2026-02-01T00:00:00Z");
        assert_eq!(ds.meta.max_revid, 9);
        assert_eq!(ds.meta.last_known_patch.as_ref().map(|p| p.number.as_str()), Some("v1.9.7.17"));
        assert_eq!(ds.meta.counts.items, 1); assert_eq!(ds.meta.counts.characters, 1);
        assert_eq!(ds.meta.diagnostics.unresolved.get("i"), Some(&1));
        let e = ds.entry(&Target::Item { id: 25 }).unwrap();
        let s = serde_json::to_string(e).unwrap();
        assert!(s.contains(r#""kind":"character","id":2"#));
        assert!(ds.entry(&Target::Character { id: 2 }).is_some());
        assert!(ds.entry(&Target::Stage { name: "x".into() }).is_none());
    }

    #[test]
    fn json_roundtrip_and_schema_check() {
        let ds = build(&raw(), &Corrections::default());
        let s = ds.to_json();
        assert_eq!(Dataset::from_json(&s).unwrap(), ds);
        let bad = s.replacen(r#""schemaVersion": 1"#, r#""schemaVersion": 99"#, 1);
        assert!(matches!(Dataset::from_json(&bad), Err(DatasetError::SchemaMismatch { found: 99, .. })));
        assert!(matches!(Dataset::from_json("{"), Err(DatasetError::Malformed { .. })));
    }
}
```

Add `tempfile = "3"` to `[dev-dependencies]` of `crates/wiki/Cargo.toml`.

- [ ] **Step 2: run, it fails**

- [ ] **Step 3: implementation** — follow the interfaces above; structure of `build`:

```rust
pub fn build(raw: &Raw, corrections: &Corrections) -> Dataset {
    let mut characters = BTreeMap::new();
    for p in raw.pages.iter().filter(|p| p.index.kind == PageKind::Character) {
        for ib in extract_infoboxes(&p.text) {
            if InfoboxKind::of(&ib.name) == Some(InfoboxKind::Character) {
                if let Some(id) = ib.params.get("id").and_then(|s| s.trim().parse::<u32>().ok()) {
                    characters.insert(p.title.clone(), id);
                    if let Some(alias) = ib.params.get("name") { characters.entry(alias.trim().to_string()).or_insert(id); }
                }
                break;
            }
        }
    }
    let r = Resolver::new(&raw.tables, &characters, corrections);
    let mut ds = Dataset::empty();
    let mut diagnostics = Diagnostics::default();
    let mut pages: Vec<&RawPage> = raw.pages.iter().collect();
    pages.sort_by(|a, b| (a.index.kind, &a.title).cmp(&(b.index.kind, &b.title)));
    for p in pages {
        let mut d = Diagnostics::default();
        for (key, entry) in parse_page(p.index.kind, &p.title, p.index.revid, &p.text, &r, &mut d) {
            match key {
                EntryKey::Item(id) => { ds.items.entry(id).or_insert(entry); }
                EntryKey::Trinket(id) => { ds.trinkets.entry(id).or_insert(entry); }
                EntryKey::Achievement(id) => { ds.achievements.entry(id).or_insert(entry); }
                EntryKey::Boss(id, v, s) => { ds.bosses.entry(Dataset::boss_key(id, v, s)).or_insert(entry); }
                EntryKey::Challenge(n) => { ds.challenges.entry(n).or_insert(entry); }
                EntryKey::Character(id) => { ds.characters.entry(id).or_insert(entry); }
            }
        }
        diagnostics.merge(&d);
        if p.index.timestamp > ds.meta.snapshot_at { ds.meta.snapshot_at = p.index.timestamp.clone(); }
        ds.meta.max_revid = ds.meta.max_revid.max(p.index.revid);
    }
    ds.meta.last_known_patch = raw.versions.iter()
        .filter_map(|v| Some((v.get("date")?.clone(), v.get("number")?.clone())))
        .max().map(|(date, number)| Patch { number, date });
    ds.meta.source = Source { name: "The Binding of Isaac: Rebirth Wiki".into(), url: "https://bindingofisaacrebirth.wiki.gg".into(), license: "CC BY-SA 4.0".into() };
    ds.meta.counts = Counts { items: ds.items.len() as u32, trinkets: ds.trinkets.len() as u32, achievements: ds.achievements.len() as u32, bosses: ds.bosses.len() as u32, challenges: ds.challenges.len() as u32, characters: ds.characters.len() as u32 };
    ds.meta.diagnostics = diagnostics;
    ds
}
```

`Dataset::entry`:
```rust
pub fn entry(&self, t: &Target) -> Option<&Entry> {
    match t {
        Target::Item { id } => self.items.get(id),
        Target::Trinket { id } => self.trinkets.get(id),
        Target::Achievement { id } => self.achievements.get(id),
        Target::Challenge { number } => self.challenges.get(number),
        Target::Character { id } => self.characters.get(id),
        Target::Entity { id, variant, subtype } => self.bosses.get(&Self::boss_key(*id, *variant, *subtype)),
        Target::Transformation { .. } | Target::Stage { .. } | Target::Room { .. } | Target::Pickup { .. } => None,
    }
}
```

Final `lib.rs` for the crate:
```rust
mod blocks; mod build; mod dataset; mod diagnostics; mod infobox; mod inline; mod model; mod page; mod raw; mod resolver; mod sections; mod template;

pub use blocks::parse_blocks;
pub use build::build;
pub use dataset::{Counts, Dataset, DatasetError, Meta, Patch, Source, SCHEMA_VERSION};
pub use diagnostics::Diagnostics;
pub use infobox::{extract_infoboxes, infobox_from, InfoboxKind, RawInfobox};
pub use inline::parse_inline;
pub use model::{Block, Dlc, Entry, Infobox, Inline, ListItem, Section, SectionKind, Style, Target};
pub use page::{parse_page, EntryKey, PageKind};
pub use raw::{page_file_name, IndexEntry, Raw, RawError, RawPage};
pub use resolver::{is_layout_template, key, Corrections, Resolution, Resolver, Row, Tables};
pub use sections::{section_kind, split_page, RawSection};
pub use template::{parse_template_at, Template};
```

- [ ] **Step 4: green tests, fmt, clippy**
- [ ] **Step 5: commit** — `wiki: pages, reading raw/, dataset build and the embedded Dataset` (include the minimal `dataset/wiki.json` and `dataset/corrections.json`)

---

### Task 8: `wiki-snapshot` tool and first snapshot

**Files:**
- Create: `crates/wiki-snapshot/Cargo.toml`, `src/main.rs`, `src/api.rs`, `src/http.rs`, `src/store.rs`
- Modify: `package.json` (root): scripts `"wiki:fetch": "cargo run -q -p wiki-snapshot -- fetch"`, `"wiki:build": "cargo run -q -p wiki-snapshot -- build"`

**Interfaces:**
- Consumes: `wiki::{PageKind, page_file_name, IndexEntry, Raw, Corrections, build, Dataset}`.
- Produces: `dataset/raw/` populated and committed.

`Cargo.toml`: dependencies `wiki`, `ureq = "3"`, `serde`, `serde_json`. Check the current version of `ureq` with `cargo search ureq`; the API used: `ureq::get(url).header("User-Agent", UA).call()?.body_mut().read_to_string()` (ureq 3) — adjust to whatever's published.

`api.rs` (pure):
```rust
pub const HOST: &str = "https://bindingofisaacrebirth.wiki.gg";
pub fn pages_url(template: &str, cont: &BTreeMap<String, String>) -> String;
// action=query&generator=embeddedin&geititle=<template>&geinamespace=0&geilimit=50&prop=revisions&rvprop=content|ids|timestamp&rvslots=main&format=json&formatversion=2&maxlag=5 + cont (url-encoded)
pub struct FetchedPage { pub title: String, pub pageid: u64, pub revid: u64, pub timestamp: String, pub text: String }
pub fn parse_pages(json: &str) -> Result<(Vec<FetchedPage>, Option<BTreeMap<String, String>>), String>;
pub fn cargo_url(table: &str, fields: &str, offset: usize) -> String;
// action=cargoquery&tables=<t>&fields=<f>&limit=500&offset=<o>&format=json&maxlag=5
pub fn parse_cargo(json: &str) -> Result<Vec<Row>, String>;   // every `title` object → Row
pub const TABLES: &[(&str, &str)] = &[
    ("collectible", "_pageName,id,dlc,alias,is_activated,unlocked_by"),
    ("trinket", "_pageName,id,dlc,alias,unlocked_by"),
    ("achievement", "_pageName,id,name,alias,dlc,description,requirements,notes"),
    ("entity", "_pageName,id,variant,subtype,type,alias,dlc,unlocked_by"),
    ("challenge", "_pageName,number,alias,dlc,unlocked_by,unlocks"),
    ("player", "_pageName,id,alias,dlc,parent"),
    ("stage", "_pageName,alias,chapter,dlc"),
    ("transformation", "_pageName,id,alias,dlc"),
    ("pickup", "_pageName,id,alias,type,dlc"),
    ("version", "_pageName,number,name,date,dlc"),
];
pub fn url_encode(s: &str) -> String;   // percent-encoding of everything except alphanumerics, '-', '_', '.', '~'
```

`store.rs` (pure filesystem, tested with `tempfile`):
```rust
/// Writes only if the content changes. Returns true if it wrote.
pub fn write_if_changed(path: &Path, content: &[u8]) -> std::io::Result<bool>;
/// Deletes files in `dir` not present in `keep`; returns the deleted names.
pub fn prune(dir: &Path, keep: &BTreeSet<String>) -> std::io::Result<Vec<String>>;
```

`http.rs`: `pub fn get(url: &str) -> Result<String, String>` with `User-Agent: IsaacDome-snapshot/<CARGO_PKG_VERSION> (+https://github.com/DomenghiniStefano/isaac-dome)`, 30 s timeout, one retry after 5 s on transport error or 429/5xx status, then a 250 ms pause before returning.

`main.rs`:
- `fetch [--out <dir>]` (default `dataset/raw`, relative to the workspace root: use `env!("CARGO_MANIFEST_DIR")/../../dataset/raw`): for each `PageKind::ALL`, page through `pages_url(kind.template(), cont)` while there's a `continue`; write `pages/<kind.dir()>/<page_file_name(title)>.wikitext` with `write_if_changed`, collect the index; then `prune` per folder; for each table in `TABLES` download in pages of 500 until a response has fewer than 500 rows; write `cargo/<table>.json` (`serde_json::to_string_pretty` of the rows, sorted by `_pageName` then `id`); finally `index.json` (sorted BTreeMap, pretty). Print for each type "N pages, M written, K deleted" and for each table "N rows".
- `build [--raw <dir>] [--out <file>]`: `Raw::load` → `build(&raw, &corrections)` (corrections from `dataset/corrections.json`, absent = default) → `write_if_changed(out, ds.to_json())`; print a readable `meta`: counts, `snapshotAt`, patch, and the three diagnostic maps sorted by descending value (first 20 entries).
- Unknown arguments → usage on stderr, exit 2. Network/IO errors → message on stderr, exit 1.

- [ ] **Step 1: `api.rs` and `store.rs` tests**

```rust
// api.rs
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn urls() {
        let u = pages_url("Template:Infobox boss", &BTreeMap::new());
        assert!(u.starts_with("https://bindingofisaacrebirth.wiki.gg/api.php?action=query&generator=embeddedin&geititle=Template%3AInfobox%20boss"));
        assert!(u.contains("&geilimit=50&") && u.contains("rvslots=main") && u.contains("maxlag=5"));
        let mut c = BTreeMap::new(); c.insert("geicontinue".into(), "1|2".into()); c.insert("continue".into(), "geicontinue||".into());
        assert!(pages_url("T", &c).ends_with("&continue=geicontinue%7C%7C&geicontinue=1%7C2"));
        assert_eq!(cargo_url("collectible", "_pageName,id", 500), "https://bindingofisaacrebirth.wiki.gg/api.php?action=cargoquery&tables=collectible&fields=_pageName%2Cid&limit=500&offset=500&format=json&maxlag=5");
    }
    #[test]
    fn parse_pages_response() {
        let j = r#"{"continue":{"geicontinue":"0|Hush","continue":"geicontinue||"},"query":{"pages":[{"pageid":11419,"title":"False PHD","revisions":[{"revid":267225,"timestamp":"2026-07-30T08:21:54Z","slots":{"main":{"content":"{{infobox}}"}}}]}]}}"#;
        let (pages, cont) = parse_pages(j).unwrap();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].title, "False PHD"); assert_eq!(pages[0].revid, 267225); assert_eq!(pages[0].text, "{{infobox}}");
        assert_eq!(cont.unwrap().get("geicontinue").map(String::as_str), Some("0|Hush"));
        let (_, cont) = parse_pages(r#"{"batchcomplete":true,"query":{"pages":[]}}"#).unwrap();
        assert!(cont.is_none());
        assert!(parse_pages(r#"{"error":{"code":"maxlag","info":"x"}}"#).is_err());
    }
    #[test]
    fn parse_cargo_response() {
        let rows = parse_cargo(r#"{"cargoquery":[{"title":{"_pageName":"False PHD","id":"654","is activated":"0"}}]}"#).unwrap();
        assert_eq!(rows[0].get("id").map(String::as_str), Some("654"));
        assert_eq!(rows[0].get("is activated").map(String::as_str), Some("0"));
    }
}
// store.rs
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn write_only_when_changed_and_prune() {
        let d = tempfile::tempdir().unwrap();
        let f = d.path().join("a.txt");
        assert!(write_if_changed(&f, b"x").unwrap());
        assert!(!write_if_changed(&f, b"x").unwrap());
        assert!(write_if_changed(&f, b"y").unwrap());
        std::fs::write(d.path().join("b.txt"), "z").unwrap();
        let removed = prune(d.path(), &["a.txt".to_string()].into_iter().collect()).unwrap();
        assert_eq!(removed, vec!["b.txt"]);
        assert!(f.exists());
    }
}
```

- [ ] **Step 2: run, it fails; Step 3: implement; Step 4: green tests, fmt, clippy**

- [ ] **Step 5: run the real snapshot**

Run: `pnpm wiki:fetch` (network, ~2 minutes). Expected: six types with counts close to 723, 188, 9 (achievement pages), 103, 45, 30; ten tables. Check `git status`: only `dataset/raw/`. Open `dataset/raw/pages/collectible/False_PHD.wikitext` and check that it starts with `{{infobox passive collectible`.

- [ ] **Step 6: two commits**

```bash
git add crates/wiki-snapshot package.json
git commit -m "wiki-snapshot: tool that downloads pages and tables and builds the dataset"
git add dataset/raw
git commit -m "dataset: first raw snapshot of the wiki"
```

---

### Task 9: derived dataset, corrections, attribution, real-data tests

**Files:**
- Modify: `dataset/wiki.json` (regenerated), `dataset/corrections.json`
- Create: `dataset/ATTRIBUTION.md`, `crates/wiki/tests/real.rs`, `crates/wiki/tests/derived.rs`

- [ ] **Step 1: corrections and attribution**

`dataset/corrections.json`:
```json
{
  "pageId": {
    "collectible": { "Tonsil": 97 }
  }
}
```
(Check first: `grep -n "| id" dataset/raw/pages/collectible/Tonsil.wikitext` should say 474; if the wiki has since fixed it, leave the map empty and note it in the commit.)

`dataset/ATTRIBUTION.md`:
```markdown
# Wiki dataset attribution

`wiki.json` derives from the text of **The Binding of Isaac: Rebirth Wiki**
(https://bindingofisaacrebirth.wiki.gg), published under a **Creative Commons
Attribution-ShareAlike 4.0** license (https://creativecommons.org/licenses/by-sa/4.0/).
The derived dataset is distributed under the same license. The snapshot date and the
highest revision are in the file's `meta` field; for each entry, `revid` identifies the
page revision. Wiki images are not included.
```

- [ ] **Step 2: regenerate**

Run: `pnpm wiki:build`. Read the printed meta. Expected: `items` ≈ 719, `trinkets` 188, `achievements` ≈ 641, `bosses` ≈ 103, `challenges` 45, `characters` ≈ 30–40. If `unknownTemplates` shows frequent templates (>100) that are links (`m`, `machine`, `bc`, `ip`…), **don't** extend the resolver here: note them in the commit message; that's review territory.

- [ ] **Step 3: real-data tests** (`crates/wiki/tests/real.rs`)

```rust
//! Pass over the real snapshot in `dataset/raw/`: it's in the repo, so no skip.
use std::path::PathBuf;
use wiki::{build, Block, Corrections, Dataset, Inline, Raw, SectionKind, Target};

fn dataset() -> Dataset {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset");
    let raw = Raw::load(&root.join("raw")).expect("dataset/raw/ in the repo");
    let corrections: Corrections = std::fs::read_to_string(root.join("corrections.json")).ok()
        .and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
    build(&raw, &corrections)
}

fn refs_to_items(inline: &[Inline]) -> usize {
    inline.iter().map(|i| match i {
        Inline::Ref { target: Target::Item { .. }, .. } => 1,
        Inline::Edition { inline, .. } => refs_to_items(inline),
        Inline::Ref { .. } | Inline::Text { .. } | Inline::Concept { .. } => 0,
    }).sum()
}

#[test]
fn counts_match_the_index() {
    let ds = dataset();
    let c = &ds.meta.counts;
    assert!(c.items >= 715 && c.items <= 735, "items {}", c.items);
    assert_eq!(c.trinkets, 188);
    assert!(c.achievements >= 630, "achievements {}", c.achievements);
    assert!(c.bosses >= 100, "bosses {}", c.bosses);
    assert_eq!(c.challenges, 45);
    assert!(c.characters >= 30, "characters {}", c.characters);
    assert!(ds.meta.max_revid > 0 && !ds.meta.snapshot_at.is_empty());
    assert!(ds.meta.last_known_patch.is_some());
}

#[test]
fn tonsil_is_corrected() {
    let ds = dataset();
    assert_eq!(ds.entry(&Target::Item { id: 97 }).map(|e| e.title.as_str()), Some("Tonsil"));
}

#[test]
fn binge_eater_notes_reference_eight_food_items() {
    let ds = dataset();
    let e = ds.entry(&Target::Item { id: 664 }).expect("Binge Eater");
    let notes = e.sections.iter().find(|s| s.kind == SectionKind::Notes).expect("Notes");
    let Block::List { items, .. } = &notes.blocks[0] else { panic!("the first note is a list") };
    let Block::List { items: foods, .. } = &items[0].children[0] else { panic!("nested list of foods") };
    assert_eq!(foods.len(), 8);
    assert_eq!(foods.iter().map(|f| refs_to_items(&f.inline)).sum::<usize>(), 8);
}

#[test]
fn false_phd_has_the_pill_table() {
    let ds = dataset();
    let e = ds.entry(&Target::Item { id: 654 }).expect("False PHD");
    let effects = e.sections.iter().find(|s| s.kind == SectionKind::Effects).expect("Effects");
    let table = effects.blocks.iter().find_map(|b| match b { Block::Table { header, rows } => Some((header, rows)), Block::Paragraph { .. } | Block::List { .. } | Block::Heading { .. } => None }).expect("table");
    assert_eq!(table.0.len(), 2);
    assert_eq!(table.1.len(), 19, "rows: 16 pill pairs + Neutral title + 3 neutral = 20 minus the header row? Check the wikitext and pin the real number.");
}

#[test]
fn hush_has_phase_headings() {
    let ds = dataset();
    let hush = ds.bosses.values().find(|e| e.title == "Hush").expect("Hush");
    let behavior = hush.sections.iter().find(|s| s.kind == SectionKind::Behavior).expect("Behavior");
    let levels: Vec<u8> = behavior.blocks.iter().filter_map(|b| match b { Block::Heading { level, .. } => Some(*level), Block::Paragraph { .. } | Block::List { .. } | Block::Table { .. } => None }).collect();
    assert!(levels.contains(&3) && levels.contains(&4));
}

#[test]
fn achievement_62_unlocks_epic_fetus_by_beating_challenge_19() {
    let ds = dataset();
    let e = ds.entry(&Target::Achievement { id: 62 }).expect("achievement 62");
    assert_eq!(e.title, "Epic Fetus");
    let wiki::Infobox::Achievement { requirements, unlocks, .. } = &e.infobox else { panic!("infobox achievement") };
    assert!(requirements.iter().any(|i| matches!(i, Inline::Ref { target: Target::Challenge { number: 19 }, .. })));
    assert_eq!(*unlocks, Some(Target::Item { id: 168 }));
}

#[test]
fn diagnostics_are_bounded() {
    let ds = dataset();
    let d = &ds.meta.diagnostics;
    let unresolved: u32 = d.unresolved.values().sum();
    assert!(unresolved < 400, "unresolved {unresolved}: {:?}", d.unresolved);
    assert!(d.pages_without_id < 10, "without id {}", d.pages_without_id);
}
```

On the False PHD table test: count the rows in the wikitext (`grep -c "^ |-" dataset/raw/pages/collectible/False_PHD.wikitext` and read the structure) and pin the number the parser spec predicts (every `|-` opens a row; the first `!` row is the header). The expected number comes from the file, not from the test's output.

`crates/wiki/tests/derived.rs`:
```rust
use std::path::PathBuf;
use wiki::{build, Corrections, Raw};

#[test]
fn wiki_json_is_the_build_of_raw() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset");
    let raw = Raw::load(&root.join("raw")).expect("dataset/raw/");
    let corrections: Corrections = serde_json::from_str(&std::fs::read_to_string(root.join("corrections.json")).expect("corrections.json")).expect("json");
    let expected = build(&raw, &corrections).to_json();
    let actual = std::fs::read_to_string(root.join("wiki.json")).expect("wiki.json");
    assert!(expected == actual, "dataset/wiki.json does not match build(raw/): rerun `pnpm wiki:build` and commit");
}
```

- [ ] **Step 4: run** `cargo test -p wiki` — all green; `Dataset::embedded()` now embeds the real file: add a test `embedded_loads()` in `dataset.rs` that does `Dataset::embedded().expect("valid embedded dataset")` and checks `meta.counts.trinkets == 188`.

- [ ] **Step 5: commit** — `dataset: derivato, correzioni, attribuzione e test sui dati reali`

---

### Task 10: `discovery` reads `LastUpdated`

**Files:**
- Modify: `crates/discovery/src/game.rs` (`Manifest`, `parse_manifest`, the point where `GameInstall` is built), `crates/discovery/src/lib.rs` (`GameInstall.updated_unix: Option<u64>`)
- Check the existing tests that build `GameInstall` or `Manifest` and update the literals.

- [ ] **Step 1: test** in `game.rs`:
```rust
#[test]
fn manifest_reads_last_updated() {
    let m = parse_manifest("\"AppState\"\n{\n\t\"installdir\"\t\t\"The Binding of Isaac Rebirth\"\n\t\"LastUpdated\"\t\t\"1757000000\"\n}\n").unwrap();
    assert_eq!(m.last_updated, Some(1757000000));
    let m = parse_manifest("\"AppState\"\n{\n\t\"installdir\"\t\t\"X\"\n}\n").unwrap();
    assert_eq!(m.last_updated, None);
}
```
- [ ] **Step 2: it fails; Step 3: implement** (`app_state.get("LastUpdated")…parse::<u64>().ok()`), propagate to `GameInstall { updated_unix: manifest.last_updated, .. }`.
- [ ] **Step 4: `cargo test -p discovery -p ipc -p app` green** (the `GameInstall` constructors in `ipc`'s tests need updating with `updated_unix: None`).
- [ ] **Step 5: commit** — `discovery: the game's update date from the appmanifest`

---

### Task 11: `ipc::wiki`, `ExtractionReport.wiki`, `wiki_entry` command

**Files:**
- Create: `crates/ipc/src/wiki.rs`
- Modify: `crates/ipc/Cargo.toml` (`wiki = { path = "../wiki" }`), `crates/ipc/src/lib.rs`, `crates/ipc/src/resources.rs` (field `wiki: WikiInfo` and a parameter on `extraction_report`), `crates/app/Cargo.toml` (`wiki`), `crates/app/src/error.rs` (`WikiUnavailable`), `crates/app/src/lib.rs` (command + handler + `extraction_report`)

**Interfaces:**
```rust
// ipc::wiki
pub use wiki::{Block, Dlc, Entry, Infobox, Inline, ListItem, Section, SectionKind, Style, Target};
#[derive(Serialize)] #[serde(rename_all = "camelCase")] pub struct PatchView { pub number: String, pub date: String }
#[derive(Serialize)] #[serde(rename_all = "camelCase")] pub struct WikiCounts { pub items: u32, pub trinkets: u32, pub achievements: u32, pub bosses: u32, pub challenges: u32, pub characters: u32 }
#[derive(Serialize)] #[serde(rename_all = "camelCase")] pub enum WikiMissingReason { SchemaMismatch, Malformed }   // bare string
#[derive(Serialize)] #[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum WikiInfo {
    Loaded { snapshot_at: String, last_known_patch: Option<PatchView>, counts: WikiCounts, unresolved: u32, unknown_templates: u32, game_newer_than_snapshot: Option<bool> },
    Missing { reason: WikiMissingReason },
}
pub fn wiki_info(dataset: Result<&Dataset, &DatasetError>, game_updated_unix: Option<u64>) -> WikiInfo;
pub fn rfc3339_to_unix(s: &str) -> Option<u64>;   // "2026-09-05T14:20:41Z" → seconds; only the format with Z
```

- [ ] **Step 1: test** in `wiki.rs`:
```rust
#[test]
fn rfc3339() {
    assert_eq!(rfc3339_to_unix("1970-01-01T00:00:00Z"), Some(0));
    assert_eq!(rfc3339_to_unix("2026-09-05T14:20:41Z"), Some(1788618041));
    assert_eq!(rfc3339_to_unix("2026-09-05"), None);
}
#[test]
fn info_shapes() {
    let missing = wiki_info(Err(&DatasetError::Malformed { reason: "x".into() }), None);
    assert_eq!(serde_json::to_value(&missing).unwrap(), serde_json::json!({"kind":"missing","reason":"malformed"}));
    let ds = Dataset::embedded().expect("embedded dataset");
    let loaded = wiki_info(Ok(ds), Some(0));
    let v = serde_json::to_value(&loaded).unwrap();
    assert_eq!(v["kind"], "loaded");
    assert_eq!(v["gameNewerThanSnapshot"], false);
    assert_eq!(v["counts"]["trinkets"], 188);
    let v = serde_json::to_value(wiki_info(Ok(ds), None)).unwrap();
    assert_eq!(v["gameNewerThanSnapshot"], serde_json::Value::Null);
    let v = serde_json::to_value(wiki_info(Ok(ds), Some(u64::MAX / 2))).unwrap();
    assert_eq!(v["gameNewerThanSnapshot"], true);
}
```
(The value 1788618041 needs checking against an independent calculation, e.g. `date -u -d "2026-09-05T14:20:41Z" +%s` in Git Bash, before pinning it.)

- [ ] **Step 2: it fails; Step 3: implement.** `rfc3339_to_unix`: days since 1970 with the "days from civil" algorithm (Howard Hinnant), no crate. `wiki_info`: `Loaded` with `game_newer_than_snapshot = game_updated_unix.zip(rfc3339_to_unix(&meta.snapshot_at)).map(|(g, s)| g > s)`.

`resources.rs`: `pub wiki: WikiInfo` in `ExtractionReport`; `extraction_report(archives, catalog, sprites, wiki: WikiInfo)`; update the existing tests, passing `wiki_info(Err(&DatasetError::Malformed{..}), None)` or building a `WikiInfo::Missing`.

`app`:
```rust
#[tauri::command]
fn wiki_entry(target: ipc::Target) -> Result<Option<ipc::Entry>, IpcError> {
    let ds = wiki::Dataset::embedded().map_err(|_| IpcError::WikiUnavailable)?;
    Ok(ds.entry(&target).cloned())
}
```
In `extraction_report`: `let wiki = ipc::wiki_info(wiki::Dataset::embedded(), d.game.as_ref().and_then(|g| g.updated_unix));` and pass it along; also on the "game absent" branch. Register `wiki_entry` in `generate_handler!`. `IpcError::WikiUnavailable` documented as: "the embedded dataset failed to load: `wiki_entry` can't answer; `ExtractionReport.wiki` says why".

`Target` needs `Deserialize` (already in `model.rs`).

- [ ] **Step 4: `cargo test --workspace`, fmt, clippy green**
- [ ] **Step 5: commit** — `ipc: WikiInfo nel report e comando wiki_entry` (and `app-shell:` if you'd rather split it into two commits)

---

### Task 12: frontend, type mirror and verification panel

**Files:**
- Modify: `ui/src/lib/ipc/types.ts`, `ui/src/lib/constants/commands.ts`, `ui/src/App.vue`
- Create: `ui/src/lib/ipc/wiki.ts`, `ui/src/components/WikiBlocks.vue`

- [ ] **Step 1: types** (in `types.ts`, add):
```ts
// --- wiki -----------------------------------------------------------------
export const SectionKind = {
  Effects: 'effects', Notes: 'notes', Synergies: 'synergies', Interactions: 'interactions',
  Bugs: 'bugs', Behavior: 'behavior', ChampionVersions: 'championVersions',
  DamageScaling: 'damageScaling', Strategies: 'strategies', Difficulty: 'difficulty',
  Reward: 'reward', Unlockable: 'unlockable',
} as const
export type SectionKind = (typeof SectionKind)[keyof typeof SectionKind]

export const Style = { Plain: 'plain', Bold: 'bold', Italic: 'italic' } as const
export type Style = (typeof Style)[keyof typeof Style]

export const Dlc = {
  Rebirth: 'rebirth', Afterbirth: 'afterbirth', AfterbirthPlus: 'afterbirthPlus',
  Repentance: 'repentance', RepentancePlus: 'repentancePlus',
} as const
export type Dlc = (typeof Dlc)[keyof typeof Dlc]

export type Target =
  | { kind: 'item'; id: number } | { kind: 'trinket'; id: number } | { kind: 'character'; id: number }
  | { kind: 'achievement'; id: number } | { kind: 'challenge'; number: number }
  | { kind: 'entity'; id: number; variant: number; subtype: number }
  | { kind: 'transformation'; id: number } | { kind: 'stage'; name: string }
  | { kind: 'room'; name: string } | { kind: 'pickup'; name: string }

export type Inline =
  | { kind: 'text'; text: string; style: Style }
  | { kind: 'ref'; target: Target; label: string }
  | { kind: 'concept'; page: string; label: string }
  | { kind: 'edition'; only: Dlc[]; inline: Inline[] }

export interface ListItem { inline: Inline[]; children: Block[] }
export type Block =
  | { kind: 'paragraph'; inline: Inline[] }
  | { kind: 'list'; ordered: boolean; items: ListItem[] }
  | { kind: 'table'; header: Inline[][]; rows: Inline[][][] }
  | { kind: 'heading'; level: number; inline: Inline[] }

export type Infobox =
  | { kind: 'item' } | { kind: 'trinket' }
  | { kind: 'achievement'; description: string; requirements: Inline[]; unlocks: Target | null }
  | { kind: 'boss'; baseHp: number | null; environment: Inline[]; pool: Inline[]; unlockedBy: Target | null }
  | { kind: 'challenge'; blindfolded: boolean; hasShops: boolean; hasTreasureRooms: boolean; items: Inline[]; trinkets: Inline[]; pickups: Inline[]; health: Inline[]; curse: Inline[]; goal: Inline[]; unlocks: Target | null; unlockedBy: Target | null }
  | { kind: 'character'; health: Inline[]; damage: string; range: string; speed: string; luck: string; shotSpeed: string; pickups: Inline[]; collectibles: Inline[]; unlockedBy: Target | null }

export interface Section { kind: SectionKind; blocks: Block[] }
export interface Entry { title: string; revid: number; infobox: Infobox; sections: Section[] }

export type WikiMissingReason = 'schemaMismatch' | 'malformed'
export type WikiInfo =
  | { kind: 'loaded'; snapshotAt: string; lastKnownPatch: { number: string; date: string } | null;
      counts: { items: number; trinkets: number; achievements: number; bosses: number; challenges: number; characters: number };
      unresolved: number; unknownTemplates: number; gameNewerThanSnapshot: boolean | null }
  | { kind: 'missing'; reason: WikiMissingReason }
```
Add `wiki: WikiInfo` to `ExtractionReport` and `| { kind: 'wikiUnavailable' }` to `IpcError`; in `App.vue`, `handleIpcError`'s `switch` gains a `case 'wikiUnavailable':` alongside the other errors it shows.

`commands.ts`: `WikiEntry: 'wiki_entry'`. `wiki.ts`:
```ts
import { invoke } from '@tauri-apps/api/core'
import { Command } from '../constants/commands'
import type { Entry, Target } from './types'

export const wikiEntry = (target: Target): Promise<Entry | null> =>
  invoke(Command.WikiEntry, { target })
```

- [ ] **Step 2: `WikiBlocks.vue`** — recursive component: props `blocks: Block[]`, emits `navigate(target: Target)`. Renders `paragraph` as `<p>`, `list` as `<ul>`/`<ol>` with `<li>` holding the inlines and, if there are `children`, a nested `<WikiBlocks>`; `table` as `<table>` inside an `overflow-x-auto` container; `heading` as `<h3>`/`<h4>`. The inlines go through an internal `WikiInline` component (can't live in the same SFC: create `ui/src/components/WikiInline.vue`, recursive for `edition`): `text` with `font-bold`/`italic` depending on `style` via an exhaustive `switch` with `assertNever`; `ref` as `<button class="underline" @click="emit('navigate', i.target)">` with the `label` and, in parentheses, `kind` and id (for verification, not design); `concept` as `<span class="underline decoration-dotted">`; `edition` as `<span>` with a `[editions]` prefix and the nested inlines. No `<style>`, no hardcoded sizes: only the semantic Tailwind classes already in use in `App.vue` (`underline`, `font-bold`, `italic`, `flex`, `gap-*`, `overflow-x-auto`).

- [ ] **Step 3: `App.vue`** — state `wikiTarget = ref<Target>({ kind: 'item', id: 664 })`, `wikiEntryView = ref<Entry | null>(null)`, `wikiId = ref('664')`; a `loadWiki(target)` function that calls `wikiEntry` and handles the error with `handleIpcError`; loaded in `load()` after `extraction`. Section:
```vue
<section v-if="extraction" class="flex flex-col gap-2">
  <h2 class="font-bold">Wiki</h2>
  <template v-if="extraction.wiki.kind === 'loaded'">
    <p>snapshot {{ extraction.wiki.snapshotAt }} · patch {{ extraction.wiki.lastKnownPatch?.number ?? '?' }} · irrisolti {{ extraction.wiki.unresolved }} · template ignoti {{ extraction.wiki.unknownTemplates }}</p>
    <p>{{ freshnessText(extraction.wiki.gameNewerThanSnapshot) }}</p>
    <p>{{ countsText(extraction.wiki.counts) }}</p>
  </template>
  <p v-else>dataset assente: {{ extraction.wiki.reason }}</p>
  <label class="flex gap-2">oggetto id <input v-model="wikiId" class="border" @keyup.enter="loadWiki({ kind: 'item', id: Number(wikiId) })" /></label>
  <template v-if="wikiEntryView">
    <h3 class="font-bold">{{ wikiEntryView.title }} (revid {{ wikiEntryView.revid }})</h3>
    <div v-for="s in wikiEntryView.sections" :key="s.kind" class="flex flex-col gap-1">
      <h4 class="font-bold">{{ s.kind }}</h4>
      <WikiBlocks :blocks="s.blocks" @navigate="loadWiki" />
    </div>
  </template>
</section>
```
`freshnessText(b: boolean | null)` → `null`: "game not from Steam: freshness can't be checked"; `true`: "the game is newer than the snapshot"; `false`: "snapshot in step with the game". (The raw `<input>` is fine as long as `components/ui/` doesn't exist yet: the verification screen already uses one; the scanner doesn't check it.)

- [ ] **Step 4: `pnpm typecheck`, `pnpm lint`, `pnpm format:check` (or `pnpm --filter ui format` then check), `pnpm scan`** clean. Run `pnpm dev` and eyeball it: Binge Eater with the eight nested entries and clickable links; clicking "Breakfast" reloads the view onto Breakfast; a nonexistent id (99999) shows an empty view with no error.

- [ ] **Step 5: commit** — `app-shell: mirror of the wiki types and a verification panel with the tree`

---

### Task 13: documentation and spec alignment

**Files:**
- Modify: `docs/superpowers/specs/2026-09-05-wiki-dataset-design.md` (the three deviations at the top of the plan), `docs/STATUS.md` (`wiki` module checked off, tool, session log), `docs/BACKLOG.md` (B1: "implementation started, plan report"), `CLAUDE.md` (module table: `wiki`, `wiki-snapshot`; scripts `wiki:fetch`/`wiki:build`; rule: `dataset/raw` and `wiki.json` get committed together, the `derived` test enforces it), `README.md` (one line about the dataset and attribution), `DESIGN-BRIEF.md` (new section with the `Entry`/`Block`/`Inline`/`Target` types and the `wiki_entry` command)
- Create: `docs/superpowers/plans/2026-09-05-wiki-dataset-report.md` (execution report: tasks, commits, gates, what the review found, meta diagnostics with the most frequent unknown templates as a list of the next ones to cover)

- [ ] **Step 1:** write and commit — `docs: dataset wiki, report di esecuzione e documenti allineati`

---

## Self-review

**Spec coverage.** §1 dataset on disk → Tasks 7, 8, 9 (index, pages, cargo, corrections, wiki.json, ATTRIBUTION, embedded, derived test). §2 tree → Tasks 1, 4, 5, 6 (normalized sections, Ref only when resolved, Edition as a node, styles, empty infoboxes for item/trinket). §3 resolution → Task 3 (tables, alias and title, corrections, layout/unknown, stage/room by name), characters from the infoboxes of their pages (deviation 1) in Task 7. §4 tool → Task 8 (UA, maxlag, pause, retry, idempotence, prune, two subcommands, pnpm script). §5 IPC → Task 10 (`LastUpdated`), 11 (`WikiInfo`, `wiki_entry`, `WikiUnavailable`), 12 (types, wrapper, panel, recursive component). §6 tests → spread out: constructs (4, 5, 6), real pass and fixed points (9), resolver (3), derived (9), tool against fake responses (8), IPC shapes (11), no skips.

**Placeholders.** The False PHD table test says how to derive the number from the file instead of pinning it blindly: that's a rule, not a TBD. The unix value of `rfc3339` needs checking with `date` before pinning it: same thing.

**Name consistency.** `parse_template_at`, `Resolver::{new, resolve, by_page_title, achievement_by_name, boss_key}`, `parse_inline(src, r, d)`, `parse_blocks(body, r, d)`, `split_page`, `section_kind`, `extract_infoboxes`, `InfoboxKind::of`, `infobox_from(kind, ib, r, d)`, `parse_page(kind, title, revid, text, r, d)`, `Raw::load`, `page_file_name`, `build(raw, corrections)`, `Dataset::{entry, embedded, from_json, to_json, boss_key}`, `wiki_info(dataset, game_updated_unix)`, `rfc3339_to_unix`, command `wiki_entry(target)`, wrapper `wikiEntry(target)`: used with these names throughout every task.
