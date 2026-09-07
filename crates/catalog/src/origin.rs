//! The DLC that introduced an item. Not in the XML: derived from id ranges, which are a
//! stable historical fact (every DLC appended its items at the end). Thresholds
//! declared here, with the edge cases pinned by tests; beyond the last known id the
//! answer is `None`, not a guess: a future patch will show up in the real test that
//! compares against `items.xml`.
//!
//! It's called `Origin`, not `Edition`: `discovery` already has an `Edition` on the
//! IPC, the edition of the installed game, with a different casing (`snake_case`). Two
//! types with the same name and different meanings would be a conflict just waiting to
//! confuse whoever reads the imports.

use serde::Serialize;

use crate::ids::ItemId;
use crate::items::ItemKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Origin {
    Rebirth,
    Afterbirth,
    AfterbirthPlus,
    Repentance,
}

/// (last included id, origin) for collectibles. Verified on 2026-09-04 against
/// `samples/sprites/items.xml`: 341 Torn Photo (last of Rebirth), 342 Blue Cap (first of
/// Afterbirth), 440 Kidney Stone (last of Afterbirth), 441 Mega Blast (first of AB+),
/// 552 Mom's Shovel (last of AB+), 553 Mucormycosis (first of Repentance), 732 Mom's
/// Ring (last, the highest id in the file).
const COLLECTIBLES: [(u32, Origin); 4] = [
    (341, Origin::Rebirth),
    (440, Origin::Afterbirth),
    (552, Origin::AfterbirthPlus),
    (732, Origin::Repentance),
];

/// (last included id, origin) for trinkets. Verified on 2026-09-04 against
/// `samples/sprites/items.xml`: 61 The Left Hand (last of Rebirth), 62 Shiny Rock (first
/// of Afterbirth), 89 Child Leash (last of Afterbirth), 90 Brown Cap (first of AB+),
/// 128 Finger Bone (last of AB+), 129 Jawbreaker (first of Repentance), 189 Sigil of
/// Baphomet (last, the highest id in the file).
const TRINKETS: [(u32, Origin); 4] = [
    (61, Origin::Rebirth),
    (89, Origin::Afterbirth),
    (128, Origin::AfterbirthPlus),
    (189, Origin::Repentance),
];

pub fn origin_of(kind: ItemKind, id: ItemId) -> Option<Origin> {
    if id.0 == 0 {
        return None;
    }
    let table: &[(u32, Origin)] = match kind {
        ItemKind::Passive | ItemKind::Active | ItemKind::Familiar => &COLLECTIBLES,
        ItemKind::Trinket => &TRINKETS,
    };
    table
        .iter()
        .find(|(last, _)| id.0 <= *last)
        .map(|(_, e)| *e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_items_fall_in_the_right_origin() {
        // Known cases, verified against the wiki and the files: id of the first item of each DLC.
        assert_eq!(
            origin_of(ItemKind::Passive, ItemId(1)),
            Some(Origin::Rebirth)
        ); // The Sad Onion
        assert_eq!(
            origin_of(ItemKind::Passive, ItemId(341)),
            Some(Origin::Rebirth)
        ); // last of Rebirth
        assert_eq!(
            origin_of(ItemKind::Passive, ItemId(342)),
            Some(Origin::Afterbirth)
        ); // first of Afterbirth
        assert_eq!(
            origin_of(ItemKind::Passive, ItemId(441)),
            Some(Origin::AfterbirthPlus)
        );
        assert_eq!(
            origin_of(ItemKind::Active, ItemId(555)),
            Some(Origin::Repentance)
        ); // Golden Razor
        assert_eq!(
            origin_of(ItemKind::Passive, ItemId(732)),
            Some(Origin::Repentance)
        );
    }

    #[test]
    fn trinkets_have_their_own_thresholds() {
        assert_eq!(
            origin_of(ItemKind::Trinket, ItemId(1)),
            Some(Origin::Rebirth)
        );
        assert_eq!(
            origin_of(ItemKind::Trinket, ItemId(62)),
            Some(Origin::Afterbirth)
        );
        assert_eq!(
            origin_of(ItemKind::Trinket, ItemId(129)),
            Some(Origin::Repentance)
        );
    }

    #[test]
    fn beyond_the_last_known_id_is_unknown_not_a_guess() {
        assert_eq!(origin_of(ItemKind::Passive, ItemId(733)), None);
        assert_eq!(origin_of(ItemKind::Trinket, ItemId(190)), None);
        assert_eq!(origin_of(ItemKind::Passive, ItemId(0)), None);
    }
}
