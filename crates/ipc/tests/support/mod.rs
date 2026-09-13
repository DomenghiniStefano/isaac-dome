//! The synthetic catalog B37's tests read. Every item, character and challenge points at an
//! achievement, which is the relationship these views show. Challenge 4 is named by two
//! achievements: the two-route case, which the real files also have.

use catalog::Catalog;

const ITEMS: &[u8] = b"<items gfxroot=\"gfx/items/\"><passive id=\"2\" gfx=\"a.png\" name=\"A\" achievement=\"1\" /><trinket id=\"1\" gfx=\"t.png\" name=\"T\" achievement=\"3\" /></items>";
const ACH: &[u8] = b"<achievements gfxroot=\"gfx/ui/achievement/\"><achievement id=\"1\" text=\"t1\" gfx=\"1.png\" /><achievement id=\"2\" text=\"t2\" gfx=\"2.png\" /><achievement id=\"3\" text=\"t3\" gfx=\"3.png\" /></achievements>";
const PLAYERS: &[u8] = b"<players portraitroot=\"gfx/ui/stage/\"><player id=\"7\" name=\"#Z_NAME\" portrait=\"z.png\" achievement=\"2\" /></players>";
const CHALLENGES: &[u8] = b"<challenges version=\"1\"><challenge id=\"4\" name=\"Both\" achievements=\"1,2\" endstage=\"1\" /></challenges>";

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
