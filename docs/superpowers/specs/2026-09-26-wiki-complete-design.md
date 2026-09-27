# `wiki` — the whole wiki, so every link on a page leads to a page (design)

**Date:** 2026-09-26
**Card:** #86 on the board, absorbs B47
**Branch:** `feature/wiki-complete`, cut from `develop`, in the worktree `isaac-dome-wiki-complete`
**Depends on:** the wiki dataset (`crates/wiki`, `crates/wiki-snapshot`, `dataset/`), the infobox
design (`2026-09-13-wiki-infobox-design.md`), the wiki screens and search
(`2026-09-12-screens-wiki-search-design.md`)
**Owner's request (2026-09-26):** "almeno così sistemiamo tutti i link"; on the landing, "pensiamo
già anche ad una grafica migliorata, più immagini mettiamo meglio è"
**Status:** decisions 1, 2, 5, 9 and 10 are **(owner)**, answered on 2026-09-26. Decision 3 was handed to
the author by the owner ("mi fido delle tue decisioni tecniche") and is marked **(delegated)** with
the rest, so the first read can overturn any of them cheaply.

## What this sub-project is

A reader of a wiki page in the app meets underlined words that do not open. Measured on the
committed snapshot (2026-09-14, revid 269333) by the dead-link tally that is this branch's first
commit (`wiki::dead_links`, printed by `pnpm wiki:build`):

| what | distinct destinations | occurrences |
|---|---|---|
| `Inline::Concept`, a `[[link]]` to a page we did not fetch | 413 | 6569 |
| `Inline::Ref` to an entity with no page (common enemies) | 341 | 3464 |
| `Inline::Ref` to a room, stage or pickup concept | 162 | 4025 |

The hand count on the card said 527 / 6413 and 443 / 3383. Before the first letter was folded the
tally also read 527 distinct concepts; folded the way MediaWiki folds it (`familiar` and
`Familiar` are one page) they are 413.
The occurrences are higher because a page with two infoboxes (Judas and Black Judas, Jacob and Esau)
gives each entry its own copy of the page's sections, and the tally walks entries. The entities are
fewer because the tally keys an entity by its `id.variant.subtype`, the identity `Dataset::entry`
looks up, where a count by the name written in `{{e|…}}` counts every spelling.

Why they are dead: the fetch chooses a page by the infobox it transcludes, and knows seven
infoboxes. The wiki's main namespace holds **1774 pages that are not redirects** (measured
2026-09-26 with `list=allpages&apfilterredir=nonredirects`). The snapshot holds 1113. The other 661
are the six infoboxes B47 counted (entity 247, monster 126, pickup 97, card 66, rune 28, stage 27),
the mechanics (Damage, Luck, Range, Tears: no infobox at all), the concepts (Champion, Greed Mode,
Curse), the machines and beggars, the rooms, and the list and disambiguation pages.

This sub-project is one sentence: **fetch every page, give every page a shape, and resolve every
link to the page it names.**

## Applicable constraints

1. **The snapshot is one pass per release, from `wiki-snapshot`, never from the app.** The fetch
   keeps its pace: 250 ms between responses, 50 pages per request, the named user agent.
2. **`dataset/raw`, the derived dataset and `corrections.json` are committed together**, and the
   `derived` test holds `derived == build(raw, corrections)`. `ATTRIBUTION.md` stays true.
3. **No game assets in the package** (constraint 3). The pictures on the new pages come from the
   user's copy of the game, at runtime, through `catalog`. The wiki's own images (Gallery,
   In-game Footage) stay out: they are assets, and they are the network.
4. **The IPC contract changes**: `Target` grows, `Infobox` grows, `WikiCounts` grows. Every
   consumer is exhaustive (`match` without `_` in Rust, `assertNever` in TypeScript), so a missed
   arm breaks the build. That is the migration plan, not a risk.
5. **Degrade, never fail.** A page the parser cannot shape still becomes an article with its
   text. A picture the game does not have is the "no art" placeholder, not an error.

## Decision 1 — the whole main namespace (owner)

The fetch enumerates namespace 0 with `generator=allpages&gapfilterredir=nonredirects`, 50 per
request with the wikitext in the same response, the way `fetch_template` already batches. What it
leaves out, each for a reason:

- **translation subpages** (`Page/de`, `Page/ko`): `is_translation_subpage` already says what they
  are, and the app is not translated from the wiki;
- **the wiki's own pages** (`Binding of Isaac: Rebirth Wiki` and its subpages): a list, not
  written in the source, since the main page is the only one and it is a portal. The list lives
  next to `is_translation_subpage` with the reason on each line.

Everything else is kept, list and disambiguation pages included: a link to `??? (Disambiguation)`
exists on real pages, and "every link opens" means that one too.

**Redirects are fetched as well**, as a map and not as pages: MediaWiki refuses `redirects` with the
`allpages` generator, so the map is read the other way round, each content page with the redirects
pointing at it (`prop=redirects`, about ten requests), into `dataset/raw/redirects.json`,
`{ "from": "to" }`, sorted. A link written to a redirect title resolves through it. Today nothing
handles redirects, so a link to `Tears Up` or `Soul Hearts` is a dead concept even where the page
exists.

The kind of a page is still decided at fetch time and recorded in `raw/index.json`: a page that
transcludes one of the known infoboxes keeps its kind (the existing `embeddedin` queries run first
and win), a page that transcludes `Infobox monster` or `Infobox entity` is an **entity**, and
everything else is an **article**. `raw/pages/<kind>/` gains `entity/` and `article/`.

## Decision 2 — two new kinds: a typed entity, and an article that is only its body (owner)

**Entity.** `PageKind::Entity`, `Infobox::Entity { … }`, filed in a new `entities` collection keyed
`id.variant.subtype` like the bosses. The key comes from the Cargo `entity` table (`_pageName` →
every row), which the fetch already downloads: 583 rows, 378 of type `monster`. A page with several
rows (a monster and its variants) is one entry reached by every one of its keys, the way a boss
page with champion subtypes already is. The infobox's parameters are read exhaustively, like every
other infobox (`tests/no_silent_parameter.rs` covers the new one by construction). The two
templates `Infobox monster` (126 pages) and `Infobox entity` (247) share this kind. Their
parameters are compared at plan time: one variant if one set covers both, two if not.
`Target::Entity` resolves the bosses first, then the entities.

**The bestiary is not linked.** `core-save` reads it and no screen shows it. The key is the same
triple, so a later bestiary screen joins on it for free. Linking it is not this card.

**Article.** `PageKind::Article`, filed in an `articles` collection keyed by the **canonical
title** (`wiki::canonical_title`, the MediaWiki rule: first letter upper-cased, `_` as space,
whitespace collapsed). It has no infobox fields. `Infobox::Article { category }` carries the
infobox template it was found under, when there was one: `card`, `rune`, `pickup`, `stage`, or
none. That is how the landing tells a card from a mechanic (decision 5), and it is the whole of what
we read from those four infoboxes: their parameters are **declared out of scope**, counted by
`Diagnostics::unknown_infoboxes` as today. That closes B47: entity and monster enumerated,
card, rune, pickup and stage enumerated as article categories with their parameters declined,
grid entity has no page.

**An article's sections keep their own titles.** `SectionKind` is a closed list of thirteen
headings that make sense on an item or a boss. A page like Damage is headed "Formula", "Damage
multipliers", "Items that affect damage". So `Section` gains a `title` and `SectionKind` gains
`Other`: a heading the list does not know is **kept under its own title** instead of discarded.
This applies to every kind, which settles the "sections discarded with real content" gap
(Bag of Crafting's recipes, Poop Varieties, Notable Rerolls…) with no per-heading mapping.
`Item Exclusion` is mapped to the existing excluded-items heading, being the same section under
another name. What stays discarded, by name: `Gallery` and `In-game Footage` (constraint 3), and
`Trivia` (decision 9).

**Resolution.** Three kinds of text now reach an article:

- `Inline::Concept { page }` becomes `Inline::Ref { target: Target::Article { title } }` when the
  canonical title, or its redirect target, is an article. A concept that still names nothing
  stays a `Concept`, and the dead-link tally lists it;
- `Target::Stage`, `Target::Room` and `Target::Concept` (the pickup concept) resolve through
  `Dataset::entry` to the article of the same canonical name, redirects included. No new variant
  is needed for them: the name they carry is a page title;
- `Target::Article { title }` is the one new `Target` variant, for a plain `[[link]]`.

## Decision 3 — the dataset on disk is split by collection, the embedded copy is compact (delegated)

Measured: `dataset/wiki.json` is 28.2 MB **pretty-printed**, 9.4 MB compact. `build.rs` deflates
it as it sits on disk, which gives 1.9 MB embedded in a 22 MB release exe. 661 more pages, most of
them long mechanics pages, take the pretty file to an estimated 40–45 MB — past GitHub's 50 MB
warning at the next growth, and a single file whose diff nobody reads.

- **On disk**: one pretty file per collection under `dataset/wiki/` — `meta.json`, `items.json`,
  `trinkets.json`, `achievements.json`, `bosses.json`, `challenges.json`, `characters.json`,
  `transformations.json`, `entities.json`, `articles.json`. The diff of a re-fetch then says which
  collections moved, and none of them is near a limit (items, the largest, is 16 MB pretty today).
  `derived` compares the whole directory; `graph`'s reader goes through `wiki::Dataset`, never
  the files by name.
- **Embedded**: `build.rs` reads the directory, re-serialises it **compact**, and deflates that one
  buffer. One inflate at startup, the same as today. Splitting the embedded copy too would buy lazy
  loading, and nothing wants it: the search index reads every page at startup anyway.
- **The ceiling**: the embedded deflate stays **under 4 MB**, twice today's, checked by a test in
  `crates/wiki` that reads `OUT_DIR`'s deflate. Measured before (1.9 MB) and after, both on the
  card, with the release exe size.

## Decision 4 — health, base stats and pools (delegated)

The measured gaps on the pages we already have, done in this branch because the parser is open:

- **Starting health.** `{{hearts|red=3}}` and `{{heart|…}}` are read (32 of 40 characters and 44
  of 45 challenges have an empty `health`). And the unknown-template fallback, which reads only the
  first positional argument, no longer returns empty in silence for a template that has only named
  arguments: that becomes a diagnostic.
- **Base stats.** A character page omits the parameters that equal the template's defaults, and
  the site shows the defaults (Isaac: Damage 3.5…). The fetch downloads
  `Template:Infobox character`'s wikitext once, the build reads its `{{{damage|3.5}}}` defaults,
  and an omitted stat takes the default. Not `players.xml`: the game file does not carry the base
  stats (the engine does), and the wiki template is the source the site itself uses.
- **Pools.** The item and trinket wikitext carries no `pool` parameter (674 of 719 and 181 of 188
  empty): the site fills it from elsewhere. The source is the user's `itempools.xml`, which
  `catalog` already reads. `ipc` joins it at the page view, so pools are the game's truth, and
  without the game the row is absent, not wrong. The dataset field that is always empty is
  removed rather than kept as a promise.
- **`{{entity table}}`** (Ultra Greed) is expanded into a table of entity references.

## Decision 5 — the landing and search (owner)

The landing gains four tiles: **Monsters** (the entity collection), **Cards and runes**,
**Pickups**, **Stages** (articles by `category`). An article with no category (mechanics,
concepts, machines, list pages) has no tile. It opens from links and from search, and search
indexes every page of every kind.

**Pictures, as many as the game gives** (the owner's words). All of them come from the user's copy
at runtime, through `catalog` and `ipc::target_sprite`, which today answers `NoArt` for stages,
rooms and pickup concepts:

- **monsters**: the entity's first animation frame. `catalog` gains an `entities2.xml` reader
  (type, variant, subtype → `anm2` file), and `anm2.rs` already renders a frame. This is a spike
  first: the plan starts by rendering ten monsters from the real archives before writing the
  reader for all of them;
- **cards and runes**: measured, there is no per-card picture in the game files: `pocketitems.xml`
  names no sprite, and `entities2.xml`'s `5.300.x` rows are one per card family, drawing its back.
  A card page has no figure; the Cards and runes tile draws the tarot back;
- **pickups**: the pickup's `anm2`, through the same `entities2.xml` reader (pickups are entities
  of type 5);
- **stages**: the stage's `gfx/ui/stage/` title art, if the archives carry one per stage, `NoArt`
  if not;
- **articles with no category** have no picture, and the page layout does not reserve space for
  one.

The landing's redesign is in the plan's last phase, shaped on the Kit page and in the browser
before it is wired: tiles with a representative sprite each, and the category lists as a grid of
sprites where the kind has pictures.

## Decision 6 — the dead-link tally is the definition of done (delegated)

`wiki::dead_links` walks the built dataset and reports, per destination, every `Concept` and every
`Ref` with no entry. It is not in the embedded dataset. It is printed by `pnpm wiki:build`, and a
test in `tests/real.rs` asserts the tally is not empty on today's snapshot (the vacuity guard). Once
the fetch lands, the same test pins the **residue**: every destination still dead is listed in
`corrections.json` under `deadLinks`, with the reason ("the wiki has no page", "a red link on the
wiki itself"). A destination not in that list fails the test, and a listed one that now resolves
fails it too (the `GONE` line of `check-doc-refs`).

## Decision 7 — the IPC and the screens (delegated)

- `Target::Article { title }` and `Infobox::Entity` / `Infobox::Article` cross the IPC through the
  generated `types.ts`. `pageKey`, `categoryOf`, `pageId`, `WikiFigure` gain their arms.
  `WikiCategory` gains `monsters`, `cardsAndRunes`, `pickups`, `stages`.
- `WikiCounts` gains the same four, and `articles` for search.
- `WikiInfobox` renders the entity infobox, and nothing for an article (no box, the body starts at
  the top).
- `WikiInline`'s `canOpen` default ("without it every reference opens, as on the Kit page") is
  left alone: it is a Kit convenience and every real page passes it.

## Decision 8 — the order of work (delegated)

1. The dead-link tally and `canonical_title` (done first, to measure before the scope moves).
2. `wiki-snapshot fetch`: allpages, redirects, the two new kinds on disk, the template-defaults
   fetch. Tested on fixtures, **not run** against the wiki until 4.
3. `crates/wiki`: `PageKind`, `Infobox::Entity`/`Article`, `Section.title` and `SectionKind::Other`,
   resolution through articles and redirects, health, stat defaults, entity table, the split
   dataset and the compact embed with its ceiling.
4. One `pnpm wiki:fetch` + `pnpm wiki:build`, committed with `raw/`, the dataset and
   `corrections.json` together; the residue listed; sizes measured.
5. `ipc` and `catalog`: the new `Target` arms, pools from `itempools.xml`, the sprite spike, then
   the readers.
6. `ui`: types, landing tiles, category lists, entity infobox, article page, and the pictures.
7. `docs/architecture.md` if a count moved, `CLAUDE.md`'s module table (`wiki`), `pnpm check`,
   and the window.

## Decision 9 — Trivia stays out (owner)

Trivia is discarded on 890 pages, and stays discarded: the least useful part of a page for "what do
I play tonight". It is listed among the exclusions of decision 10, so the choice is visible and
cheap to reverse.

## Decision 10 — everything the wiki says, or a written reason why not (owner)

The owner, 2026-09-26: *"riuscimi ad aggiungere TUTTO TUTTO TUTTO di informazioni della wiki"*,
naming the version notes ("Added in Afterbirth", "Added in Afterbirth+") in particular.

So completeness becomes a check, the same shape as the dead-link residue: every **template**, every
**section heading** and every **infobox parameter** that occurs in `dataset/raw/` is either read
into structure by the parser, or listed in `corrections.json` under `excluded` with its reason.
A test in `tests/real.rs` walks the raw wikitext and fails on a name in neither place, and on a
listed name that no longer occurs. The unknown-template fallback, which keeps the first positional
argument and drops the rest, stops being silent: a template that reaches it and is not excluded is
a failure, not a guess. The starting exclusions, each with its reason: `Gallery`,
`In-game Footage` and `[[File:…]]` (images and video, constraint 3), `<ref>` (external URLs),
`Trivia` (decision 9), and the purely presentational templates (spacing, clears, navboxes) the
inventory names.

**Editions.** `{{dlc|…}}` inline and the infobox `dlc` already reach the model (`Inline::Edition`,
`Entry.dlc`, about 4000 inline occurrences). Version text that is not a `{{dlc}}` template
("Added in …", "Removed in …", patch notes) is traced end to end by the inventory, and every page
shows **which edition added it** in its header, from `Entry.dlc`.

The inventory itself (templates, headings, parameters with their counts, and what each becomes) is
measured before the parser work starts, and the plan takes its classes as the parser's task list.

## What this sub-project does not do

- It does not link the bestiary (decision 2).
- It does not read the parameters of the card, rune, pickup and stage infoboxes (decision 2).
- It does not fetch images, video or `<ref>` citations from the wiki (constraint 3; the
  citations are URLs to Twitter and Reddit).
- It does not translate: translation subpages are skipped.

## Risks

- **Size.** 661 more pages is an estimate of 60% more text; the ceiling is the check. If it is
  crossed, the first lever is the article body of list pages that repeat the item table.
- **Parser coverage.** Mechanics pages use templates the parser has never seen (formula tables,
  `{{stat}}`-style templates). The unknown-template fallback keeps the text, and
  `Diagnostics::unresolved` counts them. The plan reads that count after step 4 and decides which
  templates earn a reader.
- **Monster sprites.** `entities2.xml` → `anm2` → frame is new ground for `catalog`; the spike
  says whether it holds before the reader is written.

## Done means

- `pnpm wiki:build` reports the dead-link tally, and it is zero or every residue is in
  `corrections.json` with its reason.
- B47 is closed with each of the seven infoboxes decided (decision 2).
- `dataset/raw`, `dataset/wiki/`, `corrections.json` and `ATTRIBUTION.md` committed together,
  `derived` green.
- The embedded deflate is under 4 MB, measured before and after.
- In a window: a link to a mechanic (Damage), to a monster (Gaper) and to a card (0 - The Fool)
  opens, and the landing shows the four new tiles with their pictures.
