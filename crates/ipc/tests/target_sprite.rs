//! `target_sprite`: from a wiki reference to the image the game actually has.
//!
//! This is the function that makes a wiki page illustrable. The wiki dataset carries
//! **text**: names, sections, and typed references (`Target`). The images live in the
//! game's archives and `catalog` is the one that knows them. This function is the
//! junction, and it lives in `ipc` because that's the only crate that knows both.
//!
//! The expected values come from the format of the game's files, not from the code's
//! own output: `bossportraits.xml` writes the portrait as
//! `Portrait_<type>.<variant>_<Name>.png`, and type and variant are exactly the key of
//! `Target::Entity`.
//!
//! That file name is the **fallback**, though, not the rule: where a wiki page names the
//! row, the page's own key is the one that answers. The tiers are unit-tested beside
//! `merge_keys`, and what they do to the installed game is in `target_sprite_real.rs`.

use catalog::Catalog;
use ipc::{target_sprite, Target, TargetSprite};

const ITEMS: &[u8] = br#"<items gfxroot="gfx/items/"><passive id="2" gfx="a.png" name="A" /><active id="5" gfx="b.png" name="B" /><trinket id="1" gfx="t.png" name="T" /></items>"#;
const ACH: &[u8] = br#"<achievements gfxroot="gfx/ui/achievement/"><achievement id="1" text="t1" gfx="1.png" /><!-- Beat Challenge #1 --><achievement id="2" text="t2" gfx="2.png" /></achievements>"#;
const PLAYERS: &[u8] =
    br##"<players portraitroot="gfx/ui/boss/"><player id="7" name="#Z_NAME" portrait="z.png" /></players>"##;
/// The boss rows carry **invented** names, and that is the point: the key of a row the
/// wiki names is the key of its page, read from the dataset compiled into this binary, so
/// a fixture called `Chub` would be answering with the real Chub's key (28.0) rather than
/// with the one written here. Names no page has isolate the fallback these tests are
/// about — the key the portrait's own file name declares.
const BOSSES: &[u8] = br#"<bosses root="resources/gfx/ui/boss/"><boss id="1" name="Quilfur" portrait="Portrait_20.0_Quilfur.png" /><boss id="2" name="Brantle" portrait="Portrait_28.1_Brantle.png" /><boss id="3" name="Nevecka" portrait="Portrait_Nevecka.png" /></bosses>"#;
const CHALLENGES: &[u8] =
    br#"<challenges><challenge name="Pitch Black" id="1" /><challenge name="Muffled" id="2" /></challenges>"#;

fn catalog_with_art() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS.to_vec()),
        "achievements.xml" => Some(ACH.to_vec()),
        "players.xml" => Some(PLAYERS.to_vec()),
        "bossportraits.xml" => Some(BOSSES.to_vec()),
        "challenges.xml" => Some(CHALLENGES.to_vec()),
        _ => None,
    })
}

fn path(c: &Catalog, t: &Target) -> String {
    match target_sprite(c, t) {
        TargetSprite::Found(s) => s.path.clone(),
        other => panic!("expected a sprite for {t:?}, got {other:?}"),
    }
}

#[test]
fn an_item_reference_resolves_to_its_icon_whatever_its_kind() {
    let c = catalog_with_art();
    // The wiki only says `Item { id }`: passives, actives and familiars share the same
    // id space, so the lookup goes through all three types.
    assert_eq!(
        path(&c, &Target::Item { id: 2 }),
        "gfx/items/collectibles/a.png"
    );
    assert_eq!(
        path(&c, &Target::Item { id: 5 }),
        "gfx/items/collectibles/b.png"
    );
}

#[test]
fn a_trinket_is_a_separate_id_space_from_the_collectibles() {
    let c = catalog_with_art();
    // Trinket 1 and passive 2 coexist; and item 1 doesn't exist among the collectibles.
    assert_eq!(
        path(&c, &Target::Trinket { id: 1 }),
        "gfx/items/trinkets/t.png"
    );
    assert!(matches!(
        target_sprite(&c, &Target::Item { id: 1 }),
        TargetSprite::Unknown
    ));
}

#[test]
fn an_achievement_and_a_character_resolve_to_their_own_art() {
    let c = catalog_with_art();
    assert_eq!(
        path(&c, &Target::Achievement { id: 1 }),
        "gfx/ui/achievement/1.png"
    );
    assert_eq!(path(&c, &Target::Character { id: 7 }), "gfx/ui/boss/z.png");
}

#[test]
fn a_boss_page_finds_its_portrait_through_the_entity_id_written_in_the_filename() {
    let c = catalog_with_art();
    // `Portrait_20.0_Quilfur.png` → entity type 20, variant 0: `bossportraits.xml` writes
    // type and variant into the file name, and that is what a row no wiki page names is
    // reached by.
    assert_eq!(
        path(
            &c,
            &Target::Entity {
                id: 20,
                variant: 0,
                subtype: 0
            }
        ),
        "gfx/ui/boss/Portrait_20.0_Quilfur.png"
    );
    // The variant matters: 28.1 is not 28.0.
    assert_eq!(
        path(
            &c,
            &Target::Entity {
                id: 28,
                variant: 1,
                subtype: 0
            }
        ),
        "gfx/ui/boss/Portrait_28.1_Brantle.png"
    );
    assert!(
        matches!(
            target_sprite(
                &c,
                &Target::Entity {
                    id: 28,
                    variant: 0,
                    subtype: 0
                }
            ),
            TargetSprite::Unknown
        ),
        "the wrong variant doesn't fall back to just any portrait"
    );
}

#[test]
fn the_subtype_does_not_take_part_in_the_match() {
    let c = catalog_with_art();
    // Portraits are declared by type and variant only: a different subtype is still
    // the same boss (its champion versions), not a boss with no portrait.
    assert_eq!(
        path(
            &c,
            &Target::Entity {
                id: 20,
                variant: 0,
                subtype: 3
            }
        ),
        "gfx/ui/boss/Portrait_20.0_Quilfur.png"
    );
}

#[test]
fn a_portrait_that_does_not_declare_an_entity_is_not_reachable_by_entity() {
    let c = catalog_with_art();
    // `Portrait_Nevecka.png` has no `<type>.<variant>` in its name. The boss exists in
    // the catalog, but it can't be reached from a `Target::Entity`: this is declared,
    // not guessed at.
    assert!(matches!(
        target_sprite(
            &c,
            &Target::Entity {
                id: 999,
                variant: 0,
                subtype: 0
            }
        ),
        TargetSprite::Unknown
    ));
}

#[test]
fn a_challenge_borrows_the_icon_of_the_achievement_it_rewards() {
    let c = catalog_with_art();
    // The game doesn't draw challenges: `gfx/challenge` doesn't exist. The only image
    // that *belongs* to a challenge is the achievement earned by completing it, and the
    // catalog already knows it (`Challenge.rewards`, from the `Beat Challenge #1` comment).
    assert_eq!(
        path(&c, &Target::Challenge { number: 1 }),
        "gfx/ui/achievement/2.png"
    );
    // Challenge 2 exists but has no known reward: this is a declared gap, not an
    // unknown id.
    assert!(matches!(
        target_sprite(&c, &Target::Challenge { number: 2 }),
        TargetSprite::NoArt
    ));
}

#[test]
fn the_targets_the_game_does_not_illustrate_say_so_instead_of_guessing() {
    let c = catalog_with_art();
    // Transformations and rooms have no image resolvable from the game's files. The
    // distinction matters for the design: `NoArt` is "there is none", `Unknown` is "I
    // don't know this" — and brief §5.6 asks for two different placeholders.
    assert!(matches!(
        target_sprite(&c, &Target::Transformation { id: 1 }),
        TargetSprite::NoArt
    ));
    assert!(matches!(
        target_sprite(
            &c,
            &Target::Room {
                name: "Devil Room".into()
            }
        ),
        TargetSprite::NoArt
    ));
    // Pickup and Stage are a different case, worth spelling out: the art **exists** in
    // the archives (`gfx/items/pick ups`, `gfx/ui/stage`), but none of the catalog's
    // nine sources names it, and the wiki identifies these two by name. Without a
    // name → file map, which we don't have today, the honest answer is "there is
    // none", not a name-similarity match.
    assert!(matches!(
        target_sprite(
            &c,
            &Target::Concept {
                name: "Black Heart".into()
            }
        ),
        TargetSprite::NoArt
    ));
    assert!(matches!(
        target_sprite(
            &c,
            &Target::Stage {
                name: "Depths".into()
            }
        ),
        TargetSprite::NoArt
    ));
}

#[test]
fn an_unknown_id_is_not_the_same_hole_as_a_missing_picture() {
    let c = catalog_with_art();
    assert!(matches!(
        target_sprite(&c, &Target::Achievement { id: 900 }),
        TargetSprite::Unknown
    ));
    assert!(matches!(
        target_sprite(&c, &Target::Character { id: 900 }),
        TargetSprite::Unknown
    ));
    assert!(matches!(
        target_sprite(&c, &Target::Challenge { number: 900 }),
        TargetSprite::Unknown
    ));
}

#[test]
fn an_empty_catalog_never_panics_and_never_invents() {
    let empty = Catalog::build(|_| None);
    for t in [
        Target::Item { id: 1 },
        Target::Trinket { id: 1 },
        Target::Character { id: 1 },
        Target::Achievement { id: 1 },
        Target::Challenge { number: 1 },
        Target::Entity {
            id: 20,
            variant: 0,
            subtype: 0,
        },
        Target::Stage {
            name: "Depths".into(),
        },
        Target::Concept {
            name: "Black Heart".into(),
        },
    ] {
        assert!(
            !matches!(target_sprite(&empty, &t), TargetSprite::Found(_)),
            "without a catalog nothing resolves: {t:?}"
        );
    }
}
