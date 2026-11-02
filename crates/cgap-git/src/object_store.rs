//! no_std object ledger: 32-byte BLAKE3 content addressing over fixed-capacity storage.
//!
//! LANE GIT-05 (v26.11.2), wire-in GAP-01 (v26.11.2). Digest-function
//! injected (tree_synth precedent): the kernel depends on no external
//! hasher; the court binds the real `cgap_blake3::hash` via the
//! dev-dependency. Mounted as `pub mod object_store` in lib.rs.
//!
//! ADDRESSING LAW (see `CANONICAL_HEADER_LAW`): objects are framed as
//! `"<tag> <decimal-len>\n" ++ payload` and digested with the injected
//! hasher (courts bind `cgap_blake3::hash`). BLAKE3 content addressing,
//! NOT canonical
//! Git SHA-1 loose-object addressing (Git frames `"<tag> <len>\0"` and
//! hashes SHA-1). No Git parity is claimed.
//!
//! Layout: a fixed array of slots kept sorted by digest (Eytzinger lookup
//! index rebuilt on insert), zero heap. Lookup is a real Eytzinger
//! binary search over `eyt[1..=count]`.

use core::cmp::Ordering;

/// Fixed object capacity of [`ObjectStore`].
pub const MAX_OBJECTS: usize = 256;
/// Fixed per-object payload capacity (no heap anywhere in this module).
pub const MAX_PAYLOAD: usize = 128;

/// 32-byte BLAKE3 digest.
pub type Digest = [u8; 32];

/// Injected digest function (`tree_synth` precedent). The kernel stays
/// zero-external-dep: the court binds the real `cgap_blake3::hash`
/// (dev-dep-only) as this `fn` pointer.
pub type HashFn = fn(&[u8]) -> Digest;

/// Canonical framing law implemented by this module — an honest divergence
/// statement, not a Git-parity claim.
pub const CANONICAL_HEADER_LAW: &str =
    "git-cgap-blake3-v1: frame = \"<tag> <len>\\n\" ++ payload; digest = BLAKE3; \
     NOT canonical Git \"<tag> <len>\\0\" SHA-1 loose-object format";

/// Typed object kinds (Git-family Commit/Tree/Blob/Tag plus Blob payloads).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectKind {
    Commit,
    Tree,
    Blob,
    Tag,
}

impl ObjectKind {
    /// Framing tag, matching the Git type vocabulary (`blob`, `commit`,
    /// `tree`, `tag`) — the tag text matches Git; the digest law does not.
    pub fn tag(self) -> &'static str {
        match self {
            ObjectKind::Commit => "commit",
            ObjectKind::Tree => "tree",
            ObjectKind::Blob => "blob",
            ObjectKind::Tag => "tag",
        }
    }
}

/// Typed store errors. Never panics on input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// `MAX_OBJECTS` slots occupied.
    Full,
    /// An object with this exact digest is already stored.
    Duplicate(Digest),
    /// No object with this digest exists.
    NotFound(Digest),
    /// Recomputed digest of the framed payload does not equal the expected one.
    DigestMismatch { expected: Digest, actual: Digest },
    /// Caller-provided output buffer too small for the framed bytes.
    BufferTooSmall { needed: usize, have: usize },
    /// Payload exceeds `MAX_PAYLOAD`.
    PayloadTooLarge { len: usize, max: usize },
}

/// One ledger entry: digest, kind, payload (inline, fixed capacity).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    pub digest: Digest,
    pub kind: ObjectKind,
    pub len: u32,
    pub payload: [u8; MAX_PAYLOAD],
}

impl Slot {
    pub const EMPTY_KIND: ObjectKind = ObjectKind::Blob;

    const fn zeroed() -> Self {
        Slot {
            digest: [0u8; 32],
            kind: Slot::EMPTY_KIND,
            len: 0,
            payload: [0u8; MAX_PAYLOAD],
        }
    }
}

/// Digest of `frame(kind, payload)` under the injected hasher — the content
/// address.
pub fn digest_of(kind: ObjectKind, payload: &[u8], hasher: HashFn) -> Result<Digest, StoreError> {
    let mut buf = [0u8; MAX_PAYLOAD + 32];
    let n = write_frame(kind, payload, &mut buf)?;
    Ok(hasher(&buf[..n]))
}

/// Framed byte length: `"<tag> <len>\n" ++ payload`.
pub fn frame_len(kind: ObjectKind, payload_len: usize) -> usize {
    let mut digits = 1;
    let mut v = payload_len;
    while v >= 10 {
        v /= 10;
        digits += 1;
    }
    kind.tag().len() + 1 + digits + 1 + payload_len
}

/// Writes the framed object into `out`, returns the written length.
pub fn write_frame(kind: ObjectKind, payload: &[u8], out: &mut [u8]) -> Result<usize, StoreError> {
    if payload.len() > MAX_PAYLOAD {
        return Err(StoreError::PayloadTooLarge {
            len: payload.len(),
            max: MAX_PAYLOAD,
        });
    }
    let needed = frame_len(kind, payload.len());
    if out.len() < needed {
        return Err(StoreError::BufferTooSmall {
            needed,
            have: out.len(),
        });
    }
    let mut i = 0;
    for b in kind.tag().as_bytes() {
        out[i] = *b;
        i += 1;
    }
    out[i] = b' ';
    i += 1;
    let mut v = payload.len();
    let mut digits = [0u8; 20];
    let mut d = 0;
    loop {
        digits[d] = b'0' + (v % 10) as u8;
        d += 1;
        v /= 10;
        if v == 0 {
            break;
        }
    }
    while d > 0 {
        d -= 1;
        out[i] = digits[d];
        i += 1;
    }
    out[i] = b'\n';
    i += 1;
    out[i..i + payload.len()].copy_from_slice(payload);
    Ok(needed)
}

fn cmp_digest(a: &Digest, b: &Digest) -> Ordering {
    a.iter().cmp(b.iter())
}

/// Fixed-capacity, heap-free object ledger with an Eytzinger lookup index.
pub struct ObjectStore {
    /// Injected digest function (never hard-codes a hash register).
    hasher: HashFn,
    count: usize,
    /// Slot array, kept sorted by digest.
    slots: [Slot; MAX_OBJECTS],
    /// Eytzinger (BFS) layout of slot indices over `slots[0..count]`,
    /// 1-based: `eyt[1]` is the root (median). Slot 0 unused sentinel.
    eyt: [usize; MAX_OBJECTS + 1],
}

impl ObjectStore {
    pub fn new(hasher: HashFn) -> Self {
        ObjectStore {
            hasher,
            count: 0,
            slots: [Slot::zeroed(); MAX_OBJECTS],
            eyt: [0usize; MAX_OBJECTS + 1],
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Sorted slot slice (read-only view for courts/audits).
    pub fn slots(&self) -> &[Slot] {
        &self.slots[..self.count]
    }

    /// Mutable slot slice — court-only seam for integrity-mutation tests
    /// (corrupt stored bytes, then require `verify` to detect the flip).
    pub fn slots_mut_for_mutation_court(&mut self) -> &mut [Slot] {
        &mut self.slots[..self.count]
    }

    /// Eytzinger index view (`eyt[1..=count]` valid).
    pub fn eytzinger(&self) -> &[usize] {
        &self.eyt[1..=self.count]
    }

    fn rebuild_eytzinger(&mut self) {
        fn place(eyt: &mut [usize], k: usize, pos: &mut usize, n: usize) {
            if k <= n {
                place(eyt, 2 * k, pos, n);
                eyt[k] = *pos;
                *pos += 1;
                place(eyt, 2 * k + 1, pos, n);
            }
        }
        let mut pos = 0usize;
        place(&mut self.eyt, 1, &mut pos, self.count);
    }

    /// Real Eytzinger binary search: one branchless-style descent from the
    /// root, returns the sorted slot index of `digest`.
    pub fn find(&self, digest: &Digest) -> Result<usize, StoreError> {
        let n = self.count;
        let mut k = 1usize;
        while k <= n {
            let cmp = cmp_digest(&self.slots[self.eyt[k]].digest, digest);
            // Less-or-equal descends right (classic `2k + (a[k] <= x)` form);
            // the trailing-zero shift then lands on the candidate slot.
            k = 2 * k + if cmp != Ordering::Greater { 1 } else { 0 };
        }
        // Fall back to the in-order position of the last left-turn node.
        k >>= k.trailing_zeros() + 1;
        if k >= 1 && self.slots[self.eyt[k]].digest == *digest {
            Ok(self.eyt[k])
        } else {
            Err(StoreError::NotFound(*digest))
        }
    }

    /// Inserts a typed object; the digest is computed from the framed bytes.
    pub fn insert(&mut self, kind: ObjectKind, payload: &[u8]) -> Result<Digest, StoreError> {
        let digest = digest_of(kind, payload, self.hasher)?;
        self.insert_prehashed(kind, payload, digest)?;
        Ok(digest)
    }

    /// Inserts with a caller-supplied digest; rejects a digest that does not
    /// match the framed payload (`DigestMismatch`) — the mutation court's seam.
    pub fn insert_checked(
        &mut self,
        kind: ObjectKind,
        payload: &[u8],
        claimed: Digest,
    ) -> Result<Digest, StoreError> {
        let actual = digest_of(kind, payload, self.hasher)?;
        if actual != claimed {
            return Err(StoreError::DigestMismatch {
                expected: claimed,
                actual,
            });
        }
        self.insert_prehashed(kind, payload, actual)?;
        Ok(actual)
    }

    fn insert_prehashed(
        &mut self,
        kind: ObjectKind,
        payload: &[u8],
        digest: Digest,
    ) -> Result<(), StoreError> {
        if self.count >= MAX_OBJECTS {
            return Err(StoreError::Full);
        }
        if self.find(&digest).is_ok() {
            return Err(StoreError::Duplicate(digest));
        }
        // Sorted insertion position (plain binary search over the sorted slots).
        let mut lo = 0usize;
        let mut hi = self.count;
        while lo < hi {
            let mid = (lo + hi) / 2;
            if cmp_digest(&self.slots[mid].digest, &digest) == Ordering::Less {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        // Shift right (fixed array, bounded by MAX_OBJECTS).
        let mut i = self.count;
        while i > lo {
            self.slots[i] = self.slots[i - 1];
            i -= 1;
        }
        let mut slot = Slot::zeroed();
        slot.digest = digest;
        slot.kind = kind;
        slot.len = payload.len() as u32;
        slot.payload[..payload.len()].copy_from_slice(payload);
        self.slots[lo] = slot;
        self.count += 1;
        self.rebuild_eytzinger();
        Ok(())
    }

    /// Fetches an object by digest: `(kind, payload)`.
    pub fn get(&self, digest: &Digest) -> Result<(ObjectKind, &[u8]), StoreError> {
        let idx = self.find(digest)?;
        let slot = &self.slots[idx];
        Ok((slot.kind, &slot.payload[..slot.len as usize]))
    }

    /// Verifies stored bytes against the ledger's own digest (integrity court seam).
    pub fn verify(&self, digest: &Digest) -> Result<ObjectKind, StoreError> {
        let idx = self.find(digest)?;
        let slot = &self.slots[idx];
        let actual = digest_of(slot.kind, &slot.payload[..slot.len as usize], self.hasher)?;
        if actual != slot.digest {
            return Err(StoreError::DigestMismatch {
                expected: slot.digest,
                actual,
            });
        }
        Ok(slot.kind)
    }

    /// Standalone digest check of arbitrary (possibly perturbed) bytes against
    /// a claimed digest — typed `DigestMismatch` on any single-byte flip.
    pub fn check(
        kind: ObjectKind,
        payload: &[u8],
        claimed: Digest,
        hasher: HashFn,
    ) -> Result<(), StoreError> {
        let actual = digest_of(kind, payload, hasher)?;
        if actual != claimed {
            return Err(StoreError::DigestMismatch {
                expected: claimed,
                actual,
            });
        }
        Ok(())
    }

    /// Byte-stable ledger identity: injected hasher over the sorted slot
    /// sequence (digest ++ kind tag ++ len ++ payload). Equal ledgers built
    /// in any insertion order hash equal; any content flip flips it.
    pub fn ledger_digest(&self) -> Digest {
        let mut buf = [0u8; MAX_OBJECTS * (32 + 8 + MAX_PAYLOAD)];
        let mut n = 0usize;
        for slot in self.slots() {
            buf[n..n + 32].copy_from_slice(&slot.digest);
            n += 32;
            let tag = slot.kind.tag().as_bytes();
            buf[n..n + tag.len()].copy_from_slice(tag);
            n += tag.len();
            let le = slot.len.to_le_bytes();
            buf[n..n + 4].copy_from_slice(&le);
            n += 4;
            let pl = slot.len as usize;
            buf[n..n + pl].copy_from_slice(&slot.payload[..pl]);
            n += pl;
        }
        (self.hasher)(&buf[..n])
    }
}
