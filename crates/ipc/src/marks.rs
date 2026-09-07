use serde::Serialize;

/// The ten verified bosses. Mother and The Beast don't yet have a column
/// for any character, so they don't appear here.
pub const BOSSES: [&str; 10] = [
    "Mom's Heart",
    "Isaac",
    "Satan",
    "Boss Rush",
    "Blue Baby",
    "The Lamb",
    "Mega Satan",
    "Greed",
    "Hush",
    "Delirium",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CharacterGroup {
    /// The 14 originals: 14-cell blocks, verified.
    Original,
    /// The Forgotten, added later: single cells.
    Forgotten,
    /// Bethany, Jacob & Esau, and the 17 Tainted: 19-cell blocks, derived.
    Later,
}

pub const CHARACTERS: [(&str, CharacterGroup); 34] = [
    ("Isaac", CharacterGroup::Original),
    ("Magdalene", CharacterGroup::Original),
    ("Cain", CharacterGroup::Original),
    ("Judas", CharacterGroup::Original),
    ("Blue Baby", CharacterGroup::Original),
    ("Eve", CharacterGroup::Original),
    ("Samson", CharacterGroup::Original),
    ("Azazel", CharacterGroup::Original),
    ("Lazarus", CharacterGroup::Original),
    ("Eden", CharacterGroup::Original),
    ("The Lost", CharacterGroup::Original),
    ("Lilith", CharacterGroup::Original),
    ("Keeper", CharacterGroup::Original),
    ("Apollyon", CharacterGroup::Original),
    ("The Forgotten", CharacterGroup::Forgotten),
    ("Bethany", CharacterGroup::Later),
    ("Jacob & Esau", CharacterGroup::Later),
    ("T. Isaac", CharacterGroup::Later),
    ("T. Magdalene", CharacterGroup::Later),
    ("T. Cain", CharacterGroup::Later),
    ("T. Judas", CharacterGroup::Later),
    ("T. Blue Baby", CharacterGroup::Later),
    ("T. Eve", CharacterGroup::Later),
    ("T. Samson", CharacterGroup::Later),
    ("T. Azazel", CharacterGroup::Later),
    ("T. Lazarus", CharacterGroup::Later),
    ("T. Eden", CharacterGroup::Later),
    ("T. The Lost", CharacterGroup::Later),
    ("T. Lilith", CharacterGroup::Later),
    ("T. Keeper", CharacterGroup::Later),
    ("T. Apollyon", CharacterGroup::Later),
    ("T. Forgotten", CharacterGroup::Later),
    ("T. Bethany", CharacterGroup::Later),
    ("T. Jacob & Esau", CharacterGroup::Later),
];

/// For each row of `CHARACTERS`, the name key in `players.xml` (without `#` and without
/// `_NAME`) and whether the row is the Tainted form. Keys repeat between the normal and
/// Tainted forms: it's the pair that identifies the character. Verified against the
/// file on 2026-09-03.
pub const CHARACTER_KEYS: [(&str, bool); 34] = [
    ("ISAAC", false),
    ("MAGDALENE", false),
    ("CAIN", false),
    ("JUDAS", false),
    ("BLUEBABY", false),
    ("EVE", false),
    ("SAMSON", false),
    ("AZAZEL", false),
    ("LAZARUS", false),
    ("EDEN", false),
    ("THE_LOST", false),
    ("LILITH", false),
    ("KEEPER", false),
    ("APOLLYON", false),
    ("THE_FORGOTTEN", false),
    ("BETHANY", false),
    ("JACOB", false),
    ("ISAAC", true),
    ("MAGDALENE", true),
    ("CAIN", true),
    ("JUDAS", true),
    ("BLUEBABY", true),
    ("EVE", true),
    ("SAMSON", true),
    ("AZAZEL", true),
    ("LAZARUS", true),
    ("EDEN", true),
    ("THE_LOST", true),
    ("LILITH", true),
    ("KEEPER", true),
    ("APOLLYON", true),
    ("THE_FORGOTTEN", true),
    ("BETHANY", true),
    ("JACOB", true),
];

/// The catalog character for row `row` of the matrix: the **first** one with that key
/// and that Tainted flag, in id order. The hidden forms (Lazarus 2, Black Judas, The
/// Soul) have their own keys and don't interfere; Esau has his own key, and the "Jacob &
/// Esau" row takes Jacob.
pub fn character_for(row: usize, catalog: &catalog::Catalog) -> Option<&catalog::Character> {
    let (key, tainted) = *CHARACTER_KEYS.get(row)?;
    let wanted = format!("{key}_NAME");
    catalog.characters().find(|c| {
        c.tainted == tainted && matches!(&c.name, catalog::Text::Key { key: k } if *k == wanted)
    })
}

/// Base of the 14-cell block, per boss. Verified (REPENTOGON + real saves).
const BLOCKS_14: [usize; 10] = [27, 41, 55, 69, 83, 97, 116, 130, 144, 173];

/// Single cells for The Forgotten, per boss. 212 belongs to another family.
const FORGOTTEN: [usize; 10] = [203, 204, 205, 206, 207, 208, 209, 210, 211, 213];

/// Base of the 19-cell block, per boss. DERIVED, not documented, and it stops
/// at Hush: from Delirium onward the regularity breaks down. `None` = not located.
const BLOCKS_19: [Option<usize>; 10] = [
    Some(214),
    Some(233),
    Some(252),
    Some(271),
    Some(290),
    Some(309),
    Some(328),
    Some(347),
    Some(366),
    None,
];

const FIRST_LATER: usize = 15;

/// Index into the counters section for the (character, boss) cell.
/// `None` when the cell isn't located in the tables.
pub fn counter_index(character: usize, boss: usize) -> Option<usize> {
    let (_, group) = *CHARACTERS.get(character)?;
    match group {
        CharacterGroup::Original => Some(*BLOCKS_14.get(boss)? + character),
        CharacterGroup::Forgotten => Some(*FORGOTTEN.get(boss)?),
        CharacterGroup::Later => (*BLOCKS_19.get(boss)?).map(|base| base + character - FIRST_LATER),
    }
}

/// A cell of the matrix. The three variants are the module's reason for existing:
/// "never done", "not readable", and "suspicious value" are three different things.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Cell {
    /// Valid mask: bits 0 and 1 = mark levels, bit 2 = unexplained third level.
    Known { bits: u8 },
    /// Index not located in the tables, or past the end of the section read.
    Unknown,
    /// Outside 0..=7: not a mask, so the index points somewhere else.
    Unexpected { value: u32 },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CharacterRow {
    pub character: String,
    pub group: CharacterGroup,
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarksTotals {
    pub cells: usize,
    pub readable: usize,
    pub unknown: usize,
    pub unexpected: usize,
    pub started: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarksMatrix {
    pub characters: Vec<CharacterRow>,
    pub bosses: Vec<String>,
    pub totals: MarksTotals,
}

/// Builds the matrix from the counters read out of the file. It assumes no fixed
/// length: an index past the section read produces `Unknown`.
pub fn marks_matrix(counters: &[u32]) -> MarksMatrix {
    let rows: Vec<CharacterRow> = CHARACTERS
        .iter()
        .enumerate()
        .map(|(c, &(name, group))| CharacterRow {
            character: name.to_string(),
            group,
            cells: (0..BOSSES.len()).map(|b| cell_at(counters, c, b)).collect(),
        })
        .collect();

    let totals = totals_of(&rows);
    MarksMatrix {
        characters: rows,
        bosses: BOSSES.iter().map(|b| b.to_string()).collect(),
        totals,
    }
}

fn cell_at(counters: &[u32], character: usize, boss: usize) -> Cell {
    match counter_index(character, boss).and_then(|i| counters.get(i)) {
        None => Cell::Unknown,
        Some(&value) if value <= 7 => Cell::Known { bits: value as u8 },
        Some(&value) => Cell::Unexpected { value },
    }
}

fn totals_of(rows: &[CharacterRow]) -> MarksTotals {
    let cells = || rows.iter().flat_map(|r| r.cells.iter());
    // `cells()` returns a fresh iterator on every call: counting one category
    // consumes the iterator, so a single one can't be reused.
    let count = |f: fn(&Cell) -> bool| cells().filter(|&c| f(c)).count();
    MarksTotals {
        cells: cells().count(),
        readable: count(|c| matches!(c, Cell::Known { .. })),
        unknown: count(|c| matches!(c, Cell::Unknown)),
        unexpected: count(|c| matches!(c, Cell::Unexpected { .. })),
        started: count(|c| matches!(c, Cell::Known { bits } if *bits != 0)),
    }
}
