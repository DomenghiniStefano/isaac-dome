# Obiettivi e il Piano diventano una schermata

**Status:** design agreed in conversation on 2026-09-22. Six decisions are the owner's and are
marked as such; the rest follow from them or from what the code already does.

Two screens answer halves of one question. *Obiettivi* says what is worth doing and *Piano* holds
what you decided to do, and the only thing joining them is a text button at the bottom of a
section. This cycle makes them one screen with two panes: where rows come from on the left, the
queue you own on the right.

**It is the first of four.** The owner's ask was "make every screen presentable", which is
seventeen routes and not one spec. The decomposition agreed in conversation:

- **this one** — Obiettivi + Piano, the pair that shares a model (node, queue, prerequisites) and
  the pair named as the worst of the lot. It also fixes the row vocabulary the others inherit.
- **next** — Sblocchi, Collezione, Sfide: the three filterable grids.
- **then** — Live, Partite, Floor: the Tool section.
- **last** — Wiki, Ricerca, Impostazioni.

Nothing here is a Rust change. §7 says why, and what that buys.

---

## 1. What is already built, read rather than assumed

- `GoalsScreen.vue` (202 lines) draws `steps.sections` — one block per reason, the `basis` decided
  in Rust — as a flat stack of `GoalCard`s, plus a *"nel tuo piano"* section reminding you of at
  most five queued rows, plus `WantBar` / `WantAnswer` for B37's "voglio X".
- `PlanScreen.vue` (103 lines) draws a line of three counts, `QueueCard` with the draggable rows,
  and `ProposalAside` — which is *Obiettivi* again, in a column, with no sections.
- The queue's move rules, the prerequisite wall, the drag, the diagnostics and the footnotes all
  work and are tested. **None of them is touched here.**
- `useDragList` reorders within one list. It has no notion of a drop coming from another list.
- `sessionDocument.ts` validates a stored tab's route name and **drops the single unreadable tab**,
  keeping the window: its own comment says *"eight tabs do not vanish because one screen was
  renamed"*.

### 1.1 The measurement this cycle starts from

`ui/src/assets/theme/typography.css`: `--text-row` is **0.8125rem (13px)**, `--text-caption`
**0.75rem (12px)**, `--text-label` **0.6875rem (11px)**.

A queue row carries eleven pieces of information — grip, position, art, title, condition, *voluto*,
*serve a «…»*, *sblocca …*, state, fan-out, *fuori dalla coda*, remove — and every one of them is
drawn inside that 2px band. The hierarchy is not weak; there is none. The eye is given no place to
start, so it reads all eleven or none.

**This is the defect, stated as a number**, and every decision below is a consequence of it.

## 2. Decision (owner) — one screen, two panes

*Obiettivi* and *Piano* stop being two locations. One screen, `ScreenHeader` as every other screen
has it (icon, title, the help tooltip), then:

- **left, "Da aggiungere"** — fixed width, the search bar always on top, and below it whatever
  answers "what could go in the queue";
- **right, "La tua coda N"** — takes the rest, and **never changes content**. It is the fixed point
  of the page: you can see what you decided while you decide what to add to it.

Side by side above `--container-wide`, stacked below it. That is the rule `PlanScreen` already
applies (`@wide/page:flex-row`), so no new threshold is introduced.

The left pane's width has a token already: `--spacing-plan-aside`, **15.5rem**. Two things about
it. It is to be **renamed** — *aside* is what that column was when it was a secondary proposal
beside the queue, and it is now the only way into it, so the name would be a wrong label of the
kind this repo re-derives things over. And 15.5rem was measured for a column of proposals, not for
one that also has to hold a search field and its results: whether it still holds is measured during
the build, and if it moves, the number and how it was arrived at go in the report.

## 3. Decision (owner) — the want is the left pane's third state

B37's "voglio X" does not get a band of its own. The left pane is *where rows come from*, and a
want is one more way in:

| what is typed | what the left pane holds |
|---|---|
| nothing | the recommendations, grouped by reason (`steps.sections`) |
| something | the answer — *per arrivare a X*, its chain, and the offer to queue it |

The search bar stays visible in both. The right pane does not move in either.

`?want=` is unchanged: it already lives on this route, so every existing link keeps working
without a line being written for it.

## 4. Decision (owner) — the row is one shape, and it does not change with the window

The row carries **five things**: grip · position · art · name · *apre N* · chevron. Everything else
lives in the expansion, which opens on the row you are looking at and nowhere else: the game's
`unlock_condition` when the file states one, the badges (*voluto*, *giocabile ora*,
*serve a «…»*, *N fuori dalla coda*), and the remove button.

The left pane uses the same row with **`+` instead of the chevron**, and without grip or position —
recommendations are not ordered.

> **An earlier decision, reversed in the same conversation, and recorded because the reasoning is
> worth keeping.** The first answer was that the row should alternate between two densities — the
> minimal one when narrow, one carrying a grey metadata line when wide. Two objections were raised
> and the owner withdrew it:
>
> 1. **It would have read the wrong width.** At `wide` the page splits into two panes, which makes
>    the queue pane *narrower*, not wider. Keyed to the page, the row would have expanded exactly
>    when it lost room. It would have had to read its own pane — which `containers.css` already
>    states as the rule: *"a component reacts to its own width, not the window's"*.
> 2. **They are not two layouts, they are two interaction models.** One has a disclosure and the
>    other does not, so the gesture would appear and disappear as the window is dragged, and an
>    already-open row would have to close itself on a resize.
>
> One density. There is no state that changes under the user's hands.

### 4.1 `apre N` carries the state in its colour

Amber when the row is playable tonight, grey when something stands in front of it. One value
carrying two readings, and the row does not grow to say it. The full state stays a badge in the
expansion, where it can be a word instead of a colour — a colour alone is not an accessible answer,
which is why it is never the *only* place the state is said.

## 4.2 Decision (owner) — the screen opens on the band the other two already have

Added on 2026-09-22, after the rest of this document, on the owner's ask to follow the style the
last cycle set. `CompletionHero.vue` (card #58) and `WikiHero.vue` established a grammar: a header
the full width of the page box — the screen hands it the gutter with `-mx-5.5` rather than the band
taking it — lit from the corner (`hero-wash`), grained where a skin has no shadow (`hero-grain`),
and **the screen stops scrolling as one block**: the band stays put and what is under it takes the
height that is left and scrolls inside itself.

Obiettivi takes that grammar, and **only that**. The band holds the `ScreenHeader` and nothing
else: no headline number, no progress bar. §5 below declined a count at the top of this screen and
that decision stands — what was declined was the *number*, not the treatment, and a band exists
here to make three screens look like one product, not to find something to put in it.

> **A drift this uncovered, fixed in the same cycle.** `docs/frontend-conventions.md` lists a
> screen as one of two shapes, *flowing* or *filling*, and files Completion under *flowing*. It has
> not been either since card #58: it is `-mx-5.5 flex h-full min-h-0 flex-col overflow-hidden` with
> a band and a filling body. The document was not updated then. This cycle adds the third shape to
> that table and lists both screens under it — a contract that describes two of three shapes is
> read as forbidding the third.

## 5. Decision (owner) — no band of counts

`righe: 12 · voluti: 4 · trascinati: 8` goes, and nothing replaces it.

The split between *wanted* and *pulled-in* is real and it matters **per row** — a wanted row can be
removed, a pulled-in one leaves when its wish does — but as a total at the top it answers no
question anybody has. Two alternatives were offered and both declined: a bar drawing the
composition, and a line reading *"5 dei 12 li puoi fare adesso"*.

The count moves to where its subject is: the right pane's heading, *La tua coda 12*. §4.2's band
does not reopen this: it carries the title and nothing countable.

## 6. Decision (owner) — Obiettivi wins the name and the route

`/progress/goals`, titled **Obiettivi** — the player's words for the question, where *piano* is the
name of the tool. `RouteName.Plan` leaves the table. Two things then have to keep working, and they
are not the same thing:

- **A URL.** `/progress/plan` stays in `routes.ts` as a plain path redirect. No name, so it is not a
  location and cannot be a tab.
- **A stored tab.** Tabs are persisted by *route name* (store migration 3). Today an unknown name
  makes `sessionDocument.ts` drop that tab — the window survives, the tab does not, silently. A
  retired-name map in `readLocation` translates `plan` → `goals` **before** `isRouteName` refuses
  it, carrying the query across.

The sidebar's Progress section goes from seven entries to six.

**`docs/architecture.md` is redrawn in the same commit.** Its header pins seventeen routes and
they become sixteen; the route table and the flowchart both lose the `plan` node.

## 7. Decision — no Rust, and that is checked rather than hoped

Everything the new row draws is already on the wire: `graph.fanOut`, `wanted`, `stepsNotQueued`,
`unlocks`, the achievement's condition, the node's state, and `steps.sections` with its `basis`.
No command, no event, no view-model and no migration changes.

The consequence is worth stating: `architecture.md`'s counts of commands, events and migrations do
not move, so the only line of that document this cycle touches is the route count. And a cycle that
cannot change the contract cannot break the four screens that share it.

## 8. Decision (owner) — dragging between the panes is not in this cycle

A row enters the queue with `+` and lands at the end. Dragging stays inside the queue, where it
already is.

The alternative — drag from left to right, choosing *where* it lands — is the gesture that would
make "linked" literal, and it was declined for this cycle on cost: `useDragList` is the composable
every list in the app drags with, tabs included, and a cross-list drop is new logic inside it. It
becomes a card of its own once this shape has been seen in a real window.

## 9. What stops existing

- `ui/src/screens/goals/GoalCard.vue` — replaced by the shared row.
- `ui/src/screens/plan/ProposalAside.vue` — it was *Obiettivi* redrawn without its sections; the
  left pane is that, done once.
- The *"nel tuo piano"* section of `GoalsScreen.vue`, and `planNow` with it: it reminded a page that
  could not see the queue. The queue is now beside it.
- The three-count line of `PlanScreen.vue`.
- `PlanScreen.vue` itself, and `RouteName.Plan`.

`QueueCard`, `QueueFootnotes`, `QueueError`, `WantBar`, `WantAnswer` and every store stay.

`QueueRow.vue` is the one that becomes the shared row, and it **leaves `screens/plan/`**: a
component both panes draw belongs to neither, and leaving it inside one screen's folder is how the
next sub-project ends up copying it instead of importing it.

## 10. How it is verified

Test-first, expected values from this document and never from the code's output.

- **The retired-name map**: a stored tab on `plan` restores on `goals`; one with `?want=` keeps it;
  a name that is neither known nor retired is still dropped alone, and its window still survives.
  This is the one piece where a bug loses something the user had.
- **The route table**: `/progress/plan` redirects; `RouteName` holds sixteen entries and the sidebar
  lists every Progress route (the existing test already asserts the second — it must stay green
  rather than be edited).
- **The row**: what is in the row and what is in the expansion, as §4 lists them; that a row with no
  `unlock_condition` opens an expansion without an invented sentence; that the left pane's row has
  no grip and no position.
- **`apre N`'s colour** follows the node's state, and the state is *also* present as a word in the
  expansion — the test is that the colour is never the only carrier.
- **The left pane's three states** are mutually exclusive and driven by the want, not by
  `steps.sections.length`: an empty section list and an active want are different pages.

What no test covers, and what therefore has to be looked at in a real window before the card leaves
`UAT`: that the two panes hold at the window floor (640×480), and that a queue of thirty rows in
the C shape is actually faster to scan than the eleven-field row it replaces. The second is the
whole point of the cycle and no assertion can make it.

## 11. What this cycle contains

1. The shared row, with its expansion, used by both panes.
2. The left pane: search bar, sections, want answer.
3. The right pane: the queue, unchanged in behaviour, redrawn in the new row.
4. The route merge, the URL redirect and the retired-name map.
5. `docs/architecture.md` redrawn; `README.md` and `docs/PROJECT.md` updated where they list the
   screens.

Out of it, by decision: dragging between panes (§8), any change to the queue's move rules, and the
other three sub-projects' screens.
