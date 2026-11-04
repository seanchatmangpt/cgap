//! AF-26 macro-benchmark: Flash Crash Clearinghouse.
//!
//! Deterministic seeded order stream (xorshift64*, seed `0xAF26_2026`) as
//! AOQ1 frames over a REAL unix-domain socketpair. Per-frame transfer
//! function: (1) decode the 64-byte AOQ1 header (AB-6 codec), (2) risk
//! gate — fixed `[u64; 10_000]` margin ledger indexed by instrument id
//! (`seq % 10_000`), wrapping arithmetic, no alloc — admit/reject typed,
//! (3) chain append receipt per admitted frame (AB-7 `AoqChain`,
//! cgap-arrow BLAKE3 as the real digest substrate).
//!
//! Toxic storm: 10% of frames are toxic (every 10th), cycling through
//! three refusal classes:
//! - `MarginExceeded` — collateral notional `u64::MAX` overflows the
//!   ledger slot (wrapping detect); caught by the gate
//! - `UnauthorizedCounterparty` — `key_id = 0xDEADBEEF`, foreign
//! - `OversizeBody` — 240 B body over the 64 B cap
//!
//! The consumer derives each frame's expected class and wire length from
//! the frame index alone (deterministic plan); the refusal census is
//! asserted against that plan — typed refusals, never panics.
//!
//! MEASURED-RECORD law (essay-claims): every number printed is a
//! measurement. The dossier numbers (sub-10 ns gate, p99 < 1 µs, 20M+
//! frames/s) appear only as TARGET-UNMEASURED rows in the receipt — this
//! bench fences nothing on the dossier.
//!
//! Modes: default scaled 1M frames (CI-adjacent); full 10M via
//! `cargo test -p cgap-aoq --test macro_clearing_stress -- --ignored`.
//!
//! The counting allocator is the crate's standard court pattern
//! (ab8_soak.rs); its `unsafe` blocks are the `GlobalAlloc` contract
//! itself — no bench logic is unsafe.

use std::alloc::{GlobalAlloc, Layout, System};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use cgap_aoq::chain::{verify_records, AoqChain, AoqChainRecord, ChainDigest};
use cgap_aoq::codec;
use cgap_aoq::error::AoqError;
use cgap_aoq::header::AoqHeader;
use cgap_arrow::blake3;

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
// Determinism
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Digest substrate (real BLAKE3 via cgap-arrow — same seam as ab8_soak)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct Blake3Digest;
impl ChainDigest for Blake3Digest {
    fn digest(&self, parts: &[&[u8]]) -> [u8; 32] {
        // Order-sensitive: 1-byte length prefix per part (fixed widths
        // 32/2/8/32 in the chain's 4-part hash input, so 1 byte suffices).
        let mut buf = [0u8; 4 * (1 + 32)];
        let mut w = 0;
        for p in parts {
            buf[w] = p.len() as u8;
            w += 1;
            buf[w..w + p.len()].copy_from_slice(p);
            w += p.len();
        }
        blake3::hash(&buf[..w])
    }
}

// ---------------------------------------------------------------------------
// Risk gate (AB-19 slab pattern: fixed array, no alloc, no panic)
// ---------------------------------------------------------------------------

const INSTRUMENTS: usize = 10_000;
const BODY_CAP: usize = 64;
const ADMITTED_KEY_ID: u64 = 7;
const FOREIGN_KEY_ID: u64 = 0xDEAD_BEEF;

// Census class codes (gate-local; header-law refusals share 0x10_04).
const C_MARGIN: u16 = 0x10_01;
const C_UNAUTHORIZED: u16 = 0x10_02;
const C_OVERSIZE: u16 = 0x10_03;
const C_HEADER: u16 = 0x10_04;

/// Typed gate decision. Never panics; every refusal carries a stable
/// class code for the census.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Decision {
    Admit,
    MarginExceeded { instrument: u32, notional: u64 },
    UnauthorizedCounterparty { got: u64 },
    OversizeBody { got: usize },
    HeaderRefused(AoqError),
}

impl Decision {
    const fn class_code(&self) -> u16 {
        match *self {
            Self::Admit => 0,
            Self::MarginExceeded { .. } => C_MARGIN,
            Self::UnauthorizedCounterparty { .. } => C_UNAUTHORIZED,
            Self::OversizeBody { .. } => C_OVERSIZE,
            Self::HeaderRefused(_) => C_HEADER,
        }
    }
}

/// Fixed 10k-slot margin ledger. Wrapping arithmetic, zero allocation.
struct Gate {
    ledger: Box<[u64; INSTRUMENTS]>,
}

impl Gate {
    fn new() -> Self {
        // Opening margin per instrument: nonzero, so a u64::MAX notional
        // ALWAYS wraps (first-touch frames refuse like any other —
        // census stays deterministic against the plan).
        Self {
            ledger: Box::new([1_000u64; INSTRUMENTS]),
        }
    }

    fn admit(&mut self, h: &AoqHeader, body: &[u8]) -> Decision {
        if h.key_id != ADMITTED_KEY_ID {
            return Decision::UnauthorizedCounterparty { got: h.key_id };
        }
        if body.len() > BODY_CAP {
            return Decision::OversizeBody { got: body.len() };
        }
        let instrument = (h.seq % INSTRUMENTS as u32) as usize;
        let mut nb = [0u8; 8];
        nb.copy_from_slice(&body[0..8]);
        let notional = u64::from_le_bytes(nb);
        let slot = &mut self.ledger[instrument];
        let new = slot.wrapping_add(notional);
        if new < *slot {
            return Decision::MarginExceeded {
                instrument: instrument as u32,
                notional,
            };
        }
        *slot = new;
        Decision::Admit
    }
}

// ---------------------------------------------------------------------------
// Frame plan: class + wire length derived from the frame index alone
// ---------------------------------------------------------------------------

const WIRE_CLEAN: usize = 64 + BODY_CAP; // 128 B
const WIRE_OVERSIZE: usize = 64 + 240; // 304 B

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Toxic {
    Margin,
    Unauthorized,
    Oversize,
}

fn plan(i: usize) -> (Option<Toxic>, usize) {
    if i % 10 == 9 {
        let toxic = match (i / 10) % 3 {
            0 => Toxic::Margin,
            1 => Toxic::Unauthorized,
            _ => Toxic::Oversize,
        };
        let wire_len = if toxic == Toxic::Oversize {
            WIRE_OVERSIZE
        } else {
            WIRE_CLEAN
        };
        (Some(toxic), wire_len)
    } else {
        (None, WIRE_CLEAN)
    }
}

// ---------------------------------------------------------------------------
// Soak
// ---------------------------------------------------------------------------

struct SoakResult {
    frames: usize,
    wall: Duration,
    lat_ns: Vec<u32>,
    admitted: usize,
    margin_refusals: usize,
    unauthorized_refusals: usize,
    oversize_refusals: usize,
    header_refusals: usize,
    alloc_calls: usize,
    alloc_bytes: usize,
    links: usize,
    head: [u8; 32],
}

fn run_soak(frames: usize) -> SoakResult {
    let (mut tx, mut rx) = UnixStream::pair().expect("socketpair");

    // Producer: builds each frame on the fly (encode is FNV-cheap), writes
    // wire units of the plan's length. Blocks when the socket buffer is
    // full — backpressure, not loss.
    let producer = thread::spawn(move || {
        let mut rng = Xorshift(0xAF26_2026);
        let mut header_buf = [0u8; 64];
        let mut body_clean: [u8; BODY_CAP];
        let mut body_big: [u8; 240];
        for i in 0..frames {
            let (toxic, wire_len) = plan(i);
            // Margin-toxic: u64::MAX notional overflows the ledger slot.
            let notional = if toxic == Some(Toxic::Margin) {
                u64::MAX
            } else {
                rng.next() % 1_000
            };
            // Clean body: seeded tail, modest notional.
            body_clean = [0u8; BODY_CAP];
            for b in body_clean.iter_mut() {
                *b = rng.next() as u8;
            }
            body_clean[0..8].copy_from_slice(&notional.to_le_bytes());
            let key_id = if toxic == Some(Toxic::Unauthorized) {
                FOREIGN_KEY_ID
            } else {
                ADMITTED_KEY_ID
            };
            let header = AoqHeader::new(1, key_id, i as u32, (i % INSTRUMENTS) as u32);
            codec::encode(&header, &mut header_buf).expect("bench: lawful header");
            if wire_len == WIRE_CLEAN {
                tx.write_all(&header_buf).unwrap();
                tx.write_all(&body_clean).unwrap();
            } else {
                // Oversize: seeded 240B body with the same notional head.
                body_big = [0u8; 240];
                for b in body_big.iter_mut() {
                    *b = rng.next() as u8;
                }
                body_big[0..8].copy_from_slice(&notional.to_le_bytes());
                tx.write_all(&header_buf).unwrap();
                tx.write_all(&body_big).unwrap();
            }
        }
    });

    // Consumer-side preallocations BEFORE the alloc census snapshot.
    let slab: &'static mut [AoqChainRecord] =
        Box::leak(vec![AoqChainRecord::zeroed(); frames].into_boxed_slice());
    let mut lat_ns: Vec<u32> = Vec::with_capacity(frames);
    let mut gate = Gate::new();
    let mut frame = [0u8; WIRE_OVERSIZE];

    let mut census = [0usize; 4]; // margin, unauthorized, oversize, header
    let mut admitted: usize = 0;

    let alloc0_calls = ALLOC_CALLS.load(Ordering::Relaxed);
    let alloc0_bytes = ALLOC_BYTES.load(Ordering::Relaxed);
    let t0 = Instant::now();

    let mut chain = AoqChain::new(Blake3Digest, slab);

    for i in 0..frames {
        let (_, wire_len) = plan(i);
        let t_start = Instant::now();

        // (1) transport: read exactly one wire unit
        rx.read_exact(&mut frame[..wire_len]).expect("uds read");

        // (2) decode (AB-6 codec) — typed refusal, never panic
        let decoded: Result<AoqHeader, AoqError> = codec::decode(&frame[..64]);
        // (3) risk gate
        let decision = match decoded {
            Ok(h) => gate.admit(&h, &frame[64..wire_len]),
            Err(e) => Decision::HeaderRefused(e),
        };
        // (4) chain append receipt per admitted frame
        if decision == Decision::Admit {
            chain.append(1, &frame[64..wire_len]).expect("chain append");
            admitted += 1;
        } else {
            let code = decision.class_code();
            let bucket = match code {
                C_MARGIN => 0,
                C_UNAUTHORIZED => 1,
                C_OVERSIZE => 2,
                _ => 3,
            };
            census[bucket] += 1;
        }
        let d = t_start.elapsed();
        lat_ns.push((d.as_secs() * 1_000_000_000 + u64::from(d.subsec_nanos())) as u32);
    }
    let wall = t0.elapsed();

    producer.join().expect("producer thread");

    let alloc_calls = ALLOC_CALLS.load(Ordering::Relaxed) - alloc0_calls;
    let alloc_bytes = ALLOC_BYTES.load(Ordering::Relaxed) - alloc0_bytes;

    // Chain integrity post-soak: independent re-walk of every landed link.
    verify_records(&Blake3Digest, chain.records()).expect("chain integrity post-soak");

    SoakResult {
        frames,
        wall,
        lat_ns,
        admitted,
        margin_refusals: census[0],
        unauthorized_refusals: census[1],
        oversize_refusals: census[2],
        header_refusals: census[3],
        alloc_calls,
        alloc_bytes,
        links: chain.len(),
        head: chain.checkpoint(),
    }
}

// ---------------------------------------------------------------------------
// Percentiles
// ---------------------------------------------------------------------------

fn percentile(sorted: &[u32], p: f64) -> u32 {
    let n = sorted.len();
    if n == 0 {
        return 0;
    }
    let idx = ((p / 100.0) * (n as f64 - 1.0)).round() as usize;
    sorted[idx.min(n - 1)]
}

fn report(name: &str, r: &SoakResult) {
    let mut sorted = r.lat_ns.clone();
    sorted.sort_unstable();
    let p50 = percentile(&sorted, 50.0);
    let p95 = percentile(&sorted, 95.0);
    let p99 = percentile(&sorted, 99.0);
    let max = sorted.last().copied().unwrap_or(0);
    let secs = r.wall.as_secs_f64();
    let tput = r.frames as f64 / secs;
    let expected_margin = (0..r.frames)
        .filter(|i| plan(*i).0 == Some(Toxic::Margin))
        .count();
    let expected_unauth = (0..r.frames)
        .filter(|i| plan(*i).0 == Some(Toxic::Unauthorized))
        .count();
    let expected_over = (0..r.frames)
        .filter(|i| plan(*i).0 == Some(Toxic::Oversize))
        .count();

    println!("\n=== AF-26 Flash Crash Clearinghouse — {name} ===");
    println!(
        "frames: {} (admitted {}, refused {})",
        r.frames,
        r.admitted,
        r.frames - r.admitted
    );
    println!("wall: {:.3?}  throughput: {:.0} frames/s", r.wall, tput);
    println!(
        "latency ns: p50 {}  p95 {}  p99 {}  max {}",
        p50, p95, p99, max
    );
    println!(
        "refusal census: margin={} (plan {})  unauthorized={} (plan {})  oversize={} (plan {})  header={} (plan 0)",
        r.margin_refusals, expected_margin, r.unauthorized_refusals, expected_unauth,
        r.oversize_refusals, expected_over, r.header_refusals
    );
    println!(
        "alloc census: {} calls / {} bytes across the hot loop (target 0)",
        r.alloc_calls, r.alloc_bytes
    );
    println!(
        "chain: {} links, head {:02x?}…, verify: OK",
        r.links,
        &r.head[..4]
    );

    // --- Gates ---
    // Toxic census must match the deterministic plan exactly.
    assert_eq!(r.margin_refusals, expected_margin, "margin census drift");
    assert_eq!(
        r.unauthorized_refusals, expected_unauth,
        "unauth census drift"
    );
    assert_eq!(r.oversize_refusals, expected_over, "oversize census drift");
    assert_eq!(r.header_refusals, 0, "unexpected codec refusals");
    assert_eq!(
        r.admitted + r.frames / 10,
        r.frames,
        "admit+refuse conservation"
    );
    // Typed refusals never panic: reaching this point IS that gate.
    // Zero-alloc court: the hot loop allocates nothing.
    assert_eq!(
        r.alloc_calls, 0,
        "hot loop allocated {} times",
        r.alloc_calls
    );
    // Chain absorbed exactly the admitted frames.
    assert_eq!(r.links, r.admitted, "chain link conservation");
}

/// Scaled mode: 1M frames, CI-adjacent.
#[test]
fn scaled_1m_flash_crash_soak() {
    let r = run_soak(1_000_000);
    report("scaled 1M frames", &r);
}

/// Full mode: 10M frames (#[ignore]; run with `-- --ignored`).
#[test]
#[ignore]
fn full_10m_flash_crash_soak() {
    let r = run_soak(10_000_000);
    report("FULL 10M frames", &r);
}
