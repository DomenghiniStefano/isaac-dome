# Floor F1 — the painted grid and the cited rules: report

**Plan:** `docs/superpowers/plans/2026-09-15-floor-grid-and-rules.md`.
**Spec:** `docs/superpowers/specs/2026-09-15-floor-secret-rooms-design.md`.
**Research report:** `docs/superpowers/reports/2026-09-15-secret-room-rules.md` (Task 1).
**Branch:** `feature/floor-grid`, cut from `develop` at `818b4e0`.

---

## What is on screen

A 13x13 grid you paint with the floor you are looking at, and the empty cells light up where
the game's own documented rules allow a Secret, Super Secret or Ultra Secret room. Every lit
cell names the rule that lit it, quotes the sentence, and gives the page.

Nothing here reads a save, a log or the network. `floor_candidates` is a pure function of the
drawing: the same grid always answers the same thing.

**The quotation on screen is not decoration.** The rules are quotations from a CC BY-SA 4.0
wiki, and showing the sentence with its URL is how the attribution reaches the person reading
the screen rather than stopping at a file nobody opens.

## What the doing corrected in the plan

**1. The three wiki pages are one page.** `Super_Secret_Room` and `Ultra_Secret_Room` are two
`#REDIRECT` stubs of 44 bytes each, both pointing at `Secret Room`. The plan's draft
`placement.json` cited `/wiki/Secret_Room` for all seven rules, which read like carelessness
and was in fact correct. Checking it cost one request and would have cost a re-derivation
later.

**2. The rules file has nine rules, not seven.** Task 1 was written against
`?action=raw` rather than a summarizer, and the raw wikitext carried two constraints the
plan's list did not have:

- `secret-neighbours-one` — *"1 neighbor locations can only happen if there are no valid 3+
  neighbor locations, and are very rare."*
- `super-secret-not-next-to-secret` — *"cannot be connected to the Secret Room"*, the second
  half of a sentence the plan quoted only the first half of.

The plan's own instruction for Task 4 is *"do not write a rule the report does not have, and
do not leave one out"*, so both are in.

**How the summarizer would have lost one of them.** A `WebFetch` on the same page returned
*"Regular Secret Rooms are usually located next to 3 or 4 rooms"* — the first half of a
sentence that continues *"while Super Secret Rooms can only be next to one room"*. For a file
whose only purpose is to be citable, a silently truncated quotation is the whole failure.
Nothing in `placement.json` passed through one.

**3. `Constraint` gained a variant, and the solver gained a phase.** This is the one real
design change, and it comes from reading two sentences next to each other:

| | |
|---|---|
| 2 neighbours | "rare but possible, **even when** there are locations with 3+ neighbors available" |
| 1 neighbour | "can **only** happen **if** there are no valid 3+ neighbor locations" |

They are not the same shape. The first is unconditional and merely ranked lower; the second is
a fallback that a better cell switches off. Encoding both as a plain `neighbourCount` at ranks
1 and 2 would have asserted, on every floor with a 3-neighbour cell, something the second
sentence denies.

So: `NeighbourCountFallback { allowed, rank, superseded_by_at_least }`. The `3` is a number the
sentence states, not a probability band anybody named — the same refusal the plan made when it
deleted `Tier`.

And the word **valid** in that sentence is why it needed a phase rather than a match arm. "No
valid 3+ location" means none that survived `secret-forbidden-neighbours`, which is not known
until the narrowing rules have run. A cell with four neighbours one of which is the Boss Room
was never a valid location, so it must not switch the fallback off:

| phase | rules | effect |
|---|---|---|
| propose | `neighbourCount`, `neighbourCountFallback` | add cells with their rank |
| narrow | `forbiddenNeighbour`, `neighbourNotSpecial`, `deadEndDistanceRank` | remove and re-rank |
| resolve | the fallbacks again | drop their cells if a **surviving** candidate has enough neighbours |

Proposing the fallback in phase 1 rather than phase 3 is what subjects its own cells to the
narrowing: a fallback cell next to the Boss Room is not a candidate either, and a phase-3
proposal would have skipped that filter with nothing going red.

**That last claim was checked, not reasoned about.** `a_three_plus_cell_the_boss_room_rules_out_does_not_switch_the_fallback_off`
was mutated against a solver that judges the fallback on the raw grid: exactly one test went
red, the other eleven stayed green. The test bites on the mistake it was written for, and no
other test covered it.

**4. The Start Room's membership is unstated, and the screen says so.** §4 of the research
report sources the Special Room list **structurally** — it is the `Rooms` page's own section
hierarchy, not a sentence about it. The Start Room appears under neither `== Normal Rooms ==`
nor `== Special Rooms ==`; on the whole page it exists only inside sentences about other rooms.

It is not pedantry: it decides a cell. A dead end hanging off the start room is a Super Secret
candidate if Start is not Special and is not one if it is. `SPECIAL_KINDS` therefore does not
carry `Start`, with a comment saying the membership is unstated rather than decided. What would
close it is a measurement, on the machine with the game.

**5. Three token families the plan assumed and this repo does not have.** The plan said to
follow `colors.css` "for light and dark", to use `--size-floor-*`, and to lean on
`--color-chart-*`. All three were wrong about this repo, and the repo won each time:

- there is **one theme**, the dark one, with no `:root`/`.dark` indirection (the design
  system's Decision 1);
- dimensions are `--spacing-*` — there is no `--size-*` family, and a second name for one idea
  is what the cleanup pass spent itself on;
- there is no `--color-chart-*`, so the three ranks got a family of their own, in a hue
  deliberately outside the brief's state colours. Green, gold, brown, grey, red and purple
  already mean done / unlockable now / blocked / unreadable / unexpected / challenge, and none
  of those is "a cited rule allows a secret room here".

**`ring-2` is in no file in this repo**, which the plan half-suspected and told the reader to
check. A candidate is drawn by its **fill**, which needs no new border token and reads at a
13x13 scale where an outline would not.

**6. A cell is the `Button` primitive with a new variant**, not a styled `<button>` — rule 4.
`ButtonVariant.Cell` carries no fill and no edge (169 borders are a lattice the eye reads
before the answer) and `ButtonSize.Cell` is the token square; the fill is the caller's, because
what a cell means is the answer the screen computes and cva cannot express a colour that
changes per cell.

## What the grid does not judge, out loud

`ultra-secret-connections` is `Unmodelled` and reaches the screen as such. The rule is stated
**two hops out** — "connect to 3+ non-red rooms *through its adjacent red rooms*" — and a red
room is the Red Key mechanic, created by an item during a run. This grid paints rooms that
exist, not rooms an item could create.

A dropped constraint would read as "nothing in the way", which is the reading this repo has
already paid for once. §3 of the research report lists the seven other claims that were looked
for and are not rules, each with why.

## One thing the spec called an inference and the wiki states outright

The spec's §2 carried "that the grid is 13 wide … is an inference, not a measurement". The
Ultra Secret note says *"locations on the **13x13 border** where a red room would normally open
to an I AM ERROR room are allowed"*. The width is now cited; **the centre is still an
inference**, because 84 being `6*13 + 6` also needs the unstated assumption that a run starts
in the middle. §2 was corrected in Task 1's commit to say which half is which.

## The session this branch was built in

Worth recording, because it changed how the work had to be done rather than what it produced.

**Three Claude sessions shared one working directory.** Seven commits into this branch, a third
session ran `git checkout -b` in it: HEAD moved off `feature/floor-grid` and every tracked file
reverted to `develop`'s version — `crates/floor/` gone, `commands.ts` without its line, the spec
back to the version with `Tier`. Nothing was lost (the commits were on origin, the uncommitted
work was copied out first), but the branch is a **variable shared between sessions** when
`git worktree list` shows one row, and a clean `git status` says only that your own work is
committed.

This branch moved to `.claude/worktrees/floor`, hidden from the shared tree through
`.git/info/exclude` rather than `.gitignore`, so the separation costs the repository nothing.

**The routing was split with the session doing the navigation refactor**, along the line that
lets both branches compile: they own `RouteName.Floor`, its path, title, origin and icon, the
sidebar entry, and the `routes.floor` message key — which their `routeTitle` cannot typecheck
without. This branch owns `[RouteName.Floor]: FloorScreen` in the `screens` record, which they
cannot add because the file does not exist on their branch, and the whole `floor` message block.

**Their refactor deleted a patch this plan asked for.** Task 8 Step 7 said to write
`needsProfile: routeOrigin[name] === TabOrigin.Progress && name !== RouteName.Floor` with a
comment explaining the exception. With Floor under a `Tool` origin whose definition is "does not
read the `.dat`", `needsProfile` derives from the origin with no exception and no comment,
because there is no longer a special case to explain.

## Verification

| | baseline (`develop`) | this branch |
|---|---|---|
| Rust passed | 894 | **929** (+35: 31 `floor`, 4 `ipc::floor_shape`) |
| Rust failed | 0 | **0** |
| skips declared | 129 | **129** |
| `sample:` lines | 79 | **79** |
| frontend tests | — | **499**, over 71 files |

typecheck, lint, `format:check` and `pnpm scan` green, 0 violations and no new exemption.
`floor` is 31 tests over four files; clippy clean at `-D warnings`.

**The machine has no game installed**, so `samples/packed` is absent and the tests behind it
skip on both sides of that table. What matters is that the two columns agree: this branch does
not make one more test skip than it found.

### The measurement that nearly went wrong twice

**First: the worktree.** `samples/` is git-ignored, so `git worktree add` does not bring it, and
the first suite run here reported 929 passed with **zero** `sample:` lines — not one test on
real data, and nothing red. Fixed with a junction, the way `samples/packed` already is. This is
now a line in `CLAUDE.md`'s **Don't** list.

**Second: how the skips were counted.** Run by hand, `cargo test --workspace -- --nocapture`
merges the harness's stdout with `test-support`'s stderr, so a `skip:` line can arrive with a
test result welded onto it and `grep -c '^skip:'` under-counts — it gave 121, then 118, then
168 across three runs whose real figures were 129, 129 and 175. `scripts/check` does not have
this problem and never did: it reads `ISAACDOME_TEST_DECLARATIONS`, a file `test-support`
appends to atomically, exactly so the summary is exact. The numbers in the table above come
from that file. **The instrument was mine, not the repo's** — worth writing down, because for
a while it looked like the reverse.
