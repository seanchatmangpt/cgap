//! GAP-02 (EXP-04 T3): in-crate chained-receipt mutation courts.
//!
//! Ports the external court's protection (crates/csct/src/bin/
//! cscb_c7_receipt_sealing.rs:73-96 verify_chain, :174-202 bitflip court)
//! into `cgap-blake3` itself, which previously had NO bitflip-rejection or
//! parent-digest-chain mutation test (dod-cgap-blake3.md:57-67).
//!
//! Structure per receipt: 8B nonce (LE) + 32B parent digest + 32B payload.
//! parent = previous receipt's digest; genesis parent = zero digest.
//! Digests are the real `cgap_blake3::hash` over the full frame.

use cgap_blake3::hash;

const GENESIS_PARENT: [u8; 32] = [0u8; 32];
const MUTATE_BYTE: usize = 8 + 32 + 7; // last payload byte of the frame
const MUTATE_MASK: u8 = 1 << 3;

struct Receipt {
    frame: Vec<u8>,
    digest: [u8; 32],
}

/// Deterministic payload from the nonce (no RNG dep needed; a simple
/// multiplicative walk over the 32 payload bytes).
fn fill_payload(frame: &mut [u8]) {
    let nonce = u64::from_le_bytes(frame[0..8].try_into().unwrap());
    let mut state = nonce.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
    for b in frame[40..72].iter_mut() {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *b = (state >> 33) as u8;
    }
}

fn seal(nonce: u64, parent: [u8; 32]) -> Receipt {
    let mut frame = vec![0u8; 72];
    frame[0..8].copy_from_slice(&nonce.to_le_bytes());
    frame[8..40].copy_from_slice(&parent);
    fill_payload(&mut frame);
    let digest = hash(&frame);
    Receipt { frame, digest }
}

/// Chain-validity court (port of verify_chain, cscb_c7:77-96): nonces
/// strictly increment from 1, parent == previous digest (genesis parent =
/// zero digest), every digest re-derives via the real hash. Typed Err on
/// the first violation.
fn verify_chain(receipts: &[Receipt]) -> Result<(), String> {
    if receipts.is_empty() {
        return Err("[GAP02-FAIL] empty chain".to_string());
    }
    let mut parent = GENESIS_PARENT;
    for (i, r) in receipts.iter().enumerate() {
        let expect = ((i + 1) as u64).to_le_bytes();
        if r.frame[0..8] != expect {
            return Err(format!("[GAP02-FAIL] nonce break at receipt {}", i));
        }
        if r.frame[8..40] != parent[..] {
            return Err(format!("[GAP02-FAIL] parent break at receipt {}", i));
        }
        if hash(&r.frame) != r.digest {
            return Err(format!("[GAP02-FAIL] digest mismatch at receipt {}", i));
        }
        parent = r.digest;
    }
    Ok(())
}

fn seal_chain(n: u64) -> Vec<Receipt> {
    let mut chain = Vec::with_capacity(n as usize);
    let mut parent = GENESIS_PARENT;
    for nonce in 1..=n {
        let r = seal(nonce, parent);
        parent = r.digest;
        chain.push(r);
    }
    chain
}
// =========================================================================
// Court 0 (baseline): the clean chain passes BEFORE any mutation.
// =========================================================================
#[test]
fn court0_clean_chain_verifies() {
    let chain = seal_chain(16);
    verify_chain(&chain).expect("clean chain must verify");
}

// =========================================================================
// Court A: bitflip one byte in a child payload -> verification rejects.
// Flip is witnessed BEFORE (clean passes) / AFTER (mutant fails).
// =========================================================================
#[test]
fn court_a_child_payload_bitflip_rejected() {
    let chain = seal_chain(16);
    // BEFORE: clean chain verifies.
    verify_chain(&chain).expect("BEFORE: clean chain must verify");

    // Land the flip on receipt 7's payload (not the root, not genesis).
    let k = 7;
    let orig = chain[k].frame[MUTATE_BYTE];
    let mut mutant_chain: Vec<Receipt> = chain
        .iter()
        .map(|r| Receipt { frame: r.frame.clone(), digest: r.digest })
        .collect();
    mutant_chain[k].frame[MUTATE_BYTE] ^= MUTATE_MASK;

    // Land-check: the flip must actually change the frame byte.
    assert_ne!(
        mutant_chain[k].frame[MUTATE_BYTE], orig,
        "flip did not land at byte {}",
        MUTATE_BYTE
    );

    // AFTER: the court must reject. Rejection lands at receipt k (digest
    // no longer re-derives over the mutated payload) or k+1 (stored
    // parent no longer equals the mutated receipt's digest).
    let err = verify_chain(&mutant_chain).expect_err("mutant chain must be REJECTED");
    assert!(
        err.contains("receipt 7") || err.contains("receipt 8"),
        "rejection should cite receipt 7 or 8, got: {}",
        err
    );

    // Differential witness: mutating ONLY the stored digest (frames
    // intact) must also be caught - proves rejection comes from digest
    // re-derivation, not a stale fixture.
    let mut digest_mutant: Vec<Receipt> = chain
        .iter()
        .map(|r| Receipt { frame: r.frame.clone(), digest: r.digest })
        .collect();
    digest_mutant[k].digest[0] ^= 1;
    assert!(
        verify_chain(&digest_mutant).is_err(),
        "stored-digest-only mutation must also be rejected"
    );
}

// =========================================================================
// Court B: duplicate/identical digest substitution -> typed FAIL.
// An attacker replaying receipt k-1's digest forward as receipt k's
// stored digest must produce a typed rejection naming digest mismatch -
// never a silent pass.
// =========================================================================
#[test]
fn court_b_duplicate_digest_substitution_typed_fail() {
    let chain = seal_chain(8);
    verify_chain(&chain).expect("BEFORE: clean chain must verify");

    let k = 5;
    let mut mutant: Vec<Receipt> = chain
        .iter()
        .map(|r| Receipt { frame: r.frame.clone(), digest: r.digest })
        .collect();
    // Identical-digest substitution: receipt k claims receipt k-1's digest.
    mutant[k].digest = chain[k - 1].digest;

    let err = verify_chain(&mutant).expect_err("duplicate-digest mutant must be REJECTED");
    assert!(
        err.contains("digest mismatch"),
        "typed FAIL must name digest mismatch, got: {}",
        err
    );

    // Witness the substitution landed: the digests were distinct BEFORE.
    assert_ne!(chain[k].digest, chain[k - 1].digest, "substitution was vacuous: digests already equal");
}

// =========================================================================
// Court C: mutate the stored ROOT digest, re-derive the true root from
// the intact frames, and assert the mismatch is detected (exit-1-class:
// the court's Err is the in-test analogue of the external bin's
// process::exit(1) typed failure).
// =========================================================================
#[test]
fn court_c_root_mutation_rederivation_mismatch() {
    let chain = seal_chain(16);
    verify_chain(&chain).expect("BEFORE: clean chain must verify");

    let last = chain.len() - 1;
    let true_root = chain[last].digest;

    // Re-derive the root from frames only (what a verifier would do).
    let derived = hash(&chain[last].frame);
    assert_eq!(derived, true_root, "BEFORE: re-derived root must match");

    // Mutate the stored root.
    let mut mutant: Vec<Receipt> = chain
        .iter()
        .map(|r| Receipt { frame: r.frame.clone(), digest: r.digest })
        .collect();
    mutant[last].digest[31] ^= 0x80;
    assert_ne!(mutant[last].digest, true_root, "root mutation did not land");

    // AFTER: full-chain court rejects, and a direct re-derivation
    // comparison also mismatches (the exit-1-class assertion).
    let err = verify_chain(&mutant).expect_err("mutated-root chain must be REJECTED");
    assert!(err.contains("digest mismatch"), "typed FAIL expected, got: {}", err);
    assert_ne!(hash(&mutant[last].frame), mutant[last].digest, "mutated root must not re-derive");
}
