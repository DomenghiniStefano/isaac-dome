# M4 sub-project 2b — Live: report

**Plan:** `docs/superpowers/plans/2026-09-14-live-screen.md`. **Spec:** §3 and §4 of
`docs/superpowers/specs/2026-09-14-m4-run-screens-design.md`. **Branch:**
`feature/live-screen`. `pnpm check` green; 6 new Rust tests, 483 frontend tests unchanged.

## What is on screen

The run being watched — character, floors, seed, what it holds — and, under it, what
finishing it would open, grouped by the cell it needs: *beat Mom's Heart with Cain*, once,
with everything under it.

## The rule, and why it is narrow

A run opens an achievement only when **every** requirement still missing from it is a mark for
the character being played. A second requirement of any kind — including a second mark of the
same character — is something one run cannot give.

The test that pins this was mutated to check it can fail: widening the rule to "any missing
mark of this character" turns it red. Without that check it would have been a test written
after the code, which proves nothing.

## The character, which the spec called the hard part

It stayed hard, and it stayed unresolved **on purpose**. The log prints a name; the game gives
a Tainted character the base form's name; so a name reaches one or two characters. The view
holds both and carries a diagnostic saying it holds both, and the screen says it in words.

Inferring the right one from the starting items — Tainted Cain starts with Bag of Crafting —
was refused in the spec and refused again here. It returns with a measurement, not with a
plausible rule.

## What the doing added to the plan

**"No profile" and "no graph" became two sentences.** The plan had one absence. Writing the
command showed that the spec's own requirement — *the run draws either way* — makes them
different messages to the person reading: one says we cannot read your progress, the other
that the game is not installed. `LiveGraph` is three cases, not an `Option`, and each has its
test.

**The twelve column names are exported rather than retyped.** They already existed for the
matrix; a second table would have been a second chance to disagree with it about what a boss
is called.

## What is left

- **Nothing of 2b's plan.** Tasks 1 to 6 are done, the last one against the fixtures.
- **Not seen against a real run.** The screen answers from the archive's open run, and there
  is one only while the game is being played with the app running. The fixture carries the
  ambiguous case precisely because a real machine shows it only then.
