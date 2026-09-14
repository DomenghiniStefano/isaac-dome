# M4 sub-project 2a — the Run screen: report

**Plan:** `docs/superpowers/plans/2026-09-14-run-screen.md`.
**Spec:** `docs/superpowers/specs/2026-09-14-m4-run-screens-design.md` (§2, §5, §6).
**Branch:** `feature/run-screen`. `pnpm check` green; 483 frontend tests over 69 files, 14 of
them new.

## What is on screen

The archive that fills itself has a screen: the totals, four filters, the diary, and the
chosen run's items under it. `Run` stops falling through to `PlaceholderScreen`.

**Nothing new crossed the IPC**, which is what the plan predicted and the reason 2a could be
split off at all: every column comes from `RunView` as sub-project 1 left it.

## What the doing corrected in the plan

**1. The detail is under the list, not inside the row.** Task 3 said "expanded row". A
virtualized list measures its rows at one height — that is what lets it draw 22 of 641 — so a
row that grows needs per-row measurement, which is a different mechanism from the one this
repo has (`useScaledRows`). The same click opens a card under the table, with room for three
groups of items. Choosing the accordion would have meant either abandoning virtualization on
a list that will grow with every run played, or building a second measuring mechanism for one
screen.

**2. The toolbar's sort group became optional.** `FilterToolbar` required `sort`, `sorts` and
`sortText`, because the two screens that had it both had something to choose between. This one
does not: its order is the archive's, not the reader's. The alternative was a single fake sort
option, a control that changes nothing — so the props are optional and the group is not drawn
when they are absent. Two screens pass them, one does not.

**3. The order was wrong the first time, and the way it was wrong is the finding.** The rule
is "newest session first, and a name that is not a clock keeps its place". Written as one
comparison function, those two rules **contradict each other**: with an unreadable name
between two readable ones, A comes before B by clock, B before C by position and C before A by
position — a cycle, and `Array.sort` given a contradictory comparator answers something
arbitrary rather than failing. The fix is to sort only the readable names, **into the slots
they already occupy**, which is a total order. The test that says so was added after the
mistake, and it fails on the first implementation.

**4. Two badges were wrong, and only a window said so.** Online and abandoned wore the
`Unknown` variant, which draws the question mark this design uses for *what the app could not
read*. A run you walked away from is a fact; so is a run played online. Nothing in the suite
draws a badge.

## What the fixtures gained

`Command.Runs` had no fixture, so the screen could not be looked at in a browser at all. The
new one is shaped to cover what the screen must survive rather than to look full: the watched
launch and two sessions, a win, a death, an abandonment and a run still open, a run whose
character was never named, an item with no name, and the `noCatalog` diagnostic beside them.

## What is not done, and is not a defect

- **Live** is 2b, with the join to the graph and the character ambiguity §3 of the spec
  measured: the archive has the name the log prints, and the game gives a Tainted character
  the base form's name.
- **The screen has not been seen against the real archive** — 28 sessions on this machine —
  only against the fixtures. The app was not launched: the owner was playing, and a dev build
  taking the foreground is not a neutral act. It is the one check left.
