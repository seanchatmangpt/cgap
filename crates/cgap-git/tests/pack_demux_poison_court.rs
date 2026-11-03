//! Pack demux poison court (lane FIX-05, v26.11.2). Courts for the two
//! RED-09 findings (red-09-alloc-sentry-breach.md W6):
//!
//! - F1: post-trailer poison tail is a typed refusal
//!   (DemuxError::TrailingBytes), never silent absorption. Anti-vacuity
//!   mutation: revert the `Phase::Done` arm to `{}` (the pre-fix state,
//!   pack_demux.rs:343 pre-FIX-05) in a /tmp sandbox copy; the court
//!   MUST flip to failure.
//! - F2: surplus objects (supply > header-declared count) are refused
//!   typed (DemuxError::ObjectSurplus guard in close_record plus the
//!   trailer-absorption channel). Anti-vacuity mutation: revert the
//!   whole FIX-05 patch (guard + Done arm) in a /tmp sandbox; the
//!   surplus-exploit pack MUST be admitted pre-fix and refused post-fix.
//!
//! Chicago: real byte-built packs via cgap_git::pack_write, no mocks.

use cgap_git::pack_demux::{Demux, DemuxError};
use cgap_git::pack_write::{encode_object_header, sha1};

/// Build a stored-framing pack (mirrors pack_demux_court.rs builder).
fn build_pack(objs: &[(u8, Vec<u8>)]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"PACK");
    out.extend_from_slice(&2u32.to_be_bytes());
    out.extend_from_slice(&(objs.len() as u32).to_be_bytes());
    for (t, body) in objs {
        out.extend_from_slice(&encode_object_header(*t, body.len() as u64));
        out.extend_from_slice(body);
    }
    let sum = sha1(&out);
    out.extend_from_slice(&sum);
    out
}

/// P1 baseline: a clean 1-object pack demuxes with zero poison.
#[test]
fn p1_clean_pack_zero_poison() {
    let pack = build_pack(&[(3u8, b"body".to_vec())]);
    let mut d = Demux::new();
    d.feed(&pack).expect("clean pack feed");
    assert!(d.is_done());
    assert_eq!(d.finish(), Ok(()));
    assert_eq!(d.trailing_poison(), 0, "clean pack must carry zero poison");
    assert_eq!(d.objects_seen(), 1);
}

/// P2 F1 COURT: 4 garbage bytes after the trailer are a typed
/// TrailingBytes refusal (RED-09 F1 witness: feed=Ok, done=true,
/// bytes_fed 34->38 pre-fix).
/// ANTI-VACUITY MUTATION: revert the Done arm to `Phase::Done => {}` in
/// a /tmp sandbox copy (run_fix05_mutation.sh, mutation m1); the court
/// MUST fail under the mutation (feed returns Ok, poison never typed).
#[test]
fn p2_post_trailer_poison_is_typed_refusal() {
    let pack = build_pack(&[(3u8, b"body".to_vec())]);
    let mut poisoned = pack.clone();
    poisoned.extend_from_slice(b"\xDE\xAD\xBE\xEF");
    let mut d = Demux::new();
    let err = d
        .feed(&poisoned)
        .expect_err("VACUOUS: post-trailer poison silently absorbed");
    assert_eq!(err, DemuxError::TrailingBytes(4));
    assert_eq!(d.trailing_poison(), 4, "poison ledger must count 4");
    assert!(d.is_done(), "trailer captured before the poison arrived");
    assert_eq!(d.finish(), Ok(()));
    assert_eq!(d.objects_seen(), 1);
    // poison does not corrupt the captured trailer
    assert_eq!(
        d.captured_trailer().expect("trailer"),
        sha1(&pack[..pack.len() - 20])
    );
}

/// P3 F2 COURT (surplus-object header lie): the pack header declares 1
/// object but TWO well-formed objects are supplied. The surplus object
/// cannot occupy a demux record slot; its bytes overflow the trailer
/// capture and MUST be refused typed, not absorbed into a fake clean
/// finish (RED-09 F2: close_record had no surplus tripwire).
/// Asserts final state: typed error, objects_seen pinned at declared.
/// ANTI-VACUITY MUTATION: restore the PRE-FIX-05 pack_demux.rs (guard +
/// Done arm both reverted; run_fix05_mutation.sh, mutation m2) in a
/// /tmp sandbox copy; the surplus pack MUST demux to Ok(done) there,
/// flipping this court to failure.
#[test]
fn p3_surplus_object_supply_is_refused() {
    // surplus object framing: 2-byte size varint (24 > 15) + 24-byte
    // body = 26 bytes. The trailer capture absorbs its first 20 bytes;
    // the remaining 6 PLUS the 20-byte (now stale) checksum all land
    // past Done -> 26 poison bytes total.
    let surplus_body = vec![b's'; 24];
    let mut pack = build_pack(&[(3u8, b"body".to_vec()), (3u8, surplus_body)]);
    pack[8..12].copy_from_slice(&1u32.to_be_bytes()); // declared 2 -> 1
    let mut d = Demux::new();
    let err = d
        .feed(&pack)
        .expect_err("VACUOUS: surplus object absorbed into a clean finish");
    assert_eq!(
        err,
        DemuxError::TrailingBytes(26),
        "surplus bytes + stale checksum past the trailer must be the typed refusal"
    );
    assert_eq!(
        d.objects_seen(),
        d.declared_objects().unwrap(),
        "objects_seen must stay pinned at the header-declared count"
    );
}
