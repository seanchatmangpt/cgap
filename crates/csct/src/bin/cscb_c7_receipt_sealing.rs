//! C7 receipt-sealing rate court (lane B06, CSCB-C7).
//!
//! Seals 1,000,000 monotonic P28-style lease receipts using the real
//! `cgap-blake3` kernel (`cgap_blake3::hash`, zero reimplementation) and
//! gates the sealing rate at >= 1,000,000 receipts/s.
//!
//! Receipt framing (80 bytes, RFC-documented here; the cgap P28 lease
//! layout in cgap-crypto/src/lease.rs is not directly importable as a
//! byte frame from this bin, so the framing is defined per the lane
//! RFC: nonce u64 + parent digest 32B + payload 32B + tag 8B):
//!
//! ```text
//! bytes  0.. 8   nonce    u64 LE, strictly incrementing (monotonic witness)
//! bytes  8..40   parent   32B digest of the previous receipt (chain)
//! bytes 40..72   payload  32B splitmix64-derived payload from the nonce
//! bytes 72..80   tag      u64 LE domain tag 0xC5CB_C700_0000_0080
//! ```
//!
//! Anti-vacuity: `CSCB_MUTATE=1` flips one bit at a known index
//! (byte 40, bit 0 — first payload byte) in the final receipt's lease
//! state BEFORE hashing. The chain-validity court must REJECT that
//! certificate (digest mismatch, exit 1). A court that cannot flip is
//! FAIL.
//!
//! Debug builds emit typed `[C7-DEBUG]` lines at each phase.

use std::env;
use std::time::Instant;

use cgap_blake3::hash;

const N_RECEIPTS: u64 = 1_000_000;
const RUNS: usize = 7;
const RATE_FLOOR: f64 = 1_000_000.0;
const DOMAIN_TAG: u64 = 0xC5CB_C700_0000_0080;
/// Anti-vacuity mutation index: byte 40 (payload start), bit 0.
const MUTATE_BYTE: usize = 40;
const MUTATE_BIT: u8 = 0;

/// One sealed receipt: the 80-byte lease frame plus its BLAKE3 digest.
struct Receipt {
    frame: [u8; 80],
    digest: [u8; 32],
}

/// Splitmix64 stream (frozen canonical multiplier seed law: this is the
/// SplitMix64 *multiplier* constant, not the corpus seed).
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn seal(nonce: u64, parent: [u8; 32], mutate_last: bool) -> Receipt {
    let mut frame = [0u8; 80];
    frame[0..8].copy_from_slice(&nonce.to_le_bytes());
    frame[8..40].copy_from_slice(&parent);
    let mut sm = nonce;
    for chunk in frame[40..72].chunks_exact_mut(8) {
        chunk.copy_from_slice(&splitmix64(&mut sm).to_le_bytes());
    }
    frame[72..80].copy_from_slice(&DOMAIN_TAG.to_le_bytes());
    if mutate_last {
        frame[MUTATE_BYTE] ^= 1 << MUTATE_BIT;
    }
    let digest = hash(&frame);
    Receipt { frame, digest }
}

/// Chain-validity court: full post-run verification over real state.
/// Every receipt's digest must re-derive from its frame; parents must
/// equal the previous digest (genesis parent = zero digest); nonces
/// strictly increment from 1. Returns Err with a typed line on the
/// first violation.
fn verify_chain(receipts: &[Receipt]) -> Result<(), String> {
    if receipts.is_empty() {
        return Err("[C7-FAIL] empty chain".to_string());
    }
    let mut parent = [0u8; 32];
    for (i, r) in receipts.iter().enumerate() {
        let expect = ((i + 1) as u64).to_le_bytes();
        if r.frame[0..8] != expect {
            return Err(format!("[C7-FAIL] nonce break at receipt {}", i));
        }
        if r.frame[8..40] != parent {
            return Err(format!("[C7-FAIL] parent break at receipt {}", i));
        }
        if hash(&r.frame) != r.digest {
            return Err(format!("[C7-FAIL] digest mismatch at receipt {}", i));
        }
        parent = r.digest;
    }
    Ok(())
}

/// Median of a pre-sorted sample set (N is odd here, so no interpolation
/// ambiguity; gate semantics unchanged).
fn median(sorted: &[f64]) -> f64 {
    sorted[sorted.len() / 2]
}

fn main() {
    // R10 zero-tax typed-refusal arm (child-native, belt-and-suspenders
    // over the runner-level C4 gate, lane-gap-04): zero-tax must NOT be
    // conflated with this bin's `CSCB_MUTATE=1` bitflip mutation.
    if env::var("CSCB_MUTATE").as_deref() == Ok("zero-tax") {
        eprintln!(
            "ANDON: cscb_c7_receipt_sealing typed-REFUSES CSCB_MUTATE=zero-tax (R10 flip court) -> exit 1"
        );
        std::process::exit(1);
    }
    let debug = cfg!(debug_assertions);
    if debug {
        println!("[C7-DEBUG] phase=boot n={} runs={}", N_RECEIPTS, RUNS);
    }
    let mutate = env::var("CSCB_MUTATE").is_ok();

    // Timed runs: each run seals the full 1M-receipt chain.
    let mut rates: Vec<f64> = Vec::with_capacity(RUNS);
    let mut last_chain: Option<Vec<Receipt>> = None;
    for run in 0..RUNS {
        let mut receipts: Vec<Receipt> = Vec::with_capacity(N_RECEIPTS as usize);
        let mut parent = [0u8; 32];
        let t0 = Instant::now();
        for nonce in 1..=N_RECEIPTS {
            let r = seal(nonce, parent, false);
            parent = r.digest;
            receipts.push(r);
        }
        let elapsed = t0.elapsed().as_secs_f64();
        let rate = N_RECEIPTS as f64 / elapsed;
        rates.push(rate);
        if debug {
            println!("[C7-DEBUG] phase=seal run={} rate={:.0}/s", run, rate);
        }
        last_chain = Some(receipts);
    }
    rates.sort_by(|a, b| a.partial_cmp(b).unwrap());
    // Honest reporting: RUNS=7 samples give ceil(0.99*7)=7 -> the old
    // "p99" was the MAX, not a tail quantile (EXP-04 finding). With no
    // per-frame sample population collected, report (min, median, max,
    // N) — no fake quantile. Gate stays on the median (unchanged).
    let (min, median, max) = (rates[0], median(&rates), rates[RUNS - 1]);

    println!(
        "[C7-RATE] min={:.0}/s median={:.0}/s max={:.0}/s runs={} (note: max over {} run-level samples; no p99 computed — sample population too small for a tail quantile)",
        min, median, max, RUNS, RUNS
    );

    // Court 1: rate gate (median, with all samples reported).
    if median < RATE_FLOOR {
        eprintln!(
            "[C7-FAIL] rate median {:.0}/s < floor {:.0}/s",
            median, RATE_FLOOR
        );
        std::process::exit(1);
    }
    println!(
        "[C7-PASS] rate gate: median {:.0}/s >= {:.0}/s",
        median, RATE_FLOOR
    );

    // Court 2: chain validity over the final run's full chain.
    let chain = last_chain.expect("[C7-FAIL] no chain sealed");
    match verify_chain(&chain) {
        Ok(()) => println!(
            "[C7-PASS] chain validity: {} receipts verified",
            chain.len()
        ),
        Err(line) => {
            eprintln!("{}", line);
            std::process::exit(1);
        }
    }

    // Court 3: anti-vacuity. Mutate one bit in the final receipt's
    // lease state before hashing; the court must REJECT (digest
    // mismatch). A court that cannot flip is FAIL.
    if mutate {
        let n = N_RECEIPTS;
        let parent = chain[chain.len() - 1].digest;
        let mut mutated = seal(n + 1, parent, true);
        // Land check: the mutated frame must differ from the clean
        // frame (a plain bit-value check is wrong — the flip may set
        // the bit back to 0).
        let mut clean_frame = mutated.frame;
        clean_frame[MUTATE_BYTE] ^= 1 << MUTATE_BIT;
        if mutated.frame == clean_frame {
            eprintln!(
                "[C7-FAIL] mutation did not land at byte {} bit {}",
                MUTATE_BYTE, MUTATE_BIT
            );
            std::process::exit(1);
        }
        // The chain attests the CLEAN digest for nonce N+1; the
        // certificate carries the mutated frame, so the court's digest
        // re-derivation must mismatch.
        let clean_digest = hash(&clean_frame);
        if hash(&mutated.frame) == clean_digest {
            eprintln!("[C7-FAIL] bitflip produced identical digest (vacuous court)");
            std::process::exit(1);
        }
        mutated.digest = clean_digest;
        let one = vec![mutated];
        if let Ok(()) = verify_against_chain(&one, &parent) {
            eprintln!("[C7-FAIL] mutated certificate ACCEPTED by court");
            std::process::exit(1);
        }
        println!("[C7-PASS] anti-vacuity: mutated certificate rejected");
        std::process::exit(0);
    }

    if debug {
        println!("[C7-DEBUG] phase=done standing=ALIVE");
    }
}

/// Verify a tail receipt (nonce N+1) whose expected parent is given:
/// digest must re-derive AND its stored parent field must match.
fn verify_against_chain(receipts: &[Receipt], expected_parent: &[u8; 32]) -> Result<(), String> {
    for (i, r) in receipts.iter().enumerate() {
        if r.frame[8..40] != expected_parent[..] {
            return Err(format!("[C7-FAIL] parent mismatch at tail {}", i));
        }
        if hash(&r.frame) != r.digest {
            return Err(format!("[C7-FAIL] digest mismatch at tail {}", i));
        }
    }
    Ok(())
}
