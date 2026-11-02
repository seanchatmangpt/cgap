//! Vertical bit-plane seen-set: fixed-capacity membership over u64 pairs.
//!
//! The closure cycle-breaker `seen(x, z)`: answers "has this pair been
//! visited?" with MSB-first vertical bit-planes instead of a hash table.
//!
//! Vendor provenance: the plane-layout discipline is vendored from
//! `crates/cgap-parquet/src/bitslice.rs` — the vertical packing contract
//! (bitslice.rs:1-33) and the MSB-first per-level cascade
//! (`scan_range_vertical_counted`, bitslice.rs:175: "MSB word first;
//! `level` is the actual bit index this word carries"). Plane 0 here is the
//! MSB of the 128-bit key, exactly as the scanner treats the last word of a
//! group as the MSB word.
//!
//! Host crate choice (lane contract asks to choose and disclose):
//! **cgap-storage** — the seen-set is the cycle-breaker for the closure
//! walk driven by the WAL/txn state this crate owns; `dred.rs` and
//! `atomic_commit.rs` are `core::`-only and this module follows the same
//! no_std-ready pattern, so the zero-external-dep property is preserved
//! (the crate's only dep remains cgap-blake3, which this module never
//! touches).

/// Key width: a u64 pair packed as `(x: u128) << 64 | z`.
pub const KEY_BITS: usize = 128;

/// One plane per key bit; plane 0 is the MSB (bitslice.rs:29 discipline).
pub const PLANE_COUNT: usize = KEY_BITS;

/// Typed refusal when the fixed capacity is exhausted (exit-contract
/// vocabulary: a loud refusal, never a silent drop).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeenFull;

/// Fixed-capacity seen-set over `(u64, u64)` pairs, stored MSB-first as
/// vertical bit-planes: `planes[p]` holds bit `127 - p` of the key at each
/// occupied slot. Membership reconstructs keys plane-by-plane.
pub struct BitPlaneSeenSet<const SLOTS: usize> {
    len: usize,
    planes: [[u64; SLOTS]; PLANE_COUNT],
}

impl<const SLOTS: usize> BitPlaneSeenSet<SLOTS> {
    pub const fn new() -> Self {
        Self {
            len: 0,
            planes: [[0; SLOTS]; PLANE_COUNT],
        }
    }

    /// Heap-lease constructor: allocates zeroed storage directly on the
    /// heap (alloc_zeroed), never staging the SLOTS*PLANE_COUNT-byte value
    /// on the caller's stack — debug builds stack-stage large inline values
    /// before the move into `Box::new(Self::new())`, so the box must happen
    /// at the allocation itself, not at the call site.
    pub fn new_boxed() -> Box<Self> {
        assert!(core::mem::size_of::<Self>() > 0);
        unsafe {
            let ptr = std::alloc::alloc_zeroed(core::alloc::Layout::new::<Self>()) as *mut Self;
            assert!(!ptr.is_null(), "bitplane seen-set alloc failed");
            Box::from_raw(ptr)
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn capacity() -> usize {
        SLOTS
    }

    pub const fn is_full(&self) -> bool {
        self.len == SLOTS
    }

    /// Insert `(x, z)`. Returns `Ok(true)` if newly inserted, `Ok(false)`
    /// if already present, `Err(SeenFull)` when capacity is exhausted and
    /// the pair is absent.
    pub fn insert(&mut self, x: u64, z: u64) -> Result<bool, SeenFull> {
        if self.contains(x, z) {
            return Ok(false);
        }
        if self.is_full() {
            return Err(SeenFull);
        }
        let slot = self.len;
        let key = ((x as u128) << 64) | z as u128;
        for p in 0..PLANE_COUNT {
            if (key >> (127 - p)) & 1 == 1 {
                self.planes[p][slot] = 1;
            }
        }
        self.len += 1;
        Ok(true)
    }

    /// Membership: reconstruct each occupied slot's 128-bit key plane-wise
    /// and compare against the probe key. Never mutates.
    pub fn contains(&self, x: u64, z: u64) -> bool {
        let key = ((x as u128) << 64) | z as u128;
        for slot in 0..self.len {
            let mut probe: u128 = 0;
            for p in 0..PLANE_COUNT {
                probe |= (self.planes[p][slot] as u128) << (127 - p);
            }
            if probe == key {
                return true;
            }
        }
        false
    }

    /// Court-only accessor: read one plane word (bit 0/1) for audit and
    /// mutation courts. Public because courts live in `tests/`.
    pub fn plane_bit(&self, plane: usize, slot: usize) -> u64 {
        self.planes[plane][slot]
    }

    /// Court-only mutator: set one plane bit. Exists so the anti-vacuity
    /// mutation court can corrupt exactly one plane bit and prove
    /// `contains()` diverges; not a general write path.
    pub fn set_plane_bit(&mut self, plane: usize, slot: usize, bit: u64) {
        self.planes[plane][slot] = bit & 1;
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.planes = [[0; SLOTS]; PLANE_COUNT]
    }
}
