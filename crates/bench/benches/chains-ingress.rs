//! BENCH-1 chains-ingress — the CHAINED ingress-security pipeline per frame.
//!
//! Business case (the 80/20): wire-speed ingress security — malformed-frame
//! dropping at the DMA boundary is the highest-volume hot path; every frame
//! pays it. This file measures the chain END TO END and every stage on the
//! SAME seeded corpus, so the Pareto split is read off one table, not
//! stitched across files.
//!
//! Chain per frame (fixed order):
//!   1. A04 projection/validate — `CgapFrameHeader::project` (zero-copy,
//!      64-aligned 128 B header region, A04 §6.4) + `validate`
//!      (structural admission: magic/version/reserved-flags/A17 §2.2
//!      attestation laws).
//!   2. A14 penstock admission — `cgap_gate::penstock::ingress` (the dam,
//!      §4 normative check order) minting the zero-copy `FrameView`.
//!   3. Admitted path only — fold the payload digest: `cgap_blake3::hash`
//!      over the dam-minted payload view (the `oabi_digest` primitive,
//!      A04 §6.1), first 8 LE bytes folded into the checksum.
//!   A frame is chain-ADMITTED iff stage 1 AND stage 2 admit; only then is
//!   stage 3 paid.
//!
//! Benches (all four over one shared corpus, ItemsCount = chain frames, so
//! the ns/frame rows compare directly in chain-frame units):
//!   - `ingress_chain`              — the full 3-stage chain (the product).
//!   - `stage_a04_project_validate` — stage 1 alone.
//!   - `stage_penstock_admission`   — stage 2 alone.
//!   - `stage_digest_fold`          — stage 3 alone (admitted subset).
//!
//! Chicago: real kernels, real packed memory, final state asserted. An
//! untimed preflight asserts every frame's manifest (stage-2 codes exact by
//! construction, garbage must-not-admit, dam-admit view geometry, and a
//! preflight/chain cross-check per frame) and records the actual pass
//! folds; every timed iteration asserts its checksum against the recorded
//! folds — a drifted kernel or corpus flips it loud. Corpus is a pure
//! function of fixed u64 seeds via canonical SplitMix64 (multiplier
//! 0xBF58_476D_1CE4_E5B9 verbatim; any variant is corpus corruption).
//! No Process.sleep.
//!
//! Memory disclosure: a 65,536-slot pool at 512 B/slot = 32 MiB, cycled
//! (i & 65_535) across the FRAMES of each const — a full 1M-slot
//! materialization would be 512 MiB, hostile to fan-out lanes sharing the
//! host. Every processed frame is real packed memory fed to the real
//! chain; N is chain admissions per pass.

use std::alloc::{alloc, dealloc, Layout};
use std::sync::atomic::{AtomicU64, Ordering};

use cgap_core::frame::{CgapFrameHeader, FrameError, FRAME_HEADER_LEN};
use cgap_gate::penstock as dam;
use divan::counter::ItemsCount;
use divan::{black_box, Bencher};

/// Fixed corpus seed (BENCH-1 chains-ingress). The corpus is a
/// deterministic function of this seed and the frame index.
const CHAIN_SEED: u64 = 0xC001_0000_5EED_0001;

/// DMA slot capacity: 128 B header + up to 384 B body. The corpus caps
/// garbage at 384 B and payloads at 256 B, so 512 covers every frame.
const SLOT_CAP: usize = 512;

/// Slots per pool (power of two — index masking). 512 B x 65,536 = 32 MiB.
const POOL: usize = 65_536;

/// Stage-1 fold code for a frame shorter than the canonical header: no
/// 128 B region exists to project (stage 1 refuses before `project`).
const EC_SHORT: u64 = 9;

/// EC_OK (A04 admit) — the stage-1 code space's zero.
const EC_OK: u64 = 0;

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

/// 64-byte-aligned slot storage — `CgapFrameHeader` pins align 64 (A04
/// §6.4) and `project` refuses misaligned regions (`Misaligned`), so every
/// slot base must be 64-aligned; the 512 B stride preserves that.
struct AlignedSlots {
    ptr: *mut u8,
    len: usize,
}

// SAFETY: owns its allocation; shared access is read-only slicing, the only
// mutable phase is corpus construction on the owning thread, and divan
// requires the bench closure (`Fn`) to share `&Pool` across its bench
// threads. No interior mutability exists.
unsafe impl Send for AlignedSlots {}
unsafe impl Sync for AlignedSlots {}

impl AlignedSlots {
    fn new(nbytes: usize) -> Self {
        let layout = Layout::from_size_align(nbytes, 64).expect("layout");
        let ptr = unsafe { alloc(layout) };
        assert!(!ptr.is_null(), "alloc failed for {nbytes} bytes");
        assert_eq!(ptr as usize % 64, 0, "allocator returned misaligned region");
        AlignedSlots { ptr, len: nbytes }
    }

    fn bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.ptr, self.len) }
    }

    /// Mutable handle for corpus construction only (never in timed loops).
    fn bytes_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl Drop for AlignedSlots {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.len, 64).expect("layout");
        unsafe { dealloc(self.ptr, layout) }
    }
}

// ---------------------------------------------------------------------------
// The chain — one implementation, composed by every bench (no drift).
// ---------------------------------------------------------------------------

/// Stage 1 — A04 projection/validate over the canonical 128 B header
/// region, folded to the checksum code space.
#[inline]
fn stage1(slot: &[u8], len: usize) -> u64 {
    if len < FRAME_HEADER_LEN {
        return EC_SHORT;
    }
    let region: &[u8; FRAME_HEADER_LEN] = slot[..FRAME_HEADER_LEN]
        .try_into()
        .expect("len >= FRAME_HEADER_LEN proven above");
    match CgapFrameHeader::project(region) {
        Ok(h) => match h.validate() {
            Ok(()) => EC_OK,
            Err(e) => err_code(e),
        },
        Err(e) => err_code(e),
    }
}

/// Stage 2 — A14 penstock admission over the declared frame length (the
/// transport's truth). Mints the zero-copy payload view on admit.
#[inline]
fn stage2<'a>(slot: &'a [u8], len: usize, view: Option<&mut dam::FrameView<'a>>) -> u32 {
    dam::ingress(slot, len, view)
}

/// Stage 3 — the ADMITTED path's fold, now two-stage (v26.11.2 lever L1):
/// escalated frames pay full BLAKE3 over the view's payload (first 8 LE
/// bytes folded); non-escalated frames fold the dam-minted prefilter mix —
/// no crypto paid on the clean subset.
#[inline]
fn stage3_escalated(view: &dam::FrameView) -> u64 {
    if view.escalate {
        let d = cgap_blake3::hash(view.payload);
        u64::from_le_bytes(d[..8].try_into().expect("32-byte digest"))
    } else {
        view.prefilter
    }
}

/// The CHAIN — one frame through all three stages. `slot` is a 512 B
/// 64-aligned DMA slot; `len` is the declared frame length. Returns the
/// frame's folded score: stage-1 code + stage-2 code + (admitted path
/// only) the two-stage fold — escalated frames pay the BLAKE3 digest,
/// clean frames fold the dam-minted prefilter (lever L1). Admit law:
/// chain-admitted iff BOTH stage 1 and stage 2 admit; only then is the
/// fold paid.
#[inline]
fn chain_frame(slot: &[u8], len: usize) -> u64 {
    let a04 = stage1(slot, len);
    let mut view = dam::FrameView {
        payload: &[],
        prefilter: 0,
        escalate: false,
    };
    let st = stage2(slot, len, Some(&mut view));
    let fold = if a04 == EC_OK && st == dam::CGAP_OK {
        stage3_escalated(&view)
    } else {
        0
    };
    a04.wrapping_add(u64::from(st)).wrapping_add(fold)
}

/// FrameError folded to a stable small code for the stage-1 checksum
/// (same code space as the F2-11 ingress bench).
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

// ---------------------------------------------------------------------------
// Seeded corpus — the dam classes (A14 §8.1), one frame per 512 B slot.
// ---------------------------------------------------------------------------

/// Dam-corpus expectation (A14 §8.1 manifest shape): exact A19 code, or
/// "garbage must not be admitted" (the gate's fixed order owns which code).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DamExpect {
    Code(u32),
    RejectAny,
}

/// Corpus frame generator — the dam classes (A14 §8.1 mix over rng % 4:
/// {0,1} garbage, {2} mutation, {3} valid; mutation wheel rng % 9 mirrors
/// the dam court's classes; MC_FLAGS_RESERVED is ADMITTED by the dam per
/// the pinned row-6 divergence, but stage 1 refuses the reserved frame
/// flag bit, so the chain still rejects that frame). One disclosed cap vs
/// the dam court's own generator: garbage length <= 384 B and payload
/// <= 256 B so every frame fits one 512 B DMA slot.
fn gen_frame(seed: u64, i: u64, slot: &mut [u8]) -> (usize, DamExpect) {
    use dam::{
        CGAP_ERR_INVALID_MAGIC, CGAP_ERR_OABI_MISMATCH, CGAP_ERR_P28_SPOLIATION, CGAP_OK,
        FLAG_ARROW, FLAG_BN3, FLAG_GITPACK, FLAG_PARQUET, HEADER_LEN as DAM_HEADER_LEN, MAGIC,
        OABI_DIGEST_LEN, OFF_ATTESTATION, OFF_FLAGS, OFF_MAGIC, OFF_OABI_DIGEST,
        OFF_PAYLOAD_BYTE_LEN, OFF_VERSION, P28_FLAG_DERIVATIVE, P28_FLAG_ROOT, P28_FLAG_TRANSPILED,
        P28_LICENSE_MAGIC, P28_SAFE_HARBOR_CEILING_CENTS, P28_VERSION,
    };
    let mut rng = SplitMix64(seed ^ i.wrapping_mul(0x2544_5491_4F6C_DD1D));
    slot.fill(0);
    let (frame_len, expected) = match rng.next() % 4 {
        0 | 1 => {
            // Garbage: uniform random bytes over a random length (<= 384 B).
            let n = (rng.next() as usize) % 384;
            let mut j = 0;
            while j < n {
                slot[j] = rng.next() as u8;
                j += 1;
            }
            (n, DamExpect::RejectAny)
        }
        _ => {
            // Valid v1 frame base, then optional single-law mutation.
            let payload_len = (rng.next() as usize) % 256;
            slot[OFF_MAGIC..OFF_MAGIC + 4].copy_from_slice(&MAGIC);
            slot[OFF_VERSION] = 1;
            let flags = match rng.next() % 4 {
                0 => FLAG_ARROW,
                1 => FLAG_PARQUET,
                2 => FLAG_BN3,
                _ => FLAG_GITPACK,
            };
            slot[OFF_FLAGS] = flags;
            slot[OFF_PAYLOAD_BYTE_LEN..OFF_PAYLOAD_BYTE_LEN + 8]
                .copy_from_slice(&(payload_len as u64).to_le_bytes());
            for b in slot[OFF_OABI_DIGEST..OFF_OABI_DIGEST + OABI_DIGEST_LEN].iter_mut() {
                *b = rng.next() as u8;
            }
            slot[OFF_ATTESTATION..OFF_ATTESTATION + 4].copy_from_slice(&P28_LICENSE_MAGIC);
            slot[OFF_ATTESTATION + 4..OFF_ATTESTATION + 6]
                .copy_from_slice(&P28_VERSION.to_le_bytes());
            let lineage = match rng.next() % 3 {
                0 => P28_FLAG_ROOT,
                1 => P28_FLAG_DERIVATIVE,
                _ => P28_FLAG_TRANSPILED,
            };
            slot[OFF_ATTESTATION + 6..OFF_ATTESTATION + 8].copy_from_slice(&lineage.to_le_bytes());
            slot[OFF_ATTESTATION + 8..OFF_ATTESTATION + 16]
                .copy_from_slice(&P28_SAFE_HARBOR_CEILING_CENTS.to_le_bytes());
            let origin: [u8; 32] = core::array::from_fn(|_| rng.next() as u8);
            slot[OFF_ATTESTATION + 16..OFF_ATTESTATION + 48].copy_from_slice(&origin);
            if lineage != P28_FLAG_ROOT {
                let at = OFF_ATTESTATION + 48;
                let receipt: [u8; 32] = core::array::from_fn(|_| rng.next() as u8);
                slot[at..at + 32].copy_from_slice(&receipt);
            }
            let frame_len = DAM_HEADER_LEN + payload_len;
            if rng.next() % 2 == 0 {
                // Valid frame (also covers the mutation/valid 50/50 split of
                // the non-garbage half: rng % 2 picks which).
                (frame_len, DamExpect::Code(CGAP_OK))
            } else {
                let expected = match rng.next() % 9 {
                    // MC_MAGIC: dam row 2.
                    0 => {
                        let bad: [u8; 4] = match rng.next() % 3 {
                            0 => *b"XGAP",
                            1 => *b"cgap",
                            _ => [0u8; 4],
                        };
                        slot[OFF_MAGIC..OFF_MAGIC + 4].copy_from_slice(&bad);
                        DamExpect::Code(CGAP_ERR_INVALID_MAGIC)
                    }
                    // MC_TRUNC: dam row 1 — declared buffer shorter than the header.
                    1 => {
                        let n = DAM_HEADER_LEN - 1 - (rng.next() as usize) % DAM_HEADER_LEN;
                        return (n, DamExpect::Code(CGAP_ERR_INVALID_MAGIC));
                    }
                    // MC_LEN_OVERRUN: dam row 3b.
                    2 => {
                        let overrun = payload_len as u64 + 1 + rng.next() % 4096;
                        slot[OFF_PAYLOAD_BYTE_LEN..OFF_PAYLOAD_BYTE_LEN + 8]
                            .copy_from_slice(&overrun.to_le_bytes());
                        DamExpect::Code(CGAP_ERR_OABI_MISMATCH)
                    }
                    // MC_DIGEST_ZERO: dam row 3a.
                    3 => {
                        for b in slot[OFF_OABI_DIGEST..OFF_OABI_DIGEST + OABI_DIGEST_LEN].iter_mut()
                        {
                            *b = 0;
                        }
                        DamExpect::Code(CGAP_ERR_OABI_MISMATCH)
                    }
                    // MC_ATT_MAGIC: dam row 4.
                    4 => {
                        let bad: [u8; 4] = match rng.next() % 3 {
                            0 => *b"P29\0",
                            1 => [0u8; 4],
                            _ => *b"p28s",
                        };
                        slot[OFF_ATTESTATION..OFF_ATTESTATION + 4].copy_from_slice(&bad);
                        DamExpect::Code(CGAP_ERR_P28_SPOLIATION)
                    }
                    // MC_ATT_RANGE: reserved bit / zero-or-double lineage / over-ceiling.
                    5 => {
                        match rng.next() % 3 {
                            0 => {
                                let f = lineage | (1u16 << (3 + rng.next() % 13));
                                slot[OFF_ATTESTATION + 6..OFF_ATTESTATION + 8]
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
                                slot[OFF_ATTESTATION + 6..OFF_ATTESTATION + 8]
                                    .copy_from_slice(&f.to_le_bytes());
                            }
                            _ => {
                                let over =
                                    P28_SAFE_HARBOR_CEILING_CENTS + 1 + rng.next() % 1_000_000;
                                slot[OFF_ATTESTATION + 8..OFF_ATTESTATION + 16]
                                    .copy_from_slice(&over.to_le_bytes());
                            }
                        }
                        DamExpect::Code(CGAP_ERR_P28_SPOLIATION)
                    }
                    // MC_ATT_RECEIPT: receipt-law flip, direction pinned by the
                    // base lineage so the law is always BROKEN.
                    6 => {
                        let at = OFF_ATTESTATION + 48;
                        if lineage == P28_FLAG_ROOT {
                            let r: [u8; 32] = core::array::from_fn(|_| rng.next() as u8);
                            slot[at..at + 32].copy_from_slice(&r);
                        } else {
                            for b in slot[at..at + 32].iter_mut() {
                                *b = 0;
                            }
                        }
                        DamExpect::Code(CGAP_ERR_P28_SPOLIATION)
                    }
                    // MC_FLAGS_RESERVED: dam row 6 — ADMITTED by the dam (A04
                    // §6.3 "ignored on read"; divergence pinned in penstock.rs),
                    // refused by stage 1 (A04 validate ReservedBitsSet).
                    7 => {
                        slot[OFF_FLAGS] = flags | 0x10;
                        DamExpect::Code(CGAP_OK)
                    }
                    // MC_V_COMBOS: valid flag combination.
                    _ => {
                        let combo = (rng.next() as u8) & 0x0F;
                        slot[OFF_FLAGS] = combo;
                        DamExpect::Code(CGAP_OK)
                    }
                };
                (frame_len, expected)
            }
        }
    };
    (frame_len, expected)
}

// ---------------------------------------------------------------------------
// Pool build + untimed preflight (R3: corpus construction outside every
// measured region; preflight asserts the manifest and records the ACTUAL
// pass folds that the timed passes' checksums must reproduce).
// ---------------------------------------------------------------------------

/// The pool: 65,536 seeded frames in 64-aligned 512 B slots, per-frame
/// declared lengths, the preflight-recorded actual pass folds (checksum
/// basis), and the admitted-subset table for the stage-3 bench.
struct Pool {
    slots: AlignedSlots,
    lens: Vec<u32>,
    chain_fold: u64,
    a04_fold: u64,
    dam_fold: u64,
    /// L1 escalated-subset fold: BLAKE3 digests summed over chain-admitted
    /// AND escalated frames only (lever L1; may legitimately be zero when
    /// the corpus holds no escalated+admitted frame — receipt-recorded).
    digest_fold: u64,
    /// Chain-admitted, NON-escalated frames' prefilter mix fold (lever L1).
    prefilter_fold: u64,
    /// Chain-admitted windows (cross-check basis; not iterated by a leg).
    #[allow(dead_code)]
    admitted: Vec<(u32, u32)>,
    /// L1 escalated subset (chain-admitted AND escalate): stage-3 digest
    /// bench windows. May be empty on this corpus (receipt-recorded).
    escalated: Vec<(u32, u32)>,
    /// L1 clean subset (chain-admitted, NOT escalate): prefilter bench.
    clean: Vec<(u32, u32)>,
}

/// Build + preflight the pool. Preflight per frame: stage-2 verdict must
/// equal its by-construction manifest row; garbage must NOT be admitted
/// (nor chain-admitted); every dam-admitted frame's view must be exact
/// geometry and zero-copy at the slot's payload base; and the inline
/// preflight fold must equal a real `chain_frame` call on the same bytes
/// (preflight/chain drift is corpus corruption, loud).
fn build_pool(seed: u64, n: usize) -> Pool {
    assert_eq!(n, POOL, "pool size is fixed (index masking)");
    let mut slots = AlignedSlots::new(n * SLOT_CAP);
    let mut lens = Vec::with_capacity(n);
    let mut scratch = vec![0u8; SLOT_CAP];
    let mut chain_fold: u64 = 0;
    let mut a04_fold: u64 = 0;
    let mut dam_fold: u64 = 0;
    let mut digest_fold: u64 = 0;
    let mut prefilter_fold: u64 = 0;
    let mut admitted: Vec<(u32, u32)> = Vec::with_capacity(n / 4 + 16);
    let mut escalated: Vec<(u32, u32)> = Vec::with_capacity(16);
    let mut clean: Vec<(u32, u32)> = Vec::with_capacity(n / 4 + 16);
    for i in 0..n {
        let (len, expect) = gen_frame(seed, i as u64, &mut scratch);
        assert!(len <= SLOT_CAP, "frame {i} exceeds its DMA slot");
        let base = i * SLOT_CAP;
        slots.bytes_mut()[base..base + len].copy_from_slice(&scratch[..len]);
        lens.push(len as u32);

        let slot: &[u8] = &slots.bytes()[base..base + SLOT_CAP];
        let a04 = stage1(slot, len);
        let mut view = dam::FrameView {
            payload: &[],
            prefilter: 0,
            escalate: false,
        };
        let st = stage2(slot, len, Some(&mut view));
        match expect {
            DamExpect::Code(code) => {
                assert_eq!(st, code, "frame {i}: stage-2 verdict drifted from manifest")
            }
            DamExpect::RejectAny => {
                assert_ne!(st, dam::CGAP_OK, "frame {i}: garbage frame was ADMITTED")
            }
        }
        if st == dam::CGAP_OK {
            let declared = u64::from_le_bytes(
                slot[dam::OFF_PAYLOAD_BYTE_LEN..dam::OFF_PAYLOAD_BYTE_LEN + 8]
                    .try_into()
                    .expect("u64 at offset 8"),
            );
            assert_eq!(
                view.payload.len() as u64,
                declared,
                "frame {i}: view geometry drifted"
            );
            assert_eq!(
                view.payload.as_ptr(),
                slot[FRAME_HEADER_LEN..].as_ptr(),
                "frame {i}: view is not zero-copy at the slot's payload base"
            );
        }
        let chain_admitted = a04 == EC_OK && st == dam::CGAP_OK;
        if expect == DamExpect::RejectAny {
            assert!(
                !chain_admitted,
                "frame {i}: garbage frame was chain-ADMITTED"
            );
        }
        let digest = if chain_admitted {
            if view.escalate {
                let d = cgap_blake3::hash(view.payload);
                u64::from_le_bytes(d[..8].try_into().expect("32-byte digest"))
            } else {
                view.prefilter
            }
        } else {
            0
        };
        if chain_admitted {
            admitted.push(((base + FRAME_HEADER_LEN) as u32, view.payload.len() as u32));
        }
        // Cross-check: the inline preflight fold and the real chain must
        // agree frame-for-frame — the timed checksum's basis is the chain.
        assert_eq!(
            chain_frame(slot, len),
            a04.wrapping_add(u64::from(st)).wrapping_add(digest),
            "frame {i}: preflight/chain fold drift"
        );
        // L1 law: the prefilter is minted on ADMIT only; a reject frame's
        // view must carry the zero default.
        if st == dam::CGAP_OK {
            assert_eq!(
                dam::prefilter8(&slot[..8]),
                view.prefilter,
                "frame {i}: dam-minted prefilter drift"
            );
        } else {
            assert_eq!(view.prefilter, 0, "frame {i}: reject view prefilter");
            assert!(!view.escalate, "frame {i}: reject view escalate bit");
        }
        if chain_admitted && view.escalate {
            digest_fold = digest_fold.wrapping_add(digest);
            escalated.push(((base + FRAME_HEADER_LEN) as u32, view.payload.len() as u32));
        } else if chain_admitted {
            prefilter_fold = prefilter_fold.wrapping_add(view.prefilter);
            clean.push(((base + FRAME_HEADER_LEN) as u32, view.payload.len() as u32));
        }
        chain_fold = chain_fold.wrapping_add(a04.wrapping_add(u64::from(st)).wrapping_add(digest));
        a04_fold = a04_fold.wrapping_add(a04);
        dam_fold = dam_fold.wrapping_add(u64::from(st));
    }
    assert!(
        !admitted.is_empty(),
        "degenerate corpus: zero chain-admitted frames"
    );
    Pool {
        slots,
        lens,
        chain_fold,
        a04_fold,
        dam_fold,
        digest_fold,
        prefilter_fold,
        admitted,
        escalated,
        clean,
    }
}

// ---------------------------------------------------------------------------
// Benches — chain + three stages, one shared corpus, ItemsCount = chain
// frames so every ns/frame row is in the same units. Consts are chain
// admissions per pass; each pass cycles the 65,536-slot pool and asserts
// its checksum against the preflight fold.
// ---------------------------------------------------------------------------

/// The chained pipeline (the product): A04 projection/validate -> A14
/// penstock admission -> (admitted path only) digest fold.
#[divan::bench(consts = [65_536, 1_048_576])]
fn ingress_chain<const FRAMES: usize>(b: Bencher) {
    let pool = build_pool(CHAIN_SEED, POOL);
    let laps = (FRAMES / POOL) as u64;
    assert_eq!(
        FRAMES % POOL,
        0,
        "FRAMES must be a pool multiple (checksum math)"
    );
    let expect = pool.chain_fold.wrapping_mul(laps);
    b.counter(ItemsCount::new(FRAMES as u64)).bench(|| {
        let bytes = pool.slots.bytes();
        let t0 = std::time::Instant::now();
        let mut acc: u64 = 0;
        for i in 0..FRAMES {
            let j = i & (POOL - 1);
            let slot = &bytes[j * SLOT_CAP..(j + 1) * SLOT_CAP];
            acc = acc.wrapping_add(chain_frame(slot, pool.lens[j] as usize));
        }
        record_pass(t0.elapsed(), FRAMES);
        black_box(acc);
        assert_eq!(
            acc, expect,
            "chain pass checksum drifted from preflight fold"
        );
    });
}

/// Stage 1 alone — A04 projection + structural validation.
#[divan::bench(consts = [65_536, 1_048_576])]
fn stage_a04_project_validate<const FRAMES: usize>(b: Bencher) {
    let pool = build_pool(CHAIN_SEED, POOL);
    let laps = (FRAMES / POOL) as u64;
    assert_eq!(FRAMES % POOL, 0, "FRAMES must be a pool multiple");
    let expect = pool.a04_fold.wrapping_mul(laps);
    b.counter(ItemsCount::new(FRAMES as u64)).bench(|| {
        let bytes = pool.slots.bytes();
        let mut acc: u64 = 0;
        for i in 0..FRAMES {
            let j = i & (POOL - 1);
            let slot = &bytes[j * SLOT_CAP..(j + 1) * SLOT_CAP];
            acc = acc.wrapping_add(stage1(slot, pool.lens[j] as usize));
        }
        black_box(acc);
        assert_eq!(
            acc, expect,
            "stage-1 pass checksum drifted from preflight fold"
        );
    });
}

/// Stage 2 alone — the A14 dam over the declared frame length.
#[divan::bench(consts = [65_536, 1_048_576])]
fn stage_penstock_admission<const FRAMES: usize>(b: Bencher) {
    let pool = build_pool(CHAIN_SEED, POOL);
    let laps = (FRAMES / POOL) as u64;
    assert_eq!(FRAMES % POOL, 0, "FRAMES must be a pool multiple");
    let expect = pool.dam_fold.wrapping_mul(laps);
    b.counter(ItemsCount::new(FRAMES as u64)).bench(|| {
        let bytes = pool.slots.bytes();
        let mut acc: u64 = 0;
        for i in 0..FRAMES {
            let j = i & (POOL - 1);
            let slot = &bytes[j * SLOT_CAP..(j + 1) * SLOT_CAP];
            let mut view = dam::FrameView {
                payload: &[],
                prefilter: 0,
                escalate: false,
            };
            let st = stage2(slot, pool.lens[j] as usize, Some(&mut view));
            black_box(view.payload.as_ptr());
            acc = acc.wrapping_add(u64::from(st));
        }
        black_box(acc);
        assert_eq!(
            acc, expect,
            "stage-2 pass checksum drifted from preflight fold"
        );
    });
}

/// Stage 3 alone — BLAKE3 digest fold over the L1 ESCALATED subset
/// (chain-admitted AND escalate — exactly the frames the chain pays the
/// full digest for). On the frozen corpus this subset may be EMPTY (the
/// flag-only escalation policy admits no reserved-flag frame into the
/// chain: stage 1 refuses reserved header flags) — the bench then proves
/// the chain pays zero digest work, which is the lever's own claim; the
/// empty-subset receipt row is recorded, not rounded up.
#[divan::bench(consts = [65_536, 1_048_576])]
fn stage_digest_fold<const FRAMES: usize>(b: Bencher) {
    let pool = build_pool(CHAIN_SEED, POOL);
    let laps = (FRAMES / POOL) as u64;
    assert_eq!(FRAMES % POOL, 0, "FRAMES must be a pool multiple");
    let expect = pool.digest_fold.wrapping_mul(laps);
    b.counter(ItemsCount::new(FRAMES as u64)).bench(|| {
        let bytes = pool.slots.bytes();
        let mut acc: u64 = 0;
        for _ in 0..laps {
            for &(off, plen) in &pool.escalated {
                let d = cgap_blake3::hash(&bytes[off as usize..off as usize + plen as usize]);
                acc = acc.wrapping_add(u64::from_le_bytes(d[..8].try_into().expect("digest u64")));
            }
        }
        black_box(acc);
        assert_eq!(
            acc, expect,
            "stage-3 escalated-subset checksum drifted from preflight fold"
        );
    });
}

/// L1 new leg — the clean subset's cheap prefilter fold (no crypto). This
/// is what every chain-admitted frame pays INSTEAD of BLAKE3 under the
/// two-stage policy.
#[divan::bench(consts = [65_536, 1_048_576])]
fn stage_prefilter_fold<const FRAMES: usize>(b: Bencher) {
    let pool = build_pool(CHAIN_SEED, POOL);
    let laps = (FRAMES / POOL) as u64;
    assert_eq!(FRAMES % POOL, 0, "FRAMES must be a pool multiple");
    let expect = pool.prefilter_fold.wrapping_mul(laps);
    b.counter(ItemsCount::new(FRAMES as u64)).bench(|| {
        let bytes = pool.slots.bytes();
        let mut acc: u64 = 0;
        for _ in 0..laps {
            for &(off, _plen) in &pool.clean {
                acc = acc.wrapping_add(dam::prefilter8(&bytes[off as usize - FRAME_HEADER_LEN..]));
            }
        }
        black_box(acc);
        assert_eq!(
            acc, expect,
            "stage-prefilter pass checksum drifted from preflight fold"
        );
    });
}

// ---------------------------------------------------------------------------
// Contention guard (red-10 exclusivity law) — rows 5/8 of
// 11-measured-claims.md are QUIET-HOST-ONLY; this bench now DETECTS a noisy
// host instead of silently producing out-of-scope numbers.
//
// Two detectors, one contract:
//   1. PRE-RUN calibration (`contention_guard`): a fixed streaming walk
//      over a 32 MiB buffer, 3 passes (full-line reads — the LLC/DRAM
//      bandwidth channel red-10 W5/W6 identified as dominant), tripped by
//      absolute ns/line or pass jitter.
//   2. IN-RUN (`record_pass`/`guard_finish`): every `ingress_chain` pass's
//      ns/frame is recorded; after `divan::main`, max/min is judged. This
//      catches the load that arrives AFTER calibration (witnessed: a
//      bw8+cpu8 co-tenant left calibration quiet on the earlier
//      min-only design while the bench itself degraded +126% median).
//
// Contract: a tripped detector writes a LOUD `CONTENTION-SUSPECT:` marker
// line to STDERR. Divan output cannot be suppressed per-sample, so the
// marker line IS the contract: CI/consumers MUST grep stderr for
// `CONTENTION-SUSPECT` and refuse to admit a marked run's numbers against
// rows 5/6/7/8 (quiet-host-scoped claims). Anti-vacuity: the induced-load
// leg (bw8+cpu8 stress processes) is the flip — a guard that never fires
// is FAIL. No Process.sleep: detection is observational (Instant around
// real walks).
// ---------------------------------------------------------------------------

/// Fixed 32 MiB walk buffer (matches the bench pool footprint — the very
/// cache-resident working set a co-tenant would evict).
const GUARD_BUF_BYTES: usize = 32 * 1024 * 1024;

/// Generous quiet-host ceiling, ns per 64 B cache line streamed. Quiet
/// Mac15,9 measures ~1-2 ns/line on this walk (receipted below); 12.0 is
/// ~6x headroom, far above any quiet-host variance, yet an order of
/// magnitude below red-10's bw-only fastest degradation (+14.8% on the
/// bench, which corresponds to tens of ns of added line latency under
/// LLC thrash).
const CONTENTION_THRESHOLD_NS: f64 = 12.0;

/// Jitter ceiling: max/min across the 3 calibration passes. Quiet hosts
/// show pass-to-pass consistency (receipted: quiet ratio ~1.0x); a
/// descheduling co-tenant stalls whole passes, so 2.0 is generous against
/// quiet variance yet catches intermittent contention.
const CONTENTION_JITTER_RATIO: f64 = 2.0;

/// Marker prefix — the grep contract.
const CONTENTION_MARK: &str = "CONTENTION-SUSPECT:";

/// One streaming walk: sum every 8-byte word of a real 32 MiB buffer
/// (full-line reads — the bandwidth channel red-10 W5/W6 identified as
/// dominant). Returns ns per 64 B cache line for each pass.
fn guard_walk(buf: &[u8], passes: u32) -> Vec<f64> {
    let lines = buf.len() / 64;
    let mut out = Vec::new();
    for _ in 0..passes {
        let start = std::time::Instant::now();
        let mut acc: u64 = 0;
        let mut i = 0;
        while i < buf.len() {
            for k in (0..64).step_by(8) {
                let b: [u8; 8] = buf[i + k..i + k + 8].try_into().expect("8-byte window");
                acc = acc.wrapping_add(u64::from_le_bytes(b));
            }
            i += 64;
        }
        let elapsed = start.elapsed().as_secs_f64();
        black_box(acc);
        out.push(elapsed * 1e9 / lines as f64);
    }
    out
}

/// Calibration: build a real 32 MiB buffer (every page touched — no
/// zero-page sharing), walk it 3 times, and trip on EITHER signal:
///   (a) absolute: min ns/64B-line exceeds CONTENTION_THRESHOLD_NS
///       (sustained bandwidth/CPU contention), or
///   (b) jitter: max/min pass ratio exceeds CONTENTION_JITTER_RATIO
///       (intermittent descheduling — the cpu-hog channel that moved
///       red-10's slowest column +173% while leaving fastest intact).
/// Called from `main` BEFORE `divan::main`. When tripped, a LOUD
/// `CONTENTION-SUSPECT:` marker goes to stderr and the run's numbers are
/// marked CONTENTION-SUSPECT: CI/consumers MUST grep stderr for that
/// marker and refuse to admit the run against quiet-host-scoped claims
/// (11-measured-claims rows 5/6/7/8). Divan cannot suppress per-sample
/// output, so the marker line is the contract. Anti-vacuity: the
/// induced-load leg is the flip — a guard that never fires is FAIL.
fn contention_guard() {
    let mut buf = vec![0u8; GUARD_BUF_BYTES];
    for chunk in buf.chunks_mut(4096) {
        chunk[0] = 0xA5;
    }
    black_box(&buf);
    let passes = guard_walk(&buf, 3);
    let min = passes.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = passes.iter().cloned().fold(0.0f64, f64::max);
    let ratio = max / min;
    let tripped = min > CONTENTION_THRESHOLD_NS || ratio > CONTENTION_JITTER_RATIO;
    let detail = format!(
        "min {:.3} / max {:.3} ns/64B-line, jitter ratio {:.2} (ceilings: {} ns, ratio {})",
        min, max, ratio, CONTENTION_THRESHOLD_NS, CONTENTION_JITTER_RATIO
    );
    if tripped {
        eprintln!(
            "{} host-noise detected: {} — this run is CONTENTION-SUSPECT; its numbers are NOT valid for quiet-host-scoped claims (11-measured-claims rows 5/6/7/8). CI/consumers must grep stderr for this marker.",
            CONTENTION_MARK, detail
        );
    } else {
        eprintln!("CONTENTION-GUARD: quiet ({})", detail);
    }
}

/// Run-global per-LEG pass observables (the in-run jitter channel). Every
/// `ingress_chain` pass records its integer ns/frame in its const's slot;
/// `guard_finish` judges worst-leg max/min AFTER `divan::main` returns.
static PASS_MIN_NSPF: [AtomicU64; 2] = [AtomicU64::new(u64::MAX), AtomicU64::new(u64::MAX)];
static PASS_MAX_NSPF: [AtomicU64; 2] = [AtomicU64::new(0), AtomicU64::new(0)];

/// Record one bench pass: integer ns/frame, PER LEG (two slots keyed by
/// the const — cross-leg min/max mixing would inflate the quiet ratio:
/// the 65,536 leg is short-pass/scheduler-noisy).
fn record_pass(elapsed: std::time::Duration, frames: usize) {
    let idx = if frames <= POOL { 0 } else { 1 };
    let per = elapsed.as_nanos() as u64 / frames.max(1) as u64;
    PASS_MIN_NSPF[idx].fetch_min(per, Ordering::Relaxed);
    PASS_MAX_NSPF[idx].fetch_max(per, Ordering::Relaxed);
}

/// Post-run evaluation of the in-run jitter observable. A pass-to-pass
/// max/min ratio above CONTENTION_JITTER_RATIO means the host degraded
/// DURING measurement — the exact failure red-10 exploited. Marker law as
/// in `contention_guard`.
fn guard_finish() {
    let mut worst = 0.0f64;
    let mut ran = false;
    for (idx, frames) in [(0usize, POOL), (1usize, 65_536 * 16)] {
        let lo = PASS_MIN_NSPF[idx].load(Ordering::Relaxed);
        let hi = PASS_MAX_NSPF[idx].load(Ordering::Relaxed);
        if lo == u64::MAX {
            continue; // leg filtered out — nothing recorded
        }
        ran = true;
        let ratio = hi as f64 / lo as f64;
        if ratio > worst {
            worst = ratio;
        }
        eprintln!(
            "CONTENTION-GUARD: leg {} ({} frames) pass ratio {:.2} (ceiling {})",
            idx, frames, ratio, CONTENTION_JITTER_RATIO
        );
    }
    if !ran {
        return; // no chain leg ran (filtered out) — nothing to judge
    }
    if worst > CONTENTION_JITTER_RATIO {
        eprintln!(
            "{} in-run host-noise: worst leg pass ns/frame max/min ratio {:.2} exceeds ceiling {} — this run is CONTENTION-SUSPECT; its numbers are NOT valid for quiet-host-scoped claims (11-measured-claims rows 5/6/7/8). CI/consumers must grep stderr for this marker.",
            CONTENTION_MARK, worst, CONTENTION_JITTER_RATIO
        );
    } else {
        eprintln!(
            "CONTENTION-GUARD: in-run quiet (worst leg ratio {:.2} <= {})",
            worst, CONTENTION_JITTER_RATIO
        );
    }
}

fn main() {
    // ZP-18-C: release-profile fence — dev-profile runs loud-SKIP.
    if let Err(v) = cgap_aoq::profile_guard::assert_release() {
        eprintln!("LOUD-SKIP {} : {v}", std::env::args().next().unwrap_or_default());
        return;
    }
    contention_guard();
    divan::main();
    guard_finish();
}
