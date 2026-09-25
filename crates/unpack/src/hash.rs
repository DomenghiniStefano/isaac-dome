//! Path hash (djb2 + FNV) on the normalized form.

use serde::Serialize;

/// The two hashes that identify a file in the archive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
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

    let djb2 = normalized
        .iter()
        .fold(5381u32, |h, &b| h.wrapping_mul(33).wrapping_add(b as u32));
    let fnv = normalized.iter().fold(0x5BB2_220Eu32, |h, &b| {
        (h ^ b as u32).wrapping_mul(0x0100_0193)
    });

    PathKey { djb2, fnv }
}
