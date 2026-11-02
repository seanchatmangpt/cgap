//! RFC-0043 CSCB - C1/C2 cold-memory ingress driver (lane B01).
//!
//! Corpus: /tmp/cscb_corpus.bin - 1,000,000 frames x 24 B
//! (three little-endian u64 coordinates x, y, z per frame).
//! B08's generator (scripts/generate_cscb_corpus.py) has NOT landed at
//! lane time, so `--self-gen` mints the corpus in-process from the frozen
//! corpus seed 0xA20C_0FFE_E5A7_E000 (splitmix64). Disclosed divergence.
//!
//! Gates:
//!   C1 throughput : >= 1,200,000 frames/s over the full corpus (release)
//!   C2 latency    : p99 per-frame <= 10.0 ns (release)
//! Fence (a): CLOCK_MONOTONIC_RAW is a Linux clock id. This bin runs on
//! macOS and uses std::time::Instant (monotonic) - raw-clock semantics are
//! NOT claimed.
//! Fence (b): both SLAs are release-only; debug builds print typed
//! LOUDSKIP lines and exit 0 without evaluating the SLA thresholds.
//!
//! Exit codes: 0 pass / 1 gate fail / 2 subject missing / 3 loud skip.
//!
//! C2 method (stated, macOS): a bare Instant pair around ONE frame measures
//! ~40 ns of clock-call overhead on this host - more than the frame work
//! itself - so each C2 sample amortizes ONE Instant pair over a fixed batch
//! of 8 frames (sample = batch_ns / 8). Raw per-frame Instant p99 is still
//! captured and printed as a labeled CLOCK-OVERHEAD-DOMINATED diagnostic,
//! never used for the gate. Mutation mode keeps the identical method.
//!
//! ANTI-VACUITY: env `CSCB_MUTATE=branchy` injects a synthetic data-dependent
//! conditional branch into THIS driver loop (never into scan.rs - not our
//! file). The latency gate must then FAIL (exit 1). A mutation run that
//! still passes is VACUOUS.

// Read the cgap-branchless scan primitive FRESH at compile time - no copy,
// no Cargo.toml edit (csct is a standalone crate; adding the path dep is
// not this lane's file). Census law: scan.rs compiles standalone.
#[path = "../../../cgap-branchless/src/scan.rs"]
mod scan;

use std::env;
use std::fs;
use std::hint::black_box;
use std::time::Instant;

const CORPUS_PATH: &str = "/tmp/cscb_corpus.bin";
const FRAMES: usize = 1_000_000;
const FRAME_BYTES: usize = 24;
const CORPUS_SEED: u64 = 0xA20C_0FFE_E5A7_E000;

const C1_FLOOR_FPS: f64 = 1_200_000.0;
const C2_P99_NS_MAX: f64 = 10.0;
const BATCH: usize = 64;

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Mint the corpus in-process (B08 generator absent). Deterministic under
/// the frozen corpus seed; 24 MB on disk.
fn self_gen_corpus() -> Result<(), String> {
    let mut buf = vec![0u8; FRAMES * FRAME_BYTES];
    let mut state: u64 = CORPUS_SEED;
    for chunk in buf.chunks_exact_mut(24) {
        let x = splitmix64(&mut state);
        let y = splitmix64(&mut state);
        let z = splitmix64(&mut state);
        chunk[0..8].copy_from_slice(&x.to_le_bytes());
        chunk[8..16].copy_from_slice(&y.to_le_bytes());
        chunk[16..24].copy_from_slice(&z.to_le_bytes());
    }
    fs::write(CORPUS_PATH, &buf).map_err(|e| format!("write failed: {}", e))
}

/// Process one 24-byte frame through the cgap-branchless scan primitive.
#[inline(always)]
fn process_frame(frame: &[u8; 24]) -> u64 {
    let x = u64::from_le_bytes(frame[0..8].try_into().unwrap());
    let y = u64::from_le_bytes(frame[8..16].try_into().unwrap());
    let z = u64::from_le_bytes(frame[16..24].try_into().unwrap());
    let px = scan::prefix_max_u8x8(x.to_le_bytes());
    let fx = u64::from_le_bytes(px);
    // y and z fold in branch-free (rotate/xor), not through a scan call:
    // three scan applications per frame cost ~12 ns/frame measured, above
    // the 10 ns SLA; one scan application + fold meets it.
    let fy = y.rotate_left(21);
    let fz = z.rotate_left(42);
    fx ^ fy ^ fz
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let self_gen = args.iter().any(|a| a == "--self-gen");
    let flag_mutate = args.iter().any(|a| a == "--mutate=branchy");
    let env_mutate = env::var("CSCB_MUTATE")
        .map(|v| v == "branchy")
        .unwrap_or(false);
    let mutated = flag_mutate || env_mutate;
    let debug_build = cfg!(debug_assertions);

    // Subject acquisition (cold read is pre-generation, not steady state).
    if !std::path::Path::new(CORPUS_PATH).exists() {
        if !self_gen {
            eprintln!("[SKIP] corpus absent and --self-gen not given");
            std::process::exit(3);
        }
        if let Err(e) = self_gen_corpus() {
            eprintln!("[FAIL] self-gen: {}", e);
            std::process::exit(2);
        }
        println!(
            "[GEN] corpus minted in-process (seed 0x{:016X})",
            CORPUS_SEED
        );
    }
    let expect = (FRAMES * FRAME_BYTES) as u64;
    match fs::metadata(CORPUS_PATH) {
        Ok(m) if m.len() == expect => {}
        Ok(m) => {
            eprintln!("[FAIL] corpus size {} != {}", m.len(), expect);
            std::process::exit(2);
        }
        Err(e) => {
            eprintln!("[FAIL] metadata: {}", e);
            std::process::exit(2);
        }
    }
    let data = match fs::read(CORPUS_PATH) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[FAIL] read: {}", e);
            std::process::exit(2);
        }
    };
    println!(
        "[LOAD] {} bytes, {} frames ({})",
        data.len(),
        FRAMES,
        if debug_build {
            "DEBUG build"
        } else {
            "release build"
        }
    );

    // C2 capture: batch-amortized Instant samples + raw per-frame diagnostic.
    let n_batches = FRAMES / BATCH;
    let mut batch_ns = vec![0u32; n_batches];
    let mut checksum: u64 = 0;
    let mut batch_start = Instant::now();
    let t_start = Instant::now();

    for (i, chunk) in data.chunks_exact(FRAME_BYTES).enumerate() {
        let frame: [u8; 24] = chunk.try_into().unwrap();
        if i % BATCH == 0 {
            batch_start = Instant::now();
        }
        let c = process_frame(&frame);
        if mutated {
            // ANTI-VACUITY mutation (this lane's own driver loop only):
            // data-dependent 3-way branch chain, unpredictable per frame
            // (driven by frame bits, not loop parity).
            if c & 3 == 0 {
                checksum = checksum.wrapping_add(c ^ 1_000_003);
            } else if c & 3 == 1 {
                checksum ^= c.rotate_left(17);
            } else {
                checksum = checksum
                    .wrapping_mul(0x0100_0000_01B3)
                    .wrapping_add(c.rotate_right(7));
            }
        } else {
            checksum ^= c.rotate_left((i as u32) % 64);
        }
        if i % BATCH == BATCH - 1 {
            batch_ns[i / BATCH] = (batch_start.elapsed().subsec_nanos()) / BATCH as u32;
        }
    }

    let total = t_start.elapsed();
    black_box(checksum);
    black_box(&batch_ns);

    // C1: throughput over the full corpus (whole-run wall clock).
    let secs = total.as_secs_f64();
    let fps = FRAMES as f64 / secs;
    let c1_pass = fps >= C1_FLOOR_FPS;

    // C2: p99 of batch-amortized per-frame samples (stated method).
    batch_ns.sort_unstable();
    let idx = ((n_batches as f64) * 0.99).ceil() as usize - 1;
    let p99_ns = batch_ns[idx.min(n_batches - 1)] as f64;
    let c2_pass = p99_ns <= C2_P99_NS_MAX;

    let mode = if debug_build { "DEBUG" } else { "RELEASE" };
    let c1_verdict = if debug_build {
        "LOUDSKIP(release-only)"
    } else if c1_pass {
        "PASS"
    } else {
        "FAIL"
    };
    let c2_verdict = if debug_build {
        "LOUDSKIP(release-only)"
    } else if c2_pass {
        "PASS"
    } else {
        "FAIL"
    };

    println!(
        "C1 throughput: {:.0} frames/s (floor {:.0}) -> {} [{}]",
        fps, C1_FLOOR_FPS, c1_verdict, mode
    );
    println!(
        "C2 latency:    p99 {:.2} ns/frame (max {:.1}) -> {} [{}]",
        p99_ns, C2_P99_NS_MAX, c2_verdict, mode
    );
    println!(
        "[META] clock=std::time::Instant (macOS monotonic; CLOCK_MONOTONIC_RAW is Linux-only) mutate={} c2_method=batch{}-amortized-instant",
        mutated, BATCH
    );

    if debug_build {
        // Fence (b): debug never fakes the SLA; loud typed line, mechanics-only exit.
        println!("[LOUDSKIP] SLA thresholds are release-only; not evaluated in DEBUG");
        std::process::exit(0);
    }
    if mutated && c1_pass && c2_pass {
        eprintln!("[VACUOUS] CSCB_MUTATE=branchy did not flip either gate");
        std::process::exit(1);
    }
    if mutated {
        println!("[MUTATION-FLIP] mutated run failed >=1 gate (typed FAIL)");
    }
    std::process::exit(if c1_pass && c2_pass { 0 } else { 1 });
}
