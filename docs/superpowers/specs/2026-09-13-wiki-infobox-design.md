# `wiki` — the infobox, and the templates we already downloaded (design)

**Date:** 2026-09-13
**Milestone:** none of M1–M5 directly. It serves the wiki dataset, closed and merged on
2026-09-06, and it is the first of three sub-projects that came out of the 2026-09-13
brainstorm; it changes the live IPC contract, so it is cheaper now than after another screen
is built on the current shape.
**Branch:** `feature/wiki-infobox`, cut from `develop` (one branch per sub-project, decided
2026-09-11)
**Depends on:** the wiki dataset (`crates/wiki`, `crates/wiki-snapshot`, `dataset/`, merged
2026-09-06); `crates/catalog`'s `items_metadata.xml` reader, used as the independent source in
two of the tests
**Owner's request (2026-09-13):** "essere sicuro di aver preso tutte le info dalla wiki, non
solo il testo, ma anche tutte le info da [la scatola] nella zona destra"
**Status:** three choices below are marked **(owner)** — they were answered in the brainstorm.
Everything else is the author's and marked **(delegated)**, so the first look can overturn it
cheaply.

## What this sub-project is

`crates/wiki` parses every page's infobox and then throws most of it away.

Two of the six variants of `Infobox` are **unit variants** — they carry no data at all:

```rust
pub enum Infobox {
    Item,        // no fields
    Trinket,     // no fields
    Achievement { … }, Boss { … }, Challenge { … }, Character { … },
}
```

`infobox_from()` matches `InfoboxKind::Collectible => Infobox::Item` and drops the parsed
parameters on the floor. Measured on the committed `dataset/wiki.json`:

| kind | entries | infobox |
|---|---|---|
| items | 719 | **all empty** |
| trinkets | 188 | **all empty** |
| achievements / bosses / challenges / characters | 820 | carry fields |

**907 entries out of 1727 reach the frontend with nothing from the right-hand box.**

The parameters are not missing from disk. They were downloaded on 2026-09-04 and are sitting
in `dataset/raw/pages/`. Counted across the committed wikitext:

- **collectible** (720 pages): `id` 724, `description` 723, `quote` 722, `quality` 719,
  `tags` 716, `dlc` 383, `unlocked by` 277, `recharge` 171, `devil price` 56, `pool` 45,
  `shop price` 16
- **trinket** (187 pages): `quote`/`id`/`description` 184, `dlc` 124, `unlocked by` 95,
  `tags` 44, `pool` 7
- and on the four kinds that *are* parsed: challenge `description` 17 and `used_character` 14,
  character `tears` 12 and `parent` 2, boss `dlc` 32 / `variant` 26 / `stage hp` 2,
  achievement `dlc` 7

A second loss has the same shape. `dataset/wiki.json`'s `meta.diagnostics.unknownTemplates`
lists 25 templates the inline parser does not understand — `m` 357,
`transformation contribution` 186, `book of virtues synergy` 157, `achievement text` 127,
`ip` 110, `bc` 59, `machine` 52, `cu` 41, `hearts` 33 and the rest. Wherever one of those
appears, the sentence reaches the frontend mangled.

So this sub-project is one question asked twice: **stop discarding input we already hold.**
It needs no new download. `pnpm wiki:fetch` is untouched; the work closes with
`pnpm wiki:build`, offline.

## Applicable constraints

1. **The IPC contract is live.** `Entry` and `Infobox` cross the boundary and are hand-mirrored
   in `ui/src/lib/ipc/types.ts`. The change is handed on in `DESIGN-BRIEF.md`, not merely
   committed.
2. **`wiki` is a pure crate.** No I/O beyond `dataset/raw/`, no network. The tool
   `wiki-snapshot` is the only thing that talks to the wiki, and it is not touched here.
3. **The three files move together.** The `derived` test enforces
   `wiki.json == build(raw, corrections)`, so `dataset/wiki.json` is regenerated and committed
   in the same commit as the parser change.
4. **Degrade, never fail.** A missing parameter is an empty value, never an error and never a
   dropped page. A template we still do not understand leaves its text in place and is counted.
5. **Never name from a guess.** Where the wiki's vocabulary is the game's and open (`tags`), we
   keep it open. Where a reading is a hypothesis (the Cargo `dlc` integer, decision 5), it is
   verified against an independent source before it is used, or it is not used.
6. **Exhaustiveness is mandatory.** No `_ =>` arm on `Infobox` or on any enum this adds.

## Decision 1 — three facts rise to `Entry` (owner)

`description`, `dlc` and `unlocked by` are not specific to a kind. The counts say so —
`description` on 723 collectibles, 184 trinkets, 17 challenges, 9 achievements; `dlc` on
383/124/32/20/7; `unlocked by` on 277/95/33/29/29 — and so does the meaning: the edition a
thing exists in is a property of the thing, not of its information box.

```rust
pub struct Entry {
    pub title: String,
    pub revid: u64,
    /// The infobox's summary line. Plain text for achievements, wikitext elsewhere: both
    /// arrive as inline so the frontend has one shape and no switch on the kind.
    pub description: Vec<Inline>,
    /// The edition codes the infobox declares, parsed. Empty when the parameter is absent.
    /// It is deliberately NOT called "introduced in" or "exists in": which of the two it
    /// means is unmeasured (see decision 5), and a name would be a guess.
    pub dlc: Vec<Dlc>,
    /// What the wiki states has to be unlocked first. `None` means "the wiki does not state
    /// one", NEVER "it is free from the start": that answer belongs to `catalog` and `graph`.
    pub unlocked_by: Option<Target>,
    pub infobox: Infobox,
    pub sections: Vec<Section>,
}
```

The alternative — repeating the three fields in all six variants — was rejected in the
brainstorm: it is the same data written six times, and it forces the frontend to switch on the
kind to read a description.

**`unlocks` does not rise.** Only achievements and challenges have it, and it points the
opposite way from `unlocked_by`. Hoisting both into one neighbourhood is a trap worth avoiding.

## Decision 2 — the variants keep only what is theirs (owner)

```rust
pub enum Infobox {
    Item {
        quote: String,
        activated: bool,
        quality: Option<i8>,
        tags: Vec<String>,
        recharge: Vec<Inline>,
        devil_price: Vec<Inline>,
        shop_price: Vec<Inline>,
        pools: Vec<Inline>,
    },
    Trinket { quote: String, tags: Vec<String>, pools: Vec<Inline> },
    Achievement { requirements: Vec<Inline>, unlocks: Option<Target> },
    Boss {
        base_hp: Option<u32>,
        stage_hp: Vec<Inline>,
        variant: Option<u32>,
        environment: Vec<Inline>,
        pool: Vec<Inline>,
    },
    Challenge {
        blindfolded: bool, has_shops: bool, has_treasure_rooms: bool,
        items: Vec<Inline>, trinkets: Vec<Inline>, pickups: Vec<Inline>,
        health: Vec<Inline>, curse: Vec<Inline>, goal: Vec<Inline>,
        character: Option<Target>, unlocks: Option<Target>,
    },
    Character {
        health: Vec<Inline>, damage: String, tears: String, range: String,
        speed: String, luck: String, shot_speed: String,
        pickups: Vec<Inline>, collectibles: Vec<Inline>, parent: Option<Target>,
    },
}
```

`activated` comes from the template name, which already distinguishes them
(`infobox activated collectible` vs `infobox passive collectible`) — `InfoboxKind::of()` merges
the two today and must stop merging them.

`quote` stays a `String` on the two item variants: it is the pickup quote, and it is the same
string as `items.xml`'s `description` attribute. That correspondence is what gives the future
compare-button something to compare, and decision 6's test is what confirms it.

## Decision 3 — `tags` is `Vec<String>`, not an enum (delegated)

The tag vocabulary is the game's, it is open, and `catalog::Metadata` already models it as
`Vec<String>`. Deriving a closed enum from 716 lines of wiki would be naming from a guess, and
a closed enum is exactly what breaks when the game adds a tag.

## Decision 4 — `recharge` and the two prices are `Vec<Inline>`, not numbers (delegated)

The 171 `recharge` values are not integers. Censused on the committed wikitext, they are room
counts (`4`, `6`, `12`), `unlimited`, `one time`, seconds (`4s`, `3s`, `10s`), and per-edition
forms like `{{dlcalt|6|r=4}}`. An `Option<u32>` would silently discard about a third of them.

The same census killed the first draft of this spec, which had `devil_price` and `shop_price`
as `Option<u32>`. They are not numbers either: of 56 `devil price` values, 20 are a bare `2`
but the rest are per-edition (`{{dlcalt|1|r=2}}` 15, `{{dlcalt|1|r+=0}}` 9, …). One
`shop price` reads `{{dlcalt|15|r=1}}0` — a template followed by a stray digit, a wiki-side
authoring quirk that a numeric parse would turn into a confident wrong answer, and that inline
parsing carries through unharmed.

`Vec<Inline>` is what the crate already uses for free wiki values, and `{{dlc|…}}` inside one
becomes `Inline::Edition` for free.

**Open for the plan:** `dlcalt` is not in the `unknownTemplates` list, which means either the
inline parser already handles it or it never reached the parser because `recharge` was never
parsed. The plan checks which, and if it is the second, `dlcalt` joins phase 2's list.

## Decision 5 — the `dlc` bitmask is a hypothesis until `catalog` confirms it (delegated)

The Cargo tables carry `dlc` as an **integer**, not as the letter code the wikitext uses.
Read as a five-bit mask over the editions, 31 is all five and 24 = `0b11000` is Repentance and
Repentance+ only. Five items, checked against `catalog::origin`'s verified id boundaries:

| item | id | origin (game) | cargo `dlc` | wikitext `dlc` |
|---|---|---|---|---|
| 1up! | 11 | Rebirth | 31 | *absent* |
| Brimstone | 118 | Rebirth | 31 | *absent* |
| **Blue Cap** | **342** | **Afterbirth** (its first item) | **31** | ***absent*** |
| Mucormycosis | 553 | Repentance (its first item) | 24 | `r` |
| 120 Volt | 559 | Repentance | 24 | `r` |

**Blue Cap is the counter-example, and it was found before a line of code was written.** It is
the first Afterbirth collectible — it does not exist in vanilla Rebirth — yet its mask says all
five editions and its wikitext parameter is absent. So the mask is *not* "the editions it
exists in", or the wiki is lax about the Afterbirth era, and the two cannot be told apart from
here. Meanwhile 96 pages carry `dlc = a` and 111 carry `a+`, so the parameter is not simply
unused for those editions.

Therefore: **phase 1 takes `dlc` from the wikitext parameter**, through the existing and tested
`Dlc::from_code`, and the field is documented as "the codes the infobox declares" — not as
"introduced in" and not as "exists in". Naming it would be the guess the repo keeps paying for.

One shape the parser must handle: the parameter is not always a single code. One page carries
`a+nr`, three codes concatenated. `Dlc::from_code` takes one code, so the plan adds a
`Dlc::parse_codes` that splits the string and **counts the leftovers in `Diagnostics`** rather
than dropping them, because a code we cannot read is exactly the kind of thing that should
surface in `meta` instead of vanishing.

The bitmask is measured over all 733 collectibles against `catalog::Origin` — an independent
source, computed from the user's own `items.xml`, which is the point — and the result is
**reported, not asserted**: the Blue Cap row above says the naive reading already fails, so a
passing assertion would mean the test is wrong. The check has a second asymmetry it must not
paper over: `catalog::Origin` has **four** variants and `wiki::Dlc` has **five**. It is not a
bijection. The comparison covers the four editions both sides name, and the count of items
whose wiki `dlc` includes `RepentancePlus` is printed alongside.

The mask becomes a second source only in a later commit, only if that measurement explains
Blue Cap. Until then it is a number we store nowhere.

## Decision 6 — where both sources speak, the test counts the disagreements (delegated)

Three fields exist on both sides: the wiki says them, and the game says them.

| wiki | game | what it settles |
|---|---|---|
| `Infobox::Item.quality` | `items_metadata.xml` `quality` | the field is read correctly |
| `Infobox::Item.tags` | `items_metadata.xml` `tags` | same, on a multi-valued field |
| `Infobox::Item.quote` | `items.xml` `description` | **that `quote` is the pickup quote at all** |

The third is not decoration. Decision 2 asserts that the wiki's `quote` and the game's
`description` are the same string; that assertion is load-bearing for the future
compare-button, and this is the test that earns it rather than assuming it.

The test does **not** fail on a disagreement. A disagreement is the very thing the
compare-button exists to show — the dataset's snapshot is from 2026-09-04 and its
`lastKnownPatch` is **v1.9.7.17, dated 2026-04-20**, while the user's install is whatever they
have. It fails only above a rate, because a systematic disagreement means we are reading the
field wrong, not that the wiki is five months old.

The rate, so the plan does not have to invent one: **5% of the items where both sides speak**,
per field, counted over `samples/packed`. Five months of patches move a handful of qualities
and a few pickup quotes; they do not move one item in twenty. A plan that finds the real rate
sits near the threshold should say so and argue the number up or down with the measurement in
hand — not silently widen it.

The same test prints the disagreeing ids, capped like the other diagnostic dumps
(`DIAGNOSTIC_ROWS`), so the output is the raw material for sub-project 2.

## Decision 7 — phase 2: the templates, with a closing criterion (owner)

The 25 entries of `unknownTemplates` are taught to the inline parser, ordered by occurrence.
Closing criterion, measurable from `meta.diagnostics` and therefore checkable by a test:

> every template with more than 50 occurrences is gone from `unknownTemplates`, and each one
> still listed is named in this spec with the reason it stays out.

Above the line today: `m` 357, `transformation contribution` 186, `book of virtues synergy` 157,
`achievement text` 127, `ip` 110, `bc` 59, `machine` 52. Below it, and therefore optional:
`cu` 41, `hearts` 33, `book of belial synergy` 32, `curse` 22, `plat` 21, `mode` 20, `heart` 17,
`achievement unlock` 13, `code` 6, `tear delay down` 6, `=` 4, `blindfolded` 2,
`citation needed` 2, `reconfirm` 2, `dlc clear` 1, `collectible table` 1, `entity table` 1,
`infobox passive collectible` 1.

Two of those below the line are worth a sentence each in the plan rather than an
implementation: `citation needed` and `reconfirm` are editorial marks with no reader-facing
content, and `infobox passive collectible` appearing *inside* a page body (1 occurrence) is a
wiki-side authoring mistake, not a template we lack.

## Decision 8 — the sections that fall through by oversight (delegated)

`meta.diagnostics.discardedSections` separates two different things, and only one of them is
intentional. Deliberate, and staying out: `Trivia` 875, `Gallery` 346, `In-game Footage` 705 +
133 + 2 + 1, `References` 100 — there is even a test pinning `section_kind("Trivia") == None`.

Falling through by oversight, and joining the mapping: `Unlockable Items` 15,
`How to Acquire` 3, `Excluded Items` 3, `Bug` 3, `Interaction` 2, `Rewards` 2. The first is the
plural of a title the mapping already knows in the singular; the rest are spelling variants of
kinds that already exist.

The long tail of one-off titles (`Blood Clots`, `Dark Esau`, `Poop Varieties`, …) stays out:
they are page-specific headings, not a kind.

## What this sub-project does not do

- **It does not make the app work without the game.** That is sub-project 2, and it needs a
  per-field source model and a decision about sprites. Registered in `docs/BACKLOG.md`.
- **It does not translate anything.** Sub-project 3. Noted there, and measured here so nobody
  repeats the search: the wiki's own translation subpages number **20 in total** across all
  1727 pages — German 14, Italian 4, Chinese 1, Korean 1, and zero for French, Spanish, Polish,
  Russian, Japanese and Portuguese. `is_translation_subpage` filters them out and is right to.
  Translation is entirely our own work, and the game's stringtable already covers eight
  languages — English, Japanese, Korean, Chinese (Simple), Russian, German, Spanish, French —
  with **no Italian**.
- **It does not fetch anything new.** The Cargo table `collectible` has seven fields in total
  (`alias`, `description`, `dlc`, `id`, `is_activated`, `link`, `unlocked_by`) and we already
  request six. It holds no `quality`, no `tags`, no `pool`, no `recharge`: those live in the
  wikitext, on disk.
- **It does not recover the pool or the Collection Grid.** Those two rows of the rendered
  infobox are computed by the template and appear in no wikitext (`pool` is present on 45 of
  720 collectible pages, the grid position on none). `Module:Item pool` on the wiki is marked
  `-- OBSOLETE MODULE!`, so the source is unidentified. With the game installed the question
  does not arise — `itempools.xml` is already parsed into `catalog`. Backlog.
- **It does not ship wiki images.** `dataset/ATTRIBUTION.md` states "Wiki images are not
  included" and that stays true. Sprites come from the user's own copy through `unpack`; the
  owner's decision (2026-09-13) is that the wiki is a per-sprite fallback only for sprites that
  turn out to be unobtainable, downloaded at runtime into the local cache, never packaged.

## Risks, and the one thing the plan must actually check

**The risk is a silent wire change.** `Entry` gaining three fields and `Infobox::Item` going
from a unit variant to a struct variant are both changes TypeScript cannot notice on its own:
the mirror in `ui/src/lib/ipc/types.ts` is written by hand. The repo has been bitten here
before — `core_save::Kind` crossed the boundary inside `ipc::SectionCount`, typed as `string` on
the TypeScript side, and a renamed variant changed the wire with the whole suite green.

Two specific traps, both with precedent in `CLAUDE.md`:

1. **`rename_all` on an enum does not rename the fields inside its struct variants.** `Infobox`
   already carries `rename_all_fields = "camelCase"`; every new field
   (`devil_price` → `devilPrice`, `shop_price`, `base_hp`, `stage_hp`, `shot_speed`) depends on
   it and must be pinned by a JSON-shape test, not assumed.
2. **`Infobox::Item` stops being a fieldless variant.** It is tagged (`tag = "kind"`), so
   `{"kind":"item"}` becomes `{"kind":"item","quote":…}` — additive on the wire, and existing
   TypeScript narrowing on `kind === 'item'` keeps working. This is the one place where the
   change is safe by construction, and the plan should say so rather than over-test it.

## Tests

Properties over the series, not pinned values — the repo's rule, and the right one here because
the dataset is regenerated from a wiki that keeps moving.

1. **No entry has an empty infobox.** 907 of 1727 do today; at the end, zero. This is also the
   non-vacuity guard: if the parser ever stops finding the templates, this is the test that
   says so, rather than a silently smaller dataset.
2. **Every parameter present in the wikitext is either in the type or named in a declared
   discard list.** No parameter disappears in silence — that is precisely the defect being
   fixed, and re-introducing it would be absurd.
3. **Where both sources speak, they agree** (decision 6): `quality` and `tags` against
   `items_metadata.xml`, `quote` against `items.xml`'s `description`, over `samples/packed`,
   counting disagreements and failing only above 5% per field. Skips with `skip: …` on stderr
   when the sample is absent, like every real-data test.
4. **`unknownTemplates` holds nothing above 50 occurrences** (decision 7), read from the built
   dataset's own `meta`.
5. **JSON shapes pinned** for `Entry` and for all six variants of `Infobox`, in the style
   `model.rs` already uses, plus the hand-written TypeScript mirror updated in the same commit.
6. **`derived` keeps passing**: `wiki.json == build(raw, corrections)`, regenerated and
   committed together.

## Handing on the contract

`DESIGN-BRIEF.md` carries the TypeScript types the design system is built on. The new `Entry`
and the six `Infobox` variants go in there in the same commit that changes
`ui/src/lib/ipc/types.ts`, and the change is announced rather than left to be discovered — the
IPC contract has had someone on the other end since 2026-09-10.
