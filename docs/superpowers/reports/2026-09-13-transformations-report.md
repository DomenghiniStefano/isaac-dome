# The transformations — report

**Branch** `feature/wiki-transformations`, cut from `develop` on 2026-09-13 after
`feature/wiki-infobox` merged. Spec
`docs/superpowers/specs/2026-09-13-transformations-design.md`, plan
`docs/superpowers/plans/archive/2026-09-13-transformations.md`. `scripts/check` green on the result:
1274 real files touched, 7 skips, all named.

**What it does.** The wiki's sixteen transformations are entries of the dataset, with the
count each page states and the set of items that counts toward it. Those travel to `graph`
in `requirements.json`, where `Requirement::Threshold` answers them against the profile, and
to the screen as a row that says `Guppy — 2 di 3` and lists what is still locked. The four
nodes that carried a hand-written "the model can't say N of these" — 65 and 161 behind
Guppy, 178 and 352 behind Beelzebub — are answered.

**What it does not do.** Task 11 of the plan, the other half of B34, was written and then
abandoned. §4 below.

---

## 1. What the wiki actually says, measured

Every design decision in the spec that survived was measured; three did not survive.

| | |
|---|---|
| `Template:Infobox transformation` | exists, transcluded by exactly the 16 pages the Cargo table has rows for |
| the Cargo table's `requirement` | **the same string on all 16 rows** — a template default, not data |
| the Cargo table's `items` | rendered HTML, **empty on 10 of 16** |
| the count, per page | `Pick up 3 …` in the body, and Necromancer puts four words between the phrase and the digit |
| the item set | two template shapes: `{{collectible rows}}` (25 uses) more common than `{{collectible table}}` (14), with an optional `dlc =` |
| Guppy | 8 contributors — the infobox's 7 **plus Kid's Drawing**, which only the body lists |
| Adult | no count, no set: its page is about three *pills* |
| Stompy | count 3, set 2 — its third contributor is a pill listed as a bullet |
| Super Bum | `id = n/a` in the infobox; the Cargo table maps that onto 1000 |
| the two sources disagreeing | **13 of 16** |

The three decisions that did not survive contact with the data:

1. **The count was to be read from `requirement` through a map of English numerals.** That
   field holds one constant across the whole table, so the map would have returned 3 for
   every transformation and its `None` branch would never have run — a rule that cannot
   fail. The count now comes from the body sentence, and `requirement` is in
   `IGNORED_PARAMS` with the reason written down.
2. **`items` from Cargo was to be one of the two sources.** It is rendered markup and mostly
   absent. The union is the infobox's own wikitext against the body's tables.
3. **The plan scheduled one dataset rebuild, at Task 6.** `crates/wiki/tests/derived.rs`
   binds `wiki.json` to `build(raw, …)`, so the snapshot refresh at Task 1 turned it red and
   it would have stayed red for four tasks. Now a global constraint: rebuild in the same
   sitting as the change that breaks it.

## 2. What the numbers are now

- **16 transformations** in the dataset and in `requirements.json`, 15 with a count, 1
  (Adult) honestly without.
- **`threshold` 4** in the real graph's requirement breakdown, where it was 0.
- **uninterpreted references 17 → 13**, and the 13 are §4.
- Unknown templates **200 → 196**: `collectible table` and its three relatives are read now,
  and `Collectible table/header` and `header transformations` are declared layout.
- Two schemas moved: `wiki::dataset::SCHEMA_VERSION` 1 → 2 and `graph::rules::SCHEMA_VERSION`
  1 → 2. The snapshot era pinned in `crates/graph/tests/embedded.rs` moved with them.

## 3. What the instruments say, and that they can speak

Every new rule has a test, and the two written alongside their implementation were checked
by mutation rather than trusted:

- killing the `Pick up` phrase fails the count test and nothing else;
- ignoring the body tables fails both union tests and nothing else;
- **swapping the two branches of the threshold's order** — asking about unresolved items
  before asking whether the profile already has enough — fails
  `an_unresolved_contributor_never_unmeets_a_met_threshold` and only that one. That ordering
  is the whole design, and it is now pinned by a test that has been shown to notice.

On the real snapshot: Guppy's set contains a trinket (the guard for the union), the
disagreement counter is above zero (the guard for the cross-check), and both outcomes of
"count read / count unread" occur (the guard against a pattern that matches everything).

## 4. Task 11 abandoned: the thirteen are not noise

B34 says the thirteen `pickup:` references are "words that became targets" and asks for a
filter. The filter was written — a concept becomes a pickup only if the wiki's `pickup`
Cargo table knows it — and then checked against the sentences:

| node | the wiki's requirement |
|---|---|
| 69, 84 | "Collect all non-DLC items, and unlock all non-DLC secrets and **endings**" |
| 324 | "Collect every entry in the **Bestiary**" |
| 276 | "Defeat Mega Satan as every character (**tainted character**…)" |

They are real requirements the model cannot express, which is exactly what their
`Verdict::Unknown` records. Dropping the reference removes the node's only uninterpreted
requirement, so the node stops being `Partial` and reads **available now** — and the count of
uninterpreted references falls, which looks like progress. The rule was also far wider than
B34 asked: 45 of 49 pickup targets dropped, `Hard mode` (38 uses), `Completion Mark` (19) and
`Donation Machine` (10) among them, each of them a judgement already recorded.

The code was reverted before it was committed. B34 is corrected in place: its transformation
half is closed, its pickup half rests on a misreading, and what is actually wrong is the
**name** — `Target::Pickup` is what a linked concept with no id becomes, and calling it
`Pickup` is what made "pickups that are not pickups" look like the problem.

## 5. What is still unread

- **Adult's condition.** Three Puberty pills. The model has no pickups, so its count and its
  set are both empty, and no achievement references it.
- **Stompy's third contributor**, also a pill. Its set is smaller than its count, so `graph`
  refuses to build a threshold for it rather than report a requirement nothing can meet.
- **13 pages state their item set twice and state it differently.** The set is the union and
  the disagreement is counted; nobody has read the thirteen to see which side is right.
- **Whether the game ships artwork for a transformation.** `TargetSprite` is `NoArt`, which
  is a placeholder for an unasked question, not an answer.
- **B38**, three pickup quotes shipping an undecoded HTML entity, found by the wiki-versus-game
  agreement test and registered rather than fixed here.
