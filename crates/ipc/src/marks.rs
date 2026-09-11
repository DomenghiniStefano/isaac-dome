use serde::Serialize;

/// The twelve columns the game's own completion widget draws. Mother and The Beast were
/// located on 2026-09-08, on the historical series: for the 14 original characters they
/// are as verified as the other ten, for The Forgotten and the 19 later characters they
/// are still unlocated — see `FORGOTTEN` and `BLOCKS_19`.
pub const BOSSES: [&str; 12] = [
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
    "Mother",
    "The Beast",
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
///
/// The last two came out of the historical series on 2026-09-08. Each was pinned three
/// ways at once, on the days the cell changed: the boss (an achievement whose wiki
/// requirement is *Mother* or *The Beast* unlocked the same day), the count (index 491
/// and 492 are that boss's kills, and they rose by exactly as many as the new marks), and
/// the character (index 188 is a bitmask of the characters that won, and it read
/// Magdalene for base+1 and Cain for base+2).
const BLOCKS_14: [usize; 12] = [27, 41, 55, 69, 83, 97, 116, 130, 144, 173, 423, 457];

/// Single cells for The Forgotten, per boss. 212 belongs to another family.
///
/// Mother and The Beast are `None` on purpose: the spacing between the two 14-blocks is
/// exactly 34 = 14 + 1 + 19, so their cells are certainly inside 423..=490, but which
/// cell is The Forgotten's cannot be told from any save we have — those 40 cells are
/// zero in every one of them. A guess here would show a mark nobody earned.
const FORGOTTEN: [Option<usize>; 12] = [
    Some(203),
    Some(204),
    Some(205),
    Some(206),
    Some(207),
    Some(208),
    Some(209),
    Some(210),
    Some(211),
    Some(213),
    None,
    None,
];

/// Base of the 19-cell block, per boss. DERIVED from the regular pattern for the first
/// nine, and each one corroborated by cells that were seen moving. `None` = not located.
///
/// Delirium closed on 2026-09-08: the base is 404, not the 386 the pattern predicted, and
/// four characters agree on it — Bethany (+0), Jacob & Esau (+1), T. Cain (+4) and
/// T. Azazel (+9), each on a day the Delirium kill counter also rose. What sits in
/// 386..=403 is still unread.
const BLOCKS_19: [Option<usize>; 12] = [
    Some(214),
    Some(233),
    Some(252),
    Some(271),
    Some(290),
    Some(309),
    Some(328),
    Some(347),
    Some(366),
    Some(404),
    None,
    None,
];

const FIRST_LATER: usize = 15;

/// Index into the counters section for the (character, boss) cell.
/// `None` when the cell isn't located in the tables.
pub fn counter_index(character: usize, boss: usize) -> Option<usize> {
    let (_, group) = *CHARACTERS.get(character)?;
    match group {
        CharacterGroup::Original => Some(*BLOCKS_14.get(boss)? + character),
        CharacterGroup::Forgotten => *FORGOTTEN.get(boss)?,
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
    /// The Tainted form. The screen groups rows the way a player does, base and Tainted,
    /// while `group` stays the file's three blocks — which is where the unread cells live.
    pub tainted: bool,
    pub cells: Vec<Cell>,
    /// The co-op menu head. `None` without a catalog, or for a character the menu doesn't
    /// draw.
    pub head_url: Option<String>,
}

/// The symbol URLs of one column, one per tier. Both `None` when the game's archives
/// aren't open: the screen draws the fallback outfit instead.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkArtView {
    pub normal_url: Option<String>,
    pub hard_url: Option<String>,
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
    /// `art[i]` draws `bosses[i]`: a parallel array, so `bosses` keeps the shape the design
    /// was built on.
    pub art: Vec<MarkArtView>,
    pub totals: MarksTotals,
}

/// Builds the matrix from the counters read out of the file. It assumes no fixed
/// length: an index past the section read produces `Unknown`.
///
/// With a catalog — the game's archives are open — rows and columns carry the URLs `icon`
/// builds for their head and their symbols; without one, none, because a URL nothing can
/// serve draws a broken image where the fallback belongs.
pub fn marks_matrix(
    counters: &[u32],
    catalog: Option<&catalog::Catalog>,
    mut icon: impl FnMut(&crate::icon::IconRef) -> Option<String>,
) -> MarksMatrix {
    use crate::icon::{IconRef, MarkTier};

    let rows: Vec<CharacterRow> = CHARACTERS
        .iter()
        .enumerate()
        .map(|(c, &(name, group))| CharacterRow {
            character: name.to_string(),
            group,
            tainted: CHARACTER_KEYS.get(c).is_some_and(|&(_, tainted)| tainted),
            cells: (0..BOSSES.len()).map(|b| cell_at(counters, c, b)).collect(),
            head_url: catalog
                .and_then(|cat| character_for(c, cat))
                .and_then(|ch| ch.head.as_ref())
                .and_then(|_| icon(&IconRef::Head { row: c })),
        })
        .collect();
    let art = (0..BOSSES.len())
        .map(|column| match catalog {
            Some(_) => MarkArtView {
                normal_url: icon(&IconRef::Mark {
                    column,
                    tier: MarkTier::Normal,
                }),
                hard_url: icon(&IconRef::Mark {
                    column,
                    tier: MarkTier::Hard,
                }),
            },
            None => MarkArtView {
                normal_url: None,
                hard_url: None,
            },
        })
        .collect();

    let totals = totals_of(&rows);
    MarksMatrix {
        characters: rows,
        bosses: BOSSES.iter().map(|b| b.to_string()).collect(),
        art,
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
        // Bit 0 or bit 1: the unconfirmed bit alone draws nothing in the grid (the
        // frontend's `markVisual`), and a total that counts what its grid doesn't show lies.
        started: count(|c| matches!(c, Cell::Known { bits } if *bits & 3 != 0)),
    }
}
