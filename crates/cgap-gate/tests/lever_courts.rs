//! L1 court — ingress attestation escalation (07-court-corpus.md §2, L1).
//!
//! Subject: `crates/cgap-gate/src/penstock.rs:491` `ingress(bytes, len, out)`.
//! Real bytes in, A19 register values out, final state asserted exactly —
//! no mocks (pure kernel, nothing to mock).
//!
//! Anti-vacuity mutation (named): flip one byte inside the 80 B P28
//! attestation slot (OFF_ATTESTATION..OFF_ATTESTATION+80) of an otherwise
//! ADMITTED frame. The kill signal is the ingress verdict flip
//! CGAP_OK -> CGAP_ERR_P28_SPOLIATION. A gate that still admits the
//! spoliated frame exits this test nonzero (the court cannot pass on a
//! vacuous subject).

use cgap_gate::penstock::{
    ingress, CGAP_ERR_P28_SPOLIATION, CGAP_OK, HEADER_LEN, HEADER_VERSION, MAGIC, OABI_DIGEST_LEN,
    OFF_ATTESTATION, OFF_FLAGS, OFF_MAGIC, OFF_OABI_DIGEST, OFF_PAYLOAD_BYTE_LEN, OFF_VERSION,
    P28_FLAG_ROOT, P28_LICENSE_MAGIC, P28_SAFE_HARBOR_CEILING_CENTS, P28_VERSION,
};

/// Build one valid, admissible v1 frame: 128 B header + payload. ROOT
/// lineage bit, zero upstream receipt, ceiling under the safe-harbor bound.
fn admitted_frame() -> Vec<u8> {
    let payload_len = 32usize;
    let mut buf = vec![0u8; HEADER_LEN + payload_len];
    buf[OFF_MAGIC..OFF_MAGIC + 4].copy_from_slice(&MAGIC);
    buf[OFF_VERSION] = HEADER_VERSION;
    buf[OFF_FLAGS] = 0; // bits 4-7 clear (FLAGS_RESERVED_MASK law)
    buf[OFF_PAYLOAD_BYTE_LEN..OFF_PAYLOAD_BYTE_LEN + 8]
        .copy_from_slice(&(payload_len as u64).to_le_bytes());
    for b in buf[OFF_OABI_DIGEST..OFF_OABI_DIGEST + OABI_DIGEST_LEN].iter_mut() {
        *b = 0xA5;
    }
    buf[OFF_ATTESTATION..OFF_ATTESTATION + 4].copy_from_slice(&P28_LICENSE_MAGIC);
    buf[OFF_ATTESTATION + 4..OFF_ATTESTATION + 6].copy_from_slice(&P28_VERSION.to_le_bytes());
    buf[OFF_ATTESTATION + 6..OFF_ATTESTATION + 8].copy_from_slice(&P28_FLAG_ROOT.to_le_bytes());
    buf[OFF_ATTESTATION + 8..OFF_ATTESTATION + 16]
        .copy_from_slice(&P28_SAFE_HARBOR_CEILING_CENTS.to_le_bytes());
    // ROOT: upstream_receipt stays all-zero (receipt law).
    buf
}

/// L1 kill: flip one attestation byte in the ADMITTED frame; ingress must
/// flip CGAP_OK -> CGAP_ERR_P28_SPOLIATION. Also flips each remaining
/// attestation-region class (version, lineage, ceiling) so the escalation
/// is proven over the whole row-5 family, not one lucky byte.
#[test]
fn l1_attestation_byte_flip_escalates_to_spoliation() {
    // Baseline: the unmutated frame is ADMITTED (CGAP_OK), payload view
    // carved at the declared length.
    let frame = admitted_frame();
    let mut view = cgap_gate::penstock::FrameView {
        payload: &[],
        prefilter: 0,
        escalate: false,
    };
    let v = ingress(&frame, frame.len(), Some(&mut view));
    assert_eq!(v, CGAP_OK, "baseline frame must be admitted pre-mutation");
    assert_eq!(view.payload.len(), 32);

    // Kill signal: one-byte flips across the attestation slot families.
    // (offset within attestation slot, expected constant identical)
    let probes: &[(&str, usize, &[u8])] = &[
        ("att-magic", 0, b"X28S"),              // row 4
        ("att-version", 4, &[0xFF, 0xFF]),      // row 5
        ("lineage-zero", 6, &[0x00, 0x00]),     // row 5
        ("lineage-two-bits", 6, &[0x03, 0x00]), // row 5
        ("ceiling-overflow", 8, &[0xFF; 8]),    // row 5
    ];
    for (name, att_off, patch) in probes {
        let mut spoliated = admitted_frame();
        let at = OFF_ATTESTATION + att_off;
        spoliated[at..at + patch.len()].copy_from_slice(patch);
        let mut view = cgap_gate::penstock::FrameView {
            payload: &[],
            prefilter: 0,
            escalate: false,
        };
        let verdict = ingress(&spoliated, spoliated.len(), Some(&mut view));
        assert_eq!(
            verdict, CGAP_ERR_P28_SPOLIATION,
            "attestation flip {name} at slot offset {att_off} must escalate"
        );
    }
}

/// Named anti-vacuity mutation for this court file: the ceiling-law byte
/// flip (row 5, `read_u64_le(bytes, a + 8) > P28_SAFE_HARBOR_CEILING_CENTS`)
/// is the exact mutant class the W3 L1 lever rewrite must preserve. If a
/// future kernel edit drops the ceiling check, this test flips to FAIL.
#[test]
fn l1_ceiling_law_flip_is_the_named_mutant() {
    let mut f = admitted_frame();
    let at = OFF_ATTESTATION + 8;
    f[at..at + 8].copy_from_slice(&u64::MAX.to_le_bytes());
    let mut view = cgap_gate::penstock::FrameView {
        payload: &[],
        prefilter: 0,
        escalate: false,
    };
    assert_eq!(
        ingress(&f, f.len(), Some(&mut view)),
        CGAP_ERR_P28_SPOLIATION
    );
}

/// L1 two-stage escalation policy court (v26.11.2 W3-L1). Asserts final
/// state: (a) a clean ADMITTED frame mints prefilter == prefilter8(header)
/// and escalate == false; (b) a row-6 reserved-flag frame is still ADMITTED
/// (A04 "ignored on read" divergence preserved) but carries escalate ==
/// true; (c) an attestation byte flip at OFF_ATTESTATION+6 still flips
/// CGAP_OK -> CGAP_ERR_P28_SPOLIATION (no escalation rewards spoliation).
///
/// Named anti-vacuity mutation: force `view.escalate = true` unconditionally
/// in penstock.rs `ingress` (escalate-always-true injection). Mutation (a)
/// flips (false != true), so this court fails under the mutant — the flip
/// is witnessed in the W3-L1 receipt (/tmp sandbox, sha256-exact restore).
#[test]
fn l1_prefilter_escalation_policy_court() {
    let frame = admitted_frame();
    let mut view = cgap_gate::penstock::FrameView {
        payload: &[],
        prefilter: 0,
        escalate: false,
    };
    let v = ingress(&frame, frame.len(), Some(&mut view));
    assert_eq!(v, CGAP_OK, "baseline admitted");
    assert!(
        !view.escalate,
        "clean frame must not carry the escalation bit"
    );
    assert_eq!(
        view.prefilter,
        cgap_gate::penstock::prefilter8(&frame[..8]),
        "admit must mint prefilter8 over the first 8 header bytes"
    );

    // Row 6: header reserved flag bit 4 — ADMITTED, escalate == true.
    let mut r6 = admitted_frame();
    r6[OFF_FLAGS] = 0x10;
    let mut view6 = cgap_gate::penstock::FrameView {
        payload: &[],
        prefilter: 0,
        escalate: false,
    };
    assert_eq!(ingress(&r6, r6.len(), Some(&mut view6)), CGAP_OK);
    assert!(
        view6.escalate,
        "row-6 reserved-flag frame must carry escalate == true"
    );

    // Attestation flip at OFF_ATTESTATION+6 still refuses (row 5 law).
    let mut sp = admitted_frame();
    let at = OFF_ATTESTATION + 6;
    sp[at..at + 2].copy_from_slice(&0x0008u16.to_le_bytes());
    let mut view_sp = cgap_gate::penstock::FrameView {
        payload: &[],
        prefilter: 0,
        escalate: false,
    };
    assert_eq!(
        ingress(&sp, sp.len(), Some(&mut view_sp)),
        CGAP_ERR_P28_SPOLIATION,
        "attestation reserved-bit flip must still be spoliation"
    );
}
