//! W2-13 L5-COURT — seal chain ASC450 metering, pre-lever control
//! (`docs/jira/v26.11.2/07-court-corpus.md` §2 L5-COURT).
//!
//! Mixed corpus through the CURRENT unfused governance path (P28 §5
//! validate -> ASC450 metering -> chatman seal), asserting the three-way
//! invariants the W3 fused arm must byte-match:
//!
//! 1. Crossing discipline: the unlicensed half of the mixed corpus crosses
//!    and emits; the licensed half never emits; the latch makes the
//!    emission one-shot.
//! 2. prev_root chaining: `prev_proof_digest` is the genesis all-zero word
//!    before any emission and the BLAKE3 digest of the emitted proof after
//!    it (chained, not parallel claims — A16 §6).
//! 3. Checksum/seal: `SignedProof::verify` and `SealedMeterState::verify`
//!    are Ok/true on the emitted artifacts and FAIL on a one-byte sandbox
//!    flip of a covered byte (anti-vacuity: the seal court flips both ways).
//!
//! The source-level mutations named by 02-pareto-levers.md §L5 (bit-flip an
//! upstream receipt in the kernel, demote a crossing, flip a receipt digest
//! in the kernel) are source mutations — the reachable in-test twins are
//! wired below and the kernel-level procedures are recorded as named
//! bench-level courts in the lane receipt
//! (`docs/jira/v26.11.2/waves/w2-13-l4l5-courts.md`).

use cgap_crypto::lease::Lease;
use cgap_p28::asc450::{
    Asc450Meter, LeaseEpochSealer, GENESIS_PREV_PROOF_DIGEST, PROOF_LEN, STATE_LEN,
    STATE_LATCHED_CLEAR, STATE_LATCHED_SET,
};
use cgap_p28::attestation::{
    instrument_header_span, origin_authority_digest, P28LicenseAttestation, ATTESTATION_LEN,
    ATTESTATION_VERSION, FLAG_DERIVATIVE, FLAG_ROOT, LICENSE_MAGIC,
    SAFE_HARBOR_CEILING_MAX_CENTS,
};

/// The REAL pinned instrument from the repo root.
fn instrument_span() -> Vec<u8> {
    let full = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../LICENSE-P28-SIL.md"
    ))
    .expect("pinned instrument readable from the repo root");
    instrument_header_span(&full)
        .map(|s| s.to_vec())
        .expect("canonical instrument has a `---` span")
}

/// Root attestation over the real instrument span.
fn root_for(span: &[u8]) -> P28LicenseAttestation {
    P28LicenseAttestation {
        license_magic: LICENSE_MAGIC,
        version: ATTESTATION_VERSION,
        attestation_flags: FLAG_ROOT,
        safe_harbor_ceiling_cents: SAFE_HARBOR_CEILING_MAX_CENTS,
        origin_authority_digest: origin_authority_digest(span),
        upstream_receipt: [0u8; 32],
    }
}

/// Derivative bound to its parent's wire bytes (§5.7); the court mutates the
/// binding to prove validate refuses.
fn derivative_from(parent: &P28LicenseAttestation) -> P28LicenseAttestation {
    P28LicenseAttestation {
        license_magic: LICENSE_MAGIC,
        version: ATTESTATION_VERSION,
        attestation_flags: FLAG_DERIVATIVE,
        safe_harbor_ceiling_cents: SAFE_HARBOR_CEILING_MAX_CENTS,
        origin_authority_digest: origin_authority_digest(&instrument_span()),
        upstream_receipt: cgap_blake3::hash(&parent.to_bytes()),
    }
}

/// A real A16 lease epoch with a deterministic key (same construction as the
/// in-crate asc450 courts); every seal consumes one tick.
fn fresh_sealer(tag: u64) -> LeaseEpochSealer {
    let mut lease_id = [0u8; 32];
    lease_id[0..8].copy_from_slice(&tag.to_le_bytes());
    let mut issued_root = [0u8; 32];
    issued_root[0..8].copy_from_slice(&tag.to_le_bytes());
    LeaseEpochSealer::new(Lease::new(lease_id, 1 << 40, issued_root))
}

/// L5-COURT pre-lever control: mixed corpus through the unfused governance
/// path; three-way invariants (crossing discipline, prev_root chaining,
/// checksum/seal) plus the one-byte flip on each sealed artifact.
#[test]
fn l5_mixed_corpus_three_way_invariants() {
    let span = instrument_span();
    let root = root_for(&span);
    // Deterministic key: a second sealer over the same lease epoch key
    // verifies artifacts sealed by the first (Lease::new is key-deterministic
    // on the tag; verify is pure and consumes no ticks).
    let sealer = fresh_sealer(0xA20C_0FFE_E5A7_E000);
    let verify_sealer = fresh_sealer(0xA20C_0FFE_E5A7_E000);
    let mut meter = Asc450Meter::new(1_000_000, &root, sealer)
        .expect("real pinned root attestation is intrinsically valid");

    // Sealed state minted pre-emission chains the GENESIS word.
    let genesis_sealed = meter.seal_state();
    assert_eq!(
        genesis_sealed.prev_proof_digest, GENESIS_PREV_PROOF_DIGEST,
        "pre-emission sealed state must chain the genesis all-zero word"
    );
    assert_eq!(genesis_sealed.latched, STATE_LATCHED_CLEAR);

    // Mixed corpus, N = 4: two licensed (below ceiling, never emit) and two
    // unlicensed (the first crosses). Crossing-capable partition = N/2 = 2;
    // exactly one crossing proof is emitted (one-shot latch) and it is
    // verified — the "proven == N/2" discipline at the latch-saturating
    // corpus size.
    const N: usize = 4;
    let mut proofs = Vec::new();
    let mut emitted = 0usize;
    for (i, (licensed, amt)) in [
        (true, 100_000u64),
        (false, 2_000_000u64),
        (true, 100_000u64),
        (false, 500_000u64),
    ]
    .into_iter()
    .enumerate()
    {
        let res = if licensed {
            meter.record_licensed(amt, i as u64).map(|_| None)
        } else {
            meter.record_unlicensed(amt, i as u64)
        };
        assert!(res.is_ok(), "corpus event {i} must be accepted, not overflowed");
        if let Ok(Some(p)) = res {
            emitted += 1;
            proofs.push(p);
        }
    }

    assert_eq!(emitted, 1, "exactly one crossing proof (one-shot latch)");
    assert_eq!(proofs.len(), 1);
    assert!(meter.is_latched(), "latch must be set after emission");
    assert_eq!(meter.event_count(), N as u64, "all N events accepted");

    let proof = &proofs[0];
    let proof_bytes = proof.to_bytes();
    assert_eq!(proof_bytes.len(), PROOF_LEN);
    // Seal/checksum: emitted proof verifies Ok.
    proof
        .verify(&verify_sealer)
        .expect("emitted proof must verify under its lease epoch");

    // prev_root chaining: sealed state minted by the emission chains the
    // BLAKE3 digest of the emitted proof.
    let sealed = meter
        .last_sealed_state()
        .expect("emission must mint a sealed state");
    assert_eq!(
        sealed.prev_proof_digest,
        Asc450Meter::<LeaseEpochSealer>::proof_digest(proof),
        "sealed state must chain the emitted proof digest"
    );
    assert_eq!(sealed.latched, STATE_LATCHED_SET);
    assert_eq!(
        sealed.attestation_digest,
        meter.attestation_digest(),
        "chain anchor must be the root attestation digest"
    );
    // Checksum flip (anti-vacuity): one flipped covered byte in the sealed
    // state's signed span must fail verify_state.
    let mut sb = sealed.to_bytes();
    assert_eq!(sb.len(), STATE_LEN);
    sb[10] ^= 0x01; // inside the 0..STATE_SIGNED_SPAN covered span
    let flipped_state =
        cgap_p28::asc450::SealedMeterState::from_bytes(&sb).expect("flip stays structural");
    assert!(
        !flipped_state.verify(&verify_sealer),
        "one flipped sealed-state byte must FAIL the state checksum"
    );

    // Checksum flip (anti-vacuity): one flipped covered byte in the proof.
    let mut pb = proof_bytes;
    pb[100] ^= 0x80; // covered by the 0..SIGNED_SPAN seal, shape-clean
    let flipped_proof =
        cgap_p28::asc450::SignedProof::from_bytes(&pb).expect("flip stays structural");
    assert!(
        flipped_proof.verify(&verify_sealer).is_err(),
        "one flipped proof byte must FAIL the proof checksum"
    );

    // Final seal chains the same proof digest (no new emission after latch).
    let final_sealed = meter.seal_state();
    assert_eq!(
        final_sealed.prev_proof_digest,
        Asc450Meter::<LeaseEpochSealer>::proof_digest(proof),
        "post-latch seals chain the same last proof digest"
    );
    assert!(
        final_sealed.verify(&verify_sealer),
        "final seal must checksum clean"
    );
}

/// L5 source-mutation twin (reachable): bit-flip the upstream receipt
/// binding a derivative to its parent — P28 §5 validate must REFUSE the
/// spoliated derivative (validate-refusal leg of the L5 kill signal).
#[test]
fn l5_upstream_receipt_flip_is_refused() {
    let span = instrument_span();
    let parent = root_for(&span);
    let child = derivative_from(&parent);
    // Positive twin: the unmutated binding validates under its parent.
    assert!(
        child.validate(&span, Some(&parent)).is_ok(),
        "honest derivative must validate against its parent"
    );

    // FLIP: one byte of upstream_receipt (offset 48, A17 §2.1).
    let mut wire = child.to_bytes();
    assert_eq!(wire.len(), ATTESTATION_LEN);
    wire[48] ^= 0x01;
    let flipped = P28LicenseAttestation::from_bytes(&wire).expect("flip stays structural");
    assert!(
        flipped.validate(&span, Some(&parent)).is_err(),
        "bit-flipped upstream receipt must be REFUSED by validate"
    );
}
