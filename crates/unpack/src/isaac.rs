//! ISAAC stream cipher, used by archives in `MiniZ` mode for "stored" blocks.
//!
//! Ported from Gibbed.Rebirth (Rick Gibbed, zlib license), `ISAAC.cs`.
//! The arithmetic in C# is `unchecked`: here every sum is `wrapping_*`, and the shifts
//! reproduce the distinction between `>>` on `int` (arithmetic) and on `uint` (logical).
//!
//! The port has no dedicated tests with known vectors: it's verified **end-to-end**,
//! because a single wrong bit in the stream makes `MiniZ`'s "stored" blocks unreadable.
//! The tests in `tests/compression_modes.rs` extract real PNGs and check their signature
//! and trailer: if the cipher were wrong, nothing recognizable would come back.

const LOG_SIZE: usize = 8;
const SIZE: usize = 1 << LOG_SIZE; // 256
const MASK: i32 = ((SIZE - 1) << 2) as i32; // 1020

pub(crate) struct Isaac {
    results: [i32; SIZE],
    state: [i32; SIZE],
    index: usize,
    a: i32,
    b: i32,
    c: i32,
}

/// `(int)((uint)v >> n)` — logical shift, not arithmetic.
#[inline]
fn lsr(v: i32, n: u32) -> i32 {
    ((v as u32) >> n) as i32
}

impl Isaac {
    /// Constructs from the 256-int seed, as `new ISAAC(int[] seed)`.
    ///
    /// The original also has an unseeded path (`Initialize(false)`): not needed here,
    /// because the seed always comes from the entry's name hash, so it isn't implemented.
    pub(crate) fn new(seed: [i32; SIZE]) -> Self {
        let mut s = Isaac {
            results: seed,
            state: [0i32; SIZE],
            index: 0,
            a: 0,
            b: 0,
            c: 0,
        };
        s.initialize();
        s
    }

    /// Derives the seed from the name hash, as `ArchiveEntry.GetISAAC()`.
    pub(crate) fn from_name_hash(name_hash_b: u32) -> Self {
        let mut seed = name_hash_b as u64;
        // The truncation to 32 bits is in the original: `(uint)(...)`.
        let mixed = (seed ^ ((seed ^ (seed << 15)) << 8) ^ (seed >> 9)) as u32;
        seed = (seed << 32) | mixed as u64;

        let mut data = [0i32; SIZE];
        for slot in data.iter_mut() {
            let part = ((seed >> 27) ^ (seed >> 45)) as u32;
            let shift = (seed >> 59) as u32; // 0..=31
            *slot = part.rotate_right(shift) as i32;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(127);
        }
        Isaac::new(data)
    }

    /// Next 32-bit value of the stream, as `Value()`.
    pub(crate) fn value(&mut self) -> i32 {
        let result = self.results[self.index];
        let next = self.index + 1;
        if next >= SIZE {
            self.generate();
            self.index = 0;
        } else {
            self.index = next;
        }
        result
    }

    #[inline]
    fn at(&self, v: i32) -> i32 {
        // `_State[(v & Mask) >> 2]`: the mask is positive, so the index stays within 0..256.
        self.state[((v & MASK) >> 2) as usize]
    }

    fn generate(&mut self) {
        self.c = self.c.wrapping_add(1);
        self.b = self.b.wrapping_add(self.c);

        // The body is the same in both half-passes: only where `j` starts changes.
        let half = SIZE / 2;
        let mut i = 0usize;
        let mut j = half;
        for _ in 0..2 {
            let stop = i + half;
            while i < stop {
                for step in 0..4 {
                    let x = self.state[i];
                    self.a = match step {
                        0 => self.a ^ (self.a << 13),
                        1 => self.a ^ lsr(self.a, 6),
                        2 => self.a ^ (self.a << 2),
                        _ => self.a ^ lsr(self.a, 16),
                    };
                    self.a = self.a.wrapping_add(self.state[j]);
                    j += 1;
                    let y = self.at(x).wrapping_add(self.a).wrapping_add(self.b);
                    self.state[i] = y;
                    // `y >> LogSize` on `int` is arithmetic: sign-extending.
                    self.b = self.at(y >> LOG_SIZE).wrapping_add(x);
                    self.results[i] = self.b;
                    i += 1;
                }
            }
            j = 0;
        }
    }

    fn initialize(&mut self) {
        let mut v = [0x9E37_79B9u32 as i32; 8];

        // Four rounds of mixing on nothing.
        for _ in 0..4 {
            mix(&mut v);
        }

        // First pass: absorbs the seed into `_Results`.
        let mut i = 0usize;
        while i < SIZE {
            for (acc, seed) in v.iter_mut().zip(&self.results[i..i + 8]) {
                *acc = acc.wrapping_add(*seed);
            }
            mix(&mut v);
            self.state[i..i + 8].copy_from_slice(&v);
            i += 8;
        }

        // Second pass: re-mixes over the state just written.
        let mut i = 0usize;
        while i < SIZE {
            for (acc, cell) in v.iter_mut().zip(&self.state[i..i + 8]) {
                *acc = acc.wrapping_add(*cell);
            }
            mix(&mut v);
            self.state[i..i + 8].copy_from_slice(&v);
            i += 8;
        }

        self.generate();
    }
}

/// ISAAC's mixing step over eight accumulators.
fn mix(v: &mut [i32; 8]) {
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = *v;
    a ^= b << 11;
    d = d.wrapping_add(a);
    b = b.wrapping_add(c);
    b ^= lsr(c, 2);
    e = e.wrapping_add(b);
    c = c.wrapping_add(d);
    c ^= d << 8;
    f = f.wrapping_add(c);
    d = d.wrapping_add(e);
    d ^= lsr(e, 16);
    g = g.wrapping_add(d);
    e = e.wrapping_add(f);
    e ^= f << 10;
    h = h.wrapping_add(e);
    f = f.wrapping_add(g);
    f ^= lsr(g, 4);
    a = a.wrapping_add(f);
    g = g.wrapping_add(h);
    g ^= h << 8;
    b = b.wrapping_add(g);
    h = h.wrapping_add(a);
    h ^= lsr(a, 9);
    c = c.wrapping_add(h);
    a = a.wrapping_add(b);
    *v = [a, b, c, d, e, f, g, h];
}
