//! Deterministic tree-entry synthesis for coordinate sets (lane GIT-06).
//!
//! # Entry framing (documented divergence from Git canonical)
//!
//! Each synthesized entry is framed as:
//!
//! ```text
//! "100644 blob " (12B) | <64-char lowercase BLAKE3 hex> (64B) | '\t' (1B)
//! | <path> (variable) | '\0' (1B)
//! ```
//!
//! Git canonical tree entries carry a 20-byte SHA-1 rendered as 40 hex
//! chars and a mode-only field ("100644"). This module deliberately
//! diverges: the digest field is a 32-byte BLAKE3 digest rendered as 64
//! hex chars, and the literal prefix is "100644 blob ". This divergence
//! is stated, not faked as parity: these frames are for cgap-internal
//! synthetic trees, not for byte-level Git object interchange.
//!
//! # Digest input
//!
//! The per-entry digest is `digest(24-byte little-endian [s, p, o])`
//! where `digest` is the injected digest function. The court binds the
//! real `cgap_blake3::hash` register (`crates/cgap-blake3`); the kernel
//! itself is digest-function-injected because `Cargo.toml` is outside
//! lane ownership and `[dependencies]` must stay empty for the
//! published zero-dep kernel.
//!
//! # Namespace -> virtual directory mapping
//!
//! | Namespace  | Directory      |
//! |------------|----------------|
//! | `Facts`    | `/facts/`      |
//! | `Rules`    | `/rules/`      |
//! | `Derived`  | `/derived/`    |
//! | `Receipts` | `/receipts/`   |
//!
//! # Path encoding
//!
//! `path = <dir> <s> "-" <p> "-" <o>` in plain decimal, e.g.
//! `/facts/1-2-3`.
//!
//! # Comparator
//!
//! Canonical order is strict lexicographic byte order over the full
//! encoded path bytes (dirs sort `/derived/` < `/facts/` < `/receipts/`
//! < `/rules/`; decimal components compare as digit strings, so "10"
//! sorts before "9"). Ties (duplicate coordinates) keep equal paths;
//! the canonical-order verifier requires strictly increasing paths, so
//! callers must dedupe coordinates. The input slice is sorted in place —
//! zero heap: insertion sort over the caller's slice plus fixed
//! 80-byte stack scratch per comparison.

// court-consumed surface (CEN2 ALLOW-WITH-REASON): tree_synth_court / soc_full_chip_court; wire consumers pending — module-level allow (uniform surface, not scattered attrs).
#![allow(dead_code)]

use core::fmt;

/// Maximum entries a single synthesis pass accepts.
pub const MAX_ENTRIES: usize = 1024;

/// Virtual directory for the [`Namespace::Facts`] projection.
pub const DIR_FACTS: &[u8] = b"/facts/";
/// Virtual directory for the [`Namespace::Rules`] projection.
pub const DIR_RULES: &[u8] = b"/rules/";
/// Virtual directory for the [`Namespace::Derived`] projection.
pub const DIR_DERIVED: &[u8] = b"/derived/";
/// Virtual directory for the [`Namespace::Receipts`] projection.
pub const DIR_RECEIPTS: &[u8] = b"/receipts/";

/// Source coordinate namespace selecting the virtual directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Namespace {
    Facts,
    Rules,
    Derived,
    Receipts,
}

impl Namespace {
    /// The virtual directory prefix for this namespace.
    pub fn dir(self) -> &'static [u8] {
        match self {
            Namespace::Facts => DIR_FACTS,
            Namespace::Rules => DIR_RULES,
            Namespace::Derived => DIR_DERIVED,
            Namespace::Receipts => DIR_RECEIPTS,
        }
    }
}

/// A source coordinate `[s, p, o]` triple.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Coord {
    pub s: u64,
    pub p: u64,
    pub o: u64,
}

impl Coord {
    pub const fn new(s: u64, p: u64, o: u64) -> Self {
        Coord { s, p, o }
    }
}

/// Errors from tree synthesis and canonical-order verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeSynthError {
    /// Entry count exceeds [`MAX_ENTRIES`].
    TooManyEntries(usize),
    /// Output buffer too small for the framed, sorted entries.
    BufferOverflow { needed: usize, capacity: usize },
    /// Canonical-order verifier found a non-lexicographic pair.
    NonCanonicalOrder { at: usize },
    /// An entry frame is malformed (bad prefix, short hex, missing separators).
    MalformedEntry { at: usize },
}

impl fmt::Display for TreeSynthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TreeSynthError::TooManyEntries(n) => {
                write!(
                    f,
                    "coordinate set of {n} entries exceeds MAX_ENTRIES={MAX_ENTRIES}"
                )
            }
            TreeSynthError::BufferOverflow { needed, capacity } => {
                write!(
                    f,
                    "output buffer too small: need {needed} bytes, capacity {capacity}"
                )
            }
            TreeSynthError::NonCanonicalOrder { at } => {
                write!(f, "entry {at} breaks canonical lexicographic path order")
            }
            TreeSynthError::MalformedEntry { at } => {
                write!(f, "entry {at} is malformed")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TreeSynthError {}

/// Encode `path = <dir><s>-<p>-<o>` into fixed stack scratch.
/// Returns the filled prefix length (always <= 64).
pub fn encode_path(ns: Namespace, c: Coord, buf: &mut [u8; 64]) -> usize {
    let dir = ns.dir();
    buf[..dir.len()].copy_from_slice(dir);
    let mut n = dir.len();
    for v in [c.s, c.p, c.o] {
        if n > dir.len() {
            buf[n] = b'-';
            n += 1;
        }
        // u64 decimal is at most 20 digits.
        let mut tmp = [0u8; 20];
        let mut m = 0;
        let mut v = v;
        if v == 0 {
            tmp[0] = b'0';
            m = 1;
        }
        while v > 0 {
            tmp[m] = b'0' + (v % 10) as u8;
            v /= 10;
            m += 1;
        }
        for i in 0..m {
            buf[n + i] = tmp[m - 1 - i];
        }
        n += m;
    }
    n
}

/// Lowercase hex of a 32-byte digest (64 ASCII chars, no heap).
fn hex32(digest: &[u8; 32], out: &mut [u8; 64]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for i in 0..32 {
        out[2 * i] = HEX[(digest[i] >> 4) as usize];
        out[2 * i + 1] = HEX[(digest[i] & 0x0f) as usize];
    }
}

/// Digest input: 24-byte little-endian [s, p, o].
fn digest_input(c: Coord) -> [u8; 24] {
    let mut b = [0u8; 24];
    b[0..8].copy_from_slice(&c.s.to_le_bytes());
    b[8..16].copy_from_slice(&c.p.to_le_bytes());
    b[16..24].copy_from_slice(&c.o.to_le_bytes());
    b
}

/// Path-only byte comparison via fixed stack scratch (canonical comparator).
fn path_cmp(a: (Namespace, Coord), b: (Namespace, Coord)) -> core::cmp::Ordering {
    let mut ba = [0u8; 64];
    let mut bb = [0u8; 64];
    let na = encode_path(a.0, a.1, &mut ba);
    let nb = encode_path(b.0, b.1, &mut bb);
    ba[..na].cmp(&bb[..nb])
}

/// Deterministically synthesize sorted, framed tree entries.
///
/// Sorts `coords` in place into canonical lexicographic path order
/// (insertion sort, zero heap), then frames each entry as documented at
/// the module head, digesting each coordinate with the injected `hasher`
/// (the court binds the real `cgap_blake3::hash`). Returns the number of
/// entries written; on success `out[..written_len]` holds the framed
/// bytes (see [`frame_len`] / [`entry_at`] for zero-copy access).
///
/// The empty set is legal: returns `Ok(0)` with an untouched buffer.
pub fn synthesize<H>(
    coords: &mut [(Namespace, Coord)],
    out: &mut [u8],
    hasher: H,
) -> Result<usize, TreeSynthError>
where
    H: Fn(&[u8]) -> [u8; 32],
{
    if coords.len() > MAX_ENTRIES {
        return Err(TreeSynthError::TooManyEntries(coords.len()));
    }
    // Insertion sort by canonical path order — stable, zero heap.
    for i in 1..coords.len() {
        let mut j = i;
        while j > 0 && path_cmp(coords[j - 1], coords[j]) == core::cmp::Ordering::Greater {
            coords.swap(j - 1, j);
            j -= 1;
        }
    }
    let mut off = 0usize;
    for &(ns, c) in coords.iter() {
        let digest = hasher(&digest_input(c));
        let mut hex = [0u8; 64];
        hex32(&digest, &mut hex);
        let mut path = [0u8; 64];
        let np = encode_path(ns, c, &mut path);
        let need = 12 + 64 + 1 + np + 1;
        if off + need > out.len() {
            return Err(TreeSynthError::BufferOverflow {
                needed: off + need,
                capacity: out.len(),
            });
        }
        out[off..off + 12].copy_from_slice(b"100644 blob ");
        out[off + 12..off + 76].copy_from_slice(&hex);
        out[off + 76] = b'\t';
        out[off + 77..off + 77 + np].copy_from_slice(&path[..np]);
        out[off + 77 + np] = b'\0';
        off += need;
    }
    Ok(off)
}

/// Total framed byte length of a synthesis run (before writing).
pub fn frame_len(coords: &[(Namespace, Coord)]) -> Result<usize, TreeSynthError> {
    if coords.len() > MAX_ENTRIES {
        return Err(TreeSynthError::TooManyEntries(coords.len()));
    }
    let mut total = 0usize;
    for &(ns, c) in coords {
        let mut path = [0u8; 64];
        total += 12 + 64 + 1 + encode_path(ns, c, &mut path) + 1;
    }
    Ok(total)
}

/// Slice the `i`-th framed entry out of a synthesized buffer.
pub fn entry_at<'a>(out: &'a [u8], i: usize) -> Result<&'a [u8], TreeSynthError> {
    let mut off = 0usize;
    let mut n = 0usize;
    while off < out.len() {
        let end = entry_end(out, off).ok_or(TreeSynthError::MalformedEntry { at: n })?;
        if n == i {
            return Ok(&out[off..end]);
        }
        off = end;
        n += 1;
    }
    Err(TreeSynthError::MalformedEntry { at: i })
}

/// Slice just the path bytes of one framed entry (`<path>` between '\t' and '\0').
pub fn entry_path<'a>(entry: &'a [u8]) -> Result<&'a [u8], TreeSynthError> {
    if entry.len() < 78 || &entry[..12] != b"100644 blob " {
        return Err(TreeSynthError::MalformedEntry { at: 0 });
    }
    let tab = 76;
    if entry[tab] != b'\t' {
        return Err(TreeSynthError::MalformedEntry { at: 0 });
    }
    let nul = entry[77..]
        .iter()
        .position(|&b| b == 0)
        .ok_or(TreeSynthError::MalformedEntry { at: 0 })?;
    Ok(&entry[77..77 + nul])
}

fn entry_end(out: &[u8], off: usize) -> Option<usize> {
    if off + 78 > out.len() || &out[off..off + 12] != b"100644 blob " || out[off + 76] != b'\t' {
        return None;
    }
    let rel = out[off + 77..].iter().position(|&b| b == 0)?;
    Some(off + 77 + rel + 1)
}

/// Canonical-order verifier: walks framed entries and requires strictly
/// increasing lexicographic path bytes. A swapped entry pair is detected
/// as a typed [`TreeSynthError::NonCanonicalOrder`].
pub fn verify_canonical_order(out: &[u8]) -> Result<usize, TreeSynthError> {
    let mut off = 0usize;
    let mut n = 0usize;
    let mut prev: Option<&[u8]> = None;
    while off < out.len() {
        let end = entry_end(out, off).ok_or(TreeSynthError::MalformedEntry { at: n })?;
        let entry = &out[off..end];
        let path = entry_path(entry)?;
        if let Some(p) = prev {
            if path <= p {
                return Err(TreeSynthError::NonCanonicalOrder { at: n });
            }
        }
        prev = Some(path);
        off = end;
        n += 1;
    }
    Ok(n)
}
