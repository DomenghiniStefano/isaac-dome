# B37 — Searching the unlock graph from the goal — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Name a thing you want — an item, a character, a boss, a challenge, or an achievement by its own name — and get the ordered series of what you still have to play for it, with one gesture that puts the whole series in the Plan.

**Architecture:** One new pure module, `crates/ipc/src/want.rs`, takes a `wiki::Target`, resolves it to the achievements that grant it, asks `graph::Eval::missing_chain` what is still missing, and orders it by enqueueing into an **empty throwaway `plan::Queue`** — so the preview and the write ("metti nel Piano", the existing `queue_add`) are one computation. The Goals screen grows a "voglio…" bar whose want lives in the URL (`?want=item:105`, the existing `pageKey` codec), and draws the answer with 3.6's cards.

**Tech Stack:** Rust (`crates/ipc`, `crates/app`), Vue 3 + TypeScript (`ui/`), Vitest, `cargo test`.

**Spec:** `docs/superpowers/specs/2026-09-13-goals-want-design.md`

## Global Constraints

- **Branch:** `feature/goals-want`, already cut from `develop` at `a83a7ef` (3.6 merged), in the worktree `C:/Projects/isaac-dome-want`. Work there, not in `C:/Projects/isaac-dome` — that copy belongs to another session.
- **The tree may hold other sessions' work.** Never `git add -A`; stage by explicit path, exactly the paths each task's commit step lists. This already went wrong once on 2026-09-13: a `git add` landed in another session's in-progress merge.
- **Every struct crossing the IPC** carries `#[serde(rename_all = "camelCase")]`; enums with struct variants also need `rename_all_fields = "camelCase"`.
- **Enums on the IPC are tagged with struct variants** (`#[serde(tag = "kind")]`). Fieldless enums we define are bare camelCase strings instead — none of the enums in this plan is fieldless, so all of them are tagged.
- **Exhaustiveness is mandatory**: no `_ =>` arm on a closed enum, in Rust or in a TypeScript `switch` (use `assertNever`). The one exception already in the repo is the pair-of-enums match in `achievement_unlocking`, whose four meaningful combinations are written out above it.
- **Never `panic!` or `unwrap()`** outside tests on data read from disk.
- **`ui/src/lib/ipc/types.ts` is hand-mirrored** and changes in the same commit as the Rust type. A contract change is handed on, not merely committed.
- **Frontend rules** (`docs/frontend-conventions.md`, enforced by `pnpm scan`): no `<style>` in SFCs, no hardcoded visual constants (no `w-[48px]`, no `:size="16"`), no `invoke()` in components, no raw `<button>`/`<input>`, no string unions (`const X = { … } as const`), no visible strings in templates — every one goes through `t()` and exists in **both** `ui/src/i18n/messages/it.ts` and `en.ts`.
- **Test-first.** The expected value comes from the spec, never from the code's current output. A failing test is first a hypothesis of a bug in the code.
- **Tests on real data skip with a note** (`skip: …` on stderr) when the sample is missing, and are reached only through `test-support`.
- **Before declaring anything done:** `pnpm check` (fmt, clippy `-D warnings`, `cargo test --workspace`, typecheck, `ui:test`, lint, format:check, scan).
- **Commits:** Conventional Commits, `type(scope): subject`, English, atomic. **Never** a `Co-Authored-By` trailer.

---

## File Structure

**Rust — `crates/ipc`**
- `src/want.rs` — **new**. The view types, the `Target` → `TargetKey` conversion, the route lookup, the four states, the ordering. The whole answer, in one file, because it is one question.
- `src/queue.rs` — `achievements_unlocking` (new, returns every route); `achievement_unlocking` rewritten over it.
- `src/lib.rs` — `mod want;` and the `pub use`.
- `tests/want.rs` — **new**. Everything on synthetic catalogs.
- `tests/want_real.rs` — **new**. The three real-data facts, each with its vacuity guard.

**Rust — `crates/app`**
- `src/commands/graph.rs` — the `want` command, wiring only.
- `src/lib.rs` — one line in `generate_handler!`.

**Frontend**
- `ui/src/lib/ipc/types.ts` — the mirror of the four new types.
- `ui/src/lib/ipc/graph.ts` — the `want(target)` wrapper.
- `ui/src/lib/constants/commands.ts` — `Want: 'want'`.
- `ui/src/lib/ipc/fixtures/graph.ts` — a fixture want, for `pnpm ui:dev` without a backend.
- `ui/src/router/routeTable.ts` — `TabLocation.query` gains `want`.
- `ui/src/lib/graph/wantLocation.ts` (+ `.test.ts`) — **new**. The want ⇄ location codec, over `pageKey`/`parsePageKey`.
- `ui/src/lib/graph/wantBlocks.ts` (+ `.test.ts`) — **new**. The pure model of what a block draws, per state.
- `ui/src/composables/useWant.ts` — **new**. The keyed read (`defineViewStore` takes no argument; a want does).
- `ui/src/screens/goals/WantBar.vue`, `ui/src/screens/goals/WantAnswer.vue` — **new**. The bar and the answer.
- `ui/src/screens/GoalsScreen.vue` — hosts both; the recommendations step aside while a want is active.
- `ui/src/i18n/messages/it.ts`, `en.ts` — the new strings.
- `ui/src/kit/sections/app/WantAnswerSection.vue` — **new**. The four states on the Kit page.

---

## Task 1: Every route, not the first one

**Files:**
- Modify: `crates/ipc/src/queue.rs:64-88`
- Test: `crates/ipc/tests/queue.rs`

**Interfaces:**
- Produces: `pub fn achievements_unlocking(c: &Catalog, key: &crate::goals::TargetKey) -> Vec<u32>` — every achievement whose `unlocks` names this target, in catalog order.
- Produces (unchanged contract): `pub fn achievement_unlocking(c: &Catalog, key: &crate::goals::TargetKey) -> Option<u32>`, now `achievements_unlocking(..).first().copied()`.

**Why:** today's `.find()` takes the first achievement granting a target and never says there were others. A challenge's `unlocked_by` is a `Vec` (`crates/catalog/src/catalog.rs:307`, `for a in &ch.unlocked_by`), so two routes are a shape the data really has.

- [ ] **Step 1: Write the failing test** — append to `crates/ipc/tests/queue.rs`

```rust
/// A challenge's `achievements` attribute is a list, so a target really can have two ways
/// in. `achievement_unlocking` answers with the first and says nothing about the second;
/// the view B37 builds has to be able to show both.
#[test]
fn a_target_named_by_two_achievements_has_two_routes() {
    let c = catalog::Catalog::build(|p| match p {
        "achievements.xml" => Some(
            b"<achievements gfxroot=\"g/\"><achievement id=\"1\" text=\"t1\" gfx=\"1.png\" /><achievement id=\"2\" text=\"t2\" gfx=\"2.png\" /></achievements>"
                .to_vec(),
        ),
        "challenges.xml" => Some(
            b"<challenges version=\"1\"><challenge id=\"4\" name=\"Both\" achievements=\"1,2\" endstage=\"1\" /></challenges>"
                .to_vec(),
        ),
        _ => None,
    });
    let key = ipc::TargetKey::Challenge { id: 4 };
    assert_eq!(ipc::achievements_unlocking(&c, &key), vec![1, 2]);
    // The old function keeps its contract: the first, and only the first.
    assert_eq!(ipc::achievement_unlocking(&c, &key), Some(1));
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p ipc --test queue a_target_named_by_two_achievements_has_two_routes`
Expected: FAIL — `achievements_unlocking` does not exist (`E0425`).

- [ ] **Step 3: Write the implementation** — in `crates/ipc/src/queue.rs`, replace the body of `achievement_unlocking` and add the new function above it

```rust
/// Every achievement whose `unlocks` names this target, in the catalog's order.
///
/// A list and not an `Option`: a challenge's `unlocked_by` is a list in the game's own file,
/// so two ways in is a shape the data has. Picking one silently is a wrong answer wearing a
/// right one's clothes.
pub fn achievements_unlocking(c: &Catalog, key: &crate::goals::TargetKey) -> Vec<u32> {
    use crate::catalog_view::item_kind;
    use crate::goals::TargetKey;
    use catalog::Unlock;
    c.achievements()
        .filter(|a| {
            c.unlocks(a.id).iter().any(|u| match (u, key) {
                (
                    Unlock::Item { kind, id },
                    TargetKey::Item {
                        item_kind: k,
                        id: want,
                    },
                ) => item_kind(*k) == *kind && id.0 == *want,
                (Unlock::Character { id }, TargetKey::Character { id: want }) => id.0 == *want,
                (Unlock::Boss { id }, TargetKey::Boss { id: want }) => id.0 == *want,
                (Unlock::Challenge { id }, TargetKey::Challenge { id: want }) => id.0 == *want,
                // A pair of enums has sixteen combinations of which four mean anything.
                // The exhaustiveness rule bans a catch-all that hides a new variant of one
                // closed enum; this one hides nothing — the four are written out above it.
                _ => false,
            })
        })
        .map(|a| a.id.0)
        .collect()
}

/// The first way in, which is all the goals import needs: it resolves a goal to one queue
/// row. Everything that has to *show* the ways uses `achievements_unlocking`.
pub fn achievement_unlocking(c: &Catalog, key: &crate::goals::TargetKey) -> Option<u32> {
    achievements_unlocking(c, key).first().copied()
}
```

- [ ] **Step 4: Export it** — in `crates/ipc/src/lib.rs`, add `achievements_unlocking` to the `pub use queue::{…}` list, keeping the list alphabetical.

- [ ] **Step 5: Run the test and the neighbours**

Run: `cargo test -p ipc --test queue`
Expected: PASS, and the existing `queue_import_goals` tests unchanged.

- [ ] **Step 6: Commit**

```bash
git add crates/ipc/src/queue.rs crates/ipc/src/lib.rs crates/ipc/tests/queue.rs
git commit -m "feat(ipc): a target names every achievement that grants it, not the first"
```

---

## Task 2: The view's shape, and the two answers that are not a route

**Files:**
- Create: `crates/ipc/src/want.rs`
- Modify: `crates/ipc/src/lib.rs`
- Test: `crates/ipc/tests/want.rs` (create)

**Interfaces:**
- Produces: `pub struct WantView { wanted: WantedView, routes: Vec<WantRoute>, diagnostics: Vec<WantDiagnostic> }`
- Produces: `pub enum WantedView { Target { target: UnlockTarget }, Achievement { achievement: AchievementRef }, Unresolved }`
- Produces: `pub struct WantRoute { node: UnlockNode, state: WantState }`
- Produces: `pub enum WantState { Done, AvailableNow, Chain { steps: Vec<UnlockNode>, unknown: u32 }, NoProfile }`
- Produces: `pub enum WantDiagnostic { NoCatalog, NoProfile, NothingUnlocks, NotUnlockable }`
- Produces: `pub fn want_view(catalog: Option<&Catalog>, view: &UnlockView, flags: Option<&[bool]>, eval: Option<&graph::evaluate::Eval>, target: &Target, icon: impl FnMut(&IconRef) -> Option<String>) -> WantView` — in this task it answers only the cases that have no route, and `flags`/`eval` are unused (`_flags`, `_eval`). **The full signature is written now on purpose**: Tasks 3, 4 and 5 add meaning, not parameters, so no test written here has to be edited later to keep compiling.
- Consumes: `UnlockView`, `UnlockNode`, `UnlockTarget`, `AchievementRef` from `crate::graph`.

- [ ] **Step 1: Write the failing test** — create `crates/ipc/tests/want.rs`

```rust
use ipc::{Target, WantDiagnostic, WantedView};

mod support;
use support::{catalog_with_achievements, empty_view};

#[test]
fn a_stage_is_not_a_thing_you_unlock() {
    let c = catalog_with_achievements();
    let v = ipc::want_view(
        Some(&c),
        &empty_view(),
        None,
        None,
        &Target::Stage {
            name: "Basement".into(),
        },
        |_| None,
    );
    assert_eq!(v.wanted, WantedView::Unresolved);
    assert!(v.routes.is_empty());
    assert_eq!(v.diagnostics, vec![WantDiagnostic::NotUnlockable]);
}

#[test]
fn without_a_catalog_nothing_resolves_and_the_view_says_so() {
    let v = ipc::want_view(None, &empty_view(), None, None, &Target::Item { id: 2 }, |_| {
        None
    });
    assert_eq!(v.wanted, WantedView::Unresolved);
    assert!(v.routes.is_empty());
    assert_eq!(v.diagnostics, vec![WantDiagnostic::NoCatalog]);
}
```

Create `crates/ipc/tests/support/mod.rs` with the two helpers (the XML is the one
`tests/graph.rs` already uses; a challenge is added for Task 3's two-route case):

```rust
use catalog::Catalog;

const ITEMS: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"2\" gfx=\"a.png\" name=\"A\" achievement=\"1\" /><trinket id=\"1\" gfx=\"t.png\" name=\"T\" achievement=\"3\" /></items>";
const ACH: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\"><achievement id=\"1\" text=\"t1\" gfx=\"1.png\" /><achievement id=\"2\" text=\"t2\" gfx=\"2.png\" /><achievement id=\"3\" text=\"t3\" gfx=\"3.png\" /></achievements>";
const PLAYERS: &[u8] = b"<players portraitroot=\"gfx/ui/stage/\"><player id=\"7\" name=\"#Z_NAME\" portrait=\"z.png\" achievement=\"2\" /></players>";
const CHALLENGES: &[u8] = b"<challenges version=\"1\"><challenge id=\"4\" name=\"Both\" achievements=\"1,2\" endstage=\"1\" /></challenges>";

/// Every item, character and challenge points at an achievement: the relationship this view
/// reads. Challenge 4 is named by two achievements, which is the two-route case.
pub fn catalog_with_achievements() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS.to_vec()),
        "achievements.xml" => Some(ACH.to_vec()),
        "players.xml" => Some(PLAYERS.to_vec()),
        "challenges.xml" => Some(CHALLENGES.to_vec()),
        _ => None,
    })
}

/// A view with no nodes: enough for the answers that never reach one.
pub fn empty_view() -> ipc::UnlockView {
    ipc::for_tests::unlock_view_of(vec![])
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test -p ipc --test want`
Expected: FAIL — `want_view` and the types do not exist.

- [ ] **Step 3: Write the implementation** — create `crates/ipc/src/want.rs`

```rust
//! "I want this — what do I have to play?", the graph read from the other end.
//!
//! Everything here is a view over what `graph.rs` already computed: this module adds no
//! traversal of its own. It resolves what you named, finds the achievements that grant it,
//! and asks the chain — in the order the Plan would play it (see `route_state`).

use catalog::Catalog;
use serde::Serialize;
use wiki::Target;

use crate::graph::{AchievementRef, UnlockNode, UnlockTarget, UnlockView};
use crate::icon::IconRef;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WantView {
    pub wanted: WantedView,
    pub routes: Vec<WantRoute>,
    pub diagnostics: Vec<WantDiagnostic>,
}

/// A want is one of two things, and `UnlockTarget` can only be one of them: it has four
/// variants and none is an achievement. Naming *Greedier!* has to reach the node whose
/// `unlocks` the catalog does not model, so the view carries both — and a third case for a
/// name the catalog no longer resolves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum WantedView {
    Target { target: UnlockTarget },
    Achievement { achievement: AchievementRef },
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WantRoute {
    pub node: UnlockNode,
    pub state: WantState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum WantState {
    /// Already yours.
    Done,
    /// Nothing in the way: play it.
    AvailableNow,
    /// The prerequisites in the order the Plan would play them, the wanted node excluded.
    /// `unknown` counts the steps — the final node included — whose requirements the graph
    /// only partly interprets: a chain that cannot see everything says so in a number.
    Chain { steps: Vec<UnlockNode>, unknown: u32 },
    /// Section 1 was not read. The route is named; where you stand is not claimed.
    NoProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum WantDiagnostic {
    NoCatalog,
    NoProfile,
    /// `routes` is empty, and this is why: no node in the graph grants this.
    NothingUnlocks,
    /// A stage, a room, a pickup, a transformation: not a thing you unlock.
    NotUnlockable,
}

pub fn want_view(
    catalog: Option<&Catalog>,
    _view: &UnlockView,
    _flags: Option<&[bool]>,
    _eval: Option<&graph::evaluate::Eval>,
    target: &Target,
    _icon: impl FnMut(&IconRef) -> Option<String>,
) -> WantView {
    let unresolved = |d: WantDiagnostic| WantView {
        wanted: WantedView::Unresolved,
        routes: Vec::new(),
        diagnostics: vec![d],
    };
    if !unlockable(target) {
        return unresolved(WantDiagnostic::NotUnlockable);
    }
    let Some(_c) = catalog else {
        return unresolved(WantDiagnostic::NoCatalog);
    };
    // Task 3 resolves the routes here.
    unresolved(WantDiagnostic::NothingUnlocks)
}

/// The four kinds the graph never grants. Written as a `match` with no `_` arm so that a new
/// `Target` variant breaks this build rather than falling silently into "not unlockable".
fn unlockable(t: &Target) -> bool {
    match t {
        Target::Item { .. }
        | Target::Trinket { .. }
        | Target::Character { .. }
        | Target::Achievement { .. }
        | Target::Challenge { .. }
        | Target::Entity { .. } => true,
        Target::Transformation { .. }
        | Target::Stage { .. }
        | Target::Room { .. }
        | Target::Pickup { .. } => false,
    }
}
```

In `crates/ipc/src/lib.rs`: add `mod want;` beside the other modules and
`pub use want::{want_view, WantDiagnostic, WantRoute, WantState, WantView, WantedView};`.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p ipc --test want`
Expected: PASS (both).

- [ ] **Step 5: Commit**

```bash
git add crates/ipc/src/want.rs crates/ipc/src/lib.rs crates/ipc/tests/want.rs crates/ipc/tests/support/mod.rs
git commit -m "feat(ipc): the want view, and the two answers that have no route"
```

---

## Task 3: From a name to the achievements that grant it

**Files:**
- Modify: `crates/ipc/src/want.rs`
- Test: `crates/ipc/tests/want.rs`

**Interfaces:**
- Produces (private): `fn key_of(c: &Catalog, t: &Target) -> Option<crate::goals::TargetKey>`
- Consumes: `crate::queue::achievements_unlocking` (Task 1), `crate::wiki_target::boss`.
- After this task `want_view` fills `wanted` and `routes`; every route's state is still `WantState::NoProfile` — Task 4 computes them.

**Note for the implementer:** the node for an achievement id is `view.nodes.iter().find(|n| matches!(n.achievement, AchievementRef::Known { id, .. } if id == want))`. `UnlockView.nodes` is one node per save slot, so a catalog achievement with no slot in this save has no node, and that route simply is not there — the same rule the rest of the app uses.

- [ ] **Step 1: Write the failing tests** — append to `crates/ipc/tests/want.rs`

```rust
use ipc::{AchievementRef, UnlockTarget, WantState};

/// A view with one node per achievement 1..=3, none done, all computable.
fn view_of(c: &catalog::Catalog) -> ipc::UnlockView {
    let flags = [false, false, false, false];
    ipc::unlock_view(Some(c), None, Some(&flags), None, None, None, |_| None)
}

#[test]
fn an_item_names_the_achievement_that_grants_it() {
    let c = catalog_with_achievements();
    let v = ipc::want_view(Some(&c), &view_of(&c), None, None, &Target::Item { id: 2 }, |_| {
        None
    });
    assert!(matches!(
        &v.wanted,
        WantedView::Target { target: UnlockTarget::Item { id: 2, .. } }
    ));
    assert_eq!(v.routes.len(), 1);
    assert!(matches!(
        &v.routes[0].node.achievement,
        AchievementRef::Known { id: 1, .. }
    ));
    assert!(v.diagnostics.is_empty());
}

#[test]
fn a_challenge_named_by_two_achievements_shows_two_routes() {
    let c = catalog_with_achievements();
    let v = ipc::want_view(
        Some(&c),
        &view_of(&c),
        None,
        None,
        &Target::Challenge { number: 4 },
        |_| None,
    );
    let ids: Vec<u32> = v
        .routes
        .iter()
        .filter_map(|r| match r.node.achievement {
            AchievementRef::Known { id, .. } => Some(id),
            AchievementRef::Unknown { .. } => None,
        })
        .collect();
    assert_eq!(ids, vec![1, 2], "both ways in, neither hidden");
}

#[test]
fn an_achievement_named_directly_is_its_own_route() {
    // *Greedier!* is the case: the catalog models no target for it, so the only way to
    // reach that node is to name the achievement.
    let c = catalog_with_achievements();
    let v = ipc::want_view(
        Some(&c),
        &view_of(&c),
        None,
        None,
        &Target::Achievement { id: 2 },
        |_| None,
    );
    assert!(matches!(
        &v.wanted,
        WantedView::Achievement {
            achievement: AchievementRef::Known { id: 2, .. }
        }
    ));
    assert_eq!(v.routes.len(), 1);
    assert!(v.diagnostics.is_empty());
}

#[test]
fn a_thing_no_achievement_grants_says_which_empty_it_is() {
    let c = catalog_with_achievements();
    // Character 7 is granted by achievement 2; character 9 is in no file at all.
    let v = ipc::want_view(
        Some(&c),
        &view_of(&c),
        None,
        None,
        &Target::Character { id: 9 },
        |_| None,
    );
    assert_eq!(v.wanted, WantedView::Unresolved);
    assert!(v.routes.is_empty());
    assert_eq!(v.diagnostics, vec![WantDiagnostic::NothingUnlocks]);
}
```

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p ipc --test want`
Expected: FAIL — `want_view` still answers `NothingUnlocks` for everything.

- [ ] **Step 3: Write the implementation** — in `crates/ipc/src/want.rs`, replace the body of `want_view` after the `unlockable` guard and add `key_of`

```rust
pub fn want_view(
    catalog: Option<&Catalog>,
    view: &UnlockView,
    _flags: Option<&[bool]>,
    _eval: Option<&graph::evaluate::Eval>,
    target: &Target,
    mut icon: impl FnMut(&IconRef) -> Option<String>,
) -> WantView {
    let unresolved = |d: WantDiagnostic| WantView {
        wanted: WantedView::Unresolved,
        routes: Vec::new(),
        diagnostics: vec![d],
    };
    if !unlockable(target) {
        return unresolved(WantDiagnostic::NotUnlockable);
    }
    let Some(c) = catalog else {
        return unresolved(WantDiagnostic::NoCatalog);
    };
    // Naming an achievement reaches the node directly: it is the only way to ask for what
    // the catalog models no target for (a mode, an event) — B37's Greed Mode case.
    let (wanted, ids) = match target {
        Target::Achievement { id } => match node_of(view, *id) {
            Some(n) => (
                WantedView::Achievement {
                    achievement: n.achievement.clone(),
                },
                vec![*id],
            ),
            None => return unresolved(WantDiagnostic::NothingUnlocks),
        },
        _ => {
            let Some(key) = key_of(c, target) else {
                return unresolved(WantDiagnostic::NothingUnlocks);
            };
            let ids = crate::queue::achievements_unlocking(c, &key);
            match crate::graph::resolve_target(c, &key, None, &mut icon) {
                Some(t) if !ids.is_empty() => (WantedView::Target { target: t }, ids),
                _ => return unresolved(WantDiagnostic::NothingUnlocks),
            }
        }
    };
    let routes = ids
        .iter()
        .filter_map(|id| node_of(view, *id))
        .map(|node| WantRoute {
            node: node.clone(),
            state: WantState::NoProfile,
        })
        .collect::<Vec<_>>();
    if routes.is_empty() {
        return unresolved(WantDiagnostic::NothingUnlocks);
    }
    WantView {
        wanted,
        routes,
        diagnostics: Vec::new(),
    }
}

fn node_of(view: &UnlockView, achievement: u32) -> Option<&UnlockNode> {
    view.nodes.iter().find(
        |n| matches!(n.achievement, AchievementRef::Known { id, .. } if id == achievement),
    )
}

/// The name you typed, as the key the catalog indexes unlocks by. The conversion lives here
/// and not on the frontend: an item's kind and a boss's entity triple are things only the
/// catalog knows, and a key assembled from a page identity would be a second mapping.
fn key_of(c: &Catalog, t: &Target) -> Option<crate::goals::TargetKey> {
    use crate::catalog_view::{kind_view, ItemKindView};
    use crate::goals::TargetKey;
    use catalog::{CharacterId, ChallengeId, ItemId, ItemKind};
    match t {
        Target::Item { id } => [ItemKind::Passive, ItemKind::Active, ItemKind::Familiar]
            .into_iter()
            .find_map(|k| c.item(k, ItemId(*id)))
            .map(|i| TargetKey::Item {
                item_kind: kind_view(i.kind),
                id: i.id.0,
            }),
        Target::Trinket { id } => c.item(ItemKind::Trinket, ItemId(*id)).map(|i| TargetKey::Item {
            item_kind: ItemKindView::Trinket,
            id: i.id.0,
        }),
        Target::Character { id } => c
            .character(CharacterId(*id))
            .map(|ch| TargetKey::Character { id: ch.id.0 }),
        Target::Challenge { number } => c
            .challenge(ChallengeId(*number))
            .map(|ch| TargetKey::Challenge { id: ch.id.0 }),
        // The boss is found by the same portrait-derived key `wiki_target` writes, never by
        // a name match: one mapping, read in both directions.
        Target::Entity { .. } => c
            .bosses()
            .find(|b| crate::wiki_target::boss(b).as_ref() == Some(t))
            .map(|b| TargetKey::Boss { id: b.id.0 }),
        Target::Achievement { .. }
        | Target::Transformation { .. }
        | Target::Stage { .. }
        | Target::Room { .. }
        | Target::Pickup { .. } => None,
    }
}
```

`resolve_target` and `wiki_target::boss` are `pub(crate)`/`pub` inside the crate already; if
`resolve_target`'s visibility is narrower than `pub(crate)`, widen it to `pub(crate)` in the
same commit rather than duplicating it.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p ipc --test want`
Expected: PASS (all six).

- [ ] **Step 5: Commit**

```bash
git add crates/ipc/src/want.rs crates/ipc/src/graph.rs crates/ipc/tests/want.rs
git commit -m "feat(ipc): a want names the achievements that grant it, all of them"
```

---

## Task 4: Three of the four states, from their own causes

**Files:**
- Modify: `crates/ipc/src/want.rs`
- Test: `crates/ipc/tests/want.rs`

**Interfaces:**
- **The signature does not change here** — Task 2 already wrote `flags` and `eval` in. What changes is that `want_view` stops ignoring `flags`: it drops the underscore and passes it on.
- Produces (private): `fn route_state(node: &UnlockNode, flags: Option<&[bool]>, eval: Option<&graph::evaluate::Eval>, view: &UnlockView) -> WantState`.

**Note:** the state is read from `node.done` and `node.graph`, never from the length of a chain — the four meanings of an empty `missing_chain` are exactly what this view exists to keep apart.

- [ ] **Step 1: Write the failing tests** — append to `crates/ipc/tests/want.rs`

```rust
use ipc::GraphInfo;

/// The same view, with the achievements in `done` marked and the graph's verdict forced.
fn view_with(c: &catalog::Catalog, done: &[u32], info: GraphInfo) -> ipc::UnlockView {
    let mut flags = [false; 4];
    for id in done {
        flags[*id as usize] = true;
    }
    let mut v = ipc::unlock_view(Some(c), None, Some(&flags), None, None, None, |_| None);
    for n in v.nodes.iter_mut() {
        n.graph = info.clone();
    }
    v
}

const COMPUTED_NOW: GraphInfo = GraphInfo::Computed {
    available_now: true,
    blocked_by: 0,
    fan_out: 0,
    steps_missing: 0,
};

#[test]
fn a_want_you_already_have_says_so() {
    let c = catalog_with_achievements();
    let v = view_with(&c, &[1], COMPUTED_NOW);
    let w = ipc::want_view(
        Some(&c),
        &v,
        Some(&[false; 4]),
        None,
        &Target::Item { id: 2 },
        |_| None,
    );
    assert_eq!(w.routes[0].state, WantState::Done);
}

#[test]
fn a_want_with_nothing_in_the_way_is_available_now() {
    let c = catalog_with_achievements();
    let v = view_with(&c, &[], COMPUTED_NOW);
    let w = ipc::want_view(
        Some(&c),
        &v,
        Some(&[false; 4]),
        None,
        &Target::Item { id: 2 },
        |_| None,
    );
    assert_eq!(w.routes[0].state, WantState::AvailableNow);
}

#[test]
fn without_the_section_the_view_names_the_route_and_claims_nothing() {
    let c = catalog_with_achievements();
    let v = view_with(&c, &[], COMPUTED_NOW);
    let w = ipc::want_view(Some(&c), &v, None, None, &Target::Item { id: 2 }, |_| None);
    assert_eq!(w.routes[0].state, WantState::NoProfile);
    assert_eq!(w.diagnostics, vec![WantDiagnostic::NoProfile]);
}

#[test]
fn the_no_profile_diagnostic_and_the_rows_cannot_disagree() {
    // The banner and the rows are two readings of one fact: the diagnostic is emitted if and
    // only if every route is NoProfile. A screen that trusted the banner while a row said
    // something else would draw a lie either way round.
    let c = catalog_with_achievements();
    for flags in [None, Some(&[false; 4][..])] {
        let v = view_with(&c, &[], COMPUTED_NOW);
        let w = ipc::want_view(Some(&c), &v, flags, None, &Target::Item { id: 2 }, |_| None);
        let all_rows = w.routes.iter().all(|r| r.state == WantState::NoProfile);
        let banner = w.diagnostics.contains(&WantDiagnostic::NoProfile);
        assert_eq!(all_rows, banner);
    }
}
```

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p ipc --test want`
Expected: FAIL — the signature has no `flags`, and every state is `NoProfile`.

- [ ] **Step 3: Write the implementation** — in `crates/ipc/src/want.rs`

Add the two parameters to `want_view`, replace the `state:` line with `state: route_state(node, flags, graph, view)`, push the diagnostic when every route is `NoProfile`, and add:

```rust
/// The state of one route, read from what the node already says. Never from the length of a
/// chain: `missing_chain` answers with an empty list for four different situations, and
/// telling them apart is this view's whole job.
fn route_state(
    node: &UnlockNode,
    flags: Option<&[bool]>,
    _eval: Option<&graph::evaluate::Eval>,
    _view: &UnlockView,
) -> WantState {
    if flags.is_none() {
        return WantState::NoProfile;
    }
    if node.done {
        return WantState::Done;
    }
    match node.graph {
        crate::graph::GraphInfo::Computed {
            available_now: true,
            ..
        } => WantState::AvailableNow,
        // Task 5 replaces this with the chain.
        _ => WantState::NoProfile,
    }
}
```

The `_ =>` arm here is temporary and Task 5 deletes it; leave the `// Task 5` comment on it so
a reviewer sees it is scaffolding, not a closed decision.

Then, in `want_view`, after `routes` is built:

```rust
    let diagnostics = routes
        .iter()
        .all(|r| r.state == WantState::NoProfile)
        .then_some(WantDiagnostic::NoProfile)
        .into_iter()
        .collect();
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p ipc --test want`
Expected: PASS (all ten).

- [ ] **Step 5: Commit**

```bash
git add crates/ipc/src/want.rs crates/ipc/tests/want.rs
git commit -m "feat(ipc): a route says done, available now, or nothing claimed"
```

---

## Task 5: The chain, in the order the Plan would play it

**Files:**
- Modify: `crates/ipc/src/want.rs`
- Test: `crates/ipc/tests/want.rs`

**Interfaces:**
- `route_state` stops ignoring `eval`. `missing_chain` lives on `graph::evaluate::Eval`, which the app already has at hand in `commands/graph.rs`; no signature changes here either.
- Consumes: `crate::queue::GraphDeps`, `plan::Queue`.

**The rule:** `missing_chain` returns a set in ascending id order, which is not a sequence to
play. The order that means something is the one `Queue::enqueue` produces — chain, wish, then
the repair that pulls prerequisites above it. Building a throwaway empty queue makes the
preview and the write (`queue_add`) one computation instead of two rules to keep aligned.

- [ ] **Step 1: Write the failing tests** — append to `crates/ipc/tests/want.rs`

```rust
/// 3 needs 2, 2 needs 1. Ids ascend here, so a test that only checked membership would pass
/// on `missing_chain`'s own order; the point is that the order comes from the dependencies.
fn chained_graph() -> graph::Graph {
    graph::for_tests::from_edges(&[(1, &[]), (2, &[1]), (3, &[2])], &[])
}

#[test]
fn a_chain_is_ordered_the_way_the_queue_orders_it() {
    let c = catalog_with_achievements();
    let g = chained_graph();
    let flags = [false, false, false, false];
    let eval = g.evaluate(&graph::FlagsOnly(Some(&flags)));
    let v = view_with(&c, &[], GraphInfo::Computed {
        available_now: false,
        blocked_by: 1,
        fan_out: 0,
        steps_missing: 2,
    });
    // Trinket 1 is granted by achievement 3, the deepest node.
    let w = ipc::want_view(
        Some(&c),
        &v,
        Some(&flags),
        Some(&eval),
        &Target::Trinket { id: 1 },
        |_| None,
    );
    let WantState::Chain { steps, unknown } = &w.routes[0].state else {
        panic!("expected a chain, got {:?}", w.routes[0].state);
    };
    let ids: Vec<u32> = steps
        .iter()
        .filter_map(|n| match n.achievement {
            AchievementRef::Known { id, .. } => Some(id),
            AchievementRef::Unknown { .. } => None,
        })
        .collect();
    assert_eq!(ids, vec![1, 2], "prerequisites first, the want excluded");
    assert_eq!(*unknown, 0);

    // The same order the Plan produces: enqueueing the want into an empty queue.
    let mut q = plan::Queue::from_rows(vec![]);
    let chain = eval.missing_chain(3, &graph::FlagsOnly(Some(&flags)));
    let mut rows: Vec<u32> = chain.clone();
    rows.push(3);
    q.enqueue(3, &chain, &ipc::GraphDeps::new(&g, Some(&flags), &rows));
    let queued: Vec<u32> = q.rows().iter().map(|r| r.achievement).collect();
    assert_eq!(queued, vec![1, 2, 3], "the preview is the queue's own order");
}

#[test]
fn a_partly_read_chain_counts_what_it_could_not_interpret() {
    let c = catalog_with_achievements();
    let g = chained_graph();
    let flags = [false, false, false, false];
    let eval = g.evaluate(&graph::FlagsOnly(Some(&flags)));
    let mut v = view_with(&c, &[], GraphInfo::Computed {
        available_now: false,
        blocked_by: 1,
        fan_out: 0,
        steps_missing: 2,
    });
    // One step of the chain has requirements the graph only partly interprets.
    v.nodes[0].graph = GraphInfo::Partial {
        blocked_by: 1,
        fan_out: 0,
        unknown: 2,
    };
    let w = ipc::want_view(
        Some(&c),
        &v,
        Some(&flags),
        Some(&eval),
        &Target::Trinket { id: 1 },
        |_| None,
    );
    let WantState::Chain { unknown, .. } = &w.routes[0].state else {
        panic!("expected a chain");
    };
    assert_eq!(*unknown, 1, "one step the app cannot fully read");
}

#[test]
fn no_route_is_an_empty_chain_that_claims_nothing_is_missing() {
    // The failure this guards: reading an empty `missing_chain` as "nothing in the way".
    // Every combination of done/available/partial must land on a state that says which.
    let c = catalog_with_achievements();
    let g = chained_graph();
    let flags = [false, false, false, false];
    let eval = g.evaluate(&graph::FlagsOnly(Some(&flags)));
    for info in [
        COMPUTED_NOW,
        GraphInfo::Computed {
            available_now: false,
            blocked_by: 1,
            fan_out: 0,
            steps_missing: 1,
        },
        GraphInfo::Partial {
            blocked_by: 0,
            fan_out: 0,
            unknown: 1,
        },
    ] {
        for done in [&[][..], &[1, 2, 3][..]] {
            let v = view_with(&c, done, info.clone());
            let w = ipc::want_view(
                Some(&c),
                &v,
                Some(&flags),
                Some(&eval),
                &Target::Trinket { id: 1 },
                |_| None,
            );
            if let WantState::Chain { steps, unknown } = &w.routes[0].state {
                assert!(
                    !steps.is_empty() || *unknown > 0,
                    "an empty chain with nothing unknown is `availableNow`, not a chain",
                );
            }
        }
    }
}
```

- [ ] **Step 2: Run them and watch them fail**

Run: `cargo test -p ipc --test want`
Expected: FAIL — no `Chain` is ever produced.

- [ ] **Step 3: Write the implementation** — replace `route_state` in `crates/ipc/src/want.rs`

```rust
fn route_state(
    node: &UnlockNode,
    flags: Option<&[bool]>,
    eval: Option<&graph::evaluate::Eval>,
    view: &UnlockView,
) -> WantState {
    let (Some(flags), Some(eval)) = (flags, eval) else {
        return WantState::NoProfile;
    };
    if node.done {
        return WantState::Done;
    }
    let AchievementRef::Known { id, .. } = node.achievement else {
        return WantState::NoProfile;
    };
    let chain = eval.missing_chain(id, &graph::FlagsOnly(Some(flags)));
    // The order is the queue's, asked rather than reinvented: `enqueue` appends the chain,
    // then the wish, then runs the repair that pulls prerequisites above it.
    let mut rows = chain.clone();
    rows.push(id);
    let mut queue = plan::Queue::from_rows(Vec::new());
    queue.enqueue(id, &chain, &crate::queue::GraphDeps::from_chains(
        rows.iter()
            .map(|a| (*a, eval.missing_chain(*a, &graph::FlagsOnly(Some(flags))))),
    ));
    let steps: Vec<UnlockNode> = queue
        .rows()
        .iter()
        .filter(|r| r.achievement != id)
        .filter_map(|r| node_of(view, r.achievement))
        .cloned()
        .collect();
    let unknown = steps
        .iter()
        .chain(std::iter::once(node))
        .filter(|n| matches!(n.graph, crate::graph::GraphInfo::Partial { .. }))
        .count() as u32;
    // An empty chain with nothing unreadable is not a chain: it is the node being playable
    // right now, which `available_now` already states.
    if steps.is_empty() && unknown == 0 {
        return WantState::AvailableNow;
    }
    WantState::Chain { steps, unknown }
}
```

Note: `GraphDeps::new` would re-walk the graph per row; `from_chains` takes the chains we
already have. Both are in `crate::queue` and both are already tested.

- [ ] **Step 4: Run the whole file**

Run: `cargo test -p ipc --test want`
Expected: PASS (all thirteen).

- [ ] **Step 5: Commit**

```bash
git add crates/ipc/src/want.rs crates/ipc/tests/want.rs
git commit -m "feat(ipc): the chain a want needs, in the order the Plan would play it"
```

---

## Task 6: The wire's shape, pinned

**Files:**
- Test: `crates/ipc/tests/want.rs`

**Why a task of its own:** `core_save::Kind` once crossed the boundary inside `SectionCount`
and a rename changed the wire with the suite green (`crates/ipc/tests/summary_shape.rs`). A
field that comes out `snake_case`, or a struct variant whose fields were not renamed, is read
as `undefined` in TypeScript with no error at all.

- [ ] **Step 1: Write the failing test** — append to `crates/ipc/tests/want.rs`

```rust
use serde_json::{json, to_value};

#[test]
fn want_view_json_shape_is_pinned() {
    let c = catalog_with_achievements();
    let g = chained_graph();
    let flags = [false, false, false, false];
    let eval = g.evaluate(&graph::FlagsOnly(Some(&flags)));
    let v = view_with(&c, &[], GraphInfo::Computed {
        available_now: false,
        blocked_by: 1,
        fan_out: 0,
        steps_missing: 2,
    });
    let w = to_value(ipc::want_view(
        Some(&c),
        &v,
        Some(&flags),
        Some(&eval),
        &Target::Trinket { id: 1 },
        |_| None,
    ))
    .unwrap();
    assert_eq!(w["wanted"]["kind"], "target");
    assert_eq!(w["wanted"]["target"]["kind"], "item");
    assert_eq!(w["wanted"]["target"]["itemKind"], "trinket");
    assert_eq!(w["routes"][0]["state"]["kind"], "chain");
    assert_eq!(w["routes"][0]["state"]["unknown"], 0);
    assert!(w["routes"][0]["state"]["steps"].is_array());
    assert!(w["routes"][0]["node"]["achievement"].is_object());

    // The three shapes that have no route, each naming which empty it is.
    let none = to_value(ipc::want_view(
        None,
        &v,
        Some(&flags),
        Some(&eval),
        &Target::Item { id: 2 },
        |_| None,
    ))
    .unwrap();
    assert_eq!(none["wanted"], json!({ "kind": "unresolved" }));
    assert_eq!(none["diagnostics"], json!([{ "kind": "noCatalog" }]));
}
```

- [ ] **Step 2: Run it**

Run: `cargo test -p ipc --test want want_view_json_shape_is_pinned`
Expected: PASS if Tasks 2–5 got the serde attributes right; a FAIL here is a real wire bug,
not an expectation to adjust.

- [ ] **Step 3: Commit**

```bash
git add crates/ipc/tests/want.rs
git commit -m "test(ipc): the want view's wire shape is pinned"
```

---

## Task 7: The command

**Files:**
- Modify: `crates/app/src/commands/graph.rs`
- Modify: `crates/app/src/lib.rs:44-66`

**Interfaces:**
- Produces: the Tauri command `want(target: wiki::Target) -> Result<ipc::WantView, IpcError>`.
- The Tauri crate is wiring and is not tested (`CLAUDE.md`): everything worth checking is in
  Tasks 1–6.

- [ ] **Step 1: Write the command** — append to `crates/app/src/commands/graph.rs`

```rust
/// "I want this — what do I have to play?". The same state `unlock` reads, asked from the
/// other end: no profile and no game are expected answers and travel in the payload.
#[tauri::command]
pub fn want(
    app: AppHandle,
    target: wiki::Target,
    state: tauri::State<'_, CatalogState>,
    resources: tauri::State<'_, ResourcesState>,
    graph: tauri::State<'_, GraphState>,
) -> Result<ipc::WantView, IpcError> {
    let (flags, counters) = progress_sections(&app)?;
    let resources = resources.get();
    let catalog = resources.and_then(|rs| state.get_or_build(rs));
    let g = catalog.and_then(|c| graph.get(c));
    let progress = ipc::SaveProgress::new(flags.as_deref(), counters.as_deref(), catalog);
    let eval = g.map(|g| g.evaluate(&progress));
    let view = ipc::unlock_view(
        catalog,
        wiki::Dataset::embedded().ok(),
        flags.as_deref(),
        g,
        eval.as_ref(),
        Some(&progress),
        icon_url,
    );
    Ok(ipc::want_view(
        catalog,
        &view,
        flags.as_deref(),
        eval.as_ref(),
        &target,
        icon_url,
    ))
}
```

- [ ] **Step 2: Register it** — in `crates/app/src/lib.rs`, add `graph::want,` after
`graph::collection,` in `generate_handler!`.

- [ ] **Step 3: Build**

Run: `cargo clippy -p app --all-targets -- -D warnings`
Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add crates/app/src/commands/graph.rs crates/app/src/lib.rs
git commit -m "feat(app): the want command"
```

---

## Task 8: What the real catalog actually says

**Files:**
- Create: `crates/ipc/tests/want_real.rs`

**Interfaces:**
- Consumes: `test_support` for the catalog and a dated save, the way `crates/ipc/tests/graph_real.rs` already does — copy its setup verbatim rather than opening `samples/` by hand.

**Every assertion here carries a vacuity guard.** A property about chains holds trivially on a
profile with no chains, and "0 passed, 12 skipped" reads as green.

- [ ] **Step 1: Write the tests** — create `crates/ipc/tests/want_real.rs`

```rust
//! B37 on the real catalog. Three facts, none of which a synthetic catalog can give:
//! that a deep want exists at all, that the order agrees with the queue's, and how many
//! targets really have more than one way in.

// The setup block is `graph_real.rs`'s: catalog, dataset, graph, a dated save's flags.
// Copy it from there, including the `skip:` note when a sample is missing.

#[test]
fn the_reference_profile_has_a_want_with_a_real_chain() {
    let Some((c, eval, flags, view)) = setup() else {
        return; // the helper already printed `skip: …`
    };
    let deep = view
        .nodes
        .iter()
        .filter_map(|n| match n.achievement {
            ipc::AchievementRef::Known { id, .. } => Some(id),
            ipc::AchievementRef::Unknown { .. } => None,
        })
        .map(|id| (id, eval.missing_chain(id, &graph::FlagsOnly(Some(&flags))).len()))
        .max_by_key(|(_, len)| *len);
    let (_, longest) = deep.expect("the catalog has achievements");
    assert!(
        longest >= 3,
        "vacuity guard: with no chain longer than two, the ordering property below proves \
         nothing. Longest chain found: {longest}",
    );
}

#[test]
fn a_wants_chain_is_what_the_queue_would_hold() {
    let Some((c, eval, flags, view)) = setup() else {
        return;
    };
    // The want: the target of the node with the longest chain, so the order is not trivial.
    let (id, chain) = longest_chain(&view, &eval, &flags);
    let Some(target) = first_target_of(&c, &view, id) else {
        eprintln!("skip: the deepest node unlocks nothing the catalog names");
        return;
    };
    let w = ipc::want_view(Some(&c), &view, Some(&flags), Some(&eval), &target, |_| None);
    let ipc::WantState::Chain { steps, .. } = &w.routes[0].state else {
        panic!("expected a chain for a node {} steps deep", chain.len());
    };
    let preview: Vec<u32> = steps
        .iter()
        .filter_map(|n| match n.achievement {
            ipc::AchievementRef::Known { id, .. } => Some(id),
            ipc::AchievementRef::Unknown { .. } => None,
        })
        .collect();
    let mut q = plan::Queue::from_rows(vec![]);
    let mut rows = chain.clone();
    rows.push(id);
    q.enqueue(
        id,
        &chain,
        &ipc::GraphDeps::from_chains(
            rows.iter()
                .map(|a| (*a, eval.missing_chain(*a, &graph::FlagsOnly(Some(&flags))))),
        ),
    );
    let queued: Vec<u32> = q
        .rows()
        .iter()
        .map(|r| r.achievement)
        .filter(|a| *a != id)
        .collect();
    assert_eq!(preview, queued);
}

#[test]
fn how_many_targets_have_more_than_one_way_in() {
    let Some((c, _, _, _)) = setup() else {
        return;
    };
    // A measurement, not a threshold: the number goes in the report. `routes` is a list
    // because a challenge's `unlocked_by` is one; if the real file never uses two, that is
    // worth knowing and worth writing down, not worth deleting the list for.
    let mut several = 0usize;
    for ch in c.challenges() {
        if ipc::achievements_unlocking(&c, &ipc::TargetKey::Challenge { id: ch.id.0 }).len() > 1 {
            several += 1;
        }
    }
    eprintln!("measured: {several} challenges are named by more than one achievement");
    assert!(
        c.challenges().count() > 0,
        "vacuity guard: no challenges read, the count above says nothing",
    );
}
```

The three helpers (`setup`, `longest_chain`, `first_target_of`) are local to this file; write
them as small private functions over what `graph_real.rs` already builds.

- [ ] **Step 2: Run it, and read the skips**

Run: `cargo test -p ipc --test want_real -- --nocapture`
Expected: PASS, with `sample: …` and `measured: …` lines visible. A silent "3 passed" here
means the samples are missing and nothing was actually checked.

- [ ] **Step 3: Commit**

```bash
git add crates/ipc/tests/want_real.rs
git commit -m "test(ipc): the want view against the real catalog, guards included"
```

---

## Task 9: The contract, mirrored and handed on

**Files:**
- Modify: `ui/src/lib/ipc/types.ts`
- Modify: `ui/src/lib/ipc/graph.ts`
- Modify: `ui/src/lib/constants/commands.ts`
- Modify: `ui/src/lib/ipc/fixtures/graph.ts`
- Modify: `DESIGN-BRIEF.md`

**Interfaces:**
- Produces: `WantView`, `WantedView`, `WantRoute`, `WantState`, `WantDiagnostic` in TypeScript;
  `want(target: Target): Promise<WantView>`.

- [ ] **Step 1: Mirror the types** — in `ui/src/lib/ipc/types.ts`, beside the other graph types

```ts
// B37: a want, and the ways in. `routes` is a list because a challenge can be named by two
// achievements; an empty list always travels with the diagnostic that says which empty it is.
export interface WantRoute {
  node: UnlockNode
  state: WantState
}

export type WantState =
  | { kind: 'done' }
  | { kind: 'availableNow' }
  | { kind: 'chain'; steps: UnlockNode[]; unknown: number }
  | { kind: 'noProfile' }

export type WantedView =
  | { kind: 'target'; target: UnlockTarget }
  | { kind: 'achievement'; achievement: AchievementRef }
  | { kind: 'unresolved' }

export type WantDiagnostic =
  | { kind: 'noCatalog' }
  | { kind: 'noProfile' }
  | { kind: 'nothingUnlocks' }
  | { kind: 'notUnlockable' }

export interface WantView {
  wanted: WantedView
  routes: WantRoute[]
  diagnostics: WantDiagnostic[]
}
```

The tag is the one exception to the no-string-unions rule (`CLAUDE.md`), so these stay as
written; nothing else here is a bare union.

- [ ] **Step 2: The wrapper and the command name**

In `ui/src/lib/constants/commands.ts` add `Want: 'want',` after `NextSteps`.
In `ui/src/lib/ipc/graph.ts`:

```ts
// `target` is the name of the Rust command's parameter. The page identity and the want are
// the same `Target`: one vocabulary, two questions.
export const want = (target: Target): Promise<WantView> =>
  call(Command.Want, { target })
```

- [ ] **Step 3: The fixture** — in `ui/src/lib/ipc/fixtures/graph.ts`, export a `wantFixture`
returning a `WantView` with one route in the `chain` state and two steps, built from the nodes
the file already fabricates. `pnpm ui:dev` runs without a backend and the new screen has to
draw there.

- [ ] **Step 4: Hand it on** — add the five types to `DESIGN-BRIEF.md` beside the other graph
view-models, with the one-line rule for each state.

- [ ] **Step 5: Check**

Run: `pnpm typecheck && pnpm scan`
Expected: clean.

- [ ] **Step 6: Commit**

```bash
git add ui/src/lib/ipc/types.ts ui/src/lib/ipc/graph.ts ui/src/lib/constants/commands.ts ui/src/lib/ipc/fixtures/graph.ts DESIGN-BRIEF.md
git commit -m "feat(ui): the want view crosses the wire"
```

---

## Task 10: The want is in the URL

**Files:**
- Create: `ui/src/lib/graph/wantLocation.ts`, `ui/src/lib/graph/wantLocation.test.ts`
- Modify: `ui/src/router/routeTable.ts:55-58`

**Interfaces:**
- Produces: `wantLocation(target: Target): TabLocation | null` — `{ name: RouteName.Goals, query: { want: pageKey(target) } }`, `null` when the target has no key.
- Produces: `wantOf(query: TabLocation['query']): Target | null` — `parsePageKey` on `want`.
- Consumes: `pageKey`, `parsePageKey` from `@/lib/wiki/pageKey`.

- [ ] **Step 1: Write the failing test** — create `ui/src/lib/graph/wantLocation.test.ts`

```ts
import { describe, expect, it } from 'vitest'
import { RouteName } from '@/router/routeTable'
import { wantLocation, wantOf } from './wantLocation'

describe('wantLocation', () => {
  it('names the Goals screen and carries the page key', () => {
    expect(wantLocation({ kind: 'character', id: 41 })).toEqual({
      name: RouteName.Goals,
      query: { want: 'character:41' },
    })
  })

  // The four kinds with no page key are the four that are not things you unlock: a link to
  // them would be a question the app cannot ask.
  it('has no location for a thing you do not unlock', () => {
    expect(wantLocation({ kind: 'stage', name: 'Basement' })).toBeNull()
  })

  it('reads the want back out of a location', () => {
    expect(wantOf({ want: 'item:105' })).toEqual({ kind: 'item', id: 105 })
  })

  // A hand-typed or stale URL is not an error: the screen shows the recommendations.
  it('reads nothing from a key it never wrote', () => {
    expect(wantOf({ want: 'mode:greed' })).toBeNull()
    expect(wantOf({})).toBeNull()
  })
})
```

- [ ] **Step 2: Run it and watch it fail**

Run: `pnpm ui:test wantLocation`
Expected: FAIL — the module does not exist.

- [ ] **Step 3: Write the implementation** — create `ui/src/lib/graph/wantLocation.ts`

```ts
import type { Target } from '@/lib/ipc/types'
import { pageKey, parsePageKey } from '@/lib/wiki/pageKey'
import { RouteName } from '@/router/routeTable'
import type { TabLocation } from '@/router/routeTable'

// A want travels as the same key a page does (`item:105`): one codec, already tested, and it
// answers `null` for exactly the four kinds nothing unlocks.
export const wantLocation = (target: Target): TabLocation | null => {
  const want = pageKey(target)
  return want === null ? null : { name: RouteName.Goals, query: { want } }
}

export const wantOf = (query: TabLocation['query']): Target | null =>
  query?.want === undefined ? null : parsePageKey(query.want)
```

- [ ] **Step 4: Widen the location type** — in `ui/src/router/routeTable.ts`, add `want?: string`
to `TabLocation['query']`, with a comment saying it is the same key shape as `page`.

- [ ] **Step 5: Run the tests**

Run: `pnpm ui:test wantLocation && pnpm typecheck`
Expected: PASS, clean.

- [ ] **Step 6: Commit**

```bash
git add ui/src/lib/graph/wantLocation.ts ui/src/lib/graph/wantLocation.test.ts ui/src/router/routeTable.ts
git commit -m "feat(ui): a want is a place, so back and forward can reach it"
```

---

## Task 11: What a block draws, as a pure function

**Files:**
- Create: `ui/src/lib/graph/wantBlocks.ts`, `ui/src/lib/graph/wantBlocks.test.ts`

**Interfaces:**
- Produces: `WantBlockKind` (`const … as const`) and
  `wantBlocks(view: WantView, queued: Set<number>): WantBlock[]` where
  `WantBlock = { kind: WantBlockKind; achievement: number; node: UnlockNode; steps: UnlockNode[]; unknown: number; queueable: boolean }`.
- Produces: `wantBanner(view: WantView): WantDiagnostic | null` — the one line above the blocks.
- Consumes: `queuedIds` from `@/lib/plan/queueRows` (the caller passes the set).

- [ ] **Step 1: Write the failing test** — create `ui/src/lib/graph/wantBlocks.test.ts`

```ts
import { describe, expect, it } from 'vitest'
import type { UnlockNode, WantView } from '@/lib/ipc/types'
import { WantBlockKind, wantBanner, wantBlocks } from './wantBlocks'

const node = (id: number): UnlockNode => ({
  achievement: { kind: 'known', id, text: `t${id}`, condition: null, iconUrl: null },
  done: false,
  unlocks: [],
  origin: null,
  missing: [],
  graph: { kind: 'computed', availableNow: false, blockedBy: 1, fanOut: 0, stepsMissing: 1 },
})

const view = (state: WantView['routes'][number]['state']): WantView => ({
  wanted: { kind: 'achievement', achievement: node(9).achievement },
  routes: [{ node: node(9), state }],
  diagnostics: [],
})

describe('wantBlocks', () => {
  it('a chain draws its steps and offers the queue', () => {
    const b = wantBlocks(
      view({ kind: 'chain', steps: [node(1), node(2)], unknown: 0 }),
      new Set(),
    )
    expect(b).toHaveLength(1)
    expect(b[0]?.kind).toBe(WantBlockKind.Chain)
    expect(b[0]?.steps.map((s) => s.achievement)).toHaveLength(2)
    expect(b[0]?.queueable).toBe(true)
  })

  it('a want already queued is not offered again', () => {
    const b = wantBlocks(
      view({ kind: 'chain', steps: [node(1)], unknown: 0 }),
      new Set([9]),
    )
    expect(b[0]?.queueable).toBe(false)
  })

  // Done and availableNow are answers, not empty chains: neither draws a step list.
  it('a want you have draws no steps and no button', () => {
    const b = wantBlocks(view({ kind: 'done' }), new Set())
    expect(b[0]?.kind).toBe(WantBlockKind.Done)
    expect(b[0]?.steps).toEqual([])
    expect(b[0]?.queueable).toBe(false)
  })

  it('without a profile the route is named and nothing is claimed', () => {
    const b = wantBlocks(view({ kind: 'noProfile' }), new Set())
    expect(b[0]?.kind).toBe(WantBlockKind.NoProfile)
    expect(b[0]?.queueable).toBe(false)
  })

  it('the banner is the first diagnostic, or nothing', () => {
    expect(wantBanner(view({ kind: 'done' }))).toBeNull()
    expect(
      wantBanner({ ...view({ kind: 'done' }), diagnostics: [{ kind: 'nothingUnlocks' }] }),
    ).toEqual({ kind: 'nothingUnlocks' })
  })
})
```

- [ ] **Step 2: Run it and watch it fail**

Run: `pnpm ui:test wantBlocks`
Expected: FAIL — the module does not exist.

- [ ] **Step 3: Write the implementation** — create `ui/src/lib/graph/wantBlocks.ts`

```ts
import { assertNever } from '@/lib/assertNever'
import type { UnlockNode, WantDiagnostic, WantView } from '@/lib/ipc/types'

// One block per way in. The kind is the state's, restated as what the block draws: the
// screen switches on this and never on `steps.length`.
export const WantBlockKind = {
  Chain: 'chain',
  Done: 'done',
  AvailableNow: 'availableNow',
  NoProfile: 'noProfile',
} as const
export type WantBlockKind = (typeof WantBlockKind)[keyof typeof WantBlockKind]

export interface WantBlock {
  kind: WantBlockKind
  achievement: number
  node: UnlockNode
  steps: UnlockNode[]
  unknown: number
  queueable: boolean
}

const idOf = (node: UnlockNode): number =>
  node.achievement.kind === 'known' ? node.achievement.id : -1

export const wantBlocks = (view: WantView, queued: Set<number>): WantBlock[] =>
  view.routes.map((route) => {
    const achievement = idOf(route.node)
    const base = { achievement, node: route.node, steps: [], unknown: 0, queueable: false }
    switch (route.state.kind) {
      case 'chain':
        return {
          ...base,
          kind: WantBlockKind.Chain,
          steps: route.state.steps,
          unknown: route.state.unknown,
          queueable: achievement >= 0 && !queued.has(achievement),
        }
      case 'availableNow':
        return {
          ...base,
          kind: WantBlockKind.AvailableNow,
          queueable: achievement >= 0 && !queued.has(achievement),
        }
      case 'done':
        return { ...base, kind: WantBlockKind.Done }
      case 'noProfile':
        return { ...base, kind: WantBlockKind.NoProfile }
      default:
        return assertNever(route.state)
    }
  })

// One line above the blocks, never assembled from the rows: the backend already decided.
export const wantBanner = (view: WantView): WantDiagnostic | null =>
  view.diagnostics[0] ?? null
```

- [ ] **Step 4: Run the tests**

Run: `pnpm ui:test wantBlocks && pnpm typecheck`
Expected: PASS, clean.

- [ ] **Step 5: Commit**

```bash
git add ui/src/lib/graph/wantBlocks.ts ui/src/lib/graph/wantBlocks.test.ts
git commit -m "feat(ui): what a want's block draws, as a pure function"
```

---

## Task 12: The bar, the answer, and the screen that hosts them

**Files:**
- Create: `ui/src/composables/useWant.ts`
- Create: `ui/src/screens/goals/WantBar.vue`, `ui/src/screens/goals/WantAnswer.vue`
- Create: `ui/src/kit/sections/app/WantAnswerSection.vue`
- Modify: `ui/src/screens/GoalsScreen.vue`
- Modify: `ui/src/i18n/messages/it.ts`, `ui/src/i18n/messages/en.ts`
- Modify: `ui/src/kit/KitPage.vue` (register the section the way the neighbouring ones are)

**Interfaces:**
- Consumes: `want` (Task 9), `wantOf`/`wantLocation` (Task 10), `wantBlocks`/`wantBanner`
  (Task 11), `search` (`@/lib/ipc/search`), `SearchRow`'s row component
  (`ui/src/components/search/SearchRow.vue`), `GoalCard.vue`, `useQueueStore`, `tracked`
  (`@/stores/tracked`).

**The composable, not a view store:** `defineViewStore` reads with no argument; a want is a
parameter. `useWant` keeps the same `view`/`status`/`error` triad through `tracked`, so the
screen's three states are drawn by the same components as everywhere else.

- [ ] **Step 1: Write the composable** — create `ui/src/composables/useWant.ts`

```ts
import { ref, watch } from 'vue'
import type { Ref } from 'vue'
import { want as readWant } from '@/lib/ipc/graph'
import type { IpcError, Target, WantView } from '@/lib/ipc/types'
import { LoadStatus } from '@/stores/loadStatus'
import { tracked } from '@/stores/tracked'

// A want is read by target, so it is a composable and not a view store: the store's `load()`
// takes no argument. The triad is the same one every screen draws, which is the point.
export const useWant = (target: Ref<Target | null>) => {
  const view = ref<WantView | null>(null) as Ref<WantView | null>
  const status = ref<LoadStatus>(LoadStatus.Idle)
  const error = ref<IpcError | null>(null)

  const load = (): Promise<void> => {
    const asked = target.value
    view.value = null
    if (asked === null) {
      status.value = LoadStatus.Idle
      return Promise.resolve()
    }
    return tracked(status, error, async () => {
      const answer = await readWant(asked)
      // The want may have changed while the command was in flight: a late answer to an old
      // question is worse than no answer.
      if (target.value === asked) view.value = answer
    })
  }

  watch(target, () => void load(), { immediate: true })
  return { view, status, error, load }
}
```

- [ ] **Step 2: Write the bar** — `ui/src/screens/goals/WantBar.vue`

The debounce and the command already live in `useSearch` (`ui/src/composables/useSearch.ts`),
which the palette uses: reuse it rather than typing a second debounce.

```vue
<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { Input } from '@/components/ui/input'
import SearchRowContent from '@/components/search/SearchRow.vue'
import { useSearch } from '@/composables/useSearch'
import { useMessages } from '@/i18n'
import { SearchLimit } from '@/lib/ipc/search'
import { wantLocation } from '@/lib/graph/wantLocation'
import { pageKey } from '@/lib/wiki/pageKey'
import { RouteName } from '@/router/routeTable'
import { useTabsStore } from '@/stores/tabs'

const tabs = useTabsStore()
const { t } = useMessages()
const { view, ask } = useSearch(SearchLimit.Palette)

const typed = ref('')
watch(typed, (query) => ask(query))

// Only what the app can actually be asked for: `pageKey` answers `null` for exactly the four
// kinds nothing unlocks, so the filter is the vocabulary, not a second list of kinds.
const hits = computed(() =>
  (view.value?.hits ?? []).filter((hit) => pageKey(hit.target) !== null),
)

const pick = (target: Parameters<typeof wantLocation>[0]) => {
  const location = wantLocation(target)
  if (location !== null) tabs.navigate(location)
  typed.value = ''
}

const clear = () => {
  typed.value = ''
  tabs.navigate({ name: RouteName.Goals })
}
</script>

<template>
  <div class="flex flex-col gap-2">
    <Input v-model="typed" :placeholder="t('want.placeholder')" />
    <ul v-if="hits.length > 0" class="flex flex-col gap-1">
      <li v-for="hit in hits" :key="hit.title">
        <Button
          :variant="ButtonVariant.Ghost"
          :size="ButtonSize.Sm"
          @click="pick(hit.target)"
        >
          <SearchRowContent :hit="hit" />
        </Button>
      </li>
    </ul>
    <Button
      v-if="typed !== ''"
      :variant="ButtonVariant.Ghost"
      :size="ButtonSize.Sm"
      @click="clear"
      >{{ t('want.clear') }}</Button
    >
  </div>
</template>
```

Import `Button`, `ButtonVariant` and `ButtonSize` from `@/components/ui/button` with the rest:
no raw `<button>` and no raw `<input>` anywhere, which `pnpm scan` checks in the pre-commit
hook.

- [ ] **Step 3: Write the answer** — `ui/src/screens/goals/WantAnswer.vue`

```vue
<script setup lang="ts">
import { Alert, AlertDescription } from '@/components/ui/alert'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { useMessages } from '@/i18n'
import { WantBlockKind } from '@/lib/graph/wantBlocks'
import type { WantBlock } from '@/lib/graph/wantBlocks'
import type { WantDiagnostic } from '@/lib/ipc/types'
import GoalCard from './GoalCard.vue'

defineProps<{
  blocks: WantBlock[]
  banner: WantDiagnostic | null
  /** Two ways in are drawn as two blocks with a heading that says so. */
  several: boolean
}>()
const emit = defineEmits<{ queue: [achievement: number] }>()
const { t } = useMessages()

// The heading per block kind. A record and not a chain of `v-if`: adding a kind to
// `WantBlockKind` without a heading then fails to compile.
const heading: Record<WantBlockKind, string> = {
  [WantBlockKind.Chain]: 'want.chain',
  [WantBlockKind.AvailableNow]: 'want.availableNow',
  [WantBlockKind.Done]: 'want.done',
  [WantBlockKind.NoProfile]: 'want.noProfile',
}
</script>

<template>
  <div class="flex flex-col gap-4">
    <Alert v-if="banner !== null">
      <AlertDescription>{{ t(`want.diagnostic.${banner.kind}`) }}</AlertDescription>
    </Alert>
    <section v-for="block in blocks" :key="block.achievement" class="flex flex-col gap-2">
      <h3>{{ t(heading[block.kind]) }}</h3>
      <p v-if="several">{{ t('want.several') }}</p>
      <!-- The steps in order, then the want itself: the position is the meaning, so the
           index is drawn and the list is never re-sorted. -->
      <GoalCard
        v-for="(step, index) in block.steps"
        :key="index"
        :node="step"
        :position="index + 1"
      />
      <GoalCard :node="block.node" :position="block.steps.length + 1" wanted />
      <p v-if="block.unknown > 0">{{ t('want.unknown', { count: block.unknown }) }}</p>
      <Button
        v-if="block.queueable"
        :variant="ButtonVariant.Secondary"
        :size="ButtonSize.Sm"
        @click="emit('queue', block.achievement)"
        >{{ t('want.addAll') }}</Button
      >
    </section>
  </div>
</template>
```

`GoalCard`'s props are 3.6's; `position` and `wanted` are the two this screen needs — add them
there with defaults, so the Goals sections keep drawing exactly as they do today. The i18n keys
are built with a template literal only for the diagnostic, whose four values are a closed enum:
if the scanner objects to the computed key, write the four out in a record like `heading`.

- [ ] **Step 4: Host them** — in `ui/src/screens/GoalsScreen.vue`

```ts
const route = useRoute()
const target = computed(() => wantOf(route.query as TabLocation['query']))
const asked = useWant(target)
const blocks = computed(() =>
  asked.view.value === null ? [] : wantBlocks(asked.view.value, queued.value),
)
```

The recommendations render `v-if="target === null"`, the answer `v-else`: one question at a
time, and clearing the bar brings the sections back.

- [ ] **Step 5: The strings** — add to both `it.ts` and `en.ts`, under a `want.` prefix: the
bar's placeholder, the four block headings, the `unknown` line (pluralised), the button, the
four diagnostics. Italian first (it is the reference), English mirrored.

- [ ] **Step 6: The Kit** — `WantAnswerSection.vue` draws the four block kinds from fixtures, so
the presentation is checked without a backend (`pnpm ui:dev`, `#kit`).

- [ ] **Step 7: Check everything**

Run: `pnpm check`
Expected: green. In particular `pnpm scan` reports 0 violations — a hardcoded size or an
untranslated string fails here, not in review.

- [ ] **Step 8: Commit**

```bash
git add ui/src/composables/useWant.ts ui/src/screens/goals/WantBar.vue ui/src/screens/goals/WantAnswer.vue ui/src/screens/GoalsScreen.vue ui/src/kit/sections/app/WantAnswerSection.vue ui/src/kit/KitPage.vue ui/src/i18n/messages/it.ts ui/src/i18n/messages/en.ts
git commit -m "feat(ui): name what you want, and the Goals screen answers with the chain"
```

---

## Task 13: The report, and the documents that track state

**Files:**
- Create: `docs/superpowers/reports/2026-09-13-goals-want-report.md`
- Modify: `docs/STATUS.md`, `docs/BACKLOG.md`

- [ ] **Step 1: Write the report** — what execution answered that the spec did not know. At
minimum: the number from Task 8's measurement (how many targets really have two ways in), any
decision the first look overturned, and whether `SearchRow.vue` was reusable in the bar.

- [ ] **Step 2: Close B37** — in `docs/BACKLOG.md`, mark the entry closed with the date and a
pointer to the report, in the form B13/B16/B24 use.

- [ ] **Step 3: Update `docs/STATUS.md`** — the session log entry and the sub-project's
checkbox. A checked box means committed work, never a note taken.

- [ ] **Step 4: Verify before claiming**

Run: `pnpm check` and `cargo test --workspace -- --nocapture | grep -c "skip:"`
Expected: green, and the skip count read rather than assumed.

- [ ] **Step 5: Commit**

```bash
git add docs/superpowers/reports/2026-09-13-goals-want-report.md docs/STATUS.md docs/BACKLOG.md
git commit -m "docs: B37 lands, and what the real catalog said about two ways in"
```

---

## Self-review notes

- **Spec coverage.** Decisions 1–10 map to tasks: 1 → 12, 2 → 3, 3 → 2/3/7, 4 → 2, 5 → 1, 6 → 5,
  7 → 4/5, 8 → 10, 9 → 12, 10 → 11/12. The spec's three risks are Task 8's three tests.
- **The one thing this plan changes about the spec.** `want_view` takes `flags` and `eval` as
  well as the catalog and the view; the spec's prose implied a smaller signature. The reason is
  in Task 4: the four meanings of an empty chain can only be told apart with the flags in hand.
- **Where a reviewer should push hardest.** Task 5: it is the only task that produces an order,
  and the property that keeps it honest compares against `plan::Queue` rather than against a
  literal list of ids.
