//! Against the real installation files (samples/packed is a junction to the game's
//! folder). They skip and say so if it's missing. The numbers are Repentance+'s, read
//! on 2026-09-03: a fixture of a known era, not constants of the product.

use catalog::{
    AchievementId, Catalog, Challenge, ChallengeId, CharacterId, Diagnostic, ItemId, ItemKind,
    Language, Origin, Text, Unlock,
};
use unpack::{Archive, ResourceSet};

fn build_or_skip() -> Option<(Catalog, ResourceSet)> {
    let rs = ResourceSet::open(&test_support::packed_dir()?);
    let c = Catalog::build(|p| rs.read(p));
    Some((c, rs))
}

#[test]
fn item_counts_by_kind_match_the_repentance_file() {
    let Some((c, _)) = build_or_skip() else {
        return;
    };
    let n = |k: ItemKind| c.items().filter(|i| i.kind == k).count();
    // items.xml from repentance.a, measured with quick-xml on 2026-09-03: 909 elements.
    // A text grep gave 911, because it counted two commented-out elements in the file
    // (PILLS_HERE_NAME and TAROT_CARD_NAME, lines 44 and 62): quick-xml ignores them,
    // correctly.
    assert_eq!(
        (
            n(ItemKind::Passive),
            n(ItemKind::Active),
            n(ItemKind::Familiar),
            n(ItemKind::Trinket)
        ),
        (425, 170, 126, 188)
    );
}

#[test]
fn all_item_names_resolve() {
    let Some((c, _)) = build_or_skip() else {
        return;
    };
    let unresolved: Vec<String> = c
        .items()
        .filter_map(|i| match &i.name {
            Text::Key { key } if c.text(&i.name, Language::English) == key => Some(key.clone()),
            Text::Key { .. } | Text::Literal { .. } => None,
        })
        .collect();
    // The two "placeholders" (PILLS_HERE_NAME, TAROT_CARD_NAME) are inside an XML
    // comment in items.xml (lines 44 and 62): they aren't elements, quick-xml ignores
    // them, so they never end up in items() and there are no unresolved names.
    assert!(unresolved.is_empty(), "{unresolved:?}");
    let onion = c
        .items()
        .find(|i| i.id.0 == 1 && i.kind == ItemKind::Passive)
        .unwrap();
    assert_eq!(c.text(&onion.name, Language::English), "The Sad Onion");
    assert_eq!(
        c.text(&onion.name, Language::German),
        "Die traurige Zwiebel"
    );
}

#[test]
fn every_sprite_path_resolves_in_the_archives() {
    let Some((c, rs)) = build_or_skip() else {
        return;
    };
    let missing: Vec<&str> = c
        .items()
        .map(|i| i.sprite.path.as_str())
        .chain(c.characters().map(|ch| ch.portrait.path.as_str()))
        .chain(
            c.characters()
                .filter_map(|ch| ch.head.as_ref())
                .map(|h| h.path.as_str()),
        )
        .filter(|p| !rs.contains(p))
        .collect();
    assert!(missing.is_empty(), "unresolvable sprites: {missing:?}");
}

#[test]
fn forty_one_characters_twenty_of_them_tainted_with_ids_21_to_40() {
    let Some((c, _)) = build_or_skip() else {
        return;
    };
    assert_eq!(c.characters().count(), 41);
    let tainted: Vec<u32> = c
        .characters()
        .filter(|ch| ch.tainted)
        .map(|ch| ch.id.0)
        .collect();
    assert_eq!(
        tainted,
        (21..=40).collect::<Vec<u32>>(),
        "the _b token rule and the id convention must agree"
    );
}

#[test]
fn thirty_seven_characters_have_a_head_cell_inside_the_sheet() {
    let Some((c, _)) = build_or_skip() else {
        return;
    };
    let with_head: Vec<u32> = c
        .characters()
        .filter(|ch| ch.head.is_some())
        .map(|ch| ch.id.0)
        .collect();
    assert_eq!(with_head.len(), 37);
    let without_head: Vec<u32> = c
        .characters()
        .filter(|ch| ch.head.is_none())
        .map(|ch| ch.id.0)
        .collect();
    let mut without_head_sorted = without_head.clone();
    without_head_sorted.sort_unstable();
    assert_eq!(without_head_sorted, vec![20, 38, 39, 40]);

    for ch in c.characters().filter(|ch| ch.head.is_some()) {
        let head = ch.head.as_ref().unwrap();
        let rect = head.rect.expect("the head is always a crop");
        assert_eq!(rect.w, 32, "character {}", ch.id.0);
        assert_eq!(rect.h, 32, "character {}", ch.id.0);
        assert!(
            rect.x + rect.w <= 192,
            "character {}: x={} w={} outside the 192x224 sheet",
            ch.id.0,
            rect.x,
            rect.w
        );
        assert!(
            rect.y + rect.h <= 224,
            "character {}: y={} h={} outside the 192x224 sheet",
            ch.id.0,
            rect.y,
            rect.h
        );
    }
}

#[test]
fn languages_are_the_eight_declared_and_none_is_italian() {
    let Some((c, _)) = build_or_skip() else {
        return;
    };
    assert_eq!(c.languages().len(), 8);
    assert!(!c
        .diagnostics()
        .iter()
        .any(|d| matches!(d, Diagnostic::UnknownLanguage { .. })));
}

#[test]
fn no_source_is_missing_or_unreadable_on_a_real_installation() {
    let Some((c, _)) = build_or_skip() else {
        return;
    };
    let bad: Vec<&Diagnostic> = c
        .diagnostics()
        .iter()
        .filter(|d| {
            matches!(
                d,
                Diagnostic::SourceMissing { .. } | Diagnostic::SourceUnreadable { .. }
            )
        })
        .collect();
    assert!(bad.is_empty(), "{bad:?}");
}

// --- Plan B: the new sources. Numbers measured on 2026-09-04 against the Repentance+
// files; four of them corrected on 2026-09-05 by the controller after the first draft's
// red run (see task-7-report.md, "Chiusura" section). ---

#[test]
fn every_item_has_quality_and_the_distribution_matches_the_metadata_file() {
    let Some((c, _)) = build_or_skip() else {
        return;
    };
    let mut by_quality = std::collections::BTreeMap::new();
    for i in c.items() {
        *by_quality.entry(i.quality).or_insert(0usize) += 1;
    }
    // items_metadata.xml: no item has quality="-1". The two textual occurrences of
    // "quality=\"-1\"" are actually the craftquality="-1" attribute (id 422 and 710): a
    // text grep had confused them with quality, quick-xml did not. Real distribution:
    // 0:239, 1:197, 2:249, 3:184, 4:40, sum 909.
    assert_eq!(by_quality.get(&None), None, "every item has a quality");
    assert_eq!(by_quality.get(&Some(-1)), None, "no item has quality -1");
    assert_eq!(by_quality[&Some(0)], 239);
    assert_eq!(by_quality[&Some(1)], 197);
    assert_eq!(by_quality[&Some(2)], 249);
    assert_eq!(by_quality[&Some(3)], 184);
    assert_eq!(by_quality[&Some(4)], 40);
    assert_eq!(by_quality.values().sum::<usize>(), 909);
}

#[test]
fn achievements_are_637_contiguous_and_283_carry_a_condition() {
    let Some((c, _)) = build_or_skip() else {
        return;
    };
    let ids: Vec<u32> = c.achievements().map(|a| a.id.0).collect();
    assert_eq!(ids, (1..=637).collect::<Vec<u32>>());
    assert_eq!(
        c.achievements()
            .filter(|a| a.unlock_condition.is_some())
            .count(),
        283
    );
    let first = c.achievement(AchievementId(1)).unwrap();
    assert_eq!(first.text, "You unlocked \"Magdalene\"");
    assert_eq!(
        first.unlock_condition.as_deref(),
        Some("have 7 or more max red hearts at one time")
    );
}

#[test]
fn pools_are_31_every_entry_is_a_known_item_and_24_items_are_in_none() {
    let Some((c, _)) = build_or_skip() else {
        return;
    };
    assert_eq!(c.pools().len(), 31);
    assert_eq!(
        c.pools().iter().map(|p| p.entries.len()).sum::<usize>(),
        2058
    );
    let in_no_pool = c
        .items()
        .filter(|i| ItemKind::COLLECTIBLES.contains(&i.kind))
        .filter(|i| i.pools.is_empty())
        .count();
    // 26 on the first pass, measured with a text grep on items.xml: it was also
    // counting the two commented-out elements id 43 and 61 (PILLS_HERE_NAME and
    // TAROT_CARD_NAME, see item_counts_by_kind_match_the_repentance_file). The catalog,
    // which ignores those two in items() and so doesn't have 43/61 among the
    // collectibles, finds 24.
    assert_eq!(in_no_pool, 24);
    for p in c.pools() {
        for e in &p.entries {
            let known = ItemKind::COLLECTIBLES
                .iter()
                .any(|&k| c.item(k, e.item).is_some());
            assert!(
                known,
                "pool {} cites item {} which the catalog doesn't have",
                p.name, e.item.0
            );
        }
    }
}

#[test]
fn challenges_are_45_and_their_achievement_lists_match_the_file() {
    let Some((c, _)) = build_or_skip() else {
        return;
    };
    let all: Vec<&Challenge> = c.challenges().collect();
    assert_eq!(all.len(), 45);
    assert_eq!(
        all.iter().filter(|ch| !ch.unlocked_by.is_empty()).count(),
        34
    );
    // 14, not 13: an earlier measurement counted only comma separators and missed challenge 44 (`490 415`).
    assert_eq!(all.iter().filter(|ch| ch.unlocked_by.len() > 1).count(), 14);
    let high_brow = all
        .iter()
        .find(|ch| ch.name == "High Brow")
        .expect("it's there");
    assert_eq!(
        high_brow.starting_items,
        vec![ItemId(209), ItemId(6), ItemId(236), ItemId(291)]
    );
}

#[test]
fn bosses_are_103_and_all_portraits_but_two_resolve() {
    let Some((c, rs)) = build_or_skip() else {
        return;
    };
    assert_eq!(c.bosses().count(), 103);
    assert_eq!(c.bosses().filter(|b| b.unlocked_by.is_some()).count(), 27);
    let missing: Vec<&str> = c
        .bosses()
        .filter(|b| !rs.contains(&b.portrait.path))
        .map(|b| b.name.as_str())
        .collect();
    assert_eq!(
        missing,
        vec!["The Beast", "Cadavra"],
        "the two portraits the archives don't have"
    );
}

#[test]
fn every_achievement_sprite_resolves_in_the_archives() {
    let Some((c, rs)) = build_or_skip() else {
        return;
    };
    let missing: Vec<u32> = c
        .achievements()
        .filter(|a| !rs.contains(&a.sprite.path))
        .map(|a| a.id.0)
        .collect();
    assert!(
        missing.is_empty(),
        "unresolvable achievement icons: {missing:?}"
    );
}

#[test]
fn origin_matches_the_known_boundaries_on_the_real_file() {
    let Some((c, _)) = build_or_skip() else {
        return;
    };
    let by = |k, id| c.item(k, ItemId(id)).expect("esiste").origin;
    assert_eq!(by(ItemKind::Passive, 1), Some(Origin::Rebirth));
    assert_eq!(by(ItemKind::Active, 555), Some(Origin::Repentance));
    assert!(
        c.items().all(|i| i.origin.is_some()),
        "every item in the file has an edition: the thresholds cover every id"
    );
}

#[test]
fn the_catalog_ids_are_a_subset_of_the_save_slots_and_the_holes_are_the_missing_ids() {
    // The save declares 733 slots for items and 642 for achievements. The catalog's
    // collectibles (kind other than Trinket) are 721, not 723: 723 was a grep count
    // that included the two commented-out elements id 43 and 61 (see
    // item_counts_by_kind_match_the_repentance_file). 733 slots - 721 items - slot 0:
    // 11 unused ids between 1 and 732, 43 and 61 included.
    let Some((c, _)) = build_or_skip() else {
        return;
    };
    let Some(save_path) = test_support::sample("20260831.rep+persistentgamedata1.dat") else {
        return;
    };
    let Ok(save) = core_save::Save::open(&save_path) else {
        test_support::skip("the 2026-08-31 profile exists but doesn't open");
        return;
    };
    let item_slots = save
        .section(core_save::Kind::Items)
        .expect("section 4")
        .count;
    let ach_slots = save
        .section(core_save::Kind::Achievements)
        .expect("section 1")
        .count;
    let max_item = c
        .items()
        .filter(|i| i.kind != ItemKind::Trinket)
        .map(|i| i.id.0)
        .max()
        .unwrap();
    let max_ach = c.achievements().map(|a| a.id.0).max().unwrap();
    assert!(
        max_item < item_slots,
        "item id {max_item} beyond the {item_slots} slots"
    );
    assert!(
        max_ach <= ach_slots,
        "achievement id {max_ach} beyond the {ach_slots} slots"
    );
    let collectibles: std::collections::BTreeSet<u32> = c
        .items()
        .filter(|i| i.kind != ItemKind::Trinket)
        .map(|i| i.id.0)
        .collect();
    assert_eq!(
        collectibles.len(),
        c.items().filter(|i| i.kind != ItemKind::Trinket).count(),
        "ids are unique across passives, actives and familiars: that's what makes the pool merge correct"
    );
    let holes: Vec<u32> = (1..item_slots)
        .filter(|id| !collectibles.contains(id))
        .collect();
    assert_eq!(
        holes,
        vec![43, 61, 235, 587, 613, 620, 630, 648, 662, 666, 718],
        "holes"
    );
}

#[test]
fn the_winning_items_xml_contains_every_id_of_the_base_game() {
    let Some(path) = test_support::packed_file("config.a") else {
        return;
    };
    let Ok(archive) = Archive::open(&path) else {
        test_support::skip("packed/config.a is present but doesn't open");
        return;
    };
    let Some(bytes) = archive.read("resources/items.xml") else {
        test_support::skip("resources/items.xml missing from config.a");
        return;
    };
    let Some((winner, _)) = build_or_skip() else {
        return;
    };
    let base = Catalog::build(|p| (p == "items.xml").then(|| bytes.clone()));
    let base_ids: std::collections::BTreeSet<(ItemKind, u32)> =
        base.items().map(|i| (i.kind, i.id.0)).collect();
    assert!(
        base_ids.len() > 300,
        "the base game has more than 300 items, otherwise the test passes vacuously: {}",
        base_ids.len()
    );
    let winner_ids: std::collections::BTreeSet<(ItemKind, u32)> =
        winner.items().map(|i| (i.kind, i.id.0)).collect();
    let missing: Vec<(ItemKind, u32)> = base_ids.difference(&winner_ids).cloned().collect();
    assert!(
        missing.is_empty(),
        "base game ids absent from the winning items.xml: {missing:?}"
    );
}

#[test]
fn the_unlock_index_carries_every_link_exactly_once() {
    let Some((c, _)) = build_or_skip() else {
        return;
    };
    let from_files = c.items().filter(|i| i.unlocked_by.is_some()).count()
        + c.characters().filter(|ch| ch.unlocked_by.is_some()).count()
        + c.bosses().filter(|b| b.unlocked_by.is_some()).count()
        + c.challenges().map(|ch| ch.unlocked_by.len()).sum::<usize>();
    let in_index: usize = c.achievements().map(|a| c.unlocks(a.id).len()).sum();
    // 370 items + 40 characters + 27 bosses + the challenge lists (2026-09-03).
    assert_eq!(in_index, from_files);
    assert!(
        from_files > 400,
        "the links are in the hundreds: {from_files}"
    );
    // Achievement 1 unlocks Magdalene (players.xml: id 1, achievement="1") and, verified
    // against the real file on 2026-09-05, also two challenges that share the same
    // unlock achievement: "The Tank" (id 5) and "Cursed!" (id 10), both
    // achievements="1" in challenges.xml. The brief assumed only Magdalene: corrected
    // here against the real data.
    assert_eq!(
        c.unlocks(AchievementId(1)),
        &[
            Unlock::Character { id: CharacterId(1) },
            Unlock::Challenge { id: ChallengeId(5) },
            Unlock::Challenge {
                id: ChallengeId(10)
            },
        ]
    );
}

/// Challenge rewards in the Repentance+ file: 39 out of 45 challenges have an
/// achievement that says it beats that challenge (20 with a hash mark in the comment,
/// 10 with the name in parentheses, 9 only in `steam_description`); challenges 31..=35
/// and 45 leave no trace.
#[test]
fn thirty_nine_challenges_have_a_reward_achievement_and_six_have_none() {
    let Some((c, _)) = build_or_skip() else {
        return;
    };
    let with_reward: Vec<u32> = c
        .challenges()
        .filter(|ch| !ch.rewards.is_empty())
        .map(|ch| ch.id.0)
        .collect();
    assert_eq!(with_reward.len(), 39);
    let without: Vec<u32> = c
        .challenges()
        .filter(|ch| ch.rewards.is_empty())
        .map(|ch| ch.id.0)
        .collect();
    assert_eq!(without, vec![31, 32, 33, 34, 35, 45]);
    // No challenge has more than one reward in today's file.
    assert!(c.challenges().all(|ch| ch.rewards.len() <= 1));

    let reward = |id: u32| c.challenge(ChallengeId(id)).unwrap().rewards.clone();
    assert_eq!(
        reward(19),
        vec![AchievementId(62)],
        "Epic Fetus, from the comment"
    );
    assert_eq!(
        reward(36),
        vec![AchievementId(517)],
        "Dirty Mind, from the steam_description"
    );
    assert_eq!(reward(44), vec![AchievementId(533)], "The Hermit");
    // The "Unlocked a new challenge." steam_description of achievements that make a
    // challenge available is not a reward: no diagnostic and no false positive.
    assert!(
        !c.diagnostics()
            .iter()
            .any(|d| matches!(d, Diagnostic::RewardForUnknownChallenge { .. })),
        "{:?}",
        c.diagnostics()
    );
}

/// Where the twelve completion marks are named, and where they are not.
///
/// `completion_widget.anm2` has one layer per mark plus `Paper`, and its glyph row holds
/// **twelve** 16 × 16 cells while the layers claim **eleven**. For a while the leftover
/// cell was taken for Delirium's mark, by elimination. It isn't: Delirium's mark is a
/// small face with two eyes, and the cell nobody claims is a different drawing whose
/// meaning we still don't know.
///
/// The name comes from Repentance+'s **online lobby**, whose `.anm2` draws the marks on
/// every player's card and is the only file in the game that names all twelve —
/// `Completion_Delirium` included. The app reads Delirium's symbol from there, so
/// this test guards both halves: the layer must exist, and the widget's gap must stay a
/// gap rather than quietly turning into a twelfth layer that would then disagree.
#[test]
fn only_the_online_lobby_names_all_twelve_marks() {
    let Some((_, rs)) = build_or_skip() else {
        return;
    };
    let Some(widget) = rs.read("gfx/ui/completion_widget.anm2") else {
        test_support::skip("gfx/ui/completion_widget.anm2 is not in the archives");
        return;
    };
    let frames = catalog::anm2_frames(&widget).expect("a real anm2 must parse");
    let claimed: std::collections::BTreeSet<u32> = frames
        .iter()
        .filter(|f| f.rect.w == 16 && f.rect.h == 16 && (f.rect.y == 96 || f.rect.y == 112))
        .map(|f| f.rect.x)
        .collect();
    let unclaimed: Vec<u32> = (0..12)
        .map(|i| i * 16)
        .filter(|x| !claimed.contains(x))
        .collect();
    assert_eq!(
        (claimed.len(), unclaimed.as_slice()),
        (11, [96].as_slice()),
        "the widget is supposed to name eleven of twelve cells and leave x = 96 unnamed"
    );
    assert!(
        !frames.iter().any(|f| f.layer == "Delirium"),
        "the widget has gained a Delirium layer: the symbol should come from here now, \
         not from the lobby"
    );

    let Some(lobby) = rs.read("gfx/ui/main menu/onlinelobby.anm2") else {
        test_support::skip("gfx/ui/main menu/onlinelobby.anm2 is not in the archives");
        return;
    };
    let lobby = catalog::anm2_frames(&lobby).expect("a real anm2 must parse");
    let marks: std::collections::BTreeSet<&str> = lobby
        .iter()
        .filter(|f| f.layer.starts_with("Completion_"))
        .map(|f| f.layer.as_str())
        .collect();
    assert_eq!(marks.len(), 12, "twelve marks were expected, got {marks:?}");
    // The one the whole detour was about, at the size and place the symbol is cut from.
    let delirium: Vec<(u32, u32)> = lobby
        .iter()
        .filter(|f| f.layer == "Completion_Delirium" && f.animation == "Background")
        .map(|f| (f.rect.x, f.rect.y))
        .collect();
    assert!(
        delirium.contains(&(224, 32)),
        "Completion_Delirium no longer crops (224, 32) in the Background animation: {delirium:?}"
    );
    assert!(
        lobby
            .iter()
            .all(|f| f.layer != "Completion_Delirium" || (f.rect.w == 16 && f.rect.h == 16)),
        "the mark is expected to be 16 x 16 like every other one"
    );
}
