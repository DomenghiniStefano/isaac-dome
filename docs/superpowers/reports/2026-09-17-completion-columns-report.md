# The Completion screen counts twice — report (2026-09-17)

Sub-project **3.9**, branch `feature/completion-columns`, plan
`docs/superpowers/plans/2026-09-17-completion-columns.md`. It closes **B22 item 4** (the
matrix's two number columns) and **B23** (the KPI strip), which are one design pass: both
entries say so, and B23's "Done when" ends on "it's the same pass that draws B22's columns".

There is no spec document. B22's entry is the spec for the reading; the **layout** is not in
`Schermate.dc.html` — that file has one total column — so the shape is this sub-project's own
and the decisions are recorded in the plan's head and repeated here.

---

## What is on screen

**Two columns instead of one slot.** The grid's trailing track became two: `normale` and
`hard`, each over the cells the save lets us read. Isaac reads `12/12 · 11/12` — full at
normal, one mark short at hard — where the interim single slot said `12/12 · 11`.

**Each column carries its own denominator.** It is what B22's own "Done when" writes (*a row
with value 2 in one cell reads 1/12 · 1/12, not 0 and 1*), and it is what makes the second
column readable on its own: under a heading of its own, `hard` is not "the number after the
dot", it is how many bosses reached the second level.

**The footer is two rows, not two numbers per cell.** A boss column is 40px: a stacked pair
fits and says nothing about which line is which. `Personaggi con il marchio` and
`Di cui in hard` use the name column that was already there, and the second label states the
relation the pair exists for — every hard mark is also a normal one.

**The done colour moved from the row to the number.** It was `tally.complete` — hard
everywhere — painted on one slot. A number that fills its denominator is done now, so Isaac is
green at normal and plain at hard, which is what he is. `0/0` is **unreadable**, never full:
that is `TallyTone`, and it is the reason the tone is a value and not an equality at the call
site.

**The strip is three tiles**: marks at normal, marks at hard, complete characters. The fourth
tile B23 offers — "characters complete at normal" — is declined: B22 has just made `complete`
mean hard everywhere and the reference profile differs by exactly one character between the
two readings, so a strip holding both would argue with itself about one word.

## What the tests pin

`ui/src/lib/completion/completionView.test.ts`, four new cases, expected values taken from
B22's "Done when" and from the reference profile — never from the module's output:

- a row hard everywhere is `12/12` and `12/12`, both full;
- a row one hard mark short is full at normal and partial at hard;
- **a bare 2 counts in both columns** — `1/12 · 1/12`, the case the entry names;
- a row with nothing readable is `0/0` **unreadable**, not full. `0 === 0` is why this test
  exists: an equality alone calls an unreadable row finished.

`GroupView` now carries its `Tally` instead of three loose numbers, because the header draws
the same two columns a row does. The existing group assertion moved onto `tally` with its
expected values unchanged, which is what says nothing moved underneath it.

## What only a browser could say, and it was wrong

The group header was a flex row whose totals ended in `ml-auto`. That lands at the
**container's** right edge — and the container is `min-w-full`, so the rows paint the whole
card, so it is wider than the tracks. The two numbers sat some 130px past the columns they
belong to. No test can see it: every assertion about those numbers was green, and the screen
was wrong.

It is a `grid grid-cols-matrix` now, with everything that is not a total sharing one cell
(`col-start-1 -col-end-3`), so the header inherits the row's tracks by construction rather
than by a width that has to be kept in step.

This is the part worth keeping: the defect was found in the first screenshot, before the merge
and before the line was ever written into *"What only a window can say"*. Looking early does
not empty that list — it shortens it.

## A red test this branch did not write

`pnpm check` came back red on `crates/ipc/tests/preview_real.rs`,
`the_two_editions_declare_different_totals_and_the_later_one_declares_more`, in Rust this
branch does not touch. It is 3.8's, from the same day, and the failure is real:

> `rep+persistentgamedata1.dat: one series, two declared totals`

The test required **one declared total per series**, and that is the exact thing the module's
own header says a patch moves — the June 2025 `rep+` save declares 641, the 2026 ones 642
(`docs/save-format.md`). It passed only while `samples/` stopped short of the patch: a fixture
of a machine, asserted as a property. The assertion is now the true one — within a series the
declared total never *shrinks* — and the comparison the test exists for runs on the latest of
each series. Fixed in its own commit, because it is its own defect.

Worth stating plainly: **the sample set found it, not the code**. The series on this machine
grew past the 641 → 642 boundary, and a property written against the shorter series turned
red the first time it was run against the longer one. That is the shape `CLAUDE.md` warns
about under "Measuring on real data", from the other side.

## What is left, and it needs a window

Four lines, in `docs/STATUS.md` under *"What only a window can say"*. Two are checks — an
unreadable section drawn faint (no fixture reaches it), and the matrix in a narrow window —
and two are judgements that are the owner's: whether the footer's two rows read as one
statement, and whether a matrix scrolled far from its headings wants them to stick.
