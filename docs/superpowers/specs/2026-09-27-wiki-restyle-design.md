# `ui` + `ipc` — the Wiki as pictures, colour and everything we know (design)

**Date:** 2026-09-27
**Card:** #90 on the board
**Branch:** `feature/wiki-restyle`, cut from `develop` at `796091f4`, worktree `isaac-dome-wiki-restyle`
**Depends on:** the complete wiki (`2026-09-26-wiki-complete-design.md`, card #86): every page
kind, `WikiPageRef.category`, entity pictures, `edition.ts`, `wiki_item_pools`
**Owner's request (2026-09-27):** *"le icone di SFIDE ed ACHIEVEMENT devono sempre avere uno SFONDO
apposito, altrimenti sono illeggibili, inoltre occhio al centering delle immagini"* — *"pensa pure
ad un restyle completo sia per questa pagina che per le liste, devono avere più info possibili,
altro esempio nei personaggi versione di rilascio, se l'ho sbloccato o meno, un po' per tutto, più
info ci sono meglio è; inoltre più le immagini occupano meglio è siccome alle persone piacciono
colori ed immagini"* — *"ogni singola cosa è importante, anche oggetti/carte/raccoglibili ecc"*
**Status:** decisions 1, 4 and 8 are **(owner)**, answered on 2026-09-27. The rest are
**(delegated)**, so the first read can overturn any of them cheaply.

## What this sub-project is

Measured on `develop` at `796091f4`:

- The landing draws each tile's sprite with `PixelSprite` directly (`WikiLanding.vue:130-140`)
  instead of `WikiFigure`, so an achievement or challenge picture (dark strokes on transparency)
  lands on the dark tile with nothing under it: grey noise. `WikiFigure` already knows those two
  kinds need `bg-mark-paper` (`WikiFigure.vue:19-45`, `AchievementArt.vue:29-36`); the lists use it
  and read fine. The sprite sits in a 48 px box (`--spacing-wiki-row-figure`) at the top of a
  112 px tile (`mb-auto`): small and off-centre.
- Every category list has the same row: figure 48×48, title, id, chevron
  (`WikiCategoryList.vue:112-139`). One filter (title substring), one sort (by title).
- A row knows `target`, `title`, `iconUrl`, `category` (`WikiPageRef`, `crates/ipc/src/wiki.rs:22-27`)
  and nothing else, while each `Entry` carries the edition set, the quality, the stats, the HP, the
  unlock condition…
- The profile already knows, in four views no wiki list reads: achievements done
  (`graph_views`), items collected and locked (`collection`), challenges done (`challenges`), marks
  per character (`marks`). The bestiary is read by `core-save` and reaches no screen.

This sub-project is one sentence: **every wiki surface shows a picture you can read, as large as it
fits, and everything the wiki, the game and the save know about the thing.**

## Applicable constraints

1. Frontend rules (`docs/frontend-conventions.md`, `pnpm scan`): no `<style>`, every visual value
   a token in `@theme`, no alpha modifiers, one theme, the 4 px grid, `components/ui/` primitives,
   no `invoke()` in components, no string unions, no visible strings in templates (i18n en + it).
2. Pictures come from the user's copy of the game at runtime (constraint 3). Without the game a
   figure degrades to its category icon, never to a hole.
3. Without a chosen save, everything but the profile state shows; the profile state appears only
   when there is one. An unreadable section degrades that one field, not the row.
4. The IPC contract changes (`pnpm ipc:types`). A new command redraws `docs/architecture.md` in
   the same commit.
5. Size is not a constraint (the owner: *"mi interessa estremamente poco"*).
6. A bestiary tally is named only where `docs/save-format.md` measured it (decision 7).

## Decision 1 — scope: the whole Wiki (owner)

The landing, the twelve category lists **and the single pages**. The single pages get the same
treatment: a large figure on its proper background, coloured edition badges, the profile state,
and every infobox field laid out as data, not prose.

## Decision 2 — one figure component, everywhere (delegated)

`WikiFigure` is the only thing that draws a wiki picture: landing tiles, list cards, list rows,
the page hero, name lists, search. It owns three rules, once:

- **the background**: achievement and challenge art always sit on `bg-mark-paper` (through
  `AchievementArt`), everything else on the figure's own surface token;
- **centring**: the picture is centred on both axes in its box;
- **scale**: pixel sprites at the largest *integer* scale that fits the box (`image-rendering:
  pixelated`), achievement paintings at their aspect (`--aspect-achievement`), letterboxed.

The sizes become a small token scale in `spacing.css` (`--spacing-figure-row`, `-card`,
`-tile`, `-hero`), each larger than today's. A test on the size-to-scale function pins the
integer rule.

## Decision 3 — what a row knows: `PageFacts` (delegated)

`WikiPageRef` gains `dlc` (the edition set, every kind) and `facts: PageFacts`, a tagged union
with one variant per kind carrying scalars already in the entry, computed once per window in
`wiki_index`. **Derived from the `Infobox`, never a second copy of its fields**: a function
`facts(entry) -> PageFacts` in `ipc`, exhaustive over `Infobox`.

| kind | facts |
|---|---|
| item | quality, template (passive/active), recharge, shop price, devil price, tags |
| trinket | tags |
| achievement | requirement text (flattened), what it unlocks (`Target`) |
| boss | base HP, floors (`environment`) |
| challenge | character (`Target`), goal, blindfolded, curse |
| character | health (flattened), damage, tears, range, speed, luck, shot speed, tainted |
| transformation | items required, contributor count |
| entity | base HP, floors |
| article | category (card, rune, pickup, stage, version); a version also its number and date from the Cargo `version` table |

## Decision 4 — lists are a card grid, or a table, per category (owner)

Default: **a grid of cards**, picture on top as large as the card, under it the name, the id, the
edition badge, the kind's facts as coloured chips and the profile state. A switch turns the list
into a **table**: one row per page, every fact a sortable column. The choice is remembered per
category as a per-viewer convenience (the same persistence the window session uses; the plan
names it). Both views are virtualized (`TanStack Virtual`).

**Filters** on every list: the title, the edition, the profile state (when there is a save), and
the kind's own: quality and type for items, tags for items and trinkets, tainted for characters,
card or rune for cards. **Sorts**: name, id, edition, and each numeric fact (quality, HP, stats,
price). Filter and sort are pure functions in `ui/src/lib/wiki/`, tested with Vitest.

## Decision 5 — colour is data (delegated)

New colour tokens in `colors.css`, each a `-surface` / `-foreground` pair like the state tokens:

- **edition**: Rebirth, Afterbirth, Afterbirth+, Repentance, Repentance+ — the badge colour;
- **quality**: 0 to 4 (and -1, the "not in pools" quality) — the pips and the chip;
- **category accent**: one per landing category — the tile's wash and the list header;
- **profile state**: reuse `--color-state-done/now/blocked/unknown`.

## Decision 6 — the profile state of a page: `wiki_progress` (delegated)

One new command, `wiki_progress`, returns the save's state for every page that has one, keyed
like the pages, as a pure function in `ipc` over what the four existing views already read:

| kind | state |
|---|---|
| achievement | done |
| item | collected (section 4), locked / unlocked and by which achievement |
| trinket, card, rune | unlocked, from the achievement that unlocks it (graph targets) |
| character | unlocked (its achievement), marks done / total |
| challenge | done / available / blocked |
| boss, entity | met, killed, killed you (decision 7) |
| transformation, stage, version, pickup | none: the save does not say |

It is a separate command and not part of `wiki_index` because the index does not depend on the
save and the progress does: switching save refreshes one, not both. Never cached as "no save".

## Decision 7 — the bestiary reaches the screen, named only where measured (owner + delegated)

The owner asked for it (owner). What it shows is delegated to what `docs/save-format.md` measured:
**tally 1 = met, tally 2 = killed, tally 4 = killed you**, keyed `type.variant.subtype` like the
entity pages. **Tally 3 is unnamed and is not shown.** The document also records that the
accumulated totals disagree with the per-window behaviour (killed > met on 213 keys), so the
numbers are shown as the save holds them and nothing is derived from comparing them: no "kill
rate", no "met but never killed".

## Decision 8 — the landing (owner scope, delegated form)

Each tile: the category's representative picture through `WikiFigure` at `--spacing-figure-tile`,
centred, on the category's accent; the name; the page count; and with a save the progress
"N of M" as a bar in the state colour (achievements done, items collected, characters unlocked,
challenges done, bosses killed). The Wiki header keeps its text and gains the snapshot facts it
already shows below.

## Decision 8b — a hero on the landing and on every list (owner)

The owner, 2026-09-27: *"anche una Hero magari?"*. **The landing** opens on a hero: a mosaic of
the category samples (an item, a boss, a character, a monster, a card…) through `WikiFigure`, the
title, the Wiki's totals (pages, snapshot) and, with a save, the overall progress (achievements
done of all, items collected of all). **Every category list** opens on a hero of its own: the
category picture at `--spacing-figure-hero` on the category accent, its name, its page count and,
with a save, its "N of M" bar. Both are pure layout over data decisions 3, 6 and 8 already
provide: no new IPC.

## Decision 9 — the single pages (delegated)

`WikiHero`: the figure at `--spacing-figure-hero` on its background, the edition badges in their
colours ("Added in … · Removed in …"), the kind's facts as chips (the same `PageFacts`), and the
profile block from `wiki_progress`. The infobox rows keep their data and take the chip and colour
language; the body sections are unchanged in content.

## Decision 10 — the order of work (delegated)

1. `ipc`: `PageFacts`, `dlc` on `WikiPageRef`, `wiki_progress` with the bestiary view — the
   contract, first, so the UI streams can start.
2. `ui` foundations: the tokens, `WikiFigure`'s three rules, the chip and badge primitives.
3. In parallel on top of 1 and 2: the landing; the lists (grid, table, filters, sorts); the single
   pages.
4. `docs/architecture.md` (one command), `pnpm check`, the window.

## What this sub-project does not do

- It does not name bestiary tally 3 or the word after the last tally.
- It does not add pictures the game files lack (cards, stages).
- It does not change the body content of a page, only how the page frames it.

## Done means

- Achievement and challenge pictures are readable everywhere, through one component.
- Every figure centred and at integer scale, larger than today.
- The landing, the twelve lists and the single pages show the facts of decision 3 and, with a
  save, the state of decision 6; lists filter and sort on them.
- `pnpm check` green; `docs/architecture.md` counts `wiki_progress`.
- Seen in a window with the game and a save, with and without the save.
