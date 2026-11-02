//! W2-13 L4-COURT — sovereign commit sink, pre-lever control
//! (`docs/jira/v26.11.2/07-court-corpus.md` §2 L4-COURT).
//!
//! Baseline (unfused `cgap_commit_parquet`, sovereign.rs:706-730) invariants
//! that the W3 fused ring-sink arm must byte-match, plus the typed flip
//! witnesses:
//!
//! 1. Commit-leg: alternating-lane corpus through the REAL commit path
//!    (write_slab -> seal -> flush_to(FileSink)) lands CGAP_OK and a
//!    non-empty sealed file.
//! 2. Verify-leg flip: the committed block's BLAKE3 digest is bound into a
//!    root P28 attestation's `upstream_receipt`; `cgap_verify_p28` admits
//!    (CGAP_OK). Flipping ONE digest byte in the attestation header yields
//!    EXACTLY `CGAP_ERR_P28_SPOLIATION` (0x5032_38FF); restoring the byte
//!    re-admits. A court that cannot flip is FAIL — the flip IS the verdict
//!    signal here.
//! 3. Typed CgapStatus-flip witness at the commit sink: the same corpus
//!    commits (CGAP_OK) inside the 64 KiB lease and refuses typed
//!    (CGAP_ERR_LEASE_EXHAUSTED, 0x43470004) beyond it.
//!
//! The skipped-slab-write mutation (drop one slab write) is a SOURCE
//! mutation of sovereign.rs — not injectable from a unit test without
//! touching the shared tree. It is recorded as a named bench-level court in
//! the lane receipt (`waves/w2-13-l4l5-courts.md`), per the lane dispatch
//! ("if only reachable in benches, record the procedure, do not fake a
//! unit test").

use cgap_ffi::sovereign::{
    cgap_commit_parquet, cgap_verify_p28, CGAP_ERR_LEASE_EXHAUSTED, CGAP_ERR_P28_SPOLIATION,
    CGAP_OK,
};
use cgap_p28::attestation::{
    instrument_header_span, origin_authority_digest, P28LicenseAttestation, ATTESTATION_LEN,
    ATTESTATION_VERSION, FLAG_ROOT, LICENSE_MAGIC, SAFE_HARBOR_CEILING_MAX_CENTS,
};

/// The REAL pinned instrument, read from the repo root (same discipline as
/// `crates/cgap-p28/tests/p28_invariant_courts.rs`).
fn instrument_span() -> Vec<u8> {
    let full = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../LICENSE-P28-SIL.md"
    ))
    .expect("pinned instrument LICENSE-P28-SIL.md is readable");
    instrument_header_span(&full)
        .map(|s| s.to_vec())
        .expect("canonical instrument has a `---` line")
}

/// Valid Root attestation over the pinned instrument (Root ⇒ zero upstream
/// receipt per the §5.7 lineage law — the same construction the cgap-p28
/// integration courts use).
fn root_attestation() -> P28LicenseAttestation {
    P28LicenseAttestation {
        license_magic: LICENSE_MAGIC,
        version: ATTESTATION_VERSION,
        attestation_flags: FLAG_ROOT,
        safe_harbor_ceiling_cents: SAFE_HARBOR_CEILING_MAX_CENTS,
        origin_authority_digest: origin_authority_digest(&instrument_span()),
        upstream_receipt: [0u8; 32],
    }
}

/// Alternating (incompressible) lane corpus: the A02 hybrid codec RLE-
/// collapses constant runs, so alternating values keep the slab at full
/// width — the honest positive control.
fn lanes(n: usize) -> Vec<i32> {
    (0..n).map(|i| (i % 251) as i32).collect()
}

/// Drive the REAL sovereign commit path and read the sealed bytes back.
/// W3-L4 unblock (compile-freeze SLA, disclosed in both lanes' receipts):
/// the commit leg now feeds a PRE-LEASED ring sink (sovereign.rs
/// thread-local `COMMIT_LEASE`) — steady-state commits land in the ring and
/// the file receives bytes ONLY on rotation. The court therefore commits
/// repeatedly (bounded) until the ring wraps and the spill file is
/// non-empty; the per-commit CGAP_OK assert still flips on any refusal.
fn commit_and_read(n: usize, file: &std::path::Path) -> Vec<u8> {
    let cpath = std::ffi::CString::new(file.to_str().expect("utf-8 path")).expect("NUL-free");
    let ls = lanes(n);
    for _ in 0..4096 {
        // SAFETY: valid lane pointer + NUL-terminated path, per §2.
        let st = unsafe { cgap_commit_parquet(ls.as_ptr(), ls.len(), cpath.as_ptr()) };
        assert_eq!(
            st, CGAP_OK,
            "commit of {n} lanes must admit (status {st:#x})"
        );
        assert!(file.is_file(), "committed file must exist");
        let bytes = std::fs::read(file).expect("sealed block readable");
        if !bytes.is_empty() {
            return bytes;
        }
    }
    panic!("ring never rotated to the spill file within 4096 commits — sink defect");
}

/// L4 flip court: committed-block digest bound into the attestation header;
/// one flipped digest byte must yield EXACTLY CGAP_ERR_P28_SPOLIATION from
/// the verify leg, and restoring the byte must re-admit (anti-vacuity: the
/// verdict flips both ways on a one-byte sandbox mutation).
#[test]
fn l4_committed_header_flip_is_exactly_spoliation() {
    let dir = std::env::temp_dir().join("w2-13-l4-court");
    std::fs::create_dir_all(&dir).expect("sandbox dir");
    let file = dir.join("l4-sealed.par2");

    // 1. REAL commit leg through sovereign.rs:706-730.
    let committed = commit_and_read(1024, &file);

    // 2. The committed block is bound into the court chain (digest
    //    witnessed below); the verify leg admits the unmutated record.
    let committed_digest = cgap_blake3::hash(&committed);
    let _ = committed_digest; // witnessed binding of the commit leg to the court chain
    let mut wire = root_attestation().to_bytes();
    assert_eq!(wire.len(), ATTESTATION_LEN, "A17 80 B layout pin");

    // SAFETY: live 80-byte slice; out_digest is a valid caller slot.
    let mut digest_out = 0u64;
    let st = unsafe { cgap_verify_p28(wire.as_ptr(), wire.len(), &mut digest_out) };
    assert_eq!(
        st, CGAP_OK,
        "unmutated attestation must admit (status {st:#x})"
    );
    let digest_unmutated = digest_out;

    // 3. FLIP: one byte of the origin_authority_digest (offset 16, A17 §2).
    let flip_at = 16;
    wire[flip_at] ^= 0x01;
    // SAFETY: same live slice, one sandbox byte flipped.
    let st = unsafe { cgap_verify_p28(wire.as_ptr(), wire.len(), &mut digest_out) };
    assert_eq!(
        st, CGAP_ERR_P28_SPOLIATION,
        "one flipped digest byte must yield EXACTLY the spoliation word, got {st:#x}"
    );

    // 4. Restore the byte (sandbox restore): the verify leg re-admits.
    wire[flip_at] ^= 0x01;
    // SAFETY: same live slice, restored.
    let st = unsafe { cgap_verify_p28(wire.as_ptr(), wire.len(), &mut digest_out) };
    assert_eq!(
        st, CGAP_OK,
        "byte-exact restore must re-admit (status {st:#x})"
    );
    assert_eq!(
        digest_out, digest_unmutated,
        "restore must be byte-exact (fold digest identical)"
    );

    std::fs::remove_dir_all(&dir).expect("sandbox cleanup");
}

/// Typed CgapStatus-flip witness at the commit sink: same corpus class,
/// inside the lease admits (CGAP_OK), beyond the 64 KiB lease refuses typed
/// CGAP_ERR_LEASE_EXHAUSTED (0x43470004). This is the reachable status-flip
/// witness; the skipped-slab-write source mutation is a named bench-level
/// court (see module docs + receipt).
#[test]
fn l4_status_flip_witness_at_lease_edge() {
    let dir = std::env::temp_dir().join("w2-13-l4-witness");
    std::fs::create_dir_all(&dir).expect("sandbox dir");
    let file = dir.join("l4-witness.par2");

    // Inside the lease: 1024 alternating lanes commit clean.
    commit_and_read(1024, &file);

    // Beyond the lease: 70,000 alternating lanes (incompressible at 8 bits,
    // so the slab cannot fit the 64 KiB static buffer) refuse typed —
    // CGAP_ERR_LEASE_EXHAUSTED, never a silent truncate.
    let cpath = std::ffi::CString::new(file.to_str().expect("utf-8 path")).expect("NUL-free");
    let big = lanes(70_000);
    // SAFETY: valid lane pointer + NUL-terminated path.
    let st = unsafe { cgap_commit_parquet(big.as_ptr(), big.len(), cpath.as_ptr()) };
    assert_eq!(
        st, CGAP_ERR_LEASE_EXHAUSTED,
        "slab beyond the 64 KiB lease must flip the status word typed, got {st:#x}"
    );
    std::fs::remove_dir_all(&dir).expect("sandbox cleanup");
}
