//! Stats-block court (lane RC-05, FIX-14 A1 / RED-16 A1).
//!
//! Asserts final state over the real recorder — no mocks. Anti-vacuity
//! mutation: swap two counter increments in a /tmp sandbox copy of
//! `stats.rs` (admitted <-> refused_invalid_magic) — the behavior court
//! MUST fail there and pass here, so a mis-wired counter cannot pass
//! vacuously. Shared tree untouched by the mutation (sandbox law).

use cgap_ffi::stats::{
    health, reset, CgasStatsBlock, Health, QuotaState, RefusalClass, StatsRecorder,
    CGAS_STATS_LAYOUT_VERSION, CGAS_STATS_MAGIC,
};

/// Layout court: size/offsets asserted (repr(C) freeze).
const _: () = {
    assert!(core::mem::size_of::<CgasStatsBlock>() == 120);
    assert!(core::mem::align_of::<CgasStatsBlock>() == 8);
    assert!(core::mem::offset_of!(CgasStatsBlock, magic) == 0);
    assert!(core::mem::offset_of!(CgasStatsBlock, layout_version) == 4);
    assert!(core::mem::offset_of!(CgasStatsBlock, ingress_admitted) == 8);
    assert!(core::mem::offset_of!(CgasStatsBlock, refused_invalid_magic) == 16);
    assert!(core::mem::offset_of!(CgasStatsBlock, refused_oabi_mismatch) == 24);
    assert!(core::mem::offset_of!(CgasStatsBlock, refused_p28_spoliation) == 32);
    assert!(core::mem::offset_of!(CgasStatsBlock, refusals_class_101) == 40);
    assert!(core::mem::offset_of!(CgasStatsBlock, quota_max_refusals) == 48);
    assert!(core::mem::offset_of!(CgasStatsBlock, quota_window_refusals) == 52);
    assert!(core::mem::offset_of!(CgasStatsBlock, quota_tripped) == 56);
    assert!(core::mem::offset_of!(CgasStatsBlock, _pad0) == 60);
    assert!(core::mem::offset_of!(CgasStatsBlock, quota_total_refusals) == 64);
    assert!(core::mem::offset_of!(CgasStatsBlock, receipts_minted) == 72);
    assert!(core::mem::offset_of!(CgasStatsBlock, last_receipt_digest) == 80);
    assert!(core::mem::offset_of!(CgasStatsBlock, block_crc) == 112);
    assert!(core::mem::offset_of!(CgasStatsBlock, _pad1) == 116);
};

/// The recorder court: counters advance on recorded events, final
/// state asserted. Uses a LOCAL recorder (never the global — reset()
/// would clobber other tests' globals; Rust runs tests in parallel).
#[test]
fn counters_advance_on_recorded_events() {
    let rec = StatsRecorder::const_new();
    // Admissions.
    rec.record_admitted();
    rec.record_admitted();
    rec.record_admitted();
    // One refusal of each taxonomy bucket.
    rec.record_refused_class(RefusalClass::InvalidMagic);
    rec.record_refused_class(RefusalClass::InvalidMagic);
    rec.record_refused_class(RefusalClass::OabiMismatch);
    rec.record_refused_class(RefusalClass::P28Spoliation);
    rec.record_refused_class(RefusalClass::Class101);
    rec.record_refused_class(RefusalClass::Class101);
    // Quota publish + receipt.
    rec.record_quota(QuotaState {
        max_refusals: 4_096,
        window_refusals: 3,
        total_refusals: 7,
        tripped: false,
    });
    let digest: [u8; 32] = core::array::from_fn(|i| (i as u8) * 7 + 1);
    rec.record_receipt(&digest);

    let b = rec.snapshot();
    assert_eq!(b.magic, CGAS_STATS_MAGIC);
    assert_eq!(b.layout_version, CGAS_STATS_LAYOUT_VERSION);
    assert_eq!(b.ingress_admitted, 3);
    assert_eq!(b.refused_invalid_magic, 2);
    assert_eq!(b.refused_oabi_mismatch, 1);
    assert_eq!(b.refused_p28_spoliation, 1);
    assert_eq!(b.refusals_class_101, 2);
    assert_eq!(b.quota_max_refusals, 4_096);
    assert_eq!(b.quota_window_refusals, 3);
    assert_eq!(b.quota_tripped, 0);
    assert_eq!(b.quota_total_refusals, 7);
    assert_eq!(b.receipts_minted, 1);
    assert_eq!(b.last_receipt_digest, digest);
    // Sealed: the snapshot CRC verifies.
    assert_eq!(health(&b), Health::Healthy);
}

/// Health verdict court (fix-14 R-A1.4): a tripped quota degrades the
/// poll verdict to typed QuotaTripped; a corrupted block refuses
/// interpretation (CrcInvalid). Final state, no clock.
#[test]
fn health_verdict_degrades_on_tripped_quota_and_crc_corruption() {
    let rec = StatsRecorder::const_new();
    rec.record_quota(QuotaState {
        max_refusals: 4_096,
        window_refusals: 4_096,
        total_refusals: 4_096,
        tripped: true,
    });
    let b = rec.snapshot();
    assert_eq!(health(&b), Health::QuotaTripped);

    // Corruption: flip one counter byte in the sealed snapshot — CRC
    // must refuse to interpret the counters.
    let mut c = b;
    c.refusals_class_101 ^= 1;
    assert_eq!(health(&c), Health::CrcInvalid);

    // Hand-built block: sealed() must be self-consistent, and a
    // hand-minted bogus CRC must refuse.
    let sealed = CgasStatsBlock::sealed();
    assert_eq!(health(&sealed), Health::Healthy);
    let mut forged = sealed;
    forged.block_crc = !forged.block_crc;
    assert_eq!(health(&forged), Health::CrcInvalid);
}

/// Reset court: all counters return to zero, header intact, CRC
/// re-seals; new events count from zero afterward.
#[test]
fn reset_returns_all_counters_to_zero() {
    let rec = StatsRecorder::const_new();
    rec.record_admitted();
    rec.record_refused_class(RefusalClass::Class101);
    rec.record_quota(QuotaState {
        max_refusals: 16,
        window_refusals: 16,
        total_refusals: 1,
        tripped: true,
    });
    let digest: [u8; 32] = core::array::from_fn(|i| i as u8 + 3);
    rec.record_receipt(&digest);
    assert_ne!(rec.snapshot().ingress_admitted, 0);

    reset(&rec);
    let b = rec.snapshot();
    assert_eq!(b.magic, CGAS_STATS_MAGIC);
    assert_eq!(b.layout_version, CGAS_STATS_LAYOUT_VERSION);
    assert_eq!(b.ingress_admitted, 0);
    assert_eq!(b.refused_invalid_magic, 0);
    assert_eq!(b.refused_oabi_mismatch, 0);
    assert_eq!(b.refused_p28_spoliation, 0);
    assert_eq!(b.refusals_class_101, 0);
    assert_eq!(b.quota_max_refusals, 0);
    assert_eq!(b.quota_window_refusals, 0);
    assert_eq!(b.quota_tripped, 0);
    assert_eq!(b.quota_total_refusals, 0);
    assert_eq!(b.receipts_minted, 0);
    assert_eq!(b.last_receipt_digest, [0u8; 32]);
    assert_eq!(health(&b), Health::Healthy);
    // Counters count again from zero.
    rec.record_admitted();
    assert_eq!(rec.snapshot().ingress_admitted, 1);
}

/// C-ABI seam court: the getter writes a sealed snapshot and returns
/// CGAP_OK; a null out-pointer is a typed refusal (CgapStatus error
/// channel, fix-14 R-A1.2).
#[test]
fn c_abi_getter_seals_and_typed_refuses_null() {
    let mut out = CgasStatsBlock::sealed();
    out.ingress_admitted = 99; // poison: getter must overwrite
    let rc = unsafe { cgap_ffi::stats::cgap_stats_snapshot(&mut out as *mut _) };
    assert_eq!(rc, 0, "CGAP_OK expected");
    assert_eq!(out.magic, CGAS_STATS_MAGIC);
    assert_eq!(out.ingress_admitted, 0, "getter must serve the global recorder");
    assert_eq!(health(&out), Health::Healthy);
    let rc_null = unsafe { cgap_ffi::stats::cgap_stats_snapshot(core::ptr::null_mut()) };
    assert_ne!(rc_null, 0, "null out must be a typed refusal");
}

/// Global accessor court: `global()` is the zero-cost default recorder
/// a static works against; recording through it advances the very
/// snapshot `cgap_stats_snapshot` serves.
#[test]
fn global_recorder_is_live_and_snapshot_backed() {
    let before = cgap_ffi::stats::global().snapshot().receipts_minted;
    let digest = core::array::from_fn(|i| (i as u8) ^ 0xA5);
    cgap_ffi::stats::global().record_receipt(&digest);
    let after = cgap_ffi::stats::global().snapshot();
    assert_eq!(after.receipts_minted, before + 1);
    assert_eq!(after.last_receipt_digest, digest);
    assert_eq!(health(&after), Health::Healthy);
}
