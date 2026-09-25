//! The catalog → wiki page mapping. It is one function because two readers need the same
//! answer: search keys its documents by it, and a requirement links to it. The test below is
//! what stops the two from drifting — a boss is found by its portrait's file name, and a
//! second copy of that rule would be wrong within a release.

use catalog::Catalog;
use ipc::SearchIndex;
use wiki::Target;

const ITEMS: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"105\" gfx=\"d6.png\" name=\"The D6\" /><trinket id=\"97\" gfx=\"t.png\" name=\"Tonsil\" /></items>";
const BOSSES: &[u8] = b"<bossportraits gfxroot=\"gfx/ui/boss/\"><boss id=\"1\" name=\"Monstro\" portrait=\"Portrait_20.0_Monstro.png\" /><boss id=\"2\" name=\"Nameless\" portrait=\"no_key.png\" /></bossportraits>";
const CHALLENGES: &[u8] =
    b"<challenges><challenge id=\"36\" name=\"Scat Man\" startingitems=\"\" /></challenges>";
const PLAYERS: &[u8] =
    b"<players portraitroot=\"gfx/ui/stage/\"><player id=\"0\" name=\"Isaac\" portrait=\"p.png\" /></players>";
const ACH: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\"><achievement id=\"1\" text=\"You unlocked The D6\" gfx=\"1.png\" /></achievements>";

fn catalog() -> Catalog {
    Catalog::build(|p| match p {
        "items.xml" => Some(ITEMS.to_vec()),
        "bossportraits.xml" => Some(BOSSES.to_vec()),
        "challenges.xml" => Some(CHALLENGES.to_vec()),
        "players.xml" => Some(PLAYERS.to_vec()),
        "achievements.xml" => Some(ACH.to_vec()),
        _ => None,
    })
}

/// Search keys every catalog entity by the mapping. These are the keys that have to come out:
/// the test states them, it does not read them back from the code that produced them.
#[test]
fn the_catalog_entities_are_keyed_by_their_wiki_target() {
    let ds = wiki::for_tests::empty_dataset();
    let docs = ipc::for_tests::documents(
        &SearchIndex::build(Ok(&ds)),
        Some(&catalog()),
        &ipc::for_tests::bosses(&catalog()),
    );
    let keys: Vec<&Target> = docs.keys().collect();

    assert!(
        keys.contains(&&Target::Item { id: 105 }),
        "a collectible is an item page"
    );
    assert!(
        keys.contains(&&Target::Trinket { id: 97 }),
        "a trinket is a trinket page, not an item one"
    );
    assert!(
        keys.contains(&&Target::Challenge { number: 36 }),
        "the wiki calls the field number, the catalog calls it id"
    );
    assert!(keys.contains(&&Target::Character { id: 0 }));
    assert!(keys.contains(&&Target::Achievement { id: 1 }));
    assert!(
        keys.contains(&&Target::Entity {
            id: 20,
            variant: 0,
            subtype: 0
        }),
        "the entity key is read from the portrait's file name, not from the BossId"
    );
    assert_eq!(
        keys.iter()
            .filter(|t| matches!(t, Target::Entity { .. }))
            .count(),
        1,
        "a portrait that declares no key names no page, and is left out rather than guessed"
    );
}
