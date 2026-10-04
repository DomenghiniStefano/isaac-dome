//! What a run's page shows beyond the list's columns: the killer and its spawner as the game's
//! own entities, each floor by its name, each pickup with its pool and floor, and the face of
//! whoever was played. One resolver carries the catalog, the wiki and the icon function, so every
//! part of a run is named and pictured the same way.

use catalog::{AchievementId, Catalog, CharacterId, ItemId, Language};
use serde::Serialize;
use wiki::{Dataset, Target};

use crate::icon::IconRef;
use crate::runs::RunItemRef;

/// An entity as the death line names it, `id.variant` — `9.0`, a shot — resolved when the
/// catalog knows the row. `raw` stays: a name nobody can give must not take the place of the one
/// thing the log said.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct EntityRef {
    pub raw: String,
    pub name: Option<String>,
    pub icon_url: Option<String>,
    /// The wiki page for that exact entity, when the wiki has one.
    pub page: Option<Target>,
}

/// One floor of a run, as the game announced it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct RunFloorView {
    pub stage: u32,
    pub stage_type: u32,
    /// The game's own name, `Basement I`. `None` when any link to it is missing, and the page
    /// shows the numbers: a name borrowed from another floor would be a guess.
    pub name: Option<String>,
    /// How many rooms the floor was built with. `None` when the log described no pass, or more
    /// than one: which of several passes was walked is not established, and this does not pick.
    pub rooms: Option<u32>,
}

/// One item picked up after the starting window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct PickupView {
    pub item: RunItemRef,
    /// The game's pool name as the log wrote it: `treasure`, `shop`, `devil`.
    pub pool: String,
    /// The index into the run's `floorDetails` of the floor it was taken on.
    pub floor: Option<u32>,
}

/// Names and pictures for a run's parts, from whatever is installed and embedded.
pub(crate) struct Resolve<'a, F> {
    pub catalog: Option<&'a Catalog>,
    pub wiki: Option<&'a Dataset>,
    pub icon: F,
}

impl<F: FnMut(&IconRef) -> Option<String>> Resolve<'_, F> {
    /// An item of a run, named and pictured when the catalog knows it. The line the fold reads is
    /// `Adding collectible N`: a trinket cannot be meant, and looking one up would put the wrong
    /// name on a run.
    pub fn item(&mut self, id: u32) -> RunItemRef {
        let found = self.catalog.and_then(|c| c.collectible(ItemId(id)));
        RunItemRef {
            id,
            name: found
                .zip(self.catalog)
                .map(|(item, c)| c.text(&item.name, Language::English).to_string()),
            icon_url: found.and_then(|item| {
                (self.icon)(&IconRef::Item {
                    kind: item.kind,
                    id,
                })
            }),
        }
    }

    pub fn pickup(&mut self, p: &run::Pickup) -> PickupView {
        PickupView {
            item: self.item(p.id),
            pool: p.pool.clone(),
            floor: p.floor,
        }
    }

    /// The killer of a death line. A string that does not read as `id.variant[.subtype]` keeps
    /// what the log wrote and nothing else.
    pub fn entity(&mut self, raw: &str) -> EntityRef {
        let Some((id, variant, subtype)) = entity_key(raw) else {
            return EntityRef {
                raw: raw.to_string(),
                name: None,
                icon_url: None,
                page: None,
            };
        };
        let page = Target::Entity {
            id,
            variant,
            subtype,
        };
        // The picture is asked for only for a row the catalog has, as an item's is: a URL for a
        // row nobody can draw would be a broken image in place of the id.
        let name = self.catalog.and_then(|c| {
            c.entity(id, variant, subtype)
                .map(|e| c.text(&e.name, Language::English).to_string())
        });
        let icon_url = match name {
            Some(_) => (self.icon)(&IconRef::Entity {
                id,
                variant,
                subtype,
            }),
            None => None,
        };
        EntityRef {
            raw: raw.to_string(),
            name,
            icon_url,
            page: self.wiki.and_then(|d| d.entry(&page)).map(|_| page),
        }
    }

    /// The spawner of a death line, or nobody: entity `0` is what the game writes when the
    /// killer spawned itself.
    pub fn spawner(&mut self, raw: &str) -> Option<EntityRef> {
        match entity_key(raw) {
            Some((0, _, _)) => None,
            Some(_) | None => Some(self.entity(raw)),
        }
    }

    /// The co-op menu head of whoever was played: by the id the log states first, by name only
    /// when there is no id — and then only when the name means one character, since a Tainted
    /// form wears its base's name.
    pub fn head(&mut self, id: Option<u32>, name: Option<&str>) -> Option<String> {
        let c = self.catalog?;
        let character = match id {
            Some(id) => c.character(CharacterId(id))?,
            None => {
                let named = crate::characters_named(c, name?, None);
                let [(only, _)] = named.as_slice() else {
                    return None;
                };
                c.character(CharacterId(*only))?
            }
        };
        character.head.as_ref()?;
        let row = crate::marks::row_for_character(character)?;
        (self.icon)(&IconRef::Head { row })
    }

    /// One floor of a run. A Greed run's floors have no name: Greed writes the normal path's
    /// numbers (`1,1` is its first floor and the normal path's Cellar I), and the table that names
    /// a floor from them is the normal path's.
    pub fn floor(&self, f: &run::Floor, greed: bool) -> RunFloorView {
        RunFloorView {
            stage: f.stage,
            stage_type: f.stage_type,
            name: match greed {
                true => None,
                false => self
                    .catalog
                    .and_then(|c| c.floor_name(f.stage, f.stage_type)),
            },
            rooms: match &f.generated {
                run::Generated::Once { rooms, .. } => Some(*rooms),
                run::Generated::NotSaid | run::Generated::Several { .. } => None,
            },
        }
    }
}

/// `9.0` -> `(9, 0, 0)`, `9.0.1` -> `(9, 0, 1)`; anything else is not an entity key.
fn entity_key(raw: &str) -> Option<(u32, u32, u32)> {
    let mut parts = raw.split('.').map(str::parse::<u32>);
    let id = parts.next()?.ok()?;
    let variant = parts.next()?.ok()?;
    let subtype = match parts.next() {
        Some(s) => s.ok()?,
        None => 0,
    };
    parts.next().is_none().then_some((id, variant, subtype))
}

/// An achievement a run unlocked, as the page lists it: named and pictured when the catalog
/// knows it, the id always.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct RunAchievementView {
    pub id: u32,
    pub text: Option<String>,
    pub icon_url: Option<String>,
}

impl<F: FnMut(&IconRef) -> Option<String>> Resolve<'_, F> {
    pub fn achievement(&mut self, id: u32) -> RunAchievementView {
        let text = self
            .catalog
            .and_then(|c| c.achievement(AchievementId(id)))
            .map(|a| a.text.clone());
        let icon_url = match text {
            Some(_) => (self.icon)(&IconRef::Achievement { id }),
            None => None,
        };
        RunAchievementView { id, text, icon_url }
    }
}
