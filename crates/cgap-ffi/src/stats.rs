//! Typed runtime stats block (FIX-14 A1 / RED-16 A1 remediation, lane RC-05).
//!
//! ZERO runtime telemetry existed before this module (red-16 audit:
//! grep 0 matches for telemetry/metrics/health across cgap-ffi +
//! cgap-storage src trees). This module closes the detection seam:
//!
//! - `CgasStatsBlock` — fixed-layout `repr(C)` snapshot (CRC'd per
//!   fix-14 R-A1.2), counters for ingress frames admitted / refused
//!   by the six-row ingress taxonomy (fix-09 §3: INVALID_MAGIC rows
//!   1-2, OABI_MISMATCH row 3, P28_SPOLIATION rows 4-5, ADMITTED row
//!   6) plus ring-level class-101 refusals, current quota state,
//!   receipts minted, last-receipt digest.
//! - `StatsRecorder` — the opt-in writer the ring ingress calls.
//! - `Record` — the hook trait so the ring owner wires the ingress
//!   with ONE line per event site (no ring-file edit by this lane).
//! - `cgap_stats_snapshot` — `extern "C"` pull-based getter
//!   (fix-14 R-A1.3): the operator polls, the kernel never sleeps.
//!
//! Kernel single-writer law (fix-14 R-A1.1): the kernel is the ONLY
//! writer of the counters; snapshots are the readers. Counters are
//! written on REAL events, never on a clock. no_std-safe — no atomic
//! RMW is assumed (riscv32imc has no A extension; only load/store),
//! so the recorder uses a documented single-writer cell, not
//! `AtomicU64`.

/// ABI magic for the stats block: "CSTB" (Cgas-Stats-Block).
pub const CGAS_STATS_MAGIC: u32 = 0x4353_5442;
/// Stats-block layout version (bump on any layout change).
pub const CGAS_STATS_LAYOUT_VERSION: u32 = 1;
/// CRC coverage: bytes [0, CRC_AT) of `CgasStatsBlock` — header +
/// counters + quota + receipt digest. The CRC word and trailing pad
/// are excluded (a CRC covering itself is undefined).
const CRC_AT: usize = 112;

/// Typed poll verdict (fix-14 R-A1.4): readiness/health is a state
/// read off recorded events, never a clock expiry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Health {
    /// Quota never tripped, CRC intact.
    Healthy,
    /// Quota escalated (sustained class-101 flood) — reportable.
    QuotaTripped,
    /// Snapshot CRC mismatch — the block was corrupted or the caller
    /// hand-built a bogus block; refuse to interpret counters.
    CrcInvalid,
}

/// Six-row ingress taxonomy buckets (fix-09 §3) + ring-level class 101.
/// cgap-ffi cannot depend on cgap-git (no manifest edge; the manifest is
/// coordinator-owned), so the taxonomy is mirrored here as a lane-local
/// value type and the mapping is court-pinned:
/// Overflow/FrameTooLarge/Empty -> `Class101` (ingress_ring.rs:167-173).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefusalClass {
    /// rows 1-2: len<128 or magic/version mismatch.
    InvalidMagic,
    /// row 3: zero oabi digest or payload_byte_len over-run.
    OabiMismatch,
    /// rows 4-5: attestation spoliation (magic/version/lineage laws).
    P28Spoliation,
    /// ring-level refusals (Overflow/FrameTooLarge/Empty).
    Class101,
}

/// Value copy of `cgap_git::ingress_ring::RefusalQuota` state at record
/// time (mapping court-pinned; see `RefusalClass` note above).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuotaState {
    pub max_refusals: u32,
    pub window_refusals: u32,
    pub total_refusals: u64,
    pub tripped: bool,
}

/// Fixed-layout stats block exported across the C-ABI seam
/// (fix-14 R-A1.2: fixed-size, CRC'd). Offsets are pinned by the
/// offset court (`tests/stats_block_court.rs`).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CgasStatsBlock {
    /// offset 0..4   — `CGAS_STATS_MAGIC`.
    pub magic: u32,
    /// offset 4..8   — `CGAS_STATS_LAYOUT_VERSION.
    pub layout_version: u32,
    /// offset 8..16  — ingress frames ADMITTED (taxonomy row 6).
    pub ingress_admitted: u64,
    /// offset 16..24 — refused: rows 1-2 (len<128 / bad magic+version).
    pub refused_invalid_magic: u64,
    /// offset 24..32 — refused: row 3 (zero digest / payload over-run).
    pub refused_oabi_mismatch: u64,
    /// offset 32..40 — refused: rows 4-5 (attestation spoliation).
    pub refused_p28_spoliation: u64,
    /// offset 40..48 — ring-level class-101 refusals (Overflow /
    /// FrameTooLarge / Empty — all map to andon class 101).
    pub refusals_class_101: u64,
    /// offset 48..52 — quota ceiling (RefusalQuota::max_refusals).
    pub quota_max_refusals: u32,
    /// offset 52..56 — refusals since the last admission.
    pub quota_window_refusals: u32,
    /// offset 56..60 — 0/1: quota escalated (`tripped`).
    pub quota_tripped: u32,
    /// offset 60..64 — explicit pad (layout stability).
    pub _pad0: u32,
    /// offset 64..72 — lifetime refusals (audit counter, never reset
    /// by window recovery — mirrors RefusalQuota::total_refusals).
    pub quota_total_refusals: u64,
    /// offset 72..80 — receipts minted by the kernel.
    pub receipts_minted: u64,
    /// offset 80..112 — digest of the last minted receipt.
    pub last_receipt_digest: [u8; 32],
    /// offset 112..116 — FNV-1a-32 over bytes [0, CRC_AT).
    pub block_crc: u32,
    /// offset 116..120 — pad; size_of == 120, align_of == 8.
    pub _pad1: u32,
}

impl CgasStatsBlock {
    /// All-zero-counter block with header set and CRC sealed.
    pub const fn sealed() -> Self {
        let mut b = CgasStatsBlock {
            magic: CGAS_STATS_MAGIC,
            layout_version: CGAS_STATS_LAYOUT_VERSION,
            ingress_admitted: 0,
            refused_invalid_magic: 0,
            refused_oabi_mismatch: 0,
            refused_p28_spoliation: 0,
            refusals_class_101: 0,
            quota_max_refusals: 0,
            quota_window_refusals: 0,
            quota_tripped: 0,
            _pad0: 0,
            quota_total_refusals: 0,
            receipts_minted: 0,
            last_receipt_digest: [0u8; 32],
            block_crc: 0,
            _pad1: 0,
        };
        b.block_crc = b.fnv1a32();
        b
    }

    /// FNV-1a-32 over bytes [0, CRC_AT). Same fold family the sealer
    /// uses (cgap-parquet footer seal); NOT cryptographic — tamper
    /// EVIDENCE for the poller, matching fix-14's "CRC'd" bar.
    pub const fn fnv1a32(&self) -> u32 {
        let bytes = self.as_crc_bytes();
        let mut h: u32 = 0x811C_9DC5;
        let mut i = 0;
        while i < CRC_AT {
            h ^= bytes[i] as u32;
            h = h.wrapping_mul(0x0100_0193);
            i += 1;
        }
        h
    }

    /// Bytes [0, CRC_AT) as a fixed array view (CRC coverage only).
    #[inline]
    const fn as_crc_bytes(&self) -> &[u8; CRC_AT] {
        // SAFETY: repr(C) with fixed layout; CRC_AT bytes are exactly
        // the fields preceding block_crc. Every field boundary in the
        // covered range is naturally aligned, so padding bytes are
        // only the explicit _pad0 and they are always written 0.
        unsafe { &*core::ptr::from_ref(self).cast::<[u8; CRC_AT]>() }
    }
}

/// Single-writer u64 cell. `core`'s atomics on riscv32imc expose only
/// load/store (no A extension, no RMW), so the recorder uses a plain
/// cell under the kernel single-writer law (module doc): exactly one
/// kernel writer increments counters; snapshot readers may observe a
/// torn-free u64 on the aligned targets the crate ships (the XIP
/// link is rv32imc, where aligned u64 load/store is single
/// instruction). NOT for multi-writer host fan-out.
#[derive(Debug)]
pub struct Cell64(core::cell::Cell<u64>);

impl Cell64 {
    pub const fn new(v: u64) -> Self {
        Cell64(core::cell::Cell::new(v))
    }
    #[inline]
    pub fn get(&self) -> u64 {
        self.0.get()
    }
    #[inline]
    pub fn set(&self, v: u64) {
        self.0.set(v);
    }
    #[inline]
    pub fn inc(&self) {
        self.0.set(self.0.get().wrapping_add(1));
    }
}
// SAFETY: single-writer law (module doc) — the kernel is the only
// writer; readers take snapshots and tolerate any single snap-in-time
// view of the counters (same guarantee class as Relaxed atomics).
unsafe impl Sync for Cell64 {}

/// The opt-in recorder: kernel-side writer for every typed ingress /
/// quota / receipt event. Const-newable so a `static` works on host
/// and XIP with no runtime init and no allocation.
#[derive(Debug)]
pub struct StatsRecorder {
    magic: Cell64,
    layout_version: Cell64,
    ingress_admitted: Cell64,
    refused_invalid_magic: Cell64,
    refused_oabi_mismatch: Cell64,
    refused_p28_spoliation: Cell64,
    refusals_class_101: Cell64,
    quota_max_refusals: Cell64,
    quota_window_refusals: Cell64,
    quota_tripped: Cell64,
    quota_total_refusals: Cell64,
    receipts_minted: Cell64,
    /// Last receipt digest — 4 u64 words, little-endian order.
    last_digest: [Cell64; 4],
}

/// Module-level default recorder (zero-cost: a `static` of cells, no
/// allocation, no init call). Ring-owner integration — documented, NOT
/// wired by this lane (ingress_ring.rs/egress_ring.rs are another
/// lane's files):
///
/// ```text
/// // crates/cgap-git/src/ingress_ring.rs, in `push` on admission:
/// cgap_ffi::stats::global().record_admitted();
/// // on each typed refusal:
/// cgap_ffi::stats::global().record_refused_class(class);
/// // on quota transition (note_refusal / note_admission sites):
/// cgap_ffi::stats::global().record_quota(quota_state);
/// ```
static GLOBAL: StatsRecorder = StatsRecorder::const_new();

/// Accessor for the module-level default recorder.
#[inline]
pub fn global() -> &'static StatsRecorder {
    &GLOBAL
}

/// Event hooks for the ring ingress (fix-14 R-A1.1: written on real
/// events, not a clock). Implementors: [`StatsRecorder`] (canon) or
/// any caller-side fan-out recorder.
pub trait Record {
    fn record_admitted(&self);
    /// `class` is the six-row taxonomy bucket (+ `Class101` for
    /// ring-level Overflow/FrameTooLarge/Empty refusals).
    fn record_refused_class(&self, class: RefusalClass);
    fn record_quota(&self, quota: QuotaState);
    fn record_receipt(&self, digest: &[u8; 32]);
}

impl StatsRecorder {
    /// Const-newable (usable in a `static`).
    pub const fn const_new() -> Self {
        StatsRecorder {
            magic: Cell64::new(CGAS_STATS_MAGIC as u64),
            layout_version: Cell64::new(CGAS_STATS_LAYOUT_VERSION as u64),
            ingress_admitted: Cell64::new(0),
            refused_invalid_magic: Cell64::new(0),
            refused_oabi_mismatch: Cell64::new(0),
            refused_p28_spoliation: Cell64::new(0),
            refusals_class_101: Cell64::new(0),
            quota_max_refusals: Cell64::new(0),
            quota_window_refusals: Cell64::new(0),
            quota_tripped: Cell64::new(0),
            quota_total_refusals: Cell64::new(0),
            receipts_minted: Cell64::new(0),
            last_digest: [const { Cell64::new(0) }; 4],
        }
    }

    /// Taxonomy row 6: one frame admitted.
    #[inline]
    pub fn record_admitted(&self) {
        self.ingress_admitted.inc();
    }

    /// One refusal, bucketed by the six-row taxonomy (+ class 101).
    #[inline]
    pub fn record_refused_class(&self, class: RefusalClass) {
        let c = match class {
            RefusalClass::InvalidMagic => &self.refused_invalid_magic,
            RefusalClass::OabiMismatch => &self.refused_oabi_mismatch,
            RefusalClass::P28Spoliation => &self.refused_p28_spoliation,
            RefusalClass::Class101 => &self.refusals_class_101,
        };
        c.inc();
    }

    /// Publish current quota state (the stats block mirrors it; the
    /// quota itself lives in the ring — never gates admission).
    pub fn record_quota(&self, q: QuotaState) {
        self.quota_max_refusals.set(u64::from(q.max_refusals));
        self.quota_window_refusals.set(u64::from(q.window_refusals));
        self.quota_tripped.set(u64::from(q.tripped));
        self.quota_total_refusals.set(q.total_refusals);
    }

    /// One receipt minted; digest published (last-writer-wins).
    pub fn record_receipt(&self, digest: &[u8; 32]) {
        self.receipts_minted.inc();
        for (w, d) in self.last_digest.iter().zip(digest.chunks_exact(8)) {
            let mut b = [0u8; 8];
            b.copy_from_slice(d);
            w.set(u64::from_le_bytes(b));
        }
    }

    /// Pull-based snapshot (fix-14 R-A1.3): the operator poll, sealed
    /// with a fresh CRC over the live counter values.
    pub fn snapshot(&self) -> CgasStatsBlock {
        let mut b = CgasStatsBlock {
            magic: self.magic.get() as u32,
            layout_version: self.layout_version.get() as u32,
            ingress_admitted: self.ingress_admitted.get(),
            refused_invalid_magic: self.refused_invalid_magic.get(),
            refused_oabi_mismatch: self.refused_oabi_mismatch.get(),
            refused_p28_spoliation: self.refused_p28_spoliation.get(),
            refusals_class_101: self.refusals_class_101.get(),
            quota_max_refusals: self.quota_max_refusals.get() as u32,
            quota_window_refusals: self.quota_window_refusals.get() as u32,
            quota_tripped: self.quota_tripped.get() as u32,
            _pad0: 0,
            quota_total_refusals: self.quota_total_refusals.get(),
            receipts_minted: self.receipts_minted.get(),
            last_receipt_digest: [0u8; 32],
            block_crc: 0,
            _pad1: 0,
        };
        for (w, slot) in b
            .last_receipt_digest
            .chunks_exact_mut(8)
            .zip(&self.last_digest)
        {
            let v = slot.get().to_le_bytes();
            w.copy_from_slice(&v);
        }
        b.block_crc = b.fnv1a32();
        b
    }
}

impl Record for StatsRecorder {
    fn record_admitted(&self) {
        self.ingress_admitted.inc();
    }
    fn record_refused_class(&self, class: RefusalClass) {
        StatsRecorder::record_refused_class(self, class);
    }
    fn record_quota(&self, quota: QuotaState) {
        StatsRecorder::record_quota(self, quota);
    }
    fn record_receipt(&self, digest: &[u8; 32]) {
        StatsRecorder::record_receipt(self, digest);
    }
}

/// C-ABI getter (fix-14 R-A1.2): writes the sealed snapshot into the
/// caller's buffer. Returns CGAP_OK (0) or a typed CgapStatus — the
/// status register (`sovereign.rs:68`) is the error channel; the
/// kernel never sleeps and never blocks the caller (R-A1.3).
///
/// # Safety
/// `out` must point to a caller-provided buffer of at least
/// `size_of::<CgasStatsBlock>()` bytes.
#[no_mangle]
pub unsafe extern "C" fn cgap_stats_snapshot(
    out: *mut CgasStatsBlock,
) -> crate::sovereign::CgapStatus {
    if out.is_null() {
        return crate::sovereign::CGAP_ERR_INVALID_MAGIC;
    }
    unsafe {
        core::ptr::write(out, global().snapshot());
    }
    crate::sovereign::CGAP_OK
}

/// Typed poll verdict off a sealed snapshot (fix-14 R-A1.4: readiness
/// is a state read off recorded events, never a clock). Returns
/// `Health::CrcInvalid` (refuse-to-interpret) when the CRC does not
/// verify — corruption precedes interpretation.
pub fn health(b: &CgasStatsBlock) -> Health {
    if b.block_crc != b.fnv1a32() {
        return Health::CrcInvalid;
    }
    if b.quota_tripped != 0 {
        return Health::QuotaTripped;
    }
    Health::Healthy
}

/// Reset every counter to zero and re-seal (operator-facing; the
/// receipt digest is zeroed too — reset is an explicit ops action;
/// the lifetime audit story lives in the receipt chain, not here).
pub fn reset(recorder: &StatsRecorder) {
    let fresh = StatsRecorder::const_new();
    recorder.magic.set(fresh.magic.get());
    recorder.layout_version.set(fresh.layout_version.get());
    recorder.ingress_admitted.set(0);
    recorder.refused_invalid_magic.set(0);
    recorder.refused_oabi_mismatch.set(0);
    recorder.refused_p28_spoliation.set(0);
    recorder.refusals_class_101.set(0);
    recorder.quota_max_refusals.set(0);
    recorder.quota_window_refusals.set(0);
    recorder.quota_tripped.set(0);
    recorder.quota_total_refusals.set(0);
    recorder.receipts_minted.set(0);
    for slot in &recorder.last_digest {
        slot.set(0);
    }
}
