use serde::Serialize;

/// The ten sections of the save.
///
/// A name here is either **structural** — the position in the file, nothing claimed about
/// the content — or **earned by a measurement of its own**. The `Unknown*` variants are
/// the first kind and say so. `LevelCounters` and `Bosses` are the second: they used to
/// read `PerChar` and `CardsPills`, labels that came from watching bits flip, and both
/// were wrong. See `docs/BACKLOG.md`, B9.
///
/// The game's own names, printed as `Reading chunk N` when it loads a profile, are strong
/// evidence but not a measurement — a log line says what the game thinks it is reading.
/// Sections 5 and 9 have such a name (Mini Bosses, GameSettings) and stay `Unknown` here
/// until one of them is checked against the bytes, which is the whole point of the
/// distinction.
///
/// **Section 8 earned its name on 2026-09-15** and stopped being `Unknown8`. A matched
/// window around one solo run: the log plays `playing cutscene 22 (The Void)` exactly once
/// and cell **22** of this section rises by exactly one, which is the second point on the
/// identity mapping after 19. The game's `Reading chunk 8 / Cutscene Counters` agrees, and
/// now it is agreeing with something rather than standing alone.
///
/// **What the name does not settle**: cell **2** also rose by one in that window, no
/// cutscene 2 was played in the log that survives, and cell 1 did **not** move although the
/// Intro (cutscene 1) did play. So the identity holds where it was checked and this section
/// is not simply "cutscene N at index N" — see `docs/BACKLOG.md`, B9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Achievements,
    Counters,
    LevelCounters,
    Items,
    Unknown5,
    Bosses,
    Challenges,
    CutsceneCounters,
    Unknown9,
    Bestiary,
}

impl Kind {
    /// The section's sequence number as it appears in the file (1..=10).
    pub fn number(self) -> u32 {
        match self {
            Kind::Achievements => 1,
            Kind::Counters => 2,
            Kind::LevelCounters => 3,
            Kind::Items => 4,
            Kind::Unknown5 => 5,
            Kind::Bosses => 6,
            Kind::Challenges => 7,
            Kind::CutsceneCounters => 8,
            Kind::Unknown9 => 9,
            Kind::Bestiary => 10,
        }
    }

    /// Maps the file's sequence number to a `Kind`. `None` if outside 1..=10.
    pub fn from_number(n: u32) -> Option<Kind> {
        Some(match n {
            1 => Kind::Achievements,
            2 => Kind::Counters,
            3 => Kind::LevelCounters,
            4 => Kind::Items,
            5 => Kind::Unknown5,
            6 => Kind::Bosses,
            7 => Kind::Challenges,
            8 => Kind::CutsceneCounters,
            9 => Kind::Unknown9,
            10 => Kind::Bestiary,
            _ => return None,
        })
    }

    /// Bytes per entry ON DISK. `None` = variable length up to `end-4`
    /// (bestiary only: the header's `count` doesn't count the records).
    pub fn bytes_per_entry(self) -> Option<usize> {
        match self {
            Kind::Achievements | Kind::Items | Kind::Unknown5 | Kind::Bosses | Kind::Challenges => {
                Some(1)
            }
            Kind::Counters | Kind::LevelCounters | Kind::CutsceneCounters | Kind::Unknown9 => {
                Some(4)
            }
            Kind::Bestiary => None,
        }
    }
}

/// A section read from the file: header + raw data bytes.
#[derive(Debug, Clone, Serialize)]
pub struct Section {
    pub kind: Kind,
    pub count: u32,
    pub f2: u32,
    pub offset: usize,
    pub bytes: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_and_from_number_are_inverse() {
        for n in 1u32..=10 {
            let k = Kind::from_number(n).expect("1..=10 is valid");
            assert_eq!(k.number(), n);
        }
        assert_eq!(Kind::from_number(0), None);
        assert_eq!(Kind::from_number(11), None);
    }

    #[test]
    fn bytes_per_entry_matches_format() {
        assert_eq!(Kind::Achievements.bytes_per_entry(), Some(1));
        assert_eq!(Kind::Items.bytes_per_entry(), Some(1));
        assert_eq!(Kind::Bosses.bytes_per_entry(), Some(1));
        assert_eq!(Kind::Challenges.bytes_per_entry(), Some(1));
        assert_eq!(Kind::Unknown5.bytes_per_entry(), Some(1));
        assert_eq!(Kind::Counters.bytes_per_entry(), Some(4));
        assert_eq!(Kind::LevelCounters.bytes_per_entry(), Some(4));
        assert_eq!(Kind::CutsceneCounters.bytes_per_entry(), Some(4));
        assert_eq!(Kind::Unknown9.bytes_per_entry(), Some(4));
        assert_eq!(Kind::Bestiary.bytes_per_entry(), None);
    }
}
