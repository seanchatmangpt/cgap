//! Caller-side AoQ event tap (PL16 caller-side law, v26.11.3).
//!
//! PL16's law: the tap is CALLER-SIDE — never inside `session.rs`/`step`.
//! Purity law (session.rs law 4: no mutation on refusal) is preserved by
//! construction: refusals emit NOTHING. The caller only calls [`AoqEventTap::tap`]
//! after a `step(&frame)` returns `Ok(())`; on `Err` the caller simply does not
//! tap. This module never touches the session (read-only accessors only) and
//! never mutates it.
//!
//! [`fingerprint()`] is already the 32-byte event payload — the row carries it
//! verbatim (no re-hash), so the OCEL consumer re-derives the chain-ordinal law
//! downstream without a second digest.
//!
//! Wire/layout laws:
//! - Ring shape mirrors `cgap-beam/src/spsc_ring.rs`: power-of-two capacity,
//!   monotonic 64-bit head counter with `& (CAP - 1)` slot masking,
//!   64-byte-aligned rows (false-sharing fence).
//! - Overwrite-on-full with a loud dropped-count accessor: an honest ring
//!   never fakes capacity. `dropped() > 0` means the consumer is losing
//!   events; the conformance court asserts the counter is loud, not silent.
//! - Reserved-zero law: the 20 reserved bytes are always zero on write.
//!
//! Downstream consumer (F5's finding, NOT integrated this lane — task 4):
//! `ggen audit conformance` + `ggen-graph/src/ocel` lifecycle machinery is
//! the designated downstream consumer; [`AoqEventRow`] is its input contract.
//! The `to_ocel_rows()` projection onto cgap-ocel2's event shape is likewise
//! deferred downstream: cgap-ffi does not depend on cgap-ocel2 (keeping the
//! tap dep-free; the projection is a consumer-side adapter over the same
//! rows — ocel2's `event_id = chain-ordinal` law maps 1:1 onto
//! [`AoqEventRow::ordinal`]).
//!
//! [`fingerprint()`]: cgap_aoq::session::Session::fingerprint
//!
//! # A note on concurrency
//! The tap is single-producer by law (one caller loop per session). A reader
//! on another thread may observe a torn `head`-vs-slots window exactly like
//! an SPSC ring's consumer side; the conformance court drives the tap
//! single-threaded (the caller-side law makes this the only lawful shape).

use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicU64, Ordering};

use cgap_aoq::session::{FrameKind, KemSeam, Session};

/// Power-of-two capacity law (compile-time assert in [`AoqEventTap::new`]).
pub const DEFAULT_TAP_CAPACITY: usize = 1024;

/// Row byte width — the frozen wire layout (64 bytes, 64-byte aligned).
pub const AOQ_EVENT_ROW_BYTES: usize = 64;

/// One tapped AoQ session event: the caller-side observation of one accepted
/// `step`, projected onto the frozen 64-byte wire row.
#[repr(C, align(64))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AoqEventRow {
    /// Session frame ordinal after the accepted step (wrapping u32).
    pub ordinal: u32,
    /// `SessionState::id()` AFTER the accepted step (the to-state).
    pub state: u8,
    /// `FrameKind::id()` of the accepted frame.
    pub kind: u8,
    /// Active epoch (RETIRED can never appear: retired frames are refused
    /// by construction and refusals emit nothing).
    pub epoch: u16,
    /// Counted rekey budget used after the accepted step.
    pub budget: u32,
    /// `Session::fingerprint()` after the accepted step — verbatim payload,
    /// no re-hash (PL16: fingerprint IS the event payload).
    pub fingerprint: [u8; 32],
    /// Reserved-zero law: always zero on write; consumers must refuse
    /// nonzero reserved bytes (forward-compat fence).
    pub reserved: [u8; 20],
}

// Layout pin: 4 + 1 + 1 + 2 + 4 + 32 + 20 = 64.
const _: () = assert!(core::mem::size_of::<AoqEventRow>() == AOQ_EVENT_ROW_BYTES);
const _: () = assert!(core::mem::align_of::<AoqEventRow>() == 64);

/// Mint-time event (what `AoqEventTap::tap` consumes). The caller captures
/// it AFTER a successful `step` via [`AoqEvent::mint`]; the tap assigns
/// nothing — every field is observed, never derived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AoqEvent {
    /// See [`AoqEventRow::ordinal`].
    pub ordinal: u32,
    /// See [`AoqEventRow::state`].
    pub state: u8,
    /// See [`AoqEventRow::kind`].
    pub kind: u8,
    /// See [`AoqEventRow::epoch`].
    pub epoch: u16,
    /// See [`AoqEventRow::budget`].
    pub budget: u32,
    /// See [`AoqEventRow::fingerprint`].
    pub fingerprint: [u8; 32],
}

impl AoqEvent {
    /// Mint an event from the post-step session snapshot plus the frame kind
    /// the caller just stepped with.
    ///
    /// CALLER-SIDE LAW: call this ONLY on the `Ok(())` arm of `step`. On
    /// `Err` build nothing — refusals emit nothing (purity + no-mutation-
    /// on-refusal law; the conformance court asserts the count law).
    #[must_use]
    pub fn mint<K: KemSeam>(session: &Session<'_, K>, kind: FrameKind) -> Self {
        Self {
            ordinal: session.ordinal(),
            state: session.state().id(),
            kind: kind.id(),
            epoch: session.epoch(),
            budget: session.budget_used(),
            fingerprint: session.fingerprint(),
        }
    }
}

/// Fixed-capacity, overwrite-on-full, append-only event ring (SPSC-style).
///
/// Mirrors `cgap-beam/src/spsc_ring.rs`'s shape: power-of-two capacity
/// (compile-time assert), monotonic 64-bit head with bitwise slot masking,
/// 64-byte-aligned [`AoqEventRow`] slots. Unlike a bounded queue, `tap`
/// NEVER fails: on full it overwrites the oldest row and increments the
/// loud [`AoqEventTap::dropped`] counter.
pub struct AoqEventTap<const CAP: usize = DEFAULT_TAP_CAPACITY> {
    /// Monotonic total-tapped counter; `head & MASK` is the write slot.
    head: AtomicU64,
    /// Oldest retained sequence; `head - tail` is occupancy.
    tail: AtomicU64,
    slots: [UnsafeCell<MaybeUninit<AoqEventRow>>; CAP],
}

impl<const CAP: usize> AoqEventTap<CAP> {
    /// Construct an empty tap. Compile-time power-of-two capacity law.
    #[allow(clippy::new_without_default)]
    #[must_use]
    pub const fn new() -> Self {
        const {
            assert!(
                CAP.is_power_of_two(),
                "AoqEventTap capacity must be power of two"
            );
        }
        Self {
            head: AtomicU64::new(0),
            tail: AtomicU64::new(0),
            slots: [const { UnsafeCell::new(MaybeUninit::uninit()) }; CAP],
        }
    }

    /// Append one accepted-step event. Overwrite-on-full: never fails.
    ///
    /// CALLER-SIDE LAW (PL16): call only after `step` returned `Ok(())`.
    /// Refusals emit nothing — the caller simply does not call `tap` on the
    /// `Err` arm; nothing in this module compensates for a missed event
    /// because a refusal is not an event.
    pub fn tap(&self, record: &AoqEvent) {
        let row = AoqEventRow {
            ordinal: record.ordinal,
            state: record.state,
            kind: record.kind,
            epoch: record.epoch,
            budget: record.budget,
            fingerprint: record.fingerprint,
            reserved: [0u8; 20],
        };
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Acquire);
        if head.wrapping_sub(tail) >= CAP as u64 {
            // Ring full: overwrite the oldest and count it loudly.
            self.tail.store(tail.wrapping_add(1), Ordering::Release);
        }
        let slot = (head & (CAP as u64 - 1)) as usize;
        unsafe {
            (*self.slots[slot].get()).write(row);
        }
        self.head.store(head.wrapping_add(1), Ordering::Release);
    }

    /// Total events tapped (monotonic; wraps naturally at u64::MAX).
    #[must_use]
    pub fn total_tapped(&self) -> u64 {
        self.head.load(Ordering::Relaxed)
    }

    /// Events currently retained (<= CAP).
    #[must_use]
    pub fn len(&self) -> usize {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        head.wrapping_sub(tail) as usize
    }

    /// True when no events are retained.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Events LOST to overwrite-on-full since construction. Loud by law:
    /// nonzero means the consumer is behind, never silently dropped.
    #[must_use]
    pub fn dropped(&self) -> u64 {
        let head = self.head.load(Ordering::Relaxed);
        // total - retained = head - CAP (once the ring has ever been full).
        head.saturating_sub(CAP as u64)
    }

    /// Drain the retained rows in tap order into `out`; returns the copied
    /// count (<= `out.len()` and <= `len()`). Non-destructive: the ring is
    /// untouched (courts replay from a stable snapshot).
    #[must_use]
    pub fn drain(&self, out: &mut [AoqEventRow]) -> usize {
        let head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        let retained = (head.wrapping_sub(tail) as usize).min(CAP);
        let n = retained.min(out.len());
        for (i, dst) in out[..n].iter_mut().enumerate() {
            let seq = head - retained as u64 + i as u64;
            let slot = (seq & (CAP as u64 - 1)) as usize;
            // SAFETY: seq in [head-retained, head) was written by `tap`
            // before the Release store on head; single-producer law.
            *dst = unsafe { (*self.slots[slot].get()).assume_init() };
        }
        n
    }
}

// SAFETY: single-producer (caller-side law); rows are Copy; slot access is
// masked by the monotonic head/tail counters exactly like spsc_ring.rs.
unsafe impl<const CAP: usize> Sync for AoqEventTap<CAP> {}
