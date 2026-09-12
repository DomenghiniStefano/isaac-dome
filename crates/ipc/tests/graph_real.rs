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

#[test]
fn the_real_profile_has_379_done_637_known_and_4_unknown_slots() {
    let Some((c, _, s)) = real() else { return };
    let flags = s.flags(Kind::Achievements).expect("section 1");
    let v = unlock_view(Some(&c), Some(&flags), None, None, |r: &ipc::IconRef| {
        Some(format!("{}://{}", ipc::ICON_SCHEME, r.to_path()))
    });
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
    let v = unlock_view(Some(&c), Some(&flags), None, None, |_| None);
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
        Some(&flags),
        Some(&g),
        Some(&e),
        |r: &ipc::IconRef| Some(format!("{}://{}", ipc::ICON_SCHEME, r.to_path())),
    );
    let steps = next_steps(&v);
    assert_eq!(steps.basis, ipc::StepsBasis::FanOut);
    assert_eq!(steps.steps.len(), 5, "the real profile has work left to do");
    assert!(steps.steps.iter().all(|n| !n.done));
    let fans: Vec<u32> = steps
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
    // Every step has a readable name and, if it unlocks an item, its icon.
    for n in &steps.steps {
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
        &mut icon,
    )
    .expect("item 1 is The Sad Onion");
    match &item {
        UnlockTarget::Item {
            item_kind,
            id,
            name,
            icon_url,
        } => {
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
        resolve_target(&c, &TargetKey::Character { id: 0 }, &mut icon).expect("character 0");
    assert_eq!(
        character,
        UnlockTarget::Character {
            id: 0,
            name: "Isaac".into(),
            tainted: false
        }
    );
    let boss = resolve_target(&c, &TargetKey::Boss { id: 1 }, &mut icon).expect("boss 1");
    assert_eq!(
        boss,
        UnlockTarget::Boss {
            id: 1,
            name: "Monstro".into()
        }
    );
    let challenge =
        resolve_target(&c, &TargetKey::Challenge { id: 1 }, &mut icon).expect("challenge 1");
    assert_eq!(
        challenge,
        UnlockTarget::Challenge {
            id: 1,
            name: "Pitch Black".into(),
            // achievements.xml: `<!-- Beat Challenge #1 -->` right above achievement 89 (Rune of Hagalaz).
            rewards: vec![89],
        }
    );

    // `key()` and `resolve_target` are each other's inverse, on real data.
    for t in [item, character, boss, challenge] {
        assert_eq!(
            resolve_target(&c, &t.key(), &mut icon).as_ref(),
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
        assert_eq!(resolve_target(&c, &key, &mut icon), None, "{key:?}");
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
    let base = resolve_target(&c, &TargetKey::Character { id: base }, &mut icon)
        .expect("the base character");
    let tainted = resolve_target(&c, &TargetKey::Character { id: tainted }, &mut icon)
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
