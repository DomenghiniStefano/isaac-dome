# catalog — normalizing the game's XMLs into a queryable catalog (design)

**Date:** 2026-09-03
**Sub-project:** M1 · fourth piece (plan A) and M2 · preparation (plan B)
**Status:** design approved verbally; pending spec review

## Context

`catalog` transforms the game's data files — seven XMLs plus the string table — into a
Rust model queryable by id: items with name, quality, and sprite; characters with
portrait and head; achievements with text and unlock condition; challenges, bosses, and
pools. It receives bytes from whoever calls it (in the app, `unpack::ResourceSet`),
doesn't touch the disk, and feeds `ipc` (Collection, Completion, Unlock screens) and the
unlock graph (M2).

Everything that follows is verified against the real files of this machine's
Repentance+ installation, on 2026-09-03, except where marked *to be verified*.

## Applicable non-negotiable constraints

1. **No game asset in the package.** No XML, no string, no item name embedded:
   everything is read at runtime from the user's copy. Test fixtures are hand-written
   fragments, not copies of the game's files.
2. **No hardcoded count.** How many items, achievements, characters, languages: the
   files say. Tests on real data pin today's observed numbers (909, 637, 41) as
   *expected values of a fixture from a known era*, not as constants in the code.
3. **Degrade, never fail.** Every file is optional. An absent or malformed file empties
   its part of the catalog and produces a typed diagnostic; the rest still gets built.
   Never `panic!` or `unwrap()` on data read from disk.
4. **Pure crate.** No I/O, no dependency on `unpack`, on Tauri, or on the filesystem.
   Bytes in, model out. That's what makes it testable without disk.

## The sources (verified against real data)

Logical paths, without root or archive: `ResourceSet` resolves those, choosing the
winner among archives (the most recent DLC). `catalog` doesn't merge different versions
of the same file: **one winner per file**. To do in plan B: a test verifying that the
winner of `items.xml` contains all the ids of the base one — it doesn't exist today.

| logical path | today's winner | contents |
|---|---|---|
| `items.xml` | `repentance.a` (135,777 bytes) | 909 elements: 425 `passive`, 170 `active`, 126 `familiar`, 188 `trinket` |
| `stringtable.sta` | `afterbirthp.a` (1.36 MB) | **XML**, not binary: 3,151 `<key>` in 14 `<category>`, 8 languages |
| `players.xml` | `repentance.a` | 41 `<player>`; no `hidden` attribute |
| `items_metadata.xml` | `repentance.a` (54,695 bytes) | 721 `<item>` + 188 `<trinket>` = 909, one per item: `quality` (−1..=4) and `tags`. **Quality and tags live here, not in `items.xml`** |
| `achievements.xml` | `repentance.a` (123,288 bytes) | **637** `<achievement>` with contiguous ids 1..=637; `text` literal English (576 with a single quote, 61 with a double quote); a comment with the condition on 283 |
| `itempools.xml` | `repentance.a` (172,669 bytes) | 31 `<Pool Name>` with 2,058 `<Item Id Weight DecreaseBy RemoveOn>`; every id exists in `items.xml`; 24 collectibles are in no pool |
| `challenges.xml` | `repentance.a` | 45 `<challenge>` with a literal `name`, `startingitems`, and **`achievements`** (plural, a comma- or space-separated list, 34 out of 45) |
| `bossportraits.xml` | `repentance.a` | 103 `<boss>` with a literal `name` and `portrait`, root `gfx/ui/boss/`; `achievement` on 27; two portraits (*The Beast*, *Cadavra*) absent from the archives |
| `gfx/ui/coop menu.anm2` | `repentance.a` | `Main` animation, 60 frames: the head crops |

### Shapes the parser must handle

- **Tab-separated attributes**, not just spaces: `<active\t\tid="555"\t...>`. In
  `items.xml` that's 233 elements out of 909. With `quick-xml` it's a non-issue; with a
  text scan it was a silent bug (it counted 678). A unit test pins it anyway.
- **Commented-out elements**: `items.xml` contains `<!-- <active ... /> -->` (two cases:
  `PILLS_HERE_NAME`, `TAROT_CARD_NAME`, lines 44 and 62). A text reader (`grep`, a
  string-based parser) counts them anyway and arrives at 911; a real XML parser ignores
  them, correctly — they're not elements. The spec adopts the parser: 909 is the right
  number.
- **Localization keys**: in the DLC files, `name="#THE_SAD_ONION_NAME"`. In the base
  game (`config.a`) these were literal names, but the base never wins the precedence.
- **Roots declared in the file**: `items.xml` has `gfxroot="gfx/items/"` and the
  subfolder depends on the type (`collectibles/` for passive, active, and familiar;
  `trinkets/` for trinket); `players.xml` has `root`, `portraitroot="gfx/ui/stage/"`,
  `nameimageroot`; `bossportraits.xml` has `root="gfx/ui/boss/"`. They're read from the
  file, never assumed.
- **XML comments as data**: in `achievements.xml` the comment preceding an
  `<achievement>` is the current unlock condition in English
  (`<!-- have 7 or more max red hearts at one time -->`). It's kept as raw text.
  **283 out of 637 achievements** have one: for the other 354, `unlock_condition` is
  `None`, and that's a fact, not a defect.
- **The achievement text is literal, with two delimiters.** `text='You unlocked
  "Magdalene"'` (single quote, 576 cases) and `text="..."` (61 cases): `quick-xml`
  treats them the same; a text scan doesn't.
- **Quality and tags in a separate file.** Repentance's `items.xml` has **no** `quality`
  and no `tags` (0 occurrences): they live in `items_metadata.xml`, with `<item id>` for
  passives, actives, and familiars and `<trinket id>` for trinkets. The `<item>`
  entries all sit under `ItemKind::Passive`; actives and familiars are looked up with
  that same key. Observed `quality`: 0..=4 (the "two −1s" from an earlier measurement
  were actually `craftquality="-1"`, a different attribute). The catalog carries it as
  `Option<i8>`; `None` when the file is missing or lacks the row.
- **Unlock links across multiple files, with different names.** `items.xml`:
  `achievement="N"` (370); `players.xml`: `achievement="N"` (40 out of 41);
  `bossportraits.xml`: `achievement="N"` (27); `challenges.xml`:
  **`achievements="42,34,53"`** — plural and a list. In the model these become
  `unlocked_by: Option<AchievementId>` for the first three and
  `unlocked_by: Vec<AchievementId>` for challenges.
- **String table**: `<languages>` declares `<language id index name>`; index 0 is
  `Key`, then English, Japanese, Korean, Chinese (Simple), Russian, German, Spanish,
  French. Each `<key name>` contains the `<string>` entries **in the order of indices
  1..8**. No Italian.
- **Unlock links**: `achievement="N"` on 370 items out of 909, on 40 characters out of
  41 (not Isaac), and on the challenges. These are graph edges and are kept as
  `Option<AchievementId>`.

## Model and public API

A `crates/catalog` crate. Small types, each with one job; no world-struct.

```rust
pub struct Catalog {
    // Key (ItemKind, ItemId): trinkets and passives share the id space, an ItemId
    // alone isn't enough to find the right item.
    items: BTreeMap<(ItemKind, ItemId), Item>,
    characters: BTreeMap<CharacterId, Character>,
    achievements: BTreeMap<AchievementId, Achievement>,
    challenges: BTreeMap<ChallengeId, Challenge>,
    bosses: BTreeMap<BossId, Boss>,
    pools: Vec<Pool>,
    strings: Option<Strings>,
    diagnostics: Vec<Diagnostic>,
}

impl Catalog {
    /// Builds the catalog, asking `read` for the bytes of each source, by logical path.
    /// Never fails: whatever is missing empties its part and ends up in `diagnostics`.
    pub fn build(read: impl FnMut(&str) -> Option<Vec<u8>>) -> Catalog;

    pub fn item(&self, kind: ItemKind, id: ItemId) -> Option<&Item>;
    pub fn items(&self) -> impl Iterator<Item = &Item>;          // ordered by type, then id
    pub fn character(&self, id: CharacterId) -> Option<&Character>;
    pub fn characters(&self) -> impl Iterator<Item = &Character>;
    pub fn achievement(&self, id: AchievementId) -> Option<&Achievement>;
    pub fn achievements(&self) -> impl Iterator<Item = &Achievement>;
    pub fn challenges(&self) -> impl Iterator<Item = &Challenge>;
    pub fn bosses(&self) -> impl Iterator<Item = &Boss>;
    pub fn pools(&self) -> &[Pool];
    /// The text of a key in the requested language; if missing, in English; if that's
    /// also missing, the key itself. Never `None`: there's always a label.
    pub fn text(&self, t: &Text, lang: Language) -> &str;
    pub fn diagnostics(&self) -> &[Diagnostic];
}
```

The logical names of the sources are a public crate constant (`SOURCES`), so callers and
tests know what will be requested without reading the implementation.

### Types

```rust
pub struct ItemId(u32);  pub struct CharacterId(u32);  pub struct AchievementId(u32);
pub struct ChallengeId(u32);  pub struct BossId(u32);          // newtype, never bare u32

pub enum ItemKind { Passive, Active, Familiar, Trinket }

pub struct Item {
    pub id: ItemId,
    pub kind: ItemKind,
    pub name: Text,
    pub description: Text,
    pub quality: Option<i8>,           // -1..=4, from items_metadata.xml; None if missing
    pub tags: Vec<String>,             // from items_metadata.xml, split on whitespace
    pub sprite: SpriteRef,
    pub unlocked_by: Option<AchievementId>,
    pub pools: Vec<PoolMembership>,    // filled in from itempools.xml
    pub origin: Option<Origin>,
}

pub struct Character {
    pub id: CharacterId,
    pub name: Text,
    pub portrait: SpriteRef,           // portraitroot + portrait
    pub head: Option<SpriteRef>,       // a cell of `coop menu.png`, if the mapping holds
    // The name keys repeat between normal and Tainted (Isaac and Tainted Isaac are
    // both #ISAAC_NAME): what distinguishes them is the `_b` token in the portrait name.
    pub tainted: bool,
    pub unlocked_by: Option<AchievementId>,
}

pub struct Achievement {
    pub id: AchievementId,
    pub text: String,                  // literal English: `You unlocked "Magdalene"`
    pub unlock_condition: Option<String>, // from the XML comment preceding the element
    pub sprite: SpriteRef,
}

pub struct Challenge { pub id: ChallengeId, pub name: String,
                       pub starting_items: Vec<ItemId>,
                       pub unlocked_by: Vec<AchievementId> }   // `achievements="42,34,53"`
pub struct Boss      { pub id: BossId, pub name: String, pub portrait: SpriteRef,
                       pub unlocked_by: Option<AchievementId> }
pub struct Pool      { pub name: String, pub entries: Vec<PoolEntry> }
pub struct PoolEntry { pub item: ItemId, pub weight: f32, pub decrease_by: f32, pub remove_on: f32 }
pub struct PoolMembership { pub pool: String, pub weight: f32 }

/// A localization key or a literal text. The keys' texts live in `Strings`, not here:
/// `Text` is a lightweight reference and `Catalog::text` resolves it. The literal form
/// serves the base game's files: there `name`/`description` aren't keys (`#...`) but
/// direct text, and it's kept as-is.
pub enum Text { Key { key: String }, Literal { text: String } }

pub enum Language { English, Japanese, Korean, ChineseSimplified, Russian, German, Spanish, French }

/// An image: a whole file, or a crop from a sheet.
pub struct SpriteRef { pub path: String, pub rect: Option<Rect> }
pub struct Rect { pub x: u32, pub y: u32, pub w: u32, pub h: u32 }

// Not `Edition`: in `discovery`, `Edition` is already the installed game's edition,
// with different casing on the IPC. Two types with the same name and different
// meanings would confuse imports.
pub enum Origin { Rebirth, Afterbirth, AfterbirthPlus, Repentance }
```

**`Text` and `Strings`.** `Strings` is the loaded string table: a map from key to texts
per language. `Text` only carries the key; names aren't copied into every `Item`. When
the file's attribute doesn't start with `#` (literal names from the base game), `Text`
keeps the literal and `Catalog::text` returns it as-is. Resolution rule: requested
language → English → key. Never `None`.

**`SpriteRef`.** `catalog` doesn't read images: it produces logical paths (and crops)
that the renderer resolves via `ResourceSet`. For items: `gfx/items/collectibles/<gfx>`
or `gfx/items/trinkets/<gfx>`. For heads: `gfx/ui/coop menu.png` with the rectangle of
the matching frame in `coop menu.anm2`.

**`origin`.** The DLC of origin isn't in the XMLs. It's derived from **id thresholds**
in a table declared in the crate (`origin_of(ItemId)`), with tests on known cases: Sad
Onion (1) → Rebirth, Golden Razor (555) → Repentance. It's not a count: it's a stable
historical fact, and it must still be corrected if a real-data test contradicts it.

### Diagnostics

A typed, exhaustive enum, with no paths and no `Debug` strings:

```rust
pub enum Diagnostic {
    SourceMissing { source: Source },                // the file wasn't there
    SourceUnreadable { source: Source },             // XML that quick-xml rejects
    ElementSkipped { source: Source, id: Option<u32>, reason: SkipReason },
    UnresolvedKey { key: String },                   // #A_NEW_KEY, with no string
    UnknownLanguage { name: String },                // a new language in the string table
    HeadSheetUnavailable,                            // anm2 absent or unmappable
}
pub enum SkipReason { MissingId, MalformedId, MissingSprite, MissingName, MalformedList }
pub enum Source { Items, Metadata, Strings, Players, Achievements, ItemPools, Challenges, BossPortraits, CoopMenuAnm2 }
```

A malformed element is skipped with a diagnostic; it doesn't take the whole file down
with it.

## Boundaries

- **`app`** reads the bytes via `ResourceSet` and calls `Catalog::build`; it keeps the
  catalog in the app's state, built once at startup (or when the installation changes).
- **`ipc`** maps `catalog`'s types into view-models under the boundary's rules
  (camelCase, tagged enums with struct variants, opaque ids where needed).
  **`ipc::resources::catalog_peek` is removed** along with its text parser and
  hand-rolled base64: the verification screen switches to the real catalog. `data_url`
  stays until the UI has a better channel for images.
- **The graph (M2)** reads `unlocked_by` and `unlock_condition`; `catalog` doesn't
  interpret them.
- **The Completion matrix** (`ipc::marks::CHARACTERS`, 34 rows) connects to
  `players.xml` (41 rows) by **name-key + Tainted pair**: keys repeat
  (`#ISAAC_NAME` covers both Isaac and Tainted Isaac, ids 0 and 21) and what
  distinguishes them is the `_b` token in the portrait name (`PlayerPortrait_Isaac_b.png`,
  `PlayerPortrait_Lazarus_b_dead.png`), the game's own convention.
  There is no `hidden` attribute; the seven extra rows are non-selectable forms
  (Lazarus 2, Black Judas, The Soul, Esau and their Tainted forms) with their own keys.
  `catalog` exposes all of them; it's the row → (key, Tainted) table in `ipc` that picks
  the 34, and a real-data test verifies that every row finds exactly one character.

## Code rules

In addition to those in `CLAUDE.md`, all of which apply:

- One module per source (`items.rs`, `strings.rs`, `players.rs`, …), each with a single
  crate-public function: `parse(bytes, &mut diagnostics) -> ParsedX`. `Catalog::build`
  composes them. No thousand-line files.
- There's a **single** XML reader, in `xml.rs`: a thin layer over `quick-xml` giving
  elements with attributes and the preceding comment. The per-source modules don't touch
  `quick-xml` directly, so swapping the library is a one-file change.
- No `_ =>` on closed enums: adding an `ItemKind` or a `Source` must break the build.
- Ids as newtypes with `Copy`, `Ord`, `Serialize`; never bare `u32` in public
  signatures.
- Numbers in real-data tests carry a comment with the date and era of the file that
  produced them.

## Tests

### Unit tests (always run, on hand-written fragments)

- Every element type, with attributes separated **by spaces and by tabs**.
- An element missing `id`, with a non-numeric `id`, without `gfx`: skipped with the
  right diagnostic, the rest of the file still read.
- String table: language order from the indices, a key with no string, an unknown
  language.
- `Catalog::text`: requested language → English → key; a literal without `#` returned
  as-is.
- Comment before an achievement: present, absent, two comments (the last one before the
  element wins).
- Absent file and non-XML file: the catalog is still built, with the right diagnostic.
- `origin_of` on known cases and on the thresholds.
- `SpriteRef` for every type: the right subfolder for the item type.

### On real data (via `samples/packed`, skip with a note if missing)

- 909 items, with the breakdown by type (425/170/126/188; era: Repentance+,
  2026-09-03).
- Names: every key resolved (the two elements once believed to be "placeholders",
  `PILLS_HERE_NAME` and `TAROT_CARD_NAME`, are inside an XML comment: they aren't
  elements, they don't count).
- Every `SpriteRef` for items, characters, and bosses is resolvable via `ResourceSet`.
- 41 characters; every row of the Completion matrix finds exactly one.
- 637 achievements with contiguous ids 1..=637; achievement 1: `text` = `You unlocked
  "Magdalene"`, `unlock_condition` = `have 7 or more max red hearts at one time`; 283
  with a condition, 354 without.
- Quality: 909 items out of 909 have one; observed distribution (via the parser, not
  grep) 0:239, 1:197, 2:249, 3:184, 4:40; no −1.
- Pools: 31; every `Item Id` exists in the catalog; 24 collectibles in no pool.
- Challenges: the real file has two shapes the parser must handle that a superficial
  reading didn't reveal: `achievements="490 415"` (**space** separator, challenge 44)
  and `startingitems="-584,34,119,214,569"` (**negative** id, challenges 37 and 38: a
  game flag, not an item — discarded from the list without invalidating the challenge).
- Challenges: 45, ids 1..=45, 34 with at least one achievement, 14 with more than one
  (an earlier measurement said 13 because it only counted comma separators and missed
  challenge 44).
- Bosses: 103; 101 resolvable portraits, *The Beast* and *Cadavra* not (absent from the
  archives: `MissingSprite` isn't the right diagnostic, the file does declare them — it's
  the resolver that can't find them, and it remains a fact to record).
- The winner of `items.xml` contains all the ids of the base `config.a`.
- Heads: if the mapping holds, frames 0, 1, 2 are non-empty crops inside the sheet.
- Cross-check with the save: the catalog's ids are a subset of section 4's slots (733),
  and the gaps are exactly the missing ids.

## Unknowns, and what happens if they don't hold

They come first in plan A as blocking spikes, half a day at most:

1. **Frame → character mapping in `coop menu.anm2`.** Hypothesis: the frame index of
   the `Main` animation matches the character id. Verified by slicing frames 0, 1, 2 and
   looking at them (Isaac, Magdalene, Cain). If it doesn't hold, `head` stays `None`,
   `HeadSheetUnavailable` goes into diagnostics, and the matrix falls back to the name:
   no screen breaks.
2. **The matrix's 34 rows against `players.xml`'s 41.** Hypothesis: matching by name
   key is enough. If a matrix row finds no matching character, or finds two, work
   stops and the file gets inspected: **the test doesn't get adjusted**.
3. ~~**`itempools.xml`** hasn't been read yet~~ — verified on 2026-09-04 against the
   winner: the `<Pool Name><Item Id Weight DecreaseBy RemoveOn>` shape holds (31 pools,
   2,058 entries). No remaining unknown for plan B: all sources are measured.
4. **The gap between the file and the save.** Section 1 of the save declares 642
   achievements, `achievements.xml` has 637; section 4 declares 733 slots, the items
   number 721 across ids up to 732 (11 unused ids between 1 and 732: 43, 61, 235, 587,
   613, 620, 630, 648, 662, 666, 718 — the first two are the commented-out elements).
   The catalog **must not** claim to cover every slot: the cross-check against the save
   only verifies that the catalog's ids are a subset of the slots, and reports the gaps
   as data.

## Plans

- **Plan A (closes M1)**: scaffold and `xml.rs`; `strings.rs`; `items.rs`; `players.rs`
  with the two spikes; `Catalog::build` and diagnostics; wiring into `app`; `ipc` reads
  from the catalog and `catalog_peek` disappears; a verification screen with real names.
- **Plan B (prepares M2)**: `metadata.rs` (quality and tags, which plan A left empty
  because it looked for them in the wrong file); `achievements.rs` with comments and
  links; `itempools.rs` with `Item.pools` filled in; `challenges.rs`; `bossportraits.rs`;
  `origin`; cross-check with the save; sources derived from `SOURCES` instead of
  repeated in `build`; `UnresolvedKey`, already emitted by plan A, extended to the new
  texts.

## Out of scope

Persistence (that's `store`'s job); image reading (paths and crops yes, pixels no); app
interface localization; interpretation of unlock conditions (that's the graph's job);
merging across versions of the same XML; `entities2.xml`, `costumes2.xml`, and the other
files no screen asks for.

## Completion criteria

- `cargo test --workspace`, `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings` clean; the suite stays green for anyone cloning without `samples/`.
- The verification screen shows **real names** next to the sprites, and the item count
  it shows matches `items.xml`.
- `ipc::resources::catalog_peek` no longer exists.
- Every unknown has a written answer in `STATO.md`: verified, or degraded as described
  here.
