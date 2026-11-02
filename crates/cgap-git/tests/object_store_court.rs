//! LANE GIT-05 court, GAP-01 wire-in: no_std object ledger over
//! fixed-capacity storage.
//!
//! Module under test is the MOUNTED lib module (`cgap_git::object_store`)
//! — GAP-01 dropped the `#[path]` mount now that lib.rs declares it.
//! The hasher is the injected `HashFn`; the court binds the real
//! `cgap_blake3::hash` via the dev-dep. Anti-vacuity: MUTATION COURT
//! (single payload-byte perturbation must flip `Ok` -> typed
//! `Err(StoreError::DigestMismatch)`); plus a stale/constant-hasher
//! rejection court (a hasher that ignores its input must be detected).
//! A court that cannot flip is FAIL. Zero-heap is asserted as a static
//! source instrument (banned-token sweep), not a runtime allocator count.

use cgap_git::object_store::*;
use ObjectKind::{Blob, Commit, Tag, Tree};

/// The real BLAKE3 register, bound as the injected `HashFn` (dev-dep).
const BLAKE3: HashFn = cgap_blake3::hash;

fn digest_hex(d: &Digest) -> String {
    d.iter().map(|b| format!("{:02x}", b)).collect()
}
use std::string::String;

/// Deterministic pseudo-random payloads (splitmix64, canonical seed law:
/// the SplitMix64 GOLDEN_GAMMA multiplier family — corpus-seed separation
/// preserved; this is a local generator seed, not a campaign seed).
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn payload(seed: u64, len: usize) -> Vec<u8> {
    let mut st = seed;
    (0..len)
        .map(|i| ((splitmix64(&mut st) >> (i % 8)) & 0xFF) as u8)
        .collect()
}

#[test]
fn roundtrip_all_kinds_and_layout_is_real_eytzinger() {
    let mut store = ObjectStore::new(BLAKE3);
    let mut digests = Vec::new();
    for (i, kind) in [Commit, Tree, Blob, Tag].iter().enumerate() {
        let p = payload(1000 + i as u64, 64);
        let d = store.insert(*kind, &p).expect("insert");
        digests.push(d);
    }
    assert_eq!(store.len(), 4);
    // Layout law: root of the eytzinger index is the in-order median slot.
    let sorted_slots = store.slots();
    // Layout law: in-order traversal of the eytzinger layout reproduces the
    // sorted slot digest order (the defining property of a BFS layout).
    let eyt = store.eytzinger();
    let n = store.len();
    let mut inorder: Vec<usize> = Vec::new();
    fn rec(eyt: &[usize], k: usize, n: usize, out: &mut Vec<usize>) {
        if k <= n {
            rec(eyt, 2 * k, n, out);
            out.push(eyt[k - 1]);
            rec(eyt, 2 * k + 1, n, out);
        }
    }
    rec(eyt, 1, n, &mut inorder);
    let sorted_digests: Vec<_> = sorted_slots.iter().map(|sl| sl.digest).collect();
    let eyt_digests: Vec<_> = inorder.iter().map(|i| sorted_slots[*i].digest).collect();
    assert_eq!(
        eyt_digests, sorted_digests,
        "eytzinger layout must be a valid BFS layout"
    );

    // Byte-exact fetch per kind.
    for (i, kind) in [Commit, Tree, Blob, Tag].iter().enumerate() {
        let (got_kind, got_payload) = store.get(&digests[i]).expect("get");
        assert_eq!(got_kind, *kind);
        assert_eq!(got_payload, payload(1000 + i as u64, 64));
    }
    // Miss is typed, not silent.
    let miss = store.get(&[0xAA; 32]);
    assert_eq!(miss, Err(StoreError::NotFound([0xAA; 32])));
}

#[test]
fn eytzinger_search_agrees_with_sorted_scan_present_and_absent() {
    let mut store = ObjectStore::new(BLAKE3);
    let mut keys = Vec::new();
    // 64 objects, digests spread deterministically; also probe absent keys
    // interleaved (one byte off each stored key) to exercise both descent arms.
    for i in 0..64u64 {
        let p = payload(7000 + i, ((i as usize) % MAX_PAYLOAD) + 1);
        let d = store.insert(Blob, &p).expect("insert");
        keys.push(d);
    }
    assert_eq!(store.len(), 64);
    for key in &keys {
        let eyt = store.find(key).expect("eyt hit");
        let linear = store
            .slots()
            .iter()
            .position(|s| &s.digest == key)
            .expect("linear hit");
        assert_eq!(eyt, linear, "eytzinger hit must match sorted scan position");
    }
    for key in &keys {
        let mut miss = *key;
        miss[0] ^= 0x80;
        if keys.contains(&miss) {
            continue;
        }
        assert!(
            matches!(store.find(&miss), Err(StoreError::NotFound(_))),
            "perturbed key must be a typed miss"
        );
    }
}

/// MUTATION COURT (anti-vacuity twin pair): an intact payload verifies Ok;
/// flipping exactly one payload byte must flip to typed DigestMismatch,
/// both at insert time (insert_checked) and verify time (check).
#[test]
fn mutation_single_payload_byte_flip_yields_typed_digest_mismatch() {
    let mut store = ObjectStore::new(BLAKE3);
    let p = payload(4242, 100);
    let d = store.insert(Blob, &p).expect("insert");
    // Baseline (control): intact bytes verify.
    assert_eq!(store.verify(&d), Ok(Blob));
    assert_eq!(ObjectStore::check(Blob, &p, d, BLAKE3), Ok(()));

    // Mutation: perturb exactly one payload byte.
    let mut q = p.clone();
    q[37] ^= 0x01;
    let err = ObjectStore::check(Blob, &q, d, BLAKE3);
    match err {
        Err(StoreError::DigestMismatch { expected, actual }) => {
            assert_eq!(expected, d);
            assert_ne!(actual, d);
        }
        other => panic!(
            "court did not flip: expected DigestMismatch, got {:?}",
            other
        ),
    }
    // insert_checked refuses a claimed digest that does not match its bytes.
    let claimed = cgap_blake3::hash(&q);
    assert_eq!(
        store.insert_checked(Blob, &p, claimed),
        Err(StoreError::DigestMismatch {
            expected: claimed,
            actual: d,
        })
    );
    // Ledger content flip: perturbing a byte changes the ledger digest.
    let before = store.ledger_digest();
    let idx = store.find(&d).expect("find");
    store.slots_mut_for_mutation_court()[idx].payload[0] ^= 0x01;
    assert_ne!(
        store.ledger_digest(),
        before,
        "ledger digest must flip on content perturbation"
    );
    assert!(matches!(
        store.verify(&d),
        Err(StoreError::DigestMismatch { .. })
    ));
}

/// Determinism court: identical object sets built via different insertion
/// orders produce identical ledger digests; a content change flips it.
#[test]
fn ledger_digest_is_byte_stable_across_insertion_orders() {
    let build = |shift: u64| -> ObjectStore {
        let mut store = ObjectStore::new(BLAKE3);
        let mut items: Vec<(ObjectKind, u64)> = vec![
            (Commit, 0),
            (Tree, 1),
            (Blob, 2),
            (Tag, 3),
            (Blob, 4),
            (Commit, 5),
        ];
        if shift == 1 {
            items.reverse();
        }
        for (kind, seed) in items {
            let p = payload(9000 + seed, 50);
            store.insert(kind, &p).expect("insert");
        }
        store
    };
    let a = build(0);
    let b = build(0);
    let rotated = build(1);
    let da = a.ledger_digest();
    assert_eq!(
        da,
        b.ledger_digest(),
        "same set, same order -> identical ledger digest"
    );
    assert_eq!(
        da,
        rotated.ledger_digest(),
        "sorted ledger must be insertion-order invariant"
    );
    // Hex stability (byte-stable framing witness).
    assert_eq!(digest_hex(&da).len(), 64);
    // Different content set flips the ledger digest.
    let mut store = ObjectStore::new(BLAKE3);
    store.insert(Blob, &payload(9999, 50)).expect("insert");
    assert_ne!(store.ledger_digest(), ObjectStore::new(BLAKE3).ledger_digest());
    assert_ne!(store.ledger_digest(), da);
}

#[test]
fn capacity_and_duplicate_courts() {
    let mut store = ObjectStore::new(BLAKE3);
    // Duplicate insert is typed Duplicate with the exact digest (pre-capacity).
    let p0 = payload(50_000, 16);
    let d0 = store.insert(Blob, &p0).expect("first insert");
    assert_eq!(store.insert(Blob, &p0), Err(StoreError::Duplicate(d0)));
    assert_eq!(store.find(&d0), Ok(0));
    // Fill to capacity with distinct payloads.
    for i in 1..MAX_OBJECTS {
        let p = payload(50_000 + i as u64, 16);
        store
            .insert(Blob, &p)
            .expect("insert must succeed under capacity");
    }
    assert_eq!(store.len(), MAX_OBJECTS);
    // One more is typed Full, and the store is uncorrupted afterwards.
    assert_eq!(
        store.insert(Blob, &payload(60_000, 16)),
        Err(StoreError::Full)
    );
    assert_eq!(store.len(), MAX_OBJECTS);
    // Payload-too-large is typed.
    let big = vec![0u8; MAX_PAYLOAD + 1];
    assert!(matches!(
        store.insert(Blob, &big),
        Err(StoreError::PayloadTooLarge { .. })
    ));
}

/// Zero-heap static instrument: the module source must not mint heap
/// containers. This is a source-scan invariant (stated), not an allocator
/// count — cgap-git has no allocator plumbing to hook in a unit test.
#[test]
fn zero_heap_static_invariant() {
    let src = include_str!("../src/object_store.rs");
    for banned in [
        "Vec<", "vec!", "String::", "format!", "Box::", "alloc::", "Rc::", "Arc::",
    ] {
        assert!(
            !src.contains(banned),
            "zero-heap violation: banned token {:?} present in object_store.rs",
            banned
        );
    }
    // Framing law is named, not implied: divergence from Git SHA-1 headers is
    // documented in a const, and framing is newline-terminated, BLAKE3-digested.
    assert!(CANONICAL_HEADER_LAW.contains("BLAKE3"));
    assert!(CANONICAL_HEADER_LAW.contains("NOT canonical Git"));
    let mut buf = [0u8; 64];
    let n = write_frame(Blob, b"hi", &mut buf).expect("frame");
    assert_eq!(&buf[..n], b"blob 2\nhi");
    assert_eq!(frame_len(Blob, 2), n);
}

/// ANTI-VACUITY (GAP-01): a hasher that IGNORES its input (constant/stale
/// digest) must be detected by the store — two distinct payloads collide,
/// so the second insert is a typed `Duplicate`, and `insert_checked` with
/// an honest claimed digest still rejects the mismatch. If this test
/// passes with a stale hasher actively corrupting the ledger, the store
/// is not actually consulting the injected digest fn.
#[test]
fn stale_constant_hasher_is_rejected_not_laundered() {
    // Stale digest fn: returns a fixed digest regardless of input bytes.
    fn stale(_data: &[u8]) -> Digest {
        [0x5A; 32]
    }
    let mut store = ObjectStore::new(stale as HashFn);
    let p1 = payload(77_001, 32);
    let p2 = payload(77_002, 32);
    assert_ne!(p1, p2, "payloads must differ so only the hasher can collide them");
    // Under the stale hasher the two distinct payloads address identically:
    // the FIRST insert lands, the SECOND is a typed Duplicate (not silent).
    let d1 = store.insert(Blob, &p1).expect("first insert under stale hasher");
    assert_eq!(d1, [0x5A; 32], "stale digest must be the returned address");
    assert_eq!(
        store.insert(Blob, &p2),
        Err(StoreError::Duplicate(d1)),
        "stale hasher must be observable as a typed duplicate collision"
    );
    // insert_checked with an honest (real BLAKE3) claimed digest must
    // reject: the store recomputes with its own hasher, mismatch surfaces.
    let honest = BLAKE3(&p2);
    assert_eq!(
        store.insert_checked(Blob, &p2, honest),
        Err(StoreError::DigestMismatch {
            expected: honest,
            actual: d1,
        })
    );
    // Control: under the real hasher the same two inserts are distinct.
    let mut good = ObjectStore::new(BLAKE3);
    let g1 = good.insert(Blob, &p1).expect("g1");
    let g2 = good.insert(Blob, &p2).expect("g2");
    assert_ne!(g1, g2, "real hasher must separate the same payloads");
}
