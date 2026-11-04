//! Chicago courts for wire-shm: real file segments, final-state asserts.
//!
//! ANTI-VACUITY: tamper court mutates a /tmp sandbox segment (never the repo
//! tree) and must observe a typed error; the unmutated control must pass.

use std::path::PathBuf;
use wire_shm::{WireError, WireRing};

fn tmp_segment(tag: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("wire-shm-court-{}-{}.seg", tag, std::process::id()));
    let _ = std::fs::remove_file(&p);
    p
}

fn cleanup(p: &PathBuf) {
    let _ = std::fs::remove_file(p);
}

#[test]
fn crc32_known_vectors() {
    assert_eq!(wire_shm::crc32_ieee(b"123456789"), 0xCBF4_3926);
    assert_eq!(wire_shm::crc32_ieee(b""), 0);
}

#[test]
fn round_trip_several_frames() {
    let path = tmp_segment("rt");
    let mut ring = WireRing::create(&path, 8).expect("create");
    let payloads: Vec<Vec<u8>> = (0u32..5)
        .map(|i| format!("payload-{i}-{}", b'x').repeat(1).into_bytes())
        .collect();
    for (i, p) in payloads.iter().enumerate() {
        ring.stamp_frame(p, i as u32).expect("stamp");
    }
    assert_eq!(ring.next_ordinal().expect("next"), 5);
    for (i, p) in payloads.iter().enumerate() {
        let got = ring.read_frame(i as u32).expect("read");
        assert_eq!(&got, p, "byte equality at ordinal {i}");
    }
    cleanup(&path);
}

#[test]
fn wraparound_reclaims_oldest_stale_read_typed() {
    let path = tmp_segment("wrap");
    let cap = 4u32;
    let mut ring = WireRing::create(&path, cap).expect("create");
    for i in 0u32..(cap + 5) {
        let payload = format!("frame-{i:04}").into_bytes();
        ring.stamp_frame(&payload, i).expect("stamp");
    }
    assert_eq!(ring.next_ordinal().expect("next"), (cap + 5) as u64);
    // Stamped ordinals 0..=8 (cap+5 frames); slot 0 (idx 0) now holds
    // ordinal 8, so ordinal 0 is reclaimed -> typed StaleOrdinal.
    let err = ring.read_frame(0).expect_err("stale ordinal must error");
    match err {
        WireError::StaleOrdinal { expected, found } => {
            assert_eq!(expected, 0);
            assert_eq!(found, 8); // slot 0 now carries ordinal 8
        }
        other => panic!("expected StaleOrdinal, got {other:?}"),
    }
    // Mid ring intact (ordinal 4 at slot 4 % 4 == 0? no: 4 % 4 == 0 too).
    // capacity 4: ordinal 4 reuses slot 0 as well, so it is also reclaimed;
    // ordinals >= 4 are... only 8 occupies slot 0. Ordinals 5,6,7 intact.
    for i in 5u32..9 {
        let got = ring.read_frame(i).expect("read live frame");
        assert_eq!(got, format!("frame-{i:04}").into_bytes());
    }
    cleanup(&path);
}

#[test]
fn frame_too_large_typed() {
    let path = tmp_segment("big");
    let mut ring = WireRing::create(&path, 2).expect("create");
    let big = vec![0u8; 5000];
    let err = ring.stamp_frame(&big, 0).expect_err("oversize must error");
    assert_eq!(
        err,
        WireError::FrameTooLarge {
            len: 5000,
            max: 4080
        }
    );
    cleanup(&path);
}

#[test]
fn persistence_across_reopen() {
    let path = tmp_segment("persist");
    let payload = b"survives-reopen".to_vec();
    {
        let mut ring = WireRing::create(&path, 4).expect("create");
        ring.stamp_frame(&payload, 42).expect("stamp ord 42");
    } // drop -> file closed
    let ring2 = WireRing::open(&path).expect("reopen");
    assert_eq!(ring2.read_frame(42).expect("read after reopen"), payload);
    assert_eq!(ring2.next_ordinal().expect("next"), 43);
    cleanup(&path);
}

// --- Tamper court + anti-vacuity ------------------------------------------

fn read_seg(p: &PathBuf) -> Vec<u8> {
    std::fs::read(p).expect("read segment")
}

#[test]
fn tamper_court_flips() {
    // ANTI-VACUITY MUTATION: corrupt the ordinal byte of a frame inside a
    // /tmp sandbox segment (never a repo file -> no sha256 restore path).
    // Mutated copy must FAIL (typed error); unmutated control must PASS.
    // A court that cannot flip is FAIL.
    let control_path = tmp_segment("tamper-control");
    let mut control = WireRing::create(&control_path, 4).expect("create control");
    let payload = b"tamper-court-payload".to_vec();
    control.stamp_frame(&payload, 7).expect("stamp control");
    let control_result = control.read_frame(7);
    assert!(
        control_result.is_ok(),
        "CONTROL (unmutated) must pass: {:?}",
        control_result.err()
    );
    assert_eq!(control_result.expect("control payload"), payload);
    cleanup(&control_path);

    // Sandbox copy: independent segment, mutated in place.
    let sandbox_path = tmp_segment("tamper-sandbox");
    {
        let mut ring = WireRing::create(&sandbox_path, 4).expect("create sandbox");
        ring.stamp_frame(&payload, 7).expect("stamp sandbox");
    }
    let raw = read_seg(&sandbox_path);
    let slot3 = &raw[4096 + 3 * 4096..];
    assert_eq!(
        &slot3[..4],
        &0x5749_5230u32.to_be_bytes(),
        "magic at slot 3"
    );

    // Locate the ordinal bytes: slot idx = 7 % 4 = 3 ->
    // BOOK_LEN + 3*SLOT_LEN + 12 = 4096 + 12288 + 12.
    let ord_off = 4096 + 3 * 4096 + 12;
    let before = raw[ord_off];
    assert_eq!(before, 0, "ordinal 7 big-endian low byte at offset 4108");
    let mut mutated = raw.clone();
    mutated[ord_off] = 9; // corrupt ordinal -> 9 != 7
    std::fs::write(&sandbox_path, &mutated).expect("write mutated segment");

    let ring = WireRing::open(&sandbox_path).expect("open mutated segment");
    let err = ring.read_frame(7).expect_err("MUTATED copy must fail");
    assert!(
        matches!(err, WireError::StaleOrdinal { .. }),
        "court must observe typed StaleOrdinal on mutation, got {err}"
    );
    cleanup(&sandbox_path);
}

#[test]
fn crc_tamper_typed() {
    // Secondary flip: corrupt payload byte -> typed CrcMismatch.
    let path = tmp_segment("crc-tamper");
    {
        let mut ring = WireRing::create(&path, 2).expect("create");
        ring.stamp_frame(b"crc court", 0).expect("stamp");
    }
    let mut raw = read_seg(&path);
    let payload_off = 4096 + 16;
    raw[payload_off] ^= 0xFF;
    std::fs::write(&path, &raw).expect("write");
    let ring = WireRing::open(&path).expect("open");
    let err = ring.read_frame(0).expect_err("crc flip must fail");
    assert!(matches!(err, WireError::CrcMismatch { .. }), "got {err}");
    cleanup(&path);
}

#[test]
fn corrupt_plen_budget_refusal_typed() {
    // Budget-refusal court (S12 zero-alloc law, oabi `_into` pattern):
    // `plen` is wire input. A corrupt plen (still <= MAX_PAYLOAD, so it
    // passes the FrameTooLarge gate) must refuse as typed BudgetTooSmall
    // against an undersized caller buffer — never panic, never over-read
    // into the caller's memory. ANTI-VACUITY: control (matching budget)
    // must PASS, mutated plen against the same budget must FAIL typed.
    let path = tmp_segment("plen-budget");
    {
        let mut ring = WireRing::create(&path, 2).expect("create");
        ring.stamp_frame(b"plen budget court", 0).expect("stamp");
    }
    let raw = read_seg(&path);
    let plen_off = 4096 + 8; // slot 0 header, payload_len bytes 8..12
    assert_eq!(&raw[plen_off..plen_off + 4], &17u32.to_be_bytes());
    let ring = WireRing::open(&path).expect("open");

    // CONTROL: budget >= 16 + 17 -> read_frame_into succeeds.
    let mut ctrl = [0u8; 64];
    let n = ring
        .read_frame_into(0, &mut ctrl)
        .expect("control must pass with sufficient budget");
    assert_eq!(n, 17);
    assert_eq!(&ctrl[..n], b"plen budget court");

    // MUTATION 1 (corrupt wire plen 17 -> 2000): same 64-byte budget now
    // refuses typed BudgetTooSmall, payload bytes untouched.
    let mut mutated = raw.clone();
    mutated[plen_off..plen_off + 4].copy_from_slice(&2000u32.to_be_bytes());
    std::fs::write(&path, &mutated).expect("write mutated");
    let ring = WireRing::open(&path).expect("open mutated");
    let err = ring
        .read_frame_into(0, &mut ctrl)
        .expect_err("mutated plen must refuse");
    assert!(
        matches!(err, WireError::BudgetTooSmall { need: 2016, have: 64 }),
        "expected typed BudgetTooSmall {{need:2016, have:64}}, got {err}"
    );

    // MUTATION 2 (plen above MAX_PAYLOAD): typed FrameTooLarge, regardless
    // of budget.
    let mut huge = raw.clone();
    huge[plen_off..plen_off + 4].copy_from_slice(&4081u32.to_be_bytes());
    std::fs::write(&path, &huge).expect("write huge");
    let ring = WireRing::open(&path).expect("open huge");
    let err = ring
        .read_frame_into(0, &mut ctrl)
        .expect_err("over-max plen must refuse");
    assert!(
        matches!(err, WireError::FrameTooLarge { len: 4081, .. }),
        "got {err}"
    );
    cleanup(&path);
}

// ===========================================================================
// CG-1 additions (v26.11.3): G3 witness-tier anti-confusion courts.
// ===========================================================================

/// Tier witness (G3, PG-6 decision: NO shared digest seam): wire-shm's
/// crc32_ieee is the TRANSPORT/accidental tier (see crates/cgap-arrow/src/
/// ipc.rs module doc "WITNESS TIERS"). COURT: the wire-shm sources contain
/// zero occurrences of "blake3" - the tamper tier's digest must not leak
/// into the transport tier. ANTI-VACUITY (sandbox-add-token): the same scan
/// over a /tmp copy of a wire-shm source with the token injected MUST
/// detect it, else the scan is vacuous. RESTORE LAW: repo sources are only
/// read; the fail-closed restore witness compares before/after bytes.
#[test]
fn wire_tier_scan_zero_blake3_with_sandbox_kill() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let src_dir = std::path::Path::new(manifest).join("src");
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    for entry in std::fs::read_dir(&src_dir).expect("read src dir") {
        let p = entry.expect("dir entry").path();
        if p.extension().map_or(false, |e| e == "rs") {
            files.push(p);
        }
    }
    assert!(!files.is_empty(), "wire-shm src has .rs files");
    let mut befores: Vec<(std::path::PathBuf, Vec<u8>)> = Vec::new();
    for f in &files {
        befores.push((f.clone(), std::fs::read(f).expect("read source")));
    }
    // Zero-token law over every wire-shm source file.
    for (f, bytes) in &befores {
        let text = String::from_utf8_lossy(bytes);
        let hits = text.lines().filter(|l| l.contains("blake3")).count();
        assert_eq!(
            hits, 0,
            "{:?} must contain zero \"blake3\" (transport tier)",
            f
        );
    }
    // ANTI-VACUITY: inject the token into a /tmp sandbox copy; the same
    // scan MUST flip.
    let sandbox = std::env::temp_dir().join(format!("cg1-ws-scan-{}.rs", std::process::id()));
    let first = &befores[0].1;
    let mutated = format!("// blake3 injected\n{}", String::from_utf8_lossy(first));
    std::fs::write(&sandbox, mutated).expect("write sandbox");
    let stext = std::fs::read_to_string(&sandbox).expect("read sandbox");
    let shits = stext.lines().filter(|l| l.contains("blake3")).count();
    let _ = std::fs::remove_file(&sandbox);
    assert!(shits >= 1, "VACUOUS SCAN: injected token undetected");

    // Fail-closed restore witness: repo sources byte-exact unchanged.
    for (f, before) in &befores {
        let after = std::fs::read(f).expect("re-read source");
        assert_eq!(before, &after, "{:?} untouched (fail-closed restore)", f);
    }
}
