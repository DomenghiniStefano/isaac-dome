# Wiki infobox Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stop `crates/wiki` from discarding infobox parameters and templates it has already downloaded, so that all 1727 dataset entries carry the right-hand box the wiki shows.

**Architecture:** Three facts common to every kind (`description`, `dlc`, `unlocked_by`) rise from the six `Infobox` variants onto `Entry`; the two unit variants `Item` and `Trinket` become struct variants holding what only they have; the inline parser learns the seven templates that appear more than fifty times. No new download: every byte this plan reads is already in `dataset/raw/`, fetched on 2026-09-04.

**Tech Stack:** Rust (crate `wiki`, pure, no I/O beyond `dataset/raw/`), `serde`, `serde_json`; the hand-written TypeScript mirror in `ui/src/lib/ipc/types.ts`; `cargo test`, `pnpm wiki:build`, `scripts/check`.

**Spec:** `docs/superpowers/specs/2026-09-13-wiki-infobox-design.md`

## Global Constraints

- **`wiki` is a pure crate.** No network, no I/O outside `dataset/raw/`. `wiki-snapshot` is not modified by this plan at all.
- **The dataset's three files move together.** `crates/wiki/tests/derived.rs` asserts `dataset/wiki.json == build(dataset/raw/, corrections.json)` byte for byte. **Every task that changes parser output must run `pnpm wiki:build` and commit `dataset/wiki.json` in the same commit**, or `derived` goes red.
- **Every struct crossing the IPC has `#[serde(rename_all = "camelCase")]`; every enum with struct variants also needs `rename_all_fields = "camelCase"`.** Without the second one, `devil_price` serializes as `devil_price` and TypeScript reads `undefined` with no error.
- **Exhaustiveness is mandatory.** No `_ =>` arm on `Infobox`, `InfoboxKind`, `Dlc`, `SectionKind`.
- **Never `panic!`/`unwrap()` outside tests** on data read from disk. A missing parameter is an empty value.
- **No parameter is dropped in silence.** Anything not mapped is counted in `Diagnostics` so it appears in `meta`.
- **Never name from a guess.** `dlc` is documented as "the codes the infobox declares", never "introduced in" or "exists in".
- **Commits:** Conventional Commits, `type(scope): subject`, scope `wiki` (or `ui` for the mirror). English. No `Co-Authored-By` trailer, ever.
- **Stage by explicit path.** Other sessions are working in this tree; `git add -A` is forbidden.
- **Before declaring done:** `pnpm check` (`cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace`, `pnpm typecheck`, `pnpm ui:test`, `pnpm lint`, `pnpm format:check`, `pnpm scan`).

---

# Phase 1 — the infobox

### Task 1: `Dlc::parse_codes`, and a counter for codes we cannot read

The `dlc` parameter is not always one code. 175 pages say `r`, 111 say `a+`, 96 say `a`, and one says `a+nr` — three codes concatenated. `Dlc::from_code` takes exactly one and returns `None` for anything else, so used as-is it would silently drop that page's editions.

**Files:**
- Modify: `crates/wiki/src/model.rs` (add `Dlc::parse_codes` next to `from_code`)
- Modify: `crates/wiki/src/diagnostics.rs` (add the counter)
- Test: `crates/wiki/src/model.rs` (the existing `mod tests`)

**Interfaces:**
- Consumes: `Dlc::from_code(&str) -> Option<Dlc>` (already exists)
- Produces: `Dlc::parse_codes(s: &str, d: &mut Diagnostics) -> Vec<Dlc>`, and `Diagnostics::unknown_dlc_code(&mut self, code: &str)`

- [ ] **Step 1: Read how `Diagnostics` counts things today**

Run: `sed -n '1,65p' crates/wiki/src/diagnostics.rs`

You need the existing pattern for a counted map (`unknown_templates`, `discarded_sections`) — copy it exactly rather than inventing a second style.

- [ ] **Step 2: Write the failing test**

Add to `mod tests` in `crates/wiki/src/model.rs`:

```rust
#[test]
fn dlc_codes_split_longest_first_and_leftovers_are_counted() {
    let mut d = Diagnostics::default();
    // One code, the common case: 175 pages say exactly this.
    assert_eq!(Dlc::parse_codes("r", &mut d), vec![Dlc::Repentance]);
    // `r+` must win over `r`: longest match first, or every `r+` reads as `r`.
    assert_eq!(Dlc::parse_codes("r+", &mut d), vec![Dlc::RepentancePlus]);
    // The real page that forced this function to exist.
    assert_eq!(
        Dlc::parse_codes("a+nr", &mut d),
        vec![Dlc::AfterbirthPlus, Dlc::Rebirth, Dlc::Repentance]
    );
    // An absent parameter is an empty list, not an error.
    assert_eq!(Dlc::parse_codes("", &mut d), Vec::<Dlc>::new());
    assert_eq!(d.unknown_dlc_codes.len(), 0);

    // What we cannot read is counted, never dropped in silence.
    assert_eq!(Dlc::parse_codes("zz", &mut d), Vec::<Dlc>::new());
    assert_eq!(d.unknown_dlc_codes.get("zz"), Some(&1));
}
```

- [ ] **Step 3: Run it and confirm it fails**

Run: `cargo test -p wiki dlc_codes_split_longest_first -- --nocapture`
Expected: FAIL to compile — `no function or associated item named parse_codes`.

- [ ] **Step 4: Add the counter to `Diagnostics`**

In `crates/wiki/src/diagnostics.rs`, add the field to the struct and the method, mirroring the existing counted maps:

```rust
/// `dlc` codes the parser does not recognize, by code. A wiki-side typo or a new
/// edition both land here rather than vanishing.
pub unknown_dlc_codes: BTreeMap<String, u32>,
```

```rust
pub fn unknown_dlc_code(&mut self, code: &str) {
    *self.unknown_dlc_codes.entry(code.to_string()).or_insert(0) += 1;
}
```

If `Diagnostics` has a `to_json`/`Serialize` surface listing its maps, add `unknown_dlc_codes` there too, so the count reaches `meta.diagnostics`. That is the whole point of counting it.

- [ ] **Step 5: Implement `parse_codes`**

In `crates/wiki/src/model.rs`, inside `impl Dlc`:

```rust
/// The codes are concatenated without a separator (`a+nr` is `a+`, `n`, `r`), so the
/// two-character ones must be tried first: matching `r` before `r+` would read every
/// `r+` as `r`. Anything left unmatched is counted, one character at a time.
pub fn parse_codes(s: &str, d: &mut Diagnostics) -> Vec<Dlc> {
    const CODES: [&str; 5] = ["a+", "r+", "n", "a", "r"];
    let s = s.trim();
    let mut out = Vec::new();
    let mut rest = s;
    while !rest.is_empty() {
        match CODES.iter().find(|c| rest.starts_with(**c)) {
            Some(c) => {
                if let Some(dlc) = Dlc::from_code(c) {
                    out.push(dlc);
                }
                rest = &rest[c.len()..];
            }
            None => {
                let mut it = rest.chars();
                let bad = it.next().map(|c| c.len_utf8()).unwrap_or(rest.len());
                d.unknown_dlc_code(&rest[..bad]);
                rest = &rest[bad..];
            }
        }
    }
    out
}
```

Note `CODES` puts the two-character codes first and `find` returns the first match, which is what makes longest-first work without sorting.

- [ ] **Step 6: Run the test**

Run: `cargo test -p wiki dlc_codes_split_longest_first -- --nocapture`
Expected: PASS.

- [ ] **Step 7: Run the crate's whole suite**

Run: `cargo test -p wiki`
Expected: all pass. No parser output changed yet, so `derived` is still green and `wiki.json` needs no rebuild.

- [ ] **Step 8: Commit**

```bash
git add crates/wiki/src/model.rs crates/wiki/src/diagnostics.rs
git commit -m "feat(wiki): parse concatenated dlc codes, count the unreadable ones"
```

---

### Task 2: the three common facts rise to `Entry`

`description`, `dlc` and `unlocked by` are on every kind. They move out of the variants and onto `Entry`. This is the first of three changes to the live IPC contract; the TypeScript mirror moves in the same commit.

**Files:**
- Modify: `crates/wiki/src/model.rs` (`Entry`, and four `Infobox` variants lose fields)
- Modify: `crates/wiki/src/infobox.rs` (`infobox_from`, plus a new `entry_facts`)
- Modify: `crates/wiki/src/page.rs` (`parse_page` fills the new fields)
- Modify: `ui/src/lib/ipc/types.ts` (the hand-written mirror)
- Test: `crates/wiki/src/model.rs`, `crates/wiki/src/page.rs`

**Interfaces:**
- Consumes: `Resolver::achievement_by_name(&str) -> Option<Target>`, `Resolver::by_page_title(&str) -> Option<Target>`, `parse_inline(&str, &Resolver, &mut Diagnostics) -> Vec<Inline>`, `Dlc::parse_codes` (Task 1)
- Produces: `Entry { title, revid, description: Vec<Inline>, dlc: Vec<Dlc>, unlocked_by: Option<Target>, infobox, sections }`, and `infobox::entry_facts(ib, r, d) -> EntryFacts`

- [ ] **Step 1: Write the failing test for the new `Entry` shape**

In `crates/wiki/src/model.rs`, replace the `Entry` assertion inside `infobox_and_section_shapes` with:

```rust
let e = Entry {
    title: "Hush".into(),
    revid: 1,
    description: vec![],
    dlc: vec![Dlc::Repentance],
    unlocked_by: None,
    infobox: Infobox::Trinket {
        quote: String::new(),
        tags: vec![],
        pools: vec![],
    },
    sections: vec![],
};
assert_eq!(
    to_value(e).unwrap(),
    json!({
        "title": "Hush",
        "revid": 1,
        "description": [],
        "dlc": ["repentance"],
        "unlockedBy": null,
        "infobox": {"kind": "trinket", "quote": "", "tags": [], "pools": []},
        "sections": []
    })
);
```

`unlockedBy` in that expectation is the assertion that `rename_all = "camelCase"` is doing its job on `Entry`. Do not weaken it.

- [ ] **Step 2: Run it and confirm it fails**

Run: `cargo test -p wiki infobox_and_section_shapes`
Expected: FAIL to compile — `Entry` has no field named `description`.

- [ ] **Step 3: Change `Entry` and strip the four variants**

In `crates/wiki/src/model.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub title: String,
    /// Wiki revision the page was read from: says how stale the data is.
    pub revid: u64,
    /// The infobox's summary line. Plain text for achievements, wikitext elsewhere: both
    /// arrive as inline so the frontend has one shape and no switch on the kind.
    pub description: Vec<Inline>,
    /// The edition codes the infobox declares, parsed. Empty when the parameter is absent.
    /// Deliberately NOT called "introduced in" or "exists in": which of the two it means is
    /// unmeasured, and a name would be a guess.
    pub dlc: Vec<Dlc>,
    /// What the wiki states has to be unlocked first. `None` means "the wiki does not state
    /// one", NEVER "it is free from the start": that answer belongs to `catalog` and `graph`.
    pub unlocked_by: Option<Target>,
    pub infobox: Infobox,
    pub sections: Vec<Section>,
}
```

Then delete `description` from `Infobox::Achievement`, and `unlocked_by` from `Infobox::Boss`, `Infobox::Challenge` and `Infobox::Character`. Leave `unlocks` on `Achievement` and `Challenge` — it points the other way and does not rise.

- [ ] **Step 4: Add `entry_facts` to `infobox.rs`**

```rust
/// The three facts every kind declares, read once per infobox.
pub struct EntryFacts {
    pub description: Vec<Inline>,
    pub dlc: Vec<Dlc>,
    pub unlocked_by: Option<Target>,
}

/// `unlocked by` is an achievement *name* on every kind that uses it — verified on the
/// 277 collectible pages that carry it ("???'s Only Friend", "A Pound of Flesh", …) —
/// so one resolution serves all six kinds.
pub fn entry_facts(ib: &RawInfobox, r: &Resolver, d: &mut Diagnostics) -> EntryFacts {
    EntryFacts {
        description: inline(ib, "description", r, d),
        dlc: Dlc::parse_codes(param(ib, "dlc"), d),
        unlocked_by: r.achievement_by_name(param(ib, "unlocked by")),
    }
}
```

Remove the now-dead reads of those three parameters from `infobox_from`.

- [ ] **Step 5: Fill the fields in `parse_page`**

In `crates/wiki/src/page.rs`, inside the loop in `parse_page`, before pushing:

```rust
let facts = entry_facts(&ib, r, d);
out.push((
    key,
    Entry {
        title: entry_title(kind, title, &ib, r),
        revid,
        description: facts.description,
        dlc: facts.dlc,
        unlocked_by: facts.unlocked_by,
        infobox: infobox_from(kind, &ib, r, d),
        sections,
    },
));
```

Add `entry_facts` to the `use crate::infobox::{…}` list at the top of the file.

- [ ] **Step 6: Write the page-level test**

In `crates/wiki/src/page.rs`'s `mod tests`:

```rust
#[test]
fn the_three_common_facts_land_on_the_entry_not_the_infobox() {
    let src = "{{infobox passive collectible\n | id = 25\n | dlc = r\n | description = Tears up\n | unlocked by = Epic Fetus\n}}\n== Effects ==\n* a\n";
    let mut d = Diagnostics::default();
    let v = parse_page("Breakfast", 7, src, &test_resolver(), &mut d);
    let e = &v[0].1;
    assert_eq!(e.dlc, vec![Dlc::Repentance]);
    assert_eq!(e.unlocked_by, Some(Target::Achievement { id: 62 }));
    assert!(matches!(
        e.description.first(),
        Some(Inline::Text { text, .. }) if text.contains("Tears up")
    ));
}
```

- [ ] **Step 7: Run both tests, then the crate**

Run: `cargo test -p wiki`
Expected: PASS. Fix any other call site the compiler names — removing fields from four variants will break their construction sites and their tests, which is the compiler doing its job.

- [ ] **Step 8: Update the TypeScript mirror**

In `ui/src/lib/ipc/types.ts`, add to `Entry` and remove from the four variants:

```ts
export interface Entry {
  title: string
  revid: number
  description: Inline[]
  dlc: Dlc[]
  unlockedBy: Target | null
  infobox: Infobox
  sections: Section[]
}
```

Delete `description` from the `achievement` variant and `unlockedBy` from `boss`, `challenge` and `character`. If `Dlc` is not already exported there, add it as a value-union per the repo's rule (`const Dlc = { … } as const`), not a `type X = 'a' | 'b'`.

- [ ] **Step 9: Regenerate the dataset**

Run: `pnpm wiki:build`
Expected: it rewrites `dataset/wiki.json`. Then `cargo test -p wiki derived` must pass; if it does not, the build did not run or did not finish.

- [ ] **Step 10: Check and commit**

```bash
pnpm typecheck && pnpm scan
git add crates/wiki/src/model.rs crates/wiki/src/infobox.rs crates/wiki/src/page.rs ui/src/lib/ipc/types.ts dataset/wiki.json
git commit -m "refactor(wiki): description, dlc and unlocked_by rise from the infobox to the entry"
```

---

### Task 3: `Infobox::Item` and `Infobox::Trinket` stop being empty

This is the defect the whole plan exists for: 907 of 1727 entries carry `{"kind":"item"}` and nothing else.

**Files:**
- Modify: `crates/wiki/src/model.rs` (the two variants)
- Modify: `crates/wiki/src/infobox.rs` (`InfoboxKind`, `infobox_from`)
- Modify: `ui/src/lib/ipc/types.ts`
- Test: `crates/wiki/src/infobox.rs`, and a new `crates/wiki/tests/infobox_filled.rs`

**Interfaces:**
- Consumes: `param`, `text`, `inline`, `leading_number` (all private helpers already in `infobox.rs`)
- Produces: `Infobox::Item { quote, activated, quality, tags, recharge, devil_price, shop_price, pools }`, `Infobox::Trinket { quote, tags, pools }`, and `InfoboxKind::{Passive, Activated}` replacing `InfoboxKind::Collectible`

- [ ] **Step 1: Write the failing test**

In `crates/wiki/src/infobox.rs`'s `mod tests`:

```rust
#[test]
fn a_collectible_infobox_keeps_its_parameters() {
    let r = test_resolver();
    let mut d = Diagnostics::default();
    // Brimstone's real infobox, trimmed to the parameters this type holds.
    let ib = raw(
        "infobox passive collectible",
        &[
            ("quote", "Blood laser barrage"),
            ("quality", "4"),
            ("tags", "devil summonable offensive"),
            ("devil price", "2"),
        ],
    );
    let Infobox::Item {
        quote,
        activated,
        quality,
        tags,
        recharge,
        devil_price,
        shop_price,
        pools,
    } = infobox_from(InfoboxKind::Passive, &ib, &r, &mut d)
    else {
        panic!()
    };
    assert_eq!(quote, "Blood laser barrage");
    assert!(!activated);
    assert_eq!(quality, Some(4));
    assert_eq!(tags, vec!["devil", "summonable", "offensive"]);
    assert!(recharge.is_empty());
    assert!(shop_price.is_empty());
    assert!(pools.is_empty());
    // A price is inline, not a number: 36 of the 56 real values are `{{dlcalt|…}}`.
    assert!(matches!(
        devil_price.first(),
        Some(Inline::Text { text, .. }) if text.trim() == "2"
    ));
}

#[test]
fn an_activated_collectible_is_marked_activated() {
    let r = test_resolver();
    let mut d = Diagnostics::default();
    let ib = raw("infobox activated collectible", &[("recharge", "6")]);
    let Infobox::Item { activated, recharge, .. } =
        infobox_from(InfoboxKind::Activated, &ib, &r, &mut d)
    else {
        panic!()
    };
    assert!(activated);
    assert!(!recharge.is_empty());
}
```

- [ ] **Step 2: Run and confirm it fails**

Run: `cargo test -p wiki a_collectible_infobox_keeps_its_parameters`
Expected: FAIL to compile — `InfoboxKind` has no variant `Passive`.

- [ ] **Step 3: Split `InfoboxKind::Collectible`**

The template name already distinguishes the two and `InfoboxKind::of` currently merges them. In `crates/wiki/src/infobox.rs`:

```rust
pub enum InfoboxKind {
    Passive,
    Activated,
    Trinket,
    Achievement,
    Boss,
    Challenge,
    Character,
}
```

```rust
pub fn of(name: &str) -> Option<InfoboxKind> {
    Some(match name {
        "infobox passive collectible" | "infobox collectible" => InfoboxKind::Passive,
        "infobox activated collectible" => InfoboxKind::Activated,
        "infobox trinket" => InfoboxKind::Trinket,
        "infobox achievement" => InfoboxKind::Achievement,
        "infobox boss" => InfoboxKind::Boss,
        "infobox challenge" => InfoboxKind::Challenge,
        "infobox character" => InfoboxKind::Character,
        _ => return None, // allowed: template name, an open-ended string
    })
}
```

`infobox collectible` (no adjective) maps to `Passive`: it is the generic form and the game has no third kind. Say so in a comment so the next reader does not re-derive it.

The compiler will now name every `match` on `InfoboxKind` that is missing an arm — `entry_key` in `page.rs` among them. Fix each by treating `Passive` and `Activated` exactly as `Collectible` was treated.

- [ ] **Step 4: Add a helper for whitespace-separated tags**

In `crates/wiki/src/infobox.rs`, next to the other helpers:

```rust
/// `"devil summonable offensive"` → three tags. The vocabulary is the game's and open, so
/// this stays `Vec<String>`: a closed enum breaks the day the game adds a tag.
fn tags(ib: &RawInfobox, name: &str) -> Vec<String> {
    param(ib, name)
        .split_whitespace()
        .map(str::to_string)
        .collect()
}
```

- [ ] **Step 5: Change the two variants and build them**

In `crates/wiki/src/model.rs`:

```rust
Item {
    /// The pickup quote — the same string as `items.xml`'s `description` attribute.
    quote: String,
    activated: bool,
    quality: Option<i8>,
    tags: Vec<String>,
    /// Not a number: the real values include `unlimited`, `one time`, `4s` and
    /// `{{dlcalt|6|r=4}}`.
    recharge: Vec<Inline>,
    devil_price: Vec<Inline>,
    shop_price: Vec<Inline>,
    pools: Vec<Inline>,
},
Trinket {
    quote: String,
    tags: Vec<String>,
    pools: Vec<Inline>,
},
```

In `infobox_from`:

```rust
InfoboxKind::Passive | InfoboxKind::Activated => Infobox::Item {
    quote: text(ib, "quote"),
    activated: matches!(kind, InfoboxKind::Activated),
    quality: param(ib, "quality").trim().parse().ok(),
    tags: tags(ib, "tags"),
    recharge: inline(ib, "recharge", r, d),
    devil_price: inline(ib, "devil price", r, d),
    shop_price: inline(ib, "shop price", r, d),
    pools: inline(ib, "pool", r, d),
},
InfoboxKind::Trinket => Infobox::Trinket {
    quote: text(ib, "quote"),
    tags: tags(ib, "tags"),
    pools: inline(ib, "pool", r, d),
},
```

`quality` parses to `Option<i8>` to match `catalog::Metadata::quality`, which is `-1..=4` in the real file.

- [ ] **Step 6: Run the two tests, then the crate**

Run: `cargo test -p wiki`
Expected: PASS.

- [ ] **Step 7: Write the property test — no entry has an empty infobox**

Create `crates/wiki/tests/infobox_filled.rs`:

```rust
//! The defect this crate carried until 2026-09-13: 907 of 1727 entries reached the
//! frontend with `{"kind":"item"}` and nothing else. This is also the non-vacuity guard —
//! if the parser ever stops finding the templates, the dataset silently shrinks and this
//! test is what says so.

use wiki::{Dataset, Infobox};

#[test]
fn every_entry_carries_a_non_empty_infobox() {
    let d = Dataset::embedded();
    let mut empty = Vec::new();
    let mut total = 0usize;
    for (title, infobox) in d.all_infoboxes() {
        total += 1;
        let is_empty = match infobox {
            Infobox::Item { quote, tags, .. } => quote.is_empty() && tags.is_empty(),
            Infobox::Trinket { quote, tags, .. } => quote.is_empty() && tags.is_empty(),
            Infobox::Achievement { requirements, .. } => requirements.is_empty(),
            Infobox::Boss { base_hp, environment, .. } => base_hp.is_none() && environment.is_empty(),
            Infobox::Challenge { goal, .. } => goal.is_empty(),
            Infobox::Character { health, .. } => health.is_empty(),
        };
        if is_empty {
            empty.push(title.to_string());
        }
    }
    // Non-vacuity: if the dataset ever comes back nearly empty, the loop above passes
    // trivially. Assert it actually ran over the whole thing.
    assert!(total > 1700, "only {total} entries: the dataset shrank");
    assert!(
        empty.len() < total / 20,
        "{} of {total} entries have an empty infobox, first few: {:?}",
        empty.len(),
        &empty[..empty.len().min(10)]
    );
}
```

If `Dataset` has no `all_infoboxes()`, add one returning `impl Iterator<Item = (&str, &Infobox)>` over the six maps — check `crates/wiki/src/dataset.rs` for how the maps are named and iterate them in a fixed order.

The threshold is "fewer than one in twenty", not zero: a handful of wiki pages genuinely declare almost nothing, and a zero-tolerance assertion would be a test that fails on the wiki's editorial choices rather than on our parser.

- [ ] **Step 8: Update the TypeScript mirror**

```ts
  | {
      kind: 'item'
      quote: string
      activated: boolean
      quality: number | null
      tags: string[]
      recharge: Inline[]
      devilPrice: Inline[]
      shopPrice: Inline[]
      pools: Inline[]
    }
  | { kind: 'trinket'; quote: string; tags: string[]; pools: Inline[] }
```

- [ ] **Step 9: Regenerate, check, commit**

```bash
pnpm wiki:build
cargo test -p wiki && pnpm typecheck && pnpm scan
git add crates/wiki/src/model.rs crates/wiki/src/infobox.rs crates/wiki/src/page.rs crates/wiki/src/dataset.rs crates/wiki/tests/infobox_filled.rs ui/src/lib/ipc/types.ts dataset/wiki.json
git commit -m "feat(wiki): items and trinkets keep their infobox instead of dropping it"
```

---

### Task 4: the parameters lost on the four kinds that were already parsed

Bosses lose `stage hp` (2) and `variant` (26); challenges lose `used_character` (14); characters lose `tears` (12) and `parent` (2).

**Files:**
- Modify: `crates/wiki/src/model.rs`, `crates/wiki/src/infobox.rs`, `ui/src/lib/ipc/types.ts`
- Test: `crates/wiki/src/infobox.rs`

**Interfaces:**
- Consumes: `Resolver::character_of_page(&str) -> Option<(u32, String)>`, `Resolver::by_page_title`
- Produces: `Infobox::Boss` gains `stage_hp: Vec<Inline>` and `variant: Option<u32>`; `Infobox::Challenge` gains `character: Option<Target>`; `Infobox::Character` gains `tears: String` and `parent: Option<Target>`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn the_four_parsed_kinds_keep_the_parameters_they_used_to_drop() {
    let r = test_resolver();
    let mut d = Diagnostics::default();

    let ib = raw("infobox boss", &[("base hp", "250 (x2)"), ("variant", "1"), ("stage hp", "300")]);
    let Infobox::Boss { variant, stage_hp, .. } = infobox_from(InfoboxKind::Boss, &ib, &r, &mut d)
    else { panic!() };
    assert_eq!(variant, Some(1));
    assert!(!stage_hp.is_empty());

    let ib = raw("infobox challenge", &[("character", "Cain")]);
    let Infobox::Challenge { character, .. } = infobox_from(InfoboxKind::Challenge, &ib, &r, &mut d)
    else { panic!() };
    assert!(character.is_some(), "the challenge's character must resolve to a target");

    let ib = raw("infobox character", &[("tears", "2.73"), ("parent", "Lazarus")]);
    let Infobox::Character { tears, parent, .. } =
        infobox_from(InfoboxKind::Character, &ib, &r, &mut d)
    else { panic!() };
    assert_eq!(tears, "2.73");
    assert!(parent.is_some());
}
```

Before writing the expectations for `character` and `parent`, run `sed -n '1,60p' crates/wiki/src/resolver.rs` and look at `mod fixtures::test_resolver` — if `Cain` or `Lazarus` are not in the fixture, add them there rather than weakening the assertion to `is_none()`.

- [ ] **Step 2: Run and confirm it fails**

Run: `cargo test -p wiki the_four_parsed_kinds_keep`
Expected: FAIL to compile — no field `variant` on `Infobox::Boss`.

- [ ] **Step 3: Add the fields**

`Boss`: `stage_hp: Vec<Inline>`, `variant: Option<u32>`. `Challenge`: `character: Option<Target>`. `Character`: `tears: String`, `parent: Option<Target>`.

In `infobox_from`:

```rust
InfoboxKind::Boss => Infobox::Boss {
    base_hp: leading_number(param(ib, "base hp")),
    stage_hp: inline(ib, "stage hp", r, d),
    variant: leading_number(param(ib, "variant")),
    environment: inline(ib, "environment", r, d),
    pool: inline(ib, "pool", r, d),
},
```

For the challenge's character and the character's parent, both are page titles or names, so use `r.by_page_title(param(ib, "character"))` and `r.by_page_title(param(ib, "parent"))`.

`stage hp` is inline rather than a number because its two real values are per-stage notes, not a scalar — the same reason as the prices in Task 3.

- [ ] **Step 4: Run the test and the crate**

Run: `cargo test -p wiki`
Expected: PASS.

- [ ] **Step 5: Mirror, regenerate, commit**

Update the three variants in `ui/src/lib/ipc/types.ts` (`stageHp`, `variant`, `character`, `tears`, `parent`), then:

```bash
pnpm wiki:build
cargo test -p wiki && pnpm typecheck
git add crates/wiki/src/model.rs crates/wiki/src/infobox.rs ui/src/lib/ipc/types.ts dataset/wiki.json
git commit -m "feat(wiki): bosses, challenges and characters keep the parameters they dropped"
```

---

### Task 5: no parameter disappears in silence

The defect being fixed was silent. This test is what stops it coming back.

**Files:**
- Create: `crates/wiki/tests/no_silent_parameter.rs`
- Modify: `crates/wiki/src/infobox.rs` (a declared list of parameters we deliberately ignore)

**Interfaces:**
- Produces: `infobox::IGNORED_PARAMS: &[&str]`

- [ ] **Step 1: Add the declared ignore list**

In `crates/wiki/src/infobox.rs`:

```rust
/// Parameters we read and deliberately do not keep, each with its reason. A parameter that
/// is in neither this list nor a type is a bug, and `no_silent_parameter` says so.
///
/// The asset names (`image name`, `costume name`, `files name`, `tear app*`, `costume*`,
/// `bomb app`, `portrait*`, `image`) name files inside the user's own copy of the game:
/// `unpack` extracts sprites from there, and no wiki image is ever shipped.
/// `name`, `id`, `number`, `link`, `alias` are the entry's identity, already resolved into
/// the key and the title before the infobox is converted.
pub const IGNORED_PARAMS: &[&str] = &[
    "image name", "costume name", "files name", "tear app name", "tear app", "tear app scale",
    "costume", "costume scale", "bomb app", "portrait", "portrait name", "image",
    "name", "id", "number", "link", "alias",
    "hidden", "character appearance", "appearance", "behavior", "is mini-boss",
    "oldpool", "special goal",
];
```

- [ ] **Step 2: Write the test**

```rust
//! Every parameter the wiki writes is either in a type or in `IGNORED_PARAMS`. The bug this
//! guards against is exactly the one fixed on 2026-09-13: parameters parsed, then dropped,
//! with a green suite.

use std::collections::BTreeMap;
use std::path::PathBuf;
// `mod infobox` is private: everything public is re-exported from the crate root, so add
// `IGNORED_PARAMS` to the `pub use infobox::{…}` line in `lib.rs` when you declare it.
use wiki::{extract_infoboxes, InfoboxKind, Raw, IGNORED_PARAMS};

/// Every parameter name any type in this crate reads. Kept by hand, next to the types it
/// mirrors: a field added without a line here trips the test, which is the point.
const KEPT: &[&str] = &[
    "description", "dlc", "unlocked by",
    "quote", "quality", "tags", "recharge", "devil price", "shop price", "pool",
    "requirements", "unlocks",
    "base hp", "stage hp", "variant", "environment",
    "blindfolded", "has shops", "has treasure rooms", "item", "trinket", "pickup",
    "health", "curse", "goal", "character",
    "damage", "tears", "range", "speed", "luck", "shot speed", "pickups", "collectibles",
    "parent",
];

#[test]
fn every_wikitext_parameter_is_kept_or_declared_ignored() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset/raw");
    let raw = Raw::load(&root).expect("dataset/raw/");
    let mut unaccounted: BTreeMap<String, u32> = BTreeMap::new();
    let mut seen = 0usize;
    for page in raw.pages() {
        for ib in extract_infoboxes(page.text()) {
            if InfoboxKind::of(&ib.name).is_none() {
                continue;
            }
            for key in ib.params.keys() {
                seen += 1;
                if !KEPT.contains(&key.as_str()) && !IGNORED_PARAMS.contains(&key.as_str()) {
                    *unaccounted.entry(key.clone()).or_insert(0) += 1;
                }
            }
        }
    }
    assert!(seen > 5000, "only {seen} parameters read: the loop did not run");
    assert!(
        unaccounted.is_empty(),
        "parameters neither kept nor declared ignored: {unaccounted:?}"
    );
}
```

`Raw::load` and the page iterator already exist — check `crates/wiki/src/raw.rs` for their exact names and adapt the two lines that use them. Do not change `raw.rs` to suit the test.

- [ ] **Step 3: Run it**

Run: `cargo test -p wiki every_wikitext_parameter_is_kept_or_declared_ignored -- --nocapture`
Expected: it may FAIL the first time, listing names. **That failure is the test working.** For each name, decide: is it data we want (add the field, in its own commit) or noise (add it to `IGNORED_PARAMS` with a reason in the doc comment)? Do not add a name to `IGNORED_PARAMS` without writing why.

- [ ] **Step 4: Make it pass, then commit**

```bash
cargo test -p wiki
git add crates/wiki/src/infobox.rs crates/wiki/tests/no_silent_parameter.rs
git commit -m "test(wiki): no infobox parameter is dropped without being declared"
```

---

### Task 6: the section titles that fall through by oversight

`meta.diagnostics.discardedSections` mixes two things. `Trivia` (875), `Gallery` (346), `In-game Footage` (841 across four spellings) and `References` (100) are out on purpose. Six others are out by accident.

**Files:**
- Modify: `crates/wiki/src/sections.rs`
- Test: `crates/wiki/src/sections.rs` (`mod tests`)

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn titles_that_used_to_fall_through_now_map() {
    assert_eq!(section_kind("Unlockable Items"), Some(SectionKind::Unlockable));
    assert_eq!(section_kind("How to Acquire"), Some(SectionKind::Unlockable));
    assert_eq!(section_kind("Excluded Items"), Some(SectionKind::Notes));
    assert_eq!(section_kind("Bug"), Some(SectionKind::Bugs));
    assert_eq!(section_kind("Interaction"), Some(SectionKind::Interactions));
    assert_eq!(section_kind("Rewards"), Some(SectionKind::Reward));
    // Still deliberately out — this half of the test is the one that protects the choice.
    assert_eq!(section_kind("Trivia"), None);
    assert_eq!(section_kind("Gallery"), None);
    assert_eq!(section_kind("In-game Footage"), None);
}
```

- [ ] **Step 2: Run and confirm it fails**

Run: `cargo test -p wiki titles_that_used_to_fall_through`
Expected: FAIL — `Unlockable Items` gives `None`.

- [ ] **Step 3: Extend the match**

```rust
"unlockable achievements" | "unlockable achievement" | "unlockable starting items"
| "unlockable items" | "how to acquire" => SectionKind::Unlockable,
"bugs" | "bug" => SectionKind::Bugs,
"interactions" | "item interactions" | "interaction" => SectionKind::Interactions,
"reward" | "rewards" => SectionKind::Reward,
"notes" | "excluded items" => SectionKind::Notes,
```

- [ ] **Step 4: Run, regenerate, commit**

```bash
cargo test -p wiki
pnpm wiki:build
cargo test -p wiki derived
git add crates/wiki/src/sections.rs dataset/wiki.json
git commit -m "fix(wiki): six section titles were falling through the mapping"
```

---

### Task 7: where both sources speak, count the disagreements

This is what earns the claim in Task 3's doc comment that the wiki's `quote` is the game's `description`.

**Files:**
- Create: `crates/ipc/tests/wiki_agrees_with_catalog.rs` (in `ipc`, because it is the only crate that already depends on both `wiki` and `catalog`)

**Interfaces:**
- Consumes: `test_support`'s accessor for `samples/packed`, `catalog::Catalog`, `wiki::Dataset::embedded()`

- [ ] **Step 1: Copy the opening from a test that already does this**

`test-support` offers `packed_dir() -> Option<PathBuf>`, `packed_file(name) -> Option<PathBuf>` and `skip(reason)`. `samples/` is only ever opened through this crate, and these functions print `sample: …` or `skip: …` on stderr so a test on real data declares which slice it ran on.

Run: `sed -n '1,30p' crates/ipc/tests/collection_real.rs`

Copy its opening lines verbatim — it already builds a `Catalog` from `samples/packed` and skips cleanly when the junction is absent. Do not write a second way of doing it.

- [ ] **Step 2: Write the test**

```rust
//! The wiki and the game both state an item's quality, its tags and its pickup quote. A
//! disagreement is not a failure — the dataset's snapshot is 2026-09-04 and its
//! `lastKnownPatch` is v1.9.7.17 (2026-04-20), while the user's install is whatever it is.
//! A *systematic* disagreement is a failure: it means we are reading the field wrong.

const MAX_DISAGREEMENT: f64 = 0.05;

#[test]
fn wiki_and_catalog_agree_on_quality_tags_and_quote() {
    let Some(packed) = test_support::packed_dir() else {
        test_support::skip("samples/packed is missing: no game to compare the wiki against");
        return;
    };
    let catalog = /* build it from `packed` exactly as collection_real.rs does */;
    let dataset = wiki::Dataset::embedded();

    let (mut both, mut bad_quality, mut bad_tags, mut bad_quote) = (0u32, 0u32, 0u32, 0u32);
    let mut examples: Vec<String> = Vec::new();

    for item in catalog.items() {
        let Some(entry) = dataset.item(item.id.0) else { continue };
        let wiki::Infobox::Item { quality, tags, quote, .. } = &entry.infobox else { continue };
        both += 1;
        if let (Some(w), Some(g)) = (quality, item.quality) {
            if w != &g {
                bad_quality += 1;
                if examples.len() < 10 {
                    examples.push(format!("{} quality wiki={w} game={g}", item.id.0));
                }
            }
        }
        if !tags.is_empty() && !item.tags.is_empty() {
            let (mut a, mut b) = (tags.clone(), item.tags.clone());
            a.sort();
            b.sort();
            if a != b {
                bad_tags += 1;
            }
        }
        // The claim under test: the wiki's `quote` IS the game's `description`.
        let game_quote = catalog.text(&item.description);
        if !quote.is_empty() && !game_quote.is_empty() && quote != &game_quote {
            bad_quote += 1;
        }
    }

    assert!(both > 500, "only {both} items matched: the join is broken, not the data");
    let rate = |n: u32| f64::from(n) / f64::from(both);
    eprintln!(
        "compared {both} items: quality {bad_quality} ({:.1}%), tags {bad_tags} ({:.1}%), quote {bad_quote} ({:.1}%)",
        rate(bad_quality) * 100.0, rate(bad_tags) * 100.0, rate(bad_quote) * 100.0
    );
    for e in &examples {
        eprintln!("  {e}");
    }
    assert!(rate(bad_quality) < MAX_DISAGREEMENT, "quality disagrees too often");
    assert!(rate(bad_tags) < MAX_DISAGREEMENT, "tags disagree too often");
    assert!(rate(bad_quote) < MAX_DISAGREEMENT, "quote disagrees too often: it may not be the pickup quote at all");
}
```

Adapt `catalog.items()`, `dataset.item(...)` and `catalog.text(...)` to the real names — run `grep -n "pub fn" crates/catalog/src/catalog.rs crates/wiki/src/dataset.rs` first.

- [ ] **Step 3: Run it with output visible**

Run: `cargo test -p ipc wiki_and_catalog_agree -- --nocapture`
Expected: either `skip: …` (no `samples/packed`) or a printed comparison. **Read the printed rates.** If one is near 5%, do not widen the constant — report the number and the examples, because a rate that high means the field is being read wrong and that is a finding, not a threshold problem.

- [ ] **Step 4: Commit**

```bash
git add crates/ipc/tests/wiki_agrees_with_catalog.rs
git commit -m "test(ipc): the wiki and the game agree on quality, tags and the pickup quote"
```

---

### Task 8: measure the Cargo `dlc` bitmask, report, assert nothing

The spec's decision 5 already found the counter-example: Blue Cap (342), the first Afterbirth collectible, has mask 31 (all five editions) and no wikitext `dlc`. This task turns that anecdote into a number over all 733 items.

**Files:**
- Create: `crates/wiki/examples/dlc_mask.rs` (an example, not a test: its output is a measurement, and nothing depends on it)

- [ ] **Step 1: Write the example**

```rust
//! Is Cargo's integer `dlc` a five-bit mask over the editions? Reads `dataset/raw/cargo/`
//! and compares against `catalog::origin`'s id boundaries, which come from the game.
//!
//! Run: cargo run -q -p wiki --example dlc_mask
//!
//! It asserts nothing. Blue Cap (342, the first Afterbirth item) already has mask 31, so a
//! passing assertion would mean the test is wrong, not that the reading is right.
```

Group the 733 collectibles by `(origin, mask)` and print the contingency table plus, for each origin, the masks seen and how many items each covers. Print separately the count of items whose *wikitext* `dlc` parses to a list including `RepentancePlus`, since `catalog::Origin` has no such variant and cannot be compared there.

- [ ] **Step 2: Run it**

Run: `cargo run -q -p wiki --example dlc_mask`
Expected: a table. Read it.

- [ ] **Step 3: Write down the answer**

Append a short section to `docs/superpowers/specs/2026-09-13-wiki-infobox-design.md` under decision 5 recording what the table says — whether the mask is "exists in", "introduced in", or neither, and what happens at the Afterbirth boundary. If it turns out to be readable, that is a *separate* future commit; this plan does not adopt it.

- [ ] **Step 4: Commit**

```bash
git add crates/wiki/examples/dlc_mask.rs docs/superpowers/specs/2026-09-13-wiki-infobox-design.md
git commit -m "chore(wiki): measure whether cargo's dlc integer is an edition bitmask"
```

---

### Task 9: hand the contract on

The IPC contract has had a design system on the other end since 2026-09-10. Three tasks above changed it.

**Files:**
- Modify: `DESIGN-BRIEF.md`, `docs/STATUS.md`

- [ ] **Step 1: Update `DESIGN-BRIEF.md`**

Find the section carrying the wiki TypeScript types and replace `Entry` and the six `Infobox` variants with what now sits in `ui/src/lib/ipc/types.ts`. They must match character for character — the brief is the design's copy of the contract, and a brief that disagrees with the types is worse than no brief.

- [ ] **Step 2: Update `docs/STATUS.md`**

Add the sub-project under the wiki dataset's heading, with the spec and plan paths, checkboxes ticked only for what is committed. A ticked box means landed work, never a noted intention.

- [ ] **Step 3: Full check**

Run: `pnpm check`
Expected: green. Then `cargo test --workspace -- --nocapture | grep -c "^skip:"` and read how many real-data tests skipped — an "N passed" does not say how many were silent.

- [ ] **Step 4: Commit**

```bash
git add DESIGN-BRIEF.md docs/STATUS.md
git commit -m "docs: hand on the wiki infobox contract change"
```

---

# Phase 2 — the templates

Seven templates appear more than fifty times and are not understood, so their sentences reach the frontend mangled. The closing criterion from the spec: **`unknownTemplates` holds nothing above 50 occurrences**, and each one still listed is named with a reason.

All seven are handled in `Resolver::resolve(template, arg) -> Resolution`, which returns `Target`, `Unresolved`, `Ignore` or `Unknown`; `Unknown` is what feeds `meta.diagnostics.unknownTemplates`.

### Task 10: `m` and `machine` → a concept link (409 occurrences)

Both name a machine: `{{m|Donation Machine}}`, `{{machine|Greed Donation Machine}}`. The game has no id for machines, and `Target` has no machine variant — `Inline::Concept` is exactly the case for "a wiki page with no id in the game".

- [ ] **Step 1: Confirm the argument shape yourself**

Run: `grep -ohE "\{\{(m|machine)\|[^}]{0,40}" dataset/raw/pages/*/*.wikitext | sort | uniq -c | sort -rn | head -20`

- [ ] **Step 2: Write the failing test**

In `crates/wiki/src/inline.rs`'s `mod tests` (or `resolver.rs`, wherever `resolve` is tested):

```rust
#[test]
fn machine_templates_become_concept_links() {
    let r = test_resolver();
    let mut d = Diagnostics::default();
    let v = parse_inline("Use the {{m|Donation Machine}} here", &r, &mut d);
    assert!(v.iter().any(|i| matches!(
        i,
        Inline::Concept { page, label } if page == "Donation Machine" && label == "Donation Machine"
    )));
    assert!(d.unknown_templates.is_empty(), "{:?}", d.unknown_templates);
}
```

- [ ] **Step 3: Run it, confirm it fails**

Run: `cargo test -p wiki machine_templates_become_concept_links`
Expected: FAIL — the inline is `Text`, and `unknown_templates` contains `m`.

- [ ] **Step 4: Implement**

`resolve` returns a `Resolution`, which has no concept variant today. Check: `grep -n "enum Resolution" -A 10 crates/wiki/src/resolver.rs`. If there is no `Resolution::Concept(String)`, add one and handle it where `resolve`'s result is consumed in `inline.rs`, turning it into `Inline::Concept { page, label }`. Then:

```rust
// The game has no id for a machine, so it is a wiki concept, not a target.
"m" | "machine" => return Resolution::Concept(arg.trim().to_string()),
```

- [ ] **Step 5: Run, regenerate, commit**

```bash
cargo test -p wiki
pnpm wiki:build
cargo test -p wiki derived
git add crates/wiki/src/resolver.rs crates/wiki/src/inline.rs dataset/wiki.json
git commit -m "feat(wiki): machine templates resolve to a concept link"
```

---

### Task 11: `ip` → the item pool, as a concept (110 occurrences)

`{{ip|Boss}}` names an item pool. Pools have no id in the game's XML either — `itempools.xml` keys them by name — so they are concepts too.

- [ ] **Step 1: Census the arguments**

Run: `grep -ohE "\{\{ip\|[^}]{0,30}" dataset/raw/pages/*/*.wikitext | sort | uniq -c | sort -rn`

- [ ] **Step 2: Write the failing test**

```rust
#[test]
fn item_pool_template_becomes_a_concept() {
    let r = test_resolver();
    let mut d = Diagnostics::default();
    let v = parse_inline("Found in the {{ip|Boss}} pool", &r, &mut d);
    assert!(v.iter().any(|i| matches!(i, Inline::Concept { page, .. } if page == "Boss")));
    assert!(d.unknown_templates.is_empty());
}
```

- [ ] **Step 3: Run, confirm it fails, implement**

Add `"ip" => return Resolution::Concept(arg.trim().to_string()),` next to the machines, with a comment saying pools are keyed by name in `itempools.xml`.

- [ ] **Step 4: Run, regenerate, commit**

```bash
cargo test -p wiki && pnpm wiki:build && cargo test -p wiki derived
git add crates/wiki/src/resolver.rs dataset/wiki.json
git commit -m "feat(wiki): the item pool template resolves to a concept link"
```

---

### Task 12: `transformation contribution` → a transformation target (186 occurrences)

`{{transformation contribution|Beelzebub}}` means "this item counts toward Beelzebub". The resolver already maps `tf` to `Target::Transformation`, so the target side is free; only the name is new.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn transformation_contribution_resolves_like_tf() {
    let r = test_resolver();
    let mut d = Diagnostics::default();
    let v = parse_inline("{{transformation contribution|Leviathan}}", &r, &mut d);
    assert!(v.iter().any(|i| matches!(
        i,
        Inline::Ref { target: Target::Transformation { .. }, .. }
    )));
    assert!(d.unknown_templates.is_empty());
}
```

If `Leviathan` is not in `test_resolver`'s transformation map, add it there.

- [ ] **Step 2: Run, confirm it fails**

Run: `cargo test -p wiki transformation_contribution_resolves_like_tf`

- [ ] **Step 3: Implement**

In `resolve`, extend the existing `tf` arm:

```rust
// `{{transformation contribution|X}}` is a sentence ("counts toward X"), but the only
// thing in it we can resolve is the transformation, same as `{{tf|X}}`.
"tf" | "transformation contribution" => self
    .transformations
    .get(&k)
    .map(|id| Target::Transformation { id: *id }),
```

- [ ] **Step 4: Run, regenerate, commit**

```bash
cargo test -p wiki && pnpm wiki:build && cargo test -p wiki derived
git add crates/wiki/src/resolver.rs dataset/wiki.json
git commit -m "feat(wiki): transformation contribution resolves to its transformation"
```

---

### Task 13: `achievement text` → several achievements (127 occurrences)

Unlike the others this one takes a **comma-separated list**: `{{achievement text | I RULE!, Backasswards, Ultra Hard, Golden God!, The D20, Celtic Cross}}`, on boss pages. `resolve` returns one `Resolution`, so this cannot be done inside it — it is handled where templates are expanded in `inline.rs`.

- [ ] **Step 1: Read how `inline.rs` expands a template**

Run: `grep -n "resolve(" -B 5 -A 20 crates/wiki/src/inline.rs | head -60`

You need the point where the template name and argument are in hand and a `Vec<Inline>` can be pushed, not a single `Inline`.

- [ ] **Step 2: Write the failing test**

```rust
#[test]
fn achievement_text_lists_every_achievement_it_names() {
    let r = test_resolver();
    let mut d = Diagnostics::default();
    let v = parse_inline("{{achievement text | Epic Fetus, Cain}}", &r, &mut d);
    let refs: Vec<_> = v
        .iter()
        .filter(|i| matches!(i, Inline::Ref { target: Target::Achievement { .. }, .. }))
        .collect();
    assert_eq!(refs.len(), 2, "both names must resolve, got {v:?}");
    assert!(d.unknown_templates.is_empty());
}
```

- [ ] **Step 3: Run, confirm it fails, implement**

Split the argument on `,`, trim each part, resolve each through `self.achievements` (the same map `a`/`achievement` uses), and push one `Inline::Ref` per resolved name with a plain `Inline::Text { text: ", " }` between them. A name that does not resolve becomes `Inline::Text` with its own label — never dropped.

- [ ] **Step 4: Run, regenerate, commit**

```bash
cargo test -p wiki && pnpm wiki:build && cargo test -p wiki derived
git add crates/wiki/src/inline.rs crates/wiki/src/resolver.rs dataset/wiki.json
git commit -m "feat(wiki): achievement text expands to one reference per name"
```

---

### Task 14: `book of virtues synergy` → its description (157 occurrences)

`{{Book of Virtues synergy|description=Spawns wisps in the middle ring…}}` carries its text in a **named** parameter. The value is the sentence the reader needs; the template name is presentation.

- [ ] **Step 1: Confirm the parameter name is always `description`**

Run: `grep -ohiE "\{\{book of virtues synergy\|[a-z ]{0,20}=" dataset/raw/pages/*/*.wikitext | sort | uniq -c`

If a second parameter name shows up, handle both; do not assume from the first three samples.

- [ ] **Step 2: Write the failing test**

```rust
#[test]
fn book_of_virtues_synergy_keeps_its_description() {
    let r = test_resolver();
    let mut d = Diagnostics::default();
    let v = parse_inline("{{Book of Virtues synergy|description=Spawns wisps}}", &r, &mut d);
    assert!(v.iter().any(|i| matches!(
        i, Inline::Text { text, .. } if text.contains("Spawns wisps")
    )));
    assert!(d.unknown_templates.is_empty());
}
```

- [ ] **Step 3: Implement**

This one needs the template's **named** parameters, which `resolve(template, arg)` does not receive. `parse_template_at` in `template.rs` already returns them — find where `inline.rs` calls it and handle this template there, parsing the `description` value through `parse_inline` recursively so links inside it still resolve.

- [ ] **Step 4: Run, regenerate, commit**

```bash
cargo test -p wiki && pnpm wiki:build && cargo test -p wiki derived
git add crates/wiki/src/inline.rs dataset/wiki.json
git commit -m "feat(wiki): the book of virtues synergy template keeps its description"
```

---

### Task 15: `bc` → bag of crafting pickups (59 occurrences)

`{{bc|18|dlc=a+}}`, `{{bc|20}}`, `{{bc|7}}` — a numeric pickup id for the Bag of Crafting, sometimes with an edition.

- [ ] **Step 1: Find out what the numbers are**

Run: `grep -ohE "\{\{bc\|[^}]{0,25}" dataset/raw/pages/*/*.wikitext | sort | uniq -c | sort -rn`

Then check whether those ids match `catalog`'s pickup ids: `grep -n "pickups" crates/wiki/src/resolver.rs`. **If they do not match, do not invent a mapping** — make `bc` an `Ignore` with a comment saying the numbering was not identified, and record it in the spec's "still listed" list. A wrong link is worse than no link.

- [ ] **Step 2: Write the test for whichever answer step 1 gave**

If the ids resolve, assert an `Inline::Ref { target: Target::Pickup { .. } }`. If they do not, assert that the template produces no `unknown_templates` entry and leaves readable text.

- [ ] **Step 3: Implement, run, regenerate, commit**

```bash
cargo test -p wiki && pnpm wiki:build && cargo test -p wiki derived
git add crates/wiki/src/resolver.rs dataset/wiki.json
git commit -m "feat(wiki): the bag of crafting template stops reading as unknown"
```

---

### Task 16: the closing criterion, as a test

**Files:**
- Create: `crates/wiki/tests/templates_understood.rs`

- [ ] **Step 1: Write the test**

```rust
//! The closing criterion from the 2026-09-13 spec: no template appears more than fifty
//! times without the parser understanding it. Below that line they are listed in the spec
//! with a reason, and this test does not police them.

use std::path::PathBuf;

use wiki::{build, Corrections, Raw};

const LIMIT: u32 = 50;

#[test]
fn no_frequent_template_is_still_unknown() {
    // Same three lines as crates/wiki/tests/derived.rs: one way to build the dataset.
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../dataset");
    let raw = Raw::load(&root.join("raw")).expect("dataset/raw/");
    let corrections: Corrections = serde_json::from_str(
        &std::fs::read_to_string(root.join("corrections.json")).expect("corrections.json"),
    )
    .expect("json");
    let dataset = build(&raw, &corrections);

    let meta = dataset.meta();
    // Non-vacuity: a build that parsed nothing has an empty map and would pass trivially.
    assert!(
        meta.counts.items > 700,
        "only {} items built: too few to trust an empty result",
        meta.counts.items
    );
    let frequent: Vec<_> = meta
        .diagnostics
        .unknown_templates
        .iter()
        .filter(|(_, n)| **n > LIMIT)
        .collect();
    assert!(
        frequent.is_empty(),
        "templates above {LIMIT} occurrences are still unknown: {frequent:?}"
    );
}
```

`dataset.meta()` and `meta.counts.items` are the shapes `dataset/wiki.json` serializes (`meta.counts.items` is 719 today). Run `grep -n "pub fn meta\|pub struct Meta\|pub struct Counts" crates/wiki/src/dataset.rs` and adapt the accessor names if they differ.

- [ ] **Step 2: Run it**

Run: `cargo test -p wiki no_frequent_template_is_still_unknown -- --nocapture`
Expected: PASS if tasks 10–15 landed. If one is still listed, it is because its task chose `Ignore` without removing it from `unknown_templates` — fix that, do not raise `LIMIT`.

- [ ] **Step 3: Record what stays out**

In the spec's decision 7, replace the "below the line" list with what `meta.diagnostics.unknownTemplates` actually holds after this phase, each with one line saying why it stays.

- [ ] **Step 4: Final full check and commit**

```bash
pnpm check
cargo test --workspace -- --nocapture 2>&1 | grep -c "^skip:"
git add crates/wiki/tests/templates_understood.rs docs/superpowers/specs/2026-09-13-wiki-infobox-design.md dataset/wiki.json
git commit -m "test(wiki): no template above fifty occurrences is left unknown"
```

---

## Notes for whoever executes this

- **`pnpm wiki:build` after every parser change, or `derived` goes red.** It is the most common way to get a confusing failure in this crate: the test says `dataset/wiki.json does not match build(raw/)` and the cause is always a forgotten rebuild.
- **The tree has other sessions' work in it.** Stage by explicit path, never `git add -A`, and say so if the tree holds changes that are not yours.
- **`samples/packed` is a junction to the installed game.** Without it, dozens of tests skip in silence. `cargo test` hides the output of passing tests, so check skips with `cargo test --workspace -- --nocapture`.
- **A failing test is first a hypothesis of a bug in the code**, not an expectation to correct. Tasks 5 and 7 are both designed to fail informatively the first time they run; read what they print before changing anything.
- **`test_resolver` may have moved by the time you read this.** It is `resolver::fixtures::test_resolver` (a `pub(crate) fn`) as of 2026-09-13, but `crates/wiki/src/lib.rs` already declares `pub mod for_tests`, part of another session's refactor of test fixtures that was in flight while this plan was written. If the `use crate::resolver::fixtures::test_resolver;` lines quoted in the tasks do not compile, run `grep -rn "fn test_resolver" crates/wiki/src/` and use whatever path it reports — do not add a second fixture.
- **`mod infobox` is private.** Everything usable from a `tests/` file comes through the re-exports at the bottom of `crates/wiki/src/lib.rs`. A new public item (`IGNORED_PARAMS`) needs a line there or the integration test will not see it.
