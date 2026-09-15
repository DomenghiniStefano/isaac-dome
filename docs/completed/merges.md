# Completed — which branch merged when

The running record that lived at the top of `docs/STATUS.md` until 2026-09-16: what landed on
`develop`, in what order, with what it carried. **Searched, never read.**

It was the head of the document whose job is to say where the project is, and it had become a
news feed — by the end it still announced `feature/screens-goals-detail` as *"waiting to merge"*
days after it had merged. The rules it was wrapped around stayed behind; only the news moved here.

**Wiki dataset merged** into `develop` on 2026-09-06 (`feature/wiki-dataset`, 29 commits,
suite green on the merge result, review of the whole branch closed). The local branch was
deleted; on origin its last published version remains.
**Design system merged** into `develop` on 2026-09-11: cycle 1 (`feature/design-system-foundations`),
then cycle 2 and screens 3.1–3.2 in one merge of `feature/design-system-screens`, which
already held `feature/design-system-components`; suite green on the merge result. Screens 3.3a
and 3.3b followed on the same branch, each merged into `develop` with the suite green (the last,
`ef962c8`, on 2026-09-11).


**Last update:** 2026-09-14. **All eight cleanup items are done** — N1, N2, N3, N4, N5,
N6, N7 and N8, each on its own branch cut from `develop` — the last two on the evening of
2026-09-14. The test-only public API has one notation
(one `pub mod for_tests` per crate, seven of them); **why a command failed is a variant, not
a sentence** (four enums, the numbers travelling as numbers, the wording in `it.ts` / `en.ts`);
**the Tauri crate is wiring again** (eleven files, none over 220 lines, `cargo test -p app`
reporting zero); one diagnostics list instead of four; one view store instead of three;
**the IPC contract is generated from the Rust types**; and **one faceted list instead of two**,
six components become three and five modules two.
**M4's sub-project 1 is closed**, 1a and 1b both: the run model, then the watcher, the archive
and the fourth migration (`feature/log-watch`). **N8 ran last**, as its own order says — *after
M4* — and closed on the evening of 2026-09-14, which empties the cleanup list.
Order: N1 → N2 → N6 → N4 → N5 → N7 → M4 sub-project 1 → **N3 → N8**. N4 and N5 were pulled
forward because they are frontend, touch no file that branch has, and N7 was blocked then;
**N3 ran before N8 rather than after**, on a machine without the game, because it is the one of
the two that needs neither the game nor a real save to be finished or believed.
**Sub-project 3.5d merged into `develop`** (`4406c49`), suite green on the merge result: a
blocked badge opens a menu whose entries are the wiki pages of what is in the way.
**The wiki's transformations merged into `develop`** on 2026-09-13, suite green on the merge
result: sixteen pages, a seventh `PageKind`, and `Requirement::Threshold` — the shape the
model could never say, *N of these items*. The four nodes behind Guppy and Beelzebub are
answered. **The other half of B34 was written and thrown away**: the thirteen `pickup:`
references are real requirements, not noise, and dropping them would have made their nodes
read *available now*. Report in
`docs/superpowers/reports/2026-09-13-transformations-report.md`.
**M4's first sub-project has its design** (`cac6914`): the run model, the `run` and
`log-watch` crates, and the backfill that makes the archive born full from the logs already
on disk.
**The landing page and the achievement detail are done** on `feature/screens-goals-detail`,
waiting to merge: "Prossimi passi" is **"Obiettivi consigliati"**, grouped by the reason a row
is suggested, and an achievement's wiki page carries a block saying where the profile stands.
**B32 and B35 closed.** Two contract changes travel with it — `NextSteps` in sections with a
`Closeness` basis, and `AchievementRef`'s `hint` renamed `condition` because it now answers
from the wiki where the game file is silent (283 of 637 before, all 637 after). **N7 has that
much more to absorb**, and `UnlockTarget` gained a field as well.
**B40, B43 and half of B42 closed** on `feature/small-three` (2026-09-14, evening), three small
entries run in parallel on disjoint files. The cross-check B42 asked for opened **B45**: four
characters — Jacob & Esau, The Forgotten, Tainted Forgotten, Tainted Lazarus — **have no page in
the wiki snapshot**, 47 requirements point at them, and the fetch cannot see it because it
enumerated the singular `Infobox character` while those four pages carry the plural one, two forms in a block — closed the same evening.

