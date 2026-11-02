//! F2-11 bench-ingress — Divan benches over the real frame ingress surfaces.
//!
//! Benches:
//! (1) `cgap_admit_frame` — A04 Rust ingress path: zero-copy projection
//!     (`CgapFrameHeader::project`) + structural admission (`validate`) over
//!     a SplitMix64-seeded corpus of 128 B `CgapFrameHeader` regions, consts
//!     [1024, 65536, 1048576] frames.
//! (2) `a14_penstock_admission` — the A14 dam (`cgap_gate::penstock::ingress`)
//!     over the dam's own class mix (A14 §8.1: 50% garbage / 25% seeded
//!     mutation / 25% valid), consts [1048576, 10485760] admissions.
//!
//! TARGET row (A14 §8.4): full gate < 4 ns/frame. This file MEASURES; the
//! receipt records MEASURED verbatim — MET or GAP, never rounded.
//!
//! Chicago: real kernel, real memory, final state asserted — an untimed
//! preflight pass asserts every frame's verdict against its by-construction
//! manifest; every timed iteration asserts the per-pass admission checksum
//! (a drifted kernel or corpus flips it, loud). Corpus is a pure function of
//! fixed u64 seeds via canonical SplitMix64 (multiplier
//! 0xBF58_476D_1CE4_E5B9 verbatim; any variant is corpus corruption).
//! No Process.sleep.
//!
//! Memory disclosure: the dam bench materializes a 65,536-frame seeded pool
//! and cycles it (i & 65_535) across the N admissions of each const — a full
//! 10M-frame materialization is ~2.2 GB and hostile to fan-out lanes sharing
//! the host. Every processed frame is real packed memory fed to the real
//! gate; the pool is the same seeded corpus, N is admissions per pass.

use std::alloc::{alloc, dealloc, Layout};

use cgap_core::frame::{CgapFrameHeader, FrameError};
use cgap_gate::penstock as dam;
use divan::counter::ItemsCount;
use divan::{black_box, Bencher};

/// Fixed corpus seed (A04 bench). The corpus is a deterministic function of
/// this seed and the frame index.
const A04_SEED: u64 = 0x0A04_0000_C6A4_0001;

/// Fixed corpus seed (A14 penstock bench).
const A14_SEED: u64 = 0x0A14_0000_5EED_0001;

/// SplitMix64 — canonical constants verbatim (increment 0x9E3779B97F4A7C15,
/// multipliers 0xBF58476D1CE4E5B9 and 0x94D049BB133111EB).
struct SplitMix64(u64);

impl SplitMix64 {
    #[inline]
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

/// 64-byte-aligned frame region storage — `CgapFrameHeader` pins align 64
/// (A04 §6.4) and `project` refuses misaligned regions (`Misaligned`), so
/// the corpus must be a 64-aligned allocation with 128 B stride.
struct AlignedFrames {
    ptr: *mut u8,
    len: usize,
}

// SAFETY: owns its allocation; `&self` access is read-only shared slicing
// (`bytes`/`region`), the only mutable phase is corpus construction on the
// owning thread, and divan requires the bench closure (`Fn + Sync`) to share
// `&AlignedFrames` across its bench threads. No interior mutability exists.
unsafe impl Send for AlignedFrames {}
unsafe impl Sync for AlignedFrames {}

impl AlignedFrames {
    fn new(nbytes: usize) -> Self {
        let layout = Layout::from_size_align(nbytes, 64).expect("layout");
        let ptr = unsafe { alloc(layout) };
        assert!(!ptr.is_null(), "alloc failed for {nbytes} bytes");
        assert_eq!(ptr as usize % 64, 0, "allocator returned misaligned region");
        AlignedFrames { ptr, len: nbytes }
    }

    fn bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// Frame `i`'s 128 B header region (128 B stride preserves 64-alignment).
    fn region(&self, i: usize) -> &[u8; 128] {
        self.bytes()[i * 128..i * 128 + 128]
            .try_into()
            .expect("stride 128")
    }

    /// Mutable handle for corpus construction only (never in the timed loop).
    fn region_mut(&mut self, i: usize) -> &mut [u8; 128] {
        self.bytes_mut()[i * 128..]
            .first_chunk_mut::<128>()
            .expect("stride 128")
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl Drop for AlignedFrames {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.len, 64).expect("layout");
        unsafe { dealloc(self.ptr, layout) }
    }
}

/// FrameError folded to a stable small code for the admission checksum.
fn err_code(e: FrameError) -> u64 {
    match e {
        FrameError::BadMagic => 1,
        FrameError::BadVersion => 2,
        FrameError::ReservedBitsSet => 3,
        FrameError::AttestationMismatch => 4,
        FrameError::Misaligned => 5,
        FrameError::DigestMismatch => 6,
        FrameError::PayloadLenMismatch => 7,
        FrameError::BufferTooSmall => 8,
    }
}

const EC_OK: u64 = 0;

/// Build the A04 corpus: `n` frames at 128 B stride, each a deterministic
/// function of (A04_SEED, i). Class wheel over rng % 8: 0..=3 valid shapes,
/// 4 magic flip (`BadMagic`), 5 version bump (`BadVersion`), 6 reserved flag
/// bit (`ReservedBitsSet`), 7 attestation law break (sub-wheel:
/// att-magic / ceiling+1 / lineage-zero, all `AttestationMismatch`). Frames
/// are written through the REAL `CgapFrameHeader::encode_into`, then mutated
/// in the encoded bytes. Returns (buffer, manifest of expected err_codes,
/// expected per-pass checksum fold).
fn build_a04_corpus(n: usize) -> (AlignedFrames, Vec<u64>, u64) {
    use cgap_core::frame::{
        FLAG_ARROW, FLAG_BN3, FLAG_GITPACK, FLAG_PARQUET, FRAME_MAGIC, FRAME_VERSION,
        LICENSE_FLAG_DERIVATIVE, LICENSE_FLAG_ROOT, LICENSE_FLAG_TRANSPILED, LICENSE_MAGIC,
        SAFE_HARBOR_CEILING_CENTS_MAX,
    };

    let mut frames = AlignedFrames::new(n * 128);
    let mut manifest = Vec::with_capacity(n);
    let mut expected_fold: u64 = 0;
    for i in 0..n {
        let mut rng = SplitMix64(A04_SEED ^ (i as u64).wrapping_mul(0x2544_5491_4F6C_DD1D));
        let flags = match rng.next() % 3 {
            0 => FLAG_ARROW,
            1 => FLAG_BN3 | FLAG_GITPACK,
            _ => FLAG_PARQUET,
        };
        let lineage = match rng.next() % 3 {
            0 => LICENSE_FLAG_ROOT,
            1 => LICENSE_FLAG_DERIVATIVE,
            _ => LICENSE_FLAG_TRANSPILED,
        };
        let attestation = {
            let origin: [u8; 32] = core::array::from_fn(|_| rng.next() as u8);
            let receipt: [u8; 32] = if lineage == LICENSE_FLAG_ROOT {
                [0u8; 32]
            } else {
                core::array::from_fn(|_| rng.next() as u8)
            };
            cgap_core::frame::P28LicenseAttestation {
                license_magic: LICENSE_MAGIC,
                version: 1,
                attestation_flags: lineage,
                safe_harbor_ceiling_cents: SAFE_HARBOR_CEILING_CENTS_MAX,
                origin_authority_digest: origin,
                upstream_receipt: receipt,
            }
        };
        let payload_len: u64 = rng.next() % 256;
        let header = CgapFrameHeader {
            magic: FRAME_MAGIC,
            version: FRAME_VERSION,
            flags,
            reserved: [0; 2],
            payload_byte_len: payload_len,
            oabi_digest: core::array::from_fn(|_| rng.next() as u8),
            attestation,
        };
        header
            .encode_into(frames.region_mut(i))
            .expect("128-byte region");
        let expected: u64 = match rng.next() % 8 {
            0..=3 => EC_OK,
            4 => {
                frames.region_mut(i)[0] = b'X'; // §8.3: magic flip -> BadMagic
                err_code(FrameError::BadMagic)
            }
            5 => {
                frames.region_mut(i)[4] = 2; // version bump -> BadVersion
                err_code(FrameError::BadVersion)
            }
            6 => {
                frames.region_mut(i)[5] |= 0x10; // reserved flag bit -> ReservedBitsSet
                err_code(FrameError::ReservedBitsSet)
            }
            _ => {
                let region = frames.region_mut(i);
                match rng.next() % 3 {
                    0 => region[48..52].copy_from_slice(b"P29\0"),
                    1 => {
                        let mut c = u64::from_le_bytes(region[56..64].try_into().unwrap());
                        c = c.wrapping_add(1);
                        region[56..64].copy_from_slice(&c.to_le_bytes());
                    }
                    _ => {
                        region[54..56].copy_from_slice(&[0, 0]); // lineage zeroed
                    }
                }
                err_code(FrameError::AttestationMismatch)
            }
        };
        manifest.push(expected);
        expected_fold = expected_fold.wrapping_add(expected);
    }
    (frames, manifest, expected_fold)
}

/// One real A04 admission — zero-copy projection + structural validation,
/// folded to the checksum code space.
#[inline]
fn admit_a04(region: &[u8; 128]) -> u64 {
    match CgapFrameHeader::project(region) {
        Ok(h) => match h.validate() {
            Ok(()) => EC_OK,
            Err(e) => err_code(e),
        },
        Err(e) => err_code(e),
    }
}

/// Untimed preflight: every frame's real verdict must equal its
/// by-construction manifest row and the fold must equal the expected
/// checksum. Drift here is a corpus/kernel defect — loud, never silent.
fn preflight_a04(frames: &AlignedFrames, manifest: &[u64], expected_fold: u64) {
    let mut fold: u64 = 0;
    for (i, &want) in manifest.iter().enumerate() {
        let got = admit_a04(frames.region(i));
        assert_eq!(
            got, want,
            "A04 corpus frame {i}: verdict drifted from manifest"
        );
        fold = fold.wrapping_add(got);
    }
    assert_eq!(fold, expected_fold, "A04 corpus checksum drift");
}

/// A04 frame ingress (lane A04 §6/§8): projection + admission per frame.
/// Consts are corpus frames; the measured op is one full pass.
#[divan::bench(consts = [1024, 65536, 1048576])]
fn cgap_admit_frame<const FRAMES: usize>(b: Bencher) {
    // Corpus + manifest + expected checksum: built ONCE, outside the timed
    // region; preflight asserts every verdict before anything is timed.
    let (frames, manifest, expected_fold) = build_a04_corpus(FRAMES);
    preflight_a04(&frames, &manifest, expected_fold);

    b.counter(ItemsCount::new(FRAMES as u64))
        .bench(|| {
            let mut acc: u64 = 0;
            for i in 0..FRAMES {
                acc = acc.wrapping_add(admit_a04(frames.region(i)));
            }
            black_box(acc);
            assert_eq!(
                acc, expected_fold,
                "A04 pass checksum drifted from manifest fold"
            );
        });
}

/// Dam-corpus expectation (A14 §8.1 manifest shape): exact A19 code, or
/// "garbage must not be admitted" (the gate's fixed order owns which code).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DamExpect {
    Code(u32),
    RejectAny,
}

/// Dam class mix (A14 §8.1 over rng % 4): {0,1} garbage, {2} mutation, {3}
/// valid. Mutation wheel rng % 9 mirrors the dam court's classes:
/// magic / trunc / len-overrun / digest-zero / att-magic / att-range /
/// att-receipt / flags-reserved (row 6: ADMITTED, divergence pinned in
/// penstock.rs) / valid-combo. One divergence from the dam's own generator,
/// disclosed: garbage length is capped at 384 B (the dam court draws up to
/// 4096 B) so the packed pool stays bounded.
fn gen_dam_frame(seed: u64, i: u64, buf: &mut [u8]) -> (usize, DamExpect) {
    use dam::{
        CGAP_ERR_INVALID_MAGIC, CGAP_ERR_OABI_MISMATCH, CGAP_ERR_P28_SPOLIATION, CGAP_OK,
        FLAG_ARROW, FLAG_BN3, FLAG_GITPACK, FLAG_PARQUET, HEADER_LEN, MAGIC, OFF_ATTESTATION,
        OFF_FLAGS, OFF_MAGIC, OFF_OABI_DIGEST, OFF_PAYLOAD_BYTE_LEN, OFF_VERSION,
        OABI_DIGEST_LEN, P28_FLAG_DERIVATIVE, P28_FLAG_ROOT, P28_FLAG_TRANSPILED,
        P28_LICENSE_MAGIC, P28_SAFE_HARBOR_CEILING_CENTS, P28_VERSION,
    };
    let mut rng = SplitMix64(seed ^ i.wrapping_mul(0x2544_5491_4F6C_DD1D));
    buf.fill(0);
    let (frame_len, expected) = match rng.next() % 4 {
        0 | 1 => {
            // Garbage: uniform random bytes over a random length (<= 384 B).
            let n = (rng.next() as usize) % 384;
            let mut j = 0;
            while j < n {
                buf[j] = rng.next() as u8;
                j += 1;
            }
            (n, DamExpect::RejectAny)
        }
        _ => {
            // Valid v1 frame base, then optional single-law mutation.
            let payload_len = (rng.next() as usize) % 256;
            buf[OFF_MAGIC..OFF_MAGIC + 4].copy_from_slice(&MAGIC);
            buf[OFF_VERSION] = 1;
            let flags = match rng.next() % 4 {
                0 => FLAG_ARROW,
                1 => FLAG_PARQUET,
                2 => FLAG_BN3,
                _ => FLAG_GITPACK,
            };
            buf[OFF_FLAGS] = flags;
            buf[OFF_PAYLOAD_BYTE_LEN..OFF_PAYLOAD_BYTE_LEN + 8]
                .copy_from_slice(&(payload_len as u64).to_le_bytes());
            for b in buf[OFF_OABI_DIGEST..OFF_OABI_DIGEST + OABI_DIGEST_LEN].iter_mut() {
                *b = rng.next() as u8;
            }
            buf[OFF_ATTESTATION..OFF_ATTESTATION + 4].copy_from_slice(&P28_LICENSE_MAGIC);
            buf[OFF_ATTESTATION + 4..OFF_ATTESTATION + 6]
                .copy_from_slice(&P28_VERSION.to_le_bytes());
            let lineage = match rng.next() % 3 {
                0 => P28_FLAG_ROOT,
                1 => P28_FLAG_DERIVATIVE,
                _ => P28_FLAG_TRANSPILED,
            };
            buf[OFF_ATTESTATION + 6..OFF_ATTESTATION + 8].copy_from_slice(&lineage.to_le_bytes());
            buf[OFF_ATTESTATION + 8..OFF_ATTESTATION + 16]
                .copy_from_slice(&P28_SAFE_HARBOR_CEILING_CENTS.to_le_bytes());
            let origin: [u8; 32] = core::array::from_fn(|_| rng.next() as u8);
            buf[OFF_ATTESTATION + 16..OFF_ATTESTATION + 48].copy_from_slice(&origin);
            if lineage != P28_FLAG_ROOT {
                let at = OFF_ATTESTATION + 48;
                let receipt: [u8; 32] = core::array::from_fn(|_| rng.next() as u8);
                buf[at..at + 32].copy_from_slice(&receipt);
            }
            let frame_len = HEADER_LEN + payload_len;
            if rng.next() % 2 == 0 {
                // Valid frame (also covers the mutation/valid 50/50 split of
                // the non-garbage half: rng % 2 picks which).
                (frame_len, DamExpect::Code(CGAP_OK))
            } else {
                let expected = match rng.next() % 9 {
                    0 => {
                        // MC_MAGIC: row 2.
                        let bad: [u8; 4] = match rng.next() % 3 {
                            0 => *b"XGAP",
                            1 => *b"cgap",
                            _ => [0u8; 4],
                        };
                        buf[OFF_MAGIC..OFF_MAGIC + 4].copy_from_slice(&bad);
                        DamExpect::Code(CGAP_ERR_INVALID_MAGIC)
                    }
                    1 => {
                        // MC_TRUNC: row 1 — declared buffer shorter than the header.
                        let n = HEADER_LEN - 1 - (rng.next() as usize) % HEADER_LEN;
                        return (n, DamExpect::Code(CGAP_ERR_INVALID_MAGIC));
                    }
                    2 => {
                        // MC_LEN_OVERRUN: row 3b.
                        let overrun = payload_len as u64 + 1 + rng.next() % 4096;
                        buf[OFF_PAYLOAD_BYTE_LEN..OFF_PAYLOAD_BYTE_LEN + 8]
                            .copy_from_slice(&overrun.to_le_bytes());
                        DamExpect::Code(CGAP_ERR_OABI_MISMATCH)
                    }
                    3 => {
                        // MC_DIGEST_ZERO: row 3a.
                        for b in buf[OFF_OABI_DIGEST..OFF_OABI_DIGEST + OABI_DIGEST_LEN].iter_mut()
                        {
                            *b = 0;
                        }
                        DamExpect::Code(CGAP_ERR_OABI_MISMATCH)
                    }
                    4 => {
                        // MC_ATT_MAGIC: row 4.
                        let bad: [u8; 4] = match rng.next() % 3 {
                            0 => *b"P29\0",
                            1 => [0u8; 4],
                            _ => *b"p28s",
                        };
                        buf[OFF_ATTESTATION..OFF_ATTESTATION + 4].copy_from_slice(&bad);
                        DamExpect::Code(CGAP_ERR_P28_SPOLIATION)
                    }
                    5 => {
                        // MC_ATT_RANGE: reserved bit / zero lineage / over-ceiling.
                        match rng.next() % 3 {
                            0 => {
                                let f =
                                    lineage | (1u16 << (3 + rng.next() % 13));
                                buf[OFF_ATTESTATION + 6..OFF_ATTESTATION + 8]
                                    .copy_from_slice(&f.to_le_bytes());
                            }
                            1 => {
                                let f = match rng.next() % 2 {
                                    0 => 0u16,
                                    _ => {
                                        let base = lineage.trailing_zeros();
                                        let extra =
                                            1u16 << ((base + 1 + (rng.next() % 2) as u32) % 3);
                                        lineage | extra
                                    }
                                };
                                buf[OFF_ATTESTATION + 6..OFF_ATTESTATION + 8]
                                    .copy_from_slice(&f.to_le_bytes());
                            }
                            _ => {
                                let over = P28_SAFE_HARBOR_CEILING_CENTS
                                    + 1
                                    + rng.next() % 1_000_000;
                                buf[OFF_ATTESTATION + 8..OFF_ATTESTATION + 16]
                                    .copy_from_slice(&over.to_le_bytes());
                            }
                        }
                        DamExpect::Code(CGAP_ERR_P28_SPOLIATION)
                    }
                    6 => {
                        // MC_ATT_RECEIPT: receipt-law flip, direction pinned by
                        // the base lineage so the law is always BROKEN.
                        let at = OFF_ATTESTATION + 48;
                        if lineage == P28_FLAG_ROOT {
                            let r: [u8; 32] = core::array::from_fn(|_| rng.next() as u8);
                            buf[at..at + 32].copy_from_slice(&r);
                        } else {
                            for b in buf[at..at + 32].iter_mut() {
                                *b = 0;
                            }
                        }
                        DamExpect::Code(CGAP_ERR_P28_SPOLIATION)
                    }
                    7 => {
                        // MC_FLAGS_RESERVED: row 6 — ADMITTED (A04 §6.3 "ignored
                        // on read"; divergence pinned in penstock.rs).
                        buf[OFF_FLAGS] = flags | 0x10;
                        DamExpect::Code(CGAP_OK)
                    }
                    _ => {
                        // MC_V_COMBOS: valid flag combination.
                        let combo = (rng.next() as u8) & 0x0F;
                        buf[OFF_FLAGS] = combo;
                        DamExpect::Code(CGAP_OK)
                    }
                };
                (frame_len, expected)
            }
        }
    };
    (frame_len, expected)
}

/// Build the dam pool: `pool` seeded frames packed contiguously with an
/// (offset, len) table, per-frame manifest asserted by an untimed preflight
/// pass (every RejectAny frame must NOT admit; every Code frame must return
/// exactly its manifest code), returning the packed bytes, the table, and
/// the preflight-recorded fold of ACTUAL verdicts (the checksum basis for
/// the timed passes).
fn build_dam_pool(seed: u64, pool: usize) -> (Vec<u8>, Vec<(u32, u32)>, u64) {
    let mut scratch = vec![0u8; 512];
    let mut packed: Vec<u8> = Vec::with_capacity(pool * 288);
    let mut table: Vec<(u32, u32)> = Vec::with_capacity(pool);
    let mut fold: u64 = 0;
    for i in 0..pool {
        let (len, expected) = gen_dam_frame(seed, i as u64, &mut scratch);
        let off = packed.len();
        packed.extend_from_slice(&scratch[..len]);
        table.push((off as u32, len as u32));
        let mut view = dam::FrameView { payload: &[], prefilter: 0, escalate: false };
        let got = dam::ingress(&packed[off..off + len], len, Some(&mut view));
        match expected {
            DamExpect::Code(code) => assert_eq!(
                got, code,
                "dam pool frame {i}: verdict drifted from manifest"
            ),
            DamExpect::RejectAny => assert_ne!(
                got,
                dam::CGAP_OK,
                "dam pool frame {i}: garbage frame was ADMITTED"
            ),
        }
        if got == dam::CGAP_OK {
            let declared = u64::from_le_bytes(
                packed[off + dam::OFF_PAYLOAD_BYTE_LEN..off + dam::OFF_PAYLOAD_BYTE_LEN + 8]
                    .try_into()
                    .expect("u64 at offset 8"),
            );
            assert_eq!(
                view.payload.len() as u64, declared,
                "dam pool frame {i}: view geometry drifted"
            );
            assert_eq!(
                view.payload.as_ptr(),
                packed[off + dam::HEADER_LEN..].as_ptr(),
                "dam pool frame {i}: view is not zero-copy at the frame's payload base"
            );
        }
        fold = fold.wrapping_add(got as u64);
    }
    (packed, table, fold)
}

/// A14 penstock admission (lane A14 §4/§8): the dam gate over the seeded
/// mixed-class corpus. Consts are admissions per pass; the pass cycles a
/// 65,536-frame seeded pool (disclosed in the module header).
#[divan::bench(consts = [1048576, 10485760])]
fn a14_penstock_admission<const FRAMES: usize>(b: Bencher) {
    const POOL: usize = 65_536;
    assert!(
        FRAMES % POOL == 0,
        "FRAMES must be a pool multiple (checksum math)"
    );
    let (packed, table, pool_fold) = build_dam_pool(A14_SEED, POOL);
    let laps = (FRAMES / POOL) as u64;
    let expected_pass_fold = pool_fold.wrapping_mul(laps);

    b.counter(ItemsCount::new(FRAMES as u64))
        .bench(|| {
            let mut acc: u64 = 0;
            for i in 0..FRAMES {
                let (off, len) = table[i & (POOL - 1)];
                let frame = &packed[off as usize..off as usize + len as usize];
                let mut view = dam::FrameView { payload: &[], prefilter: 0, escalate: false };
                let st = dam::ingress(frame, len as usize, Some(&mut view));
                acc = acc.wrapping_add(st as u64);
            }
            black_box(acc);
            assert_eq!(
                acc, expected_pass_fold,
                "dam pass checksum drifted from preflight fold"
            );
        });
}

fn main() {
    divan::main();
}
