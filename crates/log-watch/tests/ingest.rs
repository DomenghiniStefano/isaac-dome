//! Backfill and live are the same function over the same events. These tests are what says so:
//! the session path and the log path differ only in where the bytes come from.

use std::fs;
use std::path::Path;

use log_watch::Ingest;
use run::{ItemKind, ItemKinds, Outcome, Rules};
use store::Store;

/// The fold needs item kinds and must not depend on `catalog`. A table is the whole point of
/// the trait.
struct Table;
impl ItemKinds for Table {
    fn kind_of(&self, id: u32) -> ItemKind {
        match id {
            105 => ItemKind::Active,
            _ => ItemKind::Passive,
        }
    }
}

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("isaacdome.db")).unwrap();
    (dir, store)
}

/// A log the game could have written: the prefix every line carries, one run, one item, an
/// ending. Kept in one place so a test can say what it changed.
fn log_text(seed: &str, ending: &str) -> String {
    format!(
        "[INFO] - OpenGL version 4.6.0 NVIDIA 610.88\n\
         [INFO] - RNG Start Seed: {seed} (586324166) [New, 1]\n\
         [INFO] - Adding collectible 34 (Book of Belial) to player 0 (Judas) from pool treasure\n\
         [INFO] - [Frame 74] Starting room transition (type 0)\n\
         [INFO] - Level::Init m_Stage 2, m_StageType 1 Seed 408474304\n\
         [INFO] - Adding collectible 105 (The D6) to player 0 (Judas) from pool treasure\n\
         [INFO] - playing cutscene 15 ({ending}).\n"
    )
}

/// The same log without its opening banner line, for appending a second run to a file that is
/// already open.
fn without_banner(text: &str) -> String {
    text.split_once('\n')
        .map(|(_, rest)| rest.to_string())
        .unwrap()
}

fn session_folder(root: &Path, name: &str, text: &str) -> std::path::PathBuf {
    let dir = root.join("sessions").join(name);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("log.txt"), text).unwrap();
    dir
}

fn ingest<'a>(store: &'a Store, rules: &'a Rules, kinds: &'a Table) -> Ingest<'a> {
    Ingest {
        store,
        rules,
        kinds,
    }
}

#[test]
fn a_session_becomes_a_run_and_the_second_import_adds_nothing() {
    // Re-reading a folder must find the same source and import nothing twice: that is what the
    // folder's name is for.
    let (_d, store) = open();
    let rules = Rules::embedded();
    let tmp = tempfile::tempdir().unwrap();
    let folder = session_folder(
        tmp.path(),
        "09_12_2026__13_34_26",
        &log_text("AAAA AAAA", "Sheol"),
    );

    let first = ingest(&store, &rules, &Table)
        .session(&folder)
        .unwrap()
        .expect("a folder nobody has read yet");
    assert_eq!(first.runs, 1);
    assert!(first.events > 0);

    let again = ingest(&store, &rules, &Table).session(&folder).unwrap();
    assert!(again.is_none());

    let runs = store
        .cached_runs(first.source_id, rules.version())
        .unwrap()
        .unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].character.as_deref(), Some("Judas"));
    assert_eq!(runs[0].starting_items, vec![34]);
    assert_eq!(runs[0].held_active, Some(105));
    assert_eq!(
        runs[0].outcome,
        Outcome::Won {
            ending: "Sheol".to_string()
        }
    );
}

#[test]
fn backfill_reads_every_session_once_and_then_has_nothing_left_to_do() {
    let (_d, store) = open();
    let rules = Rules::embedded();
    let tmp = tempfile::tempdir().unwrap();
    session_folder(
        tmp.path(),
        "09_12_2026__13_34_26",
        &log_text("AAAA AAAA", "Sheol"),
    );
    session_folder(
        tmp.path(),
        "09_13_2026__14_48_32",
        &log_text("BBBB BBBB", "Cathedral"),
    );

    let (done, errors) = ingest(&store, &rules, &Table).backfill(tmp.path());
    assert_eq!(done.len(), 2);
    assert!(errors.is_empty(), "{errors:?}");

    let (again, _) = ingest(&store, &rules, &Table).backfill(tmp.path());
    assert!(again.is_empty(), "a second backfill imports nothing");
}

#[test]
fn a_log_that_grew_appends_only_what_is_new() {
    let (_d, store) = open();
    let rules = Rules::embedded();
    let tmp = tempfile::tempdir().unwrap();
    let log = tmp.path().join("log.txt");
    fs::write(&log, log_text("AAAA AAAA", "Sheol")).unwrap();

    let first = ingest(&store, &rules, &Table).live_log(&log).unwrap();

    let mut grown = log_text("AAAA AAAA", "Sheol");
    grown.push_str(&without_banner(&log_text("BBBB BBBB", "Cathedral")));
    fs::write(&log, &grown).unwrap();

    let second = ingest(&store, &rules, &Table).live_log(&log).unwrap();
    assert_eq!(second.source_id, first.source_id, "the same launch");
    assert_eq!(second.runs, 2, "the fold sees both runs");
    // The two runs carry the same six events each, so counting the second pass alone says
    // nothing. What says it is the total: reading from zero again would file the first run's
    // events a second time and leave eighteen.
    assert_eq!(
        store.events(second.source_id).unwrap().events.len(),
        (first.events + second.events) as usize
    );
    assert_eq!(store.events(second.source_id).unwrap().events.len(), 12);
}

#[test]
fn a_relaunch_is_a_second_source_and_the_first_keeps_its_runs() {
    // The game rewrites log.txt on every launch. The runs of the previous launch are the
    // archive: losing them is the failure backfill exists to prevent.
    let (_d, store) = open();
    let rules = Rules::embedded();
    let tmp = tempfile::tempdir().unwrap();
    let log = tmp.path().join("log.txt");
    let mut long = log_text("AAAA AAAA", "Sheol");
    long.push_str(&without_banner(&log_text("CCCC CCCC", "Sheol")));
    fs::write(&log, &long).unwrap();
    let first = ingest(&store, &rules, &Table).live_log(&log).unwrap();

    // A new launch: the same banner, different content, and shorter than what we had read.
    fs::write(&log, log_text("BBBB BBBB", "Cathedral")).unwrap();
    let second = ingest(&store, &rules, &Table).live_log(&log).unwrap();

    assert_ne!(second.source_id, first.source_id);
    let kept = store
        .cached_runs(first.source_id, rules.version())
        .unwrap()
        .unwrap();
    assert_eq!(kept[0].seed_words, "AAAA AAAA");
}

#[test]
fn a_line_the_game_had_not_finished_writing_is_read_on_the_next_pass() {
    // The read stops wherever the game was writing. The stored offset is the end of the last
    // complete line, so the half-written one is not lost.
    let (_d, store) = open();
    let rules = Rules::embedded();
    let tmp = tempfile::tempdir().unwrap();
    let log = tmp.path().join("log.txt");
    let full = log_text("AAAA AAAA", "Sheol");
    let cut = full.rfind("playing cutscene").unwrap() + 20;
    fs::write(&log, &full[..cut]).unwrap();

    let first = ingest(&store, &rules, &Table).live_log(&log).unwrap();
    let open_run = store
        .cached_runs(first.source_id, rules.version())
        .unwrap()
        .unwrap();
    assert_eq!(open_run[0].outcome, Outcome::Open, "no ending yet");

    fs::write(&log, &full).unwrap();
    let second = ingest(&store, &rules, &Table).live_log(&log).unwrap();
    assert_eq!(second.source_id, first.source_id, "still the same launch");
    let done = store
        .cached_runs(second.source_id, rules.version())
        .unwrap()
        .unwrap();
    assert_eq!(
        done[0].outcome,
        Outcome::Won {
            ending: "Sheol".to_string()
        },
        "the line that was half-written arrived"
    );
}

#[test]
fn the_logs_the_game_actually_wrote_import_into_runs() {
    // Real data, declared: a test on real data has to say which slice of the domain it ran on.
    let logs = test_support::log_samples();
    if logs.is_empty() {
        test_support::skip("no *.log.txt in samples/logs/");
        return;
    }
    let rules = Rules::embedded();
    for log in &logs {
        // One store per log: two real logs are two launches, and this test is about each of them
        // producing runs, not about how they relate.
        let (_d, store) = open();
        let done = ingest(&store, &rules, &Table).live_log(log).unwrap();
        let runs = store.cached_runs(done.source_id, rules.version()).unwrap();
        assert!(
            runs.is_some_and(|r| !r.is_empty()),
            "{} produced no run at all",
            log.display()
        );
    }
}

#[test]
fn a_launch_with_no_run_in_it_caches_an_empty_list_and_not_nothing() {
    // The shape B60 put in `samples/launches/`: the game was launched, it played the intro, it
    // shut down. The fold answers zero runs — an answer, not an absence — and until migration 5
    // the cache could not hold it: the source read back exactly like one nobody had ever folded,
    // so the next pass folded the same nothing again, and the one after that too.
    let (_d, store) = open();
    let rules = Rules::embedded();
    let tmp = tempfile::tempdir().unwrap();
    let log = tmp.path().join("log.txt");
    fs::write(
        &log,
        "[INFO] - OpenGL version 4.6.0 NVIDIA 610.88\n\
         [INFO] - playing cutscene 1 (Intro).\n",
    )
    .unwrap();

    let done = ingest(&store, &rules, &Table).live_log(&log).unwrap();
    assert_eq!(done.runs, 0, "nobody started a run");
    assert!(
        done.events > 0,
        "the launch spoke: a silent instrument would prove nothing about the fold"
    );
    assert_eq!(
        store.cached_runs(done.source_id, rules.version()).unwrap(),
        Some(vec![]),
        "folded into nothing, which is not the same as never folded"
    );
}
