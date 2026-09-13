# Obiettivi consigliati and the achievement detail — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the landing page a page a player can read — grouped by reason, with cards that say what you get and where to go — and give every achievement a detail, which is its wiki page plus what the profile knows.

**Architecture:** No new route and no new screen for the detail: `/wiki?page=achievement:<id>` grows a "Il tuo profilo" block, fed by a lookup over the `UnlockView` the graph store already holds. The landing page's contract grows from one list to a list of sections, each with the basis that produced it (`fanOut`, `closeness`). `UnlockTarget` gains the wiki page of what it names, which is what makes both the block and the cards linkable (B35).

**Tech Stack:** Rust (`crates/ipc`, `crates/app`), Vue 3 + TypeScript (`ui/`), Vitest, `cargo test`.

**Spec:** `docs/superpowers/specs/2026-09-13-goals-and-achievement-detail-design.md`

## Global Constraints

- **Branch:** `feature/screens-goals-detail`, cut from `develop` on 2026-09-13, in the worktree `C:\Projects\isaac-dome-goals`. One sub-project, one branch (`docs/STATUS.md`, 2026-09-11). The wait on `feature/wiki-infobox` was dropped: that branch touches no frontend file (spec §8).
- **The working copy is shared with other sessions that switch branches under you.** That is why this runs in its own worktree. Never `git add -A`; stage by explicit path, exactly the paths each task's commit step lists.
- **Every struct crossing the IPC** carries `#[serde(rename_all = "camelCase")]`; enums with struct variants also need `rename_all_fields = "camelCase"`.
- **Fieldless enums on the IPC are bare camelCase strings**, and their TypeScript is a union of values built with `const X = { … } as const`. `StepsBasis` is one of these — zero exceptions in the repo.
- **Exhaustiveness is mandatory**: no `_ =>` arm on a closed enum, in Rust or in a TypeScript `switch` (use `assertNever`).
- **`ui/src/lib/ipc/types.ts` is hand-mirrored** and changes in the same commit as the Rust type. A contract change is handed on, not merely committed.
- **Frontend rules** (`docs/frontend-conventions.md`, enforced by `pnpm scan`): no `<style>` in SFCs, no hardcoded visual constants (no `w-[48px]`, no `:size="16"`), no `invoke()` in components, no raw `<button>`/`<input>`, no string unions, no visible strings in templates — every one goes through `t()` and exists in **both** `ui/src/i18n/messages/it.ts` and `en.ts`.
- **Test-first.** The expected value comes from the spec, never from the code's current output. A failing test is first a hypothesis of a bug in the code.
- **Before declaring anything done:** `pnpm check` (fmt, clippy `-D warnings`, `cargo test --workspace`, typecheck, `ui:test`, lint, format:check, scan).
- **Commits:** Conventional Commits, `type(scope): subject`, English, atomic. **Never** a `Co-Authored-By` trailer.

---

## File Structure

**Rust — `crates/ipc`**
- `src/goals.rs` — `Resolved` gains `page`; `TargetKey::view` puts it on all four `UnlockTarget` variants.
- `src/graph.rs` — `resolve_target`/`target_of`/`plan_view` take the dataset; `NextSteps` becomes sections; `StepsBasis` gains `Closeness`; the closeness predicate and ordering.
- `tests/graph.rs` — sectioning, disjointness, ordering, empty sections, on synthetic catalogs.
- `tests/graph_real.rs` — the vacuity guard and the page rule on the real catalog and dataset.

**Rust — `crates/app`**
- `src/commands/plan.rs` — three `plan_view` call sites pass the embedded dataset.

**Frontend — new files**
- `ui/src/lib/graph/achievementNode.ts` (+ `.test.ts`) — the page → node lookup and its four degrading cases.
- `ui/src/lib/graph/unlockEntries.ts` (+ `.test.ts`) — what a node unlocks, as linkable rows.
- `ui/src/lib/graph/goalCard.ts` (+ `.test.ts`) — the card's model as a pure function.
- `ui/src/lib/plan/planNow.ts` (+ `.test.ts`) — the queue rows the landing page shows.
- `ui/src/components/graph/ProfileBlock.vue` — the detail block.
- `ui/src/screens/goals/GoalCard.vue` — the rewritten card.
- `ui/src/kit/sections/app/ProfileBlockSection.vue` — the block on the Kit page.

**Frontend — renamed / modified**
- `ui/src/screens/NextStepsScreen.vue` → `ui/src/screens/GoalsScreen.vue`; `ui/src/screens/nextSteps/` → `ui/src/screens/goals/`.
- `ui/src/router/routeTable.ts`, `ui/src/router/routes.ts` — `RouteName.NextSteps` → `RouteName.Goals`, path, title key, `defaultLocation`; `TabLocation.query` gains `state`.
- `ui/src/screens/UnlockScreen.vue` — reads `?state=` the way it already reads `?q=`.
- `ui/src/screens/wiki/WikiPage.vue` — hosts the block.
- `ui/src/lib/ipc/types.ts`, `ui/src/lib/ipc/fixtures/graph.ts`, `ui/src/i18n/messages/{it,en}.ts`.

---

## Task 1: `UnlockTarget` carries its wiki page (B35)

**Files:**
- Modify: `crates/ipc/src/goals.rs` (`Resolved`, `TargetKey::view`, `UnlockTarget`)
- Modify: `crates/ipc/src/graph.rs` (`resolve_target`, `target_of`, `plan_view`, their call sites)
- Modify: `crates/app/src/commands/plan.rs:29,63,88`
- Modify: `ui/src/lib/ipc/types.ts`, `ui/src/lib/ipc/fixtures/graph.ts`
- Test: `crates/ipc/tests/graph.rs`, `crates/ipc/tests/graph_real.rs`

**Interfaces:**
- Consumes: `page_of(dataset: Option<&Dataset>, target: Option<Target>) -> Option<Target>` (private in `graph.rs`); `wiki_target::{item, character, challenge, boss}`.
- Produces: `pub fn resolve_target(c: &Catalog, key: &TargetKey, dataset: Option<&Dataset>, icon: &mut impl FnMut(&IconRef) -> Option<String>) -> Option<UnlockTarget>`; `pub fn target_of(c: &Catalog, u: &Unlock, dataset: Option<&Dataset>, icon: &mut impl FnMut(&IconRef) -> Option<String>) -> UnlockTarget`; `pub fn plan_view(catalog: Option<&Catalog>, dataset: Option<&Dataset>, goals: Vec<Goal>, unreadable: Vec<GoalId>, store_unavailable: Option<StoreReason>, icon: impl FnMut(&IconRef) -> Option<String>) -> PlanView`; `UnlockTarget::{Item,Character,Boss,Challenge}` each with `page: Option<Target>`.

- [ ] **Step 1: Write the failing test** — append to `crates/ipc/tests/graph.rs`

```rust
/// A target's page follows the same rule as a requirement's: `Some` only when the dataset
/// really has the entry. With no dataset there is no page to carry, and the row still names
/// what it names — a missing page never removes a target.
#[test]
fn without_a_dataset_no_target_carries_a_page() {
    let c = catalog_with_achievements();
    let flags = [false, true, true, true];
    let v = unlock_view(Some(&c), None, Some(&flags), None, None, None, |_| None);
    let pages: Vec<Option<&wiki::Target>> = v
        .nodes
        .iter()
        .flat_map(|n| n.unlocks.iter())
        .map(|t| match t {
            ipc::UnlockTarget::Item { page, .. }
            | ipc::UnlockTarget::Character { page, .. }
            | ipc::UnlockTarget::Boss { page, .. }
            | ipc::UnlockTarget::Challenge { page, .. } => page.as_ref(),
        })
        .collect();
    assert!(
        !pages.is_empty(),
        "the fixture catalog has to produce targets, or this test asserts nothing"
    );
    assert!(pages.iter().all(Option::is_none));
}
```

- [ ] **Step 2: Run it to make sure it fails**

Run: `cargo test -p ipc --test graph without_a_dataset_no_target_carries_a_page`
Expected: FAIL to compile — `UnlockTarget::Item` has no field `page`.

- [ ] **Step 3: Add the field and the resolution**

In `crates/ipc/src/goals.rs`, add `page` to `Resolved` and to all four `UnlockTarget` variants:

```rust
pub(crate) struct Resolved {
    pub name: String,
    pub icon_url: Option<String>,
    pub rewards: Vec<u32>,
    pub tainted: bool,
    /// The dataset's page for this thing, when it has one. `None` is "nowhere to read
    /// about it": the name shows and does not link. The same rule as `RequirementView`.
    pub page: Option<Target>,
}
```

`TargetKey::view` destructures `page` and puts it on each variant (`UnlockTarget::Item { item_kind, id, name, icon_url, page }`, and so on for `Character`, `Boss`, `Challenge`).

In `crates/ipc/src/graph.rs`, `resolve_target` takes the dataset and fills it:

```rust
pub fn resolve_target(
    c: &Catalog,
    key: &TargetKey,
    dataset: Option<&Dataset>,
    icon: &mut impl FnMut(&IconRef) -> Option<String>,
) -> Option<UnlockTarget> {
    // …unchanged…
    match *key {
        TargetKey::Item { item_kind: k, id } => {
            let i = c.item(item_kind(k), ItemId(id))?;
            resolved.name = c.text(&i.name, english).to_string();
            resolved.icon_url = icon(&IconRef::Item { kind: k, id });
            resolved.page = page_of(dataset, Some(wiki_target::item(i)));
        }
        TargetKey::Character { id } => {
            let ch = c.character(CharacterId(id))?;
            resolved.name = c.text(&ch.name, english).to_string();
            resolved.tainted = ch.tainted;
            resolved.page = page_of(dataset, Some(wiki_target::character(ch)));
        }
        TargetKey::Boss { id } => {
            let b = c.boss(BossId(id))?;
            resolved.name = b.name.clone();
            // A portrait that declares no entity key names no page (`wiki_target::boss`).
            resolved.page = page_of(dataset, wiki_target::boss(b));
        }
        TargetKey::Challenge { id } => {
            let ch = c.challenge(ChallengeId(id))?;
            resolved.rewards = ch.rewards.iter().map(|a| a.0).collect();
            resolved.name = ch.name.clone();
            resolved.page = page_of(dataset, Some(wiki_target::challenge(ch)));
        }
    }
    Some(key.view(resolved))
}
```

`target_of` gains the same parameter and forwards it; the fallback arm stays
`key.view(crate::goals::Resolved::default())`, which is a target with no page. Inside
`unlock_view`, the call becomes `target_of(c, u, dataset, &mut icon)` — `dataset` is
already a parameter there. `plan_view` gains `dataset: Option<&Dataset>` as its second
parameter and passes it to `resolve_target`.

- [ ] **Step 4: Fix the three app call sites**

`crates/app/src/commands/plan.rs`, all three calls:

```rust
Ok(ipc::plan_view(
    c,
    wiki::Dataset::embedded().ok().as_ref(),
    goals,
    unreadable,
    unavailable,
    icon_url,
))
```

Check `wiki` is in `crates/app/Cargo.toml` (the `collection` command already calls
`wiki::Dataset::embedded()`, so it is).

- [ ] **Step 5: Run the test to verify it passes**

Run: `cargo test -p ipc --test graph without_a_dataset_no_target_carries_a_page`
Expected: PASS

- [ ] **Step 6: Write the real-data property**

Append to `crates/ipc/tests/graph_real.rs`, next to
`a_blocked_node_links_to_the_pages_the_dataset_has`, whose fixture helpers it reuses:

```rust
/// The other half of 3.5d's rule: what a node *unlocks* links to the page the dataset really
/// has, and to nothing else. Same shape as the requirement's, on purpose — one rule, one
/// place (`wiki_target`), two readers.
#[test]
fn what_a_node_unlocks_links_to_the_pages_the_dataset_has() {
    let Some(fixture) = real_graph_fixture() else {
        return;
    };
    let (c, ds, v) = fixture;
    let page = |t: &ipc::UnlockTarget| match t {
        ipc::UnlockTarget::Item { page, .. }
        | ipc::UnlockTarget::Character { page, .. }
        | ipc::UnlockTarget::Boss { page, .. }
        | ipc::UnlockTarget::Challenge { page, .. } => page.clone(),
    };
    let targets: Vec<&ipc::UnlockTarget> = v.nodes.iter().flat_map(|n| n.unlocks.iter()).collect();
    let linked = targets
        .iter()
        .filter_map(|t| page(t))
        .inspect(|t| assert!(ds.entry(t).is_some(), "a page that goes out has to exist"))
        .count();
    assert!(
        linked > 0,
        "on the real catalog the dataset answers for most targets: zero links means the \
         mapping stopped working, not that the wiki is empty"
    );
    let _ = c;
}
```

Read the top of `graph_real.rs` first: it already builds the real catalog, dataset and view
behind a `test_support` skip. Reuse that helper verbatim rather than writing a second one —
if it is not yet a function, extract it in this step and leave the existing test calling it.

- [ ] **Step 7: Run both tests**

Run: `cargo test -p ipc --test graph_real -- --nocapture`
Expected: PASS, or a `skip: …` line on stderr when `samples/packed` is absent.

- [ ] **Step 8: Mirror the contract in TypeScript**

`ui/src/lib/ipc/types.ts`, all four variants of `UnlockTarget`:

```ts
export type UnlockTarget =
  | {
      kind: 'item'
      itemKind: ItemKindView
      id: number
      name: string
      iconUrl: string | null
      // Where to read about it. `null` = the dataset has no page: the name shows and does
      // not link. Never a link that leads nowhere.
      page: Target | null
    }
  | { kind: 'character'; id: number; name: string; tainted: boolean; page: Target | null }
  | { kind: 'boss'; id: number; name: string; page: Target | null }
  | { kind: 'challenge'; id: number; name: string; rewards: number[]; page: Target | null }
```

Then add `page` to every `UnlockTarget` literal in `ui/src/lib/ipc/fixtures/graph.ts` — give
at least one fixture target a real page (`{ kind: 'item', id: 105 }`) and one `null`, so the
screens can be looked at in both states.

- [ ] **Step 9: Typecheck and run the frontend suite**

Run: `pnpm typecheck && pnpm ui:test`
Expected: PASS

- [ ] **Step 10: Commit**

```bash
git add crates/ipc/src/goals.rs crates/ipc/src/graph.rs crates/ipc/tests/graph.rs crates/ipc/tests/graph_real.rs crates/app/src/commands/plan.rs ui/src/lib/ipc/types.ts ui/src/lib/ipc/fixtures/graph.ts
git commit -m "feat(ipc): what a node unlocks carries the page the dataset has"
```

---

## Task 2: `NextSteps` becomes sections, with the closeness basis

**Files:**
- Modify: `crates/ipc/src/graph.rs` (`NextSteps`, `StepsBasis`, `next_steps`)
- Modify: `ui/src/lib/ipc/types.ts`, `ui/src/lib/ipc/fixtures/graph.ts`, `ui/src/screens/NextStepsScreen.vue`
- Test: `crates/ipc/tests/graph.rs`, `crates/ipc/tests/graph_real.rs`

**Interfaces:**
- Consumes: `UnlockView`, `GraphInfo`, `RequirementView`, `STEPS` (= 5).
- Produces: `pub struct StepsSection { pub basis: StepsBasis, pub steps: Vec<UnlockNode> }`; `pub struct NextSteps { pub sections: Vec<StepsSection> }`; `pub enum StepsBasis { FanOut, Closeness }`. The TypeScript mirror is `StepsSection`, `NextSteps { sections: StepsSection[] }`, `StepsBasis = { FanOut: 'fanOut', Closeness: 'closeness' }`.

- [ ] **Step 1: Write the failing tests** — append to `crates/ipc/tests/graph.rs`

```rust
/// A node whose every standing requirement is a counter is not blocked: the content is
/// reachable and only has to be played. It belongs to the section that can order it — a
/// counter carries a distance, and the fan-out does not.
#[test]
fn a_node_held_only_by_counters_goes_to_the_closeness_section() {
    let counter = |current: u32, at_least: u32| ipc::RequirementView::Counter {
        label: "Mom's Heart".into(),
        current,
        at_least,
    };
    let node = |slot: u32, fan_out: u32, missing: Vec<ipc::RequirementView>| ipc::UnlockNode {
        achievement: ipc::AchievementRef::Unknown { slot },
        done: false,
        unlocks: vec![],
        origin: None,
        missing,
        graph: GraphInfo::Computed {
            available_now: true,
            blocked_by: 0,
            fan_out,
            steps_missing: 0,
        },
    };
    let v = ipc::for_tests::unlock_view_of(vec![
        node(1, 9, vec![]),                    // nothing in the way: fan-out
        node(2, 1, vec![counter(9, 11)]),      // two to go
        node(3, 1, vec![counter(3, 11)]),      // eight to go
    ]);

    let s = next_steps(&v);
    let basis: Vec<StepsBasis> = s.sections.iter().map(|x| x.basis).collect();
    assert_eq!(basis, vec![StepsBasis::FanOut, StepsBasis::Closeness]);

    let slots = |basis: StepsBasis| -> Vec<u32> {
        s.sections
            .iter()
            .find(|x| x.basis == basis)
            .map(|x| {
                x.steps
                    .iter()
                    .map(|n| match n.achievement {
                        ipc::AchievementRef::Known { id, .. } => id,
                        ipc::AchievementRef::Unknown { slot } => slot,
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    assert_eq!(slots(StepsBasis::FanOut), vec![1]);
    assert_eq!(
        slots(StepsBasis::Closeness),
        vec![2, 3],
        "nearest the threshold first"
    );
}

/// A node is a suggestion once, or it reads as two different suggestions.
#[test]
fn the_two_sections_never_name_the_same_node() {
    let v = ipc::for_tests::unlock_view_of(vec![ipc::UnlockNode {
        achievement: ipc::AchievementRef::Unknown { slot: 4 },
        done: false,
        unlocks: vec![],
        origin: None,
        missing: vec![ipc::RequirementView::Counter {
            label: "Hush".into(),
            current: 0,
            at_least: 1,
        }],
        graph: GraphInfo::Computed {
            available_now: true,
            blocked_by: 0,
            fan_out: 40,
            steps_missing: 0,
        },
    }]);
    let s = next_steps(&v);
    assert_eq!(s.sections.len(), 1, "one node cannot fill two sections");
    assert_eq!(s.sections[0].basis, StepsBasis::Closeness);
}

/// A heading over nothing is not a state the screen should have to handle: "absent" is
/// decided once, here.
#[test]
fn a_section_with_no_steps_is_not_emitted() {
    let mut flags = vec![false; 10];
    flags[2] = true;
    let v = unlock_view(None, None, Some(&flags), None, None, None, |_| None);
    assert!(next_steps(&v).sections.is_empty());
}
```

`ipc::for_tests::unlock_view_of` may not exist yet. If it does not, add it to
`crates/ipc/src/for_tests.rs` in Step 3 — it is test-only public API and the repo already has
one `pub mod for_tests` per crate:

```rust
/// A view built straight from its nodes: the steps' ordering is a property of the nodes, and
/// building a catalog to express "fan-out 9" would test the catalog instead.
pub fn unlock_view_of(nodes: Vec<crate::UnlockNode>) -> crate::UnlockView {
    crate::UnlockView {
        nodes,
        totals: crate::UnlockTotals {
            slots: 0,
            done: 0,
            known: 0,
            unknown: 0,
        },
        diagnostics: vec![],
    }
}
```

- [ ] **Step 2: Run them to make sure they fail**

Run: `cargo test -p ipc --test graph closeness`
Expected: FAIL to compile — `StepsBasis::Closeness` and `sections` do not exist.

- [ ] **Step 3: Implement** — `crates/ipc/src/graph.rs`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NextSteps {
    pub sections: Vec<StepsSection>,
}

/// One reason, and the steps it produced. The screen draws the basis as a heading: a row is
/// worth showing only together with why it is being suggested.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepsSection {
    pub basis: StepsBasis,
    pub steps: Vec<UnlockNode>,
}

/// What a section is ordered by. A fieldless enum: on the wire `"fanOut"` / `"closeness"`.
/// `Closeness` is the basis the type was left open for — a counter is the one requirement
/// that carries a distance, so it is the only one that can order a list by nearness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StepsBasis {
    FanOut,
    Closeness,
}
```

and the function:

```rust
fn slot_of(n: &UnlockNode) -> u32 {
    match n.achievement {
        AchievementRef::Known { id, .. } => id,
        AchievementRef::Unknown { slot } => slot,
    }
}

fn fan_out_of(n: &UnlockNode) -> u32 {
    match n.graph {
        GraphInfo::Computed { fan_out, .. } | GraphInfo::Partial { fan_out, .. } => fan_out,
    }
}

/// How far the profile is from a requirement, when the requirement is a tally. Every other
/// kind answers `None`: a mark is binary and a character is a wall, neither has a distance.
fn counter_remaining(r: &RequirementView) -> Option<u32> {
    match *r {
        RequirementView::Counter {
            current, at_least, ..
        } => Some(at_least.saturating_sub(current)),
        RequirementView::Character { .. }
        | RequirementView::Boss { .. }
        | RequirementView::Challenge { .. }
        | RequirementView::Item { .. }
        | RequirementView::Gate { .. }
        | RequirementView::Mark { .. }
        | RequirementView::Unknown { .. } => None,
    }
}

/// `Some(distance)` when the node still has requirements and **every one of them** is a
/// counter: the sum is how far the profile is from the whole set. One non-counter and the
/// answer is `None` — `Option`'s `Sum` short-circuits, which is exactly the rule.
fn closeness(n: &UnlockNode) -> Option<u32> {
    if n.missing.is_empty() {
        return None;
    }
    n.missing.iter().map(counter_remaining).sum()
}

/// The steps worth playing tonight, in two sections. **Unlockable now** is the gate for both:
/// if it's blocked tonight can't touch it, and if the graph can only say `Partial` suggesting
/// it would be a guess.
///
/// `Closeness` claims first, because "two runs from it" says more about a node than its
/// fan-out does — and it claims only the ones that actually fit under `STEPS`, so a candidate
/// beyond the cap falls back to the other section instead of vanishing from both.
///
/// Ties break by slot ascending, so two calls on the same profile give the same list: an
/// order that shuffles reads as the app changing its mind.
pub fn next_steps(view: &UnlockView) -> NextSteps {
    let available: Vec<&UnlockNode> = view
        .nodes
        .iter()
        .filter(|n| {
            matches!(
                n.graph,
                GraphInfo::Computed {
                    available_now: true,
                    ..
                }
            )
        })
        .collect();

    let mut close: Vec<(u32, &UnlockNode)> = available
        .iter()
        .filter_map(|n| closeness(n).map(|d| (d, *n)))
        .collect();
    close.sort_by_key(|(d, n)| (*d, std::cmp::Reverse(fan_out_of(n)), slot_of(n)));
    close.truncate(STEPS);

    let claimed: std::collections::HashSet<u32> =
        close.iter().map(|(_, n)| slot_of(n)).collect();
    let mut open: Vec<&UnlockNode> = available
        .into_iter()
        .filter(|n| !claimed.contains(&slot_of(n)))
        .collect();
    open.sort_by_key(|n| (std::cmp::Reverse(fan_out_of(n)), slot_of(n)));
    open.truncate(STEPS);

    let sections = [
        (
            StepsBasis::FanOut,
            open.into_iter().cloned().collect::<Vec<_>>(),
        ),
        (
            StepsBasis::Closeness,
            close.into_iter().map(|(_, n)| n.clone()).collect::<Vec<_>>(),
        ),
    ]
    .into_iter()
    .filter(|(_, steps)| !steps.is_empty())
    .map(|(basis, steps)| StepsSection { basis, steps })
    .collect();

    NextSteps { sections }
}
```

Export `StepsSection` from `crates/ipc/src/lib.rs` next to `StepsBasis`.

- [ ] **Step 4: Fix the existing tests that read `.steps` and `.basis`**

`without_a_graph_there_are_no_next_steps_to_suggest` and
`next_steps_take_what_is_unlockable_now_most_fan_out_first` in `crates/ipc/tests/graph.rs`
both read the old shape. Rewrite them against sections — **do not delete them**: the first
is the "not-done is not unlockable" rule and the second is the fan-out order, and both still
hold. The first becomes `assert!(next_steps(&v).sections.is_empty())`; the second reads the
`FanOut` section's steps. Do the same for any assertion in `crates/ipc/tests/graph_real.rs`.

- [ ] **Step 5: Run the suite for the crate**

Run: `cargo test -p ipc -- --nocapture`
Expected: PASS (watch the `skip:` lines — they are not failures, but they say which slice ran)

- [ ] **Step 6: Write the vacuity guard on real data**

Append to `crates/ipc/tests/graph_real.rs`:

```rust
/// A property about "ci sei quasi" holds trivially on a profile that has no such node, and a
/// test that cannot fail reports coverage that is not there. This is the guard: on the real
/// profile the closeness section has to exist at all, or the model stopped producing it.
#[test]
fn the_real_profile_has_a_closeness_section_to_test() {
    let Some(fixture) = real_graph_fixture() else {
        return;
    };
    let (_, _, v) = fixture;
    let s = ipc::next_steps(&v);
    let close = s
        .sections
        .iter()
        .find(|x| x.basis == ipc::StepsBasis::Closeness);
    let Some(close) = close else {
        eprintln!(
            "skip: the reference profile has no node held only by counters — the closeness \
             section is untested on real data"
        );
        return;
    };
    assert!(!close.steps.is_empty(), "an emitted section is never empty");
    for n in &close.steps {
        assert!(
            !n.missing.is_empty()
                && n.missing
                    .iter()
                    .all(|r| matches!(r, ipc::RequirementView::Counter { .. })),
            "a closeness step is held by counters and nothing else"
        );
    }
}
```

- [ ] **Step 7: Run it and read the output**

Run: `cargo test -p ipc --test graph_real the_real_profile_has_a_closeness_section_to_test -- --nocapture`
Expected: PASS. **Read whether it printed the `skip:` line.** If it did, say so in the task's
report: the section is then untested on real data, which is a known gap, not a green light.

- [ ] **Step 8: Mirror the contract and keep the screen compiling**

`ui/src/lib/ipc/types.ts`:

```ts
// What a section is ordered by. No fields: a bare string, like `OriginView`.
export const StepsBasis = { FanOut: 'fanOut', Closeness: 'closeness' } as const
export type StepsBasis = (typeof StepsBasis)[keyof typeof StepsBasis]

// One reason and the steps it produced. A section is never emitted empty (the rule lives in
// Rust), so the screen never draws a heading over nothing.
export interface StepsSection {
  basis: StepsBasis
  steps: UnlockNode[]
}

export interface NextSteps {
  sections: StepsSection[]
}
```

In `ui/src/lib/ipc/fixtures/graph.ts`, build the fixture as two sections — a `fanOut` one and
a `closeness` one whose nodes carry a `counter` requirement — so the new page has something to
draw before Task 7 exists.

In `ui/src/screens/NextStepsScreen.vue`, the smallest change that compiles and keeps today's
behaviour: iterate `graph.view.steps.sections` and, inside, its `steps`. The headings, the
copy and the rename all belong to Task 7 — this step only keeps the suite green.

- [ ] **Step 9: Typecheck and run the frontend suite**

Run: `pnpm typecheck && pnpm ui:test`
Expected: PASS

- [ ] **Step 10: Commit**

```bash
git add crates/ipc/src/graph.rs crates/ipc/src/lib.rs crates/ipc/src/for_tests.rs crates/ipc/tests/graph.rs crates/ipc/tests/graph_real.rs ui/src/lib/ipc/types.ts ui/src/lib/ipc/fixtures/graph.ts ui/src/screens/NextStepsScreen.vue
git commit -m "feat(ipc): the steps come in sections, and closeness is one of them"
```

---

## Task 3: The page → node lookup, and its four degrading cases

**Files:**
- Create: `ui/src/lib/graph/achievementNode.ts`
- Test: `ui/src/lib/graph/achievementNode.test.ts`

**Interfaces:**
- Consumes: `UnlockView`, `UnlockNode`, `Target` from `@/lib/ipc/types`.
- Produces: `achievementNode(unlock: UnlockView | null, target: Target | null): UnlockNode | null`.

- [ ] **Step 1: Write the failing test**

```ts
import { describe, expect, it } from 'vitest'
import type { UnlockNode, UnlockView } from '@/lib/ipc/types'
import { achievementNode } from './achievementNode'

const node = (id: number): UnlockNode => ({
  achievement: { kind: 'known', id, text: `t${id}`, hint: null, iconUrl: null },
  done: false,
  unlocks: [],
  origin: null,
  missing: [],
  graph: { kind: 'computed', availableNow: true, blockedBy: 0, fanOut: 0, stepsMissing: 0 },
})

const view = (nodes: UnlockNode[], diagnostics: UnlockView['diagnostics'] = []): UnlockView => ({
  nodes,
  totals: { slots: 0, done: 0, known: 0, unknown: 0 },
  diagnostics,
})

describe('achievementNode', () => {
  it('finds the node the page names', () => {
    const found = achievementNode(view([node(1), node(19)]), { kind: 'achievement', id: 19 })
    expect(found?.achievement).toMatchObject({ id: 19 })
  })

  it('answers nothing for a page that is not an achievement', () => {
    expect(achievementNode(view([node(1)]), { kind: 'item', id: 1 })).toBeNull()
  })

  it('answers nothing for an achievement the catalog does not know', () => {
    expect(achievementNode(view([node(1)]), { kind: 'achievement', id: 999 })).toBeNull()
  })

  it('answers nothing without a view', () => {
    expect(achievementNode(null, { kind: 'achievement', id: 1 })).toBeNull()
  })

  // "Not done" for everything would be a lie, and a block that says it is worse than no
  // block: section 1 of the save was not read.
  it('answers nothing when the achievement section was not read', () => {
    const v = view([node(1)], [{ kind: 'noAchievementSection' }])
    expect(achievementNode(v, { kind: 'achievement', id: 1 })).toBeNull()
  })
})
```

- [ ] **Step 2: Run it to verify it fails**

Run: `pnpm --filter ui test src/lib/graph/achievementNode.test.ts`
Expected: FAIL — cannot resolve `./achievementNode`.

- [ ] **Step 3: Write the implementation**

```ts
import type { Target, UnlockNode, UnlockView } from '@/lib/ipc/types'

// The node a wiki page names, when the page is an achievement and the profile can answer for
// it. Not a join — `UnlockView` is already resolved; this picks the row.
//
// Five answers are `null`, and each one is a state the block must not draw: no view, a page
// that is not an achievement, an id the catalog does not know, and — the one that is not
// obvious — a save whose achievement section was not read. There every node reads "not done",
// which is not a fact about the profile (`UnlockDiagnostic.noAchievementSection`).
export const achievementNode = (
  unlock: UnlockView | null,
  target: Target | null,
): UnlockNode | null => {
  if (unlock === null || target === null || target.kind !== 'achievement') return null
  if (unlock.diagnostics.some((d) => d.kind === 'noAchievementSection')) return null
  return (
    unlock.nodes.find(
      (n) => n.achievement.kind === 'known' && n.achievement.id === target.id,
    ) ?? null
  )
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `pnpm --filter ui test src/lib/graph/achievementNode.test.ts`
Expected: PASS (5 tests)

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib/graph/achievementNode.ts ui/src/lib/graph/achievementNode.test.ts
git commit -m "feat(ui): the node a wiki page names, and the four times it has none"
```

---

## Task 4: What a node unlocks, as linkable rows

**Files:**
- Create: `ui/src/lib/graph/unlockEntries.ts`
- Test: `ui/src/lib/graph/unlockEntries.test.ts`

**Interfaces:**
- Consumes: `UnlockTarget.page` (Task 1); `pageLocation` from `@/lib/wiki/category`; `targetName` from `@/lib/graph/characterName`.
- Produces: `interface UnlockEntry { key: string; name: string; iconUrl: string | null; location: TabLocation | null }` and `unlockEntries(node: UnlockNode, t: Translate): UnlockEntry[]`.

- [ ] **Step 1: Write the failing test**

```ts
import { describe, expect, it } from 'vitest'
import type { UnlockNode, UnlockTarget } from '@/lib/ipc/types'
import { RouteName, WikiCategory } from '@/router/routeTable'
import { unlockEntries } from './unlockEntries'

const t = ((key: string) => key) as never

const nodeWith = (unlocks: UnlockTarget[]): UnlockNode => ({
  achievement: { kind: 'known', id: 1, text: 't', hint: null, iconUrl: null },
  done: false,
  unlocks,
  origin: null,
  missing: [],
  graph: { kind: 'computed', availableNow: true, blockedBy: 0, fanOut: 0, stepsMissing: 0 },
})

describe('unlockEntries', () => {
  it('sends an entry to the page its target carries', () => {
    const [entry] = unlockEntries(
      nodeWith([
        {
          kind: 'item',
          itemKind: 'passive',
          id: 105,
          name: 'The D6',
          iconUrl: 'isaac://item/passive/105',
          page: { kind: 'item', id: 105 },
        },
      ]),
      t,
    )
    expect(entry).toMatchObject({
      name: 'The D6',
      iconUrl: 'isaac://item/passive/105',
      location: {
        name: RouteName.Wiki,
        query: { category: WikiCategory.Items, page: 'item:105' },
      },
    })
  })

  // Never a link that leads nowhere: the row shows, it just does not go anywhere.
  it('leaves a target with no page without a location', () => {
    const [entry] = unlockEntries(
      nodeWith([{ kind: 'boss', id: 3, name: 'Nameless', page: null }]),
      t,
    )
    expect(entry.name).toBe('Nameless')
    expect(entry.location).toBeNull()
  })

  it('keys a row by its kind and id, so two targets never collide', () => {
    const entries = unlockEntries(
      nodeWith([
        { kind: 'boss', id: 3, name: 'A', page: null },
        { kind: 'character', id: 3, name: 'B', tainted: false, page: null },
      ]),
      t,
    )
    expect(new Set(entries.map((e) => e.key)).size).toBe(2)
  })
})
```

- [ ] **Step 2: Run it to verify it fails**

Run: `pnpm --filter ui test src/lib/graph/unlockEntries.test.ts`
Expected: FAIL — cannot resolve `./unlockEntries`.

- [ ] **Step 3: Write the implementation**

```ts
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import type { UnlockNode, UnlockTarget } from '@/lib/ipc/types'
import { pageLocation } from '@/lib/wiki/category'
import type { TabLocation } from '@/router/routeTable'
import { targetName } from './characterName'

type Translate = (
  key: MessageKey<MessageSchema>,
  params?: Record<string, unknown>,
) => string

// What an achievement gives you, with somewhere to read about it. The sibling of
// `missingGroups`: that one says what is in the way, this one what comes out — and both
// answer `null` rather than link to a page the dataset does not have (B35).
export interface UnlockEntry {
  key: string
  name: string
  iconUrl: string | null
  location: TabLocation | null
}

const iconOf = (target: UnlockTarget): string | null =>
  target.kind === 'item' ? target.iconUrl : null

export const unlockEntries = (node: UnlockNode, t: Translate): UnlockEntry[] =>
  node.unlocks.map((target) => ({
    key: `${target.kind}-${target.id}`,
    name: targetName(t, target),
    iconUrl: iconOf(target),
    location: target.page ? pageLocation(target.page) : null,
  }))
```

Read `@/lib/graph/characterName` before writing this: `targetName(t, target)` is the existing
function that gives a Tainted character its full name (B28). If its signature differs, follow
it rather than the snippet.

- [ ] **Step 4: Run the test to verify it passes**

Run: `pnpm --filter ui test src/lib/graph/unlockEntries.test.ts`
Expected: PASS (3 tests)

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib/graph/unlockEntries.ts ui/src/lib/graph/unlockEntries.test.ts
git commit -m "feat(ui): what a node unlocks, as rows that link to their page"
```

---

## Task 5: The "Il tuo profilo" block on the achievement page

**Files:**
- Create: `ui/src/components/graph/ProfileBlock.vue`
- Create: `ui/src/kit/sections/app/ProfileBlockSection.vue`
- Modify: `ui/src/screens/wiki/WikiPage.vue`
- Modify: `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts`
- Modify: the Kit page's section list (find it with `grep -rn "sections/app" ui/src/kit`)

**Interfaces:**
- Consumes: `achievementNode` (Task 3), `unlockEntries` (Task 4), `missingGroups` from `@/lib/graph/nodeState`, `NodeStateBadge`, the graph store and the queue store.
- Produces: `<ProfileBlock :node="UnlockNode" :queued="boolean" :can-add="boolean" :busy="boolean" @add />`.

- [ ] **Step 1: Add the copy, both languages**

`ui/src/i18n/messages/it.ts`, a new `profileBlock` group next to `graph`:

```ts
  profileBlock: {
    title: 'Il tuo profilo',
    missing: 'Cosa ti manca',
    unlocks: 'Cosa ottieni',
    opens: 'Sbloccarlo apre altre {count} cose.',
    opensNothing: 'Non apre nient’altro: è una fine di ramo.',
    stepsMissing: 'Prima servono ancora {count} sblocchi.',
    done: 'Già fatto.',
  },
```

and the same keys in `en.ts` ("Your profile", "What you are missing", "What you get",
"Unlocking it opens {count} more things.", "It opens nothing else: this is a leaf.",
"{count} unlocks are needed first.", "Already done.").

- [ ] **Step 2: Write the component**

`ui/src/components/graph/ProfileBlock.vue`. Rules that apply: no `<style>`, no hardcoded
visual constants, no raw `<button>`, every visible string through `t()`.

```vue
<script setup lang="ts">
import { ListPlusIcon } from '@lucide/vue'
import { computed } from 'vue'
import NodeStateBadge from '@/components/graph/NodeStateBadge.vue'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useGestureModifiers } from '@/composables/useGestureModifiers'
import { useMessages } from '@/i18n'
import { missingGroups } from '@/lib/graph/nodeState'
import { unlockEntries } from '@/lib/graph/unlockEntries'
import type { UnlockNode } from '@/lib/ipc/types'
import type { TabLocation } from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'

const props = defineProps<{
  node: UnlockNode
  queued: boolean
  canAdd: boolean
  busy: boolean
}>()
const emit = defineEmits<{ add: [] }>()
const { t } = useMessages()
const tabs = useTabsStore()
const { ctrl } = useGestureModifiers()

// The page has room, so the why is rows and not a menu: the menu exists because a badge is
// small (spec §3.1). Same model, same links, same gesture.
const groups = computed(() => missingGroups(props.node, t))
const gets = computed(() => unlockEntries(props.node, t))
const fanOut = computed(() => props.node.graph.fanOut)
const stepsMissing = computed(() =>
  props.node.graph.kind === 'computed' && !props.node.graph.availableNow
    ? props.node.graph.stepsMissing
    : null,
)

const open = (location: TabLocation | null) => {
  if (!location) return
  if (ctrl.value) tabs.open(location)
  else tabs.navigate(location)
}
</script>

<template>
  <Card>
    <CardHeader><CardTitle>{{ t('profileBlock.title') }}</CardTitle></CardHeader>
    <CardContent class="flex flex-col gap-4">
      <NodeStateBadge :node="node" />

      <section v-if="groups.length > 0" class="flex flex-col gap-2">
        <h3 class="text-label text-subtle-foreground">{{ t('profileBlock.missing') }}</h3>
        <ul class="flex flex-col gap-1">
          <li v-for="group in groups" :key="group.kind" class="flex flex-wrap gap-2">
            <Button
              v-for="entry in group.entries"
              :key="entry.key"
              :variant="ButtonVariant.Ghost"
              :size="ButtonSize.Compact"
              :disabled="entry.location === null"
              @click="open(entry.location)"
              >{{ entry.name }}</Button
            >
          </li>
        </ul>
      </section>

      <section v-if="gets.length > 0" class="flex flex-col gap-2">
        <h3 class="text-label text-subtle-foreground">{{ t('profileBlock.unlocks') }}</h3>
        <div class="flex flex-wrap gap-2">
          <Button
            v-for="entry in gets"
            :key="entry.key"
            :variant="ButtonVariant.Outline"
            :size="ButtonSize.Compact"
            :disabled="entry.location === null"
            @click="open(entry.location)"
            >{{ entry.name }}</Button
          >
        </div>
      </section>

      <p class="text-body text-subtle-foreground">
        {{ fanOut > 0 ? t('profileBlock.opens', { count: fanOut }) : t('profileBlock.opensNothing') }}
        <span v-if="stepsMissing !== null">{{ t('profileBlock.stepsMissing', { count: stepsMissing }) }}</span>
      </p>

      <span v-if="node.done" class="text-caption text-state-done-foreground">{{ t('profileBlock.done') }}</span>
      <span v-else-if="queued" class="text-caption text-state-done-foreground">{{ t('queue.inPlan') }}</span>
      <Button
        v-else-if="canAdd"
        :variant="ButtonVariant.Outline"
        :size="ButtonSize.Compact"
        :disabled="busy"
        @click="emit('add')"
        ><ListPlusIcon />{{ t('queue.add') }}</Button
      >
    </CardContent>
  </Card>
</template>
```

Check `ButtonVariant`/`ButtonSize` and the `Card` sub-components against
`ui/src/components/ui/` before writing — use the names that exist, and if a ghost variant is
missing, use `ButtonVariant.Outline` rather than inventing one.

- [ ] **Step 3: Wire it into the wiki page**

`ui/src/screens/wiki/WikiPage.vue`, in `<script setup>`:

```ts
import ProfileBlock from '@/components/graph/ProfileBlock.vue'
import { achievementNode } from '@/lib/graph/achievementNode'
import { canQueue, isQueued, queuedIds } from '@/lib/plan/queueRows'
import { nodeSlot } from '@/lib/graph/unlockFilter'
import { useGraphStore } from '@/stores/views'
import { useQueueStore } from '@/stores/queue'

const graph = useGraphStore()
const queue = useQueueStore()

// The wiki is reachable without a profile, so the block is absent far more often than it is
// there: `achievementNode` answers `null` for every one of those cases (spec §3.3), and the
// page renders exactly as it did before.
const node = computed(() =>
  achievementNode(graph.view?.unlock ?? null, target.value),
)
const queued = computed(() => queuedIds(queue.view))
</script>
```

and in the template, immediately before `<WikiInfobox …>`:

```vue
      <ProfileBlock
        v-if="node"
        :node="node"
        :queued="isQueued(node, queued)"
        :can-add="queue.view?.storeAvailable === true && canQueue(node, queued)"
        :busy="queue.busy"
        @add="queue.add(nodeSlot(node))"
      />
```

**Do not make the wiki page load the graph.** It reads what is there; the progress screens
load it. This is deliberate: a wiki tab must not pull a profile-shaped command.

- [ ] **Step 4: Add the Kit section**

`ui/src/kit/sections/app/ProfileBlockSection.vue` — the block in four states, from fixtures:
unlockable now, blocked with several requirements, done, and a node that unlocks nothing.
Register it wherever the Kit lists its app sections (`grep -rn "NavBarSection" ui/src/kit`).

- [ ] **Step 5: Look at it**

Run: `pnpm ui:dev`, open `#kit` and then `#/wiki?category=achievements&page=achievement:19`
with `?fixture=active`.
Expected: the block above the infobox, and nothing at all with `?fixture=none`.

- [ ] **Step 6: Run the checks**

Run: `pnpm typecheck && pnpm ui:test && pnpm lint && pnpm scan`
Expected: PASS, `0 violations`

- [ ] **Step 7: Commit**

```bash
git add ui/src/components/graph/ProfileBlock.vue ui/src/kit/sections/app/ProfileBlockSection.vue ui/src/screens/wiki/WikiPage.vue ui/src/i18n/messages/it.ts ui/src/i18n/messages/en.ts
git commit -m "feat(ui): an achievement's wiki page says where your profile stands"
```

---

## Task 6: The card says what you get, how, and why

**Files:**
- Create: `ui/src/lib/graph/goalCard.ts`, `ui/src/lib/graph/goalCard.test.ts`
- Create: `ui/src/screens/goals/GoalCard.vue` (the folder is created here; the screen moves in Task 7)
- Modify: `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts`

**Interfaces:**
- Consumes: `UnlockNode`, `targetName`, `pageLocation`.
- Produces: `interface GoalCardModel { headline: string; condition: string | null; art: string | null; fanOut: number; location: TabLocation | null }` and `goalCard(node: UnlockNode, t: Translate): GoalCardModel`.

- [ ] **Step 1: Write the failing test**

```ts
import { describe, expect, it } from 'vitest'
import type { UnlockNode } from '@/lib/ipc/types'
import { RouteName, WikiCategory } from '@/router/routeTable'
import { goalCard } from './goalCard'

const t = ((key: string) => key) as never

const base: UnlockNode = {
  achievement: {
    kind: 'known',
    id: 484,
    text: 'You unlocked "The Lost"',
    hint: 'Arriva a Home e usa la Red Key',
    iconUrl: 'isaac://achievement/484',
  },
  done: false,
  unlocks: [],
  origin: null,
  missing: [],
  graph: { kind: 'computed', availableNow: true, blockedBy: 0, fanOut: 23, stepsMissing: 0 },
}

describe('goalCard', () => {
  // B28: the file writes the base name, the player is unlocking the Tainted form. The
  // headline is what you get, in its own form — never the achievement's own text.
  it('leads with what you get, not with what the file says', () => {
    const card = goalCard(
      {
        ...base,
        unlocks: [{ kind: 'character', id: 30, name: 'The Lost', tainted: true, page: null }],
      },
      t,
    )
    expect(card.headline).toContain('Lost')
    expect(card.headline).not.toBe('You unlocked "The Lost"')
  })

  it('falls back to the achievement text when it unlocks nothing catalogued', () => {
    expect(goalCard(base, t).headline).toBe('You unlocked "The Lost"')
  })

  it('carries the game own unlock condition as the one line under it', () => {
    expect(goalCard(base, t).condition).toBe('Arriva a Home e usa la Red Key')
  })

  it('has no condition when the game states none', () => {
    const card = goalCard({ ...base, achievement: { ...base.achievement, hint: null } } as UnlockNode, t)
    expect(card.condition).toBeNull()
  })

  it('links the whole card to the achievement page', () => {
    expect(goalCard(base, t).location).toEqual({
      name: RouteName.Wiki,
      query: { category: WikiCategory.Achievements, page: 'achievement:484' },
    })
  })

  it('has nowhere to go for a slot the catalog does not name', () => {
    const card = goalCard({ ...base, achievement: { kind: 'unknown', slot: 640 } }, t)
    expect(card.location).toBeNull()
  })
})
```

- [ ] **Step 2: Run it to verify it fails**

Run: `pnpm --filter ui test src/lib/graph/goalCard.test.ts`
Expected: FAIL — cannot resolve `./goalCard`.

- [ ] **Step 3: Write the implementation**

```ts
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import type { UnlockNode } from '@/lib/ipc/types'
import { pageLocation } from '@/lib/wiki/category'
import type { TabLocation } from '@/router/routeTable'
import { targetName } from './characterName'

type Translate = (
  key: MessageKey<MessageSchema>,
  params?: Record<string, unknown>,
) => string

// More than one thing out of one achievement is rare and real (a challenge's rewards): the
// names are listed, not counted.
const Separator = ' · '

// A row of the landing page, top to bottom: what you get, how you get it, where to read
// more. The component draws it and decides nothing (`DESIGN-BRIEF.md` §7.1).
export interface GoalCardModel {
  headline: string
  /** The game's own `unlock_condition`. `null` when the file states none. */
  condition: string | null
  art: string | null
  fanOut: number
  /** The achievement's page — the detail. `null` for a slot the catalog does not name. */
  location: TabLocation | null
}

export const goalCard = (node: UnlockNode, t: Translate): GoalCardModel => {
  const a = node.achievement
  const known = a.kind === 'known'
  // What you get comes first; the achievement's text is the fallback, not the headline,
  // because the file names a Tainted character by its base form (`docs/BACKLOG.md` B28).
  const unlocked = node.unlocks.map((u) => targetName(t, u)).join(Separator)
  const fallback = known ? a.text : `${t('graph.unknownAchievement')} · ${t('graph.slot')} ${a.slot}`
  return {
    headline: unlocked === '' ? fallback : unlocked,
    condition: known ? a.hint : null,
    art: known ? a.iconUrl : null,
    fanOut: node.graph.fanOut,
    location: known ? pageLocation({ kind: 'achievement', id: a.id }) : null,
  }
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `pnpm --filter ui test src/lib/graph/goalCard.test.ts`
Expected: PASS (6 tests)

- [ ] **Step 5: Add the card's copy**

`it.ts` (and the mirror in `en.ts`):

```ts
  goals: {
    opens: 'Apre altre {count} cose',
    opensNothing: 'Non apre altro',
    seeAll: 'Vedile tutte',
  },
```

- [ ] **Step 6: Write the component** — `ui/src/screens/goals/GoalCard.vue`

Built on `goalCard()`, drawing in this order: the headline (`text-body`), the condition
(`text-caption text-subtle-foreground`, absent when `null`), the art through
`AchievementArt`, the "apre altre N cose" sentence, and the add button. **No state badge** —
every row of this page is unlockable now. The whole card navigates on click and opens beside
on Ctrl, through `useGestureModifiers` and `useTabsStore` exactly as `WhyMenu` does; the add
button must call `event.stopPropagation()` so adding to the Plan does not also navigate.

- [ ] **Step 7: Check and commit**

Run: `pnpm typecheck && pnpm ui:test && pnpm lint && pnpm scan`

```bash
git add ui/src/lib/graph/goalCard.ts ui/src/lib/graph/goalCard.test.ts ui/src/screens/goals/GoalCard.vue ui/src/i18n/messages/it.ts ui/src/i18n/messages/en.ts
git commit -m "feat(ui): a step card leads with what you get and links to its page"
```

---

## Task 7: "Obiettivi consigliati" — the page, the name, the sections

**Files:**
- Rename: `ui/src/screens/NextStepsScreen.vue` → `ui/src/screens/GoalsScreen.vue`; delete `ui/src/screens/nextSteps/StepCard.vue` (replaced by Task 6's `GoalCard.vue`)
- Modify: `ui/src/router/routeTable.ts`, `ui/src/router/routes.ts`, `ui/src/router/routeTable.test.ts`
- Modify: `ui/src/screens/UnlockScreen.vue`
- Modify: `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts`

**Interfaces:**
- Consumes: `NextSteps.sections` (Task 2), `goalCard` and `GoalCard.vue` (Task 6).
- Produces: `RouteName.Goals` (`'goals'`, path `/progress/goals`, title key `routes.goals`); `TabLocation.query.state?: string`.

- [ ] **Step 1: Rename the route, everywhere**

Run: `grep -rn "nextSteps\|NextSteps\|next-steps" ui/src crates/app/src`
Every hit is either the IPC command name (`Command.NextSteps` and the Rust
`#[tauri::command] pub fn next_steps` — **these keep their names**, they are the graph's
answer, not the screen's) or the screen's identity (`RouteName.NextSteps`, `routePath`,
`routeTitle`, `routeOrigin`, `routeIcon`, `defaultLocation`, the `screens` map, the router
test). Rename the second group to `Goals` / `'goals'` / `/progress/goals` / `routes.goals`.

- [ ] **Step 2: Run the router's own tests**

Run: `pnpm --filter ui test src/router/routeTable.test.ts`
Expected: PASS once the test's expectations are updated to `goals`.

- [ ] **Step 3: Rewrite the copy**

In `it.ts`, `routes.nextSteps` becomes `routes.goals: 'Obiettivi consigliati'` (en:
`'Suggested goals'`), and the `nextSteps` group becomes `goals`, joined with Task 6's keys:

```ts
  goals: {
    intro: 'Cose che puoi sbloccare adesso. In alto quelle che ne aprono di più, sotto quelle a cui sei più vicino.',
    fanOut: 'Aprono di più',
    closeness: 'Ci sei quasi',
    inPlan: 'Nel tuo Piano',
    opens: 'Apre altre {count} cose',
    opensNothing: 'Non apre altro',
    seeAll: 'Vedile tutte',
    noCatalogTitle: 'Niente da consigliare senza il gioco',
    noCatalog: 'Senza il gioco installato non si sa cosa sblocca cosa: meglio una lista vuota che cinque righe indovinate.',
    nothingNow: 'Non c’è niente da sbloccare adesso: o è tutto fatto, o tutto aspetta qualcos’altro.',
  },
```

The old intro — "Al massimo cinque righe… non possiamo garantirlo" — is the computation, and
it moves to the About dialog's promises (`grep -rn "promises" ui/src` for where those live).

- [ ] **Step 4: Write the section heading map**

In `ui/src/screens/goals/sectionTitle.ts`, a `Record` over the whole enum so a new basis
fails to compile:

```ts
import type { MessageKey } from '@/i18n/messageKey'
import type { MessageSchema } from '@/i18n/messages/it'
import { StepsBasis } from '@/lib/ipc/types'

// Why these rows are here. A record over the whole set: a basis with no heading is a build
// error, not a blank title.
export const sectionTitle: Record<StepsBasis, MessageKey<MessageSchema>> = {
  [StepsBasis.FanOut]: 'goals.fanOut',
  [StepsBasis.Closeness]: 'goals.closeness',
}
```

- [ ] **Step 5: Rewrite the screen**

`ui/src/screens/GoalsScreen.vue`: the same loading, error and empty handling as today, with
the list replaced by one block per section — heading from `sectionTitle`, `GoalCard` per
step, and a "vedile tutte" link at the end of each. The empty and `noCatalog` branches keep
their structure and take the new keys.

The "vedile tutte" target, for both sections, is Unlock filtered to unlockable-now:

```ts
const seeAll: TabLocation = {
  name: RouteName.Unlock,
  query: { state: NodeState.Now },
}
```

- [ ] **Step 6: Let Unlock read that filter**

`ui/src/router/routeTable.ts`, `TabLocation`:

```ts
export interface TabLocation {
  name: RouteName
  query?: { category?: WikiCategory; page?: string; q?: string; state?: string }
}
```

`ui/src/screens/UnlockScreen.vue`, beside the existing `route.query.q` watcher — same shape,
same reason (a link arrives with the list already filtered):

```ts
watch(
  () => route.query.state,
  (raw) => {
    const state = singleQuery(raw)
    if (state === null) return
    const known = stateOrder.find((s) => s === state)
    if (known) filter.value = { ...filter.value, picks: { ...filter.value.picks, [FacetId.State]: [known] } }
  },
  { immediate: true },
)
```

- [ ] **Step 7: Look at every state**

Run: `pnpm ui:dev` and walk `?fixture=active`, `?fixture=none`, `?catalog=none`,
`?queue=unavailable`.
Expected: headings only over sections that exist; no state badge on a card; a card click
navigates to the achievement page, Ctrl+click opens it beside; "vedile tutte" lands on Unlock
with **Sbloccabile ora** already picked.

- [ ] **Step 8: Run the full check**

Run: `pnpm check`
Expected: everything green, `0 violations`

- [ ] **Step 9: Commit**

```bash
git add ui/src/screens/GoalsScreen.vue ui/src/screens/goals ui/src/router ui/src/screens/UnlockScreen.vue ui/src/i18n/messages/it.ts ui/src/i18n/messages/en.ts
git rm ui/src/screens/NextStepsScreen.vue ui/src/screens/nextSteps/StepCard.vue
git commit -m "feat(ui): the landing page is Obiettivi consigliati, grouped by reason"
```

---

## Task 8: "Nel tuo Piano" on the landing page

**Files:**
- Create: `ui/src/lib/plan/planNow.ts`, `ui/src/lib/plan/planNow.test.ts`
- Modify: `ui/src/screens/GoalsScreen.vue`

**Interfaces:**
- Consumes: `QueueView`, `QueueRow`, `nodeState`/`NodeState` from `@/lib/graph/nodeState`.
- Produces: `planNow(view: QueueView | null, limit: number): QueueRow[]`.

- [ ] **Step 1: Write the failing test**

```ts
import { describe, expect, it } from 'vitest'
import type { QueueRow, QueueView } from '@/lib/ipc/types'
import { planNow } from './planNow'

const row = (id: number, availableNow: boolean, done: boolean): QueueRow => ({
  node: {
    achievement: { kind: 'known', id, text: `t${id}`, hint: null, iconUrl: null },
    done,
    unlocks: [],
    origin: null,
    missing: [],
    graph: { kind: 'computed', availableNow, blockedBy: 0, fanOut: 0, stepsMissing: 0 },
  },
  wanted: true,
  origins: [],
})

const view = (rows: QueueRow[]): QueueView => ({ rows, storeAvailable: true, diagnostics: [] })

describe('planNow', () => {
  it('keeps the queue order and only what can be played now', () => {
    const got = planNow(view([row(1, false, false), row(2, true, false), row(3, true, false)]), 5)
    expect(got.map((r) => r.node.achievement.kind === 'known' && r.node.achievement.id)).toEqual([2, 3])
  })

  it('leaves out what is already done', () => {
    expect(planNow(view([row(4, true, true)]), 5)).toEqual([])
  })

  it('answers nothing without a queue', () => {
    expect(planNow(null, 5)).toEqual([])
  })

  it('never shows more than the limit', () => {
    expect(planNow(view([row(1, true, false), row(2, true, false)]), 1)).toHaveLength(1)
  })
})
```

Check `QueueView`'s real shape in `ui/src/lib/ipc/types.ts` first and match the fixture to it.

- [ ] **Step 2: Run it to verify it fails**

Run: `pnpm --filter ui test src/lib/plan/planNow.test.ts`
Expected: FAIL — cannot resolve `./planNow`.

- [ ] **Step 3: Write the implementation**

```ts
import { NodeState, nodeState } from '@/lib/graph/nodeState'
import type { QueueRow, QueueView } from '@/lib/ipc/types'

// What you had already decided, on the page that suggests. The queue's own order is the
// answer — this filters, it never re-sorts: the order in the Plan is the user's, and a
// landing page that reordered it would be contradicting the screen that owns it.
export const planNow = (view: QueueView | null, limit: number): QueueRow[] =>
  (view?.rows ?? [])
    .filter((row) => nodeState(row.node) === NodeState.Now)
    .slice(0, limit)
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `pnpm --filter ui test src/lib/plan/planNow.test.ts`
Expected: PASS (4 tests). `nodeState` answers `Done` before it answers `Now`, which is what
makes the second test pass without a `done` check of its own.

- [ ] **Step 5: Draw the block**

In `GoalsScreen.vue`, under the sections: the heading `goals.inPlan`, the rows through
`GoalCard` (or a compact variant if the card is too tall — a prop, not a second component),
and a link to `RouteName.Plan`. **Absent** when `planNow` returns nothing, when
`queue.view === null`, or when `storeAvailable` is false — the sections above are unaffected
in all three cases.

- [ ] **Step 6: Look at it, then check**

Run: `pnpm ui:dev` with `?queue=empty`, `?queue=unavailable`, `?queue=unreadable`
Expected: the block absent in all three, the page otherwise unchanged.

Run: `pnpm check`

- [ ] **Step 7: Commit**

```bash
git add ui/src/lib/plan/planNow.ts ui/src/lib/plan/planNow.test.ts ui/src/screens/GoalsScreen.vue
git commit -m "feat(ui): the landing page shows what you had already put in the Plan"
```

---

## Task 9: Close the backlog items and write the report

**Files:**
- Modify: `docs/STATUS.md`, `docs/BACKLOG.md`
- Create: `docs/superpowers/reports/2026-09-13-goals-and-achievement-detail-report.md`

- [ ] **Step 1: Run the whole check and record the numbers**

Run: `pnpm check`
Record: how many tests passed **and how many skipped** (`cargo test --workspace -- --nocapture`
is what shows the `skip:` lines; a bare "N passed" does not say how many were silent).

- [ ] **Step 2: Mark B32 and B35 closed**

In `docs/BACKLOG.md`, add `✅ closed on 2026-09-13` to both headings, the way B24 and B28 do
it. Under B35, record the decision the item asked for: **a single target is its own link, not
a menu of one**.

- [ ] **Step 3: Update `docs/STATUS.md`**

The session log gets the sub-project, and the M2 line "Handed to the design system" —
`RequirementView` gained `mark` and `counter`, and Unlock's count rises — can now be ticked:
the closeness section is what consumes it. **A ticked box means committed work** — tick only
what actually landed, and if the closeness section skipped on real data (Task 2, Step 7), say
so in the same line rather than leaving it implied.

- [ ] **Step 4: Write the report**

`docs/superpowers/reports/2026-09-13-goals-and-achievement-detail-report.md`, following the
shape of `2026-09-07-unlock-graph-report.md`: what was built, the decisions taken during
execution and why, what the tests actually covered (including what skipped), and what is left
open — B33, B36, and the profile block for the other four kinds.

- [ ] **Step 5: Commit**

```bash
git add docs/STATUS.md docs/BACKLOG.md docs/superpowers/reports/2026-09-13-goals-and-achievement-detail-report.md
git commit -m "docs: Obiettivi consigliati and the achievement detail land, B32 and B35 closed"
```

---

## Self-review notes

- **Spec coverage.** §2 and §2.1 → Task 5 (the block hosted on the wiki page, catalog winning
  in the block). §3.1 → Tasks 3–5. §3.2 → Task 3. §3.3 → Task 3's five tests. §4.1 → Task 7
  Steps 1–3. §4.2 → Task 2 (the sectioning and its rules) and Task 7 (the headings and "vedile
  tutte"). §4.3 → Task 8. §5 → Task 6. §6.1 → Task 2. §6.2 → Task 1. §6.3 → the mirror steps
  inside Tasks 1 and 2. §7 → the test steps throughout, with the vacuity guard in Task 2 Step 6.
  §8 → the Global Constraints' branch line and Task 9's report.
- **Types are consistent across tasks**: `achievementNode` (Task 3) is consumed by Task 5;
  `unlockEntries` (Task 4) by Task 5; `goalCard` (Task 6) by Tasks 6–7; `planNow` (Task 8) by
  Task 8. `UnlockTarget.page` (Task 1) is what `unlockEntries` reads. `StepsSection` (Task 2)
  is what `sectionTitle` (Task 7) keys on.
- **Two things the executor must look up rather than trust**: the exact names in
  `ui/src/components/ui/button` and `card`, and `targetName`'s signature in
  `characterName.ts`. Both are flagged in place.
