//! Throwaway: what sections the file actually holds, header by header.
//!
//! The game's own log names eleven chunks; our table has ten and the parser reports no
//! diagnostic. One of the two accounts is wrong, and only the bytes settle it.

use core_save::Save;

fn u32_at(b: &[u8], off: usize) -> Option<u32> {
    b.get(off..off + 4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

fn main() {
    let path = std::env::args().nth(1).expect("give me a .dat");
    let bytes = std::fs::read(&path).expect("readable");
    let save = Save::parse(&bytes).expect("parses");
    println!("file: {} bytes", bytes.len());
    println!("diagnostics: {:?}", save.diagnostics);
    let mut last = 0usize;
    for s in &save.sections {
        println!(
            "kind {:?} (n={}) count={} f2={} data={} bytes at {}",
            s.kind,
            s.kind.number(),
            s.count,
            s.f2,
            s.bytes.len(),
            s.offset
        );
        last = s.offset;
    }
    // Walk on from where the last section's *declared* size ends, reading headers as three
    // little-endian u32s, exactly as the format does everywhere else.
    let mut off = last + 320;
    if std::env::args().any(|a| a == "--scan") {
        println!(
            "
--- scanning the tail for a plausible header ---"
        );
        let tail = bytes.len().saturating_sub(4);
        for p in last..tail.saturating_sub(12) {
            let (Some(k), Some(f2), Some(c)) = (
                u32_at(&bytes, p),
                u32_at(&bytes, p + 4),
                u32_at(&bytes, p + 8),
            ) else {
                break;
            };
            if (1..=20).contains(&k) && c > 0 && c < 100_000 && f2 == c * 4 {
                println!(
                    "at {p} (= section 10 start + {}): kind={k} f2={f2} count={c}",
                    p - last
                );
            }
        }
        return;
    }
    let end = bytes.len().saturating_sub(4);
    println!("\n--- walking on from {off} (end of section 10's declared 320 bytes) ---");
    while off + 12 <= end {
        let (k, f2, count) = (
            u32_at(&bytes, off),
            u32_at(&bytes, off + 4),
            u32_at(&bytes, off + 8),
        );
        let (Some(k), Some(f2), Some(count)) = (k, f2, count) else {
            break;
        };
        println!(
            "at {off}: kind={k} f2={f2} count={count}  (remaining {})",
            end - off
        );
        if count == 0 || count > 100_000 {
            break;
        }
        let per = if f2 == count * 4 { 4 } else { 1 };
        off += 12 + (count as usize) * per;
    }
}

// Scan helper kept separate: called from main via `--scan`.
