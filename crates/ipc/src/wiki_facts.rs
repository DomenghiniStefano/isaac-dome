//! `PageFacts`: what a wiki page's own infobox states, scalar by scalar, computed once per
//! window in `wiki_index` (design decision 3, `2026-09-27-wiki-restyle-design.md`).
//!
//! **Derived from the `Infobox`, never a second copy of its fields**: `facts` is one
//! exhaustive `match` on `entry.infobox`, so a ninth infobox kind has to say which variant it
//! takes. A field that is already plain text or a scalar in the infobox is carried as-is; a
//! field that is inline wikitext (`Vec<Inline>`) is flattened with `wiki::plain`, the crate's
//! one "read this the way a reader would" function (`graph::unlock::condition_of` already
//! reads an achievement's requirement the same way).

use wiki::{plain, ArticleCategory, Dataset, Entry, Infobox, Target};

/// A version article's own patch, read from the wiki's `version` Cargo table
/// (`wiki::Meta::patches`), matched by the article's own title. Distinct from `wiki::PatchView`
/// (which names the *whole dataset's* most recent patch): this one is a fact about a single
/// page.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct VersionFacts {
    pub number: String,
    pub date: String,
}

/// One page's own facts, one variant per infobox kind (CLAUDE.md: fieldless enums are bare
/// strings, but every variant here carries data, so it is tagged).
#[derive(Debug, Clone, PartialEq, serde::Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PageFacts {
    Item {
        quality: Option<i8>,
        /// From the template name (`CollectibleTemplate::Activated`): the wiki has two
        /// separate infobox templates, `infobox passive collectible` and `infobox activated
        /// collectible`.
        activated: bool,
        recharge: Option<String>,
        shop_price: Option<String>,
        devil_price: Option<String>,
        tags: Vec<String>,
    },
    Trinket {
        tags: Vec<String>,
    },
    Achievement {
        requirement: String,
        unlocks: Option<Target>,
    },
    Boss {
        base_hp: Option<u32>,
        /// The floors the environment names, once each, in the page's order: the stage
        /// references of the infobox's environment table, never its rooms or notes.
        floors: Vec<String>,
    },
    Challenge {
        character: Option<Target>,
        goal: String,
        blindfolded: bool,
        curse: String,
    },
    Character {
        /// Already plain text on `Infobox::Character` for every field but `health`: the
        /// template-default fill (design decision 4 of `2026-09-26-wiki-complete-design.md`)
        /// happens once, at build time, in the wiki crate — `facts` only carries what the
        /// entry already states.
        health: String,
        damage: String,
        tears: String,
        range: String,
        speed: String,
        luck: String,
        shot_speed: String,
        /// Read from the page's own title, not guessed: every Tainted form's title on the
        /// committed snapshot starts with "Tainted " (measured on `dataset/wiki/characters.json`,
        /// 2026-09-27 — ids 21 through 40, "Tainted Isaac" through "Tainted Soul"), and the
        /// infobox itself states no such flag.
        tainted: bool,
    },
    Transformation {
        requires: Option<u32>,
        contributors: u32,
    },
    Entity {
        base_hp: Option<u32>,
        /// The floors the environment names, once each, in the page's order: the stage
        /// references of the infobox's environment table, never its rooms or notes.
        floors: Vec<String>,
    },
    Article {
        category: Option<ArticleCategory>,
        /// Only for `category: Some(Version)`, and only when the wiki's own `version` table
        /// names this exact page.
        version: Option<VersionFacts>,
    },
}

/// `inline` flattened to plain text with `wiki::plain`, or `None` when there is nothing left
/// after trimming: an empty field on the infobox reads as "not stated", not as an empty string
/// the screen would draw as a blank chip.
fn plain_opt(inline: &[wiki::Inline]) -> Option<String> {
    let text = plain(inline);
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

/// The stages `inline` refers to, once each, in the order they first appear, looking inside
/// edition-only runs. The environment field is a table the wiki flattens into one run of text
/// and references glued together; its stage references are the floors, the rest is rooms and
/// notes a list of floors has no place for.
fn stage_names(inline: &[wiki::Inline]) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    collect_stages(inline, &mut names);
    names
}

fn collect_stages(inline: &[wiki::Inline], names: &mut Vec<String>) {
    for node in inline {
        match node {
            wiki::Inline::Ref {
                target: Target::Stage { name },
                ..
            } if !names.contains(name) => names.push(name.clone()),
            wiki::Inline::Edition { inline, .. } => collect_stages(inline, names),
            wiki::Inline::Ref { .. } | wiki::Inline::Text { .. } | wiki::Inline::Concept { .. } => {
            }
        }
    }
}

/// The wiki convention this crate has measured on the committed snapshot: a Tainted form's
/// page title always starts with "Tainted ". Not stated anywhere in the infobox itself.
fn tainted_title(title: &str) -> bool {
    title.starts_with("Tainted ")
}

/// A version article's own patch: the `version` table row whose page is this article's own
/// title, when the wiki's whole-namespace fetch recorded one.
fn version_facts(dataset: &Dataset, title: &str) -> Option<VersionFacts> {
    let patch = dataset.meta.patches.get(title)?;
    Some(VersionFacts {
        number: patch.number.clone(),
        date: patch.date.clone(),
    })
}

/// One page's facts: a short dispatcher, exhaustive over `Infobox` by variant. It names every
/// field of `Article` (the one variant it fully reads itself) but hands the rest to a one-kind
/// helper below, by reference — that is what keeps the dispatcher itself short. Each helper
/// destructures its own variant again, **without `..`**, the same reason and the same shape
/// `Infobox::inlines`/`inlines_mut` have (`wiki::model`): a field added to a variant has to be
/// named there, `_` or not, or the match stops compiling, instead of silently missing a fact.
pub fn facts(entry: &Entry, dataset: &Dataset) -> PageFacts {
    let infobox = &entry.infobox;
    match infobox {
        Infobox::Item { .. } => item_facts(infobox),
        Infobox::Trinket { .. } => trinket_facts(infobox),
        Infobox::Achievement { .. } => achievement_facts(infobox),
        Infobox::Boss { .. } => boss_facts(infobox),
        Infobox::Challenge { .. } => challenge_facts(infobox),
        Infobox::Character { .. } => character_facts(infobox, &entry.title),
        Infobox::Transformation { .. } => transformation_facts(infobox),
        Infobox::Entity { .. } => entity_facts(infobox),
        Infobox::Article { category } => article_facts(*category, dataset, &entry.title),
    }
}

/// A helper's own variant, or a panic — never reached, because `facts` above already matched
/// the variant it names before calling in: the `else` would mean this file's own dispatch
/// disagrees with itself, never a save or a wiki page, which is why it panics rather than
/// degrading. One macro rather than the same `let … else { unreachable!() }` copied eight
/// times, one per helper below.
macro_rules! own_variant {
    ($infobox:expr, $pat:pat) => {
        let $pat = $infobox else {
            unreachable!("facts: dispatcher and helper disagree on the infobox kind");
        };
    };
}

/// `Infobox::Item`'s own facts.
fn item_facts(infobox: &Infobox) -> PageFacts {
    own_variant!(
        infobox,
        Infobox::Item {
            quote: _,
            template,
            quality,
            tags,
            recharge,
            devil_price,
            shop_price,
            obtained_from: _,
        }
    );
    PageFacts::Item {
        quality: *quality,
        activated: *template == wiki::CollectibleTemplate::Activated,
        recharge: plain_opt(recharge),
        shop_price: plain_opt(shop_price),
        devil_price: plain_opt(devil_price),
        tags: tags.clone(),
    }
}

/// `Infobox::Trinket`'s own facts.
fn trinket_facts(infobox: &Infobox) -> PageFacts {
    own_variant!(
        infobox,
        Infobox::Trinket {
            quote: _,
            tags,
            obtained_from: _,
        }
    );
    PageFacts::Trinket { tags: tags.clone() }
}

/// `Infobox::Achievement`'s own facts.
fn achievement_facts(infobox: &Infobox) -> PageFacts {
    own_variant!(
        infobox,
        Infobox::Achievement {
            quote: _,
            requirements,
            notes: _,
            unlocks,
        }
    );
    PageFacts::Achievement {
        requirement: plain(requirements),
        unlocks: unlocks.clone(),
    }
}

/// `Infobox::Boss`'s own facts.
fn boss_facts(infobox: &Infobox) -> PageFacts {
    own_variant!(
        infobox,
        Infobox::Boss {
            base_hp,
            stage_hp: _,
            variant: _,
            environment,
            pool: _,
        }
    );
    PageFacts::Boss {
        base_hp: *base_hp,
        floors: stage_names(environment),
    }
}

/// `Infobox::Challenge`'s own facts.
fn challenge_facts(infobox: &Infobox) -> PageFacts {
    own_variant!(
        infobox,
        Infobox::Challenge {
            blindfolded,
            has_shops: _,
            has_treasure_rooms: _,
            items: _,
            trinkets: _,
            pickups: _,
            health: _,
            curse,
            goal,
            character,
            unlocks: _,
        }
    );
    PageFacts::Challenge {
        character: character.clone(),
        goal: plain(goal),
        blindfolded: *blindfolded,
        curse: plain(curse),
    }
}

/// `Infobox::Character`'s own facts: seven stat strings, already plain text but for `health`,
/// and the Tainted read off `title`.
fn character_facts(infobox: &Infobox, title: &str) -> PageFacts {
    own_variant!(
        infobox,
        Infobox::Character {
            health,
            damage,
            tears,
            range,
            speed,
            luck,
            shot_speed,
            pickups: _,
            collectibles: _,
            parent: _,
        }
    );
    PageFacts::Character {
        health: plain(health),
        damage: damage.clone(),
        tears: tears.clone(),
        range: range.clone(),
        speed: speed.clone(),
        luck: luck.clone(),
        shot_speed: shot_speed.clone(),
        tainted: tainted_title(title),
    }
}

/// `Infobox::Transformation`'s own facts.
fn transformation_facts(infobox: &Infobox) -> PageFacts {
    own_variant!(
        infobox,
        Infobox::Transformation {
            requires,
            contributors,
            target: _,
        }
    );
    PageFacts::Transformation {
        requires: *requires,
        contributors: contributors.len() as u32,
    }
}

/// `Infobox::Entity`'s own facts.
fn entity_facts(infobox: &Infobox) -> PageFacts {
    own_variant!(
        infobox,
        Infobox::Entity {
            base_hp,
            stage_hp: _,
            environment,
            behavior: _,
            pool: _,
            replace: _,
            replace_chance: _,
            replace_notes: _,
        }
    );
    PageFacts::Entity {
        base_hp: *base_hp,
        floors: stage_names(environment),
    }
}

/// `Infobox::Article`'s own facts: the category, and a version's own row of the wiki's
/// `version` table when it has one. Already destructured by the dispatcher above (one field),
/// so it takes the category directly rather than the whole infobox.
fn article_facts(category: Option<ArticleCategory>, dataset: &Dataset, title: &str) -> PageFacts {
    PageFacts::Article {
        category,
        version: (category == Some(ArticleCategory::Version))
            .then(|| version_facts(dataset, title))
            .flatten(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiki::for_tests::{empty_dataset, entry};
    use wiki::{CollectibleTemplate, Inline, Patch, Style};

    fn text(s: &str) -> Vec<Inline> {
        vec![Inline::Text {
            text: s.into(),
            style: Style::Plain,
        }]
    }

    #[test]
    fn an_item_carries_its_quality_and_template() {
        let e = entry(
            "X",
            Infobox::Item {
                quote: vec![],
                template: CollectibleTemplate::Activated,
                quality: Some(3),
                tags: vec![],
                recharge: text("6"),
                devil_price: vec![],
                shop_price: vec![],
                obtained_from: vec![],
            },
        );
        let f = facts(&e, &empty_dataset());
        assert_eq!(
            f,
            PageFacts::Item {
                quality: Some(3),
                activated: true,
                recharge: Some("6".into()),
                shop_price: None,
                devil_price: None,
                tags: vec![],
            }
        );
    }

    /// The template-default fill happens in the wiki crate, before `facts` ever sees the
    /// entry: a character with no declared damage still carries whatever string the entry
    /// states (design decision 4 of `2026-09-26-wiki-complete-design.md`), unchanged.
    #[test]
    fn a_characters_stat_strings_pass_through_unchanged() {
        let e = entry(
            "Isaac",
            Infobox::Character {
                health: text("3 red hearts"),
                damage: "3.50".into(),
                tears: "2.73".into(),
                range: "6.5".into(),
                speed: "1.0".into(),
                luck: "0".into(),
                shot_speed: "1.0".into(),
                pickups: vec![],
                collectibles: vec![],
                parent: None,
            },
        );
        let f = facts(&e, &empty_dataset());
        assert_eq!(
            f,
            PageFacts::Character {
                health: "3 red hearts".into(),
                damage: "3.50".into(),
                tears: "2.73".into(),
                range: "6.5".into(),
                speed: "1.0".into(),
                luck: "0".into(),
                shot_speed: "1.0".into(),
                tainted: false,
            }
        );
    }

    #[test]
    fn a_tainted_characters_title_names_it() {
        let e = entry(
            "Tainted Isaac",
            Infobox::Character {
                health: vec![],
                damage: String::new(),
                tears: String::new(),
                range: String::new(),
                speed: String::new(),
                luck: String::new(),
                shot_speed: String::new(),
                pickups: vec![],
                collectibles: vec![],
                parent: None,
            },
        );
        let PageFacts::Character { tainted, .. } = facts(&e, &empty_dataset()) else {
            panic!("expected a character");
        };
        assert!(tainted);
    }

    #[test]
    fn a_version_article_reads_its_own_row_of_the_table_by_title() {
        let mut ds = empty_dataset();
        ds.meta.patches.insert(
            "V1.9.7.17".into(),
            Patch {
                number: "v1.9.7.17".into(),
                date: "2026-04-20".into(),
            },
        );
        let e = entry(
            "V1.9.7.17",
            Infobox::Article {
                category: Some(ArticleCategory::Version),
            },
        );
        assert_eq!(
            facts(&e, &ds),
            PageFacts::Article {
                category: Some(ArticleCategory::Version),
                version: Some(VersionFacts {
                    number: "v1.9.7.17".into(),
                    date: "2026-04-20".into(),
                }),
            }
        );
    }

    #[test]
    fn an_article_with_no_row_in_the_table_carries_no_version() {
        let e = entry(
            "A Page Nobody Fetched Versions For",
            Infobox::Article {
                category: Some(ArticleCategory::Version),
            },
        );
        assert_eq!(
            facts(&e, &empty_dataset()),
            PageFacts::Article {
                category: Some(ArticleCategory::Version),
                version: None,
            }
        );
    }

    #[test]
    fn a_non_version_article_never_looks_up_the_table() {
        let mut ds = empty_dataset();
        ds.meta.patches.insert(
            "Cards".into(),
            Patch {
                number: "v0".into(),
                date: "2014-01-01".into(),
            },
        );
        let e = entry(
            "Cards",
            Infobox::Article {
                category: Some(ArticleCategory::Card),
            },
        );
        assert_eq!(
            facts(&e, &ds),
            PageFacts::Article {
                category: Some(ArticleCategory::Card),
                version: None,
            }
        );
    }

    #[test]
    fn empty_inline_fields_are_absent_not_blank() {
        let e = entry(
            "X",
            Infobox::Boss {
                base_hp: Some(300),
                stage_hp: vec![],
                variant: None,
                environment: vec![],
                pool: vec![],
            },
        );
        assert_eq!(
            facts(&e, &empty_dataset()),
            PageFacts::Boss {
                base_hp: Some(300),
                floors: vec![],
            }
        );
    }

    fn floor(name: &str) -> Inline {
        Inline::Ref {
            target: Target::Stage { name: name.into() },
            label: name.into(),
        }
    }

    #[test]
    fn floors_are_the_stages_the_environment_names_once_each_in_order() {
        // Monstro's shape on the snapshot: stage refs glued together, with rooms, notes and
        // an edition-only run in between, and a stage repeated further down the table.
        let environment = vec![
            text("Boss ").remove(0),
            floor("Basement"),
            floor("Burning Basement"),
            Inline::Ref {
                target: Target::Room {
                    name: "boss rush".into(),
                },
                label: "boss rush".into(),
            },
            text(" Double Trouble ").remove(0),
            Inline::Edition {
                only: vec![wiki::Dlc::Repentance],
                inline: vec![floor("Caves")],
            },
            floor("Basement"),
        ];
        let e = entry(
            "Monstro",
            Infobox::Boss {
                base_hp: Some(250),
                stage_hp: vec![],
                variant: None,
                environment,
                pool: vec![],
            },
        );
        assert_eq!(
            facts(&e, &empty_dataset()),
            PageFacts::Boss {
                base_hp: Some(250),
                floors: vec!["Basement".into(), "Burning Basement".into(), "Caves".into()],
            }
        );
    }
}
