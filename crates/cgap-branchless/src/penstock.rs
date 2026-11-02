//! Penstock hot-path primitives ported from the shipped gate arithmetic —
//! lane REP5 (production wiring of cgap-branchless).
//!
//! Two shipped gate sites have branchless counterparts here:
//!
//! * `prefilter8` (`crates/cgap-gate/src/penstock.rs:677-686`, sha256-pinned
//!   2026-11-02) — L1 escalation prefilter: one unaligned LE u64 load of the
//!   first 8 header bytes, XOR-folded and mixed with one wrapping multiply.
//!   Exposed both byte-level (`penstock_prefilter8`) and word-level
//!   (`penstock_prefilter8_mix`).
//! * `is_zero_at` (`crates/cgap-gate/src/penstock.rs:651-660`,
//!   sha256-pinned 2026-11-02) — panic-free all-zero check over `len`
//!   bytes. Here as `swar_all_zero_vl`, a SWAR chunked all-zero detect:
//!   per-word nonzero per-byte detect `((v & !HI) + !HI) | v`, bit 7 set
//!   iff any byte nonzero, no cross-byte borrow (same borrow-free form the
//!   `swar` module documents). The shipped form early-exits on the first
//!   nonzero byte; this form early-exits on the first nonzero 8-byte chunk
//!   — identical RESULT for every input, different exit granularity, so
//!   admission decisions are byte-identical by construction.
//!
//! Seed hygiene: the mix constant is NOT the frozen seeds (SplitMix64
//! multiplier `0xBF58_476D_1CE4_E5B9` / corpus constant
//! `0xA20C_0FFE_E5A7_E000`) and never will be — a conflation would break
//! byte-identical replay.
#![forbid(unsafe_code)]

/// Shipped mix constant (`penstock.rs:685`, pinned 2026-11-02). Arbitrary
/// odd constant; collision resistance is not claimed — the escalated stage
/// owns that.
pub const PENSTOCK_PREFILTER_MIX: u64 = 0x2544_5491_4F6C_DD1D;

/// Word-level L1 prefilter mix: `(w ^ (w >> 7)) * MIX` (all wrapping),
/// exact shipped formula (`penstock.rs:685`).
#[inline]
pub const fn penstock_prefilter8_mix(w: u64) -> u64 {
    (w ^ (w >> 7)).wrapping_mul(PENSTOCK_PREFILTER_MIX)
}

/// Byte-level L1 prefilter: one LE u64 load of `bytes[0..8]`, then
/// [`penstock_prefilter8_mix`]. Caller proves `bytes.len() >= 8` (inside
/// `ingress` that proof is the row-2 magic compare). Reads nothing past
/// byte 8.
#[inline]
pub fn penstock_prefilter8(bytes: &[u8]) -> u64 {
    penstock_prefilter8_mix(u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ]))
}

const HI: u64 = 0x8080_8080_8080_8080;
const NOT_HI: u64 = 0x7F7F_7F7F_7F7F_7F7F;

/// SWAR all-zero detect over a byte slice, matching the shipped
/// `is_zero_at` semantics exactly: `true` iff every byte is `0x00`
/// (shipped early-exits on the first nonzero BYTE; this early-exits on the
/// first nonzero 8-byte WORD — identical results for every input).
///
/// Full 8-byte chunks go through the branch-free per-word nonzero detect
/// `((v & !HI) + !HI) | v` (bit 7 of a lane set iff that byte is nonzero,
/// no cross-byte borrow); the `len % 8` tail falls back to a byte loop
/// (variable-length SWAR tail, same early-exit). Allocation-free,
/// panic-free (`get` bounds), `no_std`.
#[inline]
pub fn swar_all_zero_vl(bytes: &[u8]) -> bool {
    // E8: 2-word unrolled outer loop — halves per-iteration loop overhead
    // for full-16 chunks; identical early-exit semantics per 8-byte word
    // (byte-identical results for every input, pinned by the differential
    // courts below).
    let mut pairs = bytes.chunks_exact(16);
    for p in &mut pairs {
        let v0 = u64::from_le_bytes([p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7]]);
        if ((v0 & NOT_HI).wrapping_add(NOT_HI) | v0) & HI != 0 {
            return false;
        }
        let v1 = u64::from_le_bytes([p[8], p[9], p[10], p[11], p[12], p[13], p[14], p[15]]);
        if ((v1 & NOT_HI).wrapping_add(NOT_HI) | v1) & HI != 0 {
            return false;
        }
    }
    let mut chunks = pairs.remainder().chunks_exact(8);
    for c in &mut chunks {
        let v = u64::from_le_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]);
        // Nonzero iff any lane's high bit set after the borrow-free detect.
        if ((v & NOT_HI).wrapping_add(NOT_HI) | v) & HI != 0 {
            return false;
        }
    }
    let rem = chunks.remainder();
    let mut i = 0;
    while i < rem.len() {
        if rem[i] != 0 {
            return false;
        }
        i += 1;
    }
    true
}

#[cfg(test)]
mod courts {
    use super::*;

    fn corpus_splitmix64(mut state: u64) -> u64 {
        state = state.wrapping_add(0xBF58_476D_1CE4_E5B9);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z ^ (z >> 31)
    }

    const CORPUS_SEED: u64 = 0xA20C_0FFE_E5A7_E000;

    // -- sha256-pinned shipped oracles (copy precedent: radix.rs:33-45; the
    // -- gate is upstream of this crate so a dev-dependency would be
    // -- circular). Pins recorded 2026-11-02 from
    // -- crates/cgap-gate/src/penstock.rs (git HEAD 9a730320).

    /// Shipped `is_zero_at` verbatim (penstock.rs:651-660).
    fn shipped_is_zero_at(bytes: &[u8], off: usize, len: usize) -> bool {
        let mut i = 0;
        while i < len {
            if bytes[off + i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }

    /// Shipped `prefilter8` byte-load verbatim (penstock.rs:677-686).
    fn shipped_prefilter8(bytes: &[u8]) -> u64 {
        let mut v = [0u8; 8];
        let mut i = 0;
        while i < 8 {
            v[i] = bytes[i];
            i += 1;
        }
        let w = u64::from_le_bytes(v);
        (w ^ (w >> 7)).wrapping_mul(0x2544_5491_4F6C_DD1D)
    }

    /// Pin receipt (sha256 over the exact HEAD 9a730320 source lines):
    /// prefilter8 (penstock.rs:677-686) =
    /// 0a9447abbc89569e618f7954435b17b46084908f20a3827015cc087a05ca82c4;
    /// is_zero_at (penstock.rs:651-660) =
    /// 6e66ff3d7422086dbea3f50faf08f6144a7012f6566b6148bb18e96d6a2fc5d7.

    /// Differential: 50k random slices + exhaustive structural edges —
    /// `swar_all_zero_vl` agrees with the shipped `is_zero_at` oracle on
    /// every (offset, len) window, including tails 0..7.
    #[test]
    fn all_zero_differential_vs_shipped() {
        let mut s = CORPUS_SEED;
        for _ in 0..50_000 {
            let n = (corpus_splitmix64(s) % 96) as usize;
            s = corpus_splitmix64(s);
            let mut buf = [0u8; 96];
            let nz = (corpus_splitmix64(s) % 4) as usize;
            for _ in 0..nz {
                let idx = (corpus_splitmix64(s) % 96) as usize;
                s = corpus_splitmix64(s);
                buf[idx] = (corpus_splitmix64(s) | 1) as u8;
                s = corpus_splitmix64(s);
            }
            for off in 0..=96 - n {
                assert_eq!(
                    swar_all_zero_vl(&buf[off..off + n]),
                    shipped_is_zero_at(&buf, off, n),
                    "vl off={off} n={n}"
                );
            }
        }
    }

    /// Structural edges: empty slice, each tail 0..8, single-byte spans,
    /// all-zero vs one-nonzero per lane position, off-chunk boundaries.
    #[test]
    fn all_zero_structural_edges() {
        assert!(swar_all_zero_vl(b""));
        let zeros = [0u8; 40];
        for n in 0..=40 {
            assert!(swar_all_zero_vl(&zeros[..n]));
            for pos in 0..n {
                let mut b = zeros;
                b[pos] = 0x80;
                assert!(!swar_all_zero_vl(&b[..n]), "pos={pos} n={n}");
            }
        }
    }

    /// Differential: 50k random 8-byte buffers — `penstock_prefilter8` and
    /// the word-level mix agree with the shipped oracle on every input.
    #[test]
    fn prefilter_differential_vs_shipped() {
        let mut s = CORPUS_SEED;
        for _ in 0..50_000 {
            let mut buf = [0u8; 8];
            let w = corpus_splitmix64(s);
            s = corpus_splitmix64(s);
            buf.copy_from_slice(&w.to_le_bytes());
            let expect = shipped_prefilter8(&buf);
            assert_eq!(penstock_prefilter8(&buf), expect, "buf={buf:?}");
            assert_eq!(penstock_prefilter8_mix(w), expect, "w={w:#018x}");
        }
    }

    /// ANTI-VACUITY MUTATION M-mix: swap the shipped mix constant for the
    /// SplitMix64 multiplier (a seed-conflation corruption). The differential
    /// corpus MUST detect it — the court flips loud, proving it is not
    /// passing against a mutated oracle.
    #[test]
    fn mutation_m_mix_seed_conflation_flips() {
        let mutated = |w: u64| (w ^ (w >> 7)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        let mut s = CORPUS_SEED;
        let mut detected = 0;
        for _ in 0..1_000 {
            let w = corpus_splitmix64(s);
            s = corpus_splitmix64(s);
            if penstock_prefilter8_mix(w) != mutated(w) {
                detected += 1;
            }
        }
        assert!(detected > 990, "mutation detected on {detected}/1000");
    }

    /// ANTI-VACUITY MUTATION M-zero: make the SWAR detect "any zero byte"
    /// instead of "all zero" (the classic SWAR-zero/has-zero conflation).
    /// The structural-edge court MUST flip on a mixed buffer.
    #[test]
    fn mutation_m_zero_has_zero_conflation_flips() {
        // has-zero semantics disagrees with all-zero semantics exactly when
        // the slice mixes zero and nonzero bytes.
        let mixed = [0u8, 1, 0, 2, 0, 3, 0, 4, 0, 5];
        assert!(!swar_all_zero_vl(&mixed));
        // The mutated (has-zero) kernel disagrees with all-zero semantics on
        // this buffer AND across the differential corpus, so any regression
        // to the has-zero form flips this court.
        let mutated_has_zero = |bytes: &[u8]| bytes.iter().any(|&b| b == 0);
        assert!(mutated_has_zero(&mixed));
        assert_ne!(mutated_has_zero(&mixed), swar_all_zero_vl(&mixed));
    }
}
