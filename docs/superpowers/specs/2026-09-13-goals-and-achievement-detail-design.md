# Obiettivi consigliati, and the achievement detail (design)

**Date:** 2026-09-13
**Milestone:** M3's screen work, after 3.5d (`docs/STATUS.md`)
**Branch:** to be cut from `develop` **after `feature/wiki-infobox` merges** — see §8
**Depends on:** the infobox sub-project (`docs/superpowers/specs/2026-09-13-wiki-infobox-design.md`),
which puts `requirements`, `notes`, `description` and `unlocks` on an achievement's entry;
3.5d's blocked menu (`crates/ipc/src/wiki_target.rs`, `lib/graph/whyMenu.ts`, `WhyMenu.vue`);
the graph view store; the plan queue
**Closes:** B32 (the screen is hard to read), B35 (what a node unlocks links to its page)
**Status:** approved section by section in conversation on 2026-09-13.

## 1. What this sub-project is

Two complaints from the owner's review, and they turn out to be one piece of work.

**The landing page is bare.** "Prossimi passi" is `defaultLocation` — the first thing a
player reads — and it is five cards on a wide page. Four things are wrong with it at once:
the page holds too little, the cards say too little (that is B32), five is an arbitrary cut,
and a card is a dead end: from it you cannot go anywhere.

**There is nowhere to read about an achievement.** The app can say a node is blocked and by
what, but not *what the thing is*. The owner wants one place that answers "come lo sblocco",
"cosa ottengo", "dove sta nel grafo" and "cosa ne dice la wiki".

The second answers the first: the card's missing exit is the detail page.

## 2. The decision that shapes everything: the wiki page **is** the detail

Three shapes were weighed in conversation. A dedicated screen at `/progress/achievement/:id`
would leave **two pages for the same thing** — the wiki's own achievement page keeps
existing, nude — and would duplicate `WikiPage`'s rendering. A panel over the list carries
**no identity**: it cannot be a tab, and it would need an exception to the app's one gesture
(click navigates, Ctrl opens beside).

So: **`/wiki?page=achievement:<id>` is the detail**, and it grows a block that says what the
profile knows. Three things fall out of that for free:

- no new route, no new tab identity, no new title resolution;
- **every link that already points at an achievement page becomes the detail** — the blocked
  menu, the Collection's lock, the search palette;
- when "the rest, later" arrives (items, characters), it is one more block on the same page,
  not a second page.

The price is that `WikiPage`, today independent of the profile, takes a dependency on the
graph store. It degrades: no profile, no block (§3.3).

### 2.1 The boundary — who says what

The infobox sub-project already puts the wiki's answer on the page: `description`,
`infobox.requirements` (the condition in the wiki's words), `infobox.notes`,
`infobox.unlocks`. **The profile block repeats none of it.**

| the wiki page (already there) | the profile block (new) |
|---|---|
| what it is, what it asks, its caveats | **where you stand**: state, what is still missing, how much it opens, the one action |

Where the two sources disagree — `infobox.unlocks` (the wiki) against `node.unlocks` (the
catalog) — **the catalog wins** in the block: it is the source the graph is built on. The
wiki's claim stays in the wiki's own block, unedited. Neither is corrected against the other,
because a disagreement between them is a fact about the dataset and not an error to hide.

## 3. The block: "Il tuo profilo"

In `WikiPage.vue`, between the header and the infobox, drawn only when
`target.kind === 'achievement'`.

### 3.1 What it holds, in this order

1. **State** — `NodeStateBadge`, unchanged.
2. **What is missing** — the groups of `node.missing`, drawn as **rows, not a menu**. The
   menu exists because a badge is small; a page has room. Same model (`missingGroups`), same
   links (`pageLocation`), same gesture. A requirement with no page is plain text, never a
   link that leads nowhere.
3. **What you get** — `node.unlocks`, each with its drawing, its name, and a link to its page
   (§6.2).
4. **How much it opens** — the fan-out as a sentence, and `stepsMissing` when it is blocked.
   Never a bare number under the word "sblocca".
5. **The action** — add to the Plan, or "già in coda". The same button as the card, so the
   two screens cannot drift.

A node that is `partial` keeps its block: the state badge says `partial` and the missing rows
say what could not be read. It is never drawn as unlockable — the rule `GraphInfo` already
carries.

### 3.2 The join needs no new command

`UnlockView.nodes` already carries state, `missing`, `unlocks` and the graph info for every
achievement, and the graph store already holds it for Next steps and Unlock alike. The block
is a **lookup by achievement id** over a view-model that is already resolved — not a join of
files, which would belong in Rust. A memoized `Map` built once per view, so a page visit is
not a scan of 642 nodes.

### 3.3 Degrading

Four cases, all of them "the block is absent and the page is exactly what it is today":

- no active profile (the wiki is reachable without one);
- the graph view failed to load, or has not been loaded in this window;
- the page's id is an achievement the catalog does not know;
- the node exists but the save's achievement section was not read (`noAchievementSection`) —
  the state would read "not done" for everything, which is a lie. The diagnostic is already
  on `UnlockView`; the block says it cannot tell rather than showing a false state.

The block never blocks the page: the wiki content renders whether or not the graph answers.

## 4. "Obiettivi consigliati" — the page

### 4.1 The name and the voice

The route, the sidebar entry and the tab label become **"Obiettivi consigliati"** (B32 §1):
the five are independent, and "passo" implies a sequence that is not there.

The intro is **one sentence in the player's words** — what the list is, not how it was
computed. The current intro ("Al massimo cinque righe, tutte sbloccabili adesso: le cinque
che aprono più cose a valle. Un nodo che il grafo sa dire solo parziale non è un passo…") is
the engineer's, and it moves to the About dialog's promises, where the computation belongs.
Empty states get the same treatment: no catalogue, nothing unlockable now, everything done —
each in the player's voice.

### 4.2 The body is grouped by **reason**

That is the answer to "five is few": not more rows of the same thing, but rows that say why
they are there.

| section | what it holds | ordered by |
|---|---|---|
| **Aprono di più** | `availableNow` nodes, as today | fan-out desc, id asc |
| **Ci sei quasi** | `availableNow` nodes whose `missing` is non-empty and **entirely `counter`** | remaining (`atLeast - current`) asc, then fan-out desc, id asc |

**A node belongs to exactly one section.** "Ci sei quasi" claims first, because it says more
about the node than its fan-out does; "Aprono di più" takes the rest. A row that appeared
twice on a short page would read as two different suggestions.

**Why counters and not marks.** A node held only by marks is `availableNow` too, but a mark
is binary: there is no distance to be near. Only a counter carries `current` and `atLeast`,
so only a counter can order a list by closeness. Mark-only nodes stay in the fan-out section.

**A section that would be empty is absent** — never a heading over nothing. On a fresh
profile "ci sei quasi" may well be empty, and that is a correct page, not a broken one.

Each section ends with **"vedile tutte"**, which opens Unlock with the matching filter
already applied. Unlock is the exhaustive list; this page stays a recommendation and stops
competing with it.

### 4.3 The third block: "Nel tuo Piano"

Under the two sections, the first rows of the queue that are unlockable now, compact, with a
link to the Plan. The landing page then says both what the app suggests **and what you had
decided**, which is the half that gives the page a body.

This block is **not** part of the `NextSteps` contract: the queue is another store and
another command. It is composed on the frontend, so the Rust view-model keeps meaning exactly
"what the graph recommends". When the queue is unreadable or the store is unavailable, the
block is absent — the two sections above it are unaffected.

## 5. The card

Top to bottom: **what, how, why, action** (B32 §3).

- **The headline is what you get** — the target in its own form ("Tainted Lost", "Samson"),
  not the achievement's text as the file writes it. B28 is why: on the reference profile the
  five rows were all Tainted characters, every one of them labelled with its base name, and
  that is where the owner's confusion started.
- **Under it, the condition in one line** — `achievement.hint`, which is the game's own
  `unlock_condition` and is already on the contract. No IPC change, and no wiki read: the
  card is a recommendation, the wiki's fuller answer is one click away on the detail.
- **The drawing**, as today — its backing is B33 and stays out.
- **Why it is worth it** — "apre altre 23 cose" as a sentence.
- **The action** — add to the Plan, or "già in coda".
- **The whole card is a link to the detail**, with the app's one gesture: click navigates,
  Ctrl opens beside. The add button lives inside the card and must not navigate — the click
  that adds to the Plan stops there.

**The state badge goes.** In a list whose every row is unlockable now, it says nothing. It
stays where it means something: Unlock, the Plan, and the detail block.

## 6. The contract

### 6.1 `NextSteps` becomes sections

```rust
pub struct NextSteps { pub sections: Vec<StepsSection> }
pub struct StepsSection { pub basis: StepsBasis, pub steps: Vec<UnlockNode> }
pub enum StepsBasis { FanOut, Closeness }
```

`StepsBasis` was built to grow exactly here — its comment already names closeness as the next
basis. `STEPS = 5` stays the cap **per section**, so the page shows at most ten rows and the
recommendation stays a recommendation.

A section with no steps is not emitted, so "absent" is one fact, decided in Rust, and not two
screens each deciding what an empty array means.

### 6.2 `UnlockTarget` gains its page (B35)

`page: Option<Target>` on all four variants, `Some` only when `Dataset::entry` answers — the
same rule `RequirementView` already follows, through the same
`crates/ipc/src/wiki_target.rs`.

This rides along because without it **a boss is not linkable**: its page key is an entity
triple taken from the portrait's file name, and the frontend cannot construct it. Building
the other three frontend-side and leaving the boss as text would put one rule in two places.

B35 also asked for a decision on the gesture. It is settled here: **a single target is its
own link — the name is the link**, not a menu of one. The menu shape belongs to the blocked
badge, which names a group.

### 6.3 The mirror, and the hand-off

`ui/src/lib/ipc/types.ts` is hand-mirrored and changes with it. The IPC contract is live
material: this change is **handed on, not merely committed**. It also adds to what N7 (the
generated TypeScript) will have to absorb, and N7 is already waiting on
`feature/wiki-infobox`.

## 7. Testing

Rust, properties over values (`crates/ipc/tests/`):

- every node in every section is `Computed { available_now: true }` — a `partial` node is
  never recommended;
- every node in **Ci sei quasi** has a non-empty `missing` whose entries are all `counter`;
- the two sections are **disjoint** by achievement id;
- the order is deterministic across two calls on the same profile;
- a section that would be empty is not emitted;
- **vacuity guard**: on the real series, assert that the closeness section is actually
  populated on at least one sample — a property about "ci sei quasi" holds trivially on a
  profile that has none, and a test that cannot fail reports coverage that is not there. If
  no sample produces one, the test says so on stderr and skips with a note, the way every
  test on real data does.
- `UnlockTarget::page` is `Some` exactly when the dataset has the page, for each of the four
  kinds, read from the catalog rather than from a literal table.

Frontend (Vitest), on the pure parts:

- the lookup from a page key to its node, including the four degrading cases of §3.3;
- the card's model as a pure function — headline, condition, the "apre altre N cose"
  sentence — so the component holds no decision;
- the detail block's groups and their links, reusing `missingGroups`;
- the "Nel tuo Piano" selection.

Presentation: the Kit page, and `pnpm ui:dev` fixtures. The existing `?fixture=` and `?queue=`
outfits must still answer, and a fixture for a profile with an empty "ci sei quasi" is added —
the empty-section case has to be lookable-at without a real save.

## 8. Branch, and what is out of scope

The branch is **`feature/screens-goals-detail`, cut from `develop`** on 2026-09-13, one
sub-project per branch as decided on 2026-09-11.

This paragraph first said the work had to wait for `feature/wiki-infobox` to merge, because it
"builds on the very fields that branch adds". That was assumed, not checked, and it is wrong:
`git diff develop...feature/wiki-infobox --name-only` touches **no frontend file at all** —
`crates/wiki`, `crates/catalog`, the graph's rules, `dataset/wiki.json` and docs. The infobox
this design leans on — `Entry.description`, `Entry.unlockedBy`, `Infobox::Achievement`'s
`requirements` and `notes`, and `WikiInfobox.vue` inside `WikiPage.vue` — is already on
`develop`. What that branch is still doing is teaching the wikitext parser more templates,
which changes the dataset's *content* and not the page's shape. There is no file in common,
so there is nothing to wait for.

Out of scope, on purpose:

- **B33**, the achievement drawing's backing — a measurement in the game's files, unrelated
  to what the card says;
- **B36**, a mark and a counter linking to their boss — it needs a measurement first, and the
  block draws them as text meanwhile, exactly as the menu does today;
- **the profile block for items, characters, bosses and challenges** — decided in
  conversation: achievement now, the rest later. The block is written so that adding a kind
  is adding a component, not reshaping the page;
- **the detail as a screen of its own**, rejected in §2.
