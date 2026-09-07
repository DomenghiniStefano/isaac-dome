//! The typed tree. Everything crosses the IPC as-is: camelCase, enums with data tagged by
//! `kind`, fieldless enums as a bare string.

use serde::{Deserialize, Serialize};

/// A wiki page reduced to what's needed: the infobox and the text sections that are kept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub title: String,
    /// Wiki revision the page was read from: says how stale the data is.
    pub revid: u64,
    pub infobox: Infobox,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    pub kind: SectionKind,
    pub blocks: Vec<Block>,
}

/// Fieldless: a bare string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SectionKind {
    Effects,
    Notes,
    Synergies,
    Interactions,
    Bugs,
    Behavior,
    ChampionVersions,
    DamageScaling,
    Strategies,
    Difficulty,
    Reward,
    Unlockable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Block {
    Paragraph {
        inline: Vec<Inline>,
    },
    List {
        ordered: bool,
        items: Vec<ListItem>,
    },
    Table {
        header: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
    Heading {
        level: u8,
        inline: Vec<Inline>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    pub inline: Vec<Inline>,
    /// The lists nested under the item.
    pub children: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Inline {
    Text {
        text: String,
        style: Style,
    },
    /// A link to something the catalog knows by id.
    Ref {
        target: Target,
        label: String,
    },
    /// A link to a wiki concept page, with no id in the game.
    Concept {
        page: String,
        label: String,
    },
    /// Text valid only in some editions (`{{dlc|…}}`).
    Edition {
        only: Vec<Dlc>,
        inline: Vec<Inline>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Style {
    Plain,
    Bold,
    Italic,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Target {
    Item { id: u32 },
    Trinket { id: u32 },
    Character { id: u32 },
    Achievement { id: u32 },
    Challenge { number: u32 },
    Entity { id: u32, variant: u32, subtype: u32 },
    Transformation { id: u32 },
    Stage { name: String },
    Room { name: String },
    Pickup { name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Dlc {
    Rebirth,
    Afterbirth,
    AfterbirthPlus,
    Repentance,
    RepentancePlus,
}

impl Dlc {
    /// The `{{dlc|…}}` template codes: `n`, `a`, `a+`, `r`, `r+`. Anything else → `None`.
    pub fn from_code(code: &str) -> Option<Dlc> {
        match code.trim() {
            "n" => Some(Dlc::Rebirth),
            "a" => Some(Dlc::Afterbirth),
            "a+" => Some(Dlc::AfterbirthPlus),
            "r" => Some(Dlc::Repentance),
            "r+" => Some(Dlc::RepentancePlus),
            _ => None, // allowed: the input is an open-ended string from the wiki
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Infobox {
    Item,
    Trinket,
    Achievement {
        description: String,
        requirements: Vec<Inline>,
        unlocks: Option<Target>,
    },
    Boss {
        base_hp: Option<u32>,
        environment: Vec<Inline>,
        pool: Vec<Inline>,
        unlocked_by: Option<Target>,
    },
    Challenge {
        blindfolded: bool,
        has_shops: bool,
        has_treasure_rooms: bool,
        items: Vec<Inline>,
        trinkets: Vec<Inline>,
        pickups: Vec<Inline>,
        health: Vec<Inline>,
        curse: Vec<Inline>,
        goal: Vec<Inline>,
        unlocks: Option<Target>,
        unlocked_by: Option<Target>,
    },
    Character {
        health: Vec<Inline>,
        damage: String,
        range: String,
        speed: String,
        luck: String,
        shot_speed: String,
        pickups: Vec<Inline>,
        collectibles: Vec<Inline>,
        unlocked_by: Option<Target>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, to_value};

    #[test]
    fn inline_shapes() {
        assert_eq!(
            to_value(Inline::Text {
                text: "x".into(),
                style: Style::Bold
            })
            .unwrap(),
            json!({"kind":"text","text":"x","style":"bold"})
        );
        assert_eq!(
            to_value(Inline::Ref {
                target: Target::Item { id: 25 },
                label: "Breakfast".into()
            })
            .unwrap(),
            json!({"kind":"ref","target":{"kind":"item","id":25},"label":"Breakfast"})
        );
        assert_eq!(
            to_value(Inline::Concept {
                page: "Shot Speed".into(),
                label: "Shot Speed".into()
            })
            .unwrap(),
            json!({"kind":"concept","page":"Shot Speed","label":"Shot Speed"})
        );
        assert_eq!(
            to_value(Inline::Edition {
                only: vec![Dlc::Repentance, Dlc::RepentancePlus],
                inline: vec![]
            })
            .unwrap(),
            json!({"kind":"edition","only":["repentance","repentancePlus"],"inline":[]})
        );
    }

    #[test]
    fn target_shapes() {
        assert_eq!(
            to_value(Target::Entity {
                id: 407,
                variant: 0,
                subtype: 0
            })
            .unwrap(),
            json!({"kind":"entity","id":407,"variant":0,"subtype":0})
        );
        assert_eq!(
            to_value(Target::Challenge { number: 19 }).unwrap(),
            json!({"kind":"challenge","number":19})
        );
        assert_eq!(
            to_value(Target::Stage {
                name: "Depths".into()
            })
            .unwrap(),
            json!({"kind":"stage","name":"Depths"})
        );
    }

    #[test]
    fn block_shapes() {
        assert_eq!(
            to_value(Block::List {
                ordered: false,
                items: vec![ListItem {
                    inline: vec![],
                    children: vec![]
                }]
            })
            .unwrap(),
            json!({"kind":"list","ordered":false,"items":[{"inline":[],"children":[]}]})
        );
        assert_eq!(
            to_value(Block::Heading {
                level: 3,
                inline: vec![]
            })
            .unwrap(),
            json!({"kind":"heading","level":3,"inline":[]})
        );
        assert_eq!(
            to_value(Block::Table {
                header: vec![],
                rows: vec![]
            })
            .unwrap(),
            json!({"kind":"table","header":[],"rows":[]})
        );
    }

    #[test]
    fn infobox_and_section_shapes() {
        assert_eq!(to_value(Infobox::Item).unwrap(), json!({"kind":"item"}));
        assert_eq!(
            to_value(Infobox::Boss {
                base_hp: Some(6666),
                environment: vec![],
                pool: vec![],
                unlocked_by: None
            })
            .unwrap(),
            json!({"kind":"boss","baseHp":6666,"environment":[],"pool":[],"unlockedBy":null})
        );
        assert_eq!(
            to_value(SectionKind::ChampionVersions).unwrap(),
            json!("championVersions")
        );
        let e = Entry {
            title: "Hush".into(),
            revid: 1,
            infobox: Infobox::Item,
            sections: vec![],
        };
        assert_eq!(
            to_value(e).unwrap(),
            json!({"title":"Hush","revid":1,"infobox":{"kind":"item"},"sections":[]})
        );
    }

    #[test]
    fn roundtrip() {
        let e = Entry {
            title: "X".into(),
            revid: 2,
            infobox: Infobox::Trinket,
            sections: vec![Section {
                kind: SectionKind::Effects,
                blocks: vec![Block::Paragraph {
                    inline: vec![
                        Inline::Text {
                            text: "a".into(),
                            style: Style::Plain,
                        },
                        Inline::Ref {
                            target: Target::Trinket { id: 97 },
                            label: "Tonsil".into(),
                        },
                    ],
                }],
            }],
        };
        let s = serde_json::to_string(&e).unwrap();
        assert_eq!(serde_json::from_str::<Entry>(&s).unwrap(), e);
    }
}
