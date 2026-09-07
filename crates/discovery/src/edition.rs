//! DLC→edition map and pure helpers.

use std::collections::BTreeSet;

use crate::{Dlc, Edition};

pub(crate) const DLC_AFTERBIRTH: u32 = 401920;
pub(crate) const DLC_AFTERBIRTH_PLUS: u32 = 570660;
pub(crate) const DLC_REPENTANCE: u32 = 1426300;
pub(crate) const DLC_REPENTANCE_PLUS: u32 = 3353470;

/// Edition = the highest DLC owned; base = Rebirth.
pub(crate) fn edition_from_appids(appids: &BTreeSet<u32>) -> Edition {
    if appids.contains(&DLC_REPENTANCE_PLUS) {
        Edition::RepentancePlus
    } else if appids.contains(&DLC_REPENTANCE) {
        Edition::Repentance
    } else if appids.contains(&DLC_AFTERBIRTH_PLUS) {
        Edition::AfterbirthPlus
    } else if appids.contains(&DLC_AFTERBIRTH) {
        Edition::Afterbirth
    } else {
        Edition::Rebirth
    }
}

/// All owned DLCs, in ascending order.
pub(crate) fn dlcs_from_appids(appids: &BTreeSet<u32>) -> Vec<Dlc> {
    let mut v = Vec::new();
    if appids.contains(&DLC_AFTERBIRTH) {
        v.push(Dlc::Afterbirth);
    }
    if appids.contains(&DLC_AFTERBIRTH_PLUS) {
        v.push(Dlc::AfterbirthPlus);
    }
    if appids.contains(&DLC_REPENTANCE) {
        v.push(Dlc::Repentance);
    }
    if appids.contains(&DLC_REPENTANCE_PLUS) {
        v.push(Dlc::RepentancePlus);
    }
    v
}
