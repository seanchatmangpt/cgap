//! fips203 - ML-KEM (FIPS 203) arithmetic core for ML-KEM-768.
//!
//! # PROVENANCE
//!
//! Structure and constants derived from the pqcrystals/kyber reference
//! lineage (via the PQClean `PQCLEAN_MLKEM512_CLEAN_*` clean C sources at
//! `~/bytestar/bytepqc/src/pqclean_kyber512_clean/`; the n=256, q=3329
//! kernel is shared across all ML-KEM parameter sets). Parameters per
//! FIPS 203 (final, 2024-08): ML-KEM-768: k=3, eta1=eta2=2, du=10, dv=4.
//! The ZETAS table was generated in-session from the spec formula
//! (tmp[0]=MONT; tmp[i]=fqmul(tmp[i-1], MONT*17 mod q); bit-reversed tree;
//! centered representatives) and cross-checked entry-by-entry against the
//! PQClean literal table (court `zetas_table_pqclean_entry_check`).
//!
//! # SEAM STATUS (AB-5 / AB-2 coordination)
//!
//! Composes `crate::fips202` (AB-2, landed this session) for SHAKE-128/256
//! and the Keccak-f[1600] permutation. AB-2's `Sponge` is SHAKE-domain only
//! (0x1F pad); H and G need the 0x06 SHA3 domain, so a thin in-file wrapper
//! (`sha3_sponge`) drives AB-2's public `keccak_f1600` with SHA3 padding.
//! This wrapper is deleted when AB-2 exposes a SHA3-domain surface.
//!
//! All secret-dependent paths are branchless (mask/ct-select class,
//! ADR-04/ADR-14). `#![forbid(unsafe_code)]` crate-wide. Zero-alloc public
//! API (courted in `tests/mlkem768_zeroalloc.rs`).

// ---------------------------------------------------------------------------
// Parameters (FIPS 203, ML-KEM-768)
// ---------------------------------------------------------------------------

/// Field modulus q = 3329.
pub const Q: i16 = 3329;
/// Polynomial degree n = 256.
pub const N: usize = 256;
/// Montgomery constant R = 2^16 mod q, centered representative.
pub const MONT: i16 = -1044;
/// `QINV = q^{-1} mod 2^16 = 62209` (Montgomery reduction constant).
/// NOTE: 62209 > i16::MAX; held as u16 (C ref relies on silent wrap).
pub const QINV: u16 = 62209;
/// Barrett constant `v = (2^26 + q/2) div q = 20159`.
pub const BARRETT_V: i16 = 20159;
/// INTT final scaling `f = mont^2 / 128 = 1441`.
pub const F_MONT2_OVER_N: i16 = 1441;
/// `2^32 mod q = 1353` (poly_tomont factor).
pub const MONT2: i16 = 1353;

/// ML-KEM-768 rank k = 3.
pub const K: usize = 3;
/// CBD eta1 = 2 (FIPS 203 final).
pub const ETA1: u8 = 2;
/// CBD sampling width in bytes: eta*256/4 = 128.
pub const PRF_OUT_ETA: usize = 128;
/// Ciphertext coefficient depth du = 10.
pub const DU: u32 = 10;
/// Ciphertext secret-value depth dv = 4.
pub const DV: u32 = 4;
/// Compressed poly bytes at du=10: 256*10/8 = 320.
pub const POLY_COMPRESSED10_BYTES: usize = 320;
/// Compressed poly bytes at dv=4: 256*4/8 = 128.
pub const POLY_COMPRESSED4_BYTES: usize = 128;
/// Seed / hash / shared-secret byte count.
pub const SYMBYTES: usize = 32;
/// Encapsulation key: k*384 + 32 = 1184.
pub const EK_BYTES: usize = K * 384 + SYMBYTES;
/// Decapsulation key: k*384 + |ek| + 2*32 = 2400.
pub const DK_BYTES: usize = K * 384 + EK_BYTES + 2 * SYMBYTES;
/// Ciphertext: k*320 + 128 = 1088 (du=10 vector + dv=4 poly).
pub const CT_BYTES: usize = K * POLY_COMPRESSED10_BYTES + POLY_COMPRESSED4_BYTES;
/// Shared secret: 32.
pub const CT_LEN: usize = CT_BYTES;
pub const SS_BYTES: usize = 32;
/// XOF block (SHAKE-128 rate) = 168 bytes.
pub const XOF_BLOCKBYTES: usize = 168;

const _: () = {
    assert!(EK_BYTES == 1184);
    assert!(DK_BYTES == 2400);
    assert!(CT_BYTES == 1088);
    assert!(CT_LEN == 1088);
    assert!(SS_BYTES == 32);
    assert!(Q == 3329 && N == 256 && K == 3 && DU == 10 && DV == 4);
    // Montgomery law: q * QINV == 1 (mod 2^16); 3329*62209 = 3160*2^16 + 1.
    assert!((Q as i32 * QINV as i32) % 65536 == 1);
    // Barrett law: v = floor((2^26 + q/2) / q).
    assert!(BARRETT_V == (((1i32 << 26) + Q as i32 / 2) / Q as i32) as i16);
    // du packing law: 256*10/8 = 320, 256*4/8 = 128.
    assert!(K * 320 + 128 == 1088);
};

// ---------------------------------------------------------------------------
// SHA3-domain sponge (thin wrapper over AB-2's permutation)
// ---------------------------------------------------------------------------

mod sha3 {
    //! SHA3-domain sponge driving crate::fips202::keccak_f1600 with 0x06
    //! padding. AB-2's `Sponge` hardwires 0x1F (SHAKE) padding; H and G are
    //! SHA3-domain. Deleted when AB-2 ships a SHA3-domain surface.

    /// SHA3 (0x06-domain) sponge over concatenated parts, one-shot squeeze.
    pub fn sponge(rate: usize, parts: &[&[u8]], out: &mut [u8]) {
        use crate::fips202::keccak_f1600;
        let mut st: [u64; 25] = [0; 25];
        let mut pos = 0usize;
        for part in parts.iter() {
            for i in 0..part.len() {
                let byte = part[i];
                st[pos / 8] ^= (byte as u64) << (8 * (pos % 8));
                pos += 1;
                if pos % rate == 0 {
                    keccak_f1600(&mut st);
                    pos = 0;
                }
            }
        }
        let last = rate - 1;
        for i in pos..rate {
            let byte = (if i == pos { 0x06 } else { 0 }) | (if i == last { 0x80 } else { 0 });
            st[i / 8] ^= (byte as u64) << (8 * (i % 8));
        }
        keccak_f1600(&mut st);
        let mut done = 0usize;
        while done < out.len() {
            let take = rate.min(out.len() - done);
            for j in 0..take {
                out[done + j] = (st[j / 8] >> (8 * (j % 8))) as u8;
            }
            done += take;
            if done < out.len() {
                keccak_f1600(&mut st);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Reductions (branchless Montgomery / Barrett)
// ---------------------------------------------------------------------------

/// Branchless 16-bit select: returns `b_sel != 0 ? t : f`.
#[inline(always)]
pub(crate) const fn ct_select_i16(b_sel: i16, t: i16, f: i16) -> i16 {
    let m = (b_sel as u16).wrapping_neg();
    (f as u16 ^ (m & (f as u16 ^ t as u16))) as i16
}

/// Montgomery reduce: `a * R^-1 mod q`, result in {-q+1..q-1}. Branchless.
#[inline(always)]
pub fn montgomery_reduce(a: i32) -> i16 {
    let t = (a as u16).wrapping_mul(QINV) as i16;
    let t = (a - t as i32 * Q as i32) >> 16;
    t as i16
}

/// Barrett reduce: centered representative in {-(q-1)/2..(q-1)/2}. Branchless.
#[inline(always)]
pub fn barrett_reduce(a: i16) -> i16 {
    let t = ((BARRETT_V as i32 * a as i32 + (1 << 25)) >> 26) as i16;
    (a as i32 - t as i32 * Q as i32) as i16
}

/// `a * b * R^-1 mod q` (Montgomery multiply).
#[inline(always)]
pub fn fqmul(a: i16, b: i16) -> i16 {
    montgomery_reduce(a as i32 * b as i32)
}

/// Canonicalize centered rep to [0, q): branchless conditional add.
#[inline(always)]
pub fn cond_add_q(a: i16) -> i16 {
    // i16 >> 15 is an ARITHMETIC shift (all-ones for negatives), so
    // (-1) & Q == Q — the C ref's `((int16_t)t >> 15) & KYBER_Q`.
    a + ((a >> 15) & Q)
}

// ---------------------------------------------------------------------------
// ZETAS table (pqcrystals-lineage values, generated + digest-pinned)
// ---------------------------------------------------------------------------
/// pqcrystals/PQClean-lineage zetas table (centered Montgomery-domain i16),
/// ONE layout story — the code is the citation:
///
/// - `ZETAS[0] = -1044` is load-bearing padding (the literal table's first
///   value; every table read starts at index 1).
/// - `ZETAS[1..=127]` = the 128 forward-NTT twiddle values: `ntt` consumes
///   `ZETAS[k]` for k = 1..=127 ascending (`fips203::ntt`), the INTT consumes
///   the same segment `ZETAS[k]` for k = 127..=1 descending
///   (`fips203::intt_tomont`), and `poly_basemul_montgomery` consumes
///   `ZETAS[64..=127]` — one literal segment, three traversal windows.
/// - `ZETAS[128..=255]` = the same 128 values in reverse (INTT traversal
///   mirror; asserted by court `zetas_table_pqclean_entry_check`).
/// - `ZETAS[256..=258] = [1441, -1044, 0]`: 1441 = f = mont^2/128 (INTT
///   final scale), -1044 = MONT(1) centered, 0 padding.
///
/// Generated in-session from the spec formula (tmp[0]=MONT;
/// tmp[i]=fqmul(tmp[i-1], MONT*17 mod q); bit-reversed tree; centered
/// representatives) and cross-checked entry-by-entry against the PQClean
/// literal table (court `zetas_table_pqclean_entry_check`); blake3-digest
/// pinned (court `zetas_digest_pin`).
pub const ZETAS: [i16; 259] = [
    -1044, -758, -359, -1517, 1493, 1422, 287, 202, -171, 622, 1577, 182, 962, -1202, -1474, 1468,
    573, -1325, 264, 383, -829, 1458, -1602, -130, -681, 1017, 732, 608, -1542, 411, -205, -1571,
    1223, 652, -552, 1015, -1293, 1491, -282, -1544, 516, -8, -320, -666, -1618, -1162, 126, 1469,
    -853, -90, -271, 830, 107, -1421, -247, -951, -398, 961, -1508, -725, 448, -1065, 677, -1275,
    -1103, 430, 555, 843, -1251, 871, 1550, 105, 422, 587, 177, -235, -291, -460, 1574, 1653, -246,
    778, 1159, -147, -777, 1483, -602, 1119, -1590, 644, -872, 349, 418, 329, -156, -75, 817, 1097,
    603, 610, 1322, -1285, -1465, 384, -1215, -136, 1218, -1335, -874, 220, -1187, -1659, -1185,
    -1530, -1278, 794, -1510, -854, -870, 478, -108, -308, 996, 991, 958, -1460, 1522, 1628, 1628,
    1522, -1460, 958, 991, 996, -308, -108, 478, -870, -854, -1510, 794, -1278, -1530, -1185,
    -1659, -1187, 220, -874, -1335, 1218, -136, -1215, 384, -1465, -1285, 1322, 610, 603, 1097,
    817, -75, -156, 329, 418, 349, -872, 644, -1590, 1119, -602, 1483, -777, -147, 1159, 778, -246,
    1653, 1574, -460, -291, -235, 177, 587, 422, 105, 1550, 871, -1251, 843, 555, 430, -1103,
    -1275, 677, -1065, 448, -725, -1508, 961, -398, -951, -247, -1421, 107, 830, -271, -90, -853,
    1469, 126, -1162, -1618, -666, -320, -8, 516, -1544, -282, 1491, -1293, 1015, -552, 652, 1223,
    -1571, -205, 411, -1542, 608, 732, 1017, -681, -130, -1602, 1458, -829, 383, 264, -1325, 573,
    1468, -1474, -1202, 962, 182, 1577, 622, -171, 202, 287, 1422, 1493, -1517, -359, -758, -1044,
    1441, -1044, 0,
];
// ---------------------------------------------------------------------------
// NTT / INTT (in place, fixed trips: 7 stages x 128 butterflies)
// ---------------------------------------------------------------------------

/// Forward negacyclic NTT, in place (in: standard order, out: bit-reversed).
pub fn ntt(r: &mut [i16; N]) {
    let mut k = 1usize;
    let mut len = 128usize;
    while len >= 2 {
        let mut start = 0usize;
        while start < N {
            let zeta = ZETAS[k];
            k += 1;
            for j in start..start + len {
                let t = fqmul(zeta, r[j + len]);
                r[j + len] = r[j] - t;
                r[j] = r[j] + t;
            }
            start += 2 * len;
        }
        len >>= 1;
    }
}

/// Inverse NTT + Montgomery factor 2^16, in place (in: bit-rev, out: std).
pub fn intt_tomont(r: &mut [i16; N]) {
    let mut k = 127usize;
    let mut len = 2usize;
    while len <= 128 {
        let mut start = 0usize;
        while start < N {
            let zeta = ZETAS[k];
            k -= 1;
            for j in start..start + len {
                let t = r[j];
                r[j] = barrett_reduce(t + r[j + len]);
                r[j + len] = fqmul(zeta, r[j + len] - t);
            }
            start += 2 * len;
        }
        len <<= 1;
    }
    for c in r.iter_mut() {
        *c = fqmul(*c, F_MONT2_OVER_N);
    }
}

/// Base-case multiply in Zq[X]/(X^2 - zeta), Montgomery domain.
#[inline(always)]
fn basemul2(r: &mut [i16; 2], a: &[i16; 2], b: &[i16; 2], zeta: i16) {
    r[0] = fqmul(fqmul(a[1], b[1]), zeta);
    r[0] += fqmul(a[0], b[0]);
    r[1] = fqmul(a[0], b[1]);
    r[1] += fqmul(a[1], b[0]);
}

/// NTT-domain polynomial multiply (Montgomery domain operands).
pub fn poly_basemul_montgomery(r: &mut [i16; N], a: &[i16; N], b: &[i16; N]) {
    for i in 0..N / 4 {
        let z = ZETAS[64 + i];
        let a0 = [a[4 * i], a[4 * i + 1]];
        let b0 = [b[4 * i], b[4 * i + 1]];
        let a1 = [a[4 * i + 2], a[4 * i + 3]];
        let b1 = [b[4 * i + 2], b[4 * i + 3]];
        let mut lo = [0i16; 2];
        let mut hi = [0i16; 2];
        basemul2(&mut lo, &a0, &b0, z);
        basemul2(&mut hi, &a1, &b1, -z);
        r[4 * i] = lo[0];
        r[4 * i + 1] = lo[1];
        r[4 * i + 2] = hi[0];
        r[4 * i + 3] = hi[1];
    }
}

/// Coefficient-wise Barrett canonicalization.
pub fn poly_reduce(r: &mut [i16; N]) {
    for c in r.iter_mut() {
        *c = barrett_reduce(*c);
    }
}

/// r = a + b (no reduction).
pub fn poly_add(r: &mut [i16; N], a: &[i16; N], b: &[i16; N]) {
    for i in 0..N {
        r[i] = a[i] + b[i];
    }
}

/// r = a - b (no reduction).
pub fn poly_sub(r: &mut [i16; N], a: &[i16; N], b: &[i16; N]) {
    for i in 0..N {
        r[i] = a[i] - b[i];
    }
}

/// In-place to-Montgomery: r = r * 2^32 mod q.
pub fn poly_tomont(r: &mut [i16; N]) {
    for c in r.iter_mut() {
        *c = fqmul(*c, MONT2);
    }
}

// ---------------------------------------------------------------------------
// CBD sampling (eta = 2 primary for ML-KEM; eta = 4 for ML-DSA parity)
// ---------------------------------------------------------------------------

/// Typed refusal for an unsupported CBD width (replaces the former silent
/// `_ => ()` zero-fill, which produced all-zero secret polys unpunished).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CbdError {
    /// No masked decomposition is implemented for this eta (lawful: 2, 4).
    UnsupportedEta(u8),
}

impl core::fmt::Display for CbdError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CbdError::UnsupportedEta(e) => {
                write!(f, "poly_cbd: unsupported CBD eta {} (lawful: 2, 4)", e)
            }
        }
    }
}

/// CBD(eta) from `buf` (eta*256/4 bytes), branchless masked decomposition.
///
/// One spec law for both widths (FIPS 203/204 SamplePolyCBD, consecutive
/// per-coefficient bit fields): coefficient i consumes bits 2*eta*i ..
/// 2*eta*i + 2*eta − 1 of `buf` (LSB-first), the first eta bits are the
/// a-field and the last eta bits the b-field, r[i] = popcount(a) −
/// popcount(b).
///
/// - eta = 2: a 32-bit word (4 bytes) holds 8 coefficients in nibble i;
///   `d = (t & 0x5555_5555) + ((t >> 1) & 0x5555_5555)` gives per-nibble
///   popcounts in 2-bit fields.
/// - eta = 4: a 32-bit word holds 4 coefficients, one byte each; the
///   low nibble of byte j is the a-field, the high nibble the b-field.
///   `d = Σ_shift (t >> shift) & 0x1111_1111` gives per-nibble popcounts in
///   4-bit fields (popcount ≤ 4 < 16, so fields never collide).
pub fn poly_cbd(r: &mut [i16; N], buf: &[u8], eta: u8) -> Result<(), CbdError> {
    debug_assert!(eta != 2 && eta != 4 || buf.len() == eta as usize * N / 4);
    match eta {
        2 => {
            for (i, ch) in buf.chunks_exact(4).enumerate() {
                let t = u32::from_le_bytes([ch[0], ch[1], ch[2], ch[3]]);
                let d = (t & 0x5555_5555).wrapping_add((t >> 1) & 0x5555_5555);
                for j in 0..8 {
                    let a = (d >> (4 * j)) & 0x3;
                    let b = (d >> (4 * j + 2)) & 0x3;
                    r[8 * i + j] = (a as i16) - (b as i16);
                }
            }
            Ok(())
        }
        4 => {
            for (i, ch) in buf.chunks_exact(4).enumerate() {
                let t = u32::from_le_bytes([ch[0], ch[1], ch[2], ch[3]]);
                let d = (t & 0x1111_1111)
                    + ((t >> 1) & 0x1111_1111)
                    + ((t >> 2) & 0x1111_1111)
                    + ((t >> 3) & 0x1111_1111);
                for j in 0..4 {
                    let byte_d = d >> (8 * j);
                    let a = byte_d & 0x7; // popcount of low nibble (<= 4)
                    let b = (byte_d >> 4) & 0x7; // popcount of high nibble
                    r[4 * i + j] = (a as i16) - (b as i16);
                }
            }
            Ok(())
        }
        e => Err(CbdError::UnsupportedEta(e)),
    }
}

// ---------------------------------------------------------------------------
// ByteEncode_d / ByteDecode_d (LSB-first, d = 1..=12)
// ---------------------------------------------------------------------------

/// Serialize `f` as d-bit coefficients into `out` (256*d/8 bytes).
pub fn byte_encode<const D: usize>(f: &[i16; N], out: &mut [u8]) {
    debug_assert_eq!(out.len(), N * D / 8);
    let mut acc: u64 = 0;
    let mut bits = 0u32;
    let mut oi = 0usize;
    for i in 0..N {
        let v = cond_add_q(f[i]) as u64 & ((1u64 << D) - 1);
        acc |= v << bits;
        bits += D as u32;
        while bits >= 8 {
            out[oi] = (acc & 0xff) as u8;
            oi += 1;
            acc >>= 8;
            bits -= 8;
        }
    }
}

/// Inverse of byte_encode; unsigned representatives in [0, 2^d).
pub fn byte_decode<const D: usize>(inp: &[u8]) -> [i16; N] {
    debug_assert_eq!(inp.len(), N * D / 8);
    let mut f = [0i16; N];
    let mut acc: u64 = 0;
    let mut bits = 0u32;
    let mut ii = 0usize;
    for i in 0..N {
        while bits < D as u32 {
            acc |= (inp[ii] as u64) << bits;
            ii += 1;
            bits += 8;
        }
        f[i] = (acc & ((1u64 << D) - 1)) as i16;
        acc >>= D as u32;
        bits -= D as u32;
    }
    f
}

// ---------------------------------------------------------------------------
// Compress / decompress packers (du = 10, dv = 4) and message packing
// ---------------------------------------------------------------------------

/// Quantize: Compress_d with FIPS 203 rounding, d-bit result.
#[inline(always)]
pub fn compress_round(a: i16, d: u32) -> u16 {
    let u = cond_add_q(a) as i32;
    let num = ((u << d) as i64 + ((Q as i64) >> 1)) / Q as i64;
    (num & ((1i64 << d) - 1)) as u16
}

/// Inverse: Decompress_d.
#[inline(always)]
pub fn decompress_round(x: u16, d: u32) -> i16 {
    ((x as i64 * Q as i64 + (1i64 << (d - 1))) >> d) as i16
}

/// Bit-pack compress_d values LSB-first (generic driver).
fn pack_bits(vals: impl Iterator<Item = u16>, width: u32, out: &mut [u8]) {
    let mut acc: u64 = 0;
    let mut bits = 0u32;
    let mut oi = 0usize;
    for v in vals {
        acc |= (v as u64) << bits;
        bits += width;
        while bits >= 8 {
            out[oi] = (acc & 0xff) as u8;
            oi += 1;
            acc >>= 8;
            bits -= 8;
        }
    }
}

/// Compress_10 + pack: 320 bytes.
pub fn poly_compress10(r: &mut [u8; 320], a: &[i16; N]) {
    let mut vals = [0u16; N];
    for i in 0..N {
        vals[i] = compress_round(a[i], DU);
    }
    pack_bits(vals.iter().copied(), DU, r);
}

/// Compress_4 + pack: 128 bytes.
pub fn poly_compress4(r: &mut [u8; 128], a: &[i16; N]) {
    let mut vals = [0u16; N];
    for i in 0..N {
        vals[i] = compress_round(a[i], DV);
    }
    pack_bits(vals.iter().copied(), DV, r);
}

/// Unpack + Decompress_10.
pub fn poly_decompress10(r: &mut [i16; N], a: &[u8; 320]) {
    let mut acc: u64 = 0;
    let mut bits = 0u32;
    let mut ii = 0usize;
    for c in r.iter_mut() {
        while bits < DU {
            acc |= (a[ii] as u64) << bits;
            ii += 1;
            bits += 8;
        }
        *c = decompress_round((acc & ((1 << DU) - 1)) as u16, DU);
        acc >>= DU;
        bits -= DU;
    }
}

/// Unpack + Decompress_4.
pub fn poly_decompress4(r: &mut [i16; N], a: &[u8; 128]) {
    let mut acc: u64 = 0;
    let mut bits = 0u32;
    let mut ii = 0usize;
    for c in r.iter_mut() {
        while bits < DV {
            acc |= (a[ii] as u64) << bits;
            ii += 1;
            bits += 8;
        }
        *c = decompress_round((acc & ((1 << DV) - 1)) as u16, DV);
        acc >>= DV;
        bits -= DV;
    }
}

/// Message bytes -> polynomial ((q+1)/2 per set bit, branchless).
pub fn poly_frommsg(r: &mut [i16; N], msg: &[u8; 32]) {
    let half = (Q + 1) / 2;
    for i in 0..32 {
        for j in 0..8 {
            let bit = ((msg[i] >> j) & 1) as i16;
            r[8 * i + j] = ct_select_i16(bit, half, 0);
        }
    }
}

/// Polynomial -> message bytes (Compress_1, branchless).
pub fn poly_tomsg(msg: &mut [u8; 32], a: &[i16; N]) {
    for i in 0..32 {
        let mut m = 0u8;
        for j in 0..8 {
            m |= (compress_round(a[8 * i + j], 1) as u8) << j;
        }
        msg[i] = m;
    }
}

// ---------------------------------------------------------------------------
// FIPS 203 section-4 hash suite
// ---------------------------------------------------------------------------

/// H = SHA3-256.
pub fn hash_h(out: &mut [u8; 32], data: &[u8]) {
    sha3::sponge(136, &[data], out);
}

/// Multi-part H.
pub fn hash_h_parts(out: &mut [u8; 32], parts: &[&[u8]]) {
    sha3::sponge(136, parts, out);
}

/// G = SHA3-512 (rate 72 = (1600-1024)/8).
pub fn hash_g(out: &mut [u8; 64], data: &[u8]) {
    sha3::sponge(72, &[data], out);
}

/// Multi-part G.
pub fn hash_g_parts(out: &mut [u8; 64], parts: &[&[u8]]) {
    sha3::sponge(72, parts, out);
}

/// PRF_eta: SHAKE-256(key || nonce) into out (eta*64 bytes for eta=2).
pub fn prf(out: &mut [u8], key: &[u8; 32], nonce: u8) {
    let mut s = crate::fips202::Sponge::new(crate::fips202::Shake256::RATE);
    s.absorb(&key[..]);
    s.absorb(&[nonce]);
    s.squeeze_into(out);
}

/// J: SHAKE-256(z || c), 32 bytes.
pub fn hash_j(out: &mut [u8; 32], z: &[u8; 32], c: &[u8]) {
    let mut s = crate::fips202::Sponge::new(crate::fips202::Shake256::RATE);
    s.absorb(&z[..]);
    s.absorb(c);
    s.squeeze_into(out);
}

/// XOF for SampleNTT: SHAKE-128(seed || b0 || b1), incremental squeezing.
pub struct XofState {
    inner: crate::fips202::Sponge,
    buf: [u8; XOF_BLOCKBYTES],
    buf_pos: usize,
}

impl XofState {
    /// Absorb (seed || b0 || b1); sponge finalized lazily on first squeeze.
    pub fn new(seed: &[u8; 32], b0: u8, b1: u8) -> Self {
        let mut inner = crate::fips202::Sponge::new(crate::fips202::Shake128::RATE);
        inner.absorb(&seed[..]);
        inner.absorb(&[b0, b1]);
        XofState {
            inner,
            buf: [0; XOF_BLOCKBYTES],
            buf_pos: XOF_BLOCKBYTES,
        }
    }

    /// Fill `out` with XOF bytes, squeezing blocks on demand.
    pub fn squeeze_into(&mut self, out: &mut [u8]) {
        let mut done = 0usize;
        while done < out.len() {
            if self.buf_pos == XOF_BLOCKBYTES {
                self.inner.squeeze_into(&mut self.buf);
                self.buf_pos = 0;
            }
            let take = (XOF_BLOCKBYTES - self.buf_pos).min(out.len() - done);
            out[done..done + take].copy_from_slice(&self.buf[self.buf_pos..self.buf_pos + take]);
            done += take;
            self.buf_pos += take;
        }
    }
}

// ---------------------------------------------------------------------------
// Sampling and matrix generation
// ---------------------------------------------------------------------------

/// Rejection-uniform sampling of up to `len` values in [0, q) from `buf`.
fn rej_uniform(r: &mut [i16], len: usize, buf: &[u8]) -> usize {
    let mut ctr = 0usize;
    let mut pos = 0usize;
    while ctr < len && pos + 3 <= buf.len() {
        let b0 = buf[pos] as u16;
        let b1 = buf[pos + 1] as u16;
        let b2 = buf[pos + 2] as u16;
        let val0 = (b0 | (b1 << 8)) & 0x0fff;
        let val1 = ((b1 >> 4) | (b2 << 4)) & 0x0fff;
        pos += 3;
        if val0 < Q as u16 {
            r[ctr] = val0 as i16;
            ctr += 1;
        }
        if ctr < len && val1 < Q as u16 {
            r[ctr] = val1 as i16;
            ctr += 1;
        }
    }
    ctr
}

/// SampleNTT (FIPS 203 Alg 7): uniform in Rq via XOF rejection sampling.
pub fn sample_ntt(seed: &[u8; 32], b0: u8, b1: u8) -> [i16; N] {
    let mut r = [0i16; N];
    let mut ctr = 0usize;
    let mut xof = XofState::new(seed, b0, b1);
    let mut buf = [0u8; XOF_BLOCKBYTES];
    while ctr < N {
        xof.squeeze_into(&mut buf);
        let got = rej_uniform(&mut r[ctr..], N - ctr, &buf);
        ctr += got;
    }
    r
}

/// SamplePolyCBD from (seed, nonce) via PRF; eta hardcoded to 2 (=PRF_OUT).
pub fn sample_poly_cbd_prf(seed: &[u8; 32], nonce: u8) -> [i16; N] {
    let mut buf = [0u8; PRF_OUT_ETA];
    prf(&mut buf, seed, nonce);
    let mut p = [0i16; N];
    // eta = 2 is the ML-KEM-768 constant; the Err arm is unreachable here.
    poly_cbd(&mut p, &buf, 2).expect("eta=2 is lawful for ML-KEM CBD");
    p
}

/// Matrix generation: transposed=false gives A (entry [i][j] from
/// XOF(rho || j || i)); true gives A^T (entry [i][j] from XOF(rho || i || j)).
/// Matches the reference gen_a / gen_at index convention.
pub fn gen_matrix(a: &mut [[[i16; N]; K]; K], seed: &[u8; 32], transposed: bool) {
    for i in 0..K {
        for j in 0..K {
            let (b0, b1) = if transposed {
                (i as u8, j as u8)
            } else {
                (j as u8, i as u8)
            };
            a[i][j] = sample_ntt(seed, b0, b1);
        }
    }
}

// ---------------------------------------------------------------------------
// Wire serialization (12-bit polynomial packing)
// ---------------------------------------------------------------------------

/// Pack polynomial to 384 bytes (canonical 12-bit coefficients).
pub fn poly_tobytes(r: &mut [u8; 384], a: &[i16; N]) {
    for i in 0..N / 2 {
        let t0 = cond_add_q(a[2 * i]) as u16;
        let t1 = cond_add_q(a[2 * i + 1]) as u16;
        r[3 * i] = (t0 & 0xff) as u8;
        r[3 * i + 1] = ((t0 >> 8) | (t1 << 4)) as u8;
        r[3 * i + 2] = (t1 >> 4) as u8;
    }
}

/// Unpack 384 bytes into a polynomial (unsigned 12-bit reps).
pub fn poly_frombytes(r: &mut [i16; N], a: &[u8; 384]) {
    for i in 0..N / 2 {
        let b0 = a[3 * i] as u16;
        let b1 = a[3 * i + 1] as u16;
        let b2 = a[3 * i + 2] as u16;
        r[2 * i] = (b0 | (b1 << 8)) as i16 & 0x0fff;
        r[2 * i + 1] = ((b1 >> 4) | (b2 << 4)) as i16 & 0x0fff;
    }
}

/// r += b (no reduction; in-place accumulate form, borrowck-friendly).
pub fn poly_add_assign(r: &mut [i16; N], b: &[i16; N]) {
    for i in 0..N {
        r[i] = r[i].wrapping_add(b[i]);
    }
}

/// r -= b (no reduction; in-place accumulate form).
pub fn poly_sub_assign(r: &mut [i16; N], b: &[i16; N]) {
    for i in 0..N {
        r[i] = r[i].wrapping_sub(b[i]);
    }
}
