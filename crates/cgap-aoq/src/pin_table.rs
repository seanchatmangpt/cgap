//! AoQ peer identity pin-table (Q6 M2b, P-f scope).
//!
//! Insertion-ordered map `peer_key_id -> pinned 32B fingerprint`, seeded
//! from the session [`crate::session::Session::fingerprint`] image (the
//! fp/2 layout, session.rs:396-424 / 447-474). Three outcomes per
//! [`PinTable::verify`]:
//!
//! 1. [`PinOutcome::Pinned`] — fp matches the stored pin (admit).
//! 2. [`PinOutcome::TrustOnFirstUse`] — no pin for the peer: record fp,
//!    return TOFU.
//! 3. Refused — fp mismatches the stored pin: typed
//!    [`PinError::PinMismatch`], fail-closed, no mutation (the
//!    no-mutation-on-refusal law mirrors session.rs `step`).
//!
//! No alloc, no std: fixed-capacity insertion-ordered storage, typed
//! refusal on overflow (never eviction).
//!
//! Cross-language pin (Q6 M2b court): the constant pair
//! [`PIN_CROSS_LANG_TRANSCRIPT_HEX`] / [`PIN_CROSS_LANG_FP_HEX`] pins the
//! fp/2 image over a fixed 64-hex transcript. The Rust court hashes the
//! canonical image with the SAME substrate `fingerprint()` uses (a
//! [`KemSeam`] whose `kdf` is `cgap_blake3::hash` over the concat image
//! — byte-identical to the Go twin's `adapter.Sum32`), and the Go court
//! (arcus internal/arcus/flight/aoq_pin_table.go) hashes the identical
//! image with `adapter.Sum32`. Both sides must produce the identical
//! 32B fingerprint and consume it through their pin tables with
//! identical outcomes.

/// Fixed capacity (overflow is a typed refusal, never eviction).
pub const PIN_TABLE_CAP: usize = 64;
/// Pinned fingerprint width (fp/2 image digest width).
pub const PIN_FP_BYTES: usize = 32;
/// Fixed 64-hex transcript constant cited by both sides' cross-language
/// courts (same literal in arcus aoq_pin_table.go).
pub const PIN_CROSS_LANG_TRANSCRIPT_HEX: &str =
    "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
/// blake3(fp/2 image over the transcript const) as hex — produced by
/// each side's own fingerprint substrate; MUST byte-match the Go court's
/// `AoqPinCrossLangFpHex`.
pub const PIN_CROSS_LANG_FP_HEX: &str =
    "045968ab0cde8b7d80db62a8693e1c94598626593edf146f1a3308e590c1bf5c";

/// The admit-class outcomes of [`PinTable::verify`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinOutcome {
    /// fp matches the stored pin.
    Pinned,
    /// No pin existed; fp recorded.
    TrustOnFirstUse,
}

/// Typed pin-table refusals. Integer/fixed payloads only (alloc-free).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinError {
    /// Offered fingerprint mismatches the stored pin. `expected`/`got`
    /// are the byte-identity witness of the refused transcript (m2a
    /// confirm-tag-binds-transcript pattern).
    PinMismatch {
        /// Peer the refusal was raised for.
        peer_key_id: u64,
        /// The pinned fingerprint on record.
        expected: [u8; 32],
        /// The offered fingerprint that mismatched.
        got: [u8; 32],
    },
    /// TOFU insert into a full table (typed refusal, never eviction).
    TableFull {
        /// Peer that could not be recorded.
        peer_key_id: u64,
    },
}

/// Insertion-ordered map `peer_key_id -> pinned 32B fingerprint`.
#[derive(Debug)]
pub struct PinTable {
    ids: [u64; PIN_TABLE_CAP],
    fps: [[u8; 32]; PIN_TABLE_CAP],
    len: usize,
}

impl PinTable {
    /// An empty table.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            ids: [0; PIN_TABLE_CAP],
            fps: [[0; 32]; PIN_TABLE_CAP],
            len: 0,
        }
    }

    /// The one entry point. No pin: TOFU (record + return). Pin matches:
    /// [`PinOutcome::Pinned`]. Pin mismatches: [`PinError::PinMismatch`]
    /// and the table is byte-identical to before the call.
    pub fn verify(&mut self, peer_key_id: u64, fp: [u8; 32]) -> Result<PinOutcome, PinError> {
        if let Some(i) = self.index_of(peer_key_id) {
            if self.fps[i] != fp {
                return Err(PinError::PinMismatch {
                    peer_key_id,
                    expected: self.fps[i],
                    got: fp,
                });
            }
            return Ok(PinOutcome::Pinned);
        }
        if self.len >= PIN_TABLE_CAP {
            return Err(PinError::TableFull { peer_key_id });
        }
        self.ids[self.len] = peer_key_id;
        self.fps[self.len] = fp;
        self.len += 1;
        Ok(PinOutcome::TrustOnFirstUse)
    }

    /// The pin on record for a peer, if any.
    #[must_use]
    pub fn pin_for(&self, peer_key_id: u64) -> Option<[u8; 32]> {
        self.index_of(peer_key_id).map(|i| self.fps[i])
    }

    /// Number of pinned peers.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether the table is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Insertion-order position of a peer, if pinned.
    fn index_of(&self, peer_key_id: u64) -> Option<usize> {
        (0..self.len).find(|&i| self.ids[i] == peer_key_id)
    }
}
