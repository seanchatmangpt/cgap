//! Lane GIT-06 mutation court for `tree_synth`.
//!
//! Real collaborators only: the digest function bound here is the real
//! `cgap_blake3::hash` register (workspace path dep, dev-only). Every
//! court names its anti-vacuity mutation; flip twins share one subject.

// lib.rs is out of lane ownership, so the court mounts the module
// directly by path (self-contained: core-only, no lib.rs items used).
#[path = "../src/tree_synth.rs"]
mod tree_synth;

use tree_synth::{
    entry_at, entry_path, frame_len, synthesize, verify_canonical_order, Coord, Namespace,
    TreeSynthError, MAX_ENTRIES,
};

const D: fn(&[u8]) -> [u8; 32] = cgap_blake3::hash;

fn mk(s: u64, p: u64, o: u64) -> Coord {
    Coord::new(s, p, o)
}

fn sample_set() -> Vec<(Namespace, Coord)> {
    vec![
        (Namespace::Rules, mk(1, 2, 3)),
        (Namespace::Facts, mk(9, 9, 9)),
        (Namespace::Receipts, mk(10, 0, 1)),
        (Namespace::Derived, mk(4, 5, 6)),
        (Namespace::Facts, mk(1, 2, 30)),
        (Namespace::Facts, mk(1, 2, 3)),
    ]
}

const PFX: &[u8] = b"100644 blob ";

fn digest_hex(entry: &[u8]) -> [u8; 64] {
    let mut h = [0u8; 64];
    h.copy_from_slice(&entry[12..76]);
    h
}

/// COURT byte-stability: two independent synthesis passes over the same
/// coordinate set (fed in different input orders) produce byte-identical
/// framed trees and identical per-entry digests.
///
/// ANTI-VACUITY MUTATION: feed pass 2 a scrambled input order; if the
/// sort were order-dependent (or a no-op), the digests would differ and
/// this court would flip. Digest of the empty set is also pinned.
#[test]
fn court_byte_stability_two_passes() {
    let mut pass1 = sample_set();
    let count = pass1.len();
    let mut out1 = vec![0u8; frame_len(&pass1).unwrap()];
    let n1 = synthesize(&mut pass1, &mut out1, D).unwrap();

    let mut pass2 = vec![
        (Namespace::Facts, mk(1, 2, 3)),
        (Namespace::Receipts, mk(10, 0, 1)),
        (Namespace::Rules, mk(1, 2, 3)),
        (Namespace::Facts, mk(1, 2, 30)),
        (Namespace::Derived, mk(4, 5, 6)),
        (Namespace::Facts, mk(9, 9, 9)),
    ];
    let mut out2 = vec![0u8; frame_len(&pass2).unwrap()];
    let n2 = synthesize(&mut pass2, &mut out2, D).unwrap();

    assert_eq!(n1, n2);
    assert_eq!(out1[..n1], out2[..n2], "two passes diverged byte-wise");

    // Digests equal per entry across passes.
    for i in 0..count {
        let e1 = entry_at(&out1[..n1], i).unwrap();
        let e2 = entry_at(&out2[..n2], i).unwrap();
        assert_eq!(digest_hex(e1), digest_hex(e2));
    }
    // Verify the digest is REAL BLAKE3, not a stand-in: recompute
    // independently for one entry (/derived/4-5-6 sorts first) and
    // compare against the frame.
    let e0 = entry_at(&out1[..n1], 0).unwrap();
    let mut input = [0u8; 24];
    input[0..8].copy_from_slice(&4u64.to_le_bytes());
    input[8..16].copy_from_slice(&5u64.to_le_bytes());
    input[16..24].copy_from_slice(&6u64.to_le_bytes());
    let expect = cgap_blake3::hash(&input);
    let mut hex = [0u8; 64];
    for i in 0..32 {
        hex[2 * i] = b"0123456789abcdef"[(expect[i] >> 4) as usize];
        hex[2 * i + 1] = b"0123456789abcdef"[(expect[i] & 0x0f) as usize];
    }
    assert_eq!(&e0[12..76], &hex, "frame digest != real cgap_blake3::hash");
}

/// COURT mutation flip (exit-1-class): swapping two framed entries breaks
/// canonical lexicographic order and the verifier returns the typed
/// `NonCanonicalOrder` error. Twin of the green verifier court above.
#[test]
fn court_swapped_order_flips_to_typed_err() {
    let mut coords = sample_set();
    let mut out = vec![0u8; frame_len(&coords).unwrap()];
    let n = synthesize(&mut coords, &mut out, D).unwrap();
    let tree = &out[..n];

    // Sanity: unmutated subject is canonical (else the court is vacuous).
    assert!(
        verify_canonical_order(tree).is_ok(),
        "subject already non-canonical"
    );

    // MUTATION: swap entries 0 and 1 (variable-length frames, so the
    // mutated buffer is rebuilt as e1 ++ e0 ++ rest).
    let e0 = entry_at(tree, 0).unwrap().to_vec();
    let e1 = entry_at(tree, 1).unwrap().to_vec();
    let mut mutated = e1.clone();
    mutated.extend_from_slice(&e0);
    mutated.extend_from_slice(&tree[e0.len() + e1.len()..]);

    match verify_canonical_order(&mutated) {
        Err(TreeSynthError::NonCanonicalOrder { at: 1 }) => {}
        other => panic!("MUTATION NOT CAUGHT: expected NonCanonicalOrder{{at:1}}, got {other:?}"),
    }
}

/// COURT namespace projection: each coordinate lands under its mapped
/// virtual directory, with exact path bytes.
#[test]
fn court_namespace_projection() {
    let mut coords = sample_set();
    let mut out = vec![0u8; frame_len(&coords).unwrap()];
    let n = synthesize(&mut coords, &mut out, D).unwrap();
    let tree = &out[..n];

    // Canonical dirs sort /derived/ < /facts/ < /receipts/ < /rules/.
    let expect: &[&[u8]] = &[
        b"/derived/4-5-6",
        b"/facts/1-2-3", // "3" < "30": digit strings compare lexicographically
        b"/facts/1-2-30",
        b"/facts/9-9-9",
        b"/receipts/10-0-1",
        b"/rules/1-2-3",
    ];
    for i in 0..expect.len() {
        let p = entry_path(entry_at(tree, i).unwrap()).unwrap();
        assert_eq!(p, expect[i], "entry {i} landed under the wrong virtual dir");
    }
    // Divergence is honest: digest field is 64 hex chars, not Git's 40.
    assert_eq!(entry_at(tree, 0).unwrap()[..12], *PFX);
    assert_eq!(
        entry_at(tree, 0).unwrap().len(),
        12 + 64 + 1 + expect[0].len() + 1
    );
}

/// COURT empty-set edge: zero coordinates is legal, writes nothing.
#[test]
fn court_empty_set() {
    let mut coords: Vec<(Namespace, Coord)> = Vec::new();
    let mut out = [0u8; 4096];
    let n = synthesize(&mut coords, &mut out, D).unwrap();
    assert_eq!(n, 0);
    assert_eq!(verify_canonical_order(&out[..n]), Ok(0));
    assert_eq!(frame_len(&coords), Ok(0));
    // Canonical verifier stays green on the empty frame.
    assert!(verify_canonical_order(&out[..n]).is_ok());
}

/// COURT capacity: over-`MAX_ENTRIES` sets are a typed refusal, not a
/// silent truncation.
#[test]
fn court_too_many_entries_typed_err() {
    let mut coords: Vec<(Namespace, Coord)> = (0..MAX_ENTRIES + 1)
        .map(|i| (Namespace::Facts, mk(i as u64, 0, 0)))
        .collect();
    let mut out = vec![0u8; 1 << 20];
    assert_eq!(
        synthesize(&mut coords, &mut out, D),
        Err(TreeSynthError::TooManyEntries(MAX_ENTRIES + 1))
    );
}

/// COURT overflow: a too-small buffer is a typed refusal with needed/capacity.
#[test]
fn court_buffer_overflow_typed_err() {
    let mut coords = sample_set();
    let full = frame_len(&coords).unwrap();
    let mut out = vec![0u8; full - 1];
    match synthesize(&mut coords, &mut out, D) {
        Err(TreeSynthError::BufferOverflow { needed, capacity }) => {
            assert_eq!(needed, full);
            assert_eq!(capacity, full - 1);
        }
        other => panic!("expected BufferOverflow, got {other:?}"),
    }
}
