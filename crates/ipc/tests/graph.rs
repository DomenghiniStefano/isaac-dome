use ipc::{
    AchievementRef, GraphInfo, ItemKindView, NextSteps, OriginView, PlanDiagnostic, PlanExpansion,
    StepsBasis, UnlockDiagnostic, UnlockNode, UnlockTarget, UnlockTotals, UnlockView, STEPS,
};
use serde_json::{json, to_value, Value};

fn node(done: bool) -> UnlockNode {
    UnlockNode {
        achievement: AchievementRef::Known {
            id: 1,
            text: "You unlocked \"Magdalene\"".into(),
            hint: Some("have 7 or more max red hearts at one time".into()),
            icon_url: None,
        },
        done,
        unlocks: vec![UnlockTarget::Character {
            id: 1,
            name: "Magdalene".into(),
        }],
        origin: None,
        graph: GraphInfo::Stub,
    }
}

#[test]
fn unlock_node_json_shape_is_pinned() {
    let v = to_value(node(true)).unwrap();
    assert_eq!(v["achievement"]["kind"], "known");
    assert_eq!(v["achievement"]["id"], 1);
    assert_eq!(
        v["achievement"]["hint"],
        "have 7 or more max red hearts at one time"
    );
    assert_eq!(v["achievement"]["iconUrl"], Value::Null);
    assert_eq!(v["done"], true);
    assert_eq!(v["unlocks"][0]["kind"], "character");
    assert_eq!(v["origin"], Value::Null);
    assert_eq!(v["graph"], json!({ "kind": "stub" }));
}

#[test]
fn unknown_achievement_and_computed_graph_are_pinned_too() {
    let mut n = node(false);
    n.achievement = AchievementRef::Unknown { slot: 640 };
    n.graph = GraphInfo::Computed {
        available_now: true,
        blocked_by: 0,
        fan_out: 3,
        steps_missing: 1,
    };
    n.origin = Some(OriginView::AfterbirthPlus);
    let v = to_value(&n).unwrap();
    assert_eq!(v["achievement"], json!({ "kind": "unknown", "slot": 640 }));
    assert_eq!(
        v["graph"],
        json!({ "kind": "computed", "availableNow": true, "blockedBy": 0, "fanOut": 3, "stepsMissing": 1 })
    );
    assert_eq!(v["origin"], "afterbirthPlus");
}

#[test]
fn item_target_uses_item_kind_not_kind_for_the_item_type() {
    let t = UnlockTarget::Item {
        item_kind: ItemKindView::Trinket,
        id: 1,
        name: "Swallowed Penny".into(),
        icon_url: Some("data:image/png;base64,AA==".into()),
    };
    let v = to_value(&t).unwrap();
    assert_eq!(v["kind"], "item");
    // `itemKind` is a string: the enum has no fields, so it isn't tagged.
    assert_eq!(v["itemKind"], "trinket");
    assert_eq!(
        v.as_object().unwrap().len(),
        5,
        "kind, itemKind, id, name, iconUrl: no key overwritten"
    );
}

#[test]
fn views_and_diagnostics_are_pinned() {
    let view = UnlockView {
        nodes: vec![node(true)],
        totals: UnlockTotals {
            slots: 642,
            done: 379,
            known: 637,
            unknown: 4,
        },
        diagnostics: vec![
            UnlockDiagnostic::SlotsBeyondCatalog { count: 4 },
            UnlockDiagnostic::NoCatalog,
        ],
    };
    let v = to_value(&view).unwrap();
    assert_eq!(
        v["totals"],
        json!({ "slots": 642, "done": 379, "known": 637, "unknown": 4 })
    );
    assert_eq!(
        v["diagnostics"][0],
        json!({ "kind": "slotsBeyondCatalog", "count": 4 })
    );
    assert_eq!(v["diagnostics"][1], json!({ "kind": "noCatalog" }));

    let steps = NextSteps {
        steps: vec![],
        basis: StepsBasis::Stub,
    };
    // `StepsBasis` has no fields: it's a bare string, like `itemKind` and `origin`.
    assert_eq!(to_value(&steps).unwrap()["basis"], "stub");
    assert_eq!(
        to_value(NextSteps {
            steps: vec![],
            basis: StepsBasis::FanOut,
        })
        .unwrap()["basis"],
        "fanOut"
    );
    assert_eq!(STEPS, 5);

    // The plan is only ever built via `plan_view`: `storeAvailable` and the store
    // diagnostic are born from the same argument and can never contradict each other.
    let c = catalog_with_achievements();
    let v = to_value(plan_view(Some(&c), vec![], vec![], None, |_| None)).unwrap();
    assert_eq!(v["expansion"], json!({ "kind": "stub" }));
    assert_eq!(v["diagnostics"], json!([]));
    assert_eq!(v["storeAvailable"], true);

    let v = to_value(plan_view(
        Some(&c),
        vec![],
        vec![],
        Some("database di una versione più nuova (7 > 1)".into()),
        |_| None,
    ))
    .unwrap();
    assert_eq!(v["storeAvailable"], false);
    assert_eq!(
        v["diagnostics"],
        json!([{ "kind": "storeUnavailable", "reason": "database di una versione più nuova (7 > 1)" }])
    );

    // A database row that fails to read: the UI receives the id, not the broken JSON.
    let v = to_value(plan_view(
        Some(&c),
        vec![],
        vec![ipc::GoalId::from_str_unchecked("g9")],
        None,
        |_| None,
    ))
    .unwrap();
    assert_eq!(v["storeAvailable"], true);
    assert_eq!(
        v["diagnostics"],
        json!([{ "kind": "unreadableGoal", "id": "g9" }])
    );
}

#[test]
fn store_available_and_the_store_diagnostic_cannot_disagree() {
    let c = catalog_with_achievements();
    for reason in [None, Some("database illeggibile".to_string())] {
        let p = plan_view(Some(&c), vec![], vec![], reason.clone(), |_| None);
        let says_unavailable = p
            .diagnostics
            .iter()
            .any(|d| matches!(d, PlanDiagnostic::StoreUnavailable { .. }));
        assert_eq!(p.store_available, reason.is_none());
        assert_eq!(says_unavailable, reason.is_some());
    }
}

use catalog::Catalog;
use ipc::{next_steps, plan_view, unlock_view};

const ITEMS_WITH_ACHIEVEMENTS: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"2\" gfx=\"a.png\" name=\"A\" achievement=\"1\" /><trinket id=\"1\" gfx=\"t.png\" name=\"T\" achievement=\"3\" /></items>";
const ACH: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\"><!-- c1 --><achievement id=\"1\" text=\"t1\" gfx=\"1.png\" /><achievement id=\"2\" text=\"t2\" gfx=\"2.png\" /><achievement id=\"3\" text=\"t3\" gfx=\"3.png\" /></achievements>";
const PLAYERS: &[u8] = b"<players portraitroot=\"gfx/ui/stage/\"><player id=\"7\" name=\"#Z_NAME\" portrait=\"z.png\" achievement=\"2\" /></players>";

/// Test catalog for the graph: every item and every character points to an
/// achievement, which is the relationship these views show. Not the catalog from
/// `catalog_view.rs`, which exists to test name resolution.
fn catalog_with_achievements() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS_WITH_ACHIEVEMENTS.to_vec()),
        "achievements.xml" => Some(ACH.to_vec()),
        "players.xml" => Some(PLAYERS.to_vec()),
        _ => None,
    })
}

#[test]
fn unlock_view_maps_slots_to_achievements_and_marks_the_ones_beyond_the_catalog() {
    // 6 slots: 0 unused, 1..=3 known, 4..=5 beyond the catalog.
    let flags = [false, true, false, true, true, false];
    let v = unlock_view(Some(&catalog_with_achievements()), Some(&flags), |_| None);
    assert_eq!(v.nodes.len(), 5, "one per slot 1..=5");
    assert!(
        matches!(&v.nodes[0].achievement, AchievementRef::Known { id: 1, hint: Some(h), .. } if h == "c1")
    );
    assert!(v.nodes[0].done);
    assert!(!v.nodes[1].done);
    assert!(v.nodes[2].done);
    assert_eq!(v.nodes[3].achievement, AchievementRef::Unknown { slot: 4 });
    assert!(
        v.nodes[3].done,
        "done, but the catalog doesn't know what it is"
    );
    assert_eq!(v.nodes[4].achievement, AchievementRef::Unknown { slot: 5 });
    assert_eq!(
        v.totals,
        UnlockTotals {
            slots: 6,
            done: 3,
            known: 3,
            unknown: 2
        }
    );
    assert_eq!(
        v.diagnostics,
        vec![UnlockDiagnostic::SlotsBeyondCatalog { count: 2 }]
    );
    assert!(v.nodes.iter().all(|n| n.graph == GraphInfo::Stub));
}

#[test]
fn unlocks_and_origin_come_from_the_catalog_and_icons_only_when_they_resolve() {
    let flags = [false, false, false, false];
    let v = unlock_view(Some(&catalog_with_achievements()), Some(&flags), |p| {
        (p == "gfx/items/collectibles/a.png").then(|| vec![0x89, b'P', b'N', b'G'])
    });
    let n1 = &v.nodes[0];
    assert_eq!(n1.unlocks.len(), 1);
    assert!(
        matches!(&n1.unlocks[0], UnlockTarget::Item { item_kind: ItemKindView::Passive, id: 2, name, icon_url: Some(u) } if name == "A" && u.starts_with("data:image/png"))
    );
    assert_eq!(
        n1.origin,
        Some(OriginView::Rebirth),
        "item 2 is from Rebirth"
    );
    let n2 = &v.nodes[1];
    assert!(matches!(&n2.unlocks[0], UnlockTarget::Character { id: 7, name } if name == "Z_NAME"));
    assert_eq!(n2.origin, None, "the first target isn't an item");
    let n3 = &v.nodes[2];
    assert!(
        matches!(
            &n3.unlocks[0],
            UnlockTarget::Item {
                item_kind: ItemKindView::Trinket,
                icon_url: None,
                ..
            }
        ),
        "sprite not resolved: the row stays"
    );
}

#[test]
fn catalog_beyond_slots_and_no_catalog_degrade_with_a_diagnostic() {
    let v = unlock_view(
        Some(&catalog_with_achievements()),
        Some(&[false, true]),
        |_| None,
    );
    assert_eq!(v.nodes.len(), 1);
    assert_eq!(
        v.diagnostics,
        vec![UnlockDiagnostic::CatalogBeyondSlots { count: 2 }]
    );

    let v = unlock_view(None, Some(&[false, true, true]), |_| None);
    assert_eq!(v.nodes.len(), 2);
    assert!(v
        .nodes
        .iter()
        .all(|n| matches!(n.achievement, AchievementRef::Unknown { .. })));
    assert_eq!(v.totals.done, 2);
    assert_eq!(v.diagnostics, vec![UnlockDiagnostic::NoCatalog]);
}

/// Section 1 failing to read is not the same as a save with no achievements. The
/// catalog exists and knows 3 of them: saying "the catalog has 4 more than the file"
/// would be false, because nothing is known about the file.
#[test]
fn a_missing_achievement_section_is_declared_and_compares_nothing() {
    let v = unlock_view(Some(&catalog_with_achievements()), None, |_| None);
    assert!(v.nodes.is_empty());
    assert_eq!(
        v.totals,
        UnlockTotals {
            slots: 0,
            done: 0,
            known: 0,
            unknown: 0
        }
    );
    assert_eq!(
        v.diagnostics,
        vec![UnlockDiagnostic::NoAchievementSection],
        "no CatalogBeyondSlots: there's nothing to compare the catalog against"
    );
    assert_eq!(
        to_value(&v).unwrap()["diagnostics"],
        json!([{ "kind": "noAchievementSection" }])
    );
    // No catalog and no section: two different pieces of news, two diagnostics.
    let v = unlock_view(None, None, |_| None);
    assert_eq!(
        v.diagnostics,
        vec![
            UnlockDiagnostic::NoCatalog,
            UnlockDiagnostic::NoAchievementSection
        ]
    );
}

/// The degenerate save stays as it was: the section exists and is empty, and the view
/// says the catalog knows more than the file.
#[test]
fn an_empty_but_present_section_still_compares_with_the_catalog() {
    let v = unlock_view(Some(&catalog_with_achievements()), Some(&[]), |_| None);
    assert!(v.nodes.is_empty());
    assert_eq!(v.totals.slots, 0);
    assert_eq!(
        v.diagnostics,
        vec![UnlockDiagnostic::CatalogBeyondSlots { count: 4 }],
        "3 achievements in the catalog plus slot 0"
    );
}

#[test]
fn next_steps_are_the_first_not_done_in_slot_order_capped_at_steps() {
    let mut flags = vec![false; 10];
    flags[2] = true;
    flags[5] = true;
    let v = unlock_view(None, Some(&flags), |_| None);
    let s = next_steps(&v);
    assert_eq!(s.basis, StepsBasis::Stub);
    let slots: Vec<u32> = s
        .steps
        .iter()
        .map(|n| match n.achievement {
            AchievementRef::Unknown { slot } => slot,
            AchievementRef::Known { id, .. } => id,
        })
        .collect();
    assert_eq!(slots, vec![1, 3, 4, 6, 7]);
    assert_eq!(s.steps.len(), STEPS);
}

/// Item 2 from the test catalog: passive, name "A", sprite `gfx/items/a.png`.
fn goal(id: &str) -> ipc::Goal {
    ipc::Goal {
        id: ipc::GoalId::from_str_unchecked(id),
        target: ipc::TargetKey::Item {
            item_kind: ItemKindView::Passive,
            id: 2,
        },
        created_unix: 0,
        note: None,
    }
}

#[test]
fn plan_view_keeps_goal_order_and_reports_the_store() {
    let c = catalog_with_achievements();
    let p = plan_view(Some(&c), vec![goal("b"), goal("a")], vec![], None, |_| None);
    assert_eq!(
        p.goals.iter().map(|g| g.id.as_str()).collect::<Vec<_>>(),
        vec!["b", "a"]
    );
    assert_eq!(p.expansion, PlanExpansion::Stub);
    assert!(p.store_available);
    assert!(p.diagnostics.is_empty());
    assert!(!plan_view(Some(&c), vec![], vec![], Some("x".into()), |_| None).store_available);
    // Unreadable ids become diagnostics, one per row, in the order received.
    let bad = |s: &str| ipc::GoalId::from_str_unchecked(s);
    let p = plan_view(
        Some(&c),
        vec![goal("a")],
        vec![bad("x"), bad("y")],
        None,
        |_| None,
    );
    assert_eq!(
        p.diagnostics,
        vec![
            PlanDiagnostic::UnreadableGoal { id: bad("x") },
            PlanDiagnostic::UnreadableGoal { id: bad("y") },
        ]
    );
}

/// The database keeps the key; name and icon are born here, from the current catalog.
#[test]
fn a_goal_carries_its_key_and_the_target_resolved_now() {
    let c = catalog_with_achievements();
    let p = plan_view(Some(&c), vec![goal("g1")], vec![], None, |p| {
        (p == "gfx/items/collectibles/a.png").then(|| vec![0x89, b'P', b'N', b'G'])
    });
    let v = to_value(&p).unwrap();
    assert_eq!(v["goals"][0]["id"], "g1");
    assert_eq!(
        v["goals"][0]["key"],
        json!({ "kind": "item", "itemKind": "passive", "id": 2 })
    );
    assert_eq!(v["goals"][0]["target"]["kind"], "item");
    assert_eq!(v["goals"][0]["target"]["name"], "A");
    assert!(v["goals"][0]["target"]["iconUrl"]
        .as_str()
        .unwrap()
        .starts_with("data:image/png"));
    assert_eq!(v["goals"][0]["createdUnix"], 0);
    assert_eq!(v["goals"][0]["note"], Value::Null);
    // The key comes back from the resolved view: the two can never diverge.
    assert_eq!(
        p.goals[0].target.as_ref().unwrap().key(),
        p.goals[0].key,
        "`key()` and `resolve_target` are each other's inverse"
    );
}

/// Without a catalog the goal stays visible, without a name, and the view declares it
/// once: this is the "game not installed" case, not "goal vanished".
#[test]
fn without_a_catalog_no_goal_resolves_and_one_diagnostic_says_it() {
    let p = plan_view(None, vec![goal("a"), goal("b")], vec![], None, |_| None);
    assert_eq!(p.goals.len(), 2);
    assert!(p.goals.iter().all(|g| g.target.is_none()));
    assert_eq!(
        p.goals[0].key,
        ipc::TargetKey::Item {
            item_kind: ItemKindView::Passive,
            id: 2
        },
        "the key is always there: it's what's needed to remove it"
    );
    assert_eq!(p.diagnostics, vec![PlanDiagnostic::NoCatalog]);
    assert!(
        p.store_available,
        "the database opened fine: it's the game that's missing"
    );
    let v = to_value(&p).unwrap();
    assert_eq!(v["goals"][0]["target"], Value::Null);
    assert_eq!(v["diagnostics"], json!([{ "kind": "noCatalog" }]));
}

/// Catalog present but the key no longer exists (an id removed by a patch): the goal
/// stays and is named by id, like an unreadable row.
#[test]
fn a_key_the_catalog_does_not_know_is_named_not_dropped() {
    let c = catalog_with_achievements();
    let mut g = goal("g9");
    g.target = ipc::TargetKey::Item {
        item_kind: ItemKindView::Passive,
        id: 999_999,
    };
    let p = plan_view(Some(&c), vec![g], vec![], None, |_| None);
    assert_eq!(p.goals.len(), 1);
    assert!(p.goals[0].target.is_none());
    assert_eq!(
        p.diagnostics,
        vec![PlanDiagnostic::UnresolvedGoal {
            id: ipc::GoalId::from_str_unchecked("g9")
        }]
    );
    assert_eq!(
        to_value(&p).unwrap()["diagnostics"],
        json!([{ "kind": "unresolvedGoal", "id": "g9" }])
    );
}

/// The shape of M3, pinned before M3 exists: when `expansion` stops being `stub` the
/// frontend will receive this, and the components drawn today don't change.
#[test]
fn the_computed_plan_expansion_and_its_steps_are_pinned() {
    let e = PlanExpansion::Computed {
        steps: vec![ipc::PlanStep {
            goal: ipc::GoalId::from_str_unchecked("g1"),
            node: node(false),
            done: false,
        }],
    };
    let v = to_value(&e).unwrap();
    assert_eq!(v["kind"], "computed");
    assert_eq!(
        v["steps"][0]["goal"], "g1",
        "the step cites the goal by id, it doesn't copy it"
    );
    assert_eq!(v["steps"][0]["done"], false);
    assert_eq!(v["steps"][0]["node"]["achievement"]["kind"], "known");
    assert_eq!(v["steps"][0]["node"]["graph"], json!({ "kind": "stub" }));
    assert_eq!(
        v["steps"][0].as_object().unwrap().len(),
        3,
        "goal, node, done"
    );
}

/// A challenge among an achievement's `unlocks` carries its reward: the ids of the
/// achievements earned by completing it, read from the catalog. Without a known
/// reward, the vector is empty, not `null`.
#[test]
fn a_challenge_target_carries_the_achievements_it_rewards() {
    let ach: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\">
<achievement id=\"1\" text=\"You unlocked Challenge #19\" gfx=\"a.png\" steam_description=\"Unlocked a new challenge.\" />
<!-- Beat Challenge #19 --><achievement id=\"2\" text=\"Epic Fetus\" gfx=\"b.png\" />
<achievement id=\"3\" text=\"You unlocked Challenge #36\" gfx=\"c.png\" />
</achievements>";
    let ch: &[u8] = b"<challenges><challenge name=\"The Family Man\" id=\"19\" achievements=\"1\" /><challenge name=\"Scat Man\" id=\"36\" achievements=\"3\" /></challenges>";
    let c = Catalog::build(|p| match p {
        "achievements.xml" => Some(ach.to_vec()),
        "challenges.xml" => Some(ch.to_vec()),
        _ => None,
    });
    let v = unlock_view(Some(&c), Some(&[false, false, false, false]), |_| None);
    assert_eq!(
        v.nodes[0].unlocks,
        vec![UnlockTarget::Challenge {
            id: 19,
            name: "The Family Man".into(),
            rewards: vec![2],
        }]
    );
    assert_eq!(
        v.nodes[2].unlocks,
        vec![UnlockTarget::Challenge {
            id: 36,
            name: "Scat Man".into(),
            rewards: vec![],
        }]
    );
    let json = serde_json::to_value(&v.nodes[2].unlocks[0]).unwrap();
    assert_eq!(json["rewards"], serde_json::json!([]));
}
