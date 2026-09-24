//! Card #80, item 09, measured rather than hoped. The declared length of an entry comes from
//! the archive's index as a raw u32, and a corrupted one can claim four gigabytes. Reserving it
//! up front is an allocation that, where the allocator refuses, aborts the process — and on a
//! machine that commits memory lazily, like this one, it succeeds and hides. So this binary
//! counts: its allocator records the largest single request, and a decompressor handed a tiny
//! input and a four-gigabyte claim must never ask for more than the input could produce.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Largest;

static LARGEST: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for Largest {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LARGEST.fetch_max(layout.size(), Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Largest = Largest;

const CLAIMED: usize = u32::MAX as usize;
/// Far above anything the few bytes below could decode to, far below the claim.
const BOUND: usize = 1 << 20;

fn largest_request_during(f: impl FnOnce()) -> usize {
    LARGEST.store(0, Ordering::Relaxed);
    f();
    LARGEST.load(Ordering::Relaxed)
}

// One test in this binary: the counter is global, and tests in one binary run in parallel.
#[test]
fn a_four_gigabyte_claim_on_a_tiny_input_is_never_reserved() {
    let tiny = [0u8, 0, 0, 0, 0xAA, 0xBB, 0, 0, 0, 0, 0, 0];
    let lzw = largest_request_during(|| {
        let _ = unpack::for_tests::lzw_decompress(&tiny, 0, CLAIMED);
    });
    let miniz = largest_request_during(|| {
        let _ = unpack::for_tests::miniz_decompress(&tiny, 0, CLAIMED, 7);
    });
    let bogocrypt = largest_request_during(|| {
        let _ = unpack::for_tests::bogocrypt_decompress(&tiny, 0, CLAIMED, 7);
    });
    assert!(lzw < BOUND, "lzw asked for {lzw} bytes");
    assert!(miniz < BOUND, "miniz asked for {miniz} bytes");
    assert!(bogocrypt < BOUND, "bogocrypt asked for {bogocrypt} bytes");
}
