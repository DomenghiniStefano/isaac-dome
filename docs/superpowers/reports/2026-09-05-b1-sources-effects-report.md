# B1 — Sources for item effects, notes, and synergies: analysis report

**Date:** 2026-09-05
**Backlog entry:** `docs/BACKLOG.md`, B1 (analysis task, no implementation)
**Outcome:** one source chosen, **the English wiki on wiki.gg** (`bindingofisaacrebirth.wiki.gg`),
read via the MediaWiki API and Cargo tables, with the **dataset built at build time and shipped in
the package**; runtime updates remain optional and go through GitHub, never the wiki.
The backlog entry's questions are closed in the "The answers" section. All the numbers below
come from calls made today, not from memory; the probe scripts are throwaway.

## The discovery that changes the question

The backlog entry compared "fandom wiki (English)" with a CC BY-NC-SA license. **That's the
abandoned copy.** The community migrated the wiki to wiki.gg in 2023, and the fandom copy
stayed up only because fandom doesn't allow deleting it. The evidence, gathered today:

| | fandom | wiki.gg |
|---|---|---|
| declared license (`meta=siteinfo`) | CC BY-NC-SA | **CC BY-SA 4.0** |
| last edit of *False PHD* | 2025-10-08 | 2026-07-30 |
| item pages edited in 2026 | — | 625 of 723 |
| content of *False PHD* | missing the Bag of Crafting note, three fewer synergies (Rock Bottom, double Lucky Foot, …) | complete |
| HTML to a non-browser client | 403 | 200 |
| active users | 48 | 82 |
| `docs/PROJECT.md` cites it for the unlock graph | no | **yes** |

The question "what does NC mean for us" dissolves: the living source doesn't carry the NC
clause. One caveat is worth writing down once: the wiki was born on Gamepedia, passed through
fandom, and the historical content was written under BY-NC-SA; wiki.gg declares BY-SA 4.0 for
everything, and a reuser relies in good faith on that declaration. IsaacDome is free, with no
backend or monetization, so it would satisfy the NC clause too. **What actually binds us is BY
plus SA**: attribution in the package, and the derived dataset stays CC BY-SA.

## The sources, compared

### 1. English wiki on wiki.gg — the choice

**Access.** The MediaWiki API responds without credentials (`api.php`, MediaWiki 1.43); it only
wants an identifiable `User-Agent`. `action=parse` works (wikitext or HTML, with
`prop=sections`), as does `action=query` with generators (`generator=embeddedin` on the infobox
templates, 50 pages per call) and `Special:Export`. `robots.txt` contains `Disallow: /api.php`
for `User-agent: *`, the Cloudflare `ai-train=no` signal, and a note from the operators to
respect the license and not generate irresponsible automated traffic. That's a directive for
crawlers, not a ban on using the API, but it dictates the approach: **an identified, sequential
client, one pass per release**, and never a download per user on first launch. The 723 items
and 188 trinkets download in **19 calls** (full wikitext, 2.9 MB); the Cargo tables in about ten.

**Coverage, measured by id.** The `collectible` Cargo table has 720 rows with 719 unique ids;
the game's `items.xml` has 721 collectibles (425 passive, 170 active, 126 familiars). Two ids
are missing: **59** (the *The Book of Belial* variant with Judas's Birthright) and **656** (the
passive variant of *Damocles*), both internal variants that the wiki handles inside the main
item's page. No wiki id is missing from the game. Trinkets: **188 of 188**.
A data error on the wiki: the *Tonsil* page declares `id = 474`, which in the game is *Broken
Glass Cannon* (Tonsil is 97). A small, versioned layer of our own corrections covers it.
*(Corrected 2026-09-05: it's not a wiki error. The page has two infoboxes — the current trinket
97 and the Afterbirth+ collectible 474, replaced by Broken Glass Cannon in Repentance — and the
Cargo table confirms it, `dlc = 4` on Tonsil's row versus bit 16 on current-edition rows. The
fix in implementation isn't an id correction but a filter on the `dlc` bitmask: see
`docs/superpowers/reports/2026-09-05-wiki-dataset-report.md`.)*

**The sections we were looking for exist and are regular.** Across the 723 item pages:

| section | pages | note |
|---|---|---|
| Effects | 621 (+98 `Effect`, +5 `No Effect`) | the bulleted list with numbers |
| Notes | 616 | |
| Synergies | 493 | |
| Interactions | 478 | |
| Trivia | 617 | out of scope, but free |
| Bugs | 184 sections, 352 uses of the `{{bug}}` template | |
| In-game Footage | 572 (+105 with lowercase) | 865 YouTube embeds, to ignore |
| Unlockable Achievements | 37 | in two spellings |

Trinkets: Effects 154 (+35 `Effect`, +4 `No Effect`), Synergies 168, Notes 150, Interactions 91.
Median page 2.7 KB of wikitext, max 28 KB; 29 wikitable tables among items and 18 among
trinkets (for example the pill table on *False PHD*).

**Links to entities are templates, and templates resolve via Cargo.** Usage counts on item
pages: `{{i|…}}` item 10,788, `{{e|…}}` entity 3,150, `{{c|…}}` character
1,434, `{{t|…}}` trinket 808, `{{s|…}}` stage 606, `{{dlc…}}` 3,460 (with `{{dlc+}}`/`{{dlc-}}`
and `{{dlcalt}}`: they delimit text valid only in certain editions), `{{r|…}}` rune 792,
`{{p|…}}` pickup 456, `{{chal|…}}` challenge 134, `{{tf|…}}` transformation 75, `{{m|…}}` 171,
`{{a|…}}` achievement (209 among trinkets). The templates `i`, `e`, `t`, `s` are defined as
`{{cargo lookup}}` on the `collectible`, `entity`, `trinket`, `stage` tables: **the same
tables can be queried via `action=cargoquery`**, and they give the name/alias → id map
needed to turn `{{i|Little Baggy}}` into an internal link in the app. Tables verified:

| table | rows | useful fields |
|---|---|---|
| `collectible` | 720 | `id`, `dlc`, `alias`, `is_activated`, `link` |
| `trinket` | 188 | `id`, `dlc`, `alias`, `link` |
| `achievement` | 641 | `id`, `name`, `dlc`, `description`, `requirements`, `notes` — id 62 is *Epic Fetus*, matching our `achievements.xml` |
| `entity` | 583 | `id`, `variant`, `subtype`, `type`, `alias` |
| `challenge`, `player`, `pool_collectible`, `collectible_name` | — | challenges, characters, pools with weight, names by DLC |
| `version` | — | game patches with date: the wiki's latest noted version is V1.9.7.17, 2026-04-20 |

The `dlc` field is an integer that looks like a bitmask (24 for *False PHD*, 31 for
Rebirth items, 16 for Repentance+): to be verified in the implementation task, not here.
The `achievement` table is also material for M2, out of scope for B1.

**Languages.** English. There's a French wiki on wiki.gg (1,664 articles, complete
"Effects / Notes / Synergies / Interactions" sections, two active users) and a Chinese one
(54 articles). **No Italian one.** Pages carry `fr` and `zh` interlinks to fetch a translation
if one is ever needed.

**Updating.** Every page travels with `revid` and a timestamp; the same build pass re-run
later produces a readable diff.

### 2. External Item Descriptions (EID)

Community mod, repository `wofsauge/External-Item-Descriptions` on GitHub, last push
2026-08-06, 351 stars. Verified today:

- **No license.** No `LICENSE` file, the GitHub API responds `license: null`, the README and
  CREDITS say nothing about it, searching the code for "license" turns up zero results. By
  default the content is **all rights reserved**: it can't be redistributed in the package
  without the author's explicit consent. The CREDITS cite the wiki and platinumgod as sources.
- **Total coverage**, measured by id across the union of the three tiers (`ab+` 552, `rep` 414,
  `rep+` 28): **732 of 732 collectibles** (including internal variants) and **189 of 189**
  trinkets, in English **and Italian**. Twenty languages in total.
- **Format** Lua, one line per item: `{"654", "False PHD", "{{BlackHeart}} +1 Black
  Heart#{{Pill}} Identifies all pills#…"}` with `#` as a line break and `{{Tears}}` icons. It's
  the level of the wiki's *quote*, with numbers, in three-to-five lines: no Notes, Synergies,
  Interactions, Bugs. Parseable with a regular expression.
- **A bonus the game doesn't have**: `descriptions/names/it.lua` carries Italian names for 732
  items and 189 trinkets ("Dottorato Falsificato", "La Cipolla Triste"). The game's string
  table has eight languages, and Italian isn't one of them.

Verdict: the best source for **a one-line effect description in Italian** and for **Italian
names**, blocked by the license. Worth a request to the author, which is a task of its own
(with an uncertain outcome), not an assumption to build on.

### 3. Wikis in other languages

- **Italian fandom** (`bindingofisaacrebirth.fandom.com/it`): CC-BY-SA, 355 articles, one
  active user, 89 of 392 pages belong to the Flash edition. Rebirth item pages exist as empty
  skeletons (*Brimstone*: 398 bytes, an "Effetto:" section with no text). Discarded without
  reservation.
- **French wiki.gg**: real content, but not our users' language. Useful only as proof that
  the format holds up in a second language.

### 4. Our own curation

Writing 909 entries by hand for five sections isn't realistic: the English wiki is 3.3 MB of
wikitext maintained by 82 people. What does make sense is a **corrections layer** on top of the
derived dataset: a small, versioned file with the fixes for what the wiki genuinely gets wrong
(character infobox ids, unreliable) or the lines we want different. Near-zero cost, and it
closes the "the wiki is wrong" case.

## The answers

1. **Which source.** wiki.gg, English: Effects, Notes, Synergies, Interactions, Bugs sections
   from the wikitext; infoboxes and Cargo tables to cross-reference ids and resolve links.
2. **Under which license in the package.** CC BY-SA 4.0. The package ships an attribution file
   with the wiki's name, URL, license, snapshot date, and the pages' `revid`s; the derived
   dataset stays CC BY-SA. The app's code is separate from the dataset and has its own license.
   **The repository today declares no license** (no `LICENSE` file): that must be decided
   before the first installer, and it's independent of B1.
3. **Where the dataset lives.** **It's built at build time and lives in the package.** A repo
   command does the pass over the wiki (about thirty calls, identified and sequential), produces
   the versioned dataset next to the code, and the diff on each rebuild reads as a commit. The
   app never talks to the wiki: the optional runtime update downloads the same file from
   GitHub, as `docs/PROJECT.md` already plans for the graph. This way the wiki gets one pass per
   release, not one per user, and the app works offline from the very first launch.
4. **How it's flagged that the dataset is older than the game.** The dataset carries the
   snapshot date and the last patch the wiki knows about (the `version` table). On the game
   side, `discovery` already opens `appmanifest_250900.acf` for `installdir` and the DLCs: the
   same file has `buildid` and `LastUpdated`. If the game was updated after the snapshot, the
   screen says so in diagnostics, without blocking anything (constraint 5).
5. **How links to entities become internal links.** The wikitext parser produces a tree of our
   own (paragraphs, nested lists, tables, text with references); each `{{i|X}}` becomes
   a `{ kind: "item", id }` reference resolved via the `collectible` table (alias → id),
   `{{t}}` with `trinket`, `{{a}}` with `achievement`, `{{c}}` with the characters from
   `players.xml`, `{{chal}}` with challenges, `{{e}}` and `{{s}}` with entities and stages,
   which today have no id of ours and remain labeled text. The `{{dlc…}}` markers become an
   edition attribute on the fragment. An unknown template degrades to its text and ends up in
   a counted diagnostic, never in an error.

## What the API exposes for each entity

Inventory of the 61 Cargo tables (`action=cargotables`, `cargofields`, `cargoquery`) and of
the pages, verified with sample rows. Two channels with different roles: **the tables** give
structured fields filterable by id, but Wikitext-type fields arrive already rendered (links
become `[[File:…]]` with spans and icons); **the page wikitext** (`action=parse`) keeps
clean templates (`{{e|Mom}}`, `{{i|Lucky Foot}}`), which is what's needed to resolve
references into our own ids. Rule: ids and scalar fields from Cargo, text and infoboxes
from the page.

| entity | table (rows) | structured fields | from the page |
|---|---|---|---|
| item | `collectible` (720) + satellites by DLC: `_quote` 784, `_quality` 863, `_shop_price` 734, `_devil_price` 755, `_recharge` 204, `_tag` 1,778, `pool_collectible` 2,276, `_image` 890 | id, alias, DLC, active/passive, `unlocked_by` (achievement name, populated on **277** items versus 370 in `items.xml`), prose description | Effects, Notes, Synergies, Interactions, Bugs, Trivia, Unlockable Achievements. `bug` (718) has platform and DLC per bug; `bov_combination` (164) and `bob_combination` (33) are the effects with Book of Virtues and Book of Belial |
| trinket | `trinket` (188) + `_quote`, `_tag`, `_name`, `_image` | id, alias, DLC, `unlocked_by`, description | same as items |
| boss | `entity` (583: 103 bosses, 20 mini-bosses, 378 monsters, 82 untyped) + `entity_name`, `entity_image`, `boss_portrait` (106), `stage_entity` (1,572) | id, variant, subtype, type, alias, `unlocked_by`, stages it appears in. `entity_health` is empty | infobox: base HP, environment (stages, rooms, Double Trouble, downgraded boss), pool it drops from, unlock. Sections: Behavior with phases and HP percentages, Champion Versions, Unlockable Achievements, Damage Scaling, Notes, Strategies, Trivia, Bugs |
| achievement | `achievement` (641) + `achievement_image` | id (same as ours), name, DLC, Steam description, `requirements` (condition with links to bosses and characters), `notes` | infoboxes on the `Achievements/Rebirth 1` pages and similar, plus `link`: **the page for what it unlocks**, i.e. the condition → reward arc for M2 |
| challenge | `challenge` (45) + `challenge_name` | number, starting items and trinkets, pickups, hearts, curse, blindfolded, shops, treasure room, goal, character, unlocks, unlocked by | Difficulty, Strategy, Reward |
| character | `player` (40, variants like Esau under Jacob) + `_damage`, `_speed`, `_tears`, `_range`, `_name`, `_image` | base stats by DLC, hearts, starting items and pickups, unlock | Notes, Item Interactions, Unlockable Achievements, Unlockable Starting Items |
| other | `transformation` (16), `pool` (33), `pickup` (97: cards, runes, pills), `stage` (28, with bosses per stage), `version` (82 patches with date and Steam link), `interwiki` (2,836: fr, es, zh titles) | | |

The images (`_image`, `boss_portrait`) are files uploaded to the wiki: game assets, excluded
from the package under constraint 3, and useless anyway since we extract them from the user's
own copy.

## Real time, first launch, or build: the decision and the reasons

Three possible modes, one chosen. **Snapshot built at build time and shipped in the package**,
with an optional update that downloads the *same already-built file* from a GitHub release.
It's the model of `scripts/build-encounters.mjs` in poke-dome: source downloaded at build time,
meta `{ source, ref, schemaVersion, builtAt }`, idempotent script, network only as a fallback.
Here the "pin" isn't a SHA but the pair snapshot date plus per-page `revid`.

Why not real-time downloading from the wiki:

- the wikitext parser would run on the user's machine against live content: a template renamed
  tomorrow breaks the screen at some stranger's house, with no tests protecting it. At build
  time the same change breaks the suite, and it's caught;
- constraint 4, offline: every detail would need a "no network" state to design for;
- traffic: a thousand users opening ten items each is ten thousand calls a day with the same
  User-Agent, and the Cloudflare block hits everyone at once. One pass per release is thirty
  calls;
- latency: half a second of round trip for every detail, versus zero.

Why not downloading on first launch: it moves the same problem to the worst possible moment
(with no network the app starts empty), leaves the parser on the client, and saves only a few MB.

What goes into the dataset, **text only**: for items and trinkets, the five sections as a tree
with references resolved to ids; for bosses, Behavior, Notes, Strategies, Bugs, base HP, and
where they appear; for achievements, the resolved condition and the `link`; for challenges and
characters, the infobox fields the game doesn't write; the meta with date, `revid`, the last
patch known to the wiki, license, and attribution. Left out: In-game Footage, Gallery, Trivia,
images, and every field `catalog` already knows (quote, quality, tag, pool, prices). Size: the
raw wikitext for items and trinkets is 3.3 MB, more than half of it discarded markup; the
derived form stays under 2 MB uncompressed and under half a MB compressed in the installer.

Settings to plan for: **"Update dataset"** (manual, or a startup check, toggleable and off by
default, that compares `builtAt` and `schemaVersion` and stays silent if there's no network),
the **"open on the wiki" link** on every detail as a universal escape hatch, and the
**freshness diagnostic** against the `appmanifest`, which informs without blocking.

## What's left out and who decides it

- **The implementation task** has its own spec: the build script's language (Rust in the
  workspace, or Node in `ui/`), the dataset format (JSON in the package, or SQLite if B3 wants
  full-text search), the shape of the tree and of references over the IPC, handling of tables,
  of `{{dlc}}`, and of rarer templates (`{{m}}`, `{{r}}`, `{{p}}`, `{{tf}}`). It doesn't start
  before design, same as B3: the item detail is a screen.
- **Italian**: the sections stay in English. The only path to Italian today is EID, and it
  depends on a reply from the author.
- **The irregular cases found in the dump**, listed so they don't surprise the parser: an item
  page with a trinket infobox, an `id = -1`, the 97/113/297/530 duplicates across variant
  pages, and 102 item pages with no `Effects` section (mostly a single line under `Effect`, or
  familiars).

## How the verification was done

Throwaway `curl` calls and Node scripts in the session's working folder, with the
`User-Agent` `IsaacDome-analysis/0.1`: `meta=siteinfo` on five wikis, `action=parse` and
`prop=revisions` on *False PHD* on both copies with a `diff` of the two wikitexts, a full dump
of the pages via `generator=embeddedin` on the two infobox templates (19 calls, 911 pages),
`cargotables`, `cargofields`, and `cargoquery` on wiki.gg, comparing ids against `items.xml`
extracted from `repentance.a` with `cargo run -p unpack --example estrai`, and for EID the
GitHub API (repository and contents) plus the raw description files across three tiers and
two languages.
