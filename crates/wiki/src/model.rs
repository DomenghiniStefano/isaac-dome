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
    /// The edition codes the **infobox** declares, parsed. Empty when the parameter is
    /// absent, which is not the same as "it exists everywhere".
    ///
    /// Not to be confused with the Cargo tables' `dlc` integer, which is a different source
    /// and a different question: that one is a bitmask over the editions a row is valid in,
    /// measured on 2026-09-13 to agree with the game on 712 of 720 collectibles
    /// (`cargo run -q -p ipc --example dlc_mask`). This field does not use it.
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

/// Which of the wiki's two collectible templates the page used. Fieldless, so a bare
/// camelCase string like `SectionKind` and `Style`.
///
/// It is deliberately NOT `catalog`'s three-way item kind. The wiki has exactly two
/// templates and no familiar one, so a familiar is written with the passive template: this
/// type reports which template was read, and claims nothing about what the item is. A
/// `bool` would have been worse than either — `activated: false` would silently mean both
/// "passive" and "familiar".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CollectibleTemplate {
    Passive,
    Activated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Style {
    Plain,
    Bold,
    Italic,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ts_rs::TS)]
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
    Item {
        /// The pickup quote. Inline, not a string: 78 of 719 carry edition markup
        /// (`Boomerang tears {{dlc|r|+ DMG up + luck down}}`), and read as raw text they
        /// put wikitext on screen. Flattened, it is `items.xml`'s `description` attribute —
        /// which `wiki_agrees_with_catalog` checks rather than assumes.
        quote: Vec<Inline>,
        /// From the template name: the wiki has `infobox passive collectible` and
        /// `infobox activated collectible`, and until 2026-09-13 we merged the two.
        template: CollectibleTemplate,
        /// `-1..=4`, as `catalog::Metadata::quality`.
        quality: Option<i8>,
        /// The game's vocabulary, open by nature: a closed enum breaks the day it grows.
        tags: Vec<String>,
        /// Not a number. The real values include `unlimited`, `one time`, `4s` and
        /// `{{dlcalt|6|r=4}}`; a numeric parse would discard about a third of them.
        recharge: Vec<Inline>,
        /// Not a number either: 36 of the 56 real `devil price` values are per-edition.
        devil_price: Vec<Inline>,
        shop_price: Vec<Inline>,
        /// What the wiki says about the pools. Present on only 45 of 720 pages: the
        /// game's `itempools.xml` is the source that knows them all.
        pools: Vec<Inline>,
    },
    Trinket {
        quote: Vec<Inline>,
        tags: Vec<String>,
        pools: Vec<Inline>,
    },
    Achievement {
        requirements: Vec<Inline>,
        /// Caveats on the requirement ("Possession of The Polaroid is required…"). An
        /// achievement is a row on a storage page and carries no sections of its own, so
        /// this is the only prose it has beyond `description` and `requirements`.
        notes: Vec<Inline>,
        /// The thing this achievement unlocks. It does NOT rise to `Entry`: it points the
        /// opposite way from `unlocked_by`, and putting the two in one place is a trap.
        unlocks: Option<Target>,
    },
    Boss {
        base_hp: Option<u32>,
        /// Inline, not a number: the two real values are per-stage notes, not a scalar.
        stage_hp: Vec<Inline>,
        /// The bestiary variant, when the infobox states one.
        variant: Option<u32>,
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
        /// The character the challenge is played as, when it forces one.
        character: Option<Target>,
        unlocks: Option<Target>,
    },
    /// A transformation, and the only variant completed from outside its own infobox: the
    /// count and the set are stated in the page body, not in the box (spec §2.2, §2.4).
    Transformation {
        /// How many of `contributors` are needed. `None` when the page does not say it in a
        /// form we can read — never defaulted to three, which is what every page that does
        /// say it says, and therefore what a wrong default would be invisible against.
        requires: Option<u32>,
        /// The items and trinkets that count, in page order, deduplicated. The union of the
        /// infobox's `items` and the body's own tables: each loses something the other has.
        contributors: Vec<Target>,
        /// What the transformation acts on ("Isaac's bums"). Kept because the Cargo table
        /// declares it, and `no_silent_parameter` fails on a parameter that is neither a
        /// field nor deliberately ignored.
        target: Vec<Inline>,
    },
    Character {
        health: Vec<Inline>,
        damage: String,
        /// The fire-rate stat. It was the only one of the six the type did not carry.
        tears: String,
        range: String,
        speed: String,
        luck: String,
        shot_speed: String,
        pickups: Vec<Inline>,
        collectibles: Vec<Inline>,
        /// The character this one is a variant of (Lazarus Risen's Lazarus, Tainted's base).
        parent: Option<Target>,
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
        // `Item` stopped being a fieldless variant on 2026-09-13. It is tagged, so this is
        // additive on the wire: a TypeScript `switch` on `kind === 'item'` keeps narrowing.
        // Every camelCase key here is the assertion that `rename_all_fields` is applied.
        assert_eq!(
            to_value(Infobox::Item {
                quote: vec![Inline::Text {
                    text: "Blood laser barrage".into(),
                    style: Style::Plain,
                }],
                template: CollectibleTemplate::Passive,
                quality: Some(4),
                tags: vec!["devil".into()],
                recharge: vec![],
                devil_price: vec![],
                shop_price: vec![],
                pools: vec![],
            })
            .unwrap(),
            json!({
                "kind": "item",
                "quote": [{"kind": "text", "text": "Blood laser barrage", "style": "plain"}],
                "template": "passive",
                "quality": 4,
                "tags": ["devil"],
                "recharge": [],
                "devilPrice": [],
                "shopPrice": [],
                "pools": []
            })
        );
        assert_eq!(
            to_value(Infobox::Trinket {
                quote: vec![],
                tags: vec![],
                pools: vec![],
            })
            .unwrap(),
            json!({"kind":"trinket","quote":[],"tags":[],"pools":[]})
        );
        // `unlockedBy` is gone from the variant: it rose to `Entry` on 2026-09-13.
        assert_eq!(
            to_value(Infobox::Boss {
                base_hp: Some(6666),
                stage_hp: vec![],
                variant: Some(1),
                environment: vec![],
                pool: vec![],
            })
            .unwrap(),
            json!({
                "kind": "boss", "baseHp": 6666, "stageHp": [], "variant": 1,
                "environment": [], "pool": []
            })
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
            infobox: Infobox::Item {
                quote: vec![],
                template: CollectibleTemplate::Passive,
                quality: None,
                tags: vec![],
                recharge: vec![],
                devil_price: vec![],
                shop_price: vec![],
                pools: vec![],
            },
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
                "infobox": {
                    "kind": "item",
                    "quote": [],
                    "template": "passive",
                    "quality": null,
                    "tags": [],
                    "recharge": [],
                    "devilPrice": [],
                    "shopPrice": [],
                    "pools": []
                },
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
            infobox: Infobox::Trinket {
                quote: vec![],
                tags: vec![],
                pools: vec![],
            },
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
