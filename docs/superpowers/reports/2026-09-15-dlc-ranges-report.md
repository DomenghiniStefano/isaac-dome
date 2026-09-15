# B52 — what `n` means in a `{{dlc|…}}` code

**2026-09-15**, branch `feature/dlc-ranges`, cut from `develop`. Closes B52, opens B53.
Suite green: `scripts/check` all green, 7 skips, 1306 real files touched.

---

## 1. The question, and why it was an entry

Since 2026-09-08 this parser read a `dlc` code one letter at a time. `n`, `a`, `a+`, `r`, `r+`
were five codes, and a string was the set of editions they named. The day before this report, the
`{{bug|dlc=…}}` arm was wired up and the reading fell over: **1734 of the 4168 `{{dlc|…}}` uses**
opened a span valid in *no* edition, 1690 of which had shipped.

The entry refused to guess. The corpus said `n` modifies the code beside it — every code appears
both bare and `n`-prefixed, a bare `n` appears zero times, and Abyss carries `{{dlc|nr+}}` on an
item that exists only from Repentance — but **"new in" and "not in" have the same shape and
opposite consequences**, and if the answer were "not in" then the sentences shown unqualified were
being shown to the wrong readers. That is not a labelling task.

## 2. One query, and it answered more than the question

`https://bindingofisaacrebirth.wiki.gg/index.php?title=Template:Dlc&action=raw`, read with `curl`
and not a summarizer — the Floor session of the same day had already measured that a summarizer
returns a quotation cut in half, and here the quotation *is* the result.

`Template:Dlc` delegates to two helpers, and both are worth more than it is.

**`Template:Dlc/format` names every code in prose.** That is where `n` is written down:

```
|  7 <!-- 00111 nr  --> = [[File:Dlc nr indicator.png|…|alt=(except in Repentance and Repentance+)|Removed in Repentance]]
| 24 <!-- 11000 r   --> = [[File:Dlc r indicator.png|…|alt=(in Repentance and Repentance+)|Added in Repentance]]
```

**`n` means *not in*.** It was the expensive half of the two.

**`Template:Dlcset` is the whole dictionary**, a `#switch` from thirty codes to a five-bit mask,
stating its own bit order:

```
<!-- <Repentance †><Repentance><Afterbirth †><Afterbirth><Rebirth> -->
 |  1 | na       =  1 <!-- 00001 -->
 | 24 | r        = 24 <!-- 11000 -->
 | 31 | n | x |  = 31 <!-- 11111 -->
 | 0 <!-- invalid string! -->
```

So there was nothing left to derive. Rebirth is bit 0 and Repentance+ bit 4; `n`, `x` and the
empty string are the code for **no restriction**; and outside the thirty the wiki itself answers
`0`. A code is a **run of transitions**, not a set: `r` is "added in Repentance" and names
Repentance *and* Repentance+.

## 3. What that made wrong, measured

| | before | after |
|---|---|---|
| `{{dlc|…}}` spans with no edition named | 1734 (1690 shipped) | **0** |
| `meta.diagnostics.unknownDlcCodes` | 1832 | **0** |
| infobox `dlc` values read too narrow | 1078 of 1083 | **0** |
| spans narrowed by their page | — | **847 of 4831** |

All **seventeen** distinct codes the corpus uses are in the wiki's thirty. Nothing was left
unreadable, which is why the counter goes to zero rather than to a remainder.

**The infobox failure was the quiet one.** The same splitter read `dlc =`, so `r` lost Repentance+
on 531 pages, `a+` lost three editions on 292 and `a` four on 254. Tonsil's `a+nr` — the one page
in the snapshot with a two-transition code — gained Rebirth and Repentance outright, because the
`n` in the middle was read as a code of its own. Nobody would have noticed: an entry declaring
fewer editions than it has looks exactly like an entry.

## 4. Abyss was never a contradiction

The entry's own evidence for "`n` modifies the code beside it" was that `{{dlc|nr+}}` on Abyss
would, read as a set, be valid in Rebirth where the item is not. Read as a range it still names
four editions Abyss does not have — and the wiki does not read it alone. `{{context test}}`
**intersects the span with the page's own range** before it draws an icon. `nr+` ∩ `r` is
Repentance: *removed in Repentance+*, which is what the line says.

So the parser narrows too, as a post-pass over the entry the page produced. 847 spans move.

**The context is the page's first infobox, not each entry's own** — and that is the wiki's rule,
not a convenience: `{{page dlc}}` carries `{{assert once|page dlc}}`, so the first infobox sets
the context and a later one narrows a section instead.

**Ultra Greed is the page that said so, and the new counter is what asked.** Two bosses in one
file, `dlc = a` then `dlc = a+`, sharing one set of sections. Read as "each entry's own", the note
about the ending chest — marked `na+`, Rebirth and Afterbirth — shares no edition with Ultra
Greedier's `a+` and was thrown away. It was **1 span in 4831**, it appeared on the first run of
`spansOutsideTheirPage`, and with the rule corrected the counter reads **0**.

## 5. Two things found on the way

**`Diagnostics::merge` was dropping a counter.** `unknown_infoboxes` was added on 2026-09-14 by
B45, so that a page whose infobox has no kind would be loud; `merge` never mentioned it, and a
page's counters are merged into the snapshot's, so it shipped `{}` from the day it was written.
It now destructures its argument with **no `..`**, which makes the next omission a build error.
With it merged the snapshot reports `infobox monster: 3` — **B53**.

**`crates/ipc/examples/dlc_mask.rs` had this question open since 2026-09-13** and did not know it.
It doubted the Cargo `dlc` integer meant "exists in" because Blue Cap, the first Afterbirth
collectible, sets the Rebirth bit. Its mask is **31**: every bit, which is what `{{dlcset}}`
returns for a page declaring no range at all, as 341 of the 720 collectible pages do. Nothing
there ever claimed Blue Cap exists in Rebirth. That is five of its eight disagreements; the other
three are the id ranges rather than the mask.

The integer *is* `{{dlcset}}`'s output, which makes the Cargo table a second copy of the same
statement — so `the_cargo_dlc_integer_is_the_infobox_code_through_the_wikis_own_switch` holds the
thirty transcribed rows against **720 live pages**. Mutating one row turns it red on 173 of them.

## 6. How the table is kept honest

The thirty rows are **transcribed, not derived**. A tokenizer would accept `ra` and `rn` and
invent a mask for them, where the wiki answers `0`. But thirty rows copied by hand out of a
`#switch` are exactly where a typo hides, so a unit test re-derives each one from the transitions
its code spells and the two have to agree.

**It found a mistake the first time it ran, and in the right half.** `ana+` is 2 and the
derivation said 3: the state before the first transition is not "present", it is the *opposite* of
that transition — `na` ("removed in Afterbirth") means Rebirth alone, and `a` ("added in
Afterbirth") does not include it. The table was right; the explanation was wrong.

## 7. What was not done

`editionLabel` now renders `r` as "Repentance · Repentance+" where it used to say "Repentance".
That is faithful — `only` is the set of editions the span applies to, and it always was — but the
wiki draws **one** icon there, with the tooltip *Added in Repentance*. Whether the tag should name
the range or the transition is a question about what a reader wants to see, it wants a window, and
it is not this branch's to answer.
