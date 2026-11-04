//! AEAD_CHACHA20_POLY1305 detached mode — RFC 8439 §2.8 on the local
//! donna Poly1305 ([`crate::poly1305`]), PG-A3 spec.
//!
//! Wire shape pinned by PG-A3:
//! - `AAD_LEN = 36` (fixed AAD width — the AoQ lease/header prefix),
//! - `TAG_LEN = 16`,
//! - [`Nonce96`] is `#[repr(C)] { epoch: u32, counter: u64 }` — 12 bytes,
//!   size-pinned by const assert; serialized little-endian as
//!   `epoch || counter`, i.e. RFC 8439's "32-bit fixed-common part"
//!   (sender id) followed by the 64-bit per-stream counter.
//! - The SOLE nonce constructor is [`Nonce96::new`], which rejects the
//!   reserved epoch `0xFFFF_FFFF` (`EpochExhausted`) — no struct literal
//!   is constructed anywhere else in this kernel (count-law court).
//! - ChaCha20 block counter 0 is the Poly1305 one-time-key block
//!   (RFC 8439 §2.6); message encryption starts at block counter 1.
//! - `mac_data = aad || pad16(aad) || ct || pad16(ct) || len(aad) u64le ||
//!   len(ct) u64le` (RFC 8439 §2.8 layout).
//!
//! `seal_detached`/`open_detached` compare tags with a folded
//! constant-time equality (all 16 bytes OR-folded, then the diff folded
//! to one bit) — no early exit, no data-dependent branching.
//!
//! Zero dependencies, no handwritten assembly, no unguarded memory access
//! (crate-level forbidden).

#![allow(dead_code)] // PG-A3 wave: surface lands ahead of its consumers.

use crate::poly1305::{self, Poly1305};

/// ChaCha20 key width: 32 bytes.
pub const KEY_LEN: usize = 32;
/// Fixed AAD width: 36 bytes (AoQ lease/header prefix).
pub const AAD_LEN: usize = 36;
/// Authenticator tag width: 16 bytes.
pub const TAG_LEN: usize = 16;
/// Nonce width: 12 bytes (`epoch: u32 || counter: u64`).
pub const NONCE_LEN: usize = 12;
/// Reserved (exhausted) epoch: `Nonce96::new` refuses it.
pub const EPOCH_RESERVED: u32 = 0xFFFF_FFFF;

const BLOCK_LEN: usize = 64; // ChaCha20 block, bytes
                             // "expa nd 3 2-by te k" — RFC 8439 §2.3 constants.
const SIGMA: [u32; 4] = [0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574];

/// Detached-open failure: recomputed tag != provided tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mismatch;

/// Reserved epoch (`0xFFFF_FFFF`) refused by the sole nonce constructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpochExhausted;

/// 96-bit nonce: 32-bit epoch (stream/sender generation) + 64-bit counter.
///
/// Size-pinned (`size_of::<Nonce96>() == 12`, `align == 4`) by const
/// asserts below — the `#[repr(C, packed(4))]` layout is ABI, load-bearing
/// for the FFI tier (raw 12-byte wire/FFI nonce: `epoch u32 || counter
/// u64`, no padding). Construction flows ONLY through [`Nonce96::new`]
/// (sole-constructor law; enforced by the count-law source scan in
/// `tests/aead_courts.rs`).
#[repr(C, packed(4))]
pub struct Nonce96 {
    /// Stream epoch; `0xFFFF_FFFF` is reserved (exhaustion sentinel).
    pub epoch: u32,
    /// Per-epoch message counter (little-endian on the wire). Offset 4 in
    /// the `repr(C)` layout; may be unaligned in memory, so this struct's
    /// trait impls are handwritten BY-VALUE (derive would borrow packed
    /// fields — a compile error by design).
    pub counter: u64,
}

// Handwritten by-value impls: derives borrow `#[repr(packed)]` fields and
// are rejected by rustc — the borrow-rejection IS the layout guard.
impl core::fmt::Debug for Nonce96 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Copy packed fields to locals first (unaligned refs are UB).
        let epoch = self.epoch;
        let counter = self.counter;
        f.debug_struct("Nonce96")
            .field("epoch", &epoch)
            .field("counter", &counter)
            .finish()
    }
}

impl Clone for Nonce96 {
    fn clone(&self) -> Self {
        Self {
            epoch: self.epoch,
            counter: self.counter,
        }
    }
}
impl Copy for Nonce96 {}

impl PartialEq for Nonce96 {
    fn eq(&self, other: &Self) -> bool {
        self.epoch == other.epoch && self.counter == other.counter
    }
}
impl Eq for Nonce96 {}

// Layout pins (PG-A3): raw memory is the 12-byte wire form — no padding.
const _: () = assert!(core::mem::size_of::<Nonce96>() == NONCE_LEN);
const _: () = assert!(core::mem::align_of::<Nonce96>() == 4);

impl Nonce96 {
    /// SOLE constructor. Refuses the reserved epoch `0xFFFF_FFFF` with
    /// [`EpochExhausted`] (boundary: `0xFFFF_FFFE` is the last lawful
    /// epoch).
    pub fn new(epoch: u32, counter: u64) -> Result<Nonce96, EpochExhausted> {
        if epoch == EPOCH_RESERVED {
            return Err(EpochExhausted);
        }
        Ok(Self { epoch, counter })
    }

    /// Wire serialization: `epoch || counter`, both little-endian.
    pub fn to_bytes(self) -> [u8; NONCE_LEN] {
        let mut out = [0u8; NONCE_LEN];
        out[0..4].copy_from_slice(&self.epoch.to_le_bytes());
        out[4..12].copy_from_slice(&self.counter.to_le_bytes());
        out
    }

    /// ChaCha20 state words 13..16 (little-endian decode of the wire bytes).
    fn nonce_words(self) -> [u32; 3] {
        let b = self.to_bytes();
        [
            u32::from_le_bytes(b[0..4].try_into().expect("4 bytes")),
            u32::from_le_bytes(b[4..8].try_into().expect("4 bytes")),
            u32::from_le_bytes(b[8..12].try_into().expect("4 bytes")),
        ]
    }
}

/// One ChaCha20 quarter round (RFC 8439 §2.3).
fn quarter_round(s: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize) {
    s[a] = s[a].wrapping_add(s[b]);
    s[d] ^= s[a];
    s[d] = s[d].rotate_left(16);
    s[c] = s[c].wrapping_add(s[d]);
    s[b] ^= s[c];
    s[b] = s[b].rotate_left(12);
    s[a] = s[a].wrapping_add(s[b]);
    s[d] ^= s[a];
    s[d] = s[d].rotate_left(8);
    s[c] = s[c].wrapping_add(s[d]);
    s[b] ^= s[c];
    s[b] = s[b].rotate_left(7);
}

/// ChaCha20 block function (RFC 8439 §2.3): 20 rounds + original-state add.
fn chacha20_block(key: &[u8; KEY_LEN], counter: u32, nonce: [u32; 3]) -> [u8; BLOCK_LEN] {
    let k = |i: usize| u32::from_le_bytes(key[i * 4..i * 4 + 4].try_into().expect("4 bytes"));
    let mut s: [u32; 16] = [
        SIGMA[0],
        SIGMA[1],
        SIGMA[2],
        SIGMA[3],
        k(0),
        k(1),
        k(2),
        k(3),
        k(4),
        k(5),
        k(6),
        k(7),
        counter,
        nonce[0],
        nonce[1],
        nonce[2],
    ];
    let original = s;

    for _ in 0..10 {
        // Column rounds.
        quarter_round(&mut s, 0, 4, 8, 12);
        quarter_round(&mut s, 1, 5, 9, 13);
        quarter_round(&mut s, 2, 6, 10, 14);
        quarter_round(&mut s, 3, 7, 11, 15);
        // Diagonal rounds.
        quarter_round(&mut s, 0, 5, 10, 15);
        quarter_round(&mut s, 1, 6, 11, 12);
        quarter_round(&mut s, 2, 7, 8, 13);
        quarter_round(&mut s, 3, 4, 9, 14);
    }

    let mut out = [0u8; BLOCK_LEN];
    for i in 0..16 {
        let w = s[i].wrapping_add(original[i]);
        out[i * 4..i * 4 + 4].copy_from_slice(&w.to_le_bytes());
    }
    out
}

/// One-time Poly1305 key from ChaCha20 block counter 0 (RFC 8439 §2.6:
/// first 32 keystream bytes of block 0).
fn poly1305_key(key: &[u8; KEY_LEN], nonce: &Nonce96) -> [u8; poly1305::KEY_LEN] {
    let block = chacha20_block(key, 0, nonce.nonce_words());
    let mut k = [0u8; poly1305::KEY_LEN];
    k.copy_from_slice(&block[..poly1305::KEY_LEN]);
    k
}

/// XOR `data` with the ChaCha20 keystream starting at block counter 1
/// (counter 0 is the Poly key block — never encrypts).
fn keystream_xor(data: &mut [u8], key: &[u8; KEY_LEN], nonce: &Nonce96) {
    let nw = nonce.nonce_words();
    let mut counter: u32 = 1;
    for chunk in data.chunks_mut(BLOCK_LEN) {
        let ks = chacha20_block(key, counter, nw);
        for (b, k) in chunk.iter_mut().zip(ks.iter()) {
            *b ^= k;
        }
        counter = counter.wrapping_add(1);
    }
}

/// Folded constant-time byte-slice equality: diff accumulates over every
/// byte (no early exit), then folds to one bit. Flat control flow.
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    debug_assert_eq!(a.len(), b.len());
    let mut diff: u8 = 0;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

/// RFC 8439 §2.8 `mac_data` layout without allocation: absorb
/// `aad || pad16(aad) || ct || pad16(ct) || len(aad) u64le || len(ct) u64le`
/// into the Poly1305 state via chunked `push` (16-byte buffer, no alloc).
fn absorb_mac_data(poly: &mut Poly1305, aad: &[u8], ct: &[u8]) {
    poly1305::push(poly, aad);
    let aad_pad = (16 - (aad.len() % 16)) % 16;
    if aad_pad > 0 {
        poly1305::push(poly, &[0u8; 16][..aad_pad]);
    }
    poly1305::push(poly, ct);
    let ct_pad = (16 - (ct.len() % 16)) % 16;
    if ct_pad > 0 {
        poly1305::push(poly, &[0u8; 16][..ct_pad]);
    }
    poly1305::push(poly, &(aad.len() as u64).to_le_bytes());
    poly1305::push(poly, &(ct.len() as u64).to_le_bytes());
}

/// Detached seal (RFC 8439 §2.8): encrypt `plaintext` into `ciphertext`
/// (equal lengths), authenticate `aad || ciphertext` into `tag`.
///
/// Panics (caller-contract, checked at entry): `ciphertext.len() !=
/// plaintext.len()`.
pub fn seal_detached(
    ciphertext: &mut [u8],
    tag: &mut [u8; TAG_LEN],
    plaintext: &[u8],
    aad: &[u8; AAD_LEN],
    key: &[u8; KEY_LEN],
    nonce: &Nonce96,
) {
    assert_eq!(
        ciphertext.len(),
        plaintext.len(),
        "detached seal: ciphertext and plaintext must have equal length"
    );

    ciphertext.copy_from_slice(plaintext);
    keystream_xor(ciphertext, key, nonce);

    let pk = poly1305_key(key, nonce);
    let mut poly = poly1305::new(&pk);
    absorb_mac_data(&mut poly, aad, ciphertext);
    *tag = poly1305::finish(poly);
}

/// Detached open (RFC 8439 §2.8): authenticate `aad || ct` with the folded
/// constant-time compare, then decrypt into `plaintext` only on a match.
/// Returns [`Mismatch`] on any tag divergence (AAD, ct, key or nonce all
/// flow into the tag, so all tamper classes land here).
pub fn open_detached(
    plaintext: &mut [u8],
    ciphertext: &[u8],
    tag: &[u8; TAG_LEN],
    aad: &[u8; AAD_LEN],
    key: &[u8; KEY_LEN],
    nonce: &Nonce96,
) -> Result<(), Mismatch> {
    assert_eq!(
        plaintext.len(),
        ciphertext.len(),
        "detached open: plaintext and ciphertext must have equal length"
    );

    let pk = poly1305_key(key, nonce);
    let mut poly = poly1305::new(&pk);
    absorb_mac_data(&mut poly, aad, ciphertext);
    let expected = poly1305::finish(poly);

    if !ct_eq(&expected, tag) {
        return Err(Mismatch);
    }

    plaintext.copy_from_slice(ciphertext);
    keystream_xor(plaintext, key, nonce);
    Ok(())
}

/// `#[doc(hidden)]` fixture seam: MAC over variable-length AAD. Exists so
/// the RFC 8439 §2.8.2 published vector (12-byte AAD) is drivable exactly;
/// the production surface pins `AAD_LEN = 36`.
#[doc(hidden)]
pub fn mac_tag_var_aad(
    aad: &[u8],
    ct: &[u8],
    key: &[u8; KEY_LEN],
    nonce: &Nonce96,
) -> [u8; TAG_LEN] {
    let pk = poly1305_key(key, nonce);
    let mut poly = poly1305::new(&pk);
    absorb_mac_data(&mut poly, aad, ct);
    poly1305::finish(poly)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonce96_size_pinned() {
        assert_eq!(core::mem::size_of::<Nonce96>(), 12);
        assert_eq!(core::mem::align_of::<Nonce96>(), 4);
    }

    #[test]
    fn sole_constructor_rejects_reserved_epoch() {
        assert_eq!(
            Nonce96::new(0xFFFF_FFFE, 0),
            Ok(Nonce96::new(0xFFFF_FFFE, 0).expect("lawful epoch"))
        );
        assert_eq!(Nonce96::new(0xFFFF_FFFF, 0), Err(EpochExhausted));
    }
}
