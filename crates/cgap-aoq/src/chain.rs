//! AoqChain — header-agnostic append-only hash chain (R-H1 ruling).
//!
//! A record hashes exactly `prev_hash ‖ epoch ‖ index ‖ payload_digest` —
//! no header fields, no flags, no variable length. The record carries no
//! header projection at all: AOQ1 header law lives in [`crate::header`] /
//! [`crate::codec`] (AB-6), composed by [`crate::session`], never by the
//! chain.
//!
//! Laws (PG-A5 + R-H1):
//! 1. GENESIS is the all-zeros head of an empty chain.
//! 2. Record i's `prev_hash` is record i-1's `digest` (or GENESIS at 0).
//!    Any parent or digest break is the opaque [`Spoliation`].
//! 3. Append-only: no public truncation, rewrite, or pop. The single
//!    mutable-record seam is `cfg(test)` [`AoqChain::links_mut`], used by
//!    unit courts to prove tamper detection fires (non-vacuity).
//! 4. Compose-with-`emit_verified`: frames emitted by
//!    [`crate::session::Session::emit_verified`] append cleanly through
//!    [`AoqChain::append_verified`]. CHAIN-IS-INTEGRITY (v3 wire law):
//!    the chain does not re-derive the wire AEAD tag (it holds no
//!    keys) — the record hashes the whole wire frame with the chain's
//!    own substrate; tamper surfaces at the chain walk.
//!
//! `no_std`, alloc-free, no wall clock. Digest substrate is the
//! caller-injected [`ChainDigest`] seam (R-K1: the crate ships no crypto).

use crate::aead::TAG_LEN;
use crate::error::AoqError;

/// Digest seam: caller-injected hash substrate (32-byte digests).
pub trait ChainDigest {
    /// Order-sensitive digest over the concatenated parts.
    fn digest(&self, parts: &[&[u8]]) -> [u8; 32];
}

/// GENESIS head digest: all zeros (32 bytes).
pub const GENESIS: [u8; 32] = [0u8; 32];

/// Digest length in bytes.
pub const DIGEST_BYTES: usize = 32;

/// Parent-prefix classification width (external-slice courts only; the
/// in-chain walk is opaque per law 2).
pub const PARENT_PREFIX_BYTES: usize = 4;

/// Pinned record size.
pub const RECORD_BYTES: usize = 120;

/// Opaque spoliation witness. No public fields, no accessors; `Debug` and
/// `Display` disclose nothing. Its existence is the only fact it carries:
/// the chain failed integrity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spoliation {
    _opaque: (),
}

impl Spoliation {
    /// The single lawful constructor (crate-internal).
    pub(crate) const fn new() -> Self {
        Self { _opaque: () }
    }
}

impl core::fmt::Display for Spoliation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Opaque by law.
        f.write_str("spoliation")
    }
}

impl core::error::Error for Spoliation {}

/// The pinned 120-byte chain record (`repr(C)`), header-agnostic (R-H1).
///
/// Layout (offset / size): `prev_hash` 0/32, `payload_digest` 32/32,
/// `digest` 64/32, `index` 96/8 (u64), `epoch` 104/2 (u16), `reserved`
/// 106/14 (zero, NOT hashed). The hash input is exactly the fixed 4-part
/// `prev_hash ‖ epoch ‖ index ‖ payload_digest`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AoqChainRecord {
    /// Record i-1's digest, or [`GENESIS`] at index 0.
    pub prev_hash: [u8; 32],
    /// Digest of the appended payload bytes (`H(payload)`).
    pub payload_digest: [u8; 32],
    /// Record digest: `H(prev_hash ‖ epoch ‖ index ‖ payload_digest)`.
    pub digest: [u8; 32],
    /// Zero-based link index (u64).
    pub index: u64,
    /// Closed epoch value (u16).
    pub epoch: u16,
    /// Zero on write, not hashed (R-H1: fixed 4-part hash input).
    pub reserved: [u8; 14],
}

// Pinned layout asserts (compile-time; build fails on drift).
const _: () = assert!(core::mem::size_of::<AoqChainRecord>() == RECORD_BYTES);
const _: () = assert!(core::mem::size_of::<AoqChainRecord>() == 120);
const _: () = assert!(core::mem::align_of::<AoqChainRecord>() == 8);
const _: () = assert!(core::mem::offset_of!(AoqChainRecord, digest) == 64);
const _: () = assert!(core::mem::offset_of!(AoqChainRecord, index) == 96);

impl AoqChainRecord {
    /// Zeroed record (GENESIS parent, empty payload digest).
    #[must_use]
    pub const fn zeroed() -> Self {
        Self {
            prev_hash: GENESIS,
            payload_digest: GENESIS,
            digest: GENESIS,
            index: 0,
            epoch: 0,
            reserved: [0u8; 14],
        }
    }
}

/// Recompute the record digest: `H(prev_hash ‖ epoch ‖ index ‖
/// payload_digest)` — exactly four parts, fixed widths (R-H1).
#[must_use]
pub fn record_digest<H: ChainDigest>(h: &H, rec: &AoqChainRecord) -> [u8; 32] {
    let mut epoch_le = [0u8; 2];
    epoch_le.copy_from_slice(&rec.epoch.to_le_bytes());
    let mut index_le = [0u8; 8];
    index_le.copy_from_slice(&rec.index.to_le_bytes());
    h.digest(&[&rec.prev_hash, &epoch_le, &index_le, &rec.payload_digest])
}

/// Verify an independent record slice. Court seam for tamper batteries over
/// mutated *copies* — the chain itself stays append-only.
///
/// Errors: parent break -> `DigestMismatch` (4-byte BE prefixes as
/// expected/got); digest break -> `DigestMismatch` (digest prefixes).
pub fn verify_records<H: ChainDigest>(h: &H, records: &[AoqChainRecord]) -> Result<(), AoqError> {
    let mut prev = GENESIS;
    for rec in records.iter() {
        if rec.prev_hash != prev {
            return Err(AoqError::DigestMismatch {
                expected: u32::from_be_bytes(four(&prev)),
                got: u32::from_be_bytes(four(&rec.prev_hash)),
            });
        }
        let d = record_digest(h, rec);
        if d != rec.digest {
            return Err(AoqError::DigestMismatch {
                expected: u32::from_be_bytes(four(&d)),
                got: u32::from_be_bytes(four(&rec.digest)),
            });
        }
        prev = rec.digest;
    }
    Ok(())
}

#[inline]
fn four(bytes: &[u8; 32]) -> [u8; 4] {
    [bytes[0], bytes[1], bytes[2], bytes[3]]
}

/// Append-only chain over a caller-provided record slab. `H` is the
/// injected digest substrate. No allocation, no wall clock.
#[derive(Debug)]
pub struct AoqChain<'a, H: ChainDigest> {
    h: H,
    slab: &'a mut [AoqChainRecord],
    len: usize,
}

impl<'a, H: ChainDigest> AoqChain<'a, H> {
    /// Bind the chain to a slab. Head starts at [`GENESIS`].
    #[must_use]
    pub fn new(h: H, slab: &'a mut [AoqChainRecord]) -> Self {
        Self { h, slab, len: 0 }
    }

    /// Number of links.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Empty chain?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Read-only view of the landed records (append-only law intact: no
    /// mutable access is exposed outside the `cfg(test)` seam).
    #[must_use]
    pub fn records(&self) -> &[AoqChainRecord] {
        &self.slab[..self.len]
    }

    /// Current head digest (GENESIS when empty).
    #[must_use]
    #[inline]
    pub fn checkpoint(&self) -> [u8; 32] {
        if self.len == 0 {
            GENESIS
        } else {
            self.slab[self.len - 1].digest
        }
    }

    /// Share the checkpoint as a resumption token.
    #[must_use]
    pub fn resumption_token(&self) -> ResumptionToken {
        ResumptionToken {
            head: self.checkpoint(),
            links: self.len as u32,
        }
    }

    /// Append payload bytes as the next record: `payload_digest = H(payload)`,
    /// `index = len`, `digest = H(prev_hash ‖ epoch ‖ index ‖
    /// payload_digest)`.
    ///
    /// Refusals are typed and leave the chain unmutated (slab full ->
    /// `SlabFull`).
    pub fn append(&mut self, epoch: u16, payload: &[u8]) -> Result<u64, AoqError> {
        let payload_digest = self.h.digest(&[payload]);
        self.link(epoch, payload_digest)
    }

    /// Write the next record for an already-computed `payload_digest`.
    ///
    /// Single slab-full check + single record-digest pass; [`AoqChain::append`]
    /// and [`AoqChain::append_verified`] both land here so a verified link
    /// costs one payload hash (the tag check), not two.
    ///
    /// Refusals are typed and leave the chain unmutated (slab full ->
    /// `SlabFull`).
    fn link(&mut self, epoch: u16, payload_digest: [u8; 32]) -> Result<u64, AoqError> {
        if self.len == self.slab.len() {
            return Err(AoqError::SlabFull {
                len: self.slab.len(),
            });
        }
        let index = self.len as u64;
        let prev = self.checkpoint();
        let mut epoch_le = [0u8; 2];
        epoch_le.copy_from_slice(&epoch.to_le_bytes());
        let mut index_le = [0u8; 8];
        index_le.copy_from_slice(&index.to_le_bytes());
        let digest = self
            .h
            .digest(&[&prev, &epoch_le, &index_le, &payload_digest]);
        self.slab[self.len] = AoqChainRecord {
            prev_hash: prev,
            payload_digest,
            digest,
            index,
            epoch,
            reserved: [0u8; 14],
        };
        self.len += 1;
        Ok(index)
    }

    /// Batch-append a run of already-computed payload digests
    /// (`payload_digests[i]` lands with `epochs[i]` at the next `epochs.len()`
    /// indices). The admitted-digest entry: callers who already paid for the
    /// payload hash (e.g. a SIMD independent batch) skip the per-frame
    /// re-hash; [`AoqChain::append`] still re-hashes per call.
    ///
    /// All-or-nothing refusal law: capacity is checked FIRST, so a batch that
    /// does not fit is refused with [`AoqError::SlabFull`] `{ len: slab.len() }`
    /// and ZERO mutation — the chain never lands a partial batch (stronger
    /// than the "unmutated past that index" floor).
    ///
    /// Length-parity refusal: unequal `epochs`/`payload_digests` counts are
    /// refused typed (`SeqGap` reused as the length-parity carrier:
    /// `expected` = epochs count, `got` = digest count) — never a partial
    /// batch, never a panic.
    pub fn append_batch_admitted(
        &mut self,
        epochs: &[u16],
        payload_digests: &[[u8; 32]],
    ) -> Result<(), AoqError> {
        if epochs.len() != payload_digests.len() {
            return Err(AoqError::SeqGap {
                expected: epochs.len() as u32,
                got: payload_digests.len() as u32,
            });
        }
        if self.len + epochs.len() > self.slab.len() {
            return Err(AoqError::SlabFull {
                len: self.slab.len(),
            });
        }
        for (i, &d) in payload_digests.iter().enumerate() {
            // Cannot refuse post-pre-check; propagated for honesty, not for
            // a reachable path.
            self.link(epochs[i], d)?;
        }
        Ok(())
    }

    /// Append a v3 wire frame (`emit_verified` output) as the next
    /// record. CHAIN-IS-INTEGRITY (v3 wire law Stage 2, 2026-11-04): the
    /// wire frame's authentication is the 16-byte AEAD tag, whose keys
    /// the chain does not hold — the chain re-derives NOTHING from the
    /// wire. The record's `payload_digest` is the chain's OWN hash over
    /// the whole wire frame; tamper detection is the chain walk
    /// ([`Self::verify_all`]/[`Self::verify_span`]) against that stored
    /// hash, not a wire-tag re-derivation.
    ///
    /// Refusals are typed and leave the chain unmutated (short frame ->
    /// `HeaderTooShort`, slab full -> `SlabFull`).
    pub fn append_verified(&mut self, frame: &[u8]) -> Result<u64, AoqError> {
        if frame.len() < crate::header::SIZE_BYTES + 5 + TAG_LEN {
            return Err(AoqError::HeaderTooShort {
                got: frame.len() as u16,
            });
        }
        self.link(0, self.h.digest(&[frame]))
    }

    /// Verify the whole chain (GENESIS parent law + per-record digests).
    pub fn verify_all(&self) -> Result<(), Spoliation> {
        self.verify_span(0, self.len)
    }

    /// Verify links `[lo, hi)`. Out-of-range bounds are refused (opaque).
    pub fn verify_span(&self, lo: usize, hi: usize) -> Result<(), Spoliation> {
        if lo > hi || hi > self.len {
            return Err(Spoliation::new());
        }
        self.span_walk(lo, hi)
    }

    /// Resume from a token: the chain must hold at least `tok.links` links
    /// and the prefix `[0, tok.links)` must verify to `tok.head`. A shorter
    /// chain or a forked prefix is [`Spoliation`] — the chain moved or was
    /// rewritten behind the token's back.
    pub fn resume(&mut self, tok: ResumptionToken) -> Result<(), Spoliation> {
        if tok.links as usize > self.len {
            return Err(Spoliation::new());
        }
        self.span_walk(0, tok.links as usize)?;
        if tok.links > 0 && self.slab[tok.links as usize - 1].digest != tok.head {
            return Err(Spoliation::new());
        }
        Ok(())
    }

    fn span_walk(&self, lo: usize, hi: usize) -> Result<(), Spoliation> {
        let mut prev = GENESIS;
        if lo > 0 {
            prev = self.slab[lo - 1].digest;
        }
        for rec in &self.slab[lo..hi] {
            if rec.prev_hash != prev || record_digest(&self.h, rec) != rec.digest {
                return Err(Spoliation::new());
            }
            prev = rec.digest;
        }
        Ok(())
    }

    /// `cfg(test)`-only mutable record access. The single seam that breaks
    /// the append-only law, so unit courts prove tamper detection fires
    /// (non-vacuity: a verify that cannot be made to fail is vacuous).
    #[cfg(test)]
    pub fn links_mut(&mut self) -> &mut [AoqChainRecord] {
        &mut self.slab[..self.len]
    }
}

/// Resumption token: head digest + link count (36 bytes, `Copy`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResumptionToken {
    /// Head digest at token time.
    pub head: [u8; 32],
    /// Link count at token time.
    pub links: u32,
}

impl ResumptionToken {
    /// Byte-encode (head then links, both little-endian).
    #[must_use]
    pub fn to_bytes(&self) -> [u8; 36] {
        let mut out = [0u8; 36];
        out[0..32].copy_from_slice(&self.head);
        out[32..36].copy_from_slice(&self.links.to_le_bytes());
        out
    }

    /// Decode a 36-byte token.
    #[must_use]
    pub fn from_bytes(bytes: &[u8; 36]) -> Option<Self> {
        let mut head = [0u8; 32];
        head.copy_from_slice(&bytes[0..32]);
        let mut links_le = [0u8; 4];
        links_le.copy_from_slice(&bytes[32..36]);
        Some(Self {
            head,
            links: u32::from_le_bytes(links_le),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic test digest: dual FNV-style mixing over 32 bytes.
    #[derive(Debug, Clone, Copy)]
    struct XorDigest;

    impl ChainDigest for XorDigest {
        fn digest(&self, parts: &[&[u8]]) -> [u8; 32] {
            let mut a: u64 = 0xcbf2_9ce4_8422_2325;
            let mut b: u64 = 0x9e37_79b9_7f4a_7c15;
            for part in parts {
                for &byte in *part {
                    a ^= byte as u64;
                    a = a.wrapping_mul(0x100_0000_01b3);
                    b = (b ^ ((byte as u64) | 1)).wrapping_mul(0x9e37_79b9_7f4a_7c15);
                }
            }
            let mut out = [0u8; 32];
            for (i, chunk) in out.chunks_exact_mut(8).enumerate() {
                let v = if i % 2 == 0 {
                    a.wrapping_add(i as u64)
                } else {
                    b.wrapping_add(i as u64)
                };
                chunk.copy_from_slice(&v.to_le_bytes());
            }
            out
        }
    }

    #[test]
    fn genesis_is_all_zeros_and_verify_all_passes_empty() {
        let mut slab = [AoqChainRecord::zeroed(); 8];
        let c = AoqChain::new(XorDigest, &mut slab);
        assert_eq!(c.checkpoint(), GENESIS);
        assert!(c.is_empty());
        assert_eq!(c.verify_all(), Ok(()));
    }

    #[test]
    fn append_then_verify_all_and_span() {
        let mut slab = [AoqChainRecord::zeroed(); 8];
        let mut c = AoqChain::new(XorDigest, &mut slab);
        for i in 0u32..5 {
            c.append(i as u16, &[i as u8; 8]).unwrap();
        }
        assert_eq!(c.len(), 5);
        assert_eq!(c.checkpoint(), c.resumption_token().head);
        assert_eq!(c.verify_all(), Ok(()));
        assert_eq!(c.verify_span(1, 3), Ok(()));
        assert!(c.verify_span(2, 99).is_err());
    }

    #[test]
    fn links_mut_tamper_is_detected_as_spoliation() {
        let mut slab = [AoqChainRecord::zeroed(); 8];
        let mut c = AoqChain::new(XorDigest, &mut slab);
        c.append(0, b"alpha").unwrap();
        c.append(0, b"beta").unwrap();
        assert_eq!(c.verify_all(), Ok(()));
        let links = c.links_mut();
        links[1].payload_digest[0] ^= 0xff;
        assert_eq!(c.verify_all(), Err(Spoliation::new()));
    }

    #[test]
    fn parent_rewrite_is_spoliation_in_chain_typed_in_verify_records() {
        let mut slab = [AoqChainRecord::zeroed(); 8];
        let mut c = AoqChain::new(XorDigest, &mut slab);
        c.append(0, b"a").unwrap();
        c.append(0, b"b").unwrap();
        let links = c.links_mut();
        links[1].prev_hash[0] ^= 0xff;
        let links_static: [AoqChainRecord; 2] = links[0..2].try_into().unwrap();
        assert_eq!(c.verify_all(), Err(Spoliation::new()));
        assert!(matches!(
            verify_records(&XorDigest, &links_static),
            Err(AoqError::DigestMismatch { expected, got }) if expected != got
        ));
    }

    #[test]
    fn record_is_pinned_120b_repr_c() {
        assert_eq!(core::mem::size_of::<AoqChainRecord>(), 120);
        assert_eq!(core::mem::align_of::<AoqChainRecord>(), 8);
        assert_eq!(AoqChainRecord::zeroed().prev_hash, GENESIS);
    }
}
