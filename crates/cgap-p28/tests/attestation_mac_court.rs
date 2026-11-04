//! RC-04 — keyed attestation MAC court (RED-18 §3.3 clone vector).
//!
//! Chicago courts over real collaborators (the keyed-BLAKE3 engine in
//! cgap-crypto, the real 80-B attestation codec). No mocks.
//!
//! Named anti-vacuity mutations (both witnessed flipping in the /tmp sandbox,
//! see the lane receipt): (1) `mac_subkey` deriving with the zero key instead
//! of the device key, (2) `verify_with_key` skipping the constant-time tag
//! compare (always Ok).

use cgap_p28::attestation::{
    attestation_mac, instrument_header_span, origin_authority_digest, verify_with_key,
    AttestationMac, MacMismatch, P28LicenseAttestation, ATTESTATION_LEN, ATTESTATION_VERSION,
    FLAG_ROOT, LICENSE_MAGIC, SAFE_HARBOR_CEILING_MAX_CENTS,
};

/// Real pinned instrument bytes from the repo root (Chicago: real artifact).
fn real_instrument_span() -> Vec<u8> {
    let file = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../LICENSE-P28-SIL.md"
    ))
    .expect("pinned instrument readable");
    instrument_header_span(&file)
        .map(|s| s.to_vec())
        .expect("canonical instrument has a `---` line")
}

fn root_att(instrument: &[u8]) -> P28LicenseAttestation {
    P28LicenseAttestation {
        license_magic: LICENSE_MAGIC,
        version: ATTESTATION_VERSION,
        attestation_flags: FLAG_ROOT,
        safe_harbor_ceiling_cents: SAFE_HARBOR_CEILING_MAX_CENTS,
        origin_authority_digest: origin_authority_digest(instrument),
        upstream_receipt: [0u8; 32],
    }
}

/// Positive control: valid key + key_id -> verify Ok; deterministic across
/// two derivations; a different device key mints a different tag.
#[test]
fn valid_key_admits_and_is_deterministic() {
    let span = real_instrument_span();
    let att = root_att(&span);
    let key = [0x5Au8; 32];
    let key_id = [7u8; 16];

    let m1 = attestation_mac(&key, key_id, &att).expect("mac computes");
    let m2 = attestation_mac(&key, key_id, &att).expect("mac computes");
    assert_eq!(m1, m2, "derivation is deterministic");
    assert_ne!(m1.tag, [0u8; 32]);

    verify_with_key(&key, key_id, &att, &m1).expect("valid key must verify Ok");

    let other_key = [0xA5u8; 32];
    let m3 = attestation_mac(&other_key, key_id, &att).expect("mac computes");
    assert_ne!(m1.tag, m3.tag, "different keys must mint different tags");
    verify_with_key(&other_key, key_id, &att, &m3).expect("other key verifies its own tag");
    assert_eq!(
        verify_with_key(&key, key_id, &att, &m3),
        Err(MacMismatch),
        "key/keyed-tag pairing is strict"
    );
}
/// Clone-court (RED-18 §3.3): an attacker with the flash image (attestation
/// bytes + key_id) but a WRONG key cannot mint a valid tag; and a cloned
/// attestation body with any single tampered byte is a typed refusal.
/// Anti-vacuity: the full 640-bit single-bit mutation sweep must ALL refuse.
#[test]
fn tampered_and_cloned_refuse_typed() {
    let span = real_instrument_span();
    let att = root_att(&span);
    let key = [0x11u8; 32];
    let key_id = [0x22u8; 16];
    let mac = attestation_mac(&key, key_id, &att).expect("mac computes");

    // Cloned (image+attestation, wrong key): refusal.
    let cloned_key = [0x33u8; 32];
    assert_eq!(
        verify_with_key(&cloned_key, key_id, &att, &mac),
        Err(MacMismatch),
        "att+mac cloned under a different device key must refuse"
    );

    // Wrong key_id selector: refusal (key store resolves by key_id).
    assert_eq!(
        verify_with_key(&key, [9u8; 16], &att, &mac),
        Err(MacMismatch),
        "wrong key_id must refuse before any tag work"
    );

    // Tampered body: full 640 single-bit sweep over the 80-B record — every
    // mutation must refuse, count pinned (anti-vacuity).
    let good = att.to_bytes();
    let mut mutations_run = 0usize;
    for i in 0..ATTESTATION_LEN {
        for j in 0..8u8 {
            let mut mutated = good;
            mutated[i] ^= 1 << j;
            let parsed = P28LicenseAttestation::from_bytes(&mutated).expect("80 B parses");
            mutations_run += 1;
            assert_eq!(
                verify_with_key(&key, key_id, &parsed, &mac),
                Err(MacMismatch),
                "bit {j} of byte {i}: tampered body must refuse"
            );
        }
    }
    assert_eq!(
        mutations_run, 640,
        "all 640 mutations must run (anti-vacuity)"
    );

    // MAC wire tampering: any single-bit flip of the 48-B MAC wire also
    // refuses (tag or key_id).
    let wire = mac.to_bytes();
    let mut mac_mutations = 0usize;
    for i in 0..wire.len() {
        for j in 0..8u8 {
            let mut mw = wire;
            mw[i] ^= 1 << j;
            let parsed = AttestationMac::from_bytes(&mw).expect("48 B parses");
            mac_mutations += 1;
            assert_eq!(
                verify_with_key(&key, key_id, &att, &parsed),
                Err(MacMismatch),
                "MAC wire bit {j} of byte {i} must refuse"
            );
        }
    }
    assert_eq!(mac_mutations, 384, "all 384 MAC-wire mutations must run");
}

/// Key commitment never on disk: the 48-B MAC wire form contains neither the
/// device key nor the derived subkey as any 32-B window, and the key_id is a
/// selector only (present, but the key is absent).
#[test]
fn key_never_in_mac_wire() {
    let span = real_instrument_span();
    let att = root_att(&span);
    let key: [u8; 32] = core::array::from_fn(|i| i as u8);
    let key_id = [0xEEu8; 16];
    let mac = attestation_mac(&key, key_id, &att).expect("mac computes");
    let wire = mac.to_bytes();
    assert_eq!(wire.len(), 48);
    for w in wire.windows(32) {
        assert_ne!(w, &key[..], "device key must never appear in the MAC wire");
    }
    assert_eq!(
        AttestationMac::from_bytes(&wire).expect("round trip"),
        mac,
        "48-B wire round trip"
    );
    assert_eq!(
        AttestationMac::from_bytes(&wire[..47]),
        Err(MacMismatch),
        "truncated MAC wire fails typed"
    );
}

/// Tag depends on every attestation field (field-level discriminants, upper
/// bound on the sweep above): swapping any two distinct field values changes
/// the tag — the tag is a function of the whole 80-B body, not a subset.
#[test]
fn tag_binds_whole_body() {
    let span = real_instrument_span();
    let att = root_att(&span);
    let key = [0x44u8; 32];
    let key_id = [0x55u8; 16];
    let base = attestation_mac(&key, key_id, &att).expect("mac computes");

    // Ceiling field change (attestation still intrinsically valid).
    let mut alt = att;
    alt.safe_harbor_ceiling_cents = 1;
    let alt_mac = attestation_mac(&key, key_id, &alt).expect("mac computes");
    assert_ne!(base.tag, alt_mac.tag);
    verify_with_key(&key, key_id, &alt, &alt_mac).expect("its own mac verifies");
    assert_eq!(verify_with_key(&key, key_id, &alt, &base), Err(MacMismatch));
}
