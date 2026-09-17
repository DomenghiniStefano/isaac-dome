# 3.11 — the Challenges screen: the forty-five the app never named

B3's other half. The Collection answered "which items am I missing"; nothing in the app has ever
said what a challenge *is*, whether you have done it, or what it takes — while Unlock happily
says `Godhead · Sfida 33` and leaves it there. Decided in conversation on 2026-09-17; the two
choices the owner made are §2 and §3.

## 1. What is already built, read rather than assumed — and three things measured tonight

- **B3 is half closed already.** Its "items" half is the Collection screen (3.x), and its
  "search inside a list" half is the filter bar (3.10). What was never built is the challenges.
  The entry was written on 2026-09-05, before either.
- **`core-save` already reads section 7** (`Kind::Challenges`, 46 cells of one byte) and `diff`
  already reports the cells that newly light up.
- **`catalog` already parses `challenges.xml`**: 45 challenges with `name`, `starting_items`,
  `unlocked_by` (the achievements that gate it) and `rewards` — the last collected by
  `Catalog::build` from the achievements' own notes, because the XML does not state it.
- **Nothing is in front of them.** No `ipc` view-model, no command, no screen. This sub-project
  is all front half.
- **`queue_add` takes an achievement id**, and a challenge's reward *is* an achievement. The Plan
  join therefore costs **nothing new on the wire** — the same shape as M4's Run screen, which
  could be split off for exactly this reason.
- **`ipc::wiki` re-exports the wiki's own `Inline`, `Block`, `Infobox` and `Target`**, and the
  frontend already renders inline content. A challenge's conditions cross as what they are.
- **The Wiki section promises to work with no game and no save** — its own sidebar hint says so.
  A list carrying *your* progress cannot live there, which is what §2 decides. The Wiki's
  `Challenges` category stays what it is: the catalogue.
- **The app has one state vocabulary** — done / now / blocked / unknown, with tokens, a `Badge`
  variant each and `StateToggle`. A fifth reading of "state" would be a second convention.
- **Routes are 14 and commands 30 today**, the counts `docs/architecture.md` pins. This adds one
  of each, so **the diagram is redrawn in the same commit**.

### Measured on 2026-09-17, with `cargo run -p ipc --example probe_challenges`

- **Challenge *n* is cell *n*, and cell 0 is unused.** 46 cells for 45 challenges, the same shape
  as section 3's stages. The instrument is the catalog's reward link: "the cell is set" and
  "every reward achievement is done" agree on **39 of 39** judged rows — **21 both true, 18 both
  false**, which is what makes it a result rather than an empty column — while the off-by-one
  reading breaks 13 of the same 39. Recorded in `docs/save-format.md`.
- **The wiki has a page for every challenge**: 45 of 45 in the committed snapshot. The guard for
  the day that stops being true still has to exist (B45's lesson).
- **Whether `unlocked_by` means "all of these" or "any of these" cannot be settled by this
  instrument, and is not.** 13 done challenges have gates and all 13 have *every* gate done — but
  "all done" satisfies both readings, so the test cannot separate them. It is read as **all of**,
  §4 says why, and the measurement that would settle it is named there.

## 2. Decision — a screen under Progress, and the Wiki's category stays the catalogue

`Sfide`, the fifth entry under **Progressi**, after Unlock. A route of its own
(`/progress/challenges`), a command of its own, a tab like any other.

Not a column added to the Wiki's `Challenges` category: that section answers with no game and no
save, by promise and in its own words, and a progress column would either break that promise or
be empty half the time. The two link to each other instead — a row opens the wiki page, and the
page is the same one the category lists.

Not a block inside Completion either: that screen is one subject, characters × bosses, and a
second subject inside it would make the header's counts ambiguous — which is the defect 3.9 had
just finished removing from it.

## 3. Decision — what a row carries, and what it does

**Says**: the number and the name; the state (§4); what finishing it unlocks — the reward
achievements, with the icon Unlock already draws; and the conditions that decide whether you can
play it tonight, read from the wiki: the **character**, the **goal**, and **blindfolded**.

**Links**: the row opens the challenge's wiki page, where the rest of the infobox already lives —
items, trinkets, pickups, health, curse, shops, treasure rooms. A table row is not a page, and
copying an infobox into a grid is how a list stops being readable.

**Queues**: the reward goes into the Plan's queue with `queue_add`, the command Unlock already
calls. No new write path, no new command, no migration.

## 4. The state a challenge is in, and the one thing that is not settled

| state | when | drawn as |
|---|---|---|
| `Done` | its cell is set | the done tone |
| `Available` | every gate achievement is done, or it has none | the "now" tone |
| `Blocked` | at least one gate is not done — **and it names them** | the blocked tone, "bloccata da N" |
| `Unknown` | section 7 was not read | the unknown hatch, never "not done" |

Eleven of the 45 have no gate at all and are available from the start.

**`unlocked_by` is read as "all of", and the app says so rather than hiding it.** The two readings
differ only for a challenge with *some* gates done: "all of" calls it blocked, "any of" calls it
available. The save cannot tell them apart — it records what you finished, not what the game
*offered* — so the only instrument that can is a machine where such a challenge is checked against
the game's own challenge menu. That measurement is registered in `docs/STATUS.md`.

Reading it as "all of" is the conservative half of a real trade: it can call a challenge blocked
that the game already offers, which hides something to do, where "any of" would send a player to
a menu entry that is not there. The naming makes the error visible either way — a blocked row
lists the achievements it is waiting for, so a reader who has done "enough" of them can see the
claim and disbelieve it.

## 5. The view-model

Mirrors `CollectionView`, which is the same kind of list and already carries the conventions:

```rust
pub struct ChallengesView {
    pub challenges: Vec<ChallengeRow>,
    pub totals: ChallengeTotals,        // slots, challenges, done
    pub diagnostics: Vec<ChallengesDiagnostic>,
}

pub struct ChallengeRow {
    pub number: u32,
    pub name: String,
    pub state: ChallengeStateView,
    pub rewards: Vec<RewardView>,       // id, text, icon_url, page
    pub character: Option<Target>,      // the wiki's, when it names one
    pub goal: Option<Vec<Inline>>,
    pub blindfolded: Option<bool>,      // None: no page, which is not "no"
    pub page: Option<Target>,
}
```

`ChallengeStateView` is **tagged** — `Blocked` carries the achievements it waits for, so the whole
enum is tagged with struct variants, `rename_all` *and* `rename_all_fields`, exactly as `LockView`
is. `ChallengesDiagnostic` is tagged too: `NoCatalog`, `NoChallengesSection`, `NoWiki`.

Every `Option` here means **unread**, never a fact: `blindfolded: None` is "the dataset has no
page for this one", and it must not draw as "not blindfolded". `totals.slots` is section 7's own
length, read from the file and never the constant 46.

## 6. Degradation

- **No game**: no catalog, so there are no challenges to list at all — the same answer the
  Collection gives, with its own diagnostic and the same sentence pointing at Steam.
- **No wiki**: the rows are there without their conditions, and one diagnostic says so once,
  instead of 45 rows each saying nothing.
- **Section 7 unread**: every row is `Unknown`, the totals say `0` done out of a `slots` that is
  itself 0, and nothing reads as "you have done none of them".
- **No profile**: the screen is under Progress, so it follows the same gate the others do.

## 7. The filter bar's fourth list

`FilterBar` from 3.10, with the state row as the state (fatta / da fare / bloccata / non
leggibile), the search over the name, and in view at rest: **personaggio**. Behind the fold: what
it unlocks, and blindfolded. That is the whole filter — four facets, and the bar's shape decides
nothing new.

The one thing it does prove: the fourth screen on the bar was wired from a table and a slot list,
which is what §2 of 3.10's spec claimed and nobody had tested by adding a list.

## 8. What this does not do

- No new write command, no store migration, no change to the queue or the graph.
- It does not put the challenge into the unlock graph as a node: a challenge is not an
  achievement, and `graph` reads achievements. What a challenge *rewards* is already in there.
- It does not copy the wiki's infobox into the grid.
- It does not decide the "all of / any of" question; it names it.

## 9. What only a window can say

- the five states of a row, including a blocked one that names two gates, and an unknown one
- the row at a narrow width: the goal's inline text wraps rather than pushing the reward out
- a reward added to the Plan from here shows up in the Plan's queue, and the row says it is queued
- with `?catalog=none`, the screen says the game is missing and does not read as "no challenges"
- **the conditions, judged against the game's own challenge menu**: is `Bendato, con Cain, fino a
  Mom` what the menu says? This is also the measurement §4 waits on — a challenge with some gates
  done, checked against whether the menu offers it.
