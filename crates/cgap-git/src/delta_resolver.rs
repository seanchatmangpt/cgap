//! Bounded Git delta application + iterative chain resolution (lane GIT-04).
//!
//! Implements the real Git delta format: MSB-continuation varints for the
//! target/source size header, copy opcodes (MSB set: bits 4-6 = offset
//! byte count, bits 0-3 = size byte count, size 0 => 0x10000), insert
//! opcodes (bits 0-6 = literal length), opcode 0 = reserved => typed
//! refusal.
//!
//! Reality fence (pack.rs audit): `pack.rs` and `pack_write.rs` contain
//! NO delta-application primitives (pack v2 headers of type 6/7 are
//! refused via `GitError::UnsupportedDeltaEncoding`), so this module is
//! the crate's first and only delta applier — nothing is duplicated.
//!
//! Bounds: fixed 11.6 KiB scratch (11_878 B), max chain depth 16,
//! deterministic FNV-1a cycle detection. Iterative by construction: both
//! entrypoints are single flat passes; no function in this
//! module calls itself, so the call stack never nests — each frame holds
//! a handful of `usize` locals (< 200 B), trivially inside the 16 KiB
//! call-stack invariant. Zero heap: the module never references `alloc`
//! or `std` (court-enforced below).

// court-consumed surface (CEN2 ALLOW-WITH-REASON): delta_resolver_court / soc_full_chip_court; wire consumers pending — module-level allow (uniform surface, not scattered attrs).
#![allow(dead_code)]

/// Scratch capacity: 11.6 KiB (11.6 * 1024 = 11_878.4, floored).
pub const SCRATCH_LEN: usize = 11_878;
/// Maximum delta chain depth; deeper chains are refused, never descended into.
pub const MAX_CHAIN_DEPTH: usize = 16;

/// Typed refusals from delta application / chain resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeltaError {
    TruncatedDelta,
    ReservedOpcode,
    CopyOutOfRange,
    TargetOverrun,
    TargetSizeMismatch { produced: usize, declared: usize },
    ScratchTooSmall { needed: usize },
    BadDeltaHeader,
    ChainTooDeep { depth: usize },
    DeltaCycle { at: usize },
}

impl core::fmt::Display for DeltaError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DeltaError::TruncatedDelta => {
                write!(f, "delta bytes ended mid-opcode or mid-varint")
            }
            DeltaError::ReservedOpcode => {
                write!(f, "delta opcode 0 is reserved by the format")
            }
            DeltaError::CopyOutOfRange => {
                write!(f, "copy opcode reads past the end of the base buffer")
            }
            DeltaError::TargetOverrun => {
                write!(
                    f,
                    "opcode would produce more bytes than the declared target size"
                )
            }
            DeltaError::TargetSizeMismatch { produced, declared } => write!(
                f,
                "delta produced {produced} bytes but its header declared {declared}"
            ),
            DeltaError::ScratchTooSmall { needed } => write!(
                f,
                "declared target size {needed} exceeds the fixed {SCRATCH_LEN}-byte scratch"
            ),
            DeltaError::BadDeltaHeader => {
                write!(f, "delta size varint overflows the bounded width")
            }
            DeltaError::ChainTooDeep { depth } => write!(
                f,
                "delta chain depth {depth} exceeds the bounded maximum of {MAX_CHAIN_DEPTH}"
            ),
            DeltaError::DeltaCycle { at } => write!(
                f,
                "delta at index {at} repeats an earlier delta's fingerprint: the chain is \
                 cyclical and would never reach a fixed point"
            ),
        }
    }
}

/// Caller-provided fixed scratch (11.6 KiB). No heap, ever.
#[derive(Clone)]
pub struct Scratch {
    buf: [u8; SCRATCH_LEN],
}

impl Default for Scratch {
    fn default() -> Self {
        Scratch {
            buf: [0u8; SCRATCH_LEN],
        }
    }
}

impl Scratch {
    pub const fn new() -> Self {
        Scratch {
            buf: [0u8; SCRATCH_LEN],
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8; SCRATCH_LEN] {
        &mut self.buf
    }
}

/// MSB-continuation varint (little-endian 7-bit groups), bounded width.
fn read_varint(delta: &[u8], pos: &mut usize) -> Result<usize, DeltaError> {
    let mut result: usize = 0;
    let mut shift: u32 = 0;
    loop {
        let byte = *delta.get(*pos).ok_or(DeltaError::TruncatedDelta)?;
        *pos += 1;
        result |= ((byte & 0x7F) as usize) << shift;
        if byte & 0x80 == 0 {
            return Ok(result);
        }
        shift += 7;
        if shift > 28 {
            return Err(DeltaError::BadDeltaHeader);
        }
    }
}

/// 64-bit FNV-1a fingerprint (deterministic cycle detector).
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

/// Core applier: writes the target into `out`, returns its length.
/// (Length-returning form lets the chain resolver ping-pong two fixed
/// scratches without holding invalidating borrows across iterations.)
fn apply_into(base: &[u8], delta: &[u8], out: &mut [u8; SCRATCH_LEN]) -> Result<usize, DeltaError> {
    let mut pos: usize = 0;
    let target_len = read_varint(delta, &mut pos)?;
    let _source_len = read_varint(delta, &mut pos)?;
    if target_len > SCRATCH_LEN {
        return Err(DeltaError::ScratchTooSmall { needed: target_len });
    }
    let mut produced: usize = 0;
    while pos < delta.len() {
        let op = delta[pos];
        pos += 1;
        if op & 0x80 != 0 {
            // Copy opcode: offset varint (bits 4-6 pick byte count, LSB
            // first), size varint (bits 0-3); size 0 encodes 0x10000.
            let mut offset: usize = 0;
            let mut size: usize = 0;
            // Offset byte count is bits 4-6 (3 bits); bit 7 is the
            // copy-opcode marker and must NOT be consumed as a bit flag.
            for i in 0..3usize {
                if op & (1 << (4 + i)) != 0 {
                    let b = *delta.get(pos).ok_or(DeltaError::TruncatedDelta)?;
                    pos += 1;
                    offset |= (b as usize) << (8 * i);
                }
            }
            for i in 0..4usize {
                if op & (1 << i) != 0 {
                    let b = *delta.get(pos).ok_or(DeltaError::TruncatedDelta)?;
                    pos += 1;
                    size |= (b as usize) << (8 * i);
                }
            }
            if size == 0 {
                size = 0x10000;
            }
            let end = match offset.checked_add(size) {
                Some(e) => e,
                None => return Err(DeltaError::CopyOutOfRange),
            };
            if end > base.len() {
                return Err(DeltaError::CopyOutOfRange);
            }
            if produced + size > target_len {
                return Err(DeltaError::TargetOverrun);
            }
            out[produced..produced + size].copy_from_slice(&base[offset..end]);
            produced += size;
        } else if op == 0 {
            return Err(DeltaError::ReservedOpcode);
        } else {
            // Insert opcode: bits 0-6 are a literal byte count.
            let len = op as usize;
            if pos + len > delta.len() {
                return Err(DeltaError::TruncatedDelta);
            }
            if produced + len > target_len {
                return Err(DeltaError::TargetOverrun);
            }
            out[produced..produced + len].copy_from_slice(&delta[pos..pos + len]);
            produced += len;
            pos += len;
        }
    }
    if produced != target_len {
        return Err(DeltaError::TargetSizeMismatch {
            produced,
            declared: target_len,
        });
    }
    Ok(produced)
}

/// Apply one Git delta to `base`, writing the target into `scratch`.
/// Returns the target as a borrow of the scratch.
pub fn apply_delta<'s>(
    base: &[u8],
    delta: &[u8],
    scratch: &'s mut Scratch,
) -> Result<&'s [u8], DeltaError> {
    let produced = apply_into(base, delta, scratch.as_mut_slice())?;
    Ok(&scratch.as_slice()[..produced])
}

/// Which fixed scratch currently holds the chain's newest target.
enum Side {
    External,
    A,
    B,
}

/// Resolve a delta chain iteratively: base -> d0 -> d1 -> ... -> dn.
///
/// Ping-pong: each application reads the previous target from one fixed
/// 11.6 KiB scratch and writes the next target into the other, so no
/// heap and no in-place clobbering. Depth is capped at MAX_CHAIN_DEPTH
/// (deeper => typed refusal, never recursion), and each delta's FNV-1a
/// fingerprint is compared against every earlier fingerprint (repeat =>
/// DeltaCycle). Enforcement of iterative resolution: one flat `for`
/// loop over the chain; no function in this module calls itself, so the
/// call stack never nests and stays far inside the 16 KiB invariant.
pub fn resolve_chain<'a>(
    base: &'a [u8],
    deltas: &[&[u8]],
    scratch_a: &'a mut Scratch,
    scratch_b: &'a mut Scratch,
) -> Result<&'a [u8], DeltaError> {
    if deltas.len() > MAX_CHAIN_DEPTH {
        return Err(DeltaError::ChainTooDeep {
            depth: deltas.len(),
        });
    }
    let mut seen = [0u64; MAX_CHAIN_DEPTH];
    let mut cur = Side::External;
    let mut len: usize = 0;
    for (i, delta) in deltas.iter().enumerate() {
        let fp = fnv1a(delta);
        if seen[..i].contains(&fp) {
            return Err(DeltaError::DeltaCycle { at: i });
        }
        seen[i] = fp;
        len = match cur {
            Side::External => apply_into(base, delta, scratch_a.as_mut_slice())?,
            Side::A => apply_into(
                &scratch_a.as_slice()[..len],
                delta,
                scratch_b.as_mut_slice(),
            )?,
            Side::B => apply_into(
                &scratch_b.as_slice()[..len],
                delta,
                scratch_a.as_mut_slice(),
            )?,
        };
        cur = match cur {
            Side::External | Side::B => Side::A,
            Side::A => Side::B,
        };
    }
    match cur {
        Side::External => Ok(base),
        Side::A => Ok(&scratch_a.as_slice()[..len]),
        Side::B => Ok(&scratch_b.as_slice()[..len]),
    }
}
