//! C4 — steady-state zero-allocation court (lane B03, v26.11.2).
//!
//! Arms a [`cgap_core` tracking allocator] budget (0 allocs / 0 bytes)
//! around the CSCB ingress hot loop (full 1,000,000-frame pass over the
//! corpus + a small fold step) and asserts steady-state allocations ==
//! 0 bytes AND == 0 count.
//!
//! ANTI-VACUITY: `CSCB_MUTATE=alloc` injects one `Box::new(42u64)` into
//! the loop. The budget breaches; the interceptor records the breach in
//! `last_breach()`, DISARMS, and serves the allocation (measured
//! breach path: panicking inside a GlobalAlloc aborts the process, so
//! the typed refusal is the recorded-breach record, not a panic —
//! cgap-core/src/tracking_alloc.rs BREACH PATH note). The court then
//! observes nonzero bytes/count and exits 1.
//!
//! Registration divergence (recorded): cgap-core does not export
//! `tracking_alloc` from lib.rs and lib.rs is off-limits to this lane,
//! so the module is `#[path]`-included from its canonical source —
//! single implementation, zero drift.

#[path = "../../../cgap-core/src/tracking_alloc.rs"]
mod tracking_alloc;

use std::alloc::System;
use std::hint::black_box;

use tracking_alloc::{last_breach, start_hot_path_budget, window_counts, EnclaveTrackingAlloc};

/// Frozen canonical splitmix64 seed (sandbox law — never vary).
const SEED: u64 = 0xBF58_476D_1CE4_E5B9;
/// Corpus constant (sandbox law — never vary).
const CORPUS_CONST: u64 = 0xA20C_0FFE_E5A7_E000;
/// Frames in the ingress pass.
const FRAMES: usize = 1_000_000;
/// Frame width in bytes.
const FRAME_W: usize = 64;
/// Corpus frames resident (window scanned cyclically to FRAMES).
const CORPUS_FRAMES: usize = 1_024;

#[global_allocator]
static GA: EnclaveTrackingAlloc<System> = EnclaveTrackingAlloc { inner: System };

/// Deterministic splitmix64 next (cgap-parquet canonical form).
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Load the corpus fixture if present; otherwise self-generate
/// deterministically (B08 fixture lane in flight — disclosed fallback,
/// same rule as sibling CSCB lanes). Returns (frames, provenance).
fn load_corpus() -> (Vec<Vec<u8>>, &'static str) {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/corpus.bin");
    if let Ok(bytes) = std::fs::read(&fixture) {
        let frames: Vec<Vec<u8>> = bytes
            .chunks(FRAME_W)
            .filter(|c| c.len() == FRAME_W)
            .map(|c| c.to_vec())
            .collect();
        if !frames.is_empty() {
            return (frames, "fixture");
        }
    }
    let mut state = SEED ^ CORPUS_CONST;
    let frames: Vec<Vec<u8>> = (0..CORPUS_FRAMES)
        .map(|i| {
            let mut f = Vec::with_capacity(FRAME_W);
            for _ in 0..FRAME_W {
                f.push(splitmix64(&mut state) as u8);
            }
            black_box(i);
            f
        })
        .collect();
    (frames, "self-gen")
}

fn main() {
    // All env/path/fs work happens BEFORE the budget window is armed:
    // the window must see only the hot loop's allocations.
    let mutate = std::env::var("CSCB_MUTATE").ok().as_deref() == Some("alloc");
    let (corpus, provenance) = load_corpus();
    let warm = black_box(corpus.len());
    println!(
        "cscb_c4: corpus={} frames={} frame_w={} mutate={}",
        provenance, warm, FRAME_W, mutate
    );

    let mut acc: u64 = 0;
    // Small closure step: fold one frame into the accumulator.
    let step = |frame: &[u8], acc: &mut u64| {
        let mut h = *acc;
        for b in frame {
            h = h.rotate_left(7) ^ (*b as u64).wrapping_mul(0xA20C_0FFE_E5A7_E001);
        }
        *acc = h;
    };

    let _guard = start_hot_path_budget("cscb_c4_ingress", 0, 0);
    for i in 0..FRAMES {
        let frame: &[u8] = &corpus[i % corpus.len()];
        step(frame, &mut acc);
        if mutate && i == 0 {
            let b = Box::new(42u64);
            black_box(&b);
        }
    }
    drop(_guard);

    let (count, bytes) = window_counts();
    black_box(acc);
    if mutate {
        // Expected: the injected allocation breached, was RECORDED (the
        // typed refusal of this interceptor), and the court fails.
        match last_breach() {
            Some(br) if br.allocs > 0 && br.bytes > 0 => {
                eprintln!(
                    "C4 MUTATE: breach observed site={:?} allocs={} bytes={} offending={} -> exit 1",
                    br.site, br.allocs, br.bytes, br.offending_bytes
                );
                std::process::exit(1);
            }
            _ => {
                eprintln!("C4 VACUOUS: mutate mode saw NO breach — court cannot flip, FAIL");
                std::process::exit(1);
            }
        }
    }
    if count == 0 && bytes == 0 && last_breach().is_none() {
        println!("C4 PASS: steady-state 0 allocs / 0 bytes over {FRAMES} frames");
        return;
    }
    eprintln!(
        "C4 FAIL: window_counts=({count},{bytes}) breach={:?}",
        last_breach()
    );
    std::process::exit(1);
}
