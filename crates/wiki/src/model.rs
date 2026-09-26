//! The typed tree. Everything crosses the IPC as-is: camelCase, enums with data tagged by
//! `kind`, fieldless enums as a bare string.

use serde::{Deserialize, Serialize};

/// A wiki page reduced to what's needed: the infobox and the text sections that are kept.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub title: String,
    /// Wiki revision the page was read from: says how stale the data is.
    pub revid: u64,
    /// The summary line: the infobox's `description`, or on a boss, a character or a
    /// challenge whose infobox has none, the page's opening paragraph. **Empty on every
    /// achievement from the wiki**: what the wiki files there is the unlock paper's line,
    /// which is `Infobox::Achievement.quote`, and the summary the screen draws is composed
    /// from `unlocks` and `requirements` where the words can be translated. A hand-written
    /// description in `corrections.json` fills it, on any kind, and wins over the wiki.
    pub description: Vec<Inline>,
    /// The editions the **infobox** says the entry exists in, in release order. Empty when
    /// the parameter is absent, which is the wiki's "no restriction" and not "it exists
    /// nowhere"; a page that writes the restriction out as `n` lists all five.
    ///
    /// The parameter holds a code, and a code names a **range**: `r` is "added in
    /// Repentance", so it lists Repentance and Repentance+. Read one code at a time it came
    /// out too narrow on 1078 of the 1083 pages that carry one, until 2026-09-15.
    ///
    /// Same source as the Cargo tables' `dlc` integer, which is that code already resolved
    /// to its bitmask (1 Rebirth, 2 Afterbirth, 4 Afterbirth+, 8 Repentance,
    /// 16 Repentance+); the two agree row for row on the collectible table. This field
    /// still does not read it, because a page has an infobox whether or not it has a Cargo
    /// row.
    pub dlc: Vec<Dlc>,
    /// What the wiki states has to be unlocked first. `None` means "the wiki does not state
    /// one", NEVER "it is free from the start": that answer belongs to `catalog` and `graph`.
    pub unlocked_by: Option<Target>,
    pub infobox: Infobox,
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    pub kind: SectionKind,
    pub blocks: Vec<Block>,
}

/// Fieldless: a bare string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ts_rs::TS)]
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
    /// What a character unlocks to start a run with. A kind apart from `Unlockable` because
    /// the eight pages that have it have both, one under the other, and two sections with one
    /// name read as the same list twice.
    StartingItems,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ListItem {
    pub inline: Vec<Inline>,
    /// What sits under the item: the lists nested in it, or — under an item naming an
    /// achievement — that achievement's unlock condition, as a paragraph.
    pub children: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum CollectibleTemplate {
    Passive,
    Activated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum Style {
    Plain,
    Bold,
    Italic,
}

/// The identity of a wiki element: what a `ref` points to, and what a page load accepts.
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
    Concept { name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum Dlc {
    Rebirth,
    Afterbirth,
    AfterbirthPlus,
    Repentance,
    RepentancePlus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
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
        /// The line on the game's unlock paper, which the wiki files under `description`:
        /// "Just Stop!", "OMG!". It is the game's voice and not a description, so it is the
        /// quote, the way an item's pickup line is. Empty where the wiki wrote a placeholder
        /// ("???" on 136 rows) or nothing.
        quote: Vec<Inline>,
        requirements: Vec<Inline>,
        /// Caveats on the requirement ("Possession of The Polaroid is required…"). An
        /// achievement is a row on a storage page and carries no sections of its own, so
        /// this is the only prose it has beyond `quote` and `requirements`.
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

impl Infobox {
    /// The same fields as [`Self::inlines_mut`], read-only, destructured the same way and
    /// **without `..`**, so a field added to a variant breaks the build here instead of
    /// slipping past the guard over the dataset's text — which, naming its fields by hand,
    /// once skipped five of them. Only that guard, a test, reads it.
    #[cfg(feature = "test-api")]
    pub fn inlines(&self) -> Vec<&Vec<Inline>> {
        match self {
            Infobox::Item {
                quote,
                template: _,
                quality: _,
                tags: _,
                recharge,
                devil_price,
                shop_price,
                pools,
            } => vec![quote, recharge, devil_price, shop_price, pools],
            Infobox::Trinket {
                quote,
                tags: _,
                pools,
            } => vec![quote, pools],
            Infobox::Achievement {
                quote,
                requirements,
                notes,
                unlocks: _,
            } => vec![quote, requirements, notes],
            Infobox::Boss {
                base_hp: _,
                stage_hp,
                variant: _,
                environment,
                pool,
            } => vec![stage_hp, environment, pool],
            Infobox::Challenge {
                blindfolded: _,
                has_shops: _,
                has_treasure_rooms: _,
                items,
                trinkets,
                pickups,
                health,
                curse,
                goal,
                character: _,
                unlocks: _,
            } => vec![items, trinkets, pickups, health, curse, goal],
            Infobox::Transformation {
                requires: _,
                contributors: _,
                target,
            } => vec![target],
            Infobox::Character {
                health,
                damage: _,
                tears: _,
                range: _,
                speed: _,
                luck: _,
                shot_speed: _,
                pickups,
                collectibles,
                parent: _,
            } => vec![health, pickups, collectibles],
        }
    }

    /// Every inline field, so a pass over a page's text does not have to name them one by
    /// one. The match destructures each variant **without `..`**, so a field added later
    /// breaks the build instead of quietly staying outside every such pass.
    pub fn inlines_mut(&mut self) -> Vec<&mut Vec<Inline>> {
        match self {
            Infobox::Item {
                quote,
                template: _,
                quality: _,
                tags: _,
                recharge,
                devil_price,
                shop_price,
                pools,
            } => vec![quote, recharge, devil_price, shop_price, pools],
            Infobox::Trinket {
                quote,
                tags: _,
                pools,
            } => vec![quote, pools],
            Infobox::Achievement {
                quote,
                requirements,
                notes,
                unlocks: _,
            } => vec![quote, requirements, notes],
            Infobox::Boss {
                base_hp: _,
                stage_hp,
                variant: _,
                environment,
                pool,
            } => vec![stage_hp, environment, pool],
            Infobox::Challenge {
                blindfolded: _,
                has_shops: _,
                has_treasure_rooms: _,
                items,
                trinkets,
                pickups,
                health,
                curse,
                goal,
                character: _,
                unlocks: _,
            } => vec![items, trinkets, pickups, health, curse, goal],
            Infobox::Transformation {
                requires: _,
                contributors: _,
                target,
            } => vec![target],
            Infobox::Character {
                health,
                damage: _,
                tears: _,
                range: _,
                speed: _,
                luck: _,
                shot_speed: _,
                pickups,
                collectibles,
                parent: _,
            } => vec![health, pickups, collectibles],
        }
    }
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
