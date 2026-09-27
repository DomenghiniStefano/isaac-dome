//! The templates whose argument is a comma-separated list of names: `{{achievement text|…}}`,
//! the collectible and trinket tables, and — since card #86 — the pickup, pool, stage-monster,
//! challenge and transformation lists that share the same shape. `resolve` answers with one
//! target, so none of them can go through it — each needs a node per name.
//!
//! Two shapes come out of the same list. On a line of its own, which is how every one of the
//! 203 uses in the snapshot is written, the site expands the template into a bulleted list
//! with an icon per name, and so does `blocks`. Inside a sentence it stays a run of
//! references separated by commas.

use super::{try_comment, with_optional_edition, Out};
use crate::editions::span_restriction;
use crate::resolver::{Resolution, Resolver};
use crate::template::{parse_template_at, Template};
use crate::{Diagnostics, Inline, Style, Target};

/// Which list a template is, and so how its names resolve.
#[derive(Clone, Copy)]
pub(super) enum NameList {
    /// A boss or character page naming the achievements it unlocks.
    Achievements,
    /// How a transformation page states what counts toward it, the only complete
    /// statement of that set — the infobox's `items` misses Guppy's trinket. Also Ultra
    /// Greed's `{{entity table|…}}`, the same comma-list shape naming entities instead: the
    /// kind is the resolver's namespace, `"i"`, `"t"` or `"e"`.
    Collectibles(&'static str),
}

impl NameList {
    pub(super) fn of(template: &str) -> Option<NameList> {
        match template {
            // `achievement table` is the same list the Achievements page's own manual
            // listing would be, drawn with a Cargo query on every page that uses it in the
            // snapshot (no page carries a positional list) — read the same way regardless:
            // a query-only call has no names to give and produces an empty list, which is
            // how `collectible table` below already degrades on the `/Additions` pages.
            "achievement text" | "achievement table" => Some(NameList::Achievements),
            "collectible table" | "collectible rows" => Some(NameList::Collectibles("i")),
            "trinket table" | "trinket rows" => Some(NameList::Collectibles("t")),
            "entity table" => Some(NameList::Collectibles("e")),
            // Card #86: which monsters a floor has (a stage page's own "Monsters" section).
            "stage entities" => Some(NameList::Collectibles("e")),
            // Which items are in a pool (the Item Pool articles' own "Activated
            // Collectibles"/"Passive Collectibles" sections).
            "pool items" => Some(NameList::Collectibles("i")),
            // A pickup's own name list (cards, runes, pills…), the same shape as
            // `collectible table`/`collectible rows` for items.
            "pickup table" | "pickup rows" => Some(NameList::Collectibles("p")),
            // Which challenges a page names, and which transformations grant flight
            // (`Flight`'s own "Transformations that Grant Flight") or make up a set
            // (`Transformations`' own "Transformation Sets", Cargo-driven like
            // `achievement table` above).
            "challenge table" => Some(NameList::Collectibles("chal")),
            "transformation table" => Some(NameList::Collectibles("tf")),
            _ => None, // allowed: template names are the wiki's open vocabulary
        }
    }

    /// A collectible that does not resolve is counted where every other failed lookup is
    /// counted.
    fn target(self, name: &str, r: &Resolver, d: &mut Diagnostics) -> Option<Target> {
        match self {
            NameList::Achievements => r.achievement_by_name(name),
            NameList::Collectibles(kind) => collectible(kind, name, r, d),
        }
    }

    /// A reference where the resolver finds one, the name as text where it does not — not
    /// dropped, because a name we cannot resolve is still what the page says.
    fn node(self, name: &str, r: &Resolver, d: &mut Diagnostics) -> Inline {
        match self.target(name, r, d) {
            Some(target) => Inline::Ref {
                target,
                label: name.to_string(),
            },
            None => Inline::Text {
                text: name.to_string(),
                style: Style::Plain,
            },
        }
    }

    /// Inside a sentence: the references in a row, with the list's own separator.
    pub(super) fn push_inline(
        self,
        t: &Template,
        r: &Resolver,
        d: &mut Diagnostics,
        out: &mut Out,
    ) {
        with_optional_edition(out, d, t.named.get("dlc"), |out, d| {
            for (n, name) in names(t).enumerate() {
                if n > 0 {
                    out.buf.push_str(", ");
                }
                match self.target(name, r, d) {
                    Some(target) => out.push(Inline::Ref {
                        target,
                        label: name.to_string(),
                    }),
                    // Into the buffer, so it takes the style of the words around it.
                    None => out.buf.push_str(name),
                }
            }
        });
    }

    /// On a line of its own: one run per name. `rows | dlc = r` is Conjoined's markup for
    /// items that count only in that edition, so every item carries it — the same claim
    /// `Inline::Edition` makes everywhere else.
    /// An edition that restricts nothing — every edition named, or a code that could not be
    /// read and was counted — wraps nothing, as `Out::close` decides for the inline shape.
    fn items(self, t: &Template, r: &Resolver, d: &mut Diagnostics) -> Vec<Vec<Inline>> {
        let only = t
            .named
            .get("dlc")
            .map(|code| span_restriction(code, d))
            .filter(|only| !only.is_empty());
        names(t)
            .map(|name| {
                let node = self.node(name, r, d);
                match &only {
                    Some(only) => vec![Inline::Edition {
                        only: only.clone(),
                        inline: vec![node],
                    }],
                    None => vec![node],
                }
            })
            .collect()
    }
}

/// The items of a line that is nothing but a name-list template — trailing whitespace and a
/// comment allowed, which is how the one page that annotates its list writes it. `None` for
/// any other line, which is then prose like any other.
pub fn name_list_items(line: &str, r: &Resolver, d: &mut Diagnostics) -> Option<Vec<Vec<Inline>>> {
    let (t, end) = parse_template_at(line, 0)?;
    let list = NameList::of(&t.name)?;
    let rest = line.get(end..)?.trim();
    if !rest.is_empty() && try_comment(rest) != Some(rest.len()) {
        return None;
    }
    Some(list.items(&t, r, d))
}

/// The names in the first positional argument. Empty names are skipped.
fn names(t: &Template) -> impl Iterator<Item = &str> {
    t.args
        .first()
        .map_or("", String::as_str)
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
}

/// A reference under `resolve`'s namespace (`"i"`, `"t"`, `"e"`…), counted where every other
/// failed lookup is when the name doesn't resolve. `blocks` reaches this too, for the entity
/// a bare `{{entity row minimal|…}}` row names — the same fallback a `{{e|…}}` reference has,
/// not a second one written to agree with it.
pub(crate) fn collectible(
    kind: &str,
    name: &str,
    r: &Resolver,
    d: &mut Diagnostics,
) -> Option<Target> {
    match r.resolve(kind, name) {
        Resolution::Target(target) => Some(target),
        Resolution::Concept | Resolution::Unresolved | Resolution::Ignore | Resolution::Unknown => {
            d.unresolved(kind);
            None
        }
    }
}
