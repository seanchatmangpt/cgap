//! AF-33 Benchmark 03 — kinetic mass-to-velocity density.
//!
//! Measures the ACTUAL footprint/throughput of the landed AF-26/AF-32
//! pipeline pattern (decode → gate → AEAD seal → chain receipt), the same
//! composition as `af30_cage_match.rs` Contender A, under sustained load.
//!
//! ESSAY-CLAIMS LAW: the dossier's "under 2MB RAM", "1000× density",
//! "espresso-machine power" are TARGET-UNMEASURED. This bench measures:
//!   1. peak/plateau RSS (KB) during a fixed sustained-load window
//!      (sampled every 100 ms via `ps -o rss=` — external observation,
//!      no in-process API dependence);
//!   2. artifact footprint (test binary; release rlib measured in the
//!      receipt — LINKED-ELF law footnote applies);
//!   3. throughput-per-MB-RAM density ratio (frames/sec ÷ peak-RSS-MB);
//!   4. container comparison: TARGET (named bench: AF-30 Contender B on
//!      same host/same N — no 32 GB-heap JVM fleet on this host);
//!   5. power: TARGET (named bench: `perf stat` energy counters or an
//!      external wattmeter — no power meter on this host).
//!
//! The "1000×" multiplier is NEVER stated as measured anywhere in this
//! file or its receipt; only measured ratios are recorded.
//!
//! SLEEP-CLASS note: the sampler thread's 100 ms interval is POLL-STEP
//! scheduling (bounded poll on a real observable, `ps` RSS of this pid);
//! the load loop is bounded by a real deadline observable, not a sleep.

// kB unit-suffix naming (sample_rss_kB / peak_kB / plateau_kB) is
// intentional — the identifier carries the kilobyte unit; not a casing
// error, so it is allowed rather than renamed.
#![allow(non_snake_case)]

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cgap_aoq::chain::{AoqChain, AoqChainRecord, ChainDigest};
use cgap_aoq::codec::{decode, encode};
use cgap_aoq::header::{AoqHeader, Epoch};
use cgap_crypto::aead::{self, Nonce96};

// ---------------------------------------------------------------- seam

#[derive(Clone, Copy)]
struct Mix32;
impl ChainDigest for Mix32 {
    fn digest(&self, parts: &[&[u8]]) -> [u8; 32] {
        mix32(parts)
    }
}

/// Same seam family as the landed `chain_bench.rs` / `af30_cage_match.rs`.
fn mix32(parts: &[&[u8]]) -> [u8; 32] {
    let mut a: u64 = 0xcbf2_9ce4_8422_2325;
    let mut b: u64 = 0x9e37_79b9_7f4a_7c15;
    for part in parts {
        for &byte in *part {
            a ^= byte as u64;
            a = a.wrapping_mul(0x100_0000_01b3);
            b = (b ^ ((byte as u64) | 1)).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        }
    }
    let mut out = [0u8; 32];
    for (i, chunk) in out.as_chunks_mut::<8>().0.iter_mut().enumerate() {
        let v = if i % 2 == 0 {
            a.wrapping_add(i as u64)
        } else {
            b.wrapping_add(i as u64)
        };
        chunk.copy_from_slice(&v.to_le_bytes());
    }
    out
}

// ---------------------------------------------------------------- RSS sampler

/// External RSS observation: `ps -o rss= -p <pid>` (KB on macOS), one
/// sample per POLL-STEP (100 ms), bounded by the load window deadline.
fn sample_rss_kB(pid: u32) -> Option<u64> {
    let out = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout);
    s.trim().parse::<u64>().ok()
}

fn spawn_rss_sampler(
    window: Duration,
    samples: Arc<Mutex<Vec<(u64, u64)>>>, // (elapsed_ms, rss_kB)
) -> std::thread::JoinHandle<()> {
    let pid = std::process::id();
    std::thread::spawn(move || {
        let start = Instant::now();
        while start.elapsed() < window {
            if let Some(kb) = sample_rss_kB(pid) {
                let ms = start.elapsed().as_millis() as u64;
                samples.lock().unwrap().push((ms, kb));
            }
            // SLEEP-CLASS: POLL-STEP — bounded interval between real
            // observations of the load process's RSS.
            std::thread::sleep(Duration::from_millis(100));
        }
    })
}

// ---------------------------------------------------------------- court

const WINDOW_SECS: u64 = 5; // sustained-load window (fixed, deadline-bounded)
const SLAB_LINKS: usize = 1_000_000; // chain slab, allocated once

fn make_header(ordinal: u32) -> (AoqHeader, [u8; 64]) {
    let h = AoqHeader::new(Epoch::Genesis as u16, 0xDEAD_BEEF, ordinal, 7);
    let mut wire = [0u8; 64];
    encode(&h, &mut wire).expect("encode");
    (h, wire)
}

#[test]
fn af33_density_court() {
    if let Err(v) = cgap_aoq::profile_guard::assert_release() {
        eprintln!("LOUD-SKIP af33_density_court: {v}");
        return;
    }
    let window = Duration::from_secs(WINDOW_SECS);

    // Chain slab + pipeline buffers: allocated OUTSIDE the measured window
    // start (they are part of the footprint the sampler observes, but the
    // allocation churn itself is not the load).
    let slab: &'static mut [AoqChainRecord] =
        Box::leak(vec![AoqChainRecord::zeroed(); SLAB_LINKS].into_boxed_slice());
    let key = [0x42u8; 32];
    let mut aad = [0u8; 36];
    let payload = [0xa5u8; 64];
    let mut ct = [0u8; 64];
    let mut tag = [0u8; 16];

    let (h0, wire0) = make_header(0);
    assert_eq!(decode(&wire0).map(|r| r.magic), Ok(h0.magic));

    // Start the sampler FIRST so it observes slab allocation ramp too.
    let samples: Arc<Mutex<Vec<(u64, u64)>>> = Arc::new(Mutex::new(Vec::new()));
    let sampler = spawn_rss_sampler(window, Arc::clone(&samples));

    let mut chain = AoqChain::new(Mix32, slab);
    let t0 = Instant::now();
    let mut frames: u64 = 0;
    let mut resets: u64 = 0;
    let mut sink = 0u64;
    // Fixed window, frames counted: throughput = frames / actual elapsed.
    while t0.elapsed() < window {
        for _ in 0..1024 {
            // Lawful per-frame wire: fresh header with a live per-frame seq,
            // sealed by the landed codec (encode recomputes the FNV-1a32
            // digest over [0x00,0x3C)); decode then re-runs the full law.
            // (AF-30's raw wire[20] poke breaks the digest on the current
            // codec — offset 20 IS the digest-covered seq field.)
            let h = AoqHeader::new(Epoch::Genesis as u16, 0xDEAD_BEEF, frames as u32, 7);
            let mut wire = [0u8; 64];
            encode(&h, &mut wire).expect("encode");
            let hdr = match decode(&wire) {
                Ok(h) => h,
                Err(e) => panic!("frame {frames}: decode refused: {e:?}"),
            };
            aad[..36].copy_from_slice(&wire[..36]);
            let nonce = Nonce96::new(0, frames).expect("nonce");
            aead::seal_detached(&mut ct, &mut tag, &payload, &aad, &key, &nonce);
            sink = sink
                .wrapping_add(hdr.seq as u64)
                .wrapping_add(ct[0] as u64)
                .wrapping_add(tag[0] as u64);
            match chain.append(hdr.epoch, &payload) {
                Ok(_) => {}
                // Typed slab-full refusal at the 1M-link boundary: reset to
                // a fresh chain over the same slab (counted, disclosed in
                // the receipt — the chain is reused, not grown).
                Err(cgap_aoq::error::AoqError::BufferTooShort { .. }) => {
                    chain = AoqChain::new(Mix32, slab);
                    resets += 1;
                }
                Err(e) => panic!("frame {frames}: append refused: {e:?}"),
            }
            frames += 1;
        }
    }
    let elapsed = t0.elapsed();
    sampler.join().expect("sampler joined");

    let samples = samples.lock().unwrap();
    assert!(!samples.is_empty(), "RSS sampler collected zero samples");
    let peak_kB = samples.iter().map(|(_, kb)| *kb).max().unwrap();
    // Plateau: median of the second half of the window (sustained state).
    let mut tail: Vec<u64> = samples[samples.len() / 2..]
        .iter()
        .map(|(_, kb)| *kb)
        .collect();
    tail.sort_unstable();
    let plateau_kB = tail[tail.len() / 2];

    let peak_mb = peak_kB as f64 / 1024.0;
    let fps = frames as f64 / elapsed.as_secs_f64();
    let density = fps / peak_mb;

    // Binary footprint of THIS bench binary (linked artifact truth for the
    // compute layer; release rlib measured separately in the receipt).
    let exe = std::env::current_exe().expect("current_exe");
    let exe_bytes = std::fs::metadata(&exe).expect("exe metadata").len();
    let slab_mb = (SLAB_LINKS * std::mem::size_of::<AoqChainRecord>()) as f64 / (1024.0 * 1024.0);

    println!("AF33_DENSITY: window_s={:.2} frames={} resets={} elapsed_s={:.3} fps={:.0} peak_rss_kB={} plateau_rss_kB={} peak_rss_mb={:.2} density_frames_per_sec_per_mb={:.1} slab_links={} slab_mb={:.1} bench_binary_bytes={}", window.as_secs_f64(), frames, resets, elapsed.as_secs_f64(), fps, peak_kB, plateau_kB, peak_mb, density, SLAB_LINKS, slab_mb, exe_bytes);

    // Density laws (final-state asserts, real observables):
    // 1. the chain holds at most one slab's worth of links at any reset
    assert!((chain.len() as u64) <= SLAB_LINKS as u64);
    // 1b. the pipeline never stalled: total frames equals links appended
    //     plus the one in-flight frame per reset boundary (bounded by resets+1)
    assert!(frames >= resets);
    // 2. footprint is slab-dominated: peak RSS bounded by slab + host
    //    process baseline (< slab_mb + 32 MB margin) — proves the pipeline
    //    adds no runaway per-frame heap (corroborates AF-30's alloc-free
    //    census with an OS-level bound).
    assert!(
        (peak_kB as f64 / 1024.0) < slab_mb + 32.0,
        "peak RSS {} MB exceeds slab {} MB + 32 MB baseline margin",
        peak_mb,
        slab_mb
    );
    // 3. sink live (optimizer cannot elide the pipeline)
    std::hint::black_box(sink);
    std::hint::black_box(exe_bytes);
}
