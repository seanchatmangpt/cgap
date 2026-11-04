//! AF-35 macro-benchmark: the unified transit budget — measured, not summed.
//!
//! The dossier's arithmetic: <10 ns admission + 41.5 ns decode + 34.7 ns
//! slab push + 167 ns BLAKE3 receipt ≈ 253.2 ns total transit (3.9M
//! frames/s derived). Two audit problems this bench closes:
//!
//! 1. the "<10 ns admission" is UNSUPPORTED — it predates the ~30-41 ns
//!    timer floor (AF-26/AF-22 receipts) that bounds any per-call wall
//!    measurement of this kind;
//! 2. summing stage-minima is not a composed-pipeline prediction — cache
//!    warmth, call overhead, and pipeline dependencies (each stage feeds
//!    the next) mean composed != sum in either direction.
//!
//! Pipeline per frame, over LANDED surfaces only:
//!   (1) admission gate — AF-26's fixed `[u64; 10_000]` margin ledger,
//!       raw little-endian field loads off the wire (no decode), typed
//!       `Decision`, wrapping-overflow margin refusals;
//!   (2) header decode — AB-6 codec (`codec::decode`);
//!   (3) slab push — AB-19 pattern: fixed-array slot insertion (128 B
//!       frame into a fixed ring of slots);
//!   (4) BLAKE3 receipt — AB-7 landed chain (`AoqChain::append` via the
//!       `ChainDigest` seam over cgap-blake3, `std` dev-dep).
//!
//! MEASURED-RECORD law (essay-claims): every number printed is a
//! measurement from THIS run. Dossier rows appear as TARGET-UNMEASURED and
//! are fenced by nothing here — the point is the composed-vs-summed table.
//!
//! Courts: 0-alloc witness across the composed loop AND every stage loop;
//! chain integrity post-run (`verify_records` independent re-walk);
//! toxic-frame refusal inside the loop (typed, counted, census asserted
//! against the deterministic plan).
//!
//! Modes: default 1M frames (CI-adjacent); 10M via `--ignored`.

use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use cgap_aoq::chain::{verify_records, AoqChain, AoqChainRecord, ChainDigest};
use cgap_aoq::codec;
use cgap_aoq::header::AoqHeader;

// ---------------------------------------------------------------------------
// Counting allocator (alloc census; court instrumentation)
// ---------------------------------------------------------------------------

static ALLOC_CALLS: AtomicUsize = AtomicUsize::new(0);
static ALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);

struct Counting;

#[global_allocator]
static GLOBAL: Counting = Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(l.size(), Ordering::Relaxed);
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        unsafe { System.dealloc(p, l) }
    }
}

// ---------------------------------------------------------------------------
// Digest substrate: REAL cgap-blake3 through the landed ChainDigest seam
// ---------------------------------------------------------------------------

/// Order-sensitive fixed-width concat (1-byte length prefix per part; the
/// chain's 4-part input is fixed-width 32/2/8/32, so 1 byte suffices).
#[derive(Clone, Copy)]
struct Blake3Digest;
impl ChainDigest for Blake3Digest {
    fn digest(&self, parts: &[&[u8]]) -> [u8; 32] {
        let mut buf = [0u8; 4 * (1 + 32)];
        let mut w = 0;
        for p in parts {
            buf[w] = p.len() as u8;
            w += 1;
            buf[w..w + p.len()].copy_from_slice(p);
            w += p.len();
        }
        cgap_blake3::hash(&buf[..w])
    }
}

// ---------------------------------------------------------------------------
// Admission gate — AF-26's pattern, driven off RAW WIRE (pre-decode)
// ---------------------------------------------------------------------------

const INSTRUMENTS: usize = 10_000;
const BODY_CAP: usize = 64;
const ADMITTED_KEY_ID: u64 = 7;

/// Typed gate decision off raw wire fields. Never panics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Decision {
    Admit,
    /// Margin: u64::MAX notional wraps the ledger slot.
    MarginExceeded,
    /// Foreign key_id on the wire (no decode needed to see it).
    UnauthorizedCounterparty,
    /// Header magic refused (can only be seen after decode; this bench's
    /// plan never produces one — census slot exists for symmetry).
    HeaderRefused,
}

/// Fixed 10k-slot margin ledger. Wrapping arithmetic, zero allocation.
struct Gate {
    ledger: Box<[u64; INSTRUMENTS]>,
}

impl Gate {
    fn new() -> Self {
        // Nonzero opening margin: u64::MAX notional ALWAYS wraps.
        Self {
            ledger: Box::new([1_000u64; INSTRUMENTS]),
        }
    }

    /// Raw-wire admission check: key_id at [8,16), seq at [20,24),
    /// notional at body[0,8). Runs BEFORE decode in the pipeline order.
    fn admit_raw(&mut self, wire: &[u8], body_len: usize) -> Decision {
        let mut kb = [0u8; 8];
        kb.copy_from_slice(&wire[8..16]);
        let key_id = u64::from_le_bytes(kb);
        if key_id != ADMITTED_KEY_ID {
            return Decision::UnauthorizedCounterparty;
        }
        if body_len > BODY_CAP {
            return Decision::HeaderRefused; // oversize class not planned here
        }
        let mut sb = [0u8; 4];
        sb.copy_from_slice(&wire[20..24]);
        let seq = u32::from_le_bytes(sb);
        let instrument = (seq % INSTRUMENTS as u32) as usize;
        let mut nb = [0u8; 8];
        nb.copy_from_slice(&wire[64..72]);
        let notional = u64::from_le_bytes(nb);
        let slot = &mut self.ledger[instrument];
        let new = slot.wrapping_add(notional);
        if new < *slot {
            return Decision::MarginExceeded;
        }
        *slot = new;
        Decision::Admit
    }
}

// ---------------------------------------------------------------------------
// Frame plan (deterministic from index alone)
// ---------------------------------------------------------------------------

const WIRE_BYTES: usize = 64 + BODY_CAP; // 128 B clean frame

struct Xorshift(u64);

impl Xorshift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

/// Every 10th frame is toxic (u64::MAX notional -> margin wrap refusal).
/// Frames are pre-built before any timing: the loops measure transit, not
/// frame manufacture or encode.
const fn is_toxic(i: usize) -> bool {
    i % 10 == 9
}

fn build_frames(n: usize) -> &'static mut [[u8; WIRE_BYTES]] {
    let mut rng = Xorshift(0xAF35_2026);
    let mut header_buf = [0u8; 64];
    let mut frames: Vec<[u8; WIRE_BYTES]> = Vec::with_capacity(n);
    for i in 0..n {
        let notional = if is_toxic(i) { u64::MAX } else { rng.next() % 1_000 };
        let key_id = ADMITTED_KEY_ID; // plan: margin-toxic only
        let header = AoqHeader::new(1, key_id, i as u32, (i % INSTRUMENTS) as u32);
        codec::encode(&header, &mut header_buf).expect("bench: lawful header");
        let mut f = [0u8; WIRE_BYTES];
        f[..64].copy_from_slice(&header_buf);
        f[64..72].copy_from_slice(&notional.to_le_bytes());
        for b in &mut f[72..] {
            *b = rng.next() as u8;
        }
        frames.push(f);
    }
    Box::leak(frames.into_boxed_slice())
}

// ---------------------------------------------------------------------------
// Slab: AB-19 pattern — fixed-array slot insertion (ring of frame slots)
// ---------------------------------------------------------------------------

const SLOTS: usize = 1 << 17; // 131072 frame slots
const SLOT_BYTES: usize = WIRE_BYTES;

fn run_transit(frames_n: usize, with_per_frame_timer: bool) -> TransitResult {
    let frames = build_frames(frames_n);
    let slab: &'static mut [[u8; SLOT_BYTES]] =
        Box::leak(vec![[0u8; SLOT_BYTES]; SLOTS].into_boxed_slice());
    let chain_slab: &'static mut [AoqChainRecord] =
        Box::leak(vec![AoqChainRecord::zeroed(); frames_n].into_boxed_slice());
    let mut gate = Gate::new();
    let mut chain = AoqChain::new(Blake3Digest, chain_slab);

    let mut census = [0usize; 2]; // margin, other
    let mut admitted: usize = 0;
    let mut lat_ns: Vec<u32> = if with_per_frame_timer {
        Vec::with_capacity(frames_n)
    } else {
        Vec::new()
    };

    // Preallocations complete; the census snapshot bounds the loops below.
    let alloc0_calls = ALLOC_CALLS.load(Ordering::Relaxed);
    let alloc0_bytes = ALLOC_BYTES.load(Ordering::Relaxed);

    // --- composed pipeline: all four stages per frame ---
    let t0 = Instant::now();
    for (i, f) in frames.iter().enumerate() {
        let t_start = if with_per_frame_timer {
            Some(Instant::now())
        } else {
            None
        };
        black_box(f);
        // (1) admission gate — raw wire, pre-decode
        let decision = gate.admit_raw(f, BODY_CAP);
        // (2) header decode (AB-6 codec)
        let decoded = codec::decode(&f[..64]);
        if decoded.is_err() {
            census[1] += 1;
            if let Some(ts) = t_start {
                let d = ts.elapsed();
                lat_ns.push((d.as_secs() * 1_000_000_000 + u64::from(d.subsec_nanos())) as u32);
            }
            continue;
        }
        // (3) slab push — fixed-array slot insertion (AB-19)
        slab[i % SLOTS] = *f;
        black_box(&slab[i % SLOTS]);
        // (4) BLAKE3 receipt — landed AB-7 chain append (admitted frames)
        if decision == Decision::Admit {
            chain.append(1, &f[64..]).expect("bench: chain append");
            admitted += 1;
        } else {
            census[0] += 1;
        }
        if let Some(ts) = t_start {
            let d = ts.elapsed();
            lat_ns.push((d.as_secs() * 1_000_000_000 + u64::from(d.subsec_nanos())) as u32);
        }
    }
    let wall = t0.elapsed();

    let alloc_calls = ALLOC_CALLS.load(Ordering::Relaxed) - alloc0_calls;
    let alloc_bytes = ALLOC_BYTES.load(Ordering::Relaxed) - alloc0_bytes;

    // Court: chain integrity post-run — independent re-walk of every link.
    verify_records(&Blake3Digest, chain.records()).expect("chain integrity post-run");

    TransitResult {
        frames: frames_n,
        wall,
        lat_ns,
        admitted,
        margin_refusals: census[0],
        other_refusals: census[1],
        alloc_calls,
        alloc_bytes,
        links: chain.len(),
        head: chain.checkpoint(),
    }
}

struct TransitResult {
    frames: usize,
    wall: Duration,
    lat_ns: Vec<u32>,
    admitted: usize,
    margin_refusals: usize,
    other_refusals: usize,
    alloc_calls: usize,
    alloc_bytes: usize,
    links: usize,
    head: [u8; 32],
}

// ---------------------------------------------------------------------------
// Stage isolation: each stage alone in the same-shaped loop
// ---------------------------------------------------------------------------

/// Timer-floor measurement: cost of an `Instant::now()`/`elapsed` pair.
fn measure_timer_floor(n: usize) -> f64 {
    let t0 = Instant::now();
    let mut acc = 0u64;
    for _ in 0..n {
        let s = Instant::now();
        acc = acc.wrapping_add(s.elapsed().as_nanos() as u64);
    }
    let wall = t0.elapsed();
    black_box(acc);
    wall.as_secs_f64() * 1e9 / n as f64
}

fn run_stage(frames: &[[u8; WIRE_BYTES]], stage: Stage) -> Duration {
    let slab: &'static mut [[u8; SLOT_BYTES]] =
        Box::leak(vec![[0u8; SLOT_BYTES]; SLOTS].into_boxed_slice());
    let chain_slab: &'static mut [AoqChainRecord] =
        Box::leak(vec![AoqChainRecord::zeroed(); frames.len()].into_boxed_slice());
    let mut gate = Gate::new();
    let mut chain = AoqChain::new(Blake3Digest, chain_slab);
    let mut admitted: usize = 0;
    let mut refused: usize = 0;

    let alloc0_calls = ALLOC_CALLS.load(Ordering::Relaxed);
    let t0 = Instant::now();
    for (i, f) in frames.iter().enumerate() {
        black_box(f);
        match stage {
            Stage::GateOnly => {
                let d = gate.admit_raw(f, BODY_CAP);
                if d == Decision::Admit {
                    admitted += 1;
                } else {
                    refused += 1;
                }
            }
            Stage::DecodeOnly => {
                let h = codec::decode(&f[..64]).expect("bench: lawful header");
                black_box(h);
            }
            Stage::SlabOnly => {
                slab[i % SLOTS] = *f;
                black_box(&slab[i % SLOTS]);
            }
            Stage::Blake3HashOnly => {
                let d = cgap_blake3::hash(f);
                black_box(d);
            }
            Stage::ReceiptOnly => {
                // Landed chain append: H(payload) + H(record) = 2 BLAKE3
                // invocations + record slab write, via the AB-7 API.
                chain.append(1, &f[64..]).expect("bench: chain append");
                admitted += 1;
            }
        }
    }
    let wall = t0.elapsed();
    let alloc_calls = ALLOC_CALLS.load(Ordering::Relaxed) - alloc0_calls;
    assert_eq!(alloc_calls, 0, "stage loop {stage:?} allocated");
    black_box((admitted, refused));
    wall
}

#[derive(Debug, Clone, Copy)]
enum Stage {
    GateOnly,
    DecodeOnly,
    SlabOnly,
    Blake3HashOnly,
    ReceiptOnly,
}

// ---------------------------------------------------------------------------
// Percentiles + report
// ---------------------------------------------------------------------------

fn percentile(sorted: &[u32], p: f64) -> u32 {
    let n = sorted.len();
    if n == 0 {
        return 0;
    }
    let idx = ((p / 100.0) * (n as f64 - 1.0)).round() as usize;
    sorted[idx.min(n - 1)]
}

struct StageNs {
    gate: f64,
    decode: f64,
    slab: f64,
    hash_raw: f64,
    receipt_chain: f64,
}

fn report(name: &str, r: &TransitResult, stages: &StageNs, timer_floor_ns: f64) {
    let per_frame = r.wall.as_secs_f64() * 1e9 / r.frames as f64;
    let tput = r.frames as f64 / r.wall.as_secs_f64();
    let expected_toxic = (0..r.frames).filter(|i| is_toxic(*i)).count();

    println!("\n=== AF-35 unified transit budget — {name} ===");
    println!(
        "frames: {} (admitted {}, margin-refused {})  chain links {}",
        r.frames, r.admitted, r.margin_refusals, r.links
    );
    println!(
        "composed wall: {:.3?}  ->  per-frame {:.1} ns  ({:.0} frames/s)",
        r.wall, per_frame, tput
    );
    if !r.lat_ns.is_empty() {
        let mut sorted = r.lat_ns.clone();
        sorted.sort_unstable();
        let p50 = percentile(&sorted, 50.0);
        let p95 = percentile(&sorted, 95.0);
        let p99 = percentile(&sorted, 99.0);
        println!(
            "per-frame timer latency ns (timer-inflated, floor ~{:.0} ns): p50 {}  p95 {}  p99 {}  max {}",
            timer_floor_ns,
            p50,
            p95,
            p99,
            sorted.last().copied().unwrap_or(0)
        );
    }
    println!("measured timer floor (Instant pair, this machine): {:.1} ns", timer_floor_ns);
    println!("\nstage attribution (isolated loops, ns/frame):");
    println!("  gate (raw-wire margin ledger): {:.1}", stages.gate);
    println!("  decode (AB-6 codec):           {:.1}", stages.decode);
    println!("  slab push (AB-19 128B slot):   {:.1}", stages.slab);
    println!("  blake3 raw hash 128B:          {:.1}", stages.hash_raw);
    println!("  receipt (AB-7 chain append):   {:.1}  (2x blake3 + record write)", stages.receipt_chain);
    let summed_min = stages.gate + stages.decode + stages.slab + stages.hash_raw;
    let summed_chain = stages.gate + stages.decode + stages.slab + stages.receipt_chain;
    println!("\ncomposed-vs-summed (dossier additive model uses raw hash):");
    println!("  summed (gate+decode+slab+raw hash): {:.1} ns", summed_min);
    println!("  summed (gate+decode+slab+chain receipt): {:.1} ns", summed_chain);
    println!(
        "  composed measured: {:.1} ns  ->  divergence vs summed(raw): {:+.1} ns ({:+.0}%)",
        per_frame,
        per_frame - summed_min,
        (per_frame / summed_min - 1.0) * 100.0
    );
    println!(
        "  composed frames/s: {:.0}  (dossier TARGET-UNMEASURED: 3.9M/s @ 253.2 ns)",
        tput
    );
    println!("  alloc census: {} calls / {} bytes (target 0)", r.alloc_calls, r.alloc_bytes);
    println!("  chain head {:02x?}..., verify_records: OK", &r.head[..4]);

    // --- Courts/gates ---
    assert_eq!(
        r.margin_refusals, expected_toxic,
        "toxic margin census drift"
    );
    assert_eq!(r.other_refusals, 0, "unexpected codec refusals");
    assert_eq!(r.admitted + r.margin_refusals, r.frames, "admit+refuse conservation");
    assert_eq!(r.links, r.admitted, "chain link conservation");
    assert_eq!(r.alloc_calls, 0, "composed loop allocated {} times", r.alloc_calls);
    assert_eq!(r.alloc_bytes, 0, "composed loop allocated bytes");
}

/// Default: 1M frames, composed pass (timer-free) + timer pass + stages.
#[test]
fn composed_transit_1m() {
    let n = 1_000_000;
    let timer_floor_ns = measure_timer_floor(200_000);

    // Composed, timer-free (headline number).
    let r = run_transit(n, false);
    // Composed, per-frame Instant (percentiles; timer-inflated).
    let r_timed = run_transit(n, true);
    // Stage isolation.
    let frames = build_frames(n);
    let s = |st: Stage| run_stage(frames, st).as_secs_f64() * 1e9 / n as f64;
    let stages = StageNs {
        gate: s(Stage::GateOnly),
        decode: s(Stage::DecodeOnly),
        slab: s(Stage::SlabOnly),
        hash_raw: s(Stage::Blake3HashOnly),
        receipt_chain: s(Stage::ReceiptOnly),
    };

    report("composed 1M frames", &r, &stages, timer_floor_ns);
    report("composed 1M frames (per-frame timer)", &r_timed, &stages, timer_floor_ns);

    // The timed pass must satisfy the same courts.
    assert_eq!(r_timed.margin_refusals, r.margin_refusals);
    assert_eq!(r_timed.alloc_calls, 0);
    assert_eq!(r_timed.links, r_timed.admitted);
}

/// Full mode: 10M frames via `--ignored`.
#[test]
#[ignore]
fn composed_transit_10m() {
    let r = run_transit(10_000_000, false);
    let timer_floor_ns = measure_timer_floor(200_000);
    let frames = build_frames(10_000_000);
    let s = |st: Stage| run_stage(frames, st).as_secs_f64() * 1e9 / 10_000_000 as f64;
    let stages = StageNs {
        gate: s(Stage::GateOnly),
        decode: s(Stage::DecodeOnly),
        slab: s(Stage::SlabOnly),
        hash_raw: s(Stage::Blake3HashOnly),
        receipt_chain: s(Stage::ReceiptOnly),
    };
    report("composed 10M frames", &r, &stages, timer_floor_ns);
}
