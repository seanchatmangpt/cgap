//! Lane GIT-12 — N-head octopus merge over coordinate bitplanes.
//!
//! Merges up to [`MAX_HEADS`] (20) concurrent branch coordinate
//! bitplanes in a single pass through a 12-bit partition-radix
//! semi-join, modeled on the real `cgap-bn3::unify` radix probe
//! (`crates/cgap-bn3/src/unify.rs:838-852` — seedless payload mix
//! `h = (y ^ (y >> 16)) & 0x0FFF` over 4096 buckets). Divergence
//! disclosed: bn3 buckets a single A11 term word; here the join key is
//! the coordinate domain key `(subject, predicate)`, mixed before the
//! same seedless payload mix. No `#[path]` include was needed — the
//! radix discipline is re-derived locally because `cgap-git` is a
//! zero-external-dep published kernel (`[dependencies]` is empty and
//! stays empty; consuming `cgap-bn3` would add a path dep, which the
//! lane contract forbids via "do not edit Cargo.toml").
//!
//! Merge semantics on coordinate domains:
//! - identical triples `(s, p, o)` seen in multiple heads unify into
//!   one merged row;
//! - the same coordinate `(s, p)` with two different objects `o` is a
//!   typed domain clash ([`MergeError::DomainClash`], naming the
//!   clashing heads) — the AndonRefusal class. The [`octopus_merge_andon`]
//!   wrapper converts that refusal into a panic (process exit 101
//!   class) so the refusal cannot be silently downgraded to a value.
//!
//! Zero-alloc invariant (STATIC, instrumented): the whole merge is
//! allocation-free by construction — bucket tables live in
//! [`OctopusScratch`] (a plain `[u32; 4096]` triple, no heap), row
//! references and merged output are caller-provided slices, and the
//! crate forbids unsafe code. The court measures the delta of a global
//! counting allocator across the merge call and asserts it is 0.
//!
//! # Numerology
//!
//! The radix width is 12 bits = 4096 buckets: the same partition
//! cardinality bn3's probe uses. Bucket counts are `u32`, so a bucket
//! saturates only past 2^32 rows — out of scope for the 20-head
//! branch domain.

#![deny(missing_docs)]

use core::fmt;

/// Maximum concurrent branch heads in one octopus merge.
pub const MAX_HEADS: usize = 20;
/// Partition radix width in bits (4096 buckets, mirroring bn3 unify).
pub const RADIX_BITS: u32 = 12;
/// Number of radix partitions: `2 ^ RADIX_BITS` = 4096.
pub const RADIX_BUCKETS: usize = 1 << RADIX_BITS;
/// Mask selecting the low [`RADIX_BITS`] bits of the bucket hash.
pub const RADIX_MASK: u64 = (RADIX_BUCKETS - 1) as u64;

/// Seedless payload mix over the coordinate key, mirroring the
/// `cgap-bn3::unify::radix_bucket` discipline (unify.rs:850-852):
/// xor-fold then low-12-bit mask. No seed is involved.
#[inline]
pub fn radix_bucket(subject: u64, predicate: u64) -> usize {
    let y = subject ^ predicate.rotate_left(17);
    let y = y ^ (y >> 16);
    (y & RADIX_MASK) as usize
}

/// One branch head's coordinate bitplanes (SoA columns, row-aligned).
///
/// Construct with [`HeadInput::try_new`], which rejects ragged columns
/// with [`MergeError::HeadShape`] instead of panicking on input.
#[derive(Debug, Clone, Copy)]
pub struct HeadInput<'a> {
    /// Caller-chosen stable head identifier (surfaced verbatim in
    /// [`MergeError::DomainClash`]).
    pub head_id: usize,
    /// Subject bitplane.
    pub subject: &'a [u64],
    /// Predicate bitplane.
    pub predicate: &'a [u64],
    /// Object bitplane.
    pub object: &'a [u64],
}

impl<'a> HeadInput<'a> {
    /// Builds a head, refusing ragged SoA columns with a typed error.
    pub fn try_new(
        head_id: usize,
        subject: &'a [u64],
        predicate: &'a [u64],
        object: &'a [u64],
    ) -> Result<Self, MergeError> {
        if subject.len() != predicate.len() || subject.len() != object.len() {
            return Err(MergeError::HeadShape { head_id });
        }
        Ok(HeadInput {
            head_id,
            subject,
            predicate,
            object,
        })
    }

    /// Row count of this head (equal across all three columns).
    #[inline]
    pub fn rows(&self) -> usize {
        self.subject.len()
    }
}

/// Typed refusals from [`octopus_merge`]. The [`MergeError::DomainClash`]
/// variant is the AndonRefusal class: it names the clashing heads and
/// both conflicting object values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeError {
    /// A head's SoA columns are not row-aligned.
    HeadShape {
        /// Offending head id.
        head_id: usize,
    },
    /// More than [`MAX_HEADS`] heads were passed.
    TooManyHeads {
        /// Heads actually passed.
        got: usize,
    },
    /// Caller-provided scratch/output capacity is short.
    Capacity {
        /// Rows required (sum across heads; merged output upper bound).
        needed: usize,
        /// Slice length provided.
        available: usize,
    },
    /// The same coordinate `(s, p)` carries two different objects
    /// across (or within) heads — a domain clash. Names both clashing
    /// heads and both conflicting values.
    DomainClash {
        /// Clashing coordinate subject.
        subject: u64,
        /// Clashing coordinate predicate.
        predicate: u64,
        /// First head id holding a conflicting object.
        head_a: usize,
        /// Second head id holding a conflicting object.
        head_b: usize,
        /// Object under `(s, p)` in `head_a`.
        object_a: u64,
        /// Object under `(s, p)` in `head_b`.
        object_b: u64,
    },
}

impl fmt::Display for MergeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MergeError::HeadShape { head_id } => write!(
                f,
                "octopus merge: head {head_id} has ragged SoA columns \
                 (subject/predicate/object row counts differ)"
            ),
            MergeError::TooManyHeads { got } => write!(
                f,
                "octopus merge: {got} heads exceeds the {MAX_HEADS}-head cap"
            ),
            MergeError::Capacity { needed, available } => write!(
                f,
                "octopus merge: caller buffer holds {available} slots, merge needs {needed}"
            ),
            MergeError::DomainClash {
                subject,
                predicate,
                head_a,
                head_b,
                object_a,
                object_b,
            } => write!(
                f,
                "ANDON domain clash: coordinate (s={subject:#x}, p={predicate:#x}) carries \
                 o={object_a:#x} in head {head_a} but o={object_b:#x} in head {head_b} — \
                 heads {head_a} and {head_b} conflict; refusing the merge"
            ),
        }
    }
}

/// Caller-provided partition tables (3 x 4096 x u32 = 48 KiB of plain
/// arrays — no heap). Reset at the top of every merge call.
#[derive(Debug, Clone, Copy)]
pub struct OctopusScratch {
    bucket_count: [u32; RADIX_BUCKETS],
    bucket_off: [u32; RADIX_BUCKETS],
    emit_cursor: [u32; RADIX_BUCKETS],
    scan_cursor: [u32; RADIX_BUCKETS],
}

impl Default for OctopusScratch {
    fn default() -> Self {
        OctopusScratch {
            bucket_count: [0; RADIX_BUCKETS],
            bucket_off: [0; RADIX_BUCKETS],
            emit_cursor: [0; RADIX_BUCKETS],
            scan_cursor: [0; RADIX_BUCKETS],
        }
    }
}

impl OctopusScratch {
    /// Zeroes all partition tables (called internally per merge).
    pub fn reset(&mut self) {
        self.bucket_count = [0; RADIX_BUCKETS];
        self.bucket_off = [0; RADIX_BUCKETS];
        self.emit_cursor = [0; RADIX_BUCKETS];
        self.scan_cursor = [0; RADIX_BUCKETS];
    }
}

/// One row reference in the partitioned staging area: `(head slot,
/// row index)`. Zero-copy — the triple is re-read from the head's
/// bitplanes through this pair.
pub type RowRef = (u32, u32);

/// N-head octopus merge (single pass, 12-bit partition-radix semi-join).
///
/// Phases, all allocation-free:
/// 1. count rows per radix bucket across all heads;
/// 2. prefix-sum offsets, scatter [`RowRef`]s into caller scratch;
/// 3. walk buckets in order, dedupe/unify triples by linear scan over
///    the (small) bucket segment, emitting merged rows to the
///    caller-provided SoA output; a same-key different-object hit is a
///    typed [`MergeError::DomainClash`].
///
/// Returns the number of merged rows written to `out_*`.
pub fn octopus_merge(
    heads: &[HeadInput<'_>],
    row_refs: &mut [RowRef],
    scratch: &mut OctopusScratch,
    out_subject: &mut [u64],
    out_predicate: &mut [u64],
    out_object: &mut [u64],
) -> Result<usize, MergeError> {
    if heads.len() > MAX_HEADS {
        return Err(MergeError::TooManyHeads { got: heads.len() });
    }
    scratch.reset();
    let mut total: usize = 0;
    for head in heads {
        total += head.rows();
    }
    if total > row_refs.len() {
        return Err(MergeError::Capacity {
            needed: total,
            available: row_refs.len(),
        });
    }
    if total > out_subject.len() || total > out_predicate.len() || total > out_object.len() {
        return Err(MergeError::Capacity {
            needed: total,
            available: out_subject
                .len()
                .min(out_predicate.len())
                .min(out_object.len()),
        });
    }

    // Phase 1: bucket counts.
    for head in heads {
        for row in 0..head.rows() {
            let b = radix_bucket(head.subject[row], head.predicate[row]);
            scratch.bucket_count[b] = scratch.bucket_count[b].saturating_add(1);
        }
    }

    // Phase 2: prefix sums + scatter. bucket_off[b] is the exclusive
    // prefix; emit_cursor[b] walks forward as refs land.
    let mut running: u32 = 0;
    for b in 0..RADIX_BUCKETS {
        scratch.bucket_off[b] = running;
        scratch.emit_cursor[b] = running;
        running = running.saturating_add(scratch.bucket_count[b]);
    }
    for (slot, head) in heads.iter().enumerate() {
        for row in 0..head.rows() {
            let b = radix_bucket(head.subject[row], head.predicate[row]);
            let at = scratch.emit_cursor[b] as usize;
            row_refs[at] = (slot as u32, row as u32);
            scratch.emit_cursor[b] += 1;
        }
    }

    // Phase 3: per-bucket unify/dedupe by linear scan; emit to out.
    let mut emitted: usize = 0;
    for b in 0..RADIX_BUCKETS {
        let count = scratch.bucket_count[b] as usize;
        if count == 0 {
            continue;
        }
        let base = scratch.bucket_off[b] as usize;
        scratch.scan_cursor[b] = 0;
        for i in 0..count {
            let (slot, row) = row_refs[base + i];
            let head = &heads[slot as usize];
            let s = head.subject[row as usize];
            let p = head.predicate[row as usize];
            let o = head.object[row as usize];
            // Scan the PRIOR INPUT ROWS of this same bucket segment
            // (row_refs[base .. base + i]). Comparing against inputs —
            // not the emitted-output window — keeps the clash
            // attribution deterministic: `row_refs[base + j]` is exactly
            // the row that held `(s, p)` before row `i`, regardless of
            // how many duplicates preceded it. (The previous
            // emitted-window scan indexed `row_refs[base + j]` with an
            // OUTPUT position, so once duplicates were skipped the named
            // head_a could be whatever head sat at that input offset —
            // an order/dup-dependent misattribution, root cause of the
            // one-shot mutation_conflict_injection_flip flap.)
            let mut dup = false;
            for j in 0..i {
                let (js, jr) = row_refs[base + j];
                let jhead = &heads[js as usize];
                let es = jhead.subject[jr as usize];
                let ep = jhead.predicate[jr as usize];
                if es == s && ep == p {
                    let eo = jhead.object[jr as usize];
                    if eo == o {
                        dup = true;
                    } else {
                        return Err(MergeError::DomainClash {
                            subject: s,
                            predicate: p,
                            head_a: jhead.head_id,
                            head_b: head.head_id,
                            object_a: eo,
                            object_b: o,
                        });
                    }
                    break;
                }
            }
            if !dup {
                out_subject[emitted] = s;
                out_predicate[emitted] = p;
                out_object[emitted] = o;
                emitted += 1;
                scratch.scan_cursor[b] += 1;
            }
        }
    }
    Ok(emitted)
}

/// Andon wrapper: [`octopus_merge`] with the DomainClash refusal
/// escalated to a panic naming the clashing heads (process exit 101
/// class). A refusal must stop the line, not return a value the caller
/// can ignore.
#[cfg(feature = "std")]
pub fn octopus_merge_andon(
    heads: &[HeadInput<'_>],
    row_refs: &mut [RowRef],
    scratch: &mut OctopusScratch,
    out_subject: &mut [u64],
    out_predicate: &mut [u64],
    out_object: &mut [u64],
) -> usize {
    match octopus_merge(
        heads,
        row_refs,
        scratch,
        out_subject,
        out_predicate,
        out_object,
    ) {
        Ok(n) => n,
        Err(e) => panic!("ANDON-REFUSAL (exit 101 class): {e}"),
    }
}
