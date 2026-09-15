# Completed — the handoff to design

What closed out the structural base and was handed to the design system. **Searched, never
read.** It left `docs/STATUS.md` on 2026-09-16; the handoff happened.

## Handoff to design — what closes out the structural base

Decision from 2026-09-04. The webapp starts once the **structural base** is done, and the
structural base is defined as: **all static data normalized and all IPC contracts
fixed**, not "all real data". Two of the seven screens (Run, Live) live on data that
only exists at runtime, and three more (Next Steps, Unlock, Plan) depend on the M2
graph, which is the project's bottleneck. Waiting for that data before designing would mean
waiting for M4.

The "Vue only receives already-resolved JSON" constraint makes this possible: the only
dependency between webapp and backend is the shape of the view-models in `ipc`. Fixing that
shape with fake data behind it is, for design purposes, the same as having the real data;
when the real graph arrives the types don't change.

| Screen | Data today | What unblocks it |
|---|---|---|
| 0 Profile selection | ✅ complete | — |
| 4 Completion | ✅ complete (34 × 10 matrix, head icons) | — |
| 5 Collection | ✅ 909 items with name, sprite, quality, tags and pools | — |
| 2 Unlock | 🟢 designable on the contract: real nodes (name, icon, condition, done, what it unlocks, source DLC) for 641 slots, `graph` stubbed | M2 fills in `graph` without changing the types |
| 1 Next Steps, 3 Plan | 🟢 on the contract: the first 5 not-yet-done with `basis: stub`; real goals saved in `store`, `expansion: stub` | M2 and M3 fill in the stubs |
| 6 Run, 7 Live | 🔴 | M4 — runtime data by definition |

Path, in order:

- [x] **1. `catalog` plan B** — one session. After this, every static datum derivable from the
      game's files is normalized and the crate has its final shape.
- [x] **2. IPC contracts for the M2 and M3 screens** — spec dated 2026-09-05
      (`docs/superpowers/specs/2026-09-05-graph-contracts-design.md`), plan in nine tasks
      (`docs/superpowers/plans/archive/2026-09-05-graph-contracts.md`) executed the same day,
      report in `docs/superpowers/reports/2026-09-05-graph-contracts-report.md`. Types in `ipc`
      and TypeScript mirror in `ui/src/lib/ipc/`: a single node for the three screens, with the
      data that already exists (catalog, save, goals in `store`) **real** and whatever the
      graph doesn't know yet as a **declared** `{ kind: "stub" }` — not "fake data", which
      would be a value that looks computed. The node's three states (done · unlockable
      now · blocked by N), fan-out and missing steps have their place in
      `GraphInfo::Computed` and arrive with M2 without changing a single type. Pinned by tests on the
      JSON shape. The three decisions below are made on real data:
      - [x] Reverse index `AchievementId → what it unlocks`: `Catalog::unlocks`, built once
            in `build`, deterministic order; every `unlocked_by` appears exactly
            once (real test on the sum of edges).
      - [x] Section 1 achievement ↔ slot mapping: **`slot[id]`**, pinned on the real
            profile (169 of 171 seen items with the achievement completed). Slots 638–641 are
            beyond the catalog: 4 `unknown` nodes, not 5 — slot 0 is not a node.
      - [x] `unlock_condition` as displayable text: in the contract it's called `hint`, it's
            `string | null`, and `null` is the normal state for 354 of 637 achievements.
- [x] **3. `DESIGN-BRIEF.md` aligned** — revisited on 2026-09-05 after 1 and 2: §4's status
      light updated with the three graph screens on-contract, new §7 with the TypeScript
      types and the real/stub table, fourth question in §12 (how to draw a node whose graph
      is `stub` without it looking like missing data).
- [x] **4. Handoff to Claude Design** — **done: the package is handed over and the design
      is under way as of 2026-09-09.** Screens 0, 4 and 5 on real data, the other four
      on fixed contracts. From here M2 proceeds in parallel with the webapp without touching the
      types the frontend consumes.
      The material was ready on 2026-09-06 and **had since aged out**: the committed package was
      the output of `a9a32ee`, taken before M2 and M3 landed, where every payload still declared
      `"kind": "stub"` — 641 nodes, `basis: stub`, `expansion: stub` — and `contracts/types.ts`
      was the pre-M2 copy, with `GraphInfo = stub | computed`, no `RequirementView` and no queue.
      Handing that over buys a design of the placeholder. **The four steps below closed on
      2026-09-08**: the package on disk is the post-M2/M3 contract on the real profile, and what
      remained of this item was the act of handing it over, and that happened.

      **From here the IPC contract is live.** The rule in `CLAUDE.md` — the contract does
      not change for the convenience of a screen that doesn't exist yet — stops being a
      precaution and starts being a constraint with someone on the other end of it. Any
      change to a type in `ipc` or to `ui/src/lib/ipc/types.ts` from now on is a change to
      material a design is being built on, and has to be handed on rather than merely
      committed.

      *Known drift at handover:* the rename of sections 3 and 6 (`017131c`, 2026-09-09)
      changed two label strings in `contracts/payload/save_summary.json` —
      `per_char` → `level_counters`, `cards_pills` → `bosses` — and the matching rows of
      the brief's save-format table. No count moved and no TypeScript type changed. If the
      copy in design's hands predates that commit, those two strings are the whole
      difference.
      - [x] **a. A queue payload in `design-export`** (2026-09-08) — `queue.empty.json` and
            `queue.with_rows.json`, built through `plan::Queue::enqueue` and `GraphDeps`,
            i.e. the very functions behind the Tauri command: the package can't show an
            order the app wouldn't produce. Six tests on the pick rule.
            **The first rule was wrong and the real export caught it**: "the first node
            blocked by two or more steps" finds nothing on a profile at 385 achievements,
            and the package came out with a single row — without the one thing the queue
            exists to show. The rule is now "the deepest chain the profile actually has",
            ties broken by the lower id because a committed package has to regenerate
            identically. It degrades to a chain of one instead of to nothing.
            Also here: **`GraphDeps` moved from `crates/app` to `ipc`**, with three tests it
            never had while it sat in the crate that by convention isn't tested.
      - [x] **b. `DESIGN-BRIEF.md` re-aligned on the post-M2/M3 contract** (2026-09-08) —
            fourteen places, all the same point: `{ kind: 'stub' }` has left the wire.
            `GraphInfo` is `computed | partial`; `RequirementView` and the node's `missing[]`
            were missing from the document entirely; `StepsBasis` is `'fanOut'` and Next
            steps changed *meaning*, not just values — a `partial` node is not a step, and
            without a catalog the list is empty with a diagnostic saying so. §7.5's table
            redone, §0/§4/§10/§13 and question 4 rewritten around `partial`. **New §7.6 on
            the queue**, which the brief had nothing about at all: the types, the four states
            of a row, and the rule the whole thing rests on — a move repairs, so there is no
            rejected drop and no error toast to design.
      - [x] **c. `pnpm design:export` re-run over the whole package** (2026-09-08, evening) —
            run on the machine with the game reinstalled, the `samples/packed` junction live
            and the real profile: 8 archives, 19,473 entries, 2,074 images from the archives
            (1,639 in atlases, 435 single files), 48 sheets cut into 3,759 pieces, 10 wiki
            pages, 5,837 files, 36 MB. The package is committed on purpose (see
            `.gitignore`): Claude Design opens it from a fixed path, and it would vanish on a
            branch switch. The package README forbids the assets ending up in a public
            repository — `origin` is private, which is what makes committing them acceptable.
            **The diff says the extraction is deterministic**: 12 files changed and 2 added,
            and not one image byte moved between the `a9a32ee` output and this one. What
            changed is the whole point — `contracts/types.ts`, the unlock payloads, and the
            brief. The profile behind it: **386 of 641 done**, 620 nodes `computed` against
            21 `partial`. Two `"kind": "stub"` strings survive, both `PlanExpansion::Stub` in
            `plan.*.json`, and that one **is** current behaviour: the Plan's expansion is
            genuinely not computed, the queue is what replaced it.
      - [x] **d. The file count, here and in the package README** (2026-09-08) — the README
            was already generated (`readme(img.entries.len())` = 5,833) and correct; the stale
            number was in `DESIGN-BRIEF.md`'s header, which claimed **2035 images**. Now taken
            from the export's own report, and stated as two figures because they are two
            things: **5,833 catalogued images** (2,074 from the archives + 3,759 cut from the
            game's sheets) living in **5,837 files**, since the regular families are packed
            into atlases rather than written one file each.

Doesn't block the handoff but blocks Collection and Unlock: **`Archive::open` loads 1.3 GB**
to extract sprites (see open blockers), and the graph commands do this on every
call. Needs solving before a screen asks for a hundred icons at once, i.e. before
building the Collection screen, not before designing it.

---

