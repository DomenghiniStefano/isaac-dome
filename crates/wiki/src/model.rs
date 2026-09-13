//! The typed tree. Everything crosses the IPC as-is: camelCase, enums with data tagged by
//! `kind`, fieldless enums as a bare string.

use serde::{Deserialize, Serialize};

use crate::Diagnostics;

/// A wiki page reduced to what's needed: the infobox and the text sections that are kept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub title: String,
    /// Wiki revision the page was read from: says how stale the data is.
    pub revid: u64,
    /// The infobox's summary line. Plain text for achievements, wikitext everywhere else:
    /// both arrive as inline so the frontend has one shape and no switch on the kind.
    pub description: Vec<Inline>,
    /// The edition codes the infobox declares, parsed. Empty when the parameter is absent.
    /// Deliberately NOT called "introduced in" nor "exists in": which of the two it means
    /// is unmeasured — Blue Cap (342), the first Afterbirth item, declares neither — and a
    /// name would be a guess.
    pub dlc: Vec<Dlc>,
    /// What the wiki states has to be unlocked first. `None` means "the wiki does not state
    /// one", NEVER "it is free from the start": that answer belongs to `catalog` and `graph`.
    pub unlocked_by: Option<Target>,
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
    /// The infobox's `dlc` parameter, which concatenates codes without a separator: one
    /// page reads `a+nr`, three of them. The two-character codes are tried first, because
    /// matching `r` before `r+` would read every `r+` as `r`. What matches nothing is
    /// counted one character at a time rather than dropped: a code we cannot read means an
    /// entry declaring fewer editions than the wiki says, and that has to surface in `meta`.
    pub fn parse_codes(s: &str, d: &mut Diagnostics) -> Vec<Dlc> {
        const CODES: [&str; 5] = ["a+", "r+", "n", "a", "r"];
        let mut out = Vec::new();
        let mut rest = s.trim();
        while !rest.is_empty() {
            match CODES.iter().find(|c| rest.starts_with(**c)) {
                Some(c) => {
                    if let Some(dlc) = Dlc::from_code(c) {
                        out.push(dlc);
                    }
                    rest = &rest[c.len()..];
                }
                None => {
                    let bad = rest.chars().next().map_or(rest.len(), char::len_utf8);
                    d.unknown_dlc_code(&rest[..bad]);
                    rest = &rest[bad..];
                }
            }
        }
        out
    }

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
        requirements: Vec<Inline>,
        /// The thing this achievement unlocks. It does NOT rise to `Entry`: it points the
        /// opposite way from `unlocked_by`, and putting the two in one place is a trap.
        unlocks: Option<Target>,
    },
    Boss {
        base_hp: Option<u32>,
        environment: Vec<Inline>,
        pool: Vec<Inline>,
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
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, to_value};

    #[test]
    fn dlc_codes_split_longest_first_and_leftovers_are_counted() {
        let mut d = Diagnostics::default();
        // One code, the common case: 175 collectible pages say exactly this.
        assert_eq!(Dlc::parse_codes("r", &mut d), vec![Dlc::Repentance]);
        // `r+` must win over `r`: shortest-first would read every `r+` as `r`.
        assert_eq!(Dlc::parse_codes("r+", &mut d), vec![Dlc::RepentancePlus]);
        // The real page that forced this function to exist.
        assert_eq!(
            Dlc::parse_codes("a+nr", &mut d),
            vec![Dlc::AfterbirthPlus, Dlc::Rebirth, Dlc::Repentance]
        );
        // An absent parameter is an empty list, not an error.
        assert_eq!(Dlc::parse_codes("", &mut d), Vec::<Dlc>::new());
        assert!(d.unknown_dlc_codes.is_empty());

        // What we cannot read is counted, never dropped in silence.
        assert_eq!(Dlc::parse_codes("zz", &mut d), Vec::<Dlc>::new());
        assert_eq!(d.unknown_dlc_codes.get("z"), Some(&2));
    }

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
        // `unlockedBy` is gone from the variant: it rose to `Entry` on 2026-09-13.
        assert_eq!(
            to_value(Infobox::Boss {
                base_hp: Some(6666),
                environment: vec![],
                pool: vec![],
            })
            .unwrap(),
            json!({"kind":"boss","baseHp":6666,"environment":[],"pool":[]})
        );
        assert_eq!(
            to_value(SectionKind::ChampionVersions).unwrap(),
            json!("championVersions")
        );
        let e = Entry {
            title: "Hush".into(),
            revid: 1,
            description: vec![],
            dlc: vec![Dlc::Repentance],
            unlocked_by: None,
            infobox: Infobox::Item,
            sections: vec![],
        };
        // `unlockedBy` here is the assertion that `rename_all` is doing its job on `Entry`.
        assert_eq!(
            to_value(e).unwrap(),
            json!({
                "title": "Hush",
                "revid": 1,
                "description": [],
                "dlc": ["repentance"],
                "unlockedBy": null,
                "infobox": {"kind": "item"},
                "sections": []
            })
        );
    }

    #[test]
    fn roundtrip() {
        let e = Entry {
            title: "X".into(),
            revid: 2,
            description: vec![Inline::Text {
                text: "d".into(),
                style: Style::Plain,
            }],
            dlc: vec![Dlc::Rebirth, Dlc::RepentancePlus],
            unlocked_by: Some(Target::Achievement { id: 3 }),
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
