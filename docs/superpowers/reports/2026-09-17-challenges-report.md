# 3.11 — the Challenges screen: report

**2026-09-17**, branch `feature/challenges`. B3 closed. Spec
`docs/superpowers/specs/2026-09-17-challenges-screen-design.md`, plan
`docs/superpowers/plans/2026-09-17-challenges-screen.md`.

## What is there now

`Sfide`, the fifth entry under Progressi: the game's 45 challenges, which you have finished, what
each one takes and what it unlocks. A row carries the number, the name — which opens the
challenge's wiki page — the reward beneath it, the character the challenge forces, how far you
have to get, whether it is blindfolded, the state, and the button that puts the reward in the
Plan's queue.

One `ipc::challenges_view` joins section 7, `challenges.xml` and the wiki. One read command. No
write path was added: `queue_add` already takes an achievement and a challenge's reward is one,
which is why the Plan join cost nothing on the wire.

The screen is the **fourth list on 3.10's filter bar**, and it is the first one built on it from
nothing — which is the claim that spec made and nobody had tested by adding a list. It took a
slot table and a labels file.

## What the work corrected

**Two of B3's three halves were already closed.** The items are the Collection; "search inside a
list" is the filter bar. The entry is from 2026-09-05, before either — and it still asked for
TanStack Table, which 3.3a had declined for a measured reason. Nobody had re-read it. What was
actually missing was the challenges, and the shape of the gap was exact: `core-save` read section
7, `catalog` parsed the challenges with their rewards, and there was nothing in front of them.

**Two things the plan assumed and the code corrected.** `Dataset::embedded()` already returns a
reference, so the `.as_ref()` in the plan's snippets was one indirection too many; and the new
types had to be **declared** in `ipc::contract::render()` — the generator is a list, not a scan,
so a type nobody adds to it is simply absent from `types.ts`, silently.

**And one the screen corrected in the view-model.** The wiki gives the forced character as a
`Target` carrying an **id**, and a screen cannot name an id: the first draft of the row had a
placeholder word that would have labelled every character the same. The view-model now carries
`characterName`, read from that character's own wiki page title — the same source as the rest of
the conditions — and a test on real data asserts that every challenge which forces a character
can name it.

## Measured, and one of the three is not a confirmation

| | |
|---|---|
| challenge `n` is cell `n`, cell 0 unused | 39 of 39 judged rows agree — **21 both true, 18 both false** — while the off-by-one reading breaks 13 of the same 39 |
| the wiki has a page for every challenge | 45 of 45 in the committed snapshot |
| `unlocked_by` is "all of" or "any of" | **undecided, and left that way** |

The third is the one worth reading twice. All 13 finished challenges that have gates have *every*
gate done — and that satisfies **both** readings, because "all done" implies "any done". The
instrument cannot separate them, so the spec reads `unlocked_by` as "all of" and the screen
**names** the achievements a blocked row is waiting for, so a reader who has done enough of them
can see the claim and disbelieve it. The measurement that would settle it — a challenge with some
gates done, checked against the game's own menu — is registered in `docs/STATUS.md` rather than
guessed at.

The instrument is `cargo run -p ipc --example probe_challenges`, and the mapping is now guarded by
`crates/ipc/tests/challenges_real.rs`, which was mutated by shifting the cells one across: it
answers *"challenge 11 reads done and [99] is not earned — the cell mapping shifted"*.

## The gate

`pnpm check` **all green**: 1061 Rust and 632 ui tests, 18 declared skips, doc references with
nothing new and nothing stale, the floor raised in the same commit as the tests that raised it.

`docs/architecture.md` **was** redrawn, unlike 3.10's: a route and a command both moved, so its
header now pins **31 commands** and **15 routes**, the routes diagram has `/progress/challenges`
and the table names the commands behind it.

Two tests failed the moment the route was added, and both were the repo's own guards doing their
job: *"nothing is reachable only by typing its path"* and *"gates exactly the screens that read
the save"*. The second is a table that has to be edited by hand on purpose — adding a screen
forces whoever adds it to say which precondition it sits behind. Challenges reads section 7 of
the `.dat`, so it is behind the profile gate.

## What a browser already answered

- every state on screen at once, which a real profile cannot show: done, to do, blocked naming
  two gates, and — with `?challenges=unread` — four rows of the unknown hatch, the state row
  reading `non leggibile 4` and a diagnostic saying why. Nothing reads as "you have done none"
- the row the wiki knows nothing about says `non lo sappiamo` in both condition columns instead
  of "any character, not blindfolded"
- the queue button is absent where it would do nothing: on a finished challenge and on the one
  that rewards nothing

## What only a window — or the game — can say

Five lines in `docs/STATUS.md`'s gathered list, and one of them is also the measurement above.
The conditions have never been read beside the game's own challenge menu, which is the only place
the wiki's words and the game's can be compared.
