use core_save::{group_of, Column, ROWS};
use serde::Serialize;

/// The English name the matrix header draws for a column, and the one a tally is labelled
/// with. Exhaustive, so a thirteenth column cannot be drawn without a name.
pub const fn boss_name(column: Column) -> &'static str {
    match column {
        Column::MomsHeart => "Mom's Heart",
        Column::Isaac => "Isaac",
        Column::Satan => "Satan",
        Column::BossRush => "Boss Rush",
        Column::BlueBaby => "Blue Baby",
        Column::TheLamb => "The Lamb",
        Column::MegaSatan => "Mega Satan",
        Column::Greed => "Greed",
        Column::Hush => "Hush",
        Column::Delirium => "Delirium",
        Column::Mother => "Mother",
        Column::TheBeast => "The Beast",
    }
}

/// The twelve columns the game's own completion widget draws, by name, in the order of
/// [`Column::ALL`] — which is the one list of them (card #82, S1). Mother and The Beast were
/// located on 2026-09-08, on the historical series: for the 14 original characters they
/// are as verified as the other ten. For The Forgotten and the 19 later characters Mother
/// was located on 2026-09-20 and The Beast is still unlocated — see `FORGOTTEN` and
/// `BLOCKS_19` in `core_save::marks`.
pub const BOSSES: [&str; Column::ALL.len()] = {
    // A `const` cannot call `array::map` or an iterator: an index walk is the only way to
    // derive the array at compile time, where its length is a type.
    let mut names = [""; Column::ALL.len()];
    let mut i = 0;
    while i < names.len() {
        names[i] = boss_name(Column::ALL[i]);
        i += 1;
    }
    names
};

/// Which block family a row belongs to. Defined with the layout, because that is what it
/// describes; re-exported here because it crosses the IPC as part of a `CharacterRow`.
pub use core_save::CharacterGroup;

/// One row of the completion matrix: who the layout's row is, and how the catalog names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RosterRow {
    /// The name the matrix draws. The project's own, measured with the layout.
    pub name: &'static str,
    /// The file's block family. The same one [`group_of`] gives the row, which the
    /// assertion under [`ROSTER`] holds at compile time.
    pub group: CharacterGroup,
    /// The name key in `players.xml`, without `#` and without `_NAME`. Keys repeat between
    /// the normal and Tainted forms: it's the pair with `tainted` that identifies the
    /// character. Verified against the file on 2026-09-03.
    pub key: &'static str,
    /// The Tainted form.
    pub tainted: bool,
}

const fn base(name: &'static str, group: CharacterGroup, key: &'static str) -> RosterRow {
    RosterRow {
        name,
        group,
        key,
        tainted: false,
    }
}

/// Every Tainted row is one of the 19-cell blocks.
const fn tainted(name: &'static str, key: &'static str) -> RosterRow {
    RosterRow {
        name,
        group: CharacterGroup::Later,
        key,
        tainted: true,
    }
}

/// The 34 rows of the matrix, in the layout's order: one table, where a name and its key used
/// to be two parallel arrays that only a test kept in step (card #82, S2). Its length is the
/// layout's [`ROWS`], so a row added to one and not the other does not compile.
pub const ROSTER: [RosterRow; ROWS] = {
    use CharacterGroup::{Forgotten, Later, Original};
    [
        base("Isaac", Original, "ISAAC"),
        base("Magdalene", Original, "MAGDALENE"),
        base("Cain", Original, "CAIN"),
        base("Judas", Original, "JUDAS"),
        base("Blue Baby", Original, "BLUEBABY"),
        base("Eve", Original, "EVE"),
        base("Samson", Original, "SAMSON"),
        base("Azazel", Original, "AZAZEL"),
        base("Lazarus", Original, "LAZARUS"),
        base("Eden", Original, "EDEN"),
        base("The Lost", Original, "THE_LOST"),
        base("Lilith", Original, "LILITH"),
        base("Keeper", Original, "KEEPER"),
        base("Apollyon", Original, "APOLLYON"),
        base("The Forgotten", Forgotten, "THE_FORGOTTEN"),
        base("Bethany", Later, "BETHANY"),
        base("Jacob & Esau", Later, "JACOB"),
        tainted("T. Isaac", "ISAAC"),
        tainted("T. Magdalene", "MAGDALENE"),
        tainted("T. Cain", "CAIN"),
        tainted("T. Judas", "JUDAS"),
        tainted("T. Blue Baby", "BLUEBABY"),
        tainted("T. Eve", "EVE"),
        tainted("T. Samson", "SAMSON"),
        tainted("T. Azazel", "AZAZEL"),
        tainted("T. Lazarus", "LAZARUS"),
        tainted("T. Eden", "EDEN"),
        tainted("T. The Lost", "THE_LOST"),
        tainted("T. Lilith", "LILITH"),
        tainted("T. Keeper", "KEEPER"),
        tainted("T. Apollyon", "APOLLYON"),
        tainted("T. Forgotten", "THE_FORGOTTEN"),
        tainted("T. Bethany", "BETHANY"),
        tainted("T. Jacob & Esau", "JACOB"),
    ]
};

/// Whether `a` and `b` are the same family, in a form a `const` can evaluate.
const fn same_group(a: CharacterGroup, b: CharacterGroup) -> bool {
    use CharacterGroup::{Forgotten, Later, Original};
    matches!(
        (a, b),
        (Original, Original) | (Forgotten, Forgotten) | (Later, Later)
    )
}

/// Every row's family is the one the layout gives its index: the roster names the rows, the
/// layout decides where their cells are, and the two cannot disagree without failing to build.
const _: () = {
    // An index walk for the same reason as `BOSSES`: iterators do not run in a `const`.
    let mut row = 0;
    while row < ROSTER.len() {
        assert!(matches!(group_of(row), Some(g) if same_group(g, ROSTER[row].group)));
        row += 1;
    }
};

/// The catalog character for row `row` of the matrix: the **first** one with that key
/// and that Tainted flag, in id order. The hidden forms (Lazarus 2, Black Judas, The
/// Soul) have their own keys and don't interfere; Esau has his own key, and the "Jacob &
/// Esau" row takes Jacob.
pub fn character_for(row: usize, catalog: &catalog::Catalog) -> Option<&catalog::Character> {
    let wanted_row = ROSTER.get(row)?;
    let wanted = format!("{}_NAME", wanted_row.key);
    catalog.characters().find(|c| {
        c.tainted == wanted_row.tainted
            && matches!(&c.name, catalog::Text::Key { key: k } if *k == wanted)
    })
}

/// Index into the counters section for the (character, boss) cell, where `boss` is a
/// position in [`BOSSES`]. `None` when the cell isn't located, or either index is out of
/// range.
///
/// The tables themselves live in `core_save::marks`: a cell's index is the shape of the
/// save file, and this module draws a screen. What stays here is the translation from the
/// screen's parallel arrays to the layout's typed column.
pub fn counter_index(character: usize, boss: usize) -> Option<usize> {
    core_save::cell_index(character, *core_save::Column::ALL.get(boss)?)
}

/// The level a cell's mark reached. Fieldless, so it crosses as a bare camelCase string
/// and the TypeScript is a union of values: a tag distinguishes variants that carry
/// different data, and there is none here (CLAUDE.md, "Enums on the IPC").
///
/// Two levels and not three: bit 2 is not one of them. It says where a mark was taken,
/// which is why it travels beside this enum and not inside it.
///
/// Not to be confused with `graph::rules::MarkLevel` (`MarkLevelView` on the wire), which names the same two bits `base`
/// and `second`. That one describes a *target* — "go and take this cell at this level" —
/// and refuses `hard` on purpose, because what bit 1 means outside Greed is unmeasured.
/// Here the matrix is being drawn and `normal`/`hard` is the vocabulary its own totals
/// have carried since B22; the two names are one measurement away from becoming one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum CellLevel {
    /// Neither level bit: this character has never beaten that boss.
    Empty,
    /// Bit 0 alone, the first level.
    Normal,
    /// Bit 1, whether or not bit 0 stands with it. A bare 2 is the first level
    /// **overwritten** and not a second level taken without the first: four located cells
    /// go 1 → 2 across the 638-era series (B58, measured 2026-09-17). What bit 1 means
    /// outside Greed is still unmeasured, which is why the name stops at the level.
    Hard,
}

impl CellLevel {
    /// Whether the mark was taken at all. A `match` over the whole enum and not a `!=
    /// Empty`, so a level added later has to come here and say which side it falls on.
    pub fn reached(self) -> bool {
        match self {
            CellLevel::Empty => false,
            CellLevel::Normal | CellLevel::Hard => true,
        }
    }
}

/// A cell of the matrix. The three variants are the module's reason for existing:
/// "never done", "not readable", and "suspicious value" are three different things.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Cell {
    /// Valid mask, read here rather than at the far end of the IPC: `bits` is the value
    /// the file holds, `level` and `online` are what it means.
    ///
    /// Bit 2 is **won online** — measured 2026-09-12 on a matched window around an online
    /// co-op run, confirmed by the owner on 2026-09-20 against the game's own completion
    /// screen (`docs/save-format.md`, "Counters and marks"). It is a field and not a
    /// level because it answers a different question.
    ///
    /// `bits` stays beside the reading for the Verify page, which exists to show what the
    /// file actually holds; every screen that draws a mark reads `level` and `online`.
    Known {
        bits: u8,
        level: CellLevel,
        online: bool,
    },
    /// Index not located in the tables, or past the end of the section read.
    Unknown,
    /// Outside 0..=7: not a mask, so the index points somewhere else.
    Unexpected { value: u32 },
}

/// `tainted` groups the rows the way a player does, while `group` stays the three blocks the
/// file itself has. Every URL is `None` when the game archives are not open, and the screen
/// draws the fallback outfit.
#[derive(Debug, Clone, Serialize, ts_rs::TS)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct MarkArtView {
    pub normal_url: Option<String>,
    pub hard_url: Option<String>,
}

/// What the screen calls a column's second level — bit 1. Fieldless, so a bare camelCase
/// string on the wire.
///
/// **Not a rename of `graph::rules::MarkLevel::Second`**, which stays named for the bit. This
/// is the word a player reads, and it is decided per column because the bit does not mean
/// one thing (B66, `docs/save-format.md`): in Greed it is Ultra Greedier, measured on
/// 2026-09-12 on three characters — a *mode*, which replaces Greed rather than adding to it;
/// in the other eleven it is hard, on one observation (Mother on hard, 2026-09-20, `0 → 3`)
/// and the owner's wording for the screen (card #58). The discriminator that would settle the
/// eleven is a win on normal on an empty cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum SecondLevelView {
    Hard,
    UltraGreedier,
}

/// Exhaustive, with no `_` arm: a thirteenth column breaks the build here instead of being
/// called hard by default.
pub fn second_level(column: Column) -> SecondLevelView {
    match column {
        Column::Greed => SecondLevelView::UltraGreedier,
        Column::MomsHeart
        | Column::Isaac
        | Column::Satan
        | Column::BossRush
        | Column::BlueBaby
        | Column::TheLamb
        | Column::MegaSatan
        | Column::Hush
        | Column::Delirium
        | Column::Mother
        | Column::TheBeast => SecondLevelView::Hard,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct MarksTotals {
    pub cells: usize,
    pub readable: usize,
    pub unknown: usize,
    pub unexpected: usize,
    /// Cells that reached *a* level — bit 0 or bit 1. A mark taken on hard counts as taken
    /// on normal too (B22), so this is the larger of the two and `hard` is a subset of it,
    /// never a tally beside it.
    pub normal: usize,
    /// Cells that reached the second level — bit 1. `hard <= normal <= readable` always.
    pub hard: usize,
}

#[derive(Debug, Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct MarksMatrix {
    pub characters: Vec<CharacterRow>,
    pub bosses: Vec<String>,
    /// `art[i]` draws `bosses[i]`: a parallel array, so `bosses` keeps the shape the design
    /// was built on.
    pub art: Vec<MarkArtView>,
    /// `second_levels[i]` names `bosses[i]`'s second level, so the screen never decides which
    /// column is Greed.
    pub second_levels: Vec<SecondLevelView>,
    pub totals: MarksTotals,
    /// The game's own completion widget, drawn for this profile: one picture, composed by
    /// the protocol handler out of the paper and the symbols the columns have earned.
    ///
    /// `None` without the game's archives, like every other URL here — the band then simply
    /// has no picture, which is the one thing that never looks broken.
    pub widget_url: Option<String>,
}

/// How much of a column the emblem draws, read off the same cells the footer's two totals
/// are read off.
///
/// **Hard is every readable cell, not one of them** — the same thing `TallyTone::Full`
/// means on the screen, and the same thing B22 settled for a row: a column is done when
/// every character has done it on hard. Normal is "at least one has a level", because a
/// column nobody has touched draws nothing rather than a faint symbol the game does not
/// have. Cells the save can't be read for stay out of both, the way they stay out of every
/// denominator.
fn column_fill(rows: &[CharacterRow], column: usize) -> crate::icon::MarkFill {
    use crate::icon::MarkFill;
    let readable: Vec<CellLevel> = rows
        .iter()
        .filter_map(|r| r.cells.get(column).and_then(readable_level))
        .collect();
    let all_hard = readable.iter().all(|&l| l == CellLevel::Hard);
    if !readable.is_empty() && all_hard {
        MarkFill::Hard
    } else if readable.iter().any(|l| l.reached()) {
        MarkFill::Normal
    } else {
        MarkFill::None
    }
}

/// The level of a cell the save could be read for; `None` for the ones it could not.
fn readable_level(cell: &Cell) -> Option<CellLevel> {
    match cell {
        Cell::Known { level, .. } => Some(*level),
        Cell::Unknown | Cell::Unexpected { .. } => None,
    }
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

    let rows: Vec<CharacterRow> = ROSTER
        .iter()
        .enumerate()
        .map(|(c, row)| CharacterRow {
            character: row.name.to_string(),
            group: row.group,
            tainted: row.tainted,
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
    let widget_url = catalog.and_then(|_| {
        let fills = std::array::from_fn(|column| column_fill(&rows, column));
        icon(&IconRef::Widget { fills })
    });
    MarksMatrix {
        characters: rows,
        bosses: BOSSES.iter().map(|b| b.to_string()).collect(),
        art,
        second_levels: Column::ALL.iter().map(|&c| second_level(c)).collect(),
        totals,
        widget_url,
    }
}

// `pub(crate)`, not private: `crate::roll::roll_space` reads the same cell the matrix
// draws. A second definition of "what a cell holds" would drift from the matrix the
// Completion screen draws, the same argument `marks_totals` already carries.
pub(crate) fn cell_at(counters: &[u32], character: usize, boss: usize) -> Cell {
    match counter_index(character, boss).and_then(|i| counters.get(i)) {
        None => Cell::Unknown,
        Some(&value) if value <= 7 => {
            let bits = value as u8;
            Cell::Known {
                bits,
                level: level_of(bits),
                online: bits & 4 != 0,
            }
        }
        Some(&value) => Cell::Unexpected { value },
    }
}

/// Bit 1 decides on its own, because a mark replaces the one before it rather than
/// accumulating: requiring bit 0 as well would read a bare 2 as no level at all.
fn level_of(bits: u8) -> CellLevel {
    if bits & 2 != 0 {
        CellLevel::Hard
    } else if bits & 1 != 0 {
        CellLevel::Normal
    } else {
        CellLevel::Empty
    }
}

/// The totals alone, without the matrix. The welcome's preview needs the numbers and none of
/// the art (`docs/superpowers/specs/2026-09-17-welcome-flow-design.md` §4), and a second
/// definition of "a mark is taken" would drift from this one.
pub fn marks_totals(counters: &[u32]) -> MarksTotals {
    let cells: Vec<Cell> = (0..ROSTER.len())
        .flat_map(|c| (0..BOSSES.len()).map(move |b| cell_at(counters, c, b)))
        .collect();
    totals_from(&cells)
}

fn totals_of(rows: &[CharacterRow]) -> MarksTotals {
    let cells: Vec<Cell> = rows.iter().flat_map(|r| r.cells.iter().copied()).collect();
    totals_from(&cells)
}

fn totals_from(cells: &[Cell]) -> MarksTotals {
    let count = |f: fn(&Cell) -> bool| cells.iter().filter(|&c| f(c)).count();
    MarksTotals {
        cells: cells.len(),
        readable: count(|c| matches!(c, Cell::Known { .. })),
        unknown: count(|c| matches!(c, Cell::Unknown)),
        unexpected: count(|c| matches!(c, Cell::Unexpected { .. })),
        // Every cell that reached *a* level. Read off `level` and not off the mask: the
        // online bit alone is not a level, and a total that counted it would say a mark was
        // taken where the grid draws none.
        normal: count(|c| matches!(c, Cell::Known { level, .. } if level.reached())),
        // The ones that reached the second, so `hard <= normal` holds by construction.
        hard: count(|c| {
            matches!(
                c,
                Cell::Known {
                    level: CellLevel::Hard,
                    ..
                }
            )
        }),
    }
}
