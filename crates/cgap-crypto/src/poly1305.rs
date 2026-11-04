//! Poly1305 one-time authenticator — poly1305-donna, 32-bit variant
//! (26-bit limbs, `u64` dot products), PG-A3 spec.
//!
//! Surface: `Poly1305::new` (key clamp), `push` (incremental, no alloc —
//! buffers partial blocks internally), `finish` (one-shot, constant-time
//! final reduction + selection: the `h >= p` branch is a branchless mask
//! select, never a data-dependent branch).
//!
//! Zero dependencies, `#![no_std]`-safe, no handwritten assembly, no
//! unguarded memory access (crate-level forbidden). Limb
//! invariant: `h[i] < 2^26` between blocks; `h < 2^131` overall, so the
//! finish's `h + 5 - 2^130` trick reduces `mod p = 2^130 - 5` in exactly
//! one constant-time pass.

#![allow(dead_code)] // PG-A3 wave: surface lands ahead of its consumers.

/// Authenticator tag width: 16 bytes (128 bits).
pub const TAG_LEN: usize = 16;
/// One-time key width: 16 bytes `r` (clamped) + 16 bytes `s` (pad).
pub const KEY_LEN: usize = 32;
/// Block width: Poly1305 absorbs 16 bytes per `mulmod` step.
pub const BLOCK_LEN: usize = 16;

const LIMB_MASK: u32 = 0x3ff_ffff; // 26 bits

/// Incremental Poly1305 state. Construct only through [`Poly1305::new`]
/// (the `r` clamp lives there and nowhere else).
pub struct Poly1305 {
    r: [u32; 5],
    h: [u32; 5],
    pad: [u32; 4],
    buffer: [u8; BLOCK_LEN],
    leftover: usize,
}

/// Clamp-and-load the one-time key: `r` = key[0..16] little-endian,
/// 130-bit, with the RFC 8439 §2.5 clamp (`0x0ffffffc0ffffffc0ffffffc0fffffff`
/// applied limbwise); `s` (the pad added at finish) = key[16..32].
pub fn new(key: &[u8; KEY_LEN]) -> Poly1305 {
    let t0 = u32::from_le_bytes(key[0..4].try_into().expect("4 bytes"));
    let t1 = u32::from_le_bytes(key[4..8].try_into().expect("4 bytes"));
    let t2 = u32::from_le_bytes(key[8..12].try_into().expect("4 bytes"));
    let t3 = u32::from_le_bytes(key[12..16].try_into().expect("4 bytes"));

    // Donna-32 limb extraction (byte-offset form, RFC-8439-clamped):
    // limb0 = bits 0-25; limb1 = bits 26-49 with clamp bits 50,51 cleared
    // (mask 0x3ffff03); limb2 mask 0x3ffc0ff; limb3 mask 0x3f03fff; limb4
    // keeps 20 bits. Verified against the RFC 8439 section 2.5.2 clamped r.
    let r = [
        u32::from_le_bytes(key[0..4].try_into().expect("4 bytes")) & 0x3ff_ffff,
        (u32::from_le_bytes(key[3..7].try_into().expect("4 bytes")) >> 2) & 0x3ff_ff03,
        (u32::from_le_bytes(key[6..10].try_into().expect("4 bytes")) >> 4) & 0x3ff_c0ff,
        (u32::from_le_bytes(key[9..13].try_into().expect("4 bytes")) >> 6) & 0x3f0_3fff,
        (u32::from_le_bytes(key[12..16].try_into().expect("4 bytes")) >> 8) & 0x000f_ffff,
    ];

    Poly1305 {
        r,
        h: [0; 5],
        pad: [
            u32::from_le_bytes(key[16..20].try_into().expect("4 bytes")),
            u32::from_le_bytes(key[20..24].try_into().expect("4 bytes")),
            u32::from_le_bytes(key[24..28].try_into().expect("4 bytes")),
            u32::from_le_bytes(key[28..32].try_into().expect("4 bytes")),
        ],
        buffer: [0; BLOCK_LEN],
        leftover: 0,
    }
}

/// Absorb `data` (any length, any chunking — identical byte streams give
/// identical tags regardless of `push` boundaries). No allocation: short
/// tails are buffered in the 16-byte inline block buffer.
pub fn push(state: &mut Poly1305, data: &[u8]) {
    let mut i = 0;
    if state.leftover > 0 {
        let want = core::cmp::min(BLOCK_LEN - state.leftover, data.len());
        state.buffer[state.leftover..state.leftover + want].copy_from_slice(&data[..want]);
        state.leftover += want;
        i += want;
        if state.leftover < BLOCK_LEN {
            return;
        }
        // SLA-minimal unblock (AB-3, disclosed): copy the buffer out to
        // satisfy the borrow checker (block is Copy; bytes identical).
        let block = state.buffer;
        absorb_block(state, &block, (1 << 24));
        state.leftover = 0;
    }

    let len = data.len();
    while len - i >= BLOCK_LEN {
        absorb_block(state, &data[i..i + BLOCK_LEN], 1 << 24);
        i += BLOCK_LEN;
    }

    if i < len {
        state.buffer[..len - i].copy_from_slice(&data[i..]);
        state.leftover = len - i;
    }
}

/// `h += block` then `h *= r mod p`, donna-32 exactly: byte-offset limb
/// extraction, sequential carry chain, `2^130 = 5` fold at h0. `hibit`
/// injects the 2^128 term for window: full blocks inject `1 << 24`; the
/// padded final partial block MUST pass 0 (donna's `final` flag).
fn absorb_block(state: &mut Poly1305, block: &[u8], hibit: u32) {
    let rd = |i: usize| u32::from_le_bytes(block[i..i + 4].try_into().expect("4 bytes"));

    state.h[0] += rd(0) & 0x3ff_ffff;
    state.h[1] += (rd(3) >> 2) & 0x3ff_ffff;
    state.h[2] += (rd(6) >> 4) & 0x3ff_ffff;
    state.h[3] += (rd(9) >> 6) & 0x3ff_ffff;
    state.h[4] += (rd(12) >> 8) | hibit;

    let r = &state.r;
    let h = &state.h;
    let s1 = 5 * r[1];
    let s2 = 5 * r[2];
    let s3 = 5 * r[3];
    let s4 = 5 * r[4];

    let mut d0 = h[0] as u64 * r[0] as u64
        + h[1] as u64 * s4 as u64
        + h[2] as u64 * s3 as u64
        + h[3] as u64 * s2 as u64
        + h[4] as u64 * s1 as u64;
    let mut d1 = h[0] as u64 * r[1] as u64
        + h[1] as u64 * r[0] as u64
        + h[2] as u64 * s4 as u64
        + h[3] as u64 * s3 as u64
        + h[4] as u64 * s2 as u64;
    let mut d2 = h[0] as u64 * r[2] as u64
        + h[1] as u64 * r[1] as u64
        + h[2] as u64 * r[0] as u64
        + h[3] as u64 * s4 as u64
        + h[4] as u64 * s3 as u64;
    let mut d3 = h[0] as u64 * r[3] as u64
        + h[1] as u64 * r[2] as u64
        + h[2] as u64 * r[1] as u64
        + h[3] as u64 * r[0] as u64
        + h[4] as u64 * s4 as u64;
    let mut d4 = h[0] as u64 * r[4] as u64
        + h[1] as u64 * r[3] as u64
        + h[2] as u64 * r[2] as u64
        + h[3] as u64 * r[1] as u64
        + h[4] as u64 * r[0] as u64;

    // (partial) h %= p — donna's sequential carry chain.
    let mut c = (d0 >> 26) as u64;
    let mut h0 = (d0 & LIMB_MASK as u64) as u32;
    d1 += c;
    c = (d1 >> 26) as u64;
    let mut h1 = (d1 & LIMB_MASK as u64) as u32;
    d2 += c;
    c = (d2 >> 26) as u64;
    let mut h2 = (d2 & LIMB_MASK as u64) as u32;
    d3 += c;
    c = (d3 >> 26) as u64;
    let mut h3 = (d3 & LIMB_MASK as u64) as u32;
    d4 += c;
    c = (d4 >> 26) as u64;
    let mut h4 = (d4 & LIMB_MASK as u64) as u32;
    h0 += (c * 5) as u32;
    c = (h0 >> 26) as u64;
    h0 &= LIMB_MASK;
    h1 += c as u32;

    state.h = [h0, h1, h2, h3, h4];
}

/// Finish: absorb the padded partial block (if any), fully carry `h`,
/// constant-time reduce `h mod p` (branchless mask select between `h` and
/// `h - p`), repack to 128 bits, add `s` (mod 2^128). One-shot: consumes
/// the state.
pub fn finish(state: Poly1305) -> [u8; TAG_LEN] {
    let mut state = state;

    if state.leftover > 0 {
        let mut block = [0u8; BLOCK_LEN];
        block[..state.leftover].copy_from_slice(&state.buffer[..state.leftover]);
        block[state.leftover] = 1; // the 2^(8*leftover) term...
                                   // ...and NO 2^128 hibit: donna final flag (hibit = 0).
        absorb_block(&mut state, &block, 0);
    }

    // Fully carry h (donna finish carry chain, incl. the 5c fold at h0).
    let h = &mut state.h;
    let mut c = h[1] >> 26;
    h[1] &= LIMB_MASK;
    h[2] += c;
    c = h[2] >> 26;
    h[2] &= LIMB_MASK;
    h[3] += c;
    c = h[3] >> 26;
    h[3] &= LIMB_MASK;
    h[4] += c;
    c = h[4] >> 26;
    h[4] &= LIMB_MASK;
    h[0] += (c * 5) as u32;
    c = h[0] >> 26;
    h[0] &= LIMB_MASK;
    h[1] += c as u32;

    // Compute g = h + 5 - 2^130 (= h - p when h >= p, negative otherwise).
    let mut g0 = h[0] as u64 + 5;
    let mut gc = g0 >> 26;
    g0 &= LIMB_MASK as u64;
    let g1 = h[1] as u64 + gc;
    gc = g1 >> 26;
    let g1 = g1 & LIMB_MASK as u64;
    let g2 = h[2] as u64 + gc;
    gc = g2 >> 26;
    let g2 = g2 & LIMB_MASK as u64;
    let g3 = h[3] as u64 + gc;
    gc = g3 >> 26;
    let g3 = g3 & LIMB_MASK as u64;
    let g4 = h[4] as i64 + (gc as i64) - (1 << 26);

    // Branchless select: g4 negative <=> h < p <=> keep h. Masks are
    // data-derived constants, not branch conditions; timing is flat.
    let neg = ((g4 >> 63) as u64) & 1;
    let mask_g = neg.wrapping_sub(1) as u64;
    let mask_h = !mask_g;
    let h0 = ((h[0] as u64 & mask_h) | (g0 & mask_g)) as u32;
    let h1 = ((h[1] as u64 & mask_h) | (g1 & mask_g)) as u32;
    let h2 = ((h[2] as u64 & mask_h) | (g2 & mask_g)) as u32;
    let h3 = ((h[3] as u64 & mask_h) | (g3 & mask_g)) as u32;
    let h4 = ((h[4] as u64 & mask_h) | ((g4 as u64 & LIMB_MASK as u64) & mask_g)) as u32;

    // Repack 26-bit limbs into 32-bit words (130 bits; bits 128+ drop).
    let w0 = h0 | (h1 << 26);
    let w1 = (h1 >> 6) | (h2 << 20);
    let w2 = (h2 >> 12) | (h3 << 14);
    let w3 = (h3 >> 18) | (h4 << 8);

    // mac = (h + s) mod 2^128, with a 32-bit carry chain.
    let p = &state.pad;
    let mut f: u64 = w0 as u64 + p[0] as u64;
    let t0 = f as u32;
    f = w1 as u64 + p[1] as u64 + (f >> 32);
    let t1 = f as u32;
    f = w2 as u64 + p[2] as u64 + (f >> 32);
    let t2 = f as u32;
    f = w3 as u64 + p[3] as u64 + (f >> 32);
    let t3 = f as u32;

    let mut tag = [0u8; TAG_LEN];
    tag[0..4].copy_from_slice(&t0.to_le_bytes());
    tag[4..8].copy_from_slice(&t1.to_le_bytes());
    tag[8..12].copy_from_slice(&t2.to_le_bytes());
    tag[12..16].copy_from_slice(&t3.to_le_bytes());
    tag
}

/// One-shot convenience: `new` + `push` + `finish`.
pub fn one_shot(key: &[u8; KEY_LEN], data: &[u8]) -> [u8; TAG_LEN] {
    let mut state = new(key);
    push(&mut state, data);
    finish(state)
}
