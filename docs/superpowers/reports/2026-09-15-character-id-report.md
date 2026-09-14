# The line that says which character — and Live as a dashboard

**Branch:** `feature/character-id`, on top of `feature/live-dashboard`. `pnpm check` green;
1306 real files touched, 0 unexpected skips.

## The measurement that ended an ambiguity

Live had been saying *"il log scrive «Cain», e il gioco chiama così 2 personaggi"* — because the
item line gives a name and the game gives a Tainted character the base form's name. The spec
registered the measurement that would replace it. The owner asked for it, and the logs on this
machine answered in one grep:

```
[INFO] - [Frame: 0] Initialized player with Variant 0 and Subtype 3
```

**The subtype is the character's own id.** The solo Judas run says 3; the online session says
30 and 7 — Tainted Eden and Azazel, the two players of one run. A Tainted character has its own
subtype, so the two forms are told apart by the game itself.

**What the fold had to learn, and it is not obvious.** The line arrives **before** the seed on a
solo run and **after** it online. A fold that only looked forward would lose every solo run; one
that only looked back, every online one. So an init with no run yet is held, and taken by the run
that starts next — which clears it, or nobody would ever play a second character. In co-op the
line repeats per player and the **first** is the run's: the others are other people at the same
table.

The rules version goes to **2**, so the store re-folds instead of serving runs produced before the
line was read.

## What it changed upstream

`RunView` carries `characterId`; the join asks the catalog for *that* character; the ambiguity
diagnostic survives only for runs folded before this existed. The refusal to infer the form from
the starting items — written twice, in the spec and in the code — is untouched and now
unnecessary, which is the ending that rule was waiting for.

## Live as a dashboard

Four readings across the top, the run with its items as sprites, the completion row of whoever is
playing, and the offers with what each opens in turn. Three facts were added to the wire for it:
an item's **sprite**, an offered achievement's **fan-out**, and the **matrix row** — copied from
the matrix, never rebuilt, because a second reading of the same counters would be a second chance
to disagree with the screen that draws them all.

**Two defects of our own, both found by looking rather than by testing.** `size-icon-row` is not a
token, so the character's head was an unsized sprite — the same failure as the want suggestions
hours earlier, from the same cause. And `liveAnswer()` had **no return type**, so TypeScript never
compared the fixture with the contract: it sat in the old shape, the screen threw on it, and
`pnpm typecheck` stayed green through all of it. **A fixture nobody typed is a fixture nobody
checks.**

## Links and tooltips

One chip for everything a run names — character, items, offered achievements — carrying the
picture, the page it opens, and a tooltip with what the app knows beyond the name: the id the log
carried, and for an achievement the game's own line about how it is earned. A chip with no page
stays a chip rather than becoming a link that goes nowhere.

## What is still not seen

Neither screen has been looked at against the real archive: 28 sessions for Run, and for Live a
run in progress with the app open. The fixtures carry the shapes; the machine carries the data.
