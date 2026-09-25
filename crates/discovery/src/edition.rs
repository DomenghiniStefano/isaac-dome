//! DLC→edition map and pure helpers.

use std::collections::BTreeSet;

use crate::{Dlc, Edition};

pub(crate) const DLC_AFTERBIRTH: u32 = 401920;
pub(crate) const DLC_AFTERBIRTH_PLUS: u32 = 570660;
pub(crate) const DLC_REPENTANCE: u32 = 1426300;
pub(crate) const DLC_REPENTANCE_PLUS: u32 = 3353470;

/// Every DLC with the edition it makes, oldest first: the order `dlcs_from_appids` answers in,
/// and the reverse of the order the edition is decided in.
const LADDER: [(u32, Dlc, Edition); 4] = [
    (DLC_AFTERBIRTH, Dlc::Afterbirth, Edition::Afterbirth),
    (
        DLC_AFTERBIRTH_PLUS,
        Dlc::AfterbirthPlus,
        Edition::AfterbirthPlus,
    ),
    (DLC_REPENTANCE, Dlc::Repentance, Edition::Repentance),
    (
        DLC_REPENTANCE_PLUS,
        Dlc::RepentancePlus,
        Edition::RepentancePlus,
    ),
];

/// Edition = the highest DLC owned; base = Rebirth.
pub(crate) fn edition_from_appids(appids: &BTreeSet<u32>) -> Edition {
    LADDER
        .iter()
        .rev()
        .find(|(id, _, _)| appids.contains(id))
        .map_or(Edition::Rebirth, |(_, _, edition)| *edition)
}

/// All owned DLCs, in ascending order.
pub(crate) fn dlcs_from_appids(appids: &BTreeSet<u32>) -> Vec<Dlc> {
    LADDER
        .iter()
        .filter(|(id, _, _)| appids.contains(id))
        .map(|(_, dlc, _)| *dlc)
        .collect()
}
