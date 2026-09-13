//! Infoboxes: extracted as top-level templates, converted into the `Infobox` of their kind.

use std::collections::BTreeMap;

use crate::inline::parse_inline;
use crate::resolver::Resolver;
use crate::template::parse_template_at;
use crate::{CollectibleTemplate, Diagnostics, Dlc, Infobox, Inline, Target};

/// An `{{infobox …}}` template as-is: lowercase name and raw named parameters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawInfobox {
    pub name: String,
    pub params: BTreeMap<String, String>,
}

/// Every top-level `{{infobox …}}`, in the order they appear. Other templates are
/// skipped whole, so an infobox nested inside another template does not count.
pub fn extract_infoboxes(text: &str) -> Vec<RawInfobox> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(pos) = text.get(i..).and_then(|rest| rest.find("{{")) {
        let at = i + pos;
        match parse_template_at(text, at) {
            Some((t, end)) => {
                if t.name.starts_with("infobox") {
                    out.push(RawInfobox {
                        name: t.name,
                        params: t.named,
                    });
                }
                i = end;
            }
            None => i = at + 2, // `{{` with no close: resume from the next character
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfoboxKind {
    Passive,
    Activated,
    Trinket,
    Achievement,
    Boss,
    Challenge,
    Character,
}

impl InfoboxKind {
    /// From the template name (already lowercase) to the infobox kind.
    pub fn of(name: &str) -> Option<InfoboxKind> {
        Some(match name {
            // `infobox collectible` with no adjective is the generic form, and the game
            // has no third kind of collectible: it counts as passive.
            "infobox passive collectible" | "infobox collectible" => InfoboxKind::Passive,
            "infobox activated collectible" => InfoboxKind::Activated,
            "infobox trinket" => InfoboxKind::Trinket,
            "infobox achievement" => InfoboxKind::Achievement,
            "infobox boss" => InfoboxKind::Boss,
            "infobox challenge" => InfoboxKind::Challenge,
            "infobox character" => InfoboxKind::Character,
            _ => return None, // allowed: template name, an open-ended string
        })
    }
}

fn param<'a>(ib: &'a RawInfobox, name: &str) -> &'a str {
    ib.params.get(name).map(String::as_str).unwrap_or("")
}

fn text(ib: &RawInfobox, name: &str) -> String {
    param(ib, name).trim().to_string()
}

fn inline(ib: &RawInfobox, name: &str, r: &Resolver, d: &mut Diagnostics) -> Vec<Inline> {
    parse_inline(param(ib, name), r, d)
}

fn yes(ib: &RawInfobox, name: &str) -> bool {
    param(ib, name).trim().eq_ignore_ascii_case("yes")
}

/// `"devil summonable offensive"` → three tags. The vocabulary is the game's and open, so
/// this stays a list of strings: a closed enum breaks the day the game adds a tag.
fn tags(ib: &RawInfobox, name: &str) -> Vec<String> {
    param(ib, name)
        .split_whitespace()
        .map(str::to_string)
        .collect()
}

/// The leading digits: `"250 (x2)"` → 250, `"6666"` → 6666, no digits → `None`.
pub(crate) fn leading_number(s: &str) -> Option<u32> {
    s.trim()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .ok()
}

/// The three facts every kind declares, read once per infobox and carried on `Entry`
/// instead of being repeated in all six variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryFacts {
    pub description: Vec<Inline>,
    pub dlc: Vec<Dlc>,
    pub unlocked_by: Option<Target>,
}

/// `unlocked by` is an achievement *name* on every kind that uses it — the 277 collectible
/// pages that carry it read "???'s Only Friend", "A Pound of Flesh", … — so one resolution
/// serves all six kinds.
pub fn entry_facts(ib: &RawInfobox, r: &Resolver, d: &mut Diagnostics) -> EntryFacts {
    EntryFacts {
        description: inline(ib, "description", r, d),
        dlc: Dlc::parse_codes(param(ib, "dlc"), d),
        unlocked_by: r.achievement_by_name(param(ib, "unlocked by")),
    }
}

/// A collectible's infobox. The template it came from is passed in rather than read again,
/// because only the caller's `match` knows which of the two names the page used.
fn item_from(
    ib: &RawInfobox,
    template: CollectibleTemplate,
    r: &Resolver,
    d: &mut Diagnostics,
) -> Infobox {
    Infobox::Item {
        quote: text(ib, "quote"),
        template,
        quality: param(ib, "quality").trim().parse().ok(),
        tags: tags(ib, "tags"),
        recharge: inline(ib, "recharge", r, d),
        devil_price: inline(ib, "devil price", r, d),
        shop_price: inline(ib, "shop price", r, d),
        pools: inline(ib, "pool", r, d),
    }
}

/// Converts a raw infobox into the `Infobox` of its kind. Missing parameters count as an
/// empty string: a missing field degrades, it doesn't block the page.
pub fn infobox_from(
    kind: InfoboxKind,
    ib: &RawInfobox,
    r: &Resolver,
    d: &mut Diagnostics,
) -> Infobox {
    match kind {
        // Two arms rather than one with an inner `match kind`: an inner match would need a
        // `_` arm for the kinds this branch cannot see, and a `_` on a closed enum is what
        // stops a new variant from breaking the build.
        InfoboxKind::Passive => item_from(ib, CollectibleTemplate::Passive, r, d),
        InfoboxKind::Activated => item_from(ib, CollectibleTemplate::Activated, r, d),
        InfoboxKind::Trinket => Infobox::Trinket {
            quote: text(ib, "quote"),
            tags: tags(ib, "tags"),
            pools: inline(ib, "pool", r, d),
        },
        InfoboxKind::Achievement => Infobox::Achievement {
            requirements: inline(ib, "requirements", r, d),
            unlocks: r.by_page_title(param(ib, "link")),
        },
        InfoboxKind::Boss => Infobox::Boss {
            base_hp: leading_number(param(ib, "base hp")),
            stage_hp: inline(ib, "stage hp", r, d),
            variant: leading_number(param(ib, "variant")),
            environment: inline(ib, "environment", r, d),
            pool: inline(ib, "pool", r, d),
        },
        InfoboxKind::Challenge => Infobox::Challenge {
            blindfolded: yes(ib, "blindfolded"),
            has_shops: yes(ib, "has shops"),
            has_treasure_rooms: yes(ib, "has treasure rooms"),
            items: inline(ib, "item", r, d),
            trinkets: inline(ib, "trinket", r, d),
            pickups: inline(ib, "pickup", r, d),
            health: inline(ib, "health", r, d),
            curse: inline(ib, "curse", r, d),
            goal: inline(ib, "goal", r, d),
            character: r.by_page_title(param(ib, "character")),
            // `unlocks` is usually a page title; for achievements it's the name.
            unlocks: r
                .by_page_title(param(ib, "unlocks"))
                .or_else(|| r.achievement_by_name(param(ib, "unlocks"))),
        },
        InfoboxKind::Character => Infobox::Character {
            health: inline(ib, "health", r, d),
            damage: text(ib, "damage"),
            tears: text(ib, "tears"),
            range: text(ib, "range"),
            speed: text(ib, "speed"),
            luck: text(ib, "luck"),
            shot_speed: text(ib, "shot speed"),
            pickups: inline(ib, "pickups", r, d),
            collectibles: inline(ib, "collectibles", r, d),
            parent: r.by_page_title(param(ib, "parent")),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolver::fixtures::test_resolver;
    use crate::{Diagnostics, Infobox, Inline, Target};

    fn raw(name: &str, pairs: &[(&str, &str)]) -> RawInfobox {
        RawInfobox {
            name: name.into(),
            params: pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    #[test]
    fn extracts_many_infoboxes_from_a_storage_page() {
        let src = "{{storage page}}\n{{infobox achievement\n | name = Epic Fetus\n | link = Epic Fetus\n | description = Unlocked a new item.\n | requirements = Complete {{chal|The Family Man}}\n | id = 62\n}} {{infobox achievement\n | name = Cain\n | id = 2\n}}";
        let v = extract_infoboxes(src);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].name, "infobox achievement");
        assert_eq!(v[0].params.get("id").map(String::as_str), Some("62"));
        assert_eq!(v[1].params.get("name").map(String::as_str), Some("Cain"));
    }

    #[test]
    fn a_collectible_infobox_keeps_its_parameters() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        // Brimstone's real infobox, trimmed to the parameters this type holds.
        let ib = raw(
            "infobox passive collectible",
            &[
                ("quote", "Blood laser barrage"),
                ("quality", "4"),
                ("tags", "devil summonable offensive"),
                ("devil price", "2"),
            ],
        );
        let Infobox::Item {
            quote,
            template,
            quality,
            tags,
            recharge,
            devil_price,
            shop_price,
            pools,
        } = infobox_from(InfoboxKind::Passive, &ib, &r, &mut d)
        else {
            panic!()
        };
        assert_eq!(quote, "Blood laser barrage");
        assert_eq!(template, CollectibleTemplate::Passive);
        assert_eq!(quality, Some(4));
        assert_eq!(tags, vec!["devil", "summonable", "offensive"]);
        assert!(recharge.is_empty());
        assert!(shop_price.is_empty());
        assert!(pools.is_empty());
        // A price is inline, not a number: 36 of the 56 real values are `{{dlcalt|…}}`.
        assert!(matches!(
            devil_price.first(),
            Some(Inline::Text { text, .. }) if text.trim() == "2"
        ));
    }

    #[test]
    fn an_activated_collectible_is_marked_activated() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        let ib = raw("infobox activated collectible", &[("recharge", "6")]);
        let Infobox::Item {
            template, recharge, ..
        } = infobox_from(InfoboxKind::Activated, &ib, &r, &mut d)
        else {
            panic!()
        };
        assert_eq!(template, CollectibleTemplate::Activated);
        assert!(!recharge.is_empty());
    }

    #[test]
    fn a_trinket_infobox_keeps_its_parameters() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        let ib = raw(
            "infobox trinket",
            &[("quote", "Imaginary Friend"), ("tags", "offensive")],
        );
        let Infobox::Trinket { quote, tags, pools } =
            infobox_from(InfoboxKind::Trinket, &ib, &r, &mut d)
        else {
            panic!()
        };
        assert_eq!(quote, "Imaginary Friend");
        assert_eq!(tags, vec!["offensive"]);
        assert!(pools.is_empty());
    }

    #[test]
    fn kinds() {
        assert_eq!(
            InfoboxKind::of("infobox passive collectible"),
            Some(InfoboxKind::Passive)
        );
        assert_eq!(
            InfoboxKind::of("infobox activated collectible"),
            Some(InfoboxKind::Activated)
        );
        assert_eq!(
            InfoboxKind::of("infobox trinket"),
            Some(InfoboxKind::Trinket)
        );
        assert_eq!(InfoboxKind::of("infobox boss"), Some(InfoboxKind::Boss));
        assert_eq!(InfoboxKind::of("infobox"), None);
    }

    #[test]
    fn achievement_and_boss() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        let ib = raw(
            "infobox achievement",
            &[
                ("description", "Unlocked a new item."),
                ("requirements", "Complete {{chal|The Family Man}}"),
                ("link", "Breakfast"),
            ],
        );
        let Infobox::Achievement {
            requirements,
            unlocks,
        } = infobox_from(InfoboxKind::Achievement, &ib, &r, &mut d)
        else {
            panic!()
        };
        // `description` is no longer here: it rose to `Entry`, and `entry_facts` reads it.
        // An achievement's is plain text, so it arrives as a single `Inline::Text`.
        let facts = entry_facts(&ib, &r, &mut d);
        assert!(matches!(
            facts.description.first(),
            Some(Inline::Text { text, .. }) if text == "Unlocked a new item."
        ));
        assert!(requirements.iter().any(|i| matches!(
            i,
            Inline::Ref {
                target: Target::Challenge { number: 19 },
                ..
            }
        )));
        assert_eq!(unlocks, Some(Target::Item { id: 25 }));

        let ib = raw(
            "infobox boss",
            &[("base hp", "250 (x2)"), ("unlocked by", "Epic Fetus")],
        );
        let Infobox::Boss { base_hp, .. } = infobox_from(InfoboxKind::Boss, &ib, &r, &mut d) else {
            panic!()
        };
        assert_eq!(base_hp, Some(250));
        // Same move: the boss's `unlocked by` is now one of the three common facts.
        assert_eq!(
            entry_facts(&ib, &r, &mut d).unlocked_by,
            Some(Target::Achievement { id: 62 })
        );
    }

    #[test]
    fn entry_facts_reads_the_three_common_parameters() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        let ib = raw(
            "infobox passive collectible",
            &[
                ("description", "Tears are replaced with {{i|Breakfast}}"),
                ("dlc", "a+nr"),
                ("unlocked by", "Epic Fetus"),
            ],
        );
        let facts = entry_facts(&ib, &r, &mut d);
        // The description is wikitext, not a string: its links have to survive the move.
        assert!(facts.description.iter().any(|i| matches!(
            i,
            Inline::Ref {
                target: Target::Item { id: 25 },
                ..
            }
        )));
        assert_eq!(
            facts.dlc,
            vec![Dlc::AfterbirthPlus, Dlc::Rebirth, Dlc::Repentance]
        );
        assert_eq!(facts.unlocked_by, Some(Target::Achievement { id: 62 }));

        // An infobox that declares none of the three degrades to empty, never to an error.
        let bare = raw("infobox trinket", &[("id", "1")]);
        let facts = entry_facts(&bare, &r, &mut d);
        assert!(facts.description.is_empty());
        assert!(facts.dlc.is_empty());
        assert_eq!(facts.unlocked_by, None);
    }

    #[test]
    fn the_four_parsed_kinds_keep_the_parameters_they_used_to_drop() {
        let r = test_resolver();
        let mut d = Diagnostics::default();

        // A boss's variant and its per-stage hp: 26 and 2 real pages carry them.
        let ib = raw(
            "infobox boss",
            &[
                ("base hp", "250 (x2)"),
                ("variant", "1"),
                ("stage hp", "300"),
            ],
        );
        let Infobox::Boss {
            variant, stage_hp, ..
        } = infobox_from(InfoboxKind::Boss, &ib, &r, &mut d)
        else {
            panic!()
        };
        assert_eq!(variant, Some(1));
        assert!(!stage_hp.is_empty());

        // The character a challenge is played as: 14 real pages say it, and until
        // 2026-09-13 the field did not exist, so none of them reached the frontend.
        let ib = raw("infobox challenge", &[("character", "Isaac")]);
        let Infobox::Challenge { character, .. } =
            infobox_from(InfoboxKind::Challenge, &ib, &r, &mut d)
        else {
            panic!()
        };
        assert_eq!(character, Some(Target::Character { id: 0 }));

        let ib = raw(
            "infobox character",
            &[("tears", "2.73"), ("parent", "Tainted Isaac")],
        );
        let Infobox::Character { tears, parent, .. } =
            infobox_from(InfoboxKind::Character, &ib, &r, &mut d)
        else {
            panic!()
        };
        assert_eq!(tears, "2.73");
        assert_eq!(parent, Some(Target::Character { id: 21 }));
    }

    #[test]
    fn challenge_flags() {
        let r = test_resolver();
        let mut d = Diagnostics::default();
        let ib = raw(
            "infobox challenge",
            &[
                ("blindfolded", "yes"),
                ("has shops", "no"),
                ("unlocks", "Epic Fetus"),
            ],
        );
        let Infobox::Challenge {
            blindfolded,
            has_shops,
            has_treasure_rooms,
            unlocks,
            ..
        } = infobox_from(InfoboxKind::Challenge, &ib, &r, &mut d)
        else {
            panic!()
        };
        assert!(blindfolded);
        assert!(!has_shops);
        assert!(!has_treasure_rooms);
        assert_eq!(unlocks, Some(Target::Achievement { id: 62 }));
    }
}
