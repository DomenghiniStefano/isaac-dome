# Wiki dataset — design

**Date:** 2026-09-05
**Origin:** `docs/superpowers/plans/2026-09-05-b1-sources-effects-report.md` (B1 analysis), with
the decisions made in chat the same day: all entities in the first cycle, Rust, raw and
derived both in the repo, up to the IPC with a verification screen, typed tree.

## What changed during execution

The plan (`docs/superpowers/plans/2026-09-05-wiki-dataset.md`) and its twelve tasks
turned up facts this spec didn't know. Full report:
`docs/superpowers/plans/2026-09-05-wiki-dataset-report.md`.

- **Characters are resolved from our own map, not from the wiki's infoboxes.** The ids
  the `Infobox character` infoboxes declare are unreliable (Isaac carries Keeper's id,
  Magdalene carries Cain's), and four pages (The Forgotten, Tainted Forgotten,
  Jacob & Esau, Tainted Lazarus) use the plural template `Infobox characters`, which the
  enumeration doesn't see. The name → id map lives in `dataset/corrections.json`, key
  `characters`, derived by hand from `players.xml` (40 entries), and it wins over the
  infobox.
- **No `raw/characters.json`.** A consequence of the point above: the tool no longer
  needs to read `players.xml` from the installed game. `wiki-snapshot` depends only on
  `wiki` and `ureq`, not on `catalog`/`unpack`.
- **No `WikiState` in `app`.** `Dataset::embedded()` is already a `OnceLock` inside the
  `wiki` crate; `app` just calls it, with no duplicated managed state.
- **Edition filtering uses the Cargo tables' `dlc` bitmask** (bit 16 = Repentance+), not
  a manual correction. *Tonsil* isn't a wiki error: the page has two infoboxes, the
  current trinket 97 and the Afterbirth+ collectible 474 (replaced by Broken Glass
  Cannon), and the Cargo table confirms it (`dlc = 4`).
- **Each infobox is sorted by its own `InfoboxKind`**, not by the page's type: a single
  page can produce entries of different types (Tonsil yields trinket 97 and discards
  collectible 474, out of edition).
- **Translation subpages (`/de`, `/fr`, …) are skipped** in `fetch`: four boss pages had
  snuck in as translations, not original content.
- **`wiki.json` is embedded compressed**, not with `include_str!`. The `wiki` crate's
  `build.rs` deflates `dataset/wiki.json` (22 MB) into `$OUT_DIR` with `miniz_oxide`
  (1.6 MB compressed); the embedding sits behind the cargo feature `embedded` (on by
  default), so `wiki-snapshot` compiles without carrying the dataset along.
- **A wikilink to an entity's page becomes a `Ref`**, not just the `{{i}}`-style
  templates: `by_page_title` also resolves `[[Page|label]]`.
- **An id starting with a number followed by a note** (`349<br>{{plat|PS4}} 340`,
  common in achievements with platform-specific ids) is read as the leading number, not
  discarded as non-numeric.
- **A `Pending` set tracks pages without a revision in truncated batches**: the
  MediaWiki generator re-lists already-delivered pages even in the last batch, without
  `revisions`; `fetch` distinguishes "still to be delivered" from "never arrived"
  instead of failing on the first batch that revisits them.

The rest of the spec (§2 the tree's shape, §4 the tool, §6 the tests) remains valid; §1,
§3, and §5 are updated below wherever the code contradicts them (`raw/characters.json`,
`WikiState`, and `include_str!` are all gone).

## What we're building

A subsystem that brings into the app the text sections of the English wiki on wiki.gg
(`bindingofisaacrebirth.wiki.gg`, CC BY-SA 4.0) for items, trinkets, achievements,
bosses, challenges, and characters, as a **typed tree with entity references resolved to
our own ids**. The text is downloaded and transformed **at build time**, on the
developer's machine; the package ships the derived data; the app never talks to the
wiki.

Four pieces:

| piece | what | depends on |
|---|---|---|
| `crates/wiki` | pure crate: dataset types (serde), wikitext parser, reference resolver, `build(raw) -> Dataset`, `Dataset::embedded()` | `serde`, `serde_json` |
| `crates/wiki-snapshot` | tool binary: `fetch` (network → `dataset/raw/`), `build` (`raw/` → `dataset/wiki.json`) | `wiki`, `ureq` |
| `dataset/` | `raw/` (raw snapshot), `wiki.json` (derived), `corrections.json`, `ATTRIBUTION.md` | — |
| `ipc::wiki` + `app` + UI | `WikiInfo`, `wiki_entry` command, TypeScript mirror, panel in the verification screen | `wiki` |

`app` depends on `wiki`, never on `wiki-snapshot`. `wiki` only reads `raw/` and knows
nothing about `catalog` or the game archives.

## 1. The dataset on disk

```
dataset/
  raw/
    index.json                      # by title: kind, pageid, revid, timestamp
    pages/<kind>/<Title>.wikitext   # kind ∈ collectible trinket achievement boss challenge character
    cargo/<table>.json              # table rows, as returned by cargoquery
  corrections.json                  # our own fixes and the character map, applied during resolution
  wiki.json                         # the derived data, embedded in the binary
  ATTRIBUTION.md                    # wiki, URL, license, snapshot date; goes into the package
```

**`raw/`.** One file per page, so a commit's diff shows which page changed. The title in
the file name keeps spaces as `_` and replaces characters Windows disallows
(`?:*"<>|/\`) with `%XX`; `index.json` keeps the real title. The Cargo tables
downloaded: `collectible`, `trinket`, `achievement`, `entity`, `challenge`, `player`,
`stage`, `transformation`, `pickup`, `version`. No others (YAGNI: `catalog` already has
quotes, quality, tags, pools, prices).

**Pages by type** are enumerated with `generator=embeddedin` against the infobox
templates: `Infobox collectible` (covering passives and actives), `Infobox trinket`,
`Infobox boss`, `Infobox challenge`, `Infobox character`, and for achievements the
collection pages `Achievements/<edition> <n>`, which embed `Infobox achievement` (one
page contains many infoboxes: the parser extracts one per `Entry`).

**`wiki.json`.** A single file, `serde_json` in readable (pretty) form, ordered maps
(`BTreeMap`) so that two `build` runs on the same `raw/` produce identical bytes:

```
{
  "meta": {
    "schemaVersion": 1,
    "snapshotAt": "2026-09-05T14:20:41Z",     // the most recent timestamp in index.json
    "maxRevid": 267225,
    "lastKnownPatch": { "number": "v1.9.7.17", "date": "2026-04-20" },   // from cargo/version
    "source": { "name": "The Binding of Isaac: Rebirth Wiki", "url": "https://bindingofisaacrebirth.wiki.gg", "license": "CC BY-SA 4.0" },
    "counts": { "items": 719, "trinkets": 188, "achievements": 641, "bosses": 103, "challenges": 45, "characters": 40 },
    "diagnostics": {
      "unresolved": { "item": 3, "character": 1, ... },   // unresolved Refs, by template type
      "unknownTemplates": { "m": 171, "machine": 32, ... },
      "discardedSections": { "Trivia": 617, "Gallery": 191, ... },
      "pagesWithoutId": 2
    }
  },
  "items":        { "<id>": Entry },      // items.xml id
  "trinkets":     { "<id>": Entry },
  "achievements": { "<id>": Entry },      // achievements.xml id (verified: 62 = Epic Fetus)
  "bosses":       { "<id>.<variant>.<subtype>": Entry },
  "challenges":   { "<number>": Entry },
  "characters":   { "<id>": Entry }       // players.xml id
}
```

**`corrections.json`.** `{ "pageId": { "collectible": { "Broken page name": 999 } } }`:
per table, the page title and the right id, wins over the Cargo table — empty today
(`{"pageId": {}}`), because no real case still needs it (*Tonsil*, the initial
candidate, turned out not to be a wiki error: two infoboxes, filtered by the `dlc`
bitmask, §3). It also carries `"characters": { "Isaac": 0, … }`, our own name → id map
from `players.xml`, which wins over the infoboxes (§3). It only grows when the wiki is
actually wrong.

**Embedding.** `wiki::Dataset::embedded()` reads `dataset/wiki.json` (22 MB, pretty,
stays that way in the repo for a readable diff) compressed at build time by
`crates/wiki/build.rs` (`miniz_oxide::deflate::compress_to_vec`, ~1.6 MB in the binary)
behind the cargo feature `embedded` (on by default): `include_bytes!` on the compressed
blob, `inflate`, then `from_json`, all once inside a `OnceLock`. It returns
`Result<&'static Dataset, DatasetError>` with `SchemaMismatch { found, expected }` or
`Malformed`. No path at runtime. `wiki-snapshot` depends on `wiki` with
`default-features = false`: the tool doesn't need the embedded dataset. When the
GitHub-based update arrives, a newer and valid file in the app's data folder will win
over the embedded one; that's not part of this cycle.

## 2. The shape of the tree

Types in `crates/wiki/src/model.rs`, all `Serialize + Deserialize`,
`rename_all = "camelCase"`; data-carrying enums are tagged with `tag = "kind"` and
`rename_all_fields = "camelCase"`; data-less enums are bare strings (repo convention).

```rust
pub struct Entry { pub title: String, pub revid: u64, pub infobox: Infobox, pub sections: Vec<Section> }

pub struct Section { pub kind: SectionKind, pub blocks: Vec<Block> }

pub enum SectionKind {            // bare string
    Effects, Notes, Synergies, Interactions, Bugs, Behavior, ChampionVersions,
    DamageScaling, Strategies, Difficulty, Reward, Unlockable,
}

pub enum Block {                  // tag = "kind"
    Paragraph { inline: Vec<Inline> },
    List { ordered: bool, items: Vec<ListItem> },
    Table { header: Vec<Vec<Inline>>, rows: Vec<Vec<Vec<Inline>>> },
    Heading { level: u8, inline: Vec<Inline> },
}
pub struct ListItem { pub inline: Vec<Inline>, pub children: Vec<Block> }

pub enum Inline {                 // tag = "kind"
    Text { text: String, style: Style },          // Style: Plain | Bold | Italic, bare string
    Ref { target: Target, label: String },
    Concept { page: String, label: String },      // [[Shot Speed]], or an unresolved template
    Edition { only: Vec<Dlc>, inline: Vec<Inline> },
}

pub enum Target {                 // tag = "kind"
    Item { id: u32 }, Trinket { id: u32 }, Character { id: u32 }, Achievement { id: u32 },
    Challenge { number: u32 }, Entity { id: u32, variant: u32, subtype: u32 },
    Transformation { id: u32 }, Stage { name: String }, Room { name: String },
    Pickup { name: String },
}

pub enum Dlc { Rebirth, Afterbirth, AfterbirthPlus, Repentance, RepentancePlus }  // bare string

pub enum Infobox {                // tag = "kind"
    Item, Trinket,                                        // nothing: catalog already knows it all
    Achievement { description: String, requirements: Vec<Inline>, unlocks: Option<Target> },
    Boss { base_hp: Option<u32>, environment: Vec<Inline>, pool: Vec<Inline>, unlocked_by: Option<Target> },
    Challenge { blindfolded: bool, has_shops: bool, has_treasure_rooms: bool, items: Vec<Inline>,
                trinkets: Vec<Inline>, pickups: Vec<Inline>, health: Vec<Inline>, curse: Vec<Inline>,
                goal: Vec<Inline>, unlocks: Option<Target>, unlocked_by: Option<Target> },
    Character { health: Vec<Inline>, damage: String, range: String, speed: String, luck: String,
                shot_speed: String, pickups: Vec<Inline>, collectibles: Vec<Inline>, unlocked_by: Option<Target> },
}
```

Choices within the shape:

- **Normalized sections.** A map from the wiki's titles (compared case-insensitively,
  spaces collapsed, `[[…]]` links stripped): `Effects`/`Effect`/`No Effect` →
  `Effects`; `Notes` → `Notes`; `Synergies` → `Synergies`;
  `Interactions`/`Item Interactions` → `Interactions`; `Bugs` → `Bugs`; `Behavior` →
  `Behavior`; `Champion Versions` → `ChampionVersions`; `Damage Scaling` →
  `DamageScaling`; `Strategies`/`Strategy`/`Tips` → `Strategies`; `Difficulty` →
  `Difficulty`; `Reward` → `Reward`; `Unlockable Achievements`/`Unlockable Starting
  Items` → `Unlockable`. Everything else (Trivia, Gallery, In-game Footage, References,
  unlisted Notes…) is discarded and counted in `discardedSections` under its original
  title. Subsections (`===`, `====`) stay inside the parent section as `Heading` at
  level 3 and 4.
- **`Ref` only when resolved.** A template that finds no id becomes
  `Concept { page: X, label: X }`: the text still reads whole, only the link is lost,
  and `unresolved` counts it.
- **`Edition` is a node.** `{{dlc+|r}} … {{dlc-}}` wraps its children; `{{dlc|r}}`
  without a closing tag wraps the rest of the line (of the list item or paragraph).
  `{{dlcalt|a|r=b}}` becomes two consecutive `Edition` nodes. The codes: `a` Afterbirth,
  `a+` Afterbirth+, `r` Repentance, `r+` Repentance+, `n` Rebirth; combinations like
  `a+r` become multiple values in `only`.
- **Bold and italic** (`'''`, `''`) become `Style` on the `Text`; the rest of the inline
  syntax (`<br>`, `<u>`, `&times;`, `<ref>…</ref>`) degrades to text or is discarded.
- **`Infobox::Item` and `Trinket` are empty** on purpose: the infobox's `id` only serves
  to place the `Entry` in the right map.

## 3. Reference resolution

`wiki::Resolver` is built from `raw/cargo/*.json` and `corrections.json` (which also
carries the character map, key `characters`). Normalized keys: lowercase, spaces
collapsed, `&` and `and` equivalent. Cargo rows and infoboxes outside the current
edition (`dlc` bitmask, bit 16 = Repentance+) don't enter the resolver.

| template | table | key | target |
|---|---|---|---|
| `{{i\|X}}` `{{I\|X}}` | `collectible` | `alias` **and** `_pageName` | `Item { id }` |
| `{{t\|X}}` | `trinket` | alias, `_pageName` | `Trinket { id }` |
| `{{a\|X}}` `{{achievement\|X}}` | `achievement` | `name`, `alias` | `Achievement { id }` |
| `{{chal\|X}}` | `challenge` | alias, `number` | `Challenge { number }` |
| `{{e\|X}}` `{{E\|X}}` | `entity` | alias, `_pageName` | `Entity { id, variant, subtype }` |
| `{{tf\|X}}` | `transformation` | alias | `Transformation { id }` |
| `{{p\|X}}` | `pickup` | alias | `Pickup { name }` |
| `{{c\|X}}` `{{C\|X}}` | `corrections.json` key `characters` | English name | `Character { id }` |
| `{{s\|X}}` `{{S\|X}}` | — | — | `Stage { name }` |
| `{{r\|X}}` `{{R\|X}}` | — | — | `Room { name }` |
| `{{floor\|X}}` `{{room\|X}}` | — | — | `Stage`/`Room` |
| others | — | — | `Concept { page: first argument }`, counted in `unknownTemplates` |

The second positional argument or `text=` of these templates is the label to display
when present (`{{i|Little Baggy|Baggy}}`); otherwise the label is the name.
`{{cit|…}}`, `{{nav}}`, `{{#ev:…}}`, `{{disambig msg}}`, `{{header characters}}`,
`{{storage page}}`, `{{unlockable}}` are discarded without counting as unknown: they're
known layout templates.

The character map lives in `dataset/corrections.json` (key `characters`, 40 names →
`players.xml` ids, hand-written): the ids in the `Infobox character` infoboxes aren't
reliable (Isaac has Keeper's id on the wiki page, Magdalene has Cain's), and four pages
use the plural template `Infobox characters`, which the enumeration doesn't enumerate.
Our map wins over the infobox. This way `wiki` and `wiki-snapshot` never touch the
game's archives: no dependency on `catalog`/`unpack`.

A wikilink `[[Page|label]]` whose page resolves to a known entity also becomes a `Ref`,
not just the templates in the table above.

The `unlocked_by` and `unlocks` of the infoboxes are achievement or page names: they're
resolved via `achievement` by name, then by the page title of what they unlock.

## 4. The `wiki-snapshot` tool

`crates/wiki-snapshot`, a binary excluded from the app, depending only on `wiki` and
`ureq`. The only place in the repo that talks to the network.

- `wiki-snapshot fetch [--out dataset/raw]`. Sequential, with
  `User-Agent: IsaacDome-snapshot/<crate version> (+https://github.com/DomenghiniStefano/isaac-dome)`,
  `maxlag=5`, a 250 ms pause between calls, a single retry on 429/5xx/timeout after
  5 s. Enumeration via `generator=embeddedin` (`geilimit=50`, `prop=revisions`,
  `rvprop=content|ids|timestamp`, `rvslots=main`, `formatversion=2`), tables via
  `action=cargoquery` (`limit=500`, paginated with `offset`). Skips titles that are
  translation subpages (`/de`, `/fr`, …: a suffix of exactly two lowercase letters).
  Writes a file only if the content changed (byte-for-byte comparison): the commit diff
  is the real difference. Removes from `raw/` pages the wiki no longer has and prints
  them. Around forty calls in total.
- `wiki-snapshot build [--raw dataset/raw] [--out dataset/wiki.json]`. Offline. Calls
  `wiki::build(raw)` and writes it; prints the `meta` (counts, diagnostics) in readable
  form. Exits with a non-zero code if `raw/` is missing or unreadable, never over one
  odd page.

The pagination logic (`continue`), idempotent writing, and URL composition live in pure
functions with tests against fake responses; `ureq` is used in a single thin module.

Root-level scripts: `pnpm wiki:fetch` → `cargo run -p wiki-snapshot -- fetch`,
`pnpm wiki:build` → `cargo run -p wiki-snapshot -- build`.

## 5. The IPC boundary and the verification screen

**`wiki`:** `Dataset::embedded()`, `Dataset::entry(&Target) -> Option<&Entry>` (for
`Item`, `Trinket`, `Achievement`, `Challenge`, `Character`, `Entity` when it's a boss;
`None` for the other `Target`s), `Dataset::meta()`.

**`ipc::wiki`:**

```rust
pub enum WikiInfo {               // tag = "kind"
    Loaded { snapshot_at: String, last_known_patch: Option<PatchView>, counts: Counts,
             unresolved: u32, unknown_templates: u32, game_newer_than_snapshot: Option<bool> },
    Missing { reason: WikiMissingReason },   // SchemaMismatch | Malformed, bare string
}
pub fn wiki_info(dataset: Result<&Dataset, &DatasetError>, game_updated_unix: Option<u64>) -> WikiInfo
pub use wiki::{Entry, Section, SectionKind, Block, ListItem, Inline, Style, Target, Dlc, Infobox};
```

`game_newer_than_snapshot` compares `LastUpdated` from `appmanifest_250900.acf` with
`snapshotAt`; `discovery::Manifest` gains `last_updated: Option<u64>` and `GameInstall`
exposes it as `updated_unix`. `None` when the game isn't from Steam or the field is
missing.

**`app`:** no new state — `Dataset::embedded()` is already a `OnceLock` inside `wiki`,
`app` just calls it wherever needed. `WikiInfo` enters the `extraction_report` payload
(field `wiki`). Command `wiki_entry(target: Target) -> Result<Option<Entry>, IpcError>`,
with the variant `IpcError::WikiUnavailable` when the dataset failed to load; a `Target`
with no page is `Ok(None)`.

**Frontend:** types mirrored by hand in `ui/src/lib/ipc/types.ts` (`Entry`, `Block`,
`Inline`, `Target`, `Infobox`, `WikiInfo`, and `as const` constants for the bare-string
enums), wrapper `wikiEntry(target)` in `ui/src/lib/ipc/wiki.ts`, command in
`constants/commands.ts`. In `App.vue` a "Wiki" section: the meta (date, patch, counts,
unresolved, unknown templates, freshness), a field for an item id, and the `Entry`
rendered by a recursive `WikiBlocks.vue` component (nested lists, tables, headings,
`Edition` with an edition label) in which a `Ref` is a link that calls `wikiEntry` with
that target. This is verification, not design.

## 6. The tests

- **Parser** (`wiki`): one unit per construct, expected value derived from the wikitext
  syntax and not from the code: a two-level list, a list inside a list item, a
  `{| … |}` table with `!` and `|-`, bold and italic, `{{dlc+|r}}…{{dlc-}}` nested
  inside a list item, `{{dlc|a}}` at the start of a line, a template with a label, an
  unknown template, a discarded section, a discarded `<ref>`, an infobox with
  multi-line fields, a page with multiple achievement infoboxes. Then a **pass over
  `raw/`** with invariants: no panics, every page with an infobox and an id produces an
  `Entry`, per-type counts match `index.json` minus `pagesWithoutId`; and **real fixed
  points**: Binge Eater has exactly eight item `Ref`s in the nested list of its Notes;
  False PHD has a `Table` with 19 body rows under Effects; Hush has level-3 and level-4
  `Heading`s under `Behavior`; achievement 62 has `requirements` with a `Ref` to
  `Challenge { number: 19 }` and `unlocks = Item { id: 168 }` (Epic Fetus in
  `items.xml`, verified on 2026-09-05).
- **Resolver**: "Breakfast" → `Item 25`, "Jacob and Esau" and "Jacob & Esau" → the same
  character, "Tonsil" → `Item 97` via correction, "Mom" → `Entity`, a made-up alias →
  `Concept` with the counter incremented to 1.
- **Derived data**: `wiki.json` byte-for-byte equal to `build(raw/)`; the test fails
  with a message telling you to rerun `pnpm wiki:build`.
- **Tool**: multi-page `continue`, write-only-if-changed, removal of vanished pages, URL
  composition, all against fake responses. No network call in tests.
- **IPC**: JSON shape pinned for `Inline`, `Block`, `Target`, `Infobox`, `WikiInfo`,
  `SectionKind`, `Dlc`; `wiki_info` with and without a dataset, with and without
  `LastUpdated`.
- **Repo rules**: `cargo test --workspace`, `fmt`, `clippy -D warnings`,
  `pnpm typecheck`, `lint`, `format:check`, `scan` clean. `raw/` is in the repo: no
  skipping.

## Out of this cycle

- Runtime dataset updates from GitHub and the setting that enables them.
- The "open on the wiki" link in the detail view: that's design's job.
- Italian: EID stays blocked by licensing (report B1).
- Cargo tables beyond the ten listed; images of any kind.
- A `Target` toward non-boss monsters: `Entity` also resolves monsters, but `wiki.json`
  only keeps pages with `Infobox boss`; `entry()` returns `None` for the others.
