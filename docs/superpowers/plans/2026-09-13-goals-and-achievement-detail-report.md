# Obiettivi consigliati and the achievement detail — report

**Date:** 2026-09-13
**Branch:** `feature/screens-goals-detail`, cut from `develop`, in the worktree
`C:\Projects\isaac-dome-goals`
**Spec:** `docs/superpowers/specs/2026-09-13-goals-and-achievement-detail-design.md`
**Plan:** `docs/superpowers/plans/2026-09-13-goals-and-achievement-detail.md`
**Closes:** B32, B35

## What landed

Two complaints turned out to be one piece of work: the landing page was bare, and there was
nowhere to read about an achievement. The second is the first's missing exit.

- **An achievement's wiki page is its detail.** `/wiki?page=achievement:<id>` grows a
  "Il tuo profilo" block: state, what is still missing with links, what you get with links,
  how much it opens, and add-to-Plan. No new route, no new tab identity — and every link that
  already pointed at an achievement page became the detail for free.
- **"Obiettivi consigliati"** replaces "Prossimi passi": a name a player understands, an intro
  in their words, and a body grouped by **reason** rather than one flat five.
- **The card leads with what you get** — "The Lost", not *You unlocked "The Lost"* — with the
  condition under it, "apre altre N cose" as a sentence, and one action.
- **`UnlockTarget` carries its wiki page** (B35), which is what makes "cosa ottieni" clickable
  on both the block and the card.

## What the execution measured, and what it changed

Four things were not known when the plan was written. Each changed the work.

### 1. The wait on `feature/wiki-infobox` was imaginary

Spec §8 said this sub-project had to wait for that branch, because it "builds on the very
fields that branch adds". Assumed, not checked. `git diff develop...feature/wiki-infobox
--name-only` touches **no frontend file at all**: `crates/wiki`, `crates/catalog`, the graph's
rules, `dataset/wiki.json` and docs. The infobox this design leans on was already on
`develop`; that branch is teaching the wikitext parser more templates, which changes the
dataset's content and not the page's shape. Corrected in the spec and in the plan before any
code was written.

### 2. The closeness section is real, but not on the reference profile

The vacuity guard fired on its first run, exactly as the plan warned it might — and the reason
was not the one it guessed. On `live` (379 of 642 done) the whole view holds **zero `Counter`
requirements**: not zero closeness steps, zero counters. A counter is only reported while
`current < at_least`, and that profile crossed every one of those thresholds long ago.

Rather than accept a skip, the question was taken to a different point of the progression. The
co-op partner's profile — the one `online_logs\` leaves at the start, which `CLAUDE.md` names
as the only place a table of zeros can be tested — holds **4 counter requirements and 4 nodes
held by nothing else**. So:

- `a_young_profile_has_a_closeness_section_and_it_is_ordered_by_distance` asserts the property
  where the thing it is about exists, and **panics** rather than skips if it disappears;
- `the_reference_profile_has_crossed_every_counter_threshold` pins *why* the reference profile
  has no such section, so that "it is empty" stays a fact about the profile and never becomes
  an unnoticed fact about the code.

A third thing surfaced in the same probe and is **not** acted on here: 49 of the reference
profile's `available_now` nodes carry a standing requirement, and almost always a *character*
rather than a mark or a counter. The closeness rule (every standing requirement is a counter)
is unaffected, but "unlockable now with something still missing" is far more common than the
design assumed. Recorded here because the next screen that draws that combination will meet it.

### 3. The card's "how" line could not come from the game file

Measured on the reference profile's 637 known achievements: the game states an
`unlock_condition` for **283** and nothing for 354 — and among the **119 unlockable now**, the
pool this screen draws from, for only **16**. The plan chose `hint` to avoid a wiki read; on
this screen that choice leaves the middle of "what / how / why / action" blank seven rows out
of eight, on the very five rows that motivated B32.

B32 had anticipated it ("the wiki's requirement where the file has none"), so the fallback was
built. With the dataset answering, **637 of 637** achievements carry a line.

Three decisions inside that change are worth keeping:

- **`crates/wiki` gained `plain`, it did not gain a copy.** A private `flatten` already existed
  in `infobox.rs` — "the words of an inline run, as the reader would read them" — used for one
  infobox field. It is now public API with its own tests, and the private copy is gone.
- **The field was renamed `hint` → `condition`.** Widening `hint` in place would have been the
  worst kind of change: same wire type, new meaning, nothing to catch it. The rename forced
  every reader to be revisited — and two nobody had listed turned up, `QueueRow` and
  `UnlockRow`, which printed it under the label *"indizio del gioco:"*. That label now reads
  *"come si prende:"*, because it is no longer always the game's.
- **Unlock's search got better for free**: `unlockFilter` matches on that field, which is now
  populated for every achievement instead of 283.

### 4. Looking at it found what the tests could not

- **The Kit page had no Pinia.** The block brings in `NodeStateBadge` → `WhyMenu`, which asks
  the tabs store where a click would go, and the Kit crashed on first paint. The Kit now gets
  Pinia — its "Componenti app" half exists to host exactly these components, and the tabs store
  is pure state, so a click there moves a tab nobody draws.
- **The block was in the wrong branch of the page.** It sat inside `v-else-if="entry"`, i.e.
  only where the wiki knows the page. But what the profile knows does not depend on the
  dataset: on a page the dataset has never heard of, the state, what it unlocks and the Plan
  button are all still true. It now sits above, outside.
- **"Prima servono ancora 0 sblocchi" under a "Già fatto".** The condition read `!availableNow`,
  and a node that is *done* is not available either, so `stepsMissing` (0) was drawn. It now
  reads the real state.
- Two matters of language: a done node said *"Sbloccarlo apre altre 23 cose"* in the future
  tense, and two adjacent link-styled targets read as one name ("Cube of Meat Ball of
  Bandages").

## One deliberate deviation from the spec

Spec §5 says "the whole card is a link to the detail". It is not: **the headline and the
drawing are the links**, and the rest of the card is not clickable. A card that is itself a
link and also contains a button is a keyboard trap, the repo has no precedent for one (every
row navigates through an explicit control), and `Ref` is already the app's way of saying "this
goes to a page". Nothing is lost: the headline is the largest text on the row and the obvious
target.

## Tests

Rust — properties, never values read back from the code:

- every node in every section is `available_now`; a `partial` node is never recommended;
- a closeness step's requirements are all counters, and the sections are disjoint;
- a section that would be empty is not emitted;
- `UnlockTarget::page` is `Some` exactly when the dataset has the page, for all four kinds,
  read from the catalog and not from a literal table;
- the condition's two sources, with the game's words winning wherever the file has any.

The real-data tests ran on real samples — `sample:` on every one, no silent skip — because the
worktree's `samples/` is linked to the main copy's, `samples/packed` included.

Frontend (Vitest): the page → node lookup and its six `null` answers, what a node unlocks as
linkable rows, the card's model, and the Plan block's selection.

Looked at, not only asserted: the Kit page's four states of the block, the achievement page
with and without a dataset entry, and the landing page under `?fixture=active`,
`?catalog=none`.

`pnpm check` on the merge candidate: all green, **53 frontend test files / 345 tests**, 1274
real files touched, and **7 skips — every one of them named**, none of them this
sub-project's: two dated samples nobody has (`20240118`, `20250112`) and one series with a
single entry, where comparing two snapshots needs two. The count is written down because "N
passed" does not say how many were silent.

## Left open

- **The design pack is five fields behind.** `unlock.json` and `next_steps.json` predate the
  character's form, the requirement's page, the target's page, the steps' sections and the
  resolved condition. All five are backfilled in the fixtures with a console warning, so the
  development screens are poorer than the app in ways that are declared rather than mysterious.
  One `pnpm design:export` on a machine with the game clears all five.
- **B33** (the drawing's backing) and **B36** (a mark and a counter linking to their boss) stay
  out, as the spec said.
- **The profile block for items, characters, bosses and challenges**: the block is written so
  that adding a kind is adding a component, not reshaping the page.
- **A counter requirement shows its name and not its distance** in the block and in the badge's
  menu ("Hush", not "Hush 0/1"). The model is shared with the menu, where there is no room for
  the number; on a page there is. Not changed here because it would change both.
