//! Lane GIT-03: 32,768-byte circular ingress ring (pack-ingress staging).
//!
//! House-pattern reuse (stated per dispatch): index discipline, cache-line
//! padding, and the Acquire/Release publish law are reused from
//! `crates/cgap-core/src/dma_ring.rs` (`CachePaddedIndex`, free-running
//! wrapping counters, `&mut self` single-handle SPSC with atomics retained
//! for peer observability). The never-silent-wrap typed-refusal law is
//! reused from `cgap-parquet/src/mmap_writer.rs` (`RingSink` rotation
//! refusal). The SWAR penstock sieve is RE-IMPLEMENTED in-module from the
//! documented mask contract of `cgap-branchless/src/swar.rs` — NOT
//! `#[path]`-imported: `cgap-git` is a zero-external-dep published kernel
//! with no dependency on `cgap-branchless`, and compiling foreign source
//! into it via `#[path]` would fake a dependency edge the manifest does
//! not declare. The mask functions below mirror that file's bit-level
//! contract byte for byte (same `HI` constant, same borrow-free
//! nonzero-per-byte detect; the `LO` mask is unused by the range
//! predicate and is therefore not replicated).
//!
//! Shape: one producer, one consumer, one handle; `&mut self` enforces
//! single-accessor by construction (dma_ring law). Capacity is fully
//! usable (free-running counters distinguish full from empty by distance,
//! so no slot is sacrificed). Zero heap: the buffer is an inline
//! `[u8; CAP]`. `#![forbid(unsafe_code)]`.

#![forbid(unsafe_code)]

use core::sync::atomic::{AtomicUsize, Ordering};

/// Standard ring capacity (dispatch): 32,768 bytes.
pub const INGRESS_RING_BYTES: usize = 32_768;

/// Cache-line width for index padding (dma_ring §7.2 reuse).
pub const CACHE_LINE_BYTES: usize = 64;

/// SWAR high-bit mask per byte lane (house constant, cgap-branchless/src/swar.rs).
const HI: u64 = 0x8080_8080_8080_8080;

/// Per-byte unsigned `x < y`: bit 7 of each byte set where `x`'s byte is
/// strictly less than `y`'s. Mirrors `cgap-branchless/src/swar.rs:swar_lt`.
#[inline(always)]
const fn swar_lt(x: u64, y: u64) -> u64 {
    let low_ge = (x | HI).wrapping_sub(y & !HI) & HI;
    let high_gt = x & !y & HI;
    let high_eq = !(x ^ y) & HI;
    let ge = high_gt | (high_eq & low_ge);
    !ge & HI
}

/// Broadcast `b` to all eight byte lanes.
#[inline(always)]
const fn swar_splat(b: u8) -> u64 {
    u64::from_ne_bytes([b; 8])
}

/// Per-byte inclusive range mask: `0xFF` where `min <= byte <= max`.
/// Mirrors `cgap-branchless/src/swar.rs:swar_range_mask`.
#[inline(always)]
const fn swar_range_mask(val: u64, min: u8, max: u8) -> u64 {
    let lt_min = swar_lt(val, swar_splat(min));
    let gt_max = swar_lt(swar_splat(max), val);
    let inside = !(lt_min | gt_max) & HI;
    (inside >> 7).wrapping_mul(0xFF)
}

/// `true` iff any byte of the 8-byte word lies in `[min, max]` (unsigned).
/// This is the penstock sieve predicate: a frame is ADMITTED when at least
/// one of its bytes falls in the tag band. Branch-free per word.
#[inline(always)]
const fn word_hits_band(word: u64, min: u8, max: u8) -> bool {
    swar_range_mask(word, min, max) != 0
}

/// Typed ingress refusals. No variant is ever a silent no-op (RingSink law).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IngressError {
    /// Push refused: advancing head by `frame_len` would put head more than
    /// `CAP` bytes past tail — the writer would overwrite unconsumed bytes.
    /// Ring state is untouched on refusal (fail-closed).
    Overflow {
        /// Free-running producer counter at refusal.
        head: usize,
        /// Free-running consumer counter at refusal.
        tail: usize,
        /// Length of the refused frame.
        frame_len: usize,
    },
    /// Frame larger than the whole ring; unrepresentable at any tail.
    FrameTooLarge {
        /// Length of the refused frame.
        frame_len: usize,
    },
    /// Pop with no bytes pending (typed, never garbage).
    Empty,
}

impl IngressError {
    /// Andon refusal class (dispatch: overflow maps to the
    /// AndonRefusal-class exit-101 family — deterministic and caught by the
    /// caller as a typed `Err`, NOT a real panic abort). All ingress
    /// refusals are class 101; the court pins this mapping.
    pub fn andon_exit_class(&self) -> u32 {
        match self {
            IngressError::Overflow { .. } => 101,
            IngressError::FrameTooLarge { .. } => 101,
            IngressError::Empty => 101,
        }
    }
}

/// Capacity check as a pure named function so the mutation court can pin
/// exactly the guard whose deletion the flip twin targets.
#[inline(always)]
const fn would_overflow(head: usize, tail: usize, frame_len: usize, cap: usize) -> bool {
    head.wrapping_sub(tail) + frame_len > cap
}

/// Cache-line-isolated atomic index (dma_ring §7.2 reuse). Size pinned by
/// const assert below; a padding regression to 32 B fails to compile.
#[repr(align(64))]
struct PaddedIndex(AtomicUsize);

impl PaddedIndex {
    const fn new(v: usize) -> Self {
        PaddedIndex(AtomicUsize::new(v))
    }
    fn load(&self, ord: Ordering) -> usize {
        self.0.load(ord)
    }
    fn store(&self, v: usize, ord: Ordering) {
        self.0.store(v, ord)
    }
}

const _: () = assert!(core::mem::size_of::<PaddedIndex>() == CACHE_LINE_BYTES);
const _: () = assert!(core::mem::align_of::<PaddedIndex>() == CACHE_LINE_BYTES);

/// SPSC circular byte ring over an inline fixed buffer. One handle;
/// `&mut self` methods enforce single-accessor (dma_ring law) while the
/// head/tail atomics keep publish progress observable to a peer core.
///
/// Counters are free-running `usize`; live bytes are
/// `head.wrapping_sub(tail)` in `[0, CAP]` — the ring is FULL exactly when
/// the distance equals `CAP`, so all CAP bytes are usable. Bytes are copied
/// (never aliased) across the wrap boundary, keeping the module
/// `#![forbid(unsafe_code)]`-clean with zero heap.
pub struct IngressRing<const CAP: usize> {
    buf: [u8; CAP],
    head: PaddedIndex,
    tail: PaddedIndex,
}

impl<const CAP: usize> IngressRing<CAP> {
    /// Fresh empty ring. `const` so it can live in statics (no heap, no
    /// runtime init).
    pub const fn new() -> Self {
        IngressRing {
            buf: [0u8; CAP],
            head: PaddedIndex::new(0),
            tail: PaddedIndex::new(0),
        }
    }

    /// Free-running producer counter (monotonic, wraps at usize).
    pub fn head(&self) -> usize {
        self.head.load(Ordering::Acquire)
    }

    /// Free-running consumer counter.
    pub fn tail(&self) -> usize {
        self.tail.load(Ordering::Acquire)
    }

    /// Live bytes pending (head − tail distance), in `[0, CAP]`.
    pub fn pending(&self) -> usize {
        self.head
            .load(Ordering::Acquire)
            .wrapping_sub(self.tail.load(Ordering::Acquire))
    }

    /// Ring capacity in bytes (== [`INGRESS_RING_BYTES`] for the standard
    /// instantiation).
    pub fn capacity(&self) -> usize {
        CAP
    }

    /// Push `frame` bytes into the ring. Copies in at most two segments
    /// (pre-wrap, post-wrap) so a frame crossing the boundary is
    /// byte-exact. On [`IngressError::Overflow`] the head is NOT advanced
    /// and no bytes are written — refusal leaves the ring untouched.
    pub fn push(&mut self, frame: &[u8]) -> Result<(), IngressError> {
        if frame.len() > CAP {
            return Err(IngressError::FrameTooLarge {
                frame_len: frame.len(),
            });
        }
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        if would_overflow(head, tail, frame.len(), CAP) {
            return Err(IngressError::Overflow {
                head,
                tail,
                frame_len: frame.len(),
            });
        }
        let pos = head % CAP;
        let first = core::cmp::min(frame.len(), CAP - pos);
        self.buf[pos..pos + first].copy_from_slice(&frame[..first]);
        if frame.len() > first {
            self.buf[..frame.len() - first].copy_from_slice(&frame[first..]);
        }
        // Publish law (dma_ring §7.1 reuse): payload bytes are written
        // before the Release store makes the advance visible; observers
        // read the index with Acquire.
        self.head
            .store(head.wrapping_add(frame.len()), Ordering::Release);
        Ok(())
    }

    /// Penstock-sieved push (dispatch): ADMIT the frame only if some byte
    /// of it falls in the inclusive tag band `[min, max]`, decided by the
    /// branch-free SWAR range mask above. Returns `Ok(false)` when the
    /// sieve DROPS the frame (filtered — not an error, ring untouched),
    /// `Ok(true)` when admitted and pushed, `Err(_)` when a pushable frame
    /// hit a capacity refusal.
    pub fn push_sieved(&mut self, frame: &[u8], min: u8, max: u8) -> Result<bool, IngressError> {
        let mut i = 0usize;
        let mut hit = false;
        // 8-byte SWAR stride; tail bytes fall back to a scalar band test
        // with the same predicate (identity, not a second semantics).
        while i + 8 <= frame.len() {
            let mut w = [0u8; 8];
            w.copy_from_slice(&frame[i..i + 8]);
            if word_hits_band(u64::from_ne_bytes(w), min, max) {
                hit = true;
            }
            i += 8;
        }
        while i < frame.len() {
            if frame[i] >= min && frame[i] <= max {
                hit = true;
            }
            i += 1;
        }
        if !hit {
            return Ok(false);
        }
        self.push(frame)?;
        Ok(true)
    }

    /// Pop up to `out.len()` pending bytes into `out`, in FIFO order,
    /// copying across the wrap boundary. Returns the number of bytes moved;
    /// [`IngressError::Empty`] when nothing is pending (typed, never
    /// garbage — dma_ring §8.3 read law).
    pub fn pop(&mut self, out: &mut [u8]) -> Result<usize, IngressError> {
        let pending = self.pending();
        if pending == 0 {
            return Err(IngressError::Empty);
        }
        let n = core::cmp::min(out.len(), pending);
        let tail = self.tail.load(Ordering::Acquire);
        let pos = tail % CAP;
        let first = core::cmp::min(n, CAP - pos);
        out[..first].copy_from_slice(&self.buf[pos..pos + first]);
        if n > first {
            out[first..n].copy_from_slice(&self.buf[..n - first]);
        }
        self.tail.store(tail.wrapping_add(n), Ordering::Release);
        Ok(n)
    }
}

impl<const CAP: usize> Default for IngressRing<CAP> {
    fn default() -> Self {
        Self::new()
    }
}

/// Standard instantiation (dispatch shape): 32,768-byte inline ring.
pub type StdIngressRing = IngressRing<INGRESS_RING_BYTES>;
