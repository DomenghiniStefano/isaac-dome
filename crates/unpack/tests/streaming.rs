//! How much it costs to **open** the archives, and that reading per entry gives back
//! the same bytes.
//!
//! The eight `.a` files of a full installation weigh about 1.3 GB. Loading them into
//! RAM to answer a question about one icon is the most likely way the app crashes
//! silently on some stranger's machine — a violation of the "degrade, never fail"
//! constraint. Opening an archive must cost the index, not the file.
//!
//! The peak is measured for real: a test global allocator counts live bytes.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use unpack::{Archive, ResourceSet};

/// Bytes allocated and never yet freed, and the maximum reached. This isn't a measure
/// of the process's working set (pages mapped by the system don't pass through here):
/// it's the peak of this program's heap, which is what differs between "I read the
/// whole file" and "I read the index".
static VIVI: AtomicUsize = AtomicUsize::new(0);
static PICCO: AtomicUsize = AtomicUsize::new(0);

struct Contatore;

unsafe impl GlobalAlloc for Contatore {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            let ora = VIVI.fetch_add(layout.size(), Ordering::Relaxed) + layout.size();
            PICCO.fetch_max(ora, Ordering::Relaxed);
        }
        p
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        VIVI.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATORE: Contatore = Contatore;

/// The threshold also declared in `unpack`'s spec: opening every archive of a full
/// installation must stay **under 64 MB** of heap. It's deliberately loose — it doesn't
/// measure a line of code, it measures that the order of magnitude is the index (units
/// of MB) and not the data (more than a thousand MB).
const SOGLIA_APERTURA: usize = 64 * 1024 * 1024;

/// How many entries per archive get reread in the equivalence test. All of them would
/// be tens of thousands of decompressions: a regular stride is taken instead, and the
/// test states how many.
const VOCI_PER_ARCHIVIO: usize = 200;

#[test]
fn opening_every_archive_costs_the_index_not_the_data() {
    let Some(dir) = test_support::packed_dir() else {
        return;
    };
    let prima = VIVI.load(Ordering::Relaxed);
    PICCO.store(prima, Ordering::Relaxed);

    let rs = ResourceSet::open(&dir);
    let picco = PICCO.load(Ordering::Relaxed).saturating_sub(prima);

    // The set must have opened something, otherwise a low peak proves nothing.
    let voci: usize = rs.archives().iter().map(|a| a.entries).sum();
    assert!(
        rs.archives().len() >= 4 && voci > 10_000,
        "archives opened: {}, entries: {voci}",
        rs.archives().len()
    );
    eprintln!(
        "sample: {} archives, {voci} entries indexed, peak {} KiB",
        rs.archives().len(),
        picco / 1024
    );
    assert!(
        picco < SOGLIA_APERTURA,
        "opening the archives allocated {} MiB, above the threshold of {} MiB",
        picco / 1024 / 1024,
        SOGLIA_APERTURA / 1024 / 1024
    );
}

/// Reading one entry at a time must give the same bytes as before: the length produced
/// is the one the index declares. Runs on every archive, hence on all three compression
/// modes, and states how many entries.
#[test]
fn every_entry_still_decompresses_to_the_length_the_index_declares() {
    let Some(dir) = test_support::packed_dir() else {
        return;
    };
    for nome in [
        "config.a",
        "fonts.a",
        "animations.a",
        "graphics.a",
        "music.a",
        "afterbirth.a",
        "afterbirthp.a",
        "repentance.a",
    ] {
        let Ok(a) = Archive::open(&dir.join(nome)) else {
            test_support::skip(&format!("packed/{nome} missing"));
            continue;
        };
        let totale = a.entries().len();
        let passo = (totale / VOCI_PER_ARCHIVIO).max(1);
        let mut lette = 0;
        for i in (0..totale).step_by(passo) {
            let attesa = a.entries()[i].decompressed_len as usize;
            let Some(bytes) = a.read_entry(i) else {
                panic!("{nome}: entry {i} no longer reads");
            };
            assert_eq!(bytes.len(), attesa, "{nome}: entry {i}");
            lette += 1;
        }
        eprintln!("sample: packed/{nome} — {lette} entries reread out of {totale}");
    }
}

/// The archive's last entry is the one with no following entry to bound its data: its
/// end is the start of the index. It's the first case a wrong boundary calculation gets
/// wrong, so it's read explicitly.
#[test]
fn the_last_entry_of_an_archive_reads_too() {
    let Some(path) = test_support::packed_file("config.a") else {
        return;
    };
    let a = Archive::open(&path).expect("config.a opens");
    let ultima = a
        .entries()
        .iter()
        .enumerate()
        .max_by_key(|(_, e)| e.offset)
        .map(|(i, _)| i)
        .expect("config.a has entries");
    let attesa = a.entries()[ultima].decompressed_len as usize;
    assert_eq!(a.read_entry(ultima).map(|b| b.len()), Some(attesa));
}
