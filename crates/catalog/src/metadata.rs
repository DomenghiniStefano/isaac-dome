//! `items_metadata.xml`: quality and tags. They're not in `items.xml` (0 occurrences in
//! the Repentance file): they live here, one row per item, `<item id>` for
//! passives/actives/familiars and `<trinket id>` for trinkets. The file doesn't
//! distinguish passive, active and familiar: the key for `<item>` entries is
//! `ItemKind::Passive`, and anyone looking up an active or a familiar must search with
//! that key.

use std::collections::BTreeMap;

use crate::diagnostics::{Diagnostic, SkipReason, Source};
use crate::ids::ItemId;
use crate::items::ItemKind;
use crate::xml::{elements, Element};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Metadata {
    /// -1..=4 in the real file; `None` if the attribute is missing.
    pub quality: Option<i8>,
    pub tags: Vec<String>,
}

pub fn parse(
    bytes: &[u8],
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeMap<(ItemKind, ItemId), Metadata> {
    let els = match elements(bytes) {
        Ok(els) => els,
        Err(_) => {
            diagnostics.push(Diagnostic::SourceUnreadable {
                source: Source::Metadata,
            });
            return BTreeMap::new();
        }
    };
    let mut out = BTreeMap::new();
    for e in &els {
        let kind = match e.name.as_str() {
            "item" => ItemKind::Passive,
            "trinket" => ItemKind::Trinket,
            _ => continue, // allowed: a tag name is an open string
        };
        let Some(raw_id) = e.attr("id") else {
            diagnostics.push(skipped(None, SkipReason::MissingId));
            continue;
        };
        let Ok(id) = raw_id.parse::<u32>() else {
            diagnostics.push(skipped(None, SkipReason::MalformedId));
            continue;
        };
        out.insert((kind, ItemId(id)), metadata_of(e));
    }
    out
}

fn metadata_of(e: &Element) -> Metadata {
    Metadata {
        quality: e.attr("quality").and_then(|q| q.parse().ok()),
        tags: e
            .attr("tags")
            .map(|t| t.split_whitespace().map(str::to_string).collect())
            .unwrap_or_default(),
    }
}

fn skipped(id: Option<u32>, reason: SkipReason) -> Diagnostic {
    Diagnostic::ElementSkipped {
        source: Source::Metadata,
        id,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const META: &[u8] = b"<items>
    <item id=\"1\" quality=\"3\" tags=\"summonable tearsup offensive\"/>
\t<item\tid=\"2\"\tquality=\"-1\"\ttags=\"\"/>
    <trinket id=\"1\" quality=\"0\" tags=\"offensive\"/>
    <item id=\"3\" tags=\"x\"/>
    <item quality=\"2\"/>
    <item id=\"zz\" quality=\"2\"/>
</items>";

    fn parsed() -> (BTreeMap<(ItemKind, ItemId), Metadata>, Vec<Diagnostic>) {
        let mut d = Vec::new();
        let m = parse(META, &mut d);
        (m, d)
    }

    #[test]
    fn items_and_trinkets_are_keyed_by_kind_and_id_so_they_do_not_collide() {
        let (m, _) = parsed();
        let item1 = &m[&(ItemKind::Passive, ItemId(1))];
        let trinket1 = &m[&(ItemKind::Trinket, ItemId(1))];
        assert_eq!(item1.quality, Some(3));
        assert_eq!(item1.tags, vec!["summonable", "tearsup", "offensive"]);
        assert_eq!(trinket1.quality, Some(0));
        assert_eq!(trinket1.tags, vec!["offensive"]);
    }

    #[test]
    fn negative_quality_and_empty_tags_are_read_as_such() {
        let (m, _) = parsed();
        let item2 = &m[&(ItemKind::Passive, ItemId(2))];
        assert_eq!(item2.quality, Some(-1));
        assert!(item2.tags.is_empty());
    }

    #[test]
    fn a_row_without_quality_keeps_none_and_malformed_rows_are_skipped() {
        let (m, d) = parsed();
        assert_eq!(m[&(ItemKind::Passive, ItemId(3))].quality, None);
        assert_eq!(m.len(), 4);
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::Metadata,
            id: None,
            reason: SkipReason::MissingId
        }));
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::Metadata,
            id: None,
            reason: SkipReason::MalformedId
        }));
    }

    #[test]
    fn the_skips_come_in_file_order_and_other_elements_are_not_rows() {
        let (_, d) = parsed();
        let skipped = |reason| Diagnostic::ElementSkipped {
            source: Source::Metadata,
            id: None,
            reason,
        };
        assert_eq!(
            d,
            vec![
                skipped(SkipReason::MissingId),
                skipped(SkipReason::MalformedId)
            ],
            "the root `<items>` is not a row and is not diagnosed"
        );
    }

    #[test]
    fn a_repeated_row_is_read_as_the_last_one() {
        let mut d = Vec::new();
        let m = parse(
            b"<items><item id=\"1\" quality=\"1\"/><item id=\"1\" quality=\"4\"/></items>",
            &mut d,
        );
        assert_eq!(m.len(), 1);
        assert_eq!(m[&(ItemKind::Passive, ItemId(1))].quality, Some(4));
    }

    #[test]
    fn junk_is_empty_with_one_diagnostic() {
        let mut d = Vec::new();
        assert!(parse(b"<items><item", &mut d).is_empty());
        assert_eq!(
            d,
            vec![Diagnostic::SourceUnreadable {
                source: Source::Metadata
            }]
        );
    }
}
