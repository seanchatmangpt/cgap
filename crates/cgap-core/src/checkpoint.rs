//! PRD-6 — zero-alloc stage checkpoint (prototype, v26.11.2 starDF).
//!
//! Scoping decision (dispatch fence, honest): the snapshot covers ONLY
//! zero-alloc-stage state — ring head/tail (u64 pair), receipt nonce
//! (u64) and receipt parent ([u8; 32]) — serialized into a
//! caller-provided fixed buffer. The bn3 fixpoint derived-set is OUT OF
//! SCOPE v1: it allocates by construction (measured 2,662 allocs /
//! 650,294,994 bytes over 3 batches / 2,929,939 derived rows —
//! docs/jira/v26.11.2/lanes/lane-posh-04-sentry.md:35-41, honesty
//! window) and cannot be captured into a fixed buffer without changing
//! that measured result.
//!
//! Digest = BLAKE3 via INJECTED function pointer: cgap-core keeps E=0
//! externals; the real `cgap-blake3::hash` is bound at court call sites
//! only (§3 named-exception, dev-dependency, never in the lib target).
//!
//! Design note (PENDING-design): audit/cgapprd-04-checkpoint-design.md
//! was ABSENT at write time (PRD-4 in flight); API shape follows the
//! dispatch contract verbatim. This note must be reconciled when PRD-4
//! lands (errata-only, never silent rewrite).
//!
//! Layout (little-endian, fixed offsets):
//!   [0..8)    ring head  (u64 LE)
//!   [8..16)   ring tail  (u64 LE)
//!   [16..24)  receipt nonce (u64 LE)
//!   [24..56)  receipt parent ([u8; 32])
//!   [56..88)  BLAKE3 digest over bytes [0..56)
//! Total region = 88 bytes.

/// Serialized checkpoint region length: 56 bytes state + 32 bytes digest.
pub const CHECKPOINT_REGION_LEN: usize = 88;

/// Injected digest function: input bytes -> 32-byte digest.
pub type DigestFn = fn(&[u8]) -> [u8; 32];

/// Zero-alloc-stage state captured by a checkpoint (PRD-6 scope fence).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckpointState {
    /// Ring head index (frames published).
    pub ring_head: u64,
    /// Ring tail index (frames released).
    pub ring_tail: u64,
    /// Receipt nonce.
    pub receipt_nonce: u64,
    /// Receipt parent digest (BLAKE3 of parent receipt; zero iff root).
    pub receipt_parent: [u8; 32],
}

/// Digest mismatch or malformed region detected by [`CheckpointBuf::restore`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheckpointMismatch;

impl core::fmt::Display for CheckpointMismatch {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("checkpoint region digest mismatch")
    }
}

/// Caller-provided fixed checkpoint region (88 bytes, zero-alloc).
#[derive(Clone, Copy, Debug)]
pub struct CheckpointBuf {
    bytes: [u8; CHECKPOINT_REGION_LEN],
}

impl CheckpointBuf {
    /// Fresh, all-zero, unfilled region.
    pub const fn new() -> Self {
        Self {
            bytes: [0u8; CHECKPOINT_REGION_LEN],
        }
    }

    /// Serialize `state` into the region and seal it with `digest_fn`.
    /// Zero-alloc: writes only into the fixed buffer.
    pub fn snapshot(&mut self, state: &CheckpointState, digest_fn: DigestFn) {
        self.bytes[0..8].copy_from_slice(&state.ring_head.to_le_bytes());
        self.bytes[8..16].copy_from_slice(&state.ring_tail.to_le_bytes());
        self.bytes[16..24].copy_from_slice(&state.receipt_nonce.to_le_bytes());
        self.bytes[24..56].copy_from_slice(&state.receipt_parent);
        let d = digest_fn(&self.bytes[0..56]);
        self.bytes[56..88].copy_from_slice(&d);
    }

    /// Read-only view of the raw region (digest-court + corruption
    /// court flip surface). Zero-alloc.
    pub fn bytes(&self) -> &[u8; CHECKPOINT_REGION_LEN] {
        &self.bytes
    }

    /// Mutable view of the raw region (corruption injection).
    pub fn bytes_mut(&mut self) -> &mut [u8; CHECKPOINT_REGION_LEN] {
        &mut self.bytes
    }

    /// Validate the region digest and decode the state. Typed
    /// [`CheckpointMismatch`] on any digest mismatch (corruption flip
    /// surface). Zero-alloc.
    ///
    /// Law: an all-zero UNFILLED region does NOT restore as a valid
    /// empty checkpoint — BLAKE3 of the 56 zero state bytes is nonzero,
    /// so the trailer check refuses it (no vacuum restore).
    pub fn restore(&self, digest_fn: DigestFn) -> Result<CheckpointState, CheckpointMismatch> {
        let d = digest_fn(&self.bytes[0..56]);
        if d != self.bytes[56..88] {
            return Err(CheckpointMismatch);
        }
        let head: [u8; 8] = self.bytes[0..8].try_into().expect("exact 8-B slice");
        let tail: [u8; 8] = self.bytes[8..16].try_into().expect("exact 8-B slice");
        let nonce: [u8; 8] = self.bytes[16..24].try_into().expect("exact 8-B slice");
        let parent: [u8; 32] = self.bytes[24..56].try_into().expect("exact 32-B slice");
        Ok(CheckpointState {
            ring_head: u64::from_le_bytes(head),
            ring_tail: u64::from_le_bytes(tail),
            receipt_nonce: u64::from_le_bytes(nonce),
            receipt_parent: parent,
        })
    }
}

impl Default for CheckpointBuf {
    fn default() -> Self {
        Self::new()
    }
}
