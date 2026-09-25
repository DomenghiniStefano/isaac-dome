//! `itempools.xml`: which pool each item appears in and with what weight. In the
//! Repentance+ file of 2026-09-04 (`tests/real_data.rs`): 31 pools, 2,058 entries, every
//! `Id` in `items.xml`, and 24 collectibles in no pool at all (that's data, not a defect;
//! trinkets are never in pools by construction).

use crate::diagnostics::{Diagnostic, SkipReason, Source};
use crate::ids::ItemId;
use crate::xml::{self, Element};

const SOURCE: Source = Source::ItemPools;

#[derive(Debug, Clone, PartialEq)]
pub struct PoolEntry {
    pub item: ItemId,
    pub weight: f32,
    pub decrease_by: f32,
    pub remove_on: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pool {
    pub name: String,
    pub entries: Vec<PoolEntry>,
}

/// An item's membership in a pool, as seen from the item.
#[derive(Debug, Clone, PartialEq)]
pub struct PoolMembership {
    pub pool: String,
    pub weight: f32,
}

pub fn parse(bytes: &[u8], diagnostics: &mut Vec<Diagnostic>) -> Vec<Pool> {
    let Some(els) = xml::read(bytes, SOURCE, diagnostics) else {
        return Vec::new();
    };
    els.iter()
        .enumerate()
        .filter(|(_, e)| e.name == "Pool")
        .filter_map(|(i, _)| pool_from(&els, i, diagnostics))
        .collect()
}

/// The pool at `els[i]`. A pool without a name is skipped whole: its entries are not read,
/// so they are not diagnosed either.
fn pool_from(els: &[Element], i: usize, d: &mut Vec<Diagnostic>) -> Option<Pool> {
    let Some(name) = els[i].attr("Name") else {
        return xml::skip(SOURCE, None, SkipReason::MissingName, d);
    };
    let entries = xml::children_named(els, i, "Item")
        .into_iter()
        .filter_map(|item| entry_from(item, d))
        .collect();
    Some(Pool {
        name: name.to_string(),
        entries,
    })
}

fn entry_from(e: &Element, d: &mut Vec<Diagnostic>) -> Option<PoolEntry> {
    let id = xml::required_id(e, "Id", SOURCE, d)?;
    let num =
        |name: &str, default: f32| e.attr(name).and_then(|v| v.parse().ok()).unwrap_or(default);
    Some(PoolEntry {
        item: ItemId(id),
        weight: num("Weight", 1.0),
        decrease_by: num("DecreaseBy", 1.0),
        remove_on: num("RemoveOn", 0.1),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const POOLS: &[u8] = b"<ItemPools>
 <Pool Name=\"treasure\">
  <!--The Sad Onion-->
  <Item Id=\"1\" Weight=\"1\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/>
  <Item Id=\"2\" Weight=\"0.5\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/>
  <Item Weight=\"1\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/>
  <Item Id=\"x\" Weight=\"1\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/>
 </Pool>
 <Pool Name=\"boss\">
  <Item Id=\"1\" Weight=\"2\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/>
 </Pool>
 <Pool><Item Id=\"9\" Weight=\"1\" DecreaseBy=\"1\" RemoveOn=\"0.1\"/></Pool>
</ItemPools>";

    fn parsed() -> (Vec<Pool>, Vec<Diagnostic>) {
        let mut d = Vec::new();
        let p = parse(POOLS, &mut d);
        (p, d)
    }

    #[test]
    fn pools_keep_their_order_and_entries_with_numbers_parsed() {
        let (p, _) = parsed();
        assert_eq!(p.len(), 2, "the pool without a Name is skipped");
        assert_eq!(p[0].name, "treasure");
        assert_eq!(p[0].entries.len(), 2);
        assert_eq!(
            p[0].entries[1],
            PoolEntry {
                item: ItemId(2),
                weight: 0.5,
                decrease_by: 1.0,
                remove_on: 0.1
            }
        );
        assert_eq!(p[1].name, "boss");
    }

    #[test]
    fn malformed_entries_and_nameless_pools_are_diagnosed() {
        let (_, d) = parsed();
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::ItemPools,
            id: None,
            reason: SkipReason::MissingId
        }));
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::ItemPools,
            id: None,
            reason: SkipReason::MalformedId
        }));
        assert!(d.contains(&Diagnostic::ElementSkipped {
            source: Source::ItemPools,
            id: None,
            reason: SkipReason::MissingName
        }));
    }

    #[test]
    fn the_skips_come_in_file_order_and_a_nameless_pool_says_nothing_of_its_items() {
        let (_, d) = parsed();
        let skipped = |reason| Diagnostic::ElementSkipped {
            source: Source::ItemPools,
            id: None,
            reason,
        };
        assert_eq!(
            d,
            vec![
                skipped(SkipReason::MissingId),
                skipped(SkipReason::MalformedId),
                skipped(SkipReason::MissingName),
            ]
        );
    }

    #[test]
    fn an_item_outside_a_pool_belongs_to_none_and_a_nested_one_to_nobody() {
        // Only the direct children of a `<Pool>` are its entries.
        let mut d = Vec::new();
        let p = parse(
            b"<ItemPools><Item Id=\"1\"/><Pool Name=\"p\"><Group><Item Id=\"2\"/></Group><Item Id=\"3\"/></Pool></ItemPools>",
            &mut d,
        );
        assert_eq!(p.len(), 1);
        assert_eq!(
            p[0].entries.iter().map(|e| e.item).collect::<Vec<_>>(),
            vec![ItemId(3)]
        );
        assert!(d.is_empty());
    }

    #[test]
    fn junk_is_empty_with_one_diagnostic() {
        let mut d = Vec::new();
        assert!(parse(b"<ItemPools><Pool", &mut d).is_empty());
        assert_eq!(
            d,
            vec![Diagnostic::SourceUnreadable {
                source: Source::ItemPools
            }]
        );
    }

    #[test]
    fn a_missing_numeric_attribute_defaults_to_the_game_default_not_to_a_skip() {
        // In the real file every Item has all four attributes; if one were missing or
        // non-numeric, the game uses Weight=1, DecreaseBy=1, RemoveOn=0.1. We do the same.
        let mut d = Vec::new();
        let p = parse(
            b"<ItemPools><Pool Name=\"p\"><Item Id=\"3\"/></Pool></ItemPools>",
            &mut d,
        );
        assert_eq!(
            p[0].entries[0],
            PoolEntry {
                item: ItemId(3),
                weight: 1.0,
                decrease_by: 1.0,
                remove_on: 0.1
            }
        );
        assert!(d.is_empty());
    }
}
