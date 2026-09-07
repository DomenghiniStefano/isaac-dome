//! Catalog::build never fails: whatever is missing produces a diagnostic.

use catalog::{
    AchievementId, BossId, Catalog, ChallengeId, CharacterId, Diagnostic, ItemId, ItemKind,
    Language, Rect, Source, SpriteRef, Text, SOURCES,
};

#[test]
fn pool_membership_is_written_on_the_item_whatever_its_kind() {
    let items: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"1\" gfx=\"a.png\" name=\"A\" /><active id=\"2\" gfx=\"b.png\" name=\"B\" /><trinket id=\"1\" gfx=\"t.png\" name=\"T\" /></items>";
    let pools: &[u8] = b"<ItemPools><Pool Name=\"treasure\"><Item Id=\"1\" Weight=\"1\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/><Item Id=\"2\" Weight=\"0.5\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/></Pool><Pool Name=\"boss\"><Item Id=\"1\" Weight=\"2\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/></Pool></ItemPools>";
    let c = Catalog::build(|p| match p {
        "items.xml" => Some(items.to_vec()),
        "itempools.xml" => Some(pools.to_vec()),
        _ => None,
    });
    let a = c.item(ItemKind::Passive, ItemId(1)).unwrap();
    assert_eq!(
        a.pools.iter().map(|m| m.pool.as_str()).collect::<Vec<_>>(),
        vec!["treasure", "boss"]
    );
    let b = c.item(ItemKind::Active, ItemId(2)).unwrap();
    assert_eq!(b.pools.len(), 1);
    assert_eq!(b.pools[0].weight, 0.5);
    assert!(
        c.item(ItemKind::Trinket, ItemId(1))
            .unwrap()
            .pools
            .is_empty(),
        "trinkets are never in pools"
    );
    assert_eq!(c.pools().len(), 2);
}

#[test]
fn a_reader_that_has_nothing_yields_one_missing_diagnostic_per_source() {
    let c = Catalog::build(|_| None);
    let missing: Vec<&Source> = c
        .diagnostics()
        .iter()
        .filter_map(|d| match d {
            Diagnostic::SourceMissing { source } => Some(source),
            Diagnostic::SourceUnreadable { .. }
            | Diagnostic::ElementSkipped { .. }
            | Diagnostic::UnresolvedKey { .. }
            | Diagnostic::UnknownLanguage { .. }
            | Diagnostic::HeadSheetUnavailable
            | Diagnostic::RewardForUnknownChallenge { .. } => None,
        })
        .collect();
    assert_eq!(missing.len(), SOURCES.len());
    for (_, source) in SOURCES {
        assert!(
            missing.contains(&&source),
            "missing diagnostic for {source:?}"
        );
    }
    assert_eq!(c.items().count(), 0);
    assert_eq!(c.characters().count(), 0);
}

#[test]
fn the_reader_is_asked_for_every_source_by_logical_path() {
    let mut asked = Vec::new();
    let _ = Catalog::build(|p| {
        asked.push(p.to_string());
        None
    });
    for (path, _) in SOURCES {
        assert!(asked.contains(&path.to_string()), "did not ask for {path}");
    }
}

fn only_strings(table: &'static [u8]) -> Catalog {
    Catalog::build(move |p| (p == "stringtable.sta").then(|| table.to_vec()))
}

const TABLE: &[u8] = br#"<stringtable><languages>
  <language id="21" index="0" name="Key"/><language id="0" index="1" name="English"/>
  <language id="3" index="2" name="French"/></languages>
  <category name="Items"><key name="A_NAME"><string>Apple</string><string>Pomme</string></key>
  <key name="B_NAME"><string>Bee</string></key></category></stringtable>"#;

#[test]
fn text_resolves_requested_then_english_then_key() {
    let c = only_strings(TABLE);
    let a = Text::from_attr("#A_NAME");
    let b = Text::from_attr("#B_NAME");
    let z = Text::from_attr("#Z_NAME");
    assert_eq!(c.text(&a, Language::French), "Pomme");
    assert_eq!(
        c.text(&b, Language::French),
        "Bee",
        "missing French: English"
    );
    assert_eq!(
        c.text(&z, Language::French),
        "Z_NAME",
        "missing everything: the key"
    );
    assert_eq!(c.text(&Text::from_attr("Plain"), Language::French), "Plain");
}

#[test]
fn without_a_stringtable_every_key_falls_back_to_itself() {
    let c = Catalog::build(|_| None);
    assert_eq!(
        c.text(&Text::from_attr("#A_NAME"), Language::English),
        "A_NAME"
    );
    assert!(c.languages().is_empty());
}

#[test]
fn items_and_strings_together_give_real_names() {
    let items: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"1\" gfx=\"a.png\" name=\"#A_NAME\" description=\"#A_DESCRIPTION\" /></items>";
    let c = Catalog::build(|p| match p {
        "items.xml" => Some(items.to_vec()),
        "stringtable.sta" => Some(TABLE.to_vec()),
        _ => None,
    });
    let a = c.item(ItemKind::Passive, ItemId(1)).expect("read");
    assert_eq!(c.text(&a.name, Language::English), "Apple");
    assert_eq!(a.sprite.path, "gfx/items/collectibles/a.png");
}

#[test]
fn heads_are_none_when_the_anm2_is_missing_and_it_is_diagnosed() {
    let c = Catalog::build(|p| (p == "players.xml").then(|| PLAYERS_ONE.to_vec()));
    assert!(c.characters().all(|ch| ch.head.is_none()));
    assert!(c.diagnostics().contains(&Diagnostic::HeadSheetUnavailable));
}

const PLAYERS_ONE: &[u8] = b"<players portraitroot=\"gfx/ui/stage/\"><player id=\"0\" name=\"#ISAAC_NAME\" portrait=\"PlayerPortrait_Isaac.png\" /></players>";

#[test]
fn a_character_gets_the_cell_its_frame_points_at() {
    let c = Catalog::build(|p| match p {
        "players.xml" => Some(PLAYERS_ONE.to_vec()),
        "gfx/ui/coop menu.anm2" => Some(ANM2_TWO.to_vec()),
        _ => None,
    });
    let isaac = c.character(CharacterId(0)).expect("read");
    assert_eq!(
        isaac.head,
        Some(SpriteRef {
            path: "gfx/ui/coop menu.png".to_string(),
            rect: Some(Rect {
                x: 0,
                y: 0,
                w: 32,
                h: 32
            }),
        }),
        "Isaac is on frame 1, frame 0 is the placeholder"
    );
    assert!(!c.diagnostics().contains(&Diagnostic::HeadSheetUnavailable));
}

const ANM2_TWO: &[u8] = br#"<AnimatedActor><Animations><Animation Name="Main" FrameNum="2"><LayerAnimations>
<LayerAnimation LayerId="0"><Frame Delay="1"/><Frame XCrop="0" YCrop="0" Width="32" Height="32" Delay="1"/></LayerAnimation>
</LayerAnimations></Animation></Animations></AnimatedActor>"#;

#[test]
fn a_key_missing_from_the_stringtable_yields_exactly_one_unresolved_key_diagnostic() {
    // #A_NAME is in the table, #A_DESCRIPTION is not.
    let items: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"1\" gfx=\"a.png\" name=\"#A_NAME\" description=\"#A_DESCRIPTION\" /></items>";
    let c = Catalog::build(|p| match p {
        "items.xml" => Some(items.to_vec()),
        "stringtable.sta" => Some(TABLE.to_vec()),
        _ => None,
    });
    let unresolved: Vec<&String> = c
        .diagnostics()
        .iter()
        .filter_map(|d| match d {
            Diagnostic::UnresolvedKey { key } => Some(key),
            _ => None,
        })
        .collect();
    assert_eq!(unresolved, vec!["A_DESCRIPTION"]);
}

#[test]
fn a_key_present_in_the_stringtable_is_not_diagnosed() {
    let items: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"1\" gfx=\"a.png\" name=\"#A_NAME\" description=\"#A_NAME\" /></items>";
    let c = Catalog::build(|p| match p {
        "items.xml" => Some(items.to_vec()),
        "stringtable.sta" => Some(TABLE.to_vec()),
        _ => None,
    });
    assert!(!c
        .diagnostics()
        .iter()
        .any(|d| matches!(d, Diagnostic::UnresolvedKey { .. })));
}

#[test]
fn without_a_stringtable_each_distinct_key_is_diagnosed_once_not_once_per_element() {
    // Two items share the same name key; a third key is different.
    let items: &[u8] = b"<items gfxroot=\"gfx/items/\">
<passive id=\"1\" gfx=\"a.png\" name=\"#DUPLICATE_NAME\" description=\"#DUPLICATE_NAME\" />
<passive id=\"2\" gfx=\"b.png\" name=\"#DUPLICATE_NAME\" description=\"#OTHER_NAME\" />
</items>";
    let c = Catalog::build(|p| (p == "items.xml").then(|| items.to_vec()));
    let mut unresolved: Vec<&String> = c
        .diagnostics()
        .iter()
        .filter_map(|d| match d {
            Diagnostic::UnresolvedKey { key } => Some(key),
            _ => None,
        })
        .collect();
    unresolved.sort();
    assert_eq!(unresolved, vec!["DUPLICATE_NAME", "OTHER_NAME"]);
}

#[test]
fn quality_and_tags_come_from_the_metadata_file_and_reach_actives_too() {
    let items: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"1\" gfx=\"a.png\" name=\"A\" /><active id=\"2\" gfx=\"b.png\" name=\"B\" /><trinket id=\"1\" gfx=\"t.png\" name=\"T\" /></items>";
    let meta: &[u8] = b"<items><item id=\"1\" quality=\"4\" tags=\"offensive\"/><item id=\"2\" quality=\"1\" tags=\"\"/><trinket id=\"1\" quality=\"-1\" tags=\"lucky\"/></items>";
    let c = Catalog::build(|p| match p {
        "items.xml" => Some(items.to_vec()),
        "items_metadata.xml" => Some(meta.to_vec()),
        _ => None,
    });
    assert_eq!(
        c.item(ItemKind::Passive, ItemId(1)).unwrap().quality,
        Some(4)
    );
    assert_eq!(
        c.item(ItemKind::Active, ItemId(2)).unwrap().quality,
        Some(1),
        "an active finds its metadata under <item>"
    );
    let t = c.item(ItemKind::Trinket, ItemId(1)).unwrap();
    assert_eq!(
        (t.quality, t.tags.as_slice()),
        (Some(-1), &["lucky".to_string()][..])
    );
}

#[test]
fn sources_are_nine_and_each_path_is_asked_exactly_once() {
    let mut asked: Vec<String> = Vec::new();
    let _ = Catalog::build(|p| {
        asked.push(p.to_string());
        None
    });
    asked.sort();
    let mut expected: Vec<String> = SOURCES.iter().map(|(p, _)| p.to_string()).collect();
    expected.sort();
    assert_eq!(
        asked, expected,
        "every source asked for once, no path outside SOURCES"
    );
    assert_eq!(SOURCES.len(), 9);
}

#[test]
fn achievements_are_indexed_by_id_and_keep_their_condition() {
    let ach: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\"><!-- beat Mom --><achievement id=\"4\" text=\"t\" gfx=\"a.png\" /></achievements>";
    let c = Catalog::build(|p| (p == "achievements.xml").then(|| ach.to_vec()));
    let a = c.achievement(AchievementId(4)).expect("read");
    assert_eq!(a.unlock_condition.as_deref(), Some("beat Mom"));
    assert_eq!(c.achievements().count(), 1);
}

#[test]
fn challenges_and_bosses_are_read_and_ordered_by_id() {
    let ch: &[u8] = b"<challenges><challenge name=\"B\" id=\"2\" /><challenge name=\"A\" id=\"1\" achievements=\"3\" /></challenges>";
    let bo: &[u8] = b"<bosses root=\"gfx/ui/boss/\"><boss id=\"2\" name=\"Y\" portrait=\"y.png\" /><boss id=\"1\" name=\"X\" portrait=\"x.png\" achievement=\"9\" /></bosses>";
    let c = Catalog::build(|p| match p {
        "challenges.xml" => Some(ch.to_vec()),
        "bossportraits.xml" => Some(bo.to_vec()),
        _ => None,
    });
    let names: Vec<&str> = c.challenges().map(|x| x.name.as_str()).collect();
    assert_eq!(names, vec!["A", "B"]);
    let bosses: Vec<u32> = c.bosses().map(|b| b.id.0).collect();
    assert_eq!(bosses, vec![1, 2]);
    assert_eq!(
        c.bosses().next().unwrap().unlocked_by,
        Some(AchievementId(9))
    );
    assert_eq!(
        c.challenge(ChallengeId(1)).map(|x| x.name.as_str()),
        Some("A")
    );
    assert!(c.challenge(ChallengeId(99)).is_none());
    assert_eq!(c.boss(BossId(1)).map(|b| b.name.as_str()), Some("X"));
    assert!(c.boss(BossId(99)).is_none());
}

use catalog::Unlock;

#[test]
fn unlocks_index_inverts_every_link_in_a_deterministic_order() {
    let items: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"2\" gfx=\"a.png\" name=\"A\" achievement=\"7\" /><trinket id=\"1\" gfx=\"t.png\" name=\"T\" achievement=\"7\" /><active id=\"9\" gfx=\"b.png\" name=\"B\" achievement=\"8\" /></items>";
    let players: &[u8] = b"<players portraitroot=\"gfx/ui/stage/\"><player id=\"3\" name=\"#X_NAME\" portrait=\"x.png\" achievement=\"7\" /></players>";
    let bosses: &[u8] = b"<bosses root=\"gfx/ui/boss/\"><boss id=\"5\" name=\"Y\" portrait=\"y.png\" achievement=\"8\" /></bosses>";
    let challenges: &[u8] =
        b"<challenges><challenge name=\"C\" id=\"4\" achievements=\"7,8\" /></challenges>";
    let c = Catalog::build(|p| match p {
        "items.xml" => Some(items.to_vec()),
        "players.xml" => Some(players.to_vec()),
        "bossportraits.xml" => Some(bosses.to_vec()),
        "challenges.xml" => Some(challenges.to_vec()),
        _ => None,
    });
    // Order: by variant (Item < Character < Boss < Challenge), then by (kind, id).
    assert_eq!(
        c.unlocks(AchievementId(7)),
        &[
            Unlock::Item {
                kind: ItemKind::Passive,
                id: ItemId(2)
            },
            Unlock::Item {
                kind: ItemKind::Trinket,
                id: ItemId(1)
            },
            Unlock::Character { id: CharacterId(3) },
            Unlock::Challenge { id: ChallengeId(4) },
        ]
    );
    assert_eq!(
        c.unlocks(AchievementId(8)),
        &[
            Unlock::Item {
                kind: ItemKind::Active,
                id: ItemId(9)
            },
            Unlock::Boss { id: BossId(5) },
            Unlock::Challenge { id: ChallengeId(4) }
        ]
    );
    assert!(
        c.unlocks(AchievementId(99)).is_empty(),
        "an achievement that unlocks nothing: empty, not an error"
    );
}

/// A challenge's reward lives on the achievement, in two different places depending on
/// the file's era: the comment (`Beat Challenge #N`, `beat Challenge N (Name)`) or the
/// `steam_description` attribute (`Complete Challenge N.`). `build` collects them onto
/// the challenge.
#[test]
fn challenge_rewards_are_collected_from_comments_and_steam_descriptions() {
    let ach: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\">
<!-- Beat Challenge #19 --><achievement id=\"62\" text=\"Epic Fetus\" gfx=\"a.png\" />
<!-- beat Challenge 24 (Pay to Play) unlocks Percs/Overdose --><achievement id=\"70\" text=\"Percs\" gfx=\"b.png\" />
<achievement id=\"517\" text=\"Dirty Mind\" gfx=\"c.png\" steam_description=\"Complete Challenge 36.\" />
<!-- beat Challenge #19 --><achievement id=\"30\" text=\"second reward\" gfx=\"d.png\" />
<!-- have 7 or more max red hearts --><achievement id=\"1\" text=\"Magdalene\" gfx=\"e.png\" steam_description=\"Unlocked a new challenge.\" />
</achievements>";
    let ch: &[u8] = b"<challenges><challenge name=\"The Family Man\" id=\"19\" /><challenge name=\"PAY TO PLAY\" id=\"24\" /><challenge name=\"Scat Man\" id=\"36\" /><challenge name=\"Pitch Black\" id=\"1\" /></challenges>";
    let c = Catalog::build(|p| match p {
        "achievements.xml" => Some(ach.to_vec()),
        "challenges.xml" => Some(ch.to_vec()),
        _ => None,
    });
    let rewards = |id: u32| c.challenge(ChallengeId(id)).unwrap().rewards.clone();
    assert_eq!(
        rewards(19),
        vec![AchievementId(30), AchievementId(62)],
        "two achievements for the same challenge, sorted by id and not by reading order"
    );
    assert_eq!(rewards(24), vec![AchievementId(70)]);
    assert_eq!(
        rewards(36),
        vec![AchievementId(517)],
        "without a comment, the steam_description counts"
    );
    assert_eq!(rewards(1), Vec::<AchievementId>::new());
    assert!(
        !c.diagnostics()
            .iter()
            .any(|d| matches!(d, Diagnostic::RewardForUnknownChallenge { .. })),
        "every cited challenge exists"
    );
}

/// A reward for a challenge that `challenges.xml` doesn't have means the two files are
/// from different eras: it's flagged, and nothing that was read is lost.
#[test]
fn a_reward_for_a_challenge_the_file_does_not_have_is_a_diagnostic() {
    let ach: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\"><!-- Beat Challenge #45 --><achievement id=\"9\" text=\"t\" gfx=\"a.png\" /></achievements>";
    let ch: &[u8] = b"<challenges><challenge name=\"Pitch Black\" id=\"1\" /></challenges>";
    let c = Catalog::build(|p| match p {
        "achievements.xml" => Some(ach.to_vec()),
        "challenges.xml" => Some(ch.to_vec()),
        _ => None,
    });
    assert!(c
        .diagnostics()
        .contains(&Diagnostic::RewardForUnknownChallenge {
            achievement: AchievementId(9),
            challenge: ChallengeId(45),
        }));
    assert_eq!(c.achievements().count(), 1);
    assert!(c.challenge(ChallengeId(1)).unwrap().rewards.is_empty());
}
