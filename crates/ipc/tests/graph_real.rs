//! Against the real profile `samples/live.rep+persistentgamedata1.dat` (2026-08-31)
//! and the real catalog via `samples/packed`. They skip with a note if something is
//! missing. The numbers are fixtures of known origin, measured on 2026-09-05.

use std::collections::BTreeSet;

use catalog::{Catalog, ItemKind};
use core_save::{Kind, Save};
use ipc::{
    next_steps, resolve_target, unlock_view, AchievementRef, ItemKindView, TargetKey,
    UnlockDiagnostic, UnlockTarget,
};
use unpack::ResourceSet;

fn real() -> Option<(Catalog, ResourceSet, Save)> {
    let packed = test_support::packed_dir()?;
    // Stays `live` and not a dated file: the numbers pinned below are measured
    // against this profile, and changing it would mean deriving them again from the
    // code's own output — exactly the wrong way to write an expected value.
    let save = test_support::sample("live.rep+persistentgamedata1.dat")?;
    let rs = ResourceSet::open(&packed);
    let c = Catalog::build(|p| rs.read(p));
    // A sample that's present but unreadable isn't a silent skip: it's declared.
    let s = match Save::open(&save) {
        Ok(s) => s,
        Err(_) => {
            test_support::skip("the live profile exists but doesn't read");
            return None;
        }
    };
    Some((c, rs, s))
}

/// One named profile, with the real catalog. `real()` above is this for the one sample whose
/// numbers are pinned; this is for a question that needs a *different* point of the
/// progression — a young profile still has thresholds it hasn't crossed.
fn profile(name: &str) -> Option<(Catalog, Save)> {
    let packed = test_support::packed_dir()?;
    let path = test_support::sample(name)?;
    let c = Catalog::build(|p| ResourceSet::open(&packed).read(p));
    match Save::open(&path) {
        Ok(s) => Some((c, s)),
        Err(_) => {
            test_support::skip(&format!("{name} exists but doesn't read"));
            None
        }
    }
}

/// The whole view for a profile, built exactly the way `crates/app/src/commands/graph.rs`
/// builds it: catalog, dataset, flags, graph, evaluation **and the counters**. Without the
/// counters a `Counter` requirement has no `current`, and the question this answers would be
/// decided by the fixture rather than by the profile.
fn view_of(c: &Catalog, s: &Save) -> Option<ipc::UnlockView> {
    let flags = s.flags(Kind::Achievements)?;
    let counters = s.u32s(Kind::Counters)?;
    let g = graph::Graph::build(c, graph::rules::embedded().expect("embedded rules"));
    let progress = ipc::SaveProgress::new(Some(&flags), Some(&counters), Some(c));
    let e = g.evaluate(&progress);
    Some(unlock_view(
        Some(c),
        wiki::Dataset::embedded().ok(),
        Some(&flags),
        Some(&g),
        Some(&e),
        Some(&progress),
        |_| None,
    ))
}

#[test]
fn the_real_profile_has_379_done_637_known_and_4_unknown_slots() {
    let Some((c, _, s)) = real() else { return };
    let flags = s.flags(Kind::Achievements).expect("section 1");
    let v = unlock_view(
        Some(&c),
        None,
        Some(&flags),
        None,
        None,
        None,
        |r: &ipc::IconRef| Some(format!("{}://{}", ipc::ICON_SCHEME, r.to_path())),
    );
    // 642 flags but 641 nodes (slots 1..=641: slot 0 isn't a node); the catalog covers
    // 1..=637, so what's left beyond the catalog is 638..=641: four, not 642 - 637.
    assert_eq!(
        (
            v.totals.slots,
            v.totals.done,
            v.totals.known,
            v.totals.unknown
        ),
        (642, 379, 637, 4)
    );
    assert_eq!(
        v.diagnostics,
        vec![UnlockDiagnostic::SlotsBeyondCatalog { count: 4 }]
    );
    let unknown_done: Vec<u32> = v
        .nodes
        .iter()
        .filter_map(|n| match n.achievement {
            AchievementRef::Unknown { slot } if n.done => Some(slot),
            AchievementRef::Unknown { .. } | AchievementRef::Known { .. } => None,
        })
        .collect();
    assert_eq!(
        unknown_done,
        vec![638, 639, 641],
        "achievements newer than the catalog, done"
    );
}

#[test]
fn the_slot_id_junction_is_pinned_by_the_items_seen_in_the_save() {
    // Among the items seen (section 4) that have an achievement, how many have the
    // `done` node? slot[id]: 169 of 171; slot[id-1]: 145; slot[id+1]: 142. If it drops,
    // it's the alignment that's broken, not a number to adjust.
    let Some((c, _, s)) = real() else { return };
    let flags = s.flags(Kind::Achievements).expect("section 1");
    let seen = s.flags(Kind::Items).expect("section 4");
    let v = unlock_view(Some(&c), None, Some(&flags), None, None, None, |_| None);
    let done: BTreeSet<u32> = v
        .nodes
        .iter()
        .filter(|n| n.done)
        .filter_map(|n| match n.achievement {
            AchievementRef::Known { id, .. } => Some(id),
            AchievementRef::Unknown { .. } => None,
        })
        .collect();
    let (mut agree, mut total) = (0, 0);
    for i in c.items().filter(|i| i.kind != ItemKind::Trinket) {
        let Some(a) = i.unlocked_by else { continue };
        if !seen.get(i.id.0 as usize).copied().unwrap_or(false) {
            continue;
        }
        total += 1;
        if done.contains(&a.0) {
            agree += 1;
        }
    }
    assert_eq!((agree, total), (169, 171));
}

#[test]
fn next_steps_on_the_real_profile_are_unlockable_now_by_fan_out() {
    let Some((c, _, s)) = real() else { return };
    let flags = s.flags(Kind::Achievements).expect("section 1");
    let g = graph::Graph::build(&c, graph::rules::embedded().expect("embedded rules"));
    let e = g.evaluate(&graph::FlagsOnly(Some(&flags)));
    let v = unlock_view(
        Some(&c),
        None,
        Some(&flags),
        Some(&g),
        Some(&e),
        None,
        |r: &ipc::IconRef| Some(format!("{}://{}", ipc::ICON_SCHEME, r.to_path())),
    );
    let steps = next_steps(&v, &Default::default());
    let all: Vec<&ipc::UnlockNode> = steps.sections.iter().flat_map(|s| s.steps.iter()).collect();
    assert!(!all.is_empty(), "the real profile has work left to do");
    assert!(all.iter().all(|n| !n.done));
    // Every section is capped at STEPS, and every emitted section holds something.
    assert!(steps
        .sections
        .iter()
        .all(|s| !s.steps.is_empty() && s.steps.len() <= ipc::STEPS));
    let by_fan_out = steps
        .sections
        .iter()
        .find(|s| s.basis == ipc::StepsBasis::FanOut)
        .expect("the real profile has nodes with nothing at all in the way");
    let fans: Vec<u32> = by_fan_out
        .steps
        .iter()
        .map(|n| match n.graph {
            ipc::GraphInfo::Computed {
                available_now,
                blocked_by,
                fan_out,
                ..
            } => {
                assert!(available_now, "a step that isn't unlockable isn't a step");
                assert_eq!(blocked_by, 0);
                fan_out
            }
            ipc::GraphInfo::Partial { .. } => {
                panic!("a partial node can't be recommended: the graph can't vouch for it")
            }
        })
        .collect();
    assert!(
        fans.windows(2).all(|w| w[0] >= w[1]),
        "fan-out descending: {fans:?}"
    );
    // Every step has a readable name and, if it unlocks an item, its icon — in every
    // section, not only the one whose order was just checked.
    for n in &all {
        for t in &n.unlocks {
            match t {
                UnlockTarget::Item { name, icon_url, .. } => {
                    assert!(!name.is_empty());
                    assert!(icon_url.is_some());
                }
                UnlockTarget::Character { name, .. }
                | UnlockTarget::Boss { name, .. }
                | UnlockTarget::Challenge { name, .. } => assert!(!name.is_empty()),
            }
        }
    }
}

// --- resolve_target ---------------------------------------------------------------

/// Saved keys resolve against the real catalog: a name for all four families, an icon
/// for the item. The database keeps none of this.
#[test]
fn the_real_catalog_resolves_a_saved_key_into_a_named_target() {
    let Some((c, rs, _)) = real() else { return };
    let mut icon = |r: &ipc::IconRef| Some(format!("{}://{}", ipc::ICON_SCHEME, r.to_path()));
    // The first of each family, read from the game files on 2026-09-05.
    let item = resolve_target(
        &c,
        &TargetKey::Item {
            item_kind: ItemKindView::Passive,
            id: 1,
        },
        None,
        &mut icon,
    )
    .expect("item 1 is The Sad Onion");
    match &item {
        UnlockTarget::Item {
            item_kind,
            id,
            name,
            icon_url,
            page,
        } => {
            // No dataset was passed here: this test is about the name and the icon. What a
            // page carries when a dataset *is* there is the next test's subject.
            assert_eq!(page, &None);
            assert_eq!((*item_kind, *id), (ItemKindView::Passive, 1));
            assert_eq!(name, "The Sad Onion");
            assert_eq!(
                icon_url.as_deref(),
                Some("isaac://item/passive/1"),
                "the row carries a link, not a picture"
            );
            // The link above is only a promise; this is the half that keeps it. The view
            // no longer extracts anything, so without this the test would pass just as
            // happily on an install whose archives don't hold the sprite at all.
            let sprite = ipc::icon_source(
                &c,
                &ipc::IconRef::Item {
                    kind: ItemKindView::Passive,
                    id: 1,
                },
            )
            .expect("the catalog names a sprite for item 1");
            assert!(
                rs.read(&sprite.path)
                    .is_some_and(|png| png.starts_with(b"\x89PNG")),
                "the sprite the link points at really extracts from the archives"
            );
        }
        other => panic!("expected an item, got {other:?}"),
    }
    let character =
        resolve_target(&c, &TargetKey::Character { id: 0 }, None, &mut icon).expect("character 0");
    assert_eq!(
        character,
        UnlockTarget::Character {
            id: 0,
            name: "Isaac".into(),
            tainted: false,
            page: None
        }
    );
    let boss = resolve_target(&c, &TargetKey::Boss { id: 1 }, None, &mut icon).expect("boss 1");
    assert_eq!(
        boss,
        UnlockTarget::Boss {
            id: 1,
            name: "Monstro".into(),
            page: None
        }
    );
    let challenge =
        resolve_target(&c, &TargetKey::Challenge { id: 1 }, None, &mut icon).expect("challenge 1");
    assert_eq!(
        challenge,
        UnlockTarget::Challenge {
            id: 1,
            name: "Pitch Black".into(),
            // achievements.xml: `<!-- Beat Challenge #1 -->` right above achievement 89 (Rune of Hagalaz).
            rewards: vec![89],
            page: None,
        }
    );

    // `key()` and `resolve_target` are each other's inverse, on real data.
    for t in [item, character, boss, challenge] {
        assert_eq!(
            resolve_target(&c, &t.key(), None, &mut icon).as_ref(),
            Some(&t),
            "resolving a target's key gives back the same target"
        );
    }
}

/// A key the catalog doesn't know isn't an error and doesn't invent a name: it's
/// `None`, and the Plan names it by id.
#[test]
fn an_absurd_key_resolves_to_nothing() {
    let Some((c, _, _)) = real() else { return };
    let mut icon = |_: &ipc::IconRef| None;
    for key in [
        TargetKey::Item {
            item_kind: ItemKindView::Passive,
            id: 999_999,
        },
        TargetKey::Character { id: 999_999 },
        TargetKey::Boss { id: 999_999 },
        TargetKey::Challenge { id: 999_999 },
    ] {
        assert_eq!(resolve_target(&c, &key, None, &mut icon), None, "{key:?}");
    }
}

/// The Tainted characters share their `text` with the base form: `achievements.xml` writes
/// *You unlocked "The Lost"* for both slot 82 and slot 484, and says "Tainted" nowhere.
/// What tells the two apart is `players.xml` — player 10's portrait is
/// `Character_012_TheLost.png` and player 31's is `Character_012b_TheLost.png`, the `b` of
/// the Tainted form — so the target has to carry the flag, or two different characters go
/// out under one name.
#[test]
fn the_tainted_form_of_a_character_is_a_different_target_under_the_same_name() {
    let Some((c, _, _)) = real() else { return };
    let mut icon = |_: &ipc::IconRef| None;
    let by_achievement = |slot: u32| {
        c.characters()
            .find(|ch| ch.unlocked_by == Some(catalog::AchievementId(slot)))
            .map(|ch| ch.id.0)
    };
    let (Some(base), Some(tainted)) = (by_achievement(82), by_achievement(484)) else {
        test_support::skip("this catalog has no character unlocked by 82 and 484");
        return;
    };
    let base = resolve_target(&c, &TargetKey::Character { id: base }, None, &mut icon)
        .expect("the base character");
    let tainted = resolve_target(&c, &TargetKey::Character { id: tainted }, None, &mut icon)
        .expect("the tainted character");
    let (
        UnlockTarget::Character {
            name: base_name,
            tainted: base_flag,
            ..
        },
        UnlockTarget::Character {
            name: tainted_name,
            tainted: tainted_flag,
            ..
        },
    ) = (&base, &tainted)
    else {
        panic!("expected two characters, got {base:?} and {tainted:?}");
    };
    assert_eq!(
        base_name, tainted_name,
        "the two forms share the game's name: that is why the flag exists"
    );
    assert!(!base_flag, "slot 82 unlocks the base form");
    assert!(tainted_flag, "slot 484 unlocks the tainted form");
}

/// A requirement links to the page the dataset really has, and to nothing else. Real catalog,
/// real graph, real dataset: the mapping is only interesting where the ids are the game's.
#[test]
fn a_blocked_node_links_to_the_pages_the_dataset_has() {
    let Some((c, _, s)) = real() else { return };
    let flags = s.flags(Kind::Achievements).expect("section 1");
    let g = graph::Graph::build(&c, graph::rules::embedded().expect("embedded rules"));
    let e = g.evaluate(&graph::FlagsOnly(Some(&flags)));
    let ds = wiki::Dataset::embedded().expect("the dataset is embedded at build time");

    let page_of = |r: &ipc::RequirementView| match r {
        ipc::RequirementView::Character { page, .. }
        | ipc::RequirementView::Boss { page, .. }
        | ipc::RequirementView::Challenge { page, .. }
        | ipc::RequirementView::Item { page, .. }
        // A transformation does have a page, and since 2026-09-13 the dataset has it.
        | ipc::RequirementView::Threshold { page, .. } => page.clone(),
        // A gate, a mark, a counter and an uninterpreted label are conditions, not entities:
        // there is no page to carry (plan amendment, 2026-09-12).
        ipc::RequirementView::Gate { .. }
        | ipc::RequirementView::Mark { .. }
        | ipc::RequirementView::Counter { .. }
        | ipc::RequirementView::Unknown { .. } => None,
    };

    let v = unlock_view(
        Some(&c),
        Some(ds),
        Some(&flags),
        Some(&g),
        Some(&e),
        None,
        |_| None,
    );
    let linked = v
        .nodes
        .iter()
        .flat_map(|n| n.missing.iter())
        .filter_map(&page_of)
        .inspect(|t| assert!(ds.entry(t).is_some(), "a page that goes out has to exist"))
        .count();
    assert!(
        linked > 0,
        "the real profile has blockers the wiki documents"
    );

    // No dataset: the names still come out, and nothing links.
    let without = unlock_view(
        Some(&c),
        None,
        Some(&flags),
        Some(&g),
        Some(&e),
        None,
        |_| None,
    );
    assert!(without
        .nodes
        .iter()
        .flat_map(|n| n.missing.iter())
        .all(|r| page_of(r).is_none()));
    assert_eq!(without.totals, v.totals, "only the pages changed");
    assert_eq!(
        without.nodes.iter().map(|n| n.missing.len()).sum::<usize>(),
        v.nodes.iter().map(|n| n.missing.len()).sum::<usize>(),
        "a missing page never removes a requirement"
    );
}

/// The other half of the same rule (`docs/BACKLOG.md` B35): what a node **unlocks** links to
/// the page the dataset really has, and to nothing else. Deliberately the same shape as the
/// test above — one mapping, two readers, and a single place where it could drift.
#[test]
fn what_a_node_unlocks_links_to_the_pages_the_dataset_has() {
    let Some((c, _, s)) = real() else { return };
    let flags = s.flags(Kind::Achievements).expect("section 1");
    let ds = wiki::Dataset::embedded().expect("the dataset is embedded at build time");

    let page_of = |t: &UnlockTarget| match t {
        UnlockTarget::Item { page, .. }
        | UnlockTarget::Character { page, .. }
        | UnlockTarget::Boss { page, .. }
        | UnlockTarget::Challenge { page, .. } => page.clone(),
    };

    let v = unlock_view(Some(&c), Some(ds), Some(&flags), None, None, None, |_| None);
    let targets = || v.nodes.iter().flat_map(|n| n.unlocks.iter());
    let linked = targets()
        .filter_map(&page_of)
        .inspect(|t| assert!(ds.entry(t).is_some(), "a page that goes out has to exist"))
        .count();
    assert!(
        targets().count() > 0,
        "the real catalog has edges, or this test asserts nothing"
    );
    assert!(
        linked > 0,
        "the dataset documents most of what the game unlocks: zero links means the mapping \
         stopped working, not that the wiki is empty"
    );

    // No dataset: the names still come out, and nothing links.
    let without = unlock_view(Some(&c), None, Some(&flags), None, None, None, |_| None);
    assert!(without
        .nodes
        .iter()
        .flat_map(|n| n.unlocks.iter())
        .all(|t| page_of(t).is_none()));
    assert_eq!(
        without.nodes.iter().map(|n| n.unlocks.len()).sum::<usize>(),
        v.nodes.iter().map(|n| n.unlocks.len()).sum::<usize>(),
        "a missing page never removes a target"
    );
}

/// The vacuity guard for the closeness section, and the reason it is aimed at **this**
/// profile rather than the reference one.
///
/// Measured 2026-09-13: on `live` (379 of 642 done) the whole view holds **zero** `Counter`
/// requirements — not zero closeness steps, zero counters. A counter is only reported while
/// `current < at_least`, and that profile crossed every one of those thresholds long ago. The
/// co-op partner's profile, the one `online_logs\` leaves at the start of the progression
/// (`CLAUDE.md`, "Real-world paths"), holds 4 of them and 4 nodes held by nothing else.
///
/// So the property is asserted where the thing it is about exists. A test that cannot fail
/// reports coverage that is not there.
#[test]
fn a_young_profile_has_a_closeness_section_and_it_is_ordered_by_distance() {
    let Some((c, s)) = profile("20260912.coop-partner.persistentgamedata1.dat") else {
        return;
    };
    let Some(v) = view_of(&c, &s) else {
        test_support::skip("the young profile has no achievement or counter section");
        return;
    };

    let steps = next_steps(&v, &Default::default());
    let Some(close) = steps
        .sections
        .iter()
        .find(|x| x.basis == ipc::StepsBasis::Closeness)
    else {
        panic!(
            "the young profile had 4 nodes held only by counters when this was measured: no \
             closeness section means the model stopped producing them"
        )
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
    // The order the section promises, read back from the requirements themselves.
    let distances: Vec<u32> = close
        .steps
        .iter()
        .map(|n| {
            n.missing
                .iter()
                .map(|r| match r {
                    ipc::RequirementView::Counter {
                        current, at_least, ..
                    } => at_least.saturating_sub(*current),
                    // Unreachable: the loop above asserted every one of them is a counter.
                    // Spelled out all the same — a new variant has to break this build.
                    ipc::RequirementView::Character { .. }
                    | ipc::RequirementView::Boss { .. }
                    | ipc::RequirementView::Challenge { .. }
                    | ipc::RequirementView::Item { .. }
                    | ipc::RequirementView::Gate { .. }
                    | ipc::RequirementView::Mark { .. }
                    | ipc::RequirementView::Threshold { .. }
                    | ipc::RequirementView::Unknown { .. } => 0,
                })
                .sum()
        })
        .collect();
    assert!(
        distances.windows(2).all(|w| w[0] <= w[1]),
        "nearest the threshold first: {distances:?}"
    );
}

/// Why the reference profile has no closeness section — pinned, so that "it's empty" stays a
/// fact about the profile and never becomes an unnoticed fact about the code.
///
/// An empty section and a broken model look identical from the outside. This tells them
/// apart: on `live` there is no `Counter` requirement left standing at all, because every
/// threshold has been crossed. The day one appears, this test fails and the answer is to move
/// the expectation, not to wonder why a section reappeared.
#[test]
fn the_reference_profile_has_crossed_every_counter_threshold() {
    let Some((c, _, s)) = real() else { return };
    let Some(v) = view_of(&c, &s) else { return };
    let counters: Vec<&ipc::RequirementView> = v
        .nodes
        .iter()
        .flat_map(|n| n.missing.iter())
        .filter(|r| matches!(r, ipc::RequirementView::Counter { .. }))
        .collect();
    assert!(
        counters.is_empty(),
        "measured 2026-09-13: none stood on this profile, and these do: {counters:?}"
    );
    assert!(
        next_steps(&v, &Default::default())
            .sections
            .iter()
            .all(|x| x.basis != ipc::StepsBasis::Closeness),
        "no counter standing, so no section: the two facts are one"
    );
}

/// The one line that says **how to get it**, and why it needs two sources.
///
/// Measured 2026-09-13 on the reference profile's 637 known achievements: the game states an
/// `unlock_condition` for 283 of them and nothing for 354 — and among the 119 that are
/// unlockable *now*, the pool the landing page draws from, only **16**. A card whose "how"
/// line came from the file alone would be blank seven rows out of eight, which is what B32
/// asked to fix, not a fix for it.
///
/// So the dataset answers where the file is silent, and this pins both halves: with a dataset
/// strictly more achievements carry a line, and the ones the file already answered keep the
/// game's own words.
#[test]
fn the_wiki_answers_how_to_get_it_where_the_game_file_is_silent() {
    let Some((c, _, s)) = real() else { return };
    let flags = s.flags(Kind::Achievements).expect("section 1");

    let conditions = |dataset: Option<&wiki::Dataset>| -> Vec<(u32, Option<String>)> {
        unlock_view(Some(&c), dataset, Some(&flags), None, None, None, |_| None)
            .nodes
            .iter()
            .filter_map(|n| match &n.achievement {
                AchievementRef::Known { id, condition, .. } => Some((*id, condition.clone())),
                AchievementRef::Unknown { .. } => None,
            })
            .collect()
    };

    let from_file = conditions(None);
    let with_wiki = conditions(wiki::Dataset::embedded().ok());
    let answered = |v: &[(u32, Option<String>)]| v.iter().filter(|(_, c)| c.is_some()).count();

    assert!(
        !from_file.is_empty(),
        "the real catalog has achievements, or this test asserts nothing"
    );
    // Declared, not only asserted: how far the two sources reach is the fact this test is
    // about, and a number nobody can read is a number nobody checks.
    eprintln!(
        "condition: {} of {} known achievements from the game file, {} once the wiki answers",
        answered(&from_file),
        from_file.len(),
        answered(&with_wiki)
    );
    assert!(
        answered(&with_wiki) > answered(&from_file),
        "the dataset has to answer for some of the ones the file leaves silent: {} with the \
         wiki against {} from the file alone",
        answered(&with_wiki),
        answered(&from_file)
    );

    // The game's own words win where it has any: the wiki is the fallback, never the rewrite.
    for ((id, file), (_, wiki_too)) in from_file.iter().zip(&with_wiki) {
        if let Some(file) = file {
            assert_eq!(
                wiki_too.as_ref(),
                Some(file),
                "achievement {id}: the file stated a condition and it was replaced"
            );
        }
    }
    // A line that goes out is a line worth reading: never an empty string.
    assert!(with_wiki
        .iter()
        .filter_map(|(_, c)| c.as_ref())
        .all(|c| !c.trim().is_empty()));
}
