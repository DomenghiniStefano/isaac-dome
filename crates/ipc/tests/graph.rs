use catalog::Catalog;
use ipc::{
    next_steps, plan_view, unlock_view, AchievementRef, GraphInfo, IconRef, ItemKindView,
    NextSteps, OriginView, PlanDiagnostic, PlanExpansion, RequirementView, StepsBasis,
    StepsSection, Target, UnlockDiagnostic, UnlockNode, UnlockTarget, UnlockTotals, UnlockView,
    STEPS,
};
use serde_json::{json, to_value, Value};

fn node(done: bool) -> UnlockNode {
    UnlockNode {
        achievement: AchievementRef::Known {
            id: 1,
            text: "You unlocked \"Magdalene\"".into(),
            condition: Some("have 7 or more max red hearts at one time".into()),
            icon_url: None,
        },
        done,
        unlocks: vec![UnlockTarget::Character {
            id: 1,
            name: "Magdalene".into(),
            tainted: false,
            page: None,
        }],
        origin: None,
        missing: Vec::new(),
        graph: GraphInfo::Computed {
            available_now: false,
            blocked_by: 0,
            fan_out: 0,
            steps_missing: 0,
        },
    }
}

#[test]
fn unlock_node_json_shape_is_pinned() {
    let v = to_value(node(true)).unwrap();
    assert_eq!(v["achievement"]["kind"], "known");
    assert_eq!(v["achievement"]["id"], 1);
    assert_eq!(
        v["achievement"]["condition"],
        "have 7 or more max red hearts at one time"
    );
    assert_eq!(v["achievement"]["iconUrl"], Value::Null);
    assert_eq!(v["done"], true);
    assert_eq!(v["unlocks"][0]["kind"], "character");
    assert_eq!(v["origin"], Value::Null);
    assert_eq!(
        v["graph"],
        json!({
            "kind": "computed", "availableNow": false, "blockedBy": 0,
            "fanOut": 0, "stepsMissing": 0
        })
    );
    assert_eq!(v["missing"], json!([]));
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
        page: Some(Target::Trinket { id: 1 }),
    };
    let v = to_value(&t).unwrap();
    assert_eq!(v["kind"], "item");
    // `itemKind` is a string: the enum has no fields, so it isn't tagged.
    assert_eq!(v["itemKind"], "trinket");
    assert_eq!(v["page"], json!({ "kind": "trinket", "id": 1 }));
    assert_eq!(
        v.as_object().unwrap().len(),
        6,
        "kind, itemKind, id, name, iconUrl, page: no key overwritten"
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

    // `StepsBasis` has no fields: it's a bare string, like `itemKind` and `origin`.
    let steps = to_value(NextSteps {
        sections: vec![
            StepsSection {
                basis: StepsBasis::FanOut,
                steps: vec![],
            },
            StepsSection {
                basis: StepsBasis::Closeness,
                steps: vec![],
            },
        ],
    })
    .unwrap();
    assert_eq!(steps["sections"][0]["basis"], "fanOut");
    assert_eq!(steps["sections"][1]["basis"], "closeness");
    assert_eq!(STEPS, 5);

    // The plan is only ever built via `plan_view`: `storeAvailable` and the store
    // diagnostic are born from the same argument and can never contradict each other.
    let c = catalog_with_achievements();
    let v = to_value(plan_view(Some(&c), None, vec![], vec![], None, |_| None)).unwrap();
    assert_eq!(v["expansion"], json!({ "kind": "stub" }));
    assert_eq!(v["diagnostics"], json!([]));
    assert_eq!(v["storeAvailable"], true);

    let v = to_value(plan_view(
        Some(&c),
        None,
        vec![],
        vec![],
        Some(ipc::StoreReason::NewerSchema {
            found: 7,
            supported: 1,
        }),
        |_| None,
    ))
    .unwrap();
    assert_eq!(v["storeAvailable"], false);
    assert_eq!(
        v["diagnostics"],
        json!([{
            "kind": "storeUnavailable",
            "reason": { "kind": "newerSchema", "found": 7, "supported": 1 },
        }])
    );

    // A database row that fails to read: the UI receives the id, not the broken JSON.
    let v = to_value(plan_view(
        Some(&c),
        None,
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
    for reason in [None, Some(ipc::StoreReason::Unreadable)] {
        let p = plan_view(Some(&c), None, vec![], vec![], reason, |_| None);
        let says_unavailable = p
            .diagnostics
            .iter()
            .any(|d| matches!(d, PlanDiagnostic::StoreUnavailable { .. }));
        assert_eq!(p.store_available, reason.is_none());
        assert_eq!(says_unavailable, reason.is_some());
    }
}

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
    let v = unlock_view(
        Some(&catalog_with_achievements()),
        None,
        Some(&flags),
        None,
        None,
        None,
        |_| None,
    );
    assert_eq!(v.nodes.len(), 5, "one per slot 1..=5");
    assert!(
        matches!(&v.nodes[0].achievement, AchievementRef::Known { id: 1, condition: Some(h), .. } if h == "c1")
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
    assert!(
        v.nodes.iter().all(|n| n.graph
            == GraphInfo::Partial {
                blocked_by: 0,
                fan_out: 0,
                unknown: 1
            }),
        "no graph was passed: every node says so, and none claims to be computed"
    );
    assert!(v.nodes.iter().all(|n| n.missing.is_empty()));
}

#[test]
fn unlocks_and_origin_come_from_the_catalog_and_icons_only_when_they_resolve() {
    let flags = [false, false, false, false];
    let v = unlock_view(
        Some(&catalog_with_achievements()),
        None,
        Some(&flags),
        None,
        None,
        None,
        // Only one reference resolves. The row for the other one still has to exist, with
        // `iconUrl: null`: an item we can't picture is not an item we hide.
        |r| {
            matches!(
                r,
                IconRef::Item {
                    kind: ItemKindView::Passive,
                    id: 2
                }
            )
            .then(|| "isaac://item/passive/2".to_string())
        },
    );
    let n1 = &v.nodes[0];
    assert_eq!(n1.unlocks.len(), 1);
    assert!(
        matches!(&n1.unlocks[0], UnlockTarget::Item { item_kind: ItemKindView::Passive, id: 2, name, icon_url: Some(u), .. } if name == "A" && u == "isaac://item/passive/2")
    );
    assert_eq!(
        n1.origin,
        Some(OriginView::Rebirth),
        "item 2 is from Rebirth"
    );
    let n2 = &v.nodes[1];
    assert!(
        matches!(&n2.unlocks[0], UnlockTarget::Character { id: 7, name, tainted: false, .. } if name == "Z_NAME")
    );
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
        None,
        Some(&[false, true]),
        None,
        None,
        None,
        |_| None,
    );
    assert_eq!(v.nodes.len(), 1);
    assert_eq!(
        v.diagnostics,
        vec![UnlockDiagnostic::CatalogBeyondSlots { count: 2 }]
    );

    let v = unlock_view(
        None,
        None,
        Some(&[false, true, true]),
        None,
        None,
        None,
        |_| None,
    );
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
    let v = unlock_view(
        Some(&catalog_with_achievements()),
        None,
        None,
        None,
        None,
        None,
        |_| None,
    );
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
    let v = unlock_view(None, None, None, None, None, None, |_| None);
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
    let v = unlock_view(
        Some(&catalog_with_achievements()),
        None,
        Some(&[]),
        None,
        None,
        None,
        |_| None,
    );
    assert!(v.nodes.is_empty());
    assert_eq!(v.totals.slots, 0);
    assert_eq!(
        v.diagnostics,
        vec![UnlockDiagnostic::CatalogBeyondSlots { count: 4 }],
        "3 achievements in the catalog plus slot 0"
    );
}

#[test]
fn without_a_graph_there_are_no_next_steps_to_suggest() {
    let mut flags = vec![false; 10];
    flags[2] = true;
    flags[5] = true;
    let v = unlock_view(None, None, Some(&flags), None, None, None, |_| None);
    assert!(
        next_steps(&v, &Default::default()).sections.is_empty(),
        "not-done is not the same as unlockable: with no graph the app has nothing to \
         recommend, and the view's NoCatalog diagnostic is what says why"
    );
}

#[test]
fn next_steps_take_what_is_unlockable_now_most_fan_out_first() {
    let computed = |available_now: bool, fan_out: u32| GraphInfo::Computed {
        available_now,
        blocked_by: if available_now { 0 } else { 1 },
        fan_out,
        steps_missing: if available_now { 0 } else { 1 },
    };
    let mut nodes = Vec::new();
    for (slot, info) in [
        (1u32, computed(true, 2)),
        (2, computed(false, 9)), // blocked: not a step, however much it opens
        (3, computed(true, 7)),
        (
            4,
            GraphInfo::Partial {
                blocked_by: 0,
                fan_out: 9,
                unknown: 1,
            },
        ), // can't say
        (5, computed(true, 7)),
    ] {
        let mut n = node(false);
        n.achievement = AchievementRef::Unknown { slot };
        n.graph = info;
        nodes.push(n);
    }
    let v = UnlockView {
        nodes,
        totals: UnlockTotals {
            slots: 6,
            done: 0,
            known: 0,
            unknown: 5,
        },
        diagnostics: vec![],
    };
    let s = next_steps(&v, &Default::default());
    assert_eq!(
        slots_of(&s, StepsBasis::FanOut),
        vec![3, 5, 1],
        "fan-out descending, ties by id ascending; the blocked and the partial stay out"
    );
    assert!(slots_of(&s, StepsBasis::FanOut).len() <= STEPS);
    assert!(
        slots_of(&s, StepsBasis::Closeness).is_empty(),
        "nothing here is missing a counter, so there is no closeness section at all"
    );
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
    let p = plan_view(
        Some(&c),
        None,
        vec![goal("b"), goal("a")],
        vec![],
        None,
        |_| None,
    );
    assert_eq!(
        p.goals.iter().map(|g| g.id.as_str()).collect::<Vec<_>>(),
        vec!["b", "a"]
    );
    assert_eq!(p.expansion, PlanExpansion::Stub);
    assert!(p.store_available);
    assert!(p.diagnostics.is_empty());
    assert!(
        !plan_view(
            Some(&c),
            None,
            vec![],
            vec![],
            Some(ipc::StoreReason::Unreadable),
            |_| None
        )
        .store_available
    );
    // Unreadable ids become diagnostics, one per row, in the order received.
    let bad = |s: &str| ipc::GoalId::from_str_unchecked(s);
    let p = plan_view(
        Some(&c),
        None,
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
    let p = plan_view(Some(&c), None, vec![goal("g1")], vec![], None, |r| {
        Some(format!("{}://{}", ipc::ICON_SCHEME, r.to_path()))
    });
    let v = to_value(&p).unwrap();
    assert_eq!(v["goals"][0]["id"], "g1");
    assert_eq!(
        v["goals"][0]["key"],
        json!({ "kind": "item", "itemKind": "passive", "id": 2 })
    );
    assert_eq!(v["goals"][0]["target"]["kind"], "item");
    assert_eq!(v["goals"][0]["target"]["name"], "A");
    assert_eq!(
        v["goals"][0]["target"]["iconUrl"], "isaac://item/passive/2",
        "the goal's icon is a link resolved now, not a picture stored then"
    );
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
    let p = plan_view(None, None, vec![goal("a"), goal("b")], vec![], None, |_| {
        None
    });
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
    let p = plan_view(Some(&c), None, vec![g], vec![], None, |_| None);
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
    // A plan step carries a whole node, graph info included: after M2 that is real, and
    // `stub` is gone from the wire. The plan's own `expansion` is still stubbed — that's
    // M3, and it is a different field.
    assert_eq!(v["steps"][0]["node"]["graph"]["kind"], "computed");
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
    let v = unlock_view(
        Some(&c),
        None,
        Some(&[false, false, false, false]),
        None,
        None,
        None,
        |_| None,
    );
    assert_eq!(
        v.nodes[0].unlocks,
        vec![UnlockTarget::Challenge {
            id: 19,
            name: "The Family Man".into(),
            rewards: vec![2],
            page: None,
        }]
    );
    assert_eq!(
        v.nodes[2].unlocks,
        vec![UnlockTarget::Challenge {
            id: 36,
            name: "Scat Man".into(),
            rewards: vec![],
            page: None,
        }]
    );
    let json = serde_json::to_value(&v.nodes[2].unlocks[0]).unwrap();
    assert_eq!(json["rewards"], serde_json::json!([]));
}

// --- M2: the graph is real, and two shapes are new on the wire ---

#[test]
fn requirement_view_shapes() {
    use ipc::RequirementView;
    assert_eq!(
        to_value(RequirementView::Character {
            id: 1,
            name: "Magdalene".into(),
            tainted: false,
            page: Some(Target::Character { id: 1 }),
        })
        .unwrap(),
        json!({
            "kind": "character", "id": 1, "name": "Magdalene", "tainted": false,
            "page": { "kind": "character", "id": 1 }
        }),
        "the page is what the screen opens to read how *this* is unlocked"
    );
    assert_eq!(
        to_value(RequirementView::Item {
            item_kind: ItemKindView::Passive,
            id: 35,
            name: "The Bible".into(),
            page: None,
        })
        .unwrap(),
        json!({
            "kind": "item", "itemKind": "passive", "id": 35, "name": "The Bible",
            "page": null
        }),
        "`kind` is the tag: the item's own kind is `itemKind`, and a fieldless enum is a \
         bare string. `page: null` is 'the dataset has no page', never 'no requirement'"
    );
    assert_eq!(
        to_value(RequirementView::Gate {
            label: "The Void".into()
        })
        .unwrap(),
        json!({ "kind": "gate", "label": "The Void" }),
        "a gate is a curated condition, not an entity: it carries no page key at all"
    );
    assert_eq!(
        to_value(RequirementView::Unknown {
            label: "Guppy".into()
        })
        .unwrap(),
        json!({ "kind": "unknown", "label": "Guppy" })
    );
}

#[test]
fn graph_info_shapes() {
    assert_eq!(
        to_value(GraphInfo::Computed {
            available_now: true,
            blocked_by: 0,
            fan_out: 3,
            steps_missing: 0
        })
        .unwrap(),
        json!({
            "kind": "computed", "availableNow": true, "blockedBy": 0,
            "fanOut": 3, "stepsMissing": 0
        })
    );
    assert_eq!(
        to_value(GraphInfo::Partial {
            blocked_by: 1,
            fan_out: 2,
            unknown: 3
        })
        .unwrap(),
        json!({ "kind": "partial", "blockedBy": 1, "fanOut": 2, "unknown": 3 }),
        "rename_all_fields is what keeps blockedBy from arriving as blocked_by; and \
         `partial` carries no stepsMissing on purpose"
    );
}

#[test]
fn a_node_carries_what_it_is_missing_typed() {
    use ipc::RequirementView;
    let mut n = node(false);
    n.missing = vec![
        RequirementView::Character {
            id: 1,
            name: "Magdalene".into(),
            tainted: false,
            page: None,
        },
        RequirementView::Boss {
            id: 19,
            name: "Gish".into(),
            page: None,
        },
    ];
    let v = to_value(&n).unwrap();
    assert_eq!(v["missing"][0]["kind"], "character");
    assert_eq!(v["missing"][1]["kind"], "boss");
    assert_eq!(
        v["missing"].as_array().map(Vec::len),
        Some(2),
        "the screen groups by these: 'you're missing 1 character and 1 boss'"
    );
}

/// The pair, not the name: `players.xml` gives the two forms of a character the same name
/// key, so a target that carried only the name would send two different characters out
/// under one label (`docs/BACKLOG.md` B28).
#[test]
fn the_tainted_form_travels_as_a_flag_beside_the_shared_name() {
    use ipc::{resolve_target, TargetKey};
    // Two players with the same name key, told apart by the `b` in the portrait.
    const PAIR: &[u8] = b"<players portraitroot=\"gfx/ui/stage/\"><player id=\"10\" name=\"#THE_LOST_NAME\" portrait=\"PlayerPortrait_TheLost.png\" achievement=\"82\" /><player id=\"31\" name=\"#THE_LOST_NAME\" portrait=\"PlayerPortrait_TheLost_b.png\" achievement=\"484\" /></players>";
    let c = Catalog::build(|p| match p {
        "players.xml" => Some(PAIR.to_vec()),
        _ => None,
    });
    let mut icon = |_: &ipc::IconRef| None;
    let base =
        resolve_target(&c, &TargetKey::Character { id: 10 }, None, &mut icon).expect("player 10");
    let tainted =
        resolve_target(&c, &TargetKey::Character { id: 31 }, None, &mut icon).expect("player 31");
    let v = to_value(&tainted).unwrap();
    assert_eq!(v["kind"], "character");
    assert_eq!(v["tainted"], true);
    assert_eq!(
        v["name"],
        to_value(&base).unwrap()["name"],
        "the two forms share the game's name: the flag is what separates them"
    );
    assert_eq!(to_value(&base).unwrap()["tainted"], false);
}

/// A target's page follows the same rule as a requirement's: `Some` only when the dataset
/// really has the entry. With no dataset there is no page to carry, and the row still names
/// what it names — a missing page never removes a target.
#[test]
fn without_a_dataset_no_target_carries_a_page() {
    let c = catalog_with_achievements();
    let flags = [false, true, true, true];
    let v = unlock_view(Some(&c), None, Some(&flags), None, None, None, |_| None);
    let pages: Vec<Option<&Target>> = v
        .nodes
        .iter()
        .flat_map(|n| n.unlocks.iter())
        .map(|t| match t {
            UnlockTarget::Item { page, .. }
            | UnlockTarget::Character { page, .. }
            | UnlockTarget::Boss { page, .. }
            | UnlockTarget::Challenge { page, .. } => page.as_ref(),
        })
        .collect();
    assert!(
        !pages.is_empty(),
        "the fixture catalog has to produce targets, or this test asserts nothing"
    );
    assert!(pages.iter().all(Option::is_none));
}

/// A node whose every standing requirement is a counter is not blocked: the content is
/// reachable and only has to be played. It belongs to the section that can order it — a
/// counter carries a distance, and the fan-out does not.
#[test]
fn a_node_held_only_by_counters_goes_to_the_closeness_section() {
    let counter = |current: u32, at_least: u32| RequirementView::Counter {
        label: "Mom's Heart".into(),
        current,
        at_least,
    };
    let available = |slot: u32, fan_out: u32, missing: Vec<RequirementView>| UnlockNode {
        achievement: AchievementRef::Unknown { slot },
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
        available(1, 9, vec![]),               // nothing in the way: fan-out
        available(2, 1, vec![counter(9, 11)]), // two to go
        available(3, 1, vec![counter(3, 11)]), // eight to go
    ]);

    let s = next_steps(&v, &Default::default());
    let bases: Vec<StepsBasis> = s.sections.iter().map(|x| x.basis).collect();
    assert_eq!(bases, vec![StepsBasis::FanOut, StepsBasis::Closeness]);
    assert_eq!(slots_of(&s, StepsBasis::FanOut), vec![1]);
    assert_eq!(
        slots_of(&s, StepsBasis::Closeness),
        vec![2, 3],
        "nearest the threshold first"
    );
}

/// The slots a section names, in its own order.
fn slots_of(s: &NextSteps, basis: StepsBasis) -> Vec<u32> {
    s.sections
        .iter()
        .find(|x| x.basis == basis)
        .map(|x| {
            x.steps
                .iter()
                .map(|n| match n.achievement {
                    AchievementRef::Known { id, .. } => id,
                    AchievementRef::Unknown { slot } => slot,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// A node is a suggestion once, or it reads as two different suggestions.
#[test]
fn the_two_sections_never_name_the_same_node() {
    let v = ipc::for_tests::unlock_view_of(vec![UnlockNode {
        achievement: AchievementRef::Unknown { slot: 4 },
        done: false,
        unlocks: vec![],
        origin: None,
        missing: vec![RequirementView::Counter {
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
    let s = next_steps(&v, &Default::default());
    assert_eq!(s.sections.len(), 1, "one node cannot fill two sections");
    assert_eq!(
        s.sections[0].basis,
        StepsBasis::Closeness,
        "closeness says more about it than its fan-out does, however large"
    );
}

/// A heading over nothing is not a state the screen should have to handle: "absent" is
/// decided once, here.
#[test]
fn a_section_with_no_steps_is_not_emitted() {
    let mut flags = vec![false; 10];
    flags[2] = true;
    let v = unlock_view(None, None, Some(&flags), None, None, None, |_| None);
    assert!(next_steps(&v, &Default::default()).sections.is_empty());
}

/// A node with a mark still standing is `available_now` too, but a mark is binary: there is
/// no distance to be near, so it is ordered by what it opens and not by how close it is.
#[test]
fn a_mark_is_not_a_distance_and_stays_in_the_fan_out_section() {
    let v = ipc::for_tests::unlock_view_of(vec![UnlockNode {
        achievement: AchievementRef::Unknown { slot: 7 },
        done: false,
        unlocks: vec![],
        origin: None,
        missing: vec![RequirementView::Mark {
            character: 0,
            character_name: "Isaac".into(),
            column: ipc::MarkColumnView::MomsHeart,
            level: ipc::MarkLevelView::Base,
        }],
        graph: GraphInfo::Computed {
            available_now: true,
            blocked_by: 0,
            fan_out: 2,
            steps_missing: 0,
        },
    }]);
    let s = next_steps(&v, &Default::default());
    assert_eq!(slots_of(&s, StepsBasis::FanOut), vec![7]);
    assert!(slots_of(&s, StepsBasis::Closeness).is_empty());
}

/// `rename_all` on the enum renames the variants, not the fields inside them. Without
/// `rename_all_fields` the wire would say `at_least` and `item_kind`, TypeScript would read
/// `undefined`, and nothing would fail — the silent shape bug this repo has already paid
/// for. Pinned on the JSON, like every other variant here.
#[test]
fn the_threshold_view_is_camel_case_on_the_wire() {
    use ipc::{ItemKindView, RequirementView, ThresholdItemView};
    let v = RequirementView::Threshold {
        transformation: 0,
        label: "Guppy".into(),
        current: 2,
        at_least: 3,
        of: vec![ThresholdItemView {
            item_kind: ItemKindView::Passive,
            id: 211,
            name: "Guppy's Head".into(),
            unlocked: true,
            page: None,
        }],
        unresolved: 0,
        page: None,
    };
    assert_eq!(
        serde_json::to_value(&v).unwrap(),
        json!({
            "kind": "threshold",
            "transformation": 0,
            "label": "Guppy",
            "current": 2,
            "atLeast": 3,
            "of": [{
                "itemKind": "passive",
                "id": 211,
                "name": "Guppy's Head",
                "unlocked": true,
                "page": null
            }],
            "unresolved": 0,
            "page": null
        })
    );
}

/// N8: the pair the graph's screens are loaded with. The property is not that a struct holds
/// two fields — it is that the steps are the filter over **the view beside them**, from one
/// reading of the profile. Two commands could not say that: each rebuilt the pipeline, and a
/// save written between them made the steps describe a profile the list no longer showed.
#[test]
fn the_pair_carries_the_steps_of_the_view_it_travels_with() {
    let mut flags = vec![false; 10];
    flags[2] = true;
    let view = unlock_view(None, None, Some(&flags), None, None, None, |_| None);
    let pair = ipc::graph_views(view.clone(), &Default::default());
    assert_eq!(pair.unlock, view);
    assert_eq!(pair.steps, next_steps(&view, &Default::default()));
}

/// A known achievement that is available now, with the fan-out it would open and whatever
/// still stands in its way.
fn known_available(id: u32, fan_out: u32, missing: Vec<RequirementView>) -> UnlockNode {
    UnlockNode {
        achievement: AchievementRef::Known {
            id,
            text: format!("achievement {id}"),
            condition: None,
            icon_url: None,
        },
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
    }
}

/// A suggestion is something to add. One already in the queue is a decision taken, and
/// suggesting it again spends a place under the cap on it: the next candidate takes that
/// place instead, so the section stays as long as the profile allows.
#[test]
fn what_is_queued_is_not_suggested_and_its_place_goes_to_the_next() {
    let count = STEPS as u32 + 1;
    let nodes: Vec<UnlockNode> = (1..=count)
        .map(|id| known_available(id, 100 - id, vec![]))
        .collect();
    let v = ipc::for_tests::unlock_view_of(nodes);
    assert_eq!(
        slots_of(&next_steps(&v, &Default::default()), StepsBasis::FanOut).len(),
        STEPS,
        "the fixture has to overflow the cap, or the refill is not tested"
    );

    let queued = std::collections::BTreeSet::from([1]);
    let s = next_steps(&v, &queued);
    let expected: Vec<u32> = (2..=count).collect();
    assert_eq!(slots_of(&s, StepsBasis::FanOut), expected);
}

/// Queued and near is still queued: closeness does not hand it to the other section either.
#[test]
fn a_queued_node_is_absent_from_both_sections() {
    let near = RequirementView::Counter {
        label: "Mom's Heart".into(),
        current: 9,
        at_least: 11,
    };
    let v = ipc::for_tests::unlock_view_of(vec![
        known_available(1, 9, vec![near]),
        known_available(2, 3, vec![]),
    ]);
    let s = next_steps(&v, &std::collections::BTreeSet::from([1]));
    assert_eq!(slots_of(&s, StepsBasis::FanOut), vec![2]);
    assert!(slots_of(&s, StepsBasis::Closeness).is_empty());
}
