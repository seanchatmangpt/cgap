//! Lane GIT-13 ref-store courts (Chicago: real collaborators, final-state
//! asserts, no mocks).
//!
//! Courts:
//! C1  CAS success path (create -> cas with fresh lease -> oid/lease/slot/
//!     chain advanced).
//! C2  Out-of-order stale-parent-lease CAS => typed StaleLease refusal,
//!     table fingerprint byte-unchanged (exit-101-class, in-process Err).
//! C3  Torn-write rollback: journal record written without commit marker,
//!     replay rolls back => oid/lease restored, TornWrite refusal.
//! C4  Monotonic counter strict increment across sequential updates
//!     (ceiling exhaustion refuses, never wraps).
//! C5  Namespace/capacity refusals.
//!
//! Anti-vacuity flip twins: each court names its mutation in the receipt;
//! the mutation battery runs against /tmp sandbox copies of ref_store.rs
//! (sha256-exact restore + mtime bump), never the shared tree.

#[path = "../src/ref_store.rs"]
mod ref_store;

use ref_store::*;

const NAME_MAIN: &[u8] = b"refs/heads/main";
const NAME_AGENT: &[u8] = b"refs/agents/a7";
const OID_A: [u8; 20] = [0xAA; 20];
const OID_B: [u8; 20] = [0xBB; 20];
const OID_C: [u8; 20] = [0xCC; 20];

#[test]
fn c1_cas_success_path() {
    let mut store = RefStore::new();
    let mut seal = SealRegister::new();
    store.create(NAME_MAIN, OID_A).expect("create main");

    // Fresh parent lease: create leaves lease 0; the register counter is 0.
    assert_eq!(seal.counter(), 0);
    let stamped = store.cas(NAME_MAIN, 0, OID_B, &mut seal).expect("cas ok");
    assert_eq!(stamped, 1, "first stamp returns counter 1");

    // Final state: oid advanced, lease bound to the stamped counter.
    let e = store.get(NAME_MAIN).expect("ref live");
    assert_eq!(e.oid, OID_B);
    assert_eq!(e.lease, 1);
    assert_eq!(seal.counter(), 1, "register advanced exactly once");
    assert_ne!(seal.chain(), 0, "seal chain recorded the stamp");
    assert_ne!(&seal.slot()[0..8], &[0u8; 8], "slot holds the stamp tag");

    // Second CAS with the fresh lease succeeds again (chained).
    let s2 = store
        .cas(NAME_MAIN, 1, OID_C, &mut seal)
        .expect("chained cas ok");
    assert_eq!(s2, 2);
    assert_eq!(store.get(NAME_MAIN).unwrap().oid, OID_C);
    assert_eq!(store.get(NAME_MAIN).unwrap().lease, 2);
    assert_eq!(seal.counter(), 2);
}

#[test]
fn c2_stale_parent_lease_refusal_table_byte_unchanged() {
    let mut store = RefStore::new();
    let mut seal = SealRegister::new();
    store.create(NAME_MAIN, OID_A).expect("create");
    store
        .cas(NAME_MAIN, 0, OID_B, &mut seal)
        .expect("first update");

    // Out-of-order update: the caller replays a STALE parent lease (0)
    // while the table already advanced to lease 1.
    let before_fp = store.fingerprint();
    let err = store
        .cas(NAME_MAIN, 0, OID_C, &mut seal)
        .expect_err("stale lease must refuse");
    assert_eq!(err.kind, RefusalKind::StaleLease);
    assert_eq!(err.kind.tag(), "STALE_LEASE");
    assert!(
        err.msg.contains("stale parent lease"),
        "typed refusal carries the Andon diagnostic: {}",
        err.msg
    );

    // Byte-unchanged final state.
    assert_eq!(store.fingerprint(), before_fp, "table bytes untouched");
    assert_eq!(store.get(NAME_MAIN).unwrap().oid, OID_B, "oid unchanged");
    assert_eq!(store.get(NAME_MAIN).unwrap().lease, 1, "lease unchanged");
    assert_eq!(seal.counter(), 1, "refused update stamps nothing");
}

#[test]
fn c3_torn_write_rollback() {
    let mut store = RefStore::new();
    let mut seal = SealRegister::new();
    store.create(NAME_AGENT, OID_A).expect("create");
    store.cas(NAME_AGENT, 0, OID_B, &mut seal).expect("update");
    let good_fp = store.fingerprint();

    // Sequential write WITHOUT commit marker (torn write), then replay.
    let mut j = Journal::new();
    j.write(1, NAME_AGENT, OID_C, store.get(NAME_AGENT).unwrap().lease)
        .expect("journal write");
    let applied = j.replay(&mut store);

    // The torn record stops replay: rollback at the marker gap.
    assert!(applied.is_err(), "replay must refuse on missing marker");
    assert_eq!(applied.unwrap_err().kind, RefusalKind::TornWrite);
    assert_eq!(j.last_refusal(), Some(RefusalKind::TornWrite));

    // Final state: table restored to the pre-journal bytes.
    assert_eq!(store.fingerprint(), good_fp, "rollback left table intact");
    assert_eq!(store.get(NAME_AGENT).unwrap().oid, OID_B);

    // Now commit the marker and replay applies exactly one record.
    j.commit(1).expect("commit marker");
    assert_eq!(j.replay(&mut store), Ok(1));
    assert_eq!(store.get(NAME_AGENT).unwrap().oid, OID_C);
    assert_eq!(
        store.get(NAME_AGENT).unwrap().lease,
        2,
        "lease advanced past recorded parent"
    );
}

#[test]
fn c4_monotonic_strict_increment_no_wrap() {
    let mut store = RefStore::new();
    let mut seal = SealRegister::with_ceiling(4);
    store.create(NAME_MAIN, OID_A).expect("create");

    let mut prev = 0u64;
    for i in 1..=4u64 {
        let oid = [i as u8; 20];
        let s = store
            .cas(NAME_MAIN, prev, oid, &mut seal)
            .unwrap_or_else(|e| panic!("update {i} refused: {e:?}"));
        assert_eq!(s, i, "counter strictly increments");
        assert!(s > prev);
        assert_eq!(store.get(NAME_MAIN).unwrap().lease, i);
        prev = s;
    }
    assert_eq!(seal.counter(), 4);

    // Ceiling reached: further stamp refuses CounterExhausted, never wraps.
    let err = store
        .cas(NAME_MAIN, 4, OID_C, &mut seal)
        .expect_err("exhausted counter refuses");
    assert_eq!(err.kind, RefusalKind::CounterExhausted);
    assert_eq!(
        store.get(NAME_MAIN).unwrap().lease,
        4,
        "no wrap past ceiling"
    );
}

#[test]
fn c5_namespace_and_capacity_refusals() {
    let mut store = RefStore::new();
    let mut seal = SealRegister::new();

    // Bad namespace.
    let err = store
        .create(b"refs/tags/v1", OID_A)
        .expect_err("refs/tags refused");
    assert_eq!(err.kind, RefusalKind::BadNamespace);
    let err = store
        .cas(b"refs/orphans/x", 0, OID_A, &mut seal)
        .expect_err("unknown + bad namespace ref");
    assert_eq!(err.kind, RefusalKind::UnknownRef, "unknown ref first");
    assert_eq!(store.len(), 0);

    // refs/agents/* is a legal namespace.
    store
        .create(NAME_AGENT, OID_A)
        .expect("agents namespace ok");

    // Capacity: fill to the fixed cap.
    for i in 0..(REF_TABLE_CAP - 1) {
        let mut name = b"refs/heads/b".to_vec();
        name.push(b'a' + (i % 26) as u8);
        name.push(b'0' + (i / 26) as u8);
        store.create(&name, OID_A).expect("fill");
    }
    let err = store
        .create(b"refs/heads/overflow", OID_A)
        .expect_err("table full refuses");
    assert_eq!(err.kind, RefusalKind::TableFull);
    assert_eq!(store.len(), REF_TABLE_CAP);
}
