//! Infoboxes: extracted as top-level templates, converted into the `Infobox` of their kind.

use std::collections::BTreeMap;

use crate::inline::parse_inline;
use crate::resolver::Resolver;
use crate::template::parse_template_at;
use crate::{Diagnostics, Infobox, Inline};

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
    Collectible,
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
            "infobox passive collectible"
            | "infobox activated collectible"
            | "infobox collectible" => InfoboxKind::Collectible,
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

/// The leading digits: `"250 (x2)"` → 250, `"6666"` → 6666, no digits → `None`.
pub(crate) fn leading_number(s: &str) -> Option<u32> {
    s.trim()
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .ok()
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
        InfoboxKind::Collectible => Infobox::Item,
        InfoboxKind::Trinket => Infobox::Trinket,
        InfoboxKind::Achievement => Infobox::Achievement {
            description: text(ib, "description"),
            requirements: inline(ib, "requirements", r, d),
            unlocks: r.by_page_title(param(ib, "link")),
        },
        InfoboxKind::Boss => Infobox::Boss {
            base_hp: leading_number(param(ib, "base hp")),
            environment: inline(ib, "environment", r, d),
            pool: inline(ib, "pool", r, d),
            unlocked_by: r.achievement_by_name(param(ib, "unlocked by")),
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
            // `unlocks` is usually a page title; for achievements it's the name.
            unlocks: r
                .by_page_title(param(ib, "unlocks"))
                .or_else(|| r.achievement_by_name(param(ib, "unlocks"))),
            unlocked_by: r.achievement_by_name(param(ib, "unlocked by")),
        },
        InfoboxKind::Character => Infobox::Character {
            health: inline(ib, "health", r, d),
            damage: text(ib, "damage"),
            range: text(ib, "range"),
            speed: text(ib, "speed"),
            luck: text(ib, "luck"),
            shot_speed: text(ib, "shot speed"),
            pickups: inline(ib, "pickups", r, d),
            collectibles: inline(ib, "collectibles", r, d),
            unlocked_by: r.achievement_by_name(param(ib, "unlocked by")),
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
    fn kinds() {
        assert_eq!(
            InfoboxKind::of("infobox passive collectible"),
            Some(InfoboxKind::Collectible)
        );
        assert_eq!(
            InfoboxKind::of("infobox activated collectible"),
            Some(InfoboxKind::Collectible)
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
            description,
            requirements,
            unlocks,
        } = infobox_from(InfoboxKind::Achievement, &ib, &r, &mut d)
        else {
            panic!()
        };
        assert_eq!(description, "Unlocked a new item.");
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
        let Infobox::Boss {
            base_hp,
            unlocked_by,
            ..
        } = infobox_from(InfoboxKind::Boss, &ib, &r, &mut d)
        else {
            panic!()
        };
        assert_eq!(base_hp, Some(250));
        assert_eq!(unlocked_by, Some(Target::Achievement { id: 62 }));
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
