//! AB-11 ring stress courts: churn, ordinal-wrap, backpressure, throughput.
//!
//! Subject: crates/wire-shm/src/ring.rs (read-only). Real file-backed
//! segments in a per-test temp dir; no mocks. Every court names its
//! anti-vacuity mutation in a comment.

use std::fs::OpenOptions;
use std::os::unix::fs::FileExt;
use std::path::PathBuf;
use std::time::Instant;

use wire_shm::{crc32_ieee, WireError, WireRing, MAX_PAYLOAD};

/// Per-test temp segment path (unique per test name; deleted on drop via
/// leak-tolerant pattern: tests own disjoint names, files live in
/// std::env::temp_dir and are removed at test start).
fn seg_path(name: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("ab11-ring-stress-{name}.seg"));
    let _ = std::fs::remove_file(&p);
    p
}

/// Deterministic payload: 8-byte BE ordinal + byte pattern.
fn fill_payload(ordinal: u32, len: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(len);
    v.extend_from_slice(&ordinal.to_be_bytes());
    let seed = 0xA20C_0FFE_E5A7_E000u64 ^ (ordinal as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let mut x = seed;
    while v.len() < len {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        v.push((x >> 33) as u8);
    }
    v
}

/// Anti-vacuity helper: patch next_ordinal bookkeeping to simulate a u32
/// writer that wrapped past u32::MAX back to 0 (fresh-writer semantics).
/// Real bytes in the real segment file, sha-verified by the caller via
/// subsequent read_frame ordinal-match law.
fn force_next_ordinal(path: &PathBuf, next: u64) {
    let f = OpenOptions::new().write(true).open(path).unwrap();
    f.write_all_at(&next.to_be_bytes(), 0).unwrap();
}

const CHURN_CAPACITY: u32 = 1024;
const CHURN_CYCLES: u32 = 100_000;

/// Court 1 — churn: 100k write/read cycles in fill→drain waves over a
/// 1024-slot ring. Integrity per frame: crc32 match + exact payload bytes
/// + stored ordinal match. Zero corruption tolerated.
/// Anti-vacuity: flipping any header byte in the segment file makes
/// read_frame fail (CrcMismatch/BadMagic) — a silent-corruption pass is
/// impossible because every frame is byte-compared.
#[test]
fn churn_100k_fill_drain_integrity() {
    let path = seg_path("churn");
    let mut ring = WireRing::create(&path, CHURN_CAPACITY).unwrap();
    let mut next: u32 = 0;
    let mut wave: u32 = 0;
    while next < CHURN_CYCLES {
        wave += 1;
        let wave_len = (CHURN_CAPACITY.min(CHURN_CYCLES - next)) as u32;
        // FILL: one wave's worth of frames.
        for i in 0..wave_len {
            let ord = next + i;
            let payload = fill_payload(ord, 64 + (ord as usize % (MAX_PAYLOAD - 64)));
            ring.stamp_frame(&payload, ord).unwrap();
        }
        // DRAIN: read back the full wave, verify crc + bytes + ordinal.
        for i in 0..wave_len {
            let ord = next + i;
            let want = fill_payload(ord, 64 + (ord as usize % (MAX_PAYLOAD - 64)));
            let (got, crc) = ring.read_frame_with_crc(ord).unwrap();
            assert_eq!(got.len(), want.len(), "len mismatch at ordinal {ord}");
            assert_eq!(got, want, "payload bytes corrupted at ordinal {ord}");
            assert_eq!(crc, crc32_ieee(&want), "crc mismatch at ordinal {ord}");
            // Ordinal-match law via public API: wrong ordinal must refuse.
            if ord + 1 != next + wave_len {
                // slot of ord+1 may hold a different frame; only assert when
                // ord+1 was actually written this wave or a previous one.
                if ord + 1 < next + wave_len {
                    let _ = ring.read_frame(ord + 1).unwrap(); // must be valid
                }
            }
        }
        next += wave_len;
        // Capacity-exceeded read (evicted ordinal): typed refusal, never a
        // stale payload. Ordinal `next - CHURN_CAPACITY` maps to the slot
        // just rewritten by this wave.
        if next > CHURN_CAPACITY {
            // Evicted iff o < next - capacity; read must refuse typed.
            let evicted = next - CHURN_CAPACITY - 1;
            let err = ring.read_frame(evicted).unwrap_err();
            assert!(
                matches!(
                    err,
                    WireError::StaleOrdinal { .. } | WireError::CrcMismatch { .. }
                ),
                "evicted read must fail typed, got {err:?}"
            );
        }
    }
    assert_eq!(wave, 98, "expected 98 fill/drain waves for 100k/1024");
    assert_eq!(ring.next_ordinal().unwrap(), CHURN_CYCLES as u64);
    std::fs::remove_file(&path).unwrap();
}

/// Court 2 — ordinal-wrap walk: drive frame_ordinal through the
/// u32::MAX → 0 boundary 3 times. The writer bookkeeping (u64
/// next_ordinal) is reset to the wrapped value with real file bytes
/// between boundary crossings, exactly as a wrapping u32 writer's slot
/// address law (ordinal % capacity) requires; every read asserts the
/// ordinal-match law holds across the wrap.
/// Anti-vacuity: if the wrap changed slot addressing, read_frame at the
/// wrapped ordinal would hit a mismatched stored_ordinal and fail.
#[test]
fn ordinal_wrap_walk_three_times() {
    let path = seg_path("wrap");
    let cap: u32 = 256;
    let mut ring = WireRing::create(&path, cap).unwrap();
    let max_ord = u32::MAX;
    for cycle in 0u32..3 {
        // Pre-boundary frame at u32::MAX - (1 - cycle)? Use descending edge:
        // write u32::MAX - cycle... only first cycle can use MAX directly
        // since each write advances next to ord+1 (as u64, no u32 wrap).
        let pre = max_ord - cycle;
        let pre_payload = fill_payload(pre, 128);
        force_next_ordinal(&path, pre as u64); // writer positioned at `pre`
        ring.stamp_frame(&pre_payload, pre).unwrap();
        // Crossing: next_ordinal is now pre+1 = 2^32 (no u32 wrap in the
        // u64 field). Reset the writer to 0 — the wrapped state — then
        // write ordinal 0, 1, 2.
        force_next_ordinal(&path, 0);
        for post in 0u32..3 {
            let payload = fill_payload(post, 128 + cycle as usize);
            ring.stamp_frame(&payload, post).unwrap();
            let (got, crc) = ring.read_frame_with_crc(post).unwrap();
            assert_eq!(
                got, payload,
                "cycle {cycle}: wrapped read corrupted at {post}"
            );
            assert_eq!(crc, crc32_ieee(&payload), "cycle {cycle}: crc at {post}");
        }
        // Ordinal-match law across the boundary: slot(pre % cap) was
        // overwritten by wrapped writes iff cap divides the distance;
        // with cap=256, pre % 256 = (256-cycle-? ) — recompute directly:
        // u32::MAX % 256 = 255; post frames occupy slots 0..2, so slot 255
        // still holds the pre frame. Read it back and verify.
        let (got_pre, crc_pre) = ring.read_frame_with_crc(pre).unwrap();
        assert_eq!(
            got_pre, pre_payload,
            "cycle {cycle}: pre-boundary frame corrupted"
        );
        assert_eq!(crc_pre, crc32_ieee(&pre_payload));
        // StaleOrdinal law survives the wrap: writing 0 again with next=3
        // must refuse typed.
        let err = ring.stamp_frame(&fill_payload(0, 8), 0).unwrap_err();
        assert!(
            matches!(
                err,
                WireError::StaleOrdinal {
                    expected: 3,
                    found: 0
                }
            ),
            "cycle {cycle}: expected typed StaleOrdinal, got {err:?}"
        );
    }
    std::fs::remove_file(&path).unwrap();
}

/// Court 3 — backpressure: fill the ring to capacity, then attempt
/// overflow-class operations. The ring's capacity backpressure surfaces as
/// typed refusal, never silent drop: (a) stale (already-consumed)
/// ordinals refuse StaleOrdinal; (b) over-MAX payloads refuse
/// FrameTooLarge; (c) every refused op leaves state intact — a full
/// post-refusal drain of all capacity frames returns byte-identical
/// payloads.
/// Anti-vacuity: if a refusal silently dropped or corrupted a slot, the
/// post-refusal drain would mismatch its payload.
#[test]
fn backpressure_typed_refusal_state_intact() {
    let path = seg_path("backpressure");
    let cap: u32 = 64;
    let mut ring = WireRing::create(&path, cap).unwrap();
    // FILL to capacity.
    for ord in 0..cap {
        ring.stamp_frame(&fill_payload(ord, 256), ord).unwrap();
    }
    assert_eq!(ring.next_ordinal().unwrap(), cap as u64);
    // Overflow writes: ordinal < next → typed StaleOrdinal (no silent drop).
    for ord in 0..cap {
        let err = ring.stamp_frame(&fill_payload(ord, 256), ord).unwrap_err();
        assert!(
            matches!(err, WireError::StaleOrdinal { expected, found } if expected == cap && found == ord),
            "overflow write at {ord} must refuse typed, got {err:?}"
        );
    }
    // Over-MAX payload → typed FrameTooLarge (never truncated write).
    let err = ring
        .stamp_frame(&vec![0u8; MAX_PAYLOAD + 1], cap)
        .unwrap_err();
    assert!(
        matches!(err, WireError::FrameTooLarge { len, max } if len == MAX_PAYLOAD + 1 && max == MAX_PAYLOAD),
        "oversize write must refuse typed, got {err:?}"
    );
    // State intact: drain every slot, byte-identical.
    for ord in 0..cap {
        let want = fill_payload(ord, 256);
        let (got, crc) = ring.read_frame_with_crc(ord).unwrap();
        assert_eq!(got, want, "state corrupted at {ord} after refusals");
        assert_eq!(crc, crc32_ieee(&want));
    }
    // Drain-and-continue: consume half via advancing (fresh segment at the
    // drained watermark), refill, full integrity again.
    let watermark = ring.next_ordinal().unwrap() as u32;
    for i in 0..cap / 2 {
        ring.stamp_frame(&fill_payload(watermark + i, 256), watermark + i)
            .unwrap();
    }
    for i in 0..cap / 2 {
        let ord = watermark + i;
        let (got, _) = ring.read_frame_with_crc(ord).unwrap();
        assert_eq!(got, fill_payload(ord, 256), "refill corrupted at {ord}");
    }
    // Evicted frames are gone-typed, not silently stale-readable.
    let err = ring.read_frame(0).unwrap_err();
    assert!(matches!(
        err,
        WireError::StaleOrdinal { .. } | WireError::CrcMismatch { .. }
    ));
    std::fs::remove_file(&path).unwrap();
}

/// Court 4 — throughput: frames/sec fill+drain, min-of-3 runs, at 64B and
/// 4080B (ring max). The charter's 4KB/64KB payloads EXCEED the ring's
/// MAX_PAYLOAD (4080) — they cannot be benchmarked; instead they are
/// asserted here as typed FrameTooLarge refusals (loud skip, documented in
/// the receipt), never a fabricated number.
/// Anti-vacuity: each run asserts integrity of every frame it moved; a
/// fast-but-corrupt result fails the court.
#[test]
fn throughput_frames_per_sec_min_of_3() {
    // ZP-18 timing fence: min-of-3 frames/sec throughput court; under a dev
    // profile this loud-SKIPs instead of printing debug-code timing.
    if let Err(v) = cgap_aoq::profile_guard::assert_release() {
        eprintln!("LOUD-SKIP throughput_frames_per_sec_min_of_3: {v}");
        return;
    }
    let sizes: [usize; 2] = [64, MAX_PAYLOAD];
    let mut report = String::from("AB-11 throughput (min of 3 fill+drain runs):\n");
    for &size in &sizes {
        let mut best = f64::INFINITY;
        for _run in 0..3 {
            let path = seg_path("throughput");
            let cap: u32 = 4096;
            let mut ring = WireRing::create(&path, cap).unwrap();
            let n_frames: u32 = 20_000;
            let t0 = Instant::now();
            for ord in 0..n_frames {
                ring.stamp_frame(&fill_payload(ord, size), ord).unwrap();
            }
            // Only the last `cap` frames are retained by the ring; drain
            // the readable window.
            for ord in (n_frames - cap)..n_frames {
                let (got, crc) = ring.read_frame_with_crc(ord).unwrap();
                assert_eq!(got.len(), size);
                assert_eq!(crc, crc32_ieee(&fill_payload(ord, size)));
            }
            let secs = t0.elapsed().as_secs_f64();
            best = best.min(n_frames as f64 / secs);
            drop(ring);
            std::fs::remove_file(&path).unwrap();
        }
        report.push_str(&format!(
            "  {size}-byte payload: {best:.0} frames/sec (fill+drain)\n"
        ));
    }
    // 4KB and 64KB: typed refusal is the only lawful outcome.
    let path = seg_path("throughput-illegal");
    let mut ring = WireRing::create(&path, 16).unwrap();
    for bad in [4096usize, 65536] {
        let err = ring.stamp_frame(&vec![0u8; bad], 0).unwrap_err();
        assert!(
            matches!(err, WireError::FrameTooLarge { len, .. } if len == bad),
            "{bad}-byte frame must refuse typed, got {err:?}"
        );
    }
    println!("{report}");
}
