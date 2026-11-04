//! aoq_bridge — the host crypto bridge for the AoQ seam (AB-5 / P2-Q1
//! resolution lane).
//!
//! Dep-edge law: `cgap-ffi -> {cgap-aoq, cgap-crypto}`. `cgap-aoq` ships no
//! crypto (R-K1); the real ML-KEM-768 + AEAD bodies live in cgap-crypto and
//! are wired to the seam HERE, at this layer — never inside cgap-aoq.
//!
//! Seams provided:
//! - `BridgeKem`: a `cgap_aoq::session::KemSeam` over the real ML-KEM-768
//!   (`cgap_crypto::kem`) with OS entropy wired at this layer.
//! - `aead_seal` / `aead_open`: the AEAD seam surface over
//!   `cgap_crypto::aead` (RFC 8439 detached seal/open, AAD_LEN = 36,
//!   TAG_LEN = 16), keyed by a domain-separated derivation from the session
//!   shared secret.
//! - `OsEntropy`: the OS randomness seam for the sidecar — fail-closed
//!   `os_fill` /dev/urandom fill with typed refusals, no RNG crate dep.
//!
//! DISCLOSED DEVIATION (CG8 supersession): cgap-crypto's
//! `keyed_digest_parts` was not landed at this lane's edit time (grep:
//! zero matches outside cgap-crypto's own tests), so the KDF substrate is
//! plain `keyed_digest` over a u64-LE length-prefixed concatenation of the
//! parts. When CG8 lands `keyed_digest_parts`, THIS module is the
//! superseded surface: swap the `bridge_kdf` body, keep the trait impl.
//!
//! HOST-SURFACE ONLY: `std` (`std::fs`, `Vec`) is used, so the module is
//! `cfg(not(target_os = "none"))`-gated in `lib.rs` exactly like
//! `aoq_session` (the XIP staticlib stays allocator/std-free).

use core::error::Error;
use core::fmt;

use cgap_aoq::session::{KemSeam, SessionError, CT_BYTES, EK_BYTES, SS_BYTES};
use cgap_crypto::aead::{self, Nonce96};
use cgap_crypto::chatman::keyed_digest;
use cgap_crypto::kem;

const _: () = {
    assert!(EK_BYTES == kem::EK_LEN, "ek width mismatch");
    assert!(CT_BYTES == kem::CT_LEN, "ct width mismatch");
    assert!(SS_BYTES == kem::SS_LEN, "ss width mismatch");
};

// ---------------------------------------------------------------------------
// OS randomness seam
// ---------------------------------------------------------------------------

/// Why an OS entropy fill refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntropyFault {
    /// `/dev/urandom` could not be opened.
    OpenFailed,
    /// A read errored or returned a zero-byte read mid-fill.
    ShortRead,
}

impl fmt::Display for EntropyFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::OpenFailed => f.write_str("os entropy: /dev/urandom open failed"),
            Self::ShortRead => f.write_str("os entropy: short read from /dev/urandom"),
        }
    }
}

impl Error for EntropyFault {}

/// Fail-closed OS entropy fill: loops until `buf` is full from
/// `/dev/urandom`; any open fault, error, or zero-byte mid-fill read is a
/// typed [`EntropyFault`] — never a fallback, never zeros.
pub fn os_fill(buf: &mut [u8]) -> Result<(), EntropyFault> {
    use std::fs::File;
    use std::io::Read;

    let mut f = File::open("/dev/urandom").map_err(|_| EntropyFault::OpenFailed)?;
    let mut got = 0usize;
    while got < buf.len() {
        let n = f
            .read(&mut buf[got..])
            .map_err(|_| EntropyFault::ShortRead)?;
        if n == 0 {
            return Err(EntropyFault::ShortRead);
        }
        got += n;
    }
    Ok(())
}

/// The OS randomness seam object (the sidecar injects this, not a stub).
///
/// Zero-sized; every draw is a fresh fail-closed [`os_fill`] so no entropy
/// state is cached or reused across draws.
#[derive(Debug, Clone, Copy, Default)]
pub struct OsEntropy;

impl OsEntropy {
    /// Fill `buf` from the OS entropy source (fail-closed).
    ///
    /// # Errors
    /// [`EntropyFault`] on any open/read fault.
    pub fn fill(&self, buf: &mut [u8]) -> Result<(), EntropyFault> {
        os_fill(buf)
    }

    /// Fresh 32 random bytes.
    ///
    /// # Errors
    /// [`EntropyFault`] on any open/read fault.
    pub fn bytes32(&self) -> Result<[u8; 32], EntropyFault> {
        let mut b = [0u8; 32];
        self.fill(&mut b)?;
        Ok(b)
    }
}

// ---------------------------------------------------------------------------
// KDF: chatman keyed_digest over a length-prefixed concat (CG8-superseded)
// ---------------------------------------------------------------------------

/// Domain-separation label baked into every bridge KDF evaluation.
const KDF_LABEL: &[u8] = b"cgap-ffi/aoq-bridge/kdf/v1";

/// Order-sensitive 32-byte bridge KDF over `parts`:
/// `keyed_digest(key, LABEL || sum(u64le(len_i) || part_i))`.
///
/// The length prefixes make the concatenation unambiguous (order-sensitive,
/// no cross-boundary collisions); the label binds every evaluation to this
/// bridge's domain. See the module-level CG8 supersession disclosure.
///
/// # Panics
/// `keyed_digest` refuses only on length-accounting overflow of its own
/// internal bound, which is impossible for any host-bounded input here;
/// `expect` is the fail-closed surface for that impossible class (a silent
/// deterministic fallback would launder the failure).
fn bridge_kdf(key: &[u8; 32], parts: &[&[u8]]) -> [u8; 32] {
    // HOST-SURFACE: alloc is lawful in this host-gated module.
    let cap = KDF_LABEL.len() + parts.iter().map(|p| 8 + p.len()).sum::<usize>();
    let mut buf = Vec::with_capacity(cap);
    buf.extend_from_slice(KDF_LABEL);
    for p in parts {
        buf.extend_from_slice(&(p.len() as u64).to_le_bytes());
        buf.extend_from_slice(p);
    }
    keyed_digest(key, &buf)
        .expect("keyed_digest length-overflow refusal is impossible for bounded host input")
}

// ---------------------------------------------------------------------------
// Bridge KEM seam
// ---------------------------------------------------------------------------

/// Real ML-KEM-768 `KemSeam` over `cgap_crypto::kem`, OS entropy wired at
/// this layer, Chatman KDF substrate.
///
/// Construct via `BridgeKem::new` with a caller-supplied 32-byte KDF domain
/// key (the sidecar pins one; courts pin their own).
pub struct BridgeKem {
    kdf_key: [u8; 32],
    entropy: OsEntropy,
}

impl BridgeKem {
    /// Construct with a caller-supplied KDF domain key and the OS entropy
    /// seam.
    #[must_use]
    pub const fn new(kdf_key: [u8; 32], entropy: OsEntropy) -> Self {
        Self { kdf_key, entropy }
    }

    /// Deterministic ML-KEM-768 keygen with OS entropy for `d` and `z`:
    /// returns `(ek, dk)`.
    ///
    /// # Errors
    /// [`EntropyFault`] on any entropy fault.
    pub fn keypair(&self) -> Result<([u8; EK_BYTES], [u8; kem::DK_LEN]), EntropyFault> {
        let d = self.entropy.bytes32()?;
        let z = self.entropy.bytes32()?;
        let mut ek = [0u8; EK_BYTES];
        let mut dk = [0u8; kem::DK_LEN];
        kem::keygen(&d, &z, &mut ek, &mut dk);
        Ok((ek, dk))
    }

    /// One fresh encapsulation to `ek`: OS-entropy `m`, real ML-KEM-768
    /// `encaps`, `(ct, ss)`.
    ///
    /// # Errors
    /// [`EntropyFault`] on any entropy fault (inside the trait surface the
    /// same fault surfaces as `SessionError::KemRefused { op: 0 }`).
    pub fn encapsulate_os(
        &self,
        ek: &[u8; EK_BYTES],
    ) -> Result<([u8; CT_BYTES], [u8; SS_BYTES]), EntropyFault> {
        let m = self.entropy.bytes32()?;
        let mut ct = [0u8; CT_BYTES];
        let mut ss = [0u8; SS_BYTES];
        kem::encaps(ek, &m, &mut ct, &mut ss);
        Ok((ct, ss))
    }
}

impl KemSeam for BridgeKem {
    fn encapsulate(
        &self,
        ek: &[u8; EK_BYTES],
    ) -> Result<([u8; CT_BYTES], [u8; SS_BYTES]), SessionError> {
        // Entropy fault inside the trait surface: typed refusal, op 0
        // (encapsulate) per the SessionError::KemRefused op table.
        self.encapsulate_os(ek)
            .map_err(|_| SessionError::KemRefused { op: 0 })
    }

    fn decapsulate(&self, dk: &[u8], ct: &[u8; CT_BYTES]) -> Result<[u8; SS_BYTES], SessionError> {
        let dk: &[u8; kem::DK_LEN] = dk
            .try_into()
            .map_err(|_| SessionError::KemRefused { op: 1 })?;
        Ok(kem::decaps(dk, ct))
    }

    fn kdf(&self, parts: &[&[u8]]) -> [u8; 32] {
        bridge_kdf(&self.kdf_key, parts)
    }
}

// ---------------------------------------------------------------------------
// AEAD seam surface (cgap-aoq's aead facade is const-refusal; bridge here)
// ---------------------------------------------------------------------------

/// Why an AEAD bridge operation refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AeadFault {
    /// Output buffer mis-sized vs the plaintext/ciphertext length.
    LengthMismatch {
        /// The lawful length.
        expected: usize,
        /// The presented length.
        got: usize,
    },
    /// Tag mismatch on open (AAD/ct/key/nonce tamper class).
    Mismatch,
}

impl fmt::Display for AeadFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::LengthMismatch { expected, got } => {
                write!(f, "aead length mismatch: expected {expected}B, got {got}B")
            }
            Self::Mismatch => f.write_str("aead tag mismatch"),
        }
    }
}

impl Error for AeadFault {}

/// Domain-separation label for the AEAD key derivation from a session
/// shared secret.
const AEAD_KEY_LABEL: &[u8] = b"cgap-ffi/aoq-bridge/aead/v1";

/// Derive the AEAD key from a session shared secret (domain-separated
/// through the bridge KDF).
#[must_use]
pub fn aead_key_from_ss(kdf_key: &[u8; 32], ss: &[u8; 32]) -> [u8; 32] {
    bridge_kdf(kdf_key, &[AEAD_KEY_LABEL, ss])
}

/// Detached AEAD seal over the real RFC 8439 body (`cgap_crypto::aead`).
///
/// `ciphertext` must be exactly `plaintext.len()`; `aad` is the pinned
/// 36-byte lease/header prefix (`cgap_crypto::aead::AAD_LEN`).
///
/// # Errors
/// [`AeadFault::LengthMismatch`] on a mis-sized `ciphertext` (checked here
/// so the internal seal contract surfaces as a typed refusal, not a panic).
pub fn aead_seal(
    key: &[u8; 32],
    nonce: &Nonce96,
    aad: &[u8; aead::AAD_LEN],
    plaintext: &[u8],
    ciphertext: &mut [u8],
) -> Result<[u8; aead::TAG_LEN], AeadFault> {
    if ciphertext.len() != plaintext.len() {
        return Err(AeadFault::LengthMismatch {
            expected: plaintext.len(),
            got: ciphertext.len(),
        });
    }
    let mut tag = [0u8; aead::TAG_LEN];
    aead::seal_detached(ciphertext, &mut tag, plaintext, aad, key, nonce);
    Ok(tag)
}

/// Detached AEAD open over the real RFC 8439 body (`cgap_crypto::aead`).
/// `tag` is the 16-byte tag `aead_seal` returned.
///
/// # Errors
/// [`AeadFault::LengthMismatch`] on a mis-sized `plaintext`;
/// [`AeadFault::Mismatch`] on any tag divergence.
pub fn aead_open(
    key: &[u8; 32],
    nonce: &Nonce96,
    aad: &[u8; aead::AAD_LEN],
    tag: &[u8; aead::TAG_LEN],
    ciphertext: &[u8],
    plaintext: &mut [u8],
) -> Result<(), AeadFault> {
    if plaintext.len() != ciphertext.len() {
        return Err(AeadFault::LengthMismatch {
            expected: ciphertext.len(),
            got: plaintext.len(),
        });
    }
    aead::open_detached(plaintext, ciphertext, tag, aad, key, nonce)
        .map_err(|aead::Mismatch| AeadFault::Mismatch)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pinned court domain key (courts pin their own; the sidecar pins its
    /// own).
    const COURT_KEY: [u8; 32] = [7u8; 32];

    #[test]
    fn kdf_is_order_sensitive_and_unambiguous() {
        let a = bridge_kdf(&COURT_KEY, &[b"ab", b"c"]);
        let b = bridge_kdf(&COURT_KEY, &[b"a", b"bc"]);
        assert_ne!(a, b, "length prefixes must keep part boundaries load-bearing");
        assert_eq!(a, bridge_kdf(&COURT_KEY, &[b"ab", b"c"]), "deterministic");
        assert_ne!(
            bridge_kdf(&[8u8; 32], &[b"ab", b"c"]),
            a,
            "the KDF key binds the digest"
        );
    }

    #[test]
    fn kdf_handles_empty_and_multi_part() {
        let e = bridge_kdf(&COURT_KEY, &[b""]);
        let multi = bridge_kdf(&COURT_KEY, &[b"", b"x", &[0u8; 33], &[0xFF; 9]]);
        assert_ne!(e, multi);
    }

    #[test]
    fn os_entropy_fills_and_differs() {
        let ent = OsEntropy;
        let mut a = [0u8; 32];
        ent.fill(&mut a).expect("os entropy");
        let b = ent.bytes32().expect("os entropy");
        assert_ne!(a, [0u8; 32], "zeros would mean a silent fallback");
        assert_ne!(a, b, "two fresh 32-byte draws differ");
    }

    #[test]
    fn keypair_roundtrip_via_bridge_seam() {
        let bridge = BridgeKem::new(COURT_KEY, OsEntropy);
        let (ek, dk) = bridge.keypair().expect("entropy");
        let (ct, ss_enc) = bridge.encapsulate_os(&ek).expect("entropy");
        let ss_dec = bridge.decapsulate(&dk, &ct).expect("decaps");
        assert_eq!(ss_enc, ss_dec, "ML-KEM-768 shared secrets must match");
        assert_ne!(ss_enc, [0u8; 32]);
    }

    #[test]
    fn decapsulate_refuses_wrong_dk_width() {
        let bridge = BridgeKem::new(COURT_KEY, OsEntropy);
        let err = bridge
            .decapsulate(&[0u8; 16], &[0u8; CT_BYTES])
            .expect_err("width refusal");
        assert_eq!(err, SessionError::KemRefused { op: 1 });
    }

    #[test]
    fn aead_seal_open_roundtrip_and_tamper() {
        let bridge = BridgeKem::new(COURT_KEY, OsEntropy);
        let (ek, _dk) = bridge.keypair().expect("entropy");
        let (_ct, ss) = bridge.encapsulate_os(&ek).expect("entropy");
        let key = aead_key_from_ss(&COURT_KEY, &ss);
        let nonce = Nonce96::new(1, 0).expect("lawful epoch");
        let aad = [9u8; aead::AAD_LEN];
        let pt = [0xA5u8; 71];
        let mut ct = [0u8; 71];
        let tag = aead_seal(&key, &nonce, &aad, &pt, &mut ct).expect("seal");
        let mut rt = [0u8; 71];
        aead_open(&key, &nonce, &aad, &tag, &ct, &mut rt).expect("open");
        assert_eq!(rt, pt);
        let mut bad_ct = ct;
        bad_ct[0] ^= 1;
        assert_eq!(
            aead_open(&key, &nonce, &aad, &tag, &bad_ct, &mut rt),
            Err(AeadFault::Mismatch),
            "flipped ct byte must flip the court"
        );
        let mut bad_aad = aad;
        bad_aad[3] ^= 0x80;
        assert_eq!(
            aead_open(&key, &nonce, &bad_aad, &tag, &ct, &mut rt),
            Err(AeadFault::Mismatch),
            "flipped AAD byte must flip the court"
        );
    }

    #[test]
    fn aead_refuses_mis_sized_buffers() {
        let key = [1u8; 32];
        let nonce = Nonce96::new(1, 0).expect("lawful epoch");
        let aad = [0u8; aead::AAD_LEN];
        let pt = [0u8; 16];
        let mut ct = [0u8; 15];
        assert!(matches!(
            aead_seal(&key, &nonce, &aad, &pt, &mut ct),
            Err(AeadFault::LengthMismatch { .. })
        ));
        let mut pt_out = [0u8; 17];
        assert!(matches!(
            aead_open(&key, &nonce, &aad, &[0u8; 16], &pt, &mut pt_out),
            Err(AeadFault::LengthMismatch { .. })
        ));
    }
}
