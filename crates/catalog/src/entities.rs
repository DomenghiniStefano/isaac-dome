//! `entities2.xml`: every entity the game defines — monsters, pickups, effects, the player
//! itself — with the `.anm2` that draws it. 1337 rows on the Repentance+ file of 2026-09-26
//! (`tests/real_data.rs`), keyed the way the game and the wiki both do: `(id, variant,
//! subtype)`, the same triple `wiki::Target::Entity` carries.
//!
//! **This module stops at which file to read next, not at a picture.** An anm2 is one file
//! per row, and 1337 of them is not one scene shared by every row the way `bossportraits.xml`'s
//! is: reading every row's frame at catalog build time means reading and decompressing 1337
//! small archive entries on every launch, for pictures most sessions never open. So a row
//! carries its `anm2_path` and nothing cropped from it yet — `anm2::frames` (already public) is
//! there for whoever reads that one file, on the one request that actually wants it.

use crate::diagnostics::{Diagnostic, SkipReason, Source};
use crate::text::Text;
use crate::xml::{self, Element};

const SOURCE: Source = Source::Entities;

/// One row of `entities2.xml`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entity {
    pub id: u32,
    pub variant: u32,
    /// `0` when the row declares none: most rows draw only one form, and the file leaves the
    /// attribute out rather than writing `subtype="0"` on every one of them.
    pub subtype: u32,
    pub name: Text,
    /// Logical path of the `.anm2` that draws this row: the file's own `anm2root` plus its
    /// `anm2path`. Not a `SpriteRef` — an anm2 is a document, not a picture.
    pub anm2_path: String,
}

pub fn parse(bytes: &[u8], diagnostics: &mut Vec<Diagnostic>) -> Vec<Entity> {
    let Some(els) = xml::read(bytes, SOURCE, diagnostics) else {
        return Vec::new();
    };
    let anm2root = xml::root_attr(&els, "entities", "anm2root", "gfx");
    els.iter()
        .filter(|e| e.name == "entity")
        .filter_map(|e| entity_from(e, &anm2root, diagnostics))
        .collect()
}

fn entity_from(e: &Element, anm2root: &str, d: &mut Vec<Diagnostic>) -> Option<Entity> {
    let id = xml::required_id(e, "id", SOURCE, d)?;
    let path = xml::required_attr(e, "anm2path", id, SkipReason::MissingSprite, SOURCE, d)?;
    // `anm2path=""` is a real row (id 9001, "Spidermod Text" — measured on the installed game
    // on 2026-09-26): the attribute is declared and empty, not absent. An empty path resolves
    // to no file at all, so it's the same skip as a missing one.
    if path.is_empty() {
        return xml::skip(SOURCE, Some(id), SkipReason::MissingSprite, d);
    }
    let name = xml::required_attr(e, "name", id, SkipReason::MissingName, SOURCE, d)?;
    Some(Entity {
        id,
        variant: attr_u32(e, "variant"),
        subtype: attr_u32(e, "subtype"),
        name: Text::from_attr(name),
        anm2_path: format!("{anm2root}/{path}"),
    })
}

/// A numeric attribute that defaults to `0` when absent or malformed, rather than skipping the
/// row: `entities2.xml` leaves `variant` and `subtype` out for the base form, and a missing
/// `subtype` is the game's own "default form", never a reason to drop the row.
fn attr_u32(e: &Element, attr: &str) -> u32 {
    e.attr(attr).and_then(|v| v.parse().ok()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENTITIES: &[u8] = b"<entities anm2root=\"gfx/\" version=\"5\">
\t<entity id=\"10\" variant=\"0\" name=\"#GAPER\" anm2path=\"010.000_Gaper.anm2\" />
\t<entity id=\"23\" variant=\"0\" subtype=\"1\" name=\"My Shadow\" anm2path=\"023.000.001_My Shadow.anm2\" />
\t<entity id=\"5\" variant=\"10\" name=\"Heart\" anm2path=\"005.010_Heart.anm2\" />
\t<entity variant=\"0\" name=\"NoId\" anm2path=\"x.anm2\" />
\t<entity id=\"abc\" variant=\"0\" name=\"BadId\" anm2path=\"x.anm2\" />
\t<entity id=\"7\" variant=\"0\" name=\"NoAnm2\" />
\t<entity id=\"8\" variant=\"0\" anm2path=\"x.anm2\" />
</entities>";

    fn parsed() -> (Vec<Entity>, Vec<Diagnostic>) {
        let mut d = Vec::new();
        let rows = parse(ENTITIES, &mut d);
        (rows, d)
    }

    #[test]
    fn a_row_carries_its_key_name_and_anm2_path_under_the_declared_root() {
        let (rows, _) = parsed();
        let gaper = rows.iter().find(|r| r.id == 10).unwrap();
        assert_eq!(gaper.variant, 0);
        assert_eq!(gaper.subtype, 0, "no subtype attribute: the default form");
        assert_eq!(gaper.anm2_path, "gfx/010.000_Gaper.anm2");
        assert_eq!(gaper.name, Text::from_attr("#GAPER"));
    }

    #[test]
    fn a_declared_subtype_is_kept_and_a_literal_name_stays_literal() {
        let (rows, _) = parsed();
        let shadow = rows.iter().find(|r| r.id == 23).unwrap();
        assert_eq!(shadow.subtype, 1);
        assert_eq!(
            shadow.name,
            Text::Literal {
                text: "My Shadow".to_string()
            }
        );
    }

    #[test]
    fn a_pickup_keeps_its_variant_the_same_way_a_monster_does() {
        let (rows, _) = parsed();
        let heart = rows.iter().find(|r| r.id == 5).unwrap();
        assert_eq!(heart.variant, 10);
        assert_eq!(heart.subtype, 0);
    }

    #[test]
    fn a_declared_but_empty_anm2path_is_the_same_skip_as_a_missing_one() {
        // Real row on the installed game, id 9001 "Spidermod Text": the attribute is there
        // and empty, not absent.
        let mut d = Vec::new();
        let rows = parse(
            b"<entities><entity id=\"9001\" variant=\"1\" name=\"Spidermod Text\" anm2path=\"\" /></entities>",
            &mut d,
        );
        assert!(rows.is_empty());
        assert_eq!(
            d,
            vec![Diagnostic::ElementSkipped {
                source: Source::Entities,
                id: Some(9001),
                reason: SkipReason::MissingSprite
            }]
        );
    }

    #[test]
    fn without_a_root_the_default_is_gfx() {
        let mut d = Vec::new();
        let rows = parse(
            b"<entities><entity id=\"1\" variant=\"0\" name=\"A\" anm2path=\"a.anm2\" /></entities>",
            &mut d,
        );
        assert_eq!(rows[0].anm2_path, "gfx/a.anm2");
    }

    #[test]
    fn malformed_rows_are_skipped_with_a_reason_and_the_rest_is_read() {
        let (rows, d) = parsed();
        assert_eq!(rows.len(), 3);
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::Entities,
            id: None,
            reason: SkipReason::MissingId
        }));
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::Entities,
            id: None,
            reason: SkipReason::MalformedId
        }));
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::Entities,
            id: Some(7),
            reason: SkipReason::MissingSprite
        }));
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::Entities,
            id: Some(8),
            reason: SkipReason::MissingName
        }));
    }

    #[test]
    fn the_skips_come_in_file_order_with_the_anm2_checked_before_the_name() {
        let mut d = Vec::new();
        let rows = parse(
            b"<entities>
<entity id=\"x\" variant=\"0\" name=\"A\" anm2path=\"a.anm2\" />
<entity variant=\"0\" name=\"B\" anm2path=\"b.anm2\" />
<entity id=\"3\" variant=\"0\" />
<entity id=\"4\" variant=\"0\" anm2path=\"d.anm2\" />
</entities>",
            &mut d,
        );
        assert!(rows.is_empty());
        let skipped = |id, reason| Diagnostic::ElementSkipped {
            source: Source::Entities,
            id,
            reason,
        };
        assert_eq!(
            d,
            vec![
                skipped(None, SkipReason::MalformedId),
                skipped(None, SkipReason::MissingId),
                skipped(Some(3), SkipReason::MissingSprite),
                skipped(Some(4), SkipReason::MissingName),
            ]
        );
    }

    #[test]
    fn junk_is_empty_with_one_diagnostic() {
        let mut d = Vec::new();
        assert!(parse(b"<entities><entity", &mut d).is_empty());
        assert_eq!(
            d,
            vec![Diagnostic::SourceUnreadable {
                source: Source::Entities
            }]
        );
    }
}
