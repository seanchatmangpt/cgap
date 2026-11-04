//! Courts for the Q6 M2b peer identity pin-table. Real collaborators,
//! assert final state; every refusal court names its anti-vacuity
//! mutation.

use cgap_aoq::pin_table::{PinError, PinOutcome, PinTable, PIN_TABLE_CAP};
use cgap_aoq::session::{KemSeam, EK_BYTES};

const TX_HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

/// Deterministic fingerprint per index (courts only).
fn fp_at(n: u8) -> [u8; 32] {
    let mut fp = [0u8; 32];
    for (i, b) in fp.iter_mut().enumerate() {
        *b = (u16::from(n) * 3 + i as u16) as u8;
    }
    fp
}

fn hex32(fp: &[u8; 32]) -> String {
    let mut s = String::new();
    for b in fp {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Court KEM whose kdf is BLAKE3 over the concatenated parts — the same
/// substrate Go Fingerprint() uses (adapter.Sum32), so the cross-
/// language court is a true byte-match.
struct Blake3Kem;

impl KemSeam for Blake3Kem {
    fn encapsulate(
        &self,
        _ek: &[u8; EK_BYTES],
    ) -> Result<([u8; 1088], [u8; 32]), cgap_aoq::session::SessionError> {
        Ok(([0u8; 1088], [0u8; 32]))
    }
    fn decapsulate(&self, _dk: &[u8], _ct: &[u8; 1088]) -> Result<[u8; 32], cgap_aoq::session::SessionError> {
        Ok([0u8; 32])
    }
    fn kdf(&self, parts: &[&[u8]]) -> [u8; 32] {
        // Concat is the exact fp/2 image shape; blake3 over the concat
        // byte-matches Go adapter.Sum32 (test build: std/alloc allowed).
        let mut buf = std::vec::Vec::with_capacity(1216);
        for part in parts {
            buf.extend_from_slice(part);
        }
        cgap_blake3::hash(&buf)
    }
}

#[test]
fn tofu_records_and_returns() {
    let mut t = PinTable::new();
    let fp = fp_at(1);
    assert_eq!(t.verify(7, fp), Ok(PinOutcome::TrustOnFirstUse));
    assert_eq!(t.pin_for(7), Some(fp));
    assert_eq!(t.len(), 1);
}

#[test]
fn second_conn_same_fp_is_pinned() {
    let mut t = PinTable::new();
    let fp = fp_at(1);
    let _ = t.verify(7, fp);
    assert_eq!(t.verify(7, fp), Ok(PinOutcome::Pinned));
}

#[test]
fn mismatch_refused_fail_closed_no_mutation() {
    let mut t = PinTable::new();
    let good = fp_at(1);
    let _ = t.verify(7, good);
    let bad = fp_at(2);
    let err = t.verify(7, bad).unwrap_err();
    // Byte-identity of the refused transcript (m2a pattern).
    assert_eq!(
        err,
        PinError::PinMismatch { peer_key_id: 7, expected: good, got: bad }
    );
    // No mutation on refusal.
    assert_eq!(t.pin_for(7), Some(good));
    assert_eq!(t.len(), 1);
}

#[test]
fn capacity_refused_no_mutation() {
    let mut t = PinTable::new();
    for i in 0..PIN_TABLE_CAP {
        assert_eq!(t.verify(i as u64 + 1, fp_at((i % 255 + 1) as u8)), Ok(PinOutcome::TrustOnFirstUse));
    }
    let before = t.len();
    assert_eq!(
        t.verify(999, fp_at(200)),
        Err(PinError::TableFull { peer_key_id: 999 })
    );
    assert_eq!(t.len(), before, "table mutated on capacity refusal");
}

#[test]
fn cross_language_fingerprint_and_pin_outcomes() {
    let mut tx = [0u8; 32];
    for (i, b) in tx.iter_mut().enumerate() {
        *b = i as u8;
    }
    let mut sk = [0u8; 32];
    let mut rk = [0u8; 32];
    sk.fill(0xA5);
    rk.fill(0x5A);
    // Canonical fp/2 image: domain + state=Established(2) + epoch LE2=1
    // + keyID LE8=7 + ordinal LE4=0 + budget LE4=0 + transcript(32) +
    // pendingCt(1088 zeros) + sendKey(0xA5*32) + recvKey(0x5A*32).
    // Total 1216 bytes.
    let mut img = std::vec::Vec::with_capacity(1216);
    img.extend_from_slice(b"cgap-aoq/fp/2");
    img.push(2u8);
    img.extend_from_slice(&1u16.to_le_bytes());
    img.extend_from_slice(&7u64.to_le_bytes());
    img.extend_from_slice(&0u32.to_le_bytes());
    img.extend_from_slice(&0u32.to_le_bytes());
    img.extend_from_slice(&tx);
    img.extend_from_slice(&[0u8; 1088]);
    img.extend_from_slice(&sk);
    img.extend_from_slice(&rk);
    assert_eq!(img.len(), 1216);
    // The fingerprint substrate is fingerprint()'s substrate: a KemSeam
    // kdf over the image; blake3-concat byte-matches Go adapter.Sum32.
    let kem = Blake3Kem;
    let fp = kem.kdf(&[&img]);
    assert_eq!(hex32(&fp), cgap_aoq::pin_table::PIN_CROSS_LANG_FP_HEX,
        "CROSS-LANGUAGE PIN MISMATCH: rust={} want={}", hex32(&fp),
        cgap_aoq::pin_table::PIN_CROSS_LANG_FP_HEX);
    // The pin table consumes the pinned fingerprint identically: TOFU,
    // Pinned, mismatch refusal with byte-identity witness, no mutation.
    let mut t = PinTable::new();
    assert_eq!(t.verify(7, fp), Ok(PinOutcome::TrustOnFirstUse));
    assert_eq!(t.verify(7, fp), Ok(PinOutcome::Pinned));
    let mut flipped = fp;
    flipped = flipped.map(|b| !b);
    match t.verify(7, flipped) {
        Err(PinError::PinMismatch { peer_key_id: 7, expected, got }) => {
            assert_eq!(expected, fp);
            assert_eq!(got, flipped);
        }
        other => panic!("cross-lang mismatch = {other:?}, want PinMismatch"),
    }
    assert_eq!(t.pin_for(7), Some(fp));
    assert_eq!(t.len(), 1);
    // The transcript constant the Go court cites is this one.
    assert_eq!(TX_HEX, cgap_aoq::pin_table::PIN_CROSS_LANG_TRANSCRIPT_HEX);
}
