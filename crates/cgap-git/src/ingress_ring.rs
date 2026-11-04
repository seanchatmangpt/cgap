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

// court-consumed surface (CEN2 ALLOW-WITH-REASON): ingress_ring_court / posh_wire_court / posh_alloc_sentry_court / posh_refusal_matrix_court / breach_regression_court / soc_full_chip_court / quota_alloc_purity_court / andon_liveness_court; wire consumers pending — module-level allow (uniform surface, not scattered attrs).
#![allow(dead_code)]

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

/// Typed refusal-quota seam (FIX-04, red-14 hardening): class-101
/// refusals are trip-able by any caller at zero cost, so a sustained
/// refusal FLOOD escalates to this REPORTABLE, RECOVERABLE typed state
/// instead of being invisible. The quota is OBSERVATIONAL ONLY — it never
/// gates admission (a garbage flood cannot wedge the ring), carries no
/// process-exit semantics, and gives a future host that maps
/// `andon_exit_class` to `process::exit` a rate-limit signal to front it
/// with (red-14 §2 recommendation). Recovery is a successful admission
/// resetting the sliding window; the lifetime total is never reset.
///
/// ADV-12 repair: the escalation is LEVEL-TRIGGERED, not edge-triggered.
/// `tripped` is sticky — one admitted frame does NOT erase it. It clears
/// only after (a) an explicit [`RefusalQuota::acknowledge`], or (b) a
/// sustained quiet window of `CLEAN_ADMISSIONS_TO_CLEAR` consecutive
/// clean admissions. `window_refusals` saturates at `u32::MAX` instead
/// of overflowing (a debug-build panic / release wrap under a
/// zero-admission flood).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefusalQuota {
    /// Refusals tolerated in the current window before `tripped`.
    pub max_refusals: u32,
    /// Refusals since the last successful admission (window). Saturates
    /// at `u32::MAX` — typed saturation, never a wrap or panic.
    pub window_refusals: u32,
    /// Lifetime refusals observed (never reset — audit counter).
    pub total_refusals: u64,
    /// Escalated state: the window hit `max_refusals`. Sticky until
    /// acknowledged or `CLEAN_ADMISSIONS_TO_CLEAR` clean admissions.
    pub tripped: bool,
    /// Consecutive clean admissions since the last refusal (private:
    /// drives the level-triggered auto-clear).
    clean_streak: u32,
}

impl RefusalQuota {
    /// Default quota: 4,096 sustained refusals in one window trip the
    /// escalation (far above any legitimate retry burst, far below a
    /// red-14-class 1,000,000-frame flood).
    pub const DEFAULT_MAX_REFUSALS: u32 = 4_096;

    /// Level-triggered auto-clear threshold (ADV-12 repair): after a
    /// trip, `tripped` clears only after this many CONSECUTIVE clean
    /// admissions — a named const, never a single-frame edge.
    pub const CLEAN_ADMISSIONS_TO_CLEAR: u32 = 1024;

    pub const fn new() -> Self {
        RefusalQuota {
            max_refusals: Self::DEFAULT_MAX_REFUSALS,
            window_refusals: 0,
            total_refusals: 0,
            tripped: false,
            clean_streak: 0,
        }
    }

    /// Record one class-101 refusal. Escalates (sets `tripped`) exactly
    /// when the window count reaches `max_refusals`. The window count
    /// saturates at `u32::MAX` (no overflow panic in debug, no silent
    /// wrap in release).
    pub fn note_refusal(&mut self) {
        self.window_refusals = self.window_refusals.saturating_add(1);
        self.total_refusals = self.total_refusals.wrapping_add(1);
        self.clean_streak = 0;
        if self.window_refusals >= self.max_refusals {
            self.tripped = true;
        }
    }

    /// Record one successful admission: the window resets (recoverable)
    /// and the clean streak grows. The escalated state is LEVEL-TRIGGERED:
    /// `tripped` stays set through a single admission and clears only
    /// after `CLEAN_ADMISSIONS_TO_CLEAR` consecutive clean admissions.
    /// The lifetime total is preserved for audit.
    pub fn note_admission(&mut self) {
        self.window_refusals = 0;
        if self.tripped {
            self.clean_streak = self.clean_streak.saturating_add(1);
            if self.clean_streak >= Self::CLEAN_ADMISSIONS_TO_CLEAR {
                self.tripped = false;
                self.clean_streak = 0;
            }
        }
    }

    /// Explicit host acknowledgement: clear the escalation NOW (the
    /// level-triggered manual path — a host that has observed and acted
    /// on the flood may drop the signal immediately).
    pub fn acknowledge(&mut self) {
        self.tripped = false;
        self.window_refusals = 0;
        self.clean_streak = 0;
    }

    /// `true` while the quota is escalated (reportable flood state).
    /// Sticky: survives individual admitted frames until acknowledged or
    /// a sustained quiet window.
    pub fn tripped(&self) -> bool {
        self.tripped
    }
}

impl Default for RefusalQuota {
    fn default() -> Self {
        Self::new()
    }
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
    quota: RefusalQuota,
}

impl<const CAP: usize> IngressRing<CAP> {
    /// Fresh empty ring. `const` so it can live in statics (no heap, no
    /// runtime init).
    pub const fn new() -> Self {
        IngressRing {
            buf: [0u8; CAP],
            head: PaddedIndex::new(0),
            tail: PaddedIndex::new(0),
            quota: RefusalQuota::new(),
        }
    }

    /// Reportable refusal-quota state (FIX-04): observability seam for
    /// sustained class-101 floods; never gates admission.
    pub fn refusal_quota(&self) -> &RefusalQuota {
        &self.quota
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
            self.quota.note_refusal();
            return Err(IngressError::FrameTooLarge {
                frame_len: frame.len(),
            });
        }
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        if would_overflow(head, tail, frame.len(), CAP) {
            self.quota.note_refusal();
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
        self.quota.note_admission();
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
            self.quota.note_refusal();
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
