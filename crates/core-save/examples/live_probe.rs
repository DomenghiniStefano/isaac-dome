//! Throwaway probe: what the game does to its own files while a run is in progress.
//!
//! Not a test and not shipped — it answers questions no fixture can, because they are all
//! about a file another process is writing right now:
//!
//! 1. **When does the game write the save, and is the write atomic?** The app re-reads on
//!    change; if the game truncates and rewrites in place we can read a torn file, which
//!    is a "degrade, never fail" case we have never actually observed.
//! 2. **What moves when it writes mid-run?** `core_save::diff` already answers this
//!    between two dated snapshots; here it answers it between one second and the next.
//! 3. **How fast does `log.txt` reach the disk?** That decides whether a Live screen is
//!    live or several rooms behind (backlog B8).
//!
//! Read-only throughout: it opens nothing for writing but its own report.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use core_save::{diff, Kind, Save};

/// Cheap content fingerprint. Not a checksum of the format — just "did these bytes move".
fn fingerprint(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x1000_0000_01b3);
    }
    h
}

/// How many changed counters to name before summarising the rest. A write during a boss
/// fight moves a handful; a line that named all of them would be unreadable.
const NAMED_SHOWN: usize = 8;

fn describe(previous: Option<&Save>, now: &Save) -> String {
    let Some(before) = previous else {
        let done = |k: Kind| {
            now.flags(k)
                .map(|f| f.iter().filter(|b| **b).count())
                .unwrap_or(0)
        };
        return format!(
            "initial\tach {}\titems {}\tchallenges {}",
            done(Kind::Achievements),
            done(Kind::Items),
            done(Kind::Challenges)
        );
    };
    let d = diff(before, now);
    let mut parts = Vec::new();
    if !d.achievements.is_empty() {
        parts.push(format!("ACHIEVEMENTS {:?}", d.achievements));
    }
    if !d.items.is_empty() {
        parts.push(format!("items {:?}", d.items));
    }
    if !d.challenges.is_empty() {
        parts.push(format!("challenges {:?}", d.challenges));
    }
    if !d.bosses.is_empty() {
        parts.push(format!("bosses {:?}", d.bosses));
    }
    if !d.counters.is_empty() {
        let named: Vec<String> = d
            .counters
            .iter()
            .take(NAMED_SHOWN)
            .map(|(i, a, b)| format!("{} [{i}] {a}->{b}", label(*i)))
            .collect();
        let rest = d.counters.len().saturating_sub(NAMED_SHOWN);
        parts.push(if rest > 0 {
            format!("counters {} (+{rest} more)", named.join(" "))
        } else {
            format!("counters {}", named.join(" "))
        });
    }
    if parts.is_empty() {
        // The bytes moved but nothing we decode did: the header's u32 at 0x10, which
        // CLAUDE.md records as "changes on every save, meaning unknown", is the candidate.
        "bytes moved, no decoded field changed".into()
    } else {
        parts.join("\t")
    }
}

fn main() {
    let home = std::env::var("USERPROFILE").unwrap_or_else(|_| ".".into());
    let docs = PathBuf::from(&home).join("Documents/My Games/Binding of Isaac Repentance+");
    let remote = PathBuf::from("C:/Program Files (x86)/Steam/userdata/123456789/250900/remote");

    let log = docs.join("log.txt");
    let saves: Vec<PathBuf> = ["rep+persistentgamedata1.dat", "rep+persistentgamedata2.dat"]
        .iter()
        .map(|n| remote.join(n))
        .collect();

    let out = PathBuf::from("samples/logs/probe.tsv");
    let mut report = std::fs::File::create(&out).expect("the probe writes its own report");
    let started = Instant::now();
    let mut prints: BTreeMap<PathBuf, u64> = BTreeMap::new();
    let mut last: BTreeMap<PathBuf, Save> = BTreeMap::new();
    let mut log_size: u64 = 0;

    writeln!(report, "# elapsed\tsubject\tevent").ok();
    // Half a second: the question is the size of the gap between the game's write and our
    // read, so polling has to be finer than the answer we expect.
    while started.elapsed() < Duration::from_secs(3600) {
        if let Ok(m) = std::fs::metadata(&log) {
            if m.len() != log_size {
                writeln!(
                    report,
                    "{:.1}\tlog.txt\t{} bytes ({:+})",
                    started.elapsed().as_secs_f64(),
                    m.len(),
                    m.len() as i64 - log_size as i64
                )
                .ok();
                log_size = m.len();
            }
        }
        for p in &saves {
            let Ok(bytes) = std::fs::read(p) else {
                continue;
            };
            let f = fingerprint(&bytes);
            if prints.get(p) == Some(&f) {
                continue;
            }
            prints.insert(p.clone(), f);
            let name = p
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let at = started.elapsed().as_secs_f64();
            match Save::parse(&bytes) {
                Ok(s) => {
                    writeln!(
                        report,
                        "{at:.1}\t{name}\tCHANGED\t{} bytes\t{}",
                        bytes.len(),
                        describe(last.get(p), &s)
                    )
                    .ok();
                    last.insert(p.clone(), s);
                }
                // The interesting line. A torn write shows up here and nowhere else.
                Err(e) => {
                    writeln!(
                        report,
                        "{at:.1}\t{name}\tPARSE FAILED\t{} bytes\t{e:?}",
                        bytes.len()
                    )
                    .ok();
                }
            }
        }
        report.flush().ok();
        std::thread::sleep(Duration::from_millis(500));
    }
}

// --- counter labels, ported from `reference/isaac_counters.py` ---
//
// Only so the live stream reads as actions instead of indices. A label derived from the
// regular block pattern rather than documented by REPENTOGON is prefixed `~`, because that
// pattern is known to break down past Hush and a guess must never look like a fact.

const CHARS_14: [&str; 14] = [
    "Isaac",
    "Magdalene",
    "Cain",
    "Judas",
    "Blue Baby",
    "Eve",
    "Samson",
    "Azazel",
    "Lazarus",
    "Eden",
    "The Lost",
    "Lilith",
    "Keeper",
    "Apollyon",
];

const CHARS_19: [&str; 19] = [
    "Bethany",
    "Jacob & Esau",
    "T. Isaac",
    "T. Magdalene",
    "T. Cain",
    "T. Judas",
    "T. Blue Baby",
    "T. Eve",
    "T. Samson",
    "T. Azazel",
    "T. Lazarus",
    "T. Eden",
    "T. The Lost",
    "T. Lilith",
    "T. Keeper",
    "T. Apollyon",
    "T. Forgotten",
    "T. Bethany",
    "T. Jacob & Esau",
];

const DOCUMENTED: &[(usize, &str)] = &[
    (0, "NULL"),
    (1, "MOM_KILLS"),
    (2, "ROCKS_DESTROYED"),
    (3, "TINTED_ROCKS_DESTROYED"),
    (4, "SUPER_SPECIAL_ROCKS_DESTROYED"),
    (5, "POOP_DESTROYED"),
    (6, "PILLS_EATEN"),
    (7, "XIII_DEATH_CARD_USED"),
    (8, "UNKNOWN_EVENT_8"),
    (9, "ARCADES_ENTERED"),
    (10, "DEATHS"),
    (11, "ISAAC_KILLS"),
    (12, "SHOPKEEPER_KILLED"),
    (13, "SATAN_KILLS"),
    (14, "SHELLGAMES_PLAYED"),
    (15, "ANGEL_DEALS_TAKEN"),
    (16, "DEVIL_DEALS_TAKEN"),
    (17, "BLOOD_DONATION_MACHINE_USED"),
    (18, "SLOT_MACHINES_BROKEN"),
    (19, "UNKNOWN_EVENT_19"),
    (20, "DONATION_MACHINE_COUNTER"),
    (21, "EDEN_TOKENS"),
    (22, "STREAK_COUNTER"),
    (23, "BEST_STREAK"),
    (24, "BLUE_BABY_KILLS"),
    (25, "LAMB_KILLS"),
    (26, "MEGA_SATAN_KILLS"),
    (111, "BOSSRUSHS_CLEARED"),
    (112, "UNKNOWN_EVENT_112"),
    (113, "NEGATIVE_STREAK_COUNTER"),
    (114, "EDENS_BLESSINGS_NEXT_RUN"),
    (115, "GREED_DONATION_MACHINE_COUNTER"),
    (158, "HUSH_KILLS"),
    (187, "DELIRIUM_KILLS"),
    (188, "CHARACTER_LAST_RUN_WIN"),
    (189, "UNKNOWN_EVENT_189"),
    (190, "DAILYS_PLAYED"),
    (191, "DAILY_RUN_IS_ACTIVE"),
    (192, "DAILYS_STREAK"),
    (193, "DAILYS_WON"),
    (194, "RAINBOW_POOP_DESTROYED"),
    (195, "BATTERIES_COLLECTED"),
    (196, "CARDS_USED"),
    (197, "SHOP_ITEMS_BOUGHT"),
    (198, "CHESTS_OPENED_WITH_KEY"),
    (199, "SECRET_ROOMS_WALLS_OPENED"),
    (200, "BLOOD_CLOT_ITEM_AQUIRED"),
    (201, "RUBBER_CEMENT_ITEM_AQUIRED"),
    (202, "BEDS_USED"),
    (212, "GREED_COINS_DONATED_FORGOTTEN"),
];

/// One cell per original character, for each boss.
const BLOCKS_14: &[(&str, usize)] = &[
    ("Mom's Heart", 27),
    ("Isaac", 41),
    ("Satan", 55),
    ("Boss Rush", 69),
    ("Blue Baby", 83),
    ("The Lamb", 97),
    ("Mega Satan", 116),
    ("Greed", 130),
    ("Hush", 144),
    ("Delirium", 173),
];

/// The Forgotten's own cells, added after the original fourteen.
const FORGOTTEN: &[(usize, &str)] = &[
    (203, "Mom's Heart"),
    (204, "Isaac"),
    (205, "Satan"),
    (206, "Boss Rush"),
    (207, "Blue Baby"),
    (208, "The Lamb"),
    (209, "Mega Satan"),
    (210, "Greed"),
    (211, "Hush"),
    (213, "Delirium"),
];

/// Bethany, Jacob & Esau and the seventeen Tainted. Derived from the pattern, not
/// documented: the regularity is known to break down past Hush.
const BLOCKS_19: &[(&str, usize)] = &[
    ("Mom's Heart", 214),
    ("Isaac", 233),
    ("Satan", 252),
    ("Boss Rush", 271),
    ("Blue Baby", 290),
    ("The Lamb", 309),
    ("Mega Satan", 328),
    ("Greed", 347),
    ("Hush", 366),
];

fn label(i: usize) -> String {
    if let Some((_, n)) = DOCUMENTED.iter().find(|(k, _)| *k == i) {
        return (*n).to_string();
    }
    if let Some((_, boss)) = FORGOTTEN.iter().find(|(k, _)| *k == i) {
        return format!("MARK/{boss}/The Forgotten");
    }
    for (boss, base) in BLOCKS_14 {
        if i >= *base && i < base + CHARS_14.len() {
            return format!("MARK/{boss}/{}", CHARS_14[i - base]);
        }
    }
    for (boss, base) in BLOCKS_19 {
        if i >= *base && i < base + CHARS_19.len() {
            return format!("~MARK/{boss}/{}", CHARS_19[i - base]);
        }
    }
    // 385 and the 404-522 tail have no label anyone has confirmed.
    "?".to_string()
}
