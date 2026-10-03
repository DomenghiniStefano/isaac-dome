//! A completion-matrix column's page, against the installed game and the embedded dataset: the
//! page a mark and a tally link to in the blocked menu (B36). The rule is read from the catalog
//! — each column's `bossportraits.xml` row, keyed the way the boss's own link is keyed — so a
//! patch that renames a row or adds a page moves the answer with it.

mod support;

use core_save::Column;
use ipc::{boss_name, Target};
use support::real_catalog;

#[test]
fn every_column_links_to_the_page_of_what_it_is() {
    let Some(c) = real_catalog() else { return };
    let Ok(ds) = wiki::Dataset::embedded() else {
        test_support::skip("wiki dataset not embedded");
        return;
    };
    let mut entities = 0usize;
    for column in Column::ALL {
        let page = ipc::for_tests::column_page(&c, column);
        let name = boss_name(column);
        match column {
            // Not entities, by decision: a room-and-event and a game mode, linked to the
            // articles the wiki writes about them.
            Column::BossRush | Column::Greed => assert!(
                matches!(page, Some(Target::Article { .. })),
                "{name}: {page:?}"
            ),
            Column::MomsHeart
            | Column::Isaac
            | Column::Satan
            | Column::BlueBaby
            | Column::TheLamb
            | Column::MegaSatan
            | Column::Hush
            | Column::Delirium
            | Column::Mother
            | Column::TheBeast => {
                let Some(Target::Entity { id, variant, .. }) = page else {
                    panic!("{name}: no boss page ({page:?})");
                };
                // The page reached is the boss's: its title is the column's name, give or take
                // the wiki's disambiguation — *Isaac (Boss)*, *??? (Boss)* for Blue Baby.
                let title = &ds
                    .bosses
                    .get(&wiki::Dataset::boss_key(id, variant, 0))
                    .expect("page_of kept it, so the dataset has it")
                    .title;
                eprintln!("sample: {name} -> {id}.{variant} {title}");
                entities += 1;
            }
        }
    }
    assert_eq!(entities, 10, "ten columns are a boss");
}
