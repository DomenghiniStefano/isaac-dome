//! `itempools.xml`: which pool each item appears in and with what weight. 31 pools in
//! the Repentance file, 2,058 entries; every `Id` exists in `items.xml`, and 24
//! collectibles are in no pool at all (that's data, not a defect; trinkets are never in
//! pools by construction).

use crate::diagnostics::{Diagnostic, SkipReason, Source};
use crate::ids::ItemId;
use crate::strings::children_named;
use crate::xml::{elements, Element};

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
    let els = match elements(bytes) {
        Ok(els) => els,
        Err(_) => {
            diagnostics.push(Diagnostic::SourceUnreadable {
                source: Source::ItemPools,
            });
            return Vec::new();
        }
    };
    let mut pools = Vec::new();
    for (i, e) in els.iter().enumerate() {
        if e.name != "Pool" {
            continue;
        }
        let Some(name) = e.attr("Name") else {
            diagnostics.push(skipped(None, SkipReason::MissingName));
            continue;
        };
        let entries = children_named(&els, i, "Item")
            .into_iter()
            .filter_map(|item| entry_from(item, diagnostics))
            .collect();
        pools.push(Pool {
            name: name.to_string(),
            entries,
        });
    }
    pools
}

fn entry_from(e: &Element, d: &mut Vec<Diagnostic>) -> Option<PoolEntry> {
    let Some(raw_id) = e.attr("Id") else {
        d.push(skipped(None, SkipReason::MissingId));
        return None;
    };
    let Ok(id) = raw_id.parse::<u32>() else {
        d.push(skipped(None, SkipReason::MalformedId));
        return None;
    };
    let num =
        |name: &str, default: f32| e.attr(name).and_then(|v| v.parse().ok()).unwrap_or(default);
    Some(PoolEntry {
        item: ItemId(id),
        weight: num("Weight", 1.0),
        decrease_by: num("DecreaseBy", 1.0),
        remove_on: num("RemoveOn", 0.1),
    })
}

fn skipped(id: Option<u32>, reason: SkipReason) -> Diagnostic {
    Diagnostic::ElementSkipped {
        source: Source::ItemPools,
        id,
        reason,
    }
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
