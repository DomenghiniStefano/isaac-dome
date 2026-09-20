//! Where the completion marks and the located tallies live inside section 2.
//!
//! This is the file's shape, which is why it is here: a cell's index is the same kind of
//! knowledge as a section header's offset. It used to live in the view-model that draws
//! the matrix, which made the screen the only place that knew it — and left the graph,
//! which now has to ask the same question, with nowhere to ask it.
//!
//! Nothing here reads a file. It maps (row, column) and a tally's name onto indices, and
//! says `None` for the cells nobody has located.

use serde::Serialize;

/// The twelve columns the game's own completion widget draws, in its order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Column {
    MomsHeart,
    Isaac,
    Satan,
    BossRush,
    BlueBaby,
    TheLamb,
    MegaSatan,
    Greed,
    Hush,
    Delirium,
    Mother,
    TheBeast,
}

impl Column {
    /// Position in the per-boss tables below. Written out rather than derived, so that
    /// reordering the enum cannot silently reindex every table.
    pub fn position(self) -> usize {
        match self {
            Column::MomsHeart => 0,
            Column::Isaac => 1,
            Column::Satan => 2,
            Column::BossRush => 3,
            Column::BlueBaby => 4,
            Column::TheLamb => 5,
            Column::MegaSatan => 6,
            Column::Greed => 7,
            Column::Hush => 8,
            Column::Delirium => 9,
            Column::Mother => 10,
            Column::TheBeast => 11,
        }
    }

    pub const ALL: [Column; 12] = [
        Column::MomsHeart,
        Column::Isaac,
        Column::Satan,
        Column::BossRush,
        Column::BlueBaby,
        Column::TheLamb,
        Column::MegaSatan,
        Column::Greed,
        Column::Hush,
        Column::Delirium,
        Column::Mother,
        Column::TheBeast,
    ];
}

/// Which block family a row belongs to. It is the file's structure, not the player's way
/// of grouping characters: the three families are where the unread cells live.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum CharacterGroup {
    /// The 14 originals: 14-cell blocks, verified.
    Original,
    /// The Forgotten, added later: single cells.
    Forgotten,
    /// Bethany, Jacob & Esau, and the 17 Tainted: 19-cell blocks, derived.
    Later,
}

/// How many rows the matrix has, and which family each belongs to.
pub const ROWS: usize = 34;

const FIRST_LATER: usize = 15;

pub fn group_of(row: usize) -> Option<CharacterGroup> {
    match row {
        0..=13 => Some(CharacterGroup::Original),
        14 => Some(CharacterGroup::Forgotten),
        15..=33 => Some(CharacterGroup::Later),
        _ => None,
    }
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
/// Mother closed on 2026-09-20, on a window where T. Eden beat Mother: `[449]` was the only
/// cell to move in all of 423..=490, which puts the 19-block at 438 and leaves **437** — the
/// one cell over in 437..=456 — to The Forgotten. Not a cell anybody was seen earning, but
/// the only one the measured block does not claim.
///
/// The Beast is still `None`, and on purpose. The spacing between the two 14-blocks is
/// exactly 34 = 14 + 1 + 19, so its cell is certainly inside 471..=490, and Mother's group
/// now says where a group of 34 puts The Forgotten. That is an inference from one worked
/// example and not a window on this half: a run of The Beast with any of those 20
/// characters is what closes it. A guess here would show a mark nobody earned.
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
    Some(437),
    None,
];

/// Base of the 19-cell block, per boss. DERIVED from the regular pattern for the first
/// nine, and each one corroborated by cells that were seen moving. `None` = not located.
///
/// Delirium closed on 2026-09-08: the base is 404, not the 386 the pattern predicted, and
/// four characters agree on it — Bethany (+0), Jacob & Esau (+1), T. Cain (+4) and
/// T. Azazel (+9), each on a day the Delirium kill counter also rose. What sits in
/// 386..=403 is still unread.
///
/// Mother closed on 2026-09-20, and on **one** character rather than four: T. Eden (+11),
/// row 26, whose cell `[449]` was the only one to move in 423..=490 across the window. The
/// three facts that pinned 423 and 457 all agree on it — achievement 567, whose requirement
/// is *Mother* + *Tainted Eden*; the Mother kills tally up by exactly one; and index 188 at
/// `1 << 30`, which is that character's id and not its row here. One character is enough
/// where four were needed for Delirium because this block is bracketed on both sides by
/// measured bases: 423..=436 below it, 457 above, with no slack for it to sit anywhere else.
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
    Some(438),
    None,
];

/// Index into section 2 for the (row, column) cell. `None` when the cell isn't located in
/// the tables, or the row isn't one of the 34.
pub fn cell_index(row: usize, column: Column) -> Option<usize> {
    let boss = column.position();
    match group_of(row)? {
        CharacterGroup::Original => Some(BLOCKS_14[boss] + row),
        CharacterGroup::Forgotten => FORGOTTEN[boss],
        CharacterGroup::Later => BLOCKS_19[boss].map(|base| base + row - FIRST_LATER),
    }
}

/// The tallies of section 2 whose index is located.
///
/// Typed rather than a string, so the crates that name one cannot drift apart in silence:
/// a rules file is allowed to carry `"hushKills"` precisely because the number lives here,
/// and an exhaustive match is what keeps the two spellings joined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CounterKey {
    HushKills,
    DeliriumKills,
    MotherKills,
    BeastKills,
}

/// Located on the series: 158 and 187 carry documented REPENTOGON names, 491 and 492 were
/// pinned on 2026-09-08 by rising with the Mother and The Beast mark blocks.
pub fn counter_index_of(key: CounterKey) -> usize {
    match key {
        CounterKey::HushKills => 158,
        CounterKey::DeliriumKills => 187,
        CounterKey::MotherKills => 491,
        CounterKey::BeastKills => 492,
    }
}
