//! How each achievement in a list of names is unlocked. On the site `{{achievement text|…}}`
//! expands every name into the name and its unlock condition ("Guardian Angel — Defeat Satan
//! as Magdalene"); the page itself carries only the names, and the condition lives on the
//! achievement's own row, as its `requirements`. So it is joined here, once every page is read.

use std::collections::BTreeMap;

use crate::dataset::Dataset;
use crate::{Block, Infobox, Inline, ListItem, SectionKind, Target};

/// The unlock condition of every achievement that states one, by id.
pub(crate) type Conditions = BTreeMap<u32, Vec<Inline>>;

/// Every list item of every page that names one achievement gets that achievement's
/// condition. Only the top level of a section's lists: a list of names is never nested.
pub(crate) fn add_conditions(ds: &mut Dataset) {
    let conditions = conditions(ds);
    for entry in ds.entries_mut() {
        for section in &mut entry.sections {
            let add = if section.kind == SectionKind::StartingItems {
                add_leading_condition
            } else {
                add_condition
            };
            for block in &mut section.blocks {
                if let Block::List { items, .. } = block {
                    items.iter_mut().for_each(|item| add(item, &conditions));
                }
            }
        }
    }
}

fn conditions(ds: &Dataset) -> Conditions {
    ds.achievements
        .iter()
        .filter_map(|(id, entry)| match &entry.infobox {
            Infobox::Achievement { requirements, .. } if !requirements.is_empty() => {
                Some((*id, requirements.clone()))
            }
            Infobox::Achievement { .. }
            | Infobox::Item { .. }
            | Infobox::Trinket { .. }
            | Infobox::Boss { .. }
            | Infobox::Character { .. }
            | Infobox::Challenge { .. }
            | Infobox::Transformation { .. } => None,
        })
        .collect()
}

/// Puts the achievement's condition under an item that names one achievement and nothing
/// else. Under it, as a paragraph of its own, rather than after it: the item stays a name —
/// which is how the page draws its icon — and the condition, which runs to several editions
/// on some achievements, reads as the line below.
pub(crate) fn add_condition(item: &mut ListItem, conditions: &Conditions) {
    if item.inline.len() == 1 {
        add_leading_condition(item, conditions);
    }
}

/// The same for an item that opens with an achievement and goes on — which is prose anywhere
/// but in "Unlockable Starting Items", where every item is "achievement - what it gives".
pub(crate) fn add_leading_condition(item: &mut ListItem, conditions: &Conditions) {
    if !item.children.is_empty() {
        return;
    }
    let Some(Inline::Ref {
        target: Target::Achievement { id },
        ..
    }) = item.inline.first()
    else {
        return;
    };
    if let Some(condition) = conditions.get(id) {
        item.children.push(Block::Paragraph {
            inline: condition.clone(),
        });
    }
}

// Tests extract one variant and panic on the rest: the wildcard is the assertion.
#[allow(clippy::wildcard_enum_match_arm)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Block, Style, Target};

    fn text(s: &str) -> Inline {
        Inline::Text {
            text: s.into(),
            style: Style::Plain,
        }
    }
    fn achievement(id: u32) -> Inline {
        Inline::Ref {
            target: Target::Achievement { id },
            label: format!("achievement {id}"),
        }
    }
    fn item(inline: Vec<Inline>) -> ListItem {
        ListItem {
            inline,
            children: Vec::new(),
        }
    }
    fn conditions() -> Conditions {
        BTreeMap::from([(7, vec![text("Defeat Satan as Magdalene")])])
    }

    #[test]
    fn a_named_achievement_carries_its_condition_below_it() {
        let mut it = item(vec![achievement(7)]);
        add_condition(&mut it, &conditions());
        assert_eq!(it.inline, vec![achievement(7)], "the name stays a name");
        assert_eq!(
            it.children,
            vec![Block::Paragraph {
                inline: vec![text("Defeat Satan as Magdalene")]
            }]
        );
    }

    #[test]
    fn an_achievement_that_states_no_condition_is_left_as_it_is() {
        let mut it = item(vec![achievement(8)]);
        add_condition(&mut it, &conditions());
        assert!(it.children.is_empty(), "{it:?}");
    }

    /// A sentence that opens with an achievement is prose: the condition would interrupt it.
    #[test]
    fn an_item_that_says_more_than_the_name_is_left_as_it_is() {
        let mut it = item(vec![achievement(7), text(" - Start with The D6")]);
        add_condition(&mut it, &conditions());
        assert!(it.children.is_empty(), "{it:?}");
    }

    /// Except where every item is written that way: "Unlockable Starting Items" is a list of
    /// "achievement - what it gives", and how to get the achievement is the point of the list.
    #[test]
    fn a_starting_item_carries_the_condition_of_the_achievement_it_opens_with() {
        let mut it = item(vec![achievement(7), text(" - Start with The D6")]);
        add_leading_condition(&mut it, &conditions());
        assert_eq!(
            it.children,
            vec![Block::Paragraph {
                inline: vec![text("Defeat Satan as Magdalene")]
            }]
        );
    }

    #[test]
    fn a_name_that_is_not_an_achievement_is_left_as_it_is() {
        let mut it = item(vec![Inline::Ref {
            target: Target::Item { id: 7 },
            label: "item 7".into(),
        }]);
        add_condition(&mut it, &conditions());
        assert!(it.children.is_empty(), "{it:?}");
    }

    /// An item with a list nested under it already has something below the name.
    #[test]
    fn an_item_with_blocks_below_it_is_left_as_it_is() {
        let nested = Block::List {
            ordered: false,
            items: vec![item(vec![text("n")])],
        };
        let mut it = ListItem {
            inline: vec![achievement(7)],
            children: vec![nested.clone()],
        };
        add_condition(&mut it, &conditions());
        assert_eq!(it.children, vec![nested]);
    }
}
