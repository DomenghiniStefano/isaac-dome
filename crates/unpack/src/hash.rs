//! Path hash (djb2 + FNV) on the normalized form.

use serde::Serialize;

/// The two hashes that identify a file in the archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PathKey {
    pub djb2: u32,
    pub fnv: u32,
}

/// Computes djb2 and FNV on the normalized path: lowercase and `/` separators.
pub fn path_key(path: &str) -> PathKey {
    let normalized: Vec<u8> = path
        .chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| if c == '\\' { '/' } else { c })
        .collect::<String>()
        .into_bytes();

    let mut djb2: u32 = 5381;
    for &b in &normalized {
        djb2 = djb2.wrapping_mul(33).wrapping_add(b as u32);
    }

    let mut fnv: u32 = 0x5BB2_220E;
    for &b in &normalized {
        fnv = (fnv ^ b as u32).wrapping_mul(0x0100_0193);
    }

    PathKey { djb2, fnv }
}
