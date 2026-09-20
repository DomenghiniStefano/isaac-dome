# The secret room placement rules, quoted with their sources

**Plan:** `docs/superpowers/plans/2026-09-15-floor-grid-and-rules.md`, Task 1.
**Spec:** `docs/superpowers/specs/2026-09-15-floor-secret-rooms-design.md`, §4.
**Branch:** `feature/floor-grid`. No code: this file is the source every later test's expected
value is read from, and `crates/floor/rules/placement.json` is a transcription of §2.

---

## 1. What was read

| page | url | outcome |
|---|---|---|
| Secret Room | https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room | read, 39,230 bytes of wikitext |
| Super Secret Room | https://bindingofisaacrebirth.wiki.gg/wiki/Super_Secret_Room | `#REDIRECT [[Secret Room#Super_Secret_Rooms]]` |
| Ultra Secret Room | https://bindingofisaacrebirth.wiki.gg/wiki/Ultra_Secret_Room | `#REDIRECT [[Secret Room#Ultra Secret Rooms]]` |
| Rooms | https://bindingofisaacrebirth.wiki.gg/wiki/Rooms | read for §4, 23,704 bytes of wikitext |

**Read on 2026-09-15.**

**The three pages of the plan are one page.** Both Super Secret Room and Ultra Secret Room are
redirects, and the 44 bytes above are their entire contents. So every rule in §2 cites
`/wiki/Secret_Room`, and that is the real page rather than a convenient one — which is what the
plan asked to be checked.

**How it was read.** `?action=raw`, the same endpoint `crates/wiki-snapshot/src/api.rs` already
uses, so the sentences below are the wikitext and not a rendering of it. A summarizer was tried
first and returned *"Regular Secret Rooms are usually located next to 3 or 4 rooms"* — the first
half of a sentence whose second half is `while Super Secret Rooms can only be next to one room`.
For a file whose only purpose is to be citable, a truncated quotation is already the failure, so
nothing here passed through one.

Quotations are the wikitext with its templates resolved to the text they render: `{{r|Boss
Room}}s` reads *Boss Rooms*, `{{i|X-Ray Vision}}` reads *X-Ray Vision*. Nothing else is changed,
and no sentence is joined to another.

### Licence

The text quoted below is from **The Binding of Isaac: Rebirth Wiki**
(https://bindingofisaacrebirth.wiki.gg), published under a **Creative Commons
Attribution-ShareAlike 4.0** license (https://creativecommons.org/licenses/by-sa/4.0/). The
rules file derived from it, `crates/floor/rules/placement.json`, is distributed under the same
license and carries the attribution and the date in its own header.

This is the wiki `dataset/ATTRIBUTION.md` already names, so the licence question was settled for
this repo before F1 existed and no second answer was invented.

---

## 2. The rules

Fourteen rows — nine on 2026-09-15, and five more on 2026-09-20 when §3's dismissal of the whole
Ultra Secret paragraph turned out to be wrong. §6 is that correction. Each one is a sentence the
wiki states and this grid can be held to.

**The order is the file's order and the file's order is load-bearing.** `floor::solve` walks the
rules once, and a narrowing rule only ever removes: one that runs before anything has been
proposed narrows an empty list. Within a target, every rule that proposes comes first.

| id | target | constraint, in one sentence | quotation | url |
|---|---|---|---|---|
| `secret-neighbours` | secret | The cell touches 3 or 4 painted rooms, and neither count is preferred over the other. | "Secret Rooms are equally as likely to be in a valid location with 3 neighbors, as it is with 4 neighbors." | https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room |
| `secret-neighbours-two` | secret | A cell touching 2 rooms is a candidate as well, ranked below, and stays one even when cells with 3 or more exist. | "2 neighbor locations are rare but possible, even when there are locations with 3+ neighbors available." | https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room |
| `secret-neighbours-one` | secret | A cell touching 1 room is a candidate only when no valid cell touches 3 or more. | "1 neighbor locations can only happen if there are no valid 3+ neighbor locations, and are very rare." | https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room |
| `secret-forbidden-neighbours` | secret | The cell touches no Boss Room, no Super Secret Room and no other Secret Room. | "Secret Rooms can exist next to all types of rooms except Boss Rooms, Super Secret Rooms, and other Secret Rooms" | https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room |
| `super-secret-dead-end` | superSecret | The cell touches exactly one painted room. | "Super Secret Rooms are only located next to one other room" | https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room |
| `super-secret-neighbour-not-special` | superSecret | That one room is not a Special Room. | "this room can't be a Special Room; in other words, it is placed on one of the floor's dead ends, like any other Special Room" | https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room |
| `super-secret-not-next-to-secret` | superSecret | The cell does not touch the Secret Room. | "cannot be connected to the Secret Room" | https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room |
| `super-secret-second-longest` | superSecret | Among the dead ends, it is the one needing the 2nd most rooms walked from the start room. | "Super Secret Rooms replace the dead-end room that would require the 2nd most rooms walked through from the start room to access" | https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room |
| `ultra-secret-connections` | ultraSecret | The cell reaches 3 or more rooms through the red rooms that could open beside it. | "Ultra Secret rooms are most likely generated in spots that connect to 3+ non-red rooms through its adjacent red rooms (different squares in L rooms count as 2)." | https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room |
| `ultra-secret-connections-two` | ultraSecret | It reaches exactly 2, ranked below. | "They can be connected to 2 or 1 non-red rooms through its adjacent red rooms, but a specific 3+ room location is 11.5x more likely than a specific 2 room location" | https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room |
| `ultra-secret-connections-one` | ultraSecret | It reaches exactly 1, ranked below that — and **not** switched off by a better cell. See §6. | "If there is no 3+ room location available then a specific 2 room location is 11.5x more likely than a specific 1 room location." | https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room |
| `ultra-secret-not-connected` | ultraSecret | The cell touches no painted room at all. | "Ultra Secret Rooms are special rooms that are not connected to any other room on the map directly." | https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room |
| `ultra-secret-red-room-invalid` | ultraSecret | None of the red rooms that could open beside it touches a Secret, Super Secret, Curse or Boss Room. | "Ultra Secret Rooms can't be connected to red rooms that connect to Secret Rooms, Super Secret Rooms, or Curse Rooms, and can't be in a location where any of its adjacent red rooms are invalid, such as next to a Boss Room" | https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room |
| `ultra-secret-shapes` | ultraSecret | Unmodelled — see §3. | "next to the sides of narrow rooms, or any room that can't have a red room opened on that specific side, however locations on the 13x13 border where a red room would normally open to an I AM ERROR room are allowed" | https://bindingofisaacrebirth.wiki.gg/wiki/Secret_Room |

### Two rules the plan's list did not have

`secret-neighbours-one` and `super-secret-not-next-to-secret` are not in the plan's draft of
`placement.json`. Both are quoted above, and the plan's own instruction for Task 4 is *"do not
write a rule the report does not have, and do not leave one out"*, so both are rules.

`secret-neighbours-one` is stated twice on the page, once in the lead and once in the notes,
and the two halves say different things:

> "Very rarely, the regular **Secret Room** may also be located next to only one room."
> — the lead, which says it happens

> "1 neighbor locations can only happen if there are no valid 3+ neighbor locations, and are
> very rare."
> — the notes, which says when

### The asymmetry that decides how the three count rules are encoded

Read the two quotations next to each other:

- 2 neighbours — "rare but possible, **even when** there are locations with 3+ neighbors available"
- 1 neighbour — "**can only happen if** there are no valid 3+ neighbor locations"

They are not the same shape. The first is unconditional and merely ranked lower; the second is a
**fallback**, switched off by the existence of a better cell. Encoding them the same way, as two
`neighbourCount` rules at ranks 1 and 2, would state that a 1-neighbour cell is a candidate on a
floor that has a 3-neighbour one — which this sentence denies. §5 says what that costs Task 4
and Task 5.

Note also the word **valid**: the condition is about 3+ locations that survive
`secret-forbidden-neighbours`, not about 3+ locations on the painted grid. A cell with four
neighbours one of which is the Boss Room does not suppress the fallback, because it was never a
valid location.

### Corroboration, not a rule

The lead states the first and fifth rows a second time, in one sentence, and the two statements
agree:

> "Regular Secret Rooms are usually located next to 3 or 4 rooms, while **Super Secret Room**s
> can only be next to one room; they are placed on one of the dead ends generated on the map,
> similar to other Special Rooms."

`super-secret-second-longest` has its *why* stated in the same note, which is the reason the rank
is 1 and not something else:

> "For reference, the **Boss Room** takes the dead-end room that would require the most rooms
> walked through, and the **Shop** takes the room that would require the 3rd most. This means
> that the Super Secret Room is equally as far or further from the start room than the Shop."

Neither is a row of its own: a rule repeated is still one rule.

---

## 3. What could not be sourced

Everything here was looked for and either not found, or found and not evaluable on a grid of
painted cells. **None of it becomes a rule**, and the ones that belong to a target reach the
screen as `Unmodelled` rather than as silence.

| claim | why it is not a rule |
|---|---|
| "Entrances to Secret Rooms will never have rocks or gaps in the way." | A fact about the room's **contents**. A painted minimap carries which cells are rooms, never what is inside one. |
| "Rooms adjacent to Secret Rooms will always have a clear, walkable path to the middle of the wall where Isaac can place a bomb." | Same: the interior of the neighbour, which nothing on this screen draws. |
| ~~The whole Ultra Secret rule.~~ **Withdrawn on 2026-09-20 — see §6.** | ~~It is stated **two hops out** — "connect to 3+ non-red rooms *through its adjacent red rooms*" — and a red room is the Red Key mechanic, created by an item during the run. The grid paints rooms that exist; it does not paint rooms an item could create.~~ Two hops is a distance, not an obstacle, and the cell in between is an **empty** one — which is the thing this grid knows best. |
| ~~"Ultra Secret Rooms can't be connected to red rooms that connect to Secret Rooms, Super Secret Rooms, or Curse Rooms"~~ **Withdrawn on 2026-09-20 — see §6.** | ~~Same two hops. Evaluating it would mean modelling where a red room may open, which is its own set of rules (the same note lists four more conditions, including narrow rooms and blocked sides).~~ Four of the conditions are about a room's **shape**, and only those four are out of reach. |
| "next to the sides of narrow rooms, or any room that can't have a red room opened on that specific side" | Whether a side can open depends on the shape of the room behind it, and `floor::Shape` has one variant: every room on this grid is one square. An L room's two squares — which the counting sentence says "count as 2" — and a narrow room's long side cannot be drawn here, so they cannot be judged. This is what is left of the row above it, and it keeps the Ultra Secret target's `Unmodelled` seat. |
| "a specific 3+ room location is 11.5x more likely than a specific 2 room location" | A ratio, and the wiki's own HTML comment says it is carried over from the regular Secret Room and is "based on testing". This screen ranks; it does not state odds. Nothing on it would be more true for carrying an 11.5. **Since 2026-09-20 the sentence is the quotation on `ultra-secret-connections-two`**, because it is where the page puts 2 below 3+ — the ordering is the rule, and the number is still nowhere in the file. |
| "are particularly likely to appear near the Boss Room" / "likely to spawn in between the shop and the Boss Room" | A **tendency**, and the page states it as a consequence of generation order rather than as a constraint. A tendency cannot reject a cell, and this screen's whole claim is that a lit cell is allowed by a cited rule. |
| "Loops and large rooms connected to more than one room on any given side can tamper with this logic, however." | The wiki's own caveat on `super-secret-second-longest`. It is not a rule; it is the page saying its rule has exceptions it does not enumerate. Recorded so nobody reads that rule as exact. |
| Whether the Start Room is a Special Room. | Not stated anywhere. See §4. |

### One thing the spec called an inference and the wiki states outright

`docs/superpowers/specs/2026-09-15-floor-secret-rooms-design.md` §2 says the grid being 13 wide
is "an inference, not a measurement". The Ultra Secret note states the number:

> "however locations on the **13x13 border** where a red room would normally open to an
> **I AM ERROR** room are allowed."

This does not make 84 the centre by measurement — that is still `6 * 13 + 6` on top of a width
the wiki now states, plus the unstated assumption that a run starts in the middle. **The width is
cited; the centre is still an inference**, and the spec's §2 was corrected in this pass to say
which half is which. What would close the other half is one real floor painted by hand against
the game's own minimap: a measurement, and one this machine cannot take.

---

## 4. What a Special Room is

`super-secret-neighbour-not-special` needs the list, and the list needs its own citation. The
`Secret Room` page never enumerates it; the `Rooms` page does, **structurally** — the
enumeration is that page's own section hierarchy, not a sentence in it:

```
== Normal Rooms ==
=== Closets ===
== Special Rooms ==
=== Boss Rooms ===            ==== Mini-Boss Room ====
=== Treasure Rooms ===        ==== Normal/Golden ====  ==== Silver ====  ==== Devil ====
=== Planetariums ===          === Shops ===            === Arcades ===
=== Challenge Rooms ===       ==== Boss Challenge Room ====
=== Curse Rooms ===           === Sacrifice Rooms ===
=== Secret Rooms ===          === Super Secret Rooms ===  === Ultra Secret Rooms ===
=== Libraries ===             === Devil Rooms ===      === Angel Rooms ===
=== Vaults ===                === Dice Rooms ===       === Bedrooms ===
=== Crawl Spaces ===          ==== Black Markets ====
=== Angel Room Shops ===      === Secret Shops ===     === Red Rooms ===
=== I AM ERROR Rooms ===      === Boss Rush ===        === Mega Satan Arena ===
=== ??? Entrance ===          === Mother Arena ===     === Greed Exit Rooms ===
=== Alternate Path Exit Rooms ===  === Grave Rooms ===
=== Mirror Room ===           === Mine Cart Room ===
```

A structural citation is the stronger kind here and the repo already prefers it: a section's
membership is the page's own organisation, not a sentence somebody wrote about it. The same page
states the Super Secret rule a second time in those words —

> "Super Secret Rooms must be bombed into, and are only located next to one other **non-special**
> Room."

— which is `super-secret-neighbour-not-special` sourced twice, from two pages.

**Of the kinds `floor::RoomKind` can paint**, the list above makes these Special: Boss,
Treasure, Shop, Curse, Challenge, Sacrifice, Arcade, Library, Miniboss, Secret, Super Secret,
Ultra Secret. That is exactly the plan's `SPECIAL_KINDS`, so nothing is removed from it.

`Normal` is not Special: it is the other top-level section, and Closets sit under it.

### The Start Room is stated nowhere, and that is the answer

**It is in neither section.** It is not under `== Normal Rooms ==` and not under
`== Special Rooms ==`; on the whole page it appears only inside sentences about *other* rooms —
"as far from the starting room as possible", "the starting room of each floor" — never as a kind
with a place in the taxonomy.

So its membership is **not stated**, and per the plan that makes it an `Unmodelled` entry rather
than a decision made in silence. It is not a pedantic gap: it decides a cell. A dead end hanging
off the Start Room is a Super Secret candidate if Start is not Special and is not one if it is,
and the honest answer on this grid is that the wiki does not say. The screen says so on that
cell instead of quietly picking.

What would close it: one floor where the only dead end touches the start room, painted against
the game's own minimap, with the Super Secret room found. That is a measurement, not a reading —
it belongs to the machine with the game.

---

## 5. What this pass changes in the plan

The plan asked for four sections. This is a fifth because three of the findings above change
Task 4 and Task 5, and a report that hid them to keep a section count would have been obeyed and
useless.

**1. `placement.json` has nine rules, not seven.** `secret-neighbours-one` and
`super-secret-not-next-to-secret` are added, both quoted in §2.

**2. `Constraint` needs a variant the plan does not have.** Encoding "1 neighbour only if no
valid 3+ location" as a plain `neighbourCount` at rank 2 would assert something the sentence
denies. Proposed:

```rust
NeighbourCountFallback { allowed: Vec<u8>, rank: u8, superseded_by_at_least: u8 }
```

`superseded_by_at_least: 3` is read off the words "no valid 3+ neighbor locations" — a number the
sentence states, not a band anybody named. This is the same refusal the plan already made when it
deleted `Tier`.

**3. The solver gains a phase, because "valid" means "after narrowing".** The plan's `solve` is
one pass down an ordered rule list where count rules propose and the others narrow. A fallback
cannot be resolved in that pass: its condition asks whether a 3+ cell *survived* the narrowing,
which is not known until the narrowing has run. So:

| phase | rules | what it does |
|---|---|---|
| propose | `neighbourCount`, `neighbourCountFallback` | add cells, each with its rank |
| narrow | `forbiddenNeighbour`, `neighbourNotSpecial`, `deadEndDistanceRank` | remove and re-rank |
| resolve | `neighbourCountFallback` again | drop its cells if any surviving candidate has `neighbours >= superseded_by_at_least` |

Proposing the fallback in phase 1 rather than phase 3 is what subjects its own cells to
`secret-forbidden-neighbours` — a fallback cell next to the Boss Room is not a candidate either,
and a phase-3 proposal would have skipped that filter with nothing going red.

**4. The order of rules within the file still matters**, exactly as Task 4 says, but now only
within a phase. The file lists the three secret count rules, then the narrowing one, then the
super secret rules; Task 5's tests keep both halves of the boss check, the grid that must be
rejected and the one that must not.

---

## 6. The Ultra Secret correction, 2026-09-20 (B68)

**What was wrong.** §3 dismissed the entire Ultra Secret paragraph as unmodellable, on this
reasoning: *"a red room is the Red Key mechanic, created by an item during the run. The grid
paints rooms that exist; it does not paint rooms an item could create."* Every word of that is
true about **red rooms**, and none of it is true about the rule, because the rule is not about
red rooms that exist. It is about the **sides where one could open**, and a side where a red room
could open is an **empty cell** — the one thing a painted minimap has more of than anything else.
Two hops is a distance. It was read as an obstacle.

The cost was not an approximation, it was silence: the target had exactly one rule, that rule was
`Unmodelled`, and `solve` never proposes a cell for one. The Ultra tab read **0 on every grid
there is**, including a fully painted floor — and it read that way for five days without anything
going red, because a target answering nothing is indistinguishable from a target answering
correctly that there is nowhere.

**And §3 never looked at the first sentence of the section.** *"Ultra Secret Rooms are special
rooms that are not connected to any other room on the map directly"* is a plain adjacency
constraint on painted cells, the simplest thing on this page to evaluate, and it was not in §2 or
§3 at all. Dismissing the hard sentence took the easy one with it: that is the shape of the
mistake, and it is why the rule here is to source a paragraph sentence by sentence rather than to
judge it whole.

**How it surfaced.** The owner opened the screen on 2026-09-20 and put it beside
https://tboisecretroomfinder.com, which answers the same question from the same painted grid and
lights cells where ours lit none. The comparison is what made the silence visible; nothing in the
suite could have.

### What was re-read

The same page, the same way — `?action=raw` — on **2026-09-20**. **39,230 bytes, byte-for-byte
the size §1 records for 2026-09-15**: the page has not changed, so this is a correction to how it
was read and not to what it said. `crates/floor/rules/placement.json` carries `"read":
"2026-09-20"` and `"version": 2` for that second reading.

### What a red room is, on this grid

For a candidate cell:

- its **red rooms** are its empty orthogonal neighbours — a painted neighbour is not one, because
  the room is already there, and a side off the edge of the grid is not one either;
- the rooms it **reaches** are the painted cells touching those red rooms, counted **distinct**
  and never counting the candidate itself. Two red rooms beside the same room are one way in.
  Nothing on this grid is a red room, so "non-red rooms" is every painted room it reaches;
- the ceiling is **twelve** — four sides, three rooms each — which is why the count is a `u8`.

A side off the edge is the sentence the page states outright: *"locations on the 13x13 border
where a red room would normally open to an I AM ERROR room are allowed."* Two of a corner cell's
four sides are off the grid, and that is not two sides that failed.

### The one that is deliberately not a fallback

`secret-neighbours-one` and `ultra-secret-connections-one` look like the same sentence and are
not:

> "1 neighbor locations **can only happen if** there are no valid 3+ neighbor locations"
> — Secret Room, and `NeighbourCountFallback` switches it off

> "1 room locations being **virtually impossible** if there is a 3+ location available"
> — Ultra Secret Room, and nothing switches it off

*Virtually*, and the page's own HTML comment says why: *"It seems sometimes the last dead end
created cannot connect to the Ultra Secret Room, which can create a scenario where a 1 room
location is chosen when a 3+ location is available."* A fallback here would drop the cells the
wiki says are sometimes the answer. So the 1-room band is a plain `redRoomConnections` at rank 2:
it ranks last, it is never removed. `a_one_room_spot_still_stands_while_a_three_room_spot_does`
is that decision, pinned.

### The phases, with the five new rules in them

| phase | rules | what it does |
|---|---|---|
| propose | `neighbourCount`, `neighbourCountFallback`, `redRoomConnections` | add cells, each with its rank |
| narrow | `forbiddenNeighbour`, `neighbourNotSpecial`, `noPaintedNeighbour`, `redRoomForbiddenNeighbour`, `deadEndDistanceRank` | remove and re-rank |
| resolve | `neighbourCountFallback` again | drop its cells if any surviving candidate has `neighbours >= superseded_by_at_least` |

`redRoomConnections` carries `at_least` with an `at_most` that is `null` for the open band: "3+"
is what the sentence says, and writing `[3, 4, … 12]` would put a ceiling in the file that the
wiki does not state. Without `at_most` on the two bands below it, a cell reaching five rooms
would be proposed once per band.

`Candidate.neighbours` stays what it has always been — the rooms **touching** the cell, which is
zero for every Ultra candidate once `ultra-secret-not-connected` has run. The count that earned
it is measured two cells out and is not the same thing, so it does not go in that field; the rank
carries the band, which is all the screen draws.

### Where the reference site goes further than the page

`tboisecretroomfinder.com` also drops any location with a painted orthogonal neighbour, which is
`ultra-secret-not-connected` above and is the same call — but the page has a bullet that reads
against it: *"If other rooms exist adjacent to it, the doors will not open from inside the Ultra
Secret Room."* That sentence presupposes the adjacency it forbids. It is about **doors**, and the
rule we encode is about doors too, so both readings survive it; the reason the strict one wins is
the other sentence, *"can't be in a location where **any** of its adjacent red rooms are
invalid"* — a painted neighbour is a side that can never host a red room. Recorded because it is
the line to suspect first if a real floor ever shows an Ultra Secret Room with a room against it.
