//! RFC-0089 CSCB p99 conformance suite (RX-2) — 5-injection court.
//!
//! Composes LANDED surfaces into the RFC-0089 §4 injection mechanics,
//! extending AF-37's macro benches to the spec's exact parameters:
//!
//! | inj | spec mechanic (§4) | court |
//! |---|---|---|
//! | I1 bimodal deluge | 64B control frames alternating with random 2.1-3.8KB
//!   ragged payloads; >=500k req/sec/core | `rfc0089_conformance_suite` (I1) |
//! | I2 stall-walk | 25% of sinks freeze 200ms/sec cycles; healthy-lane
//!   isolation measured | same (I2) |
//! | I3 cache-stride | randomized non-contiguous frame processing +
//!   background sweeper striding a >16MB buffer | same (I3) |
//! | I4 tamper inundation | 60% corrupted traffic; admit-vs-reject latency
//!   parity, |delta| <= 50ns ABSOLUTE (§6, supersedes AF-37's 1.5x) | same (I4) |
//! | I5 churn | 200 randomized kills across 8x500 fanout per AB-17 | same (I5) |
//!
//! HONEST SCALING (charter): the spec's 100M/50M-frame scale is the
//! `#[ignore]`d full mode (`rfc0089_full_mode --ignored`); the always-on
//! suite runs a scaled proxy and every scorecard row says which it is
//! (MEASURED scaled vs TARGET extrapolated — never rounded up).
//!
//! PROFILE GUARD (RFC §5.2, RX-4's `profile_guard`): every perf-fenced
//! injection loud-SKIPs on a non-release profile with a typed reason
//! (AF-25's two-axis pattern, generalized). Structural courts (I5 churn,
//! tamper typing, KEM seam) run in BOTH profiles.
//!
//! ML-KEM DISCLOSURE (AB-5 in flight): the KEM seam is bound to the same
//! deterministic test-KEM as AF-1's corridor court (dk == ek, Shake256).
//! Rebinding to real ML-KEM-768 is the close condition for the UNSUPPORTED
//! row — see the receipt.
//!
//! TIMING COMPOSITION (disclosed, AF-22 methodology): the timed op for
//! I1/I3/I4 is the PROCESSING path — per frame: expected-tag re-derivation
//! (blake3(header[0..60) || body)[..16], the R-K1 convention) +
//! `verify_window` (every framing/header/tag gate). I4's parity delta is
//! computed over this COMMON verification prefix for both classes — the
//! post-verification admit ledger (gate + chain append) is excluded from
//! BOTH sides equally, so the delta isolates verifier parity (the quantity
//! the 50ns fence governs). I1 additionally reports end-to-end req/s.
//! `Instant::now()` pair overhead (~30-40 ns) rides in every sample.
//! I5 measures kill/respawn recovery, AB-17's quantity.
//!
//! Chicago courts: real UDS streams, real threads, real bytes, zero mocks;
//! counting allocator census per the house pattern (ab8_soak.rs); seeded
//! Xorshift determinism; no Process.sleep — bounded polls on observables
//! (freeze = skip-read window with an Instant deadline, I2). Never runs git.

use std::alloc::{GlobalAlloc, Layout, System};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

use cgap_aoq::chain::{AoqChain, AoqChainRecord, ChainDigest};
use cgap_aoq::codec;
use cgap_aoq::error::AoqError;
use cgap_aoq::header::AoqHeader;
use cgap_aoq::profile_guard;
use cgap_aoq::session::{KemSeam, CT_BYTES, EK_BYTES, SS_BYTES};
use cgap_aoq::verify::{verify_window, BODY_LEN_AT, TAG_BYTES};
use cgap_aoq::writer::AoqWriter;
use cgap_arrow::blake3;
use cgap_crypto::fips202::Shake256;

// ---------------------------------------------------------------------------
// RFC-0089 §4 spec constants
// ---------------------------------------------------------------------------

/// I1: control-frame body (the "64B control frames" class).
const I1_CONTROL_BODY: usize = 64;
/// I1: ragged-payload lower bound (2.1 KB).
const I1_RAGGED_LO: usize = 2_100;
/// I1: ragged-payload upper bound (3.8 KB inclusive span).
const I1_RAGGED_SPAN: usize = 1_700;
/// I1: throughput target, req/sec/core.
const I1_TARGET_RPS_CORE: u64 = 500_000;
/// I1: scaled-mode frame count (always-on suite).
const I1_FRAMES: usize = 200_000;
/// I1: full-mode frame count (spec scale, `#[ignore]`).
const I1_FULL_FRAMES: usize = 100_000_000;

/// I2: lanes in the stall-walk mesh.
const I2_LANES: usize = 4;
/// I2: frames per lane (scaled mode).
const I2_FRAMES_PER_LANE: usize = 4_000;
/// I2: 25% of sinks freeze.
const I2_STALLED_LANES: usize = 1;
/// I2: processing slice between freeze windows (frames).
const I2_SLICE: usize = 512;
/// I2: freeze window width — the skip-read window, in producer frames.
const I2_FREEZE_FRAMES: usize = 256;
/// I2: freeze deadline cap (bounded stall injection; never a sleep).
const I2_FREEZE_CAP_MS: u64 = 10;

/// I3: frames in the stride slab (scaled mode).
const I3_FRAMES: usize = 50_000;
/// I3: sweeper buffer — strictly greater than the spec's 16MB.
const I3_SWEEPER_BYTES: usize = 24 * 1024 * 1024;
/// I3: body size in the slab (64B control class).
const I3_BODY: usize = 64;

/// I4: frames under tamper inundation (scaled mode).
const I4_FRAMES: usize = 20_000;
/// I4: corrupted-traffic fraction (60%).
const I4_CORRUPT_PCT: u64 = 60;
/// I4: admit-vs-reject parity fence — ABSOLUTE nanoseconds (RFC §6;
/// supersedes AF-37's 1.5x ratio).
const I4_PARITY_FENCE_NS: u32 = 50;

/// I5: lanes in the churn fanout.
const I5_LANES: usize = 8;
/// I5: frames per lane.
const I5_FRAMES_PER_LANE: usize = 500;
/// I5: randomized kills across the fanout.
const I5_KILLS: usize = 200;

/// Shared lane law.
const EPOCH: u16 = 1;
const KEY_ID: u64 = 0x005E_C1D;
const INSTRUMENTS: usize = 10_000;
/// Largest body the suite builds (I1 ragged cap).
const BODY_CAP: usize = 4_096;
/// Window prefix + metadata span (body starts at 128; EOS after pad).
const BODY_AT: usize = 8 + 120;
/// Largest window: 136 + body + alignment-64 pad.
const WIN_MAX: usize = 136 + BODY_CAP + ((BODY_CAP + 63) & !63);

/// Scorecard classification (RFC §6, per RX-1's annotation scheme).
/// MEASURED = measured this run against the target; TARGET = spec target
/// asserted but only a scaled proxy measured (labeled extrapolation);
/// UNSUPPORTED = surface not admitted this run (typed reason).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Class {
    Meets,
    Unmet,
    TargetExtrapolated,
    Unsupported,
    Skipped,
}

impl Class {
    const fn word(self) -> &'static str {
        match self {
            Self::Meets => "MEASURED/MEETS",
            Self::Unmet => "MEASURED/UNMET",
            Self::TargetExtrapolated => "TARGET/extrapolated",
            Self::Unsupported => "UNSUPPORTED",
            Self::Skipped => "SKIP",
        }
    }
}

/// One scorecard row (printed and hashed into the G7 report digest).
struct Row {
    gate: &'static str,
    target: String,
    measured: String,
    class: Class,
}

// ---------------------------------------------------------------------------
// Counting allocator (alloc census; court instrumentation, ab8_soak pattern)
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
    unsafe fn dealloc(&self, _p: *mut u8, _l: Layout) {
        // deallocation is not censused (house pattern: alloc-side only)
    }
}

fn alloc_snapshot() -> (usize, usize) {
    (
        ALLOC_CALLS.load(Ordering::Relaxed),
        ALLOC_BYTES.load(Ordering::Relaxed),
    )
}

fn alloc_delta((c0, b0): (usize, usize)) -> (usize, usize) {
    (
        ALLOC_CALLS.load(Ordering::Relaxed) - c0,
        ALLOC_BYTES.load(Ordering::Relaxed) - b0,
    )
}

// ---------------------------------------------------------------------------
// Determinism + percentiles
// ---------------------------------------------------------------------------

struct Xorshift(u64);

impl Xorshift {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

fn percentile(sorted: &[u32], p: f64) -> u32 {
    let n = sorted.len();
    if n == 0 {
        return 0;
    }
    let idx = ((p / 100.0) * (n as f64 - 1.0)).round() as usize;
    sorted[idx.min(n - 1)]
}

/// Sort, print a distribution line, return (p50, p99).
fn dist_line(name: &str, samples: &mut Vec<u32>) -> (u32, u32) {
    samples.sort_unstable();
    let p50 = percentile(samples, 50.0);
    let p99 = percentile(samples, 99.0);
    println!(
        "  {:<30} n={:>7}  p50 {:>7}  p99 {:>8}  max {:>9} ns",
        name,
        samples.len(),
        p50,
        p99,
        samples.last().copied().unwrap_or(0)
    );
    (p50, p99)
}

fn ns(d: std::time::Duration) -> u32 {
    (d.as_secs() * 1_000_000_000 + u64::from(d.subsec_nanos())) as u32
}

// ---------------------------------------------------------------------------
// Frame construction + the shared processing op (AB-8 writer, R-K1 tags)
// ---------------------------------------------------------------------------

/// Encode one lawful frame window into `out`; returns the window length.
/// Tag per the R-K1 convention: blake3(header[0x00..0x3C) || body)[..16].
fn build_frame(
    writer: &mut AoqWriter,
    out: &mut [u8; WIN_MAX],
    tag_scratch: &mut [u8; 60 + BODY_CAP],
    body: &[u8],
) -> usize {
    assert!(body.len() <= BODY_CAP, "bench body cap");
    let (ordinal, seq) = writer.cursor();
    let h = AoqHeader::new(EPOCH, KEY_ID, ordinal, seq);
    let mut hb = [0u8; 64];
    codec::encode(&h, &mut hb).expect("bench: lawful header");
    tag_scratch[..60].copy_from_slice(&hb[..60]);
    tag_scratch[60..60 + body.len()].copy_from_slice(body);
    let tag: [u8; TAG_BYTES] = blake3::hash(&tag_scratch[..60 + body.len()])[..TAG_BYTES]
        .try_into()
        .expect("blake3 32B");
    writer
        .write_window(out, EPOCH, KEY_ID, &tag, body)
        .expect("bench: lawful frame")
}

/// Consumer-side expected-tag re-derivation (R-K1): the caller has read
/// the full window into `win[0..win_len]`; body_len is peeked from the pad.
fn expected_tag(win: &[u8], bl: usize, scratch: &mut [u8; 60 + BODY_CAP]) -> [u8; TAG_BYTES] {
    scratch[..60].copy_from_slice(&win[8..68]);
    scratch[60..60 + bl].copy_from_slice(&win[BODY_AT..BODY_AT + bl]);
    blake3::hash(&scratch[..60 + bl])[..TAG_BYTES].try_into().expect("blake3 32B")
}

/// Per-frame risk gate (the AF-26 gate shape: key-id law + u64 wrapping
/// margin ledger, fixed 10k slots, zero-alloc).
struct Gate {
    ledger: Box<[u64; INSTRUMENTS]>,
}

impl Gate {
    fn new() -> Self {
        Self { ledger: Box::new([1_000u64; INSTRUMENTS]) }
    }
    /// Ok(()) = admit; Err(code) = typed refusal class.
    fn admit(&mut self, h: &AoqHeader, body: &[u8]) -> Result<(), u16> {
        if h.key_id != KEY_ID {
            return Err(0x10_02);
        }
        let instrument = (h.seq % INSTRUMENTS as u32) as usize;
        let mut nb = [0u8; 8];
        nb.copy_from_slice(&body[..8]);
        let notional = u64::from_le_bytes(nb);
        let slot = &mut self.ledger[instrument];
        let new = slot.wrapping_add(notional);
        if new < *slot {
            return Err(0x10_01);
        }
        *slot = new;
        Ok(())
    }
}

/// Real cgap-arrow blake3 as the chain digest seam (ab-12 substrate).
struct Blake3Digest;

impl ChainDigest for Blake3Digest {
    fn digest(&self, parts: &[&[u8]]) -> [u8; 32] {
        let mut acc = blake3::hash(parts[0]);
        for p in &parts[1..] {
            let next = blake3::hash(p);
            let mut cat = [0u8; 64];
            cat[..32].copy_from_slice(&acc);
            cat[32..].copy_from_slice(&next);
            acc = blake3::hash(&cat);
        }
        acc
    }
}

/// Leak a chain slab (AB-19 preallocation pattern; 'static chain).
fn chain_slab(n: usize) -> &'static mut [AoqChainRecord] {
    Box::leak(vec![AoqChainRecord::zeroed(); n].into_boxed_slice())
}

/// Read one full variable-length window from the stream into `buf`
/// (prefix+meta first, body_len peek, remainder). Returns window length.
fn read_window(rx: &mut UnixStream, buf: &mut [u8; WIN_MAX]) -> usize {
    rx.read_exact(&mut buf[..BODY_AT]).expect("bench read meta");
    let bl = u32::from_le_bytes(buf[BODY_LEN_AT..BODY_LEN_AT + 4].try_into().expect("4B")) as usize;
    assert!(bl <= BODY_CAP, "bench body cap on wire");
    let rest = bl + ((bl + 63) & !63) + 8;
    rx.read_exact(&mut buf[BODY_AT..BODY_AT + rest]).expect("bench read body");
    BODY_AT + rest
}

/// The COMMON verification prefix both I4 classes time: expected-tag
/// re-derivation + verify_window (every framing/header/tag gate).
fn verify_prefix(
    win: &[u8],
    bl: usize,
    ord: u32,
    seq: u32,
    scratch: &mut [u8; 60 + BODY_CAP],
) -> Result<(), AoqError> {
    let tag = expected_tag(win, bl, scratch);
    let vw = verify_window(win, ord, seq, &tag)?;
    profile_guard::fence(vw.window_len());
    Ok(())
}

// ---------------------------------------------------------------------------
// I1 — bimodal deluge (64B control alternating with 2.1-3.8KB ragged)
// ---------------------------------------------------------------------------

/// Runs I1 at `frames` scale over a real UDS pair. Returns (achieved
/// req/sec for the single-producer/single-consumer pair, processing p50,
/// processing p99, consumer alloc delta).
fn run_i1(frames: usize) -> (u64, u32, u32, (usize, usize)) {
    let (mut tx, mut rx) = UnixStream::pair().expect("socketpair");
    let producer = thread::spawn(move || {
        let mut writer = AoqWriter::new();
        let mut rng = Xorshift(0x0089_00i64 as u64 | (1 << 32));
        let mut out = [0u8; WIN_MAX];
        let mut scratch = [0u8; 60 + BODY_CAP];
        for i in 0..frames {
            let mut body = [0u8; BODY_CAP];
            let bl = if i % 2 == 0 {
                I1_CONTROL_BODY // 64B control frame
            } else {
                I1_RAGGED_LO + (rng.next() as usize % I1_RAGGED_SPAN) // 2.1-3.8KB ragged
            };
            for b in body[..bl].iter_mut() {
                *b = rng.next() as u8;
            }
            body[..8].copy_from_slice(&(rng.next() % 1_000).to_le_bytes());
            let wl = build_frame(&mut writer, &mut out, &mut scratch, &body[..bl]);
            tx.write_all(&out[..wl]).expect("producer write");
        }
    });
    let mut lat: Vec<u32> = Vec::with_capacity(frames);
    let mut win = [0u8; WIN_MAX];
    let mut scratch = [0u8; 60 + BODY_CAP];
    let alloc0 = alloc_snapshot();
    let wall0 = Instant::now();
    for i in 0..frames {
        let wl = read_window(&mut rx, &mut win);
        let bl = u32::from_le_bytes(win[BODY_LEN_AT..BODY_LEN_AT + 4].try_into().expect("4B")) as usize;
        let t0 = Instant::now();
        let r = verify_prefix(&win[..wl], bl, i as u32, i as u32, &mut scratch);
        lat.push(ns(t0.elapsed()));
        r.expect("I1 frame lawful");
    }
    let wall = wall0.elapsed();
    producer.join().expect("producer thread");
    let alloc = alloc_delta(alloc0);
    let (p50, p99) = dist_line("I1 processing latency", &mut lat);
    let rps = (frames as f64 / wall.as_secs_f64()) as u64;
    println!("  I1 pair req/s (end-to-end wall): {rps}");
    (rps, p50, p99, alloc)
}

// ---------------------------------------------------------------------------
// I2 — stall-walk (25% of sinks freeze; skip-read window, never a sleep)
// ---------------------------------------------------------------------------

/// Freeze = the consumer STOPS READING for `I2_FREEZE_FRAMES` producer
/// frames (bounded poll on the produced-counter observable, capped by an
/// Instant deadline). No Process.sleep anywhere.
fn freeze_skip_read(produced: &AtomicUsize, skip: usize, deadline: Instant) {
    let start = produced.load(Ordering::Relaxed);
    let target = start.saturating_add(skip);
    while produced.load(Ordering::Relaxed) < target && Instant::now() < deadline {
        std::thread::yield_now();
    }
}

/// One lane: producer streams `frames` lawful frames over a real UDS pair;
/// `stalled` consumers freeze every 4th slice (200ms/sec cycle, scaled to
/// frames). Processing samples are timed on the consumer. When `collect`
/// is set the samples are returned (healthy lanes); stalled-lane samples
/// are discarded (the stall lane is not the measured subject).
fn run_i2_lane(
    lane: usize,
    frames: usize,
    stalled: bool,
    collect: bool,
) -> thread::JoinHandle<(Vec<u32>, usize)> {
    thread::spawn(move || {
        let (mut tx, mut rx) = UnixStream::pair().expect("socketpair");
        let produced = Arc::new(AtomicUsize::new(0));
        let produced_w = Arc::clone(&produced);
        let writer_thread = thread::spawn(move || {
            let mut writer = AoqWriter::new();
            let mut rng = Xorshift(0x0089_00A0 + lane as u64);
            let mut out = [0u8; WIN_MAX];
            let mut scratch = [0u8; 60 + BODY_CAP];
            for i in 0..frames {
                let mut body = [0u8; I1_CONTROL_BODY];
                for b in body.iter_mut() {
                    *b = rng.next() as u8;
                }
                body[..8].copy_from_slice(&(rng.next() % 1_000).to_le_bytes());
                let wl = build_frame(&mut writer, &mut out, &mut scratch, &body);
                tx.write_all(&out[..wl]).expect("I2 producer write");
                produced_w.store(i + 1, Ordering::Relaxed);
            }
        });
        let mut lat: Vec<u32> = Vec::new();
        let mut win = [0u8; WIN_MAX];
        let mut scratch = [0u8; 60 + BODY_CAP];
        let mut freezes = 0usize;
        for i in 0..frames {
            if stalled && i % I2_SLICE == 0 && i > 0 {
                freeze_skip_read(
                    &produced,
                    I2_FREEZE_FRAMES,
                    Instant::now() + std::time::Duration::from_millis(I2_FREEZE_CAP_MS),
                );
                freezes += 1;
            }
            let wl = read_window(&mut rx, &mut win);
            let bl = u32::from_le_bytes(
                win[BODY_LEN_AT..BODY_LEN_AT + 4].try_into().expect("4B"),
            ) as usize;
            let t0 = Instant::now();
            let r = verify_prefix(&win[..wl], bl, i as u32, i as u32, &mut scratch);
            lat.push(ns(t0.elapsed()));
            r.expect("I2 frame lawful");
        }
        writer_thread.join().expect("I2 producer join");
        if collect {
            (lat, freezes)
        } else {
            (Vec::new(), freezes)
        }
    })
}

// ---------------------------------------------------------------------------
// I3 — cache-stride (randomized non-contiguous processing + 24MB sweeper)
// ---------------------------------------------------------------------------

/// Prebuilds `frames` lawful windows into one slab; returns the slab,
/// per-frame offsets, and per-frame body lengths.
fn build_i3_slab(frames: usize) -> (std::sync::Arc<Vec<u8>>, Vec<usize>, Vec<usize>) {
    let mut writer = AoqWriter::new();
    let mut rng = Xorshift(0x0089_00B0);
    let mut out = [0u8; WIN_MAX];
    let mut scratch = [0u8; 60 + BODY_CAP];
    let mut slab: Vec<u8> = Vec::with_capacity(frames * 264);
    let mut offs = Vec::with_capacity(frames);
    let mut blens = Vec::with_capacity(frames);
    for _ in 0..frames {
        let mut body = [0u8; I3_BODY];
        for b in body.iter_mut() {
            *b = rng.next() as u8;
        }
        body[..8].copy_from_slice(&(rng.next() % 1_000).to_le_bytes());
        let wl = build_frame(&mut writer, &mut out, &mut scratch, &body);
        offs.push(slab.len());
        slab.extend_from_slice(&out[..wl]);
        blens.push(I3_BODY);
    }
    (std::sync::Arc::new(slab), offs, blens)
}

/// The background sweeper: strides a >16MB buffer cache-line-by-cache-line
/// until `done` flips (bounded poll on the observable; yield between passes).
fn sweeper(buf: Vec<u8>, done: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut sink = 0u8;
        while !done.load(Ordering::Relaxed) {
            let mut i = 0;
            while i < buf.len() {
                sink = sink.wrapping_add(buf[i]);
                i += 64; // one cache line per step
            }
            std::thread::yield_now();
        }
        profile_guard::fence(sink);
    })
}

/// Fisher-Yates shuffle (seeded) of 0..n — the non-contiguous order.
fn shuffled(n: usize, seed: u64) -> Vec<usize> {
    let mut rng = Xorshift(seed);
    let mut v: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (rng.next() as usize) % (i + 1);
        v.swap(i, j);
    }
    v
}

// ---------------------------------------------------------------------------
// I5 — churn (200 randomized kills across the 8x500 fanout, AB-17 pattern)
// ---------------------------------------------------------------------------

/// One churn worker cycle: processes frames [from, until) — building each
/// frame with its own writer, verifying it, appending to the lane chain —
/// and stops at the cycle's kill point if one is assigned (kill happens
/// mid-fanout, at the frame boundary, per the landed AB-17/AF-32 pattern).
fn churn_cycle(
    lane: usize,
    from: usize,
    until: usize,
    kill_at: Option<usize>,
    chain: &Mutex<AoqChain<'static, Blake3Digest>>,
    delivered: &Mutex<Vec<bool>>,
) -> usize {
    let mut writer = AoqWriter::new();
    let mut rng = Xorshift(0x0089_00C0 + lane as u64);
    let mut out = [0u8; WIN_MAX];
    let mut scratch = [0u8; 60 + BODY_CAP];
    let mut win = [0u8; WIN_MAX];
    let mut done = from;
    for i in from..until {
        if Some(i) == kill_at {
            break; // churn kill: mid-stream, at the frame boundary
        }
        let mut body = [0u8; I1_CONTROL_BODY];
        for b in body.iter_mut() {
            *b = rng.next() as u8;
        }
        body[..8].copy_from_slice(&(rng.next() % 1_000).to_le_bytes());
        let wl = build_frame(&mut writer, &mut out, &mut scratch, &body);
        // Wire-law round trip within the cycle window: verify the just-
        // written window at the writer's pre-commit cursor.
        let (ord, seq) = writer.cursor();
        let bl = u32::from_le_bytes(
            out[BODY_LEN_AT..BODY_LEN_AT + 4].try_into().expect("4B"),
        ) as usize;
        let mut tag_scratch = [0u8; 60 + BODY_CAP];
        verify_prefix(&out[..wl], bl, ord, seq, &mut tag_scratch).expect("I5 frame lawful");
        let mut chain = chain.lock().expect("chain lock");
        chain.append(EPOCH, &out[BODY_AT..BODY_AT + bl]).expect("chain append");
        drop(chain);
        let mut d = delivered.lock().expect("delivered lock");
        assert!(!d[lane * I5_FRAMES_PER_LANE + i], "I5: frame delivered twice");
        d[lane * I5_FRAMES_PER_LANE + i] = true;
        drop(d);
        // Window reuse: copy into the consumer-side buffer to mirror the
        // real read path (kill mid-cycle may abandon this buffer's tail).
        win[..wl].copy_from_slice(&out[..wl]);
        profile_guard::fence(win[0]);
        done = i + 1;
    }
    done
}

/// Drives one lane's churn: repeated cycles with seeded randomized kill
/// points, respawning until the lane's 500 frames are all delivered.
/// Returns (executed kills, mean cycle ns).
fn churn_lane(
    lane: usize,
    kill_points: &[usize],
    chain: &Mutex<AoqChain<'static, Blake3Digest>>,
    delivered: &Mutex<Vec<bool>>,
) -> (usize, u128) {
    let mut from = 0usize;
    let mut respawns = 0usize;
    let mut kills_done = 0usize;
    let mut respawn_ns_total: u128 = 0;
    let mut ki = 0usize;
    while from < I5_FRAMES_PER_LANE {
        let kill_at = if ki < kill_points.len() && kill_points[ki] > from {
            ki += 1;
            Some(kill_points[ki - 1])
        } else {
            if ki < kill_points.len() && kill_points[ki] <= from {
                ki += 1; // stale kill point (already-resumed cursor): skip
            }
            None
        };
        let t0 = Instant::now();
        let done = churn_cycle(lane, from, I5_FRAMES_PER_LANE, kill_at, chain, delivered);
        respawn_ns_total += u128::from(t0.elapsed().as_nanos());
        respawns += 1;
        let killed = kill_at.is_some() && done < I5_FRAMES_PER_LANE && done == kill_at.unwrap();
        if killed {
            kills_done += 1;
        } else {
            // sanity: a non-killed cycle must have run to the lane end
            assert_eq!(done, I5_FRAMES_PER_LANE, "I5 lane stall");
        }
        from = done;
    }
    (kills_done, respawn_ns_total / respawns.max(1) as u128)
}

// ---------------------------------------------------------------------------
// The conformance suite: I1-I5 in sequence, then the G1-G7 scorecard
// ---------------------------------------------------------------------------

#[test]
fn rfc0089_conformance_suite() {
    let release = profile_guard::assert_release().is_ok();
    let mut rows: Vec<Row> = Vec::new();

    // ---- I1 bimodal deluge (release-fenced; loud SKIP in debug) ----------
    if release {
        let (rps, p50, p99, alloc) = run_i1(I1_FRAMES);
        let class = if rps >= I1_TARGET_RPS_CORE { Class::Meets } else { Class::Unmet };
        rows.push(Row {
            gate: "G1 I1 throughput (scaled 200k-frame proxy)",
            target: format!(">={} req/s/pair", I1_TARGET_RPS_CORE),
            measured: format!("{rps} req/s (p50 {p50} ns, p99 {p99} ns)"),
            class,
        });
        // G6: the consumer pipeline must be zero-alloc under the deluge.
        let class = if alloc == (0, 0) { Class::Meets } else { Class::Unmet };
        rows.push(Row {
            gate: "G6 I1 consumer alloc census",
            target: "0 allocs / 0 bytes".into(),
            measured: format!("{:?} (calls, bytes)", alloc),
            class,
        });
    } else {
        println!("I1 SKIP: {}", profile_guard::assert_release().unwrap_err());
        rows.push(Row {
            gate: "G1 I1 throughput",
            target: format!(">={} req/s/pair", I1_TARGET_RPS_CORE),
            measured: "not measured (non-release profile)".into(),
            class: Class::Skipped,
        });
    }

    // ---- I2 stall-walk (release-fenced) ----------------------------------
    if release {
        // Baseline: same lane shape, no stalls.
        let (base, _) = run_i2_lane(9, I2_FRAMES_PER_LANE, false, true)
            .join()
            .expect("baseline lane");
        let mut base = base;
        let (b50, b99) = dist_line("I2 baseline (no stall)", &mut base);
        // Mesh: 4 lanes, 25% stalled (lane 2), healthy lanes measured.
        let handles: Vec<_> = (0..I2_LANES)
            .map(|lane| {
                let stalled = lane == I2_STALLED_LANES; // exactly 25% of sinks
                run_i2_lane(lane, I2_FRAMES_PER_LANE, stalled, lane != I2_STALLED_LANES)
            })
            .collect();
        let mut healthy: Vec<u32> = Vec::new();
        let mut freezes = 0usize;
        for h in handles {
            let (lat, fz) = h.join().expect("I2 lane");
            freezes += fz;
            healthy.extend(lat);
        }
        let (h50, h99) = dist_line("I2 healthy lanes under stall", &mut healthy);
        let ratio = f64::from(h99) / f64::from(b99.max(1));
        println!("  I2 freeze windows executed: {freezes}");
        let class = if ratio <= 1.5 { Class::Meets } else { Class::Unmet };
        rows.push(Row {
            gate: "G2 I2 healthy-lane isolation",
            target: "healthy p99 <= 1.5x baseline p99".into(),
            measured: format!(
                "baseline p50 {b50} p99 {b99} ns; healthy p50 {h50} p99 {h99} ns; ratio {ratio:.3}x"
            ),
            class,
        });
    } else {
        println!("I2 SKIP: {}", profile_guard::assert_release().unwrap_err());
        rows.push(Row {
            gate: "G2 I2 healthy-lane isolation",
            target: "healthy p99 <= 1.5x baseline p99".into(),
            measured: "not measured (non-release profile)".into(),
            class: Class::Skipped,
        });
    }

    // ---- I3 cache-stride (release-fenced) --------------------------------
    if release {
        let (slab, offs, blens) = build_i3_slab(I3_FRAMES);
        // Baseline: contiguous order.
        let mut base: Vec<u32> = Vec::with_capacity(I3_FRAMES);
        let mut scratch = [0u8; 60 + BODY_CAP];
        for (i, (&off, &bl)) in offs.iter().zip(&blens).enumerate() {
            let win = &slab[off..off + 264];
            let t0 = Instant::now();
            verify_prefix(win, bl, i as u32, i as u32, &mut scratch).expect("I3 baseline lawful");
            base.push(ns(t0.elapsed()));
        }
        let (_b50, b99) = dist_line("I3 contiguous decode", &mut base);
        // Sweeper + shuffled order.
        let done = Arc::new(AtomicBool::new(false));
        let sw = sweeper(vec![0u8; I3_SWEEPER_BYTES], Arc::clone(&done));
        let mut stride: Vec<u32> = Vec::with_capacity(I3_FRAMES);
        for (k, &fi) in shuffled(I3_FRAMES, 0x0089_00D0).iter().enumerate() {
            let off = offs[fi];
            let bl = blens[fi];
            let t0 = Instant::now();
            verify_prefix(&slab[off..off + 264], bl, fi as u32, fi as u32, &mut scratch)
                .expect("I3 strided lawful");
            stride.push(ns(t0.elapsed()));
            let _ = k;
        }
        done.store(true, Ordering::Relaxed);
        sw.join().expect("sweeper join");
        let (_s50, s99) = dist_line("I3 strided decode under sweeper", &mut stride);
        let ratio = f64::from(s99) / f64::from(b99.max(1));
        rows.push(Row {
            gate: "G3 I3 cache-stride decode inflation",
            target: "strided p99 <= 2x contiguous p99 (AF-37 ref: 450 ns decode p99)".into(),
            measured: format!("contiguous p99 {b99} ns; strided p99 {s99} ns; ratio {ratio:.3}x"),
            class: if ratio <= 2.0 { Class::Meets } else { Class::Unmet },
        });
    } else {
        println!("I3 SKIP: {}", profile_guard::assert_release().unwrap_err());
        rows.push(Row {
            gate: "G3 I3 cache-stride decode inflation",
            target: "strided p99 <= 2x contiguous p99".into(),
            measured: "not measured (non-release profile)".into(),
            class: Class::Skipped,
        });
    }

    // ---- I4 tamper inundation (release-fenced; 60% corrupt, 50ns fence) --
    if release {
        let (mut tx, mut rx) = UnixStream::pair().expect("socketpair");
        let corrupt_count = Arc::new(AtomicUsize::new(0));
        let cc = Arc::clone(&corrupt_count);
        let producer = thread::spawn(move || {
            let mut writer = AoqWriter::new();
            let mut rng = Xorshift(0x0089_00E0);
            let mut out = [0u8; WIN_MAX];
            let mut scratch = [0u8; 60 + BODY_CAP];
            for _ in 0..I4_FRAMES {
                let mut body = [0u8; I1_CONTROL_BODY];
                for b in body.iter_mut() {
                    *b = rng.next() as u8;
                }
                body[..8].copy_from_slice(&(rng.next() % 1_000).to_le_bytes());
                let wl = build_frame(&mut writer, &mut out, &mut scratch, &body);
                if rng.next() % 100 < I4_CORRUPT_PCT {
                    out[76] ^= 0x40; // tamper: flip one tag byte in the window
                    cc.fetch_add(1, Ordering::Relaxed);
                }
                tx.write_all(&out[..wl]).expect("I4 producer write");
            }
        });
        let mut admit: Vec<u32> = Vec::new();
        let mut reject: Vec<u32> = Vec::new();
        let mut win = [0u8; WIN_MAX];
        let mut scratch = [0u8; 60 + BODY_CAP];
        for i in 0..I4_FRAMES {
            let wl = read_window(&mut rx, &mut win);
            let bl = u32::from_le_bytes(
                win[BODY_LEN_AT..BODY_LEN_AT + 4].try_into().expect("4B"),
            ) as usize;
            let t0 = Instant::now();
            let r = verify_prefix(&win[..wl], bl, i as u32, i as u32, &mut scratch);
            let d = ns(t0.elapsed());
            match r {
                Ok(()) => admit.push(d),
                Err(AoqError::DigestMismatch { .. }) => reject.push(d),
                Err(other) => panic!("I4: corruption refused off-class: {other:?}"),
            }
        }
        producer.join().expect("I4 producer");
        let rejected = corrupt_count.load(Ordering::Relaxed);
        assert_eq!(admit.len() + reject.len(), I4_FRAMES, "I4 accounting");
        assert_eq!(reject.len(), rejected, "I4: every tampered frame refused");
        let (a50, a99) = dist_line("I4 admit latency", &mut admit);
        let (r50, r99) = dist_line("I4 reject latency", &mut reject);
        let delta = (i64::from(a50) - i64::from(r50)).unsigned_abs() as u32;
        let class = if delta <= I4_PARITY_FENCE_NS { Class::Meets } else { Class::Unmet };
        rows.push(Row {
            gate: "G4 I4 tamper parity (absolute fence, RFC §6)",
            target: format!("|admit p50 - reject p50| <= {} ns", I4_PARITY_FENCE_NS),
            measured: format!(
                "admit p50 {a50}/p99 {a99} ns; reject p50 {r50}/p99 {r99} ns; delta {delta} ns; \
                 corrupt {rejected}/{I4_FRAMES}"
            ),
            class,
        });
    } else {
        println!("I4 SKIP: {}", profile_guard::assert_release().unwrap_err());
        rows.push(Row {
            gate: "G4 I4 tamper parity",
            target: format!("|admit p50 - reject p50| <= {} ns", I4_PARITY_FENCE_NS),
            measured: "not measured (non-release profile)".into(),
            class: Class::Skipped,
        });
    }

    // ---- I5 churn (structural: runs in BOTH profiles) --------------------
    {
        let kill_points: Vec<Vec<usize>> = {
            let mut rng = Xorshift(0x0089_00F0);
            let mut per_lane: Vec<Vec<usize>> = vec![Vec::new(); I5_LANES];
            for _ in 0..I5_KILLS {
                let lane = (rng.next() as usize) % I5_LANES;
                let frame = 1 + (rng.next() as usize) % (I5_FRAMES_PER_LANE - 1);
                per_lane[lane].push(frame);
            }
            per_lane.iter_mut().for_each(|v| v.sort_unstable());
            per_lane
        };
        let unique_kills: usize = kill_points
            .iter()
            .map(|v| v.len() - v.windows(2).filter(|w| w[0] == w[1]).count())
            .sum();
        let delivered = Arc::new(Mutex::new(vec![false; I5_LANES * I5_FRAMES_PER_LANE]));
        let mut handles = Vec::new();
        for lane in 0..I5_LANES {
            let slab = chain_slab(I5_FRAMES_PER_LANE + 1);
            let chain = Arc::new(Mutex::new(AoqChain::new(Blake3Digest, slab)));
            let delivered = Arc::clone(&delivered);
            let kps = kill_points[lane].clone();
            handles.push(thread::spawn(move || {
                let (kills, mean_cycle_ns) = churn_lane(lane, &kps, &chain, &delivered);
                let chain = Arc::try_unwrap(chain)
                    .ok()
                    .and_then(|m| m.into_inner().ok())
                    .expect("I5: sole chain owner at join");
                (kills, mean_cycle_ns, chain)
            }));
        }
        let mut total_kills = 0usize;
        for h in handles {
            let (kills, mean_cycle_ns, mut chain) = h.join().expect("I5 lane join");
            total_kills += kills;
            println!("  I5 lane: {kills} kills, mean cycle {mean_cycle_ns} ns");
            // Post-churn chain integrity: append-only survival across kills.
            chain.verify_all().expect("I5: lane chain spoliation");
        }
        assert_eq!(
            total_kills, unique_kills,
            "I5: every seeded kill point must execute exactly once"
        );
        println!("  I5 kills executed: {total_kills}/{unique_kills}");
        let d = delivered.lock().expect("delivered lock");
        let ok = d.iter().all(|&b| b);
        assert!(ok, "I5: every frame in the 8x500 fanout delivered exactly once");
        let class = if ok { Class::Meets } else { Class::Unmet };
        rows.push(Row {
            gate: "G5 I5 churn zero-loss",
            target: format!(
                "{} kills / {}x{} fanout, zero loss or duplication",
                I5_KILLS, I5_LANES, I5_FRAMES_PER_LANE
            ),
            measured: format!("all {} frames delivered once; {} respawns", I5_LANES * I5_FRAMES_PER_LANE, total_kills),
            class,
        });
    }

    // ---- Spec-scale rows (honest scaling; never rounded up) --------------
    rows.push(Row {
        gate: "G1x I1 spec scale (100M frames, full mode)",
        target: format!(">={} req/s/core at 100M frames", I1_TARGET_RPS_CORE),
        measured: format!(
            "scaled proxy only this run ({} frames); full mode: cargo test --test \
             rfc0089_conformance rfc0089_full_mode -- --ignored",
            I1_FRAMES
        ),
        class: Class::TargetExtrapolated,
    });
    rows.push(Row {
        gate: "G4x I4 spec scale (50M tampered frames, full mode)",
        target: "|delta| <= 50 ns at 50M frames".into(),
        measured: "scaled proxy only this run (20k frames)".into(),
        class: Class::TargetExtrapolated,
    });
    rows.push(Row {
        gate: "G8 ML-KEM-768 real kernel",
        target: "real FIPS 203 kernel on the session KEM seam".into(),
        measured: "test-KEM fallback (dk==ek, Shake256) — AB-5 in flight; \
                   disclosed in module header + receipt"
            .into(),
        class: Class::Unsupported,
    });

    // ---- Scorecard print + G7 report integrity (sovereign BLAKE3) --------
    println!("\n  === RFC-0089 §6 scorecard (G1-G7 + annotations) ===");
    let mut report = String::new();
    for r in &rows {
        let line = format!(
            "| {:<46} | {:<42} | {:<52} | {} |\n",
            r.gate,
            r.target,
            r.measured,
            r.class.word()
        );
        println!("{}", line.trim_end());
        report.push_str(&line);
    }
    let digest = cgap_blake3::hash(report.as_bytes());
    println!("  G7 report digest (cgap-blake3): {}", hex(&digest));
    rows.push(Row {
        gate: "G7 report integrity",
        target: "verdict report hashed with the sovereign BLAKE3".into(),
        measured: format!("digest {}", hex(&digest)),
        class: Class::Meets,
    });
    assert!(
        rows.iter().filter(|r| r.class == Class::Skipped).count() == 0 || !release,
        "release run must not carry SKIP rows"
    );
    println!("  scorecard rows: {} (release={release})", rows.len());
}

/// Lowercase hex for a 32-byte digest.
fn hex(d: &[u8; 32]) -> String {
    d.iter().map(|b| format!("{b:02x}")).collect()
}

/// Spec-scale full mode: I1 at the RFC's 100M-frame scale. `#[ignore]`d —
/// run with `cargo test -p cgap-aoq --test rfc0089_conformance \
/// rfc0089_full_mode --release -- --ignored`. Extrapolation labeled.
#[test]
#[ignore = "spec scale (100M frames); always-on suite is the scaled proxy"]
fn rfc0089_full_mode() {
    if profile_guard::assert_release().is_err() {
        panic!("SKIP full mode: release profile required");
    }
    println!("I1 FULL MODE: {} frames (spec scale)", I1_FULL_FRAMES);
    let (rps, p50, p99, alloc) = run_i1(I1_FULL_FRAMES);
    let class = if rps >= I1_TARGET_RPS_CORE { "MEASURED/MEETS" } else { "MEASURED/UNMET" };
    println!(
        "G1 full-mode row: {rps} req/s vs target {} — {class} (p50 {p50} p99 {p99} ns, alloc {alloc:?})",
        I1_TARGET_RPS_CORE
    );
}

// ---------------------------------------------------------------------------
// Structural court — tamper refusal typing (both profiles; anti-vacuity:
// counts must match the injected corruption exactly)
// ---------------------------------------------------------------------------

#[test]
fn rfc0089_tamper_refusal_typing() {
    let frames = 600;
    let (mut tx, mut rx) = UnixStream::pair().expect("socketpair");
    let corrupt_count = Arc::new(AtomicUsize::new(0));
    let cc = Arc::clone(&corrupt_count);
    let producer = thread::spawn(move || {
        let mut writer = AoqWriter::new();
        let mut rng = Xorshift(0x0089_1000);
        let mut out = [0u8; WIN_MAX];
        let mut scratch = [0u8; 60 + BODY_CAP];
        for _ in 0..frames {
            let mut body = [0u8; I1_CONTROL_BODY];
            for b in body.iter_mut() {
                *b = rng.next() as u8;
            }
            body[..8].copy_from_slice(&(rng.next() % 1_000).to_le_bytes());
            let wl = build_frame(&mut writer, &mut out, &mut scratch, &body);
            if rng.next() % 100 < I4_CORRUPT_PCT {
                out[76] ^= 0x40; // tag-byte tamper
                cc.fetch_add(1, Ordering::Relaxed);
            }
            tx.write_all(&out[..wl]).expect("typing producer write");
        }
    });
    let mut win = [0u8; WIN_MAX];
    let mut scratch = [0u8; 60 + BODY_CAP];
    let (mut admits, mut rejects) = (0usize, 0usize);
    for i in 0..frames {
        let wl = read_window(&mut rx, &mut win);
        let bl = u32::from_le_bytes(win[BODY_LEN_AT..BODY_LEN_AT + 4].try_into().expect("4B")) as usize;
        match verify_prefix(&win[..wl], bl, i as u32, i as u32, &mut scratch) {
            Ok(()) => admits += 1,
            Err(AoqError::DigestMismatch { .. }) => rejects += 1,
            Err(other) => panic!("off-class refusal: {other:?}"),
        }
    }
    producer.join().expect("typing producer");
    let corrupt = corrupt_count.load(Ordering::Relaxed);
    assert_eq!(admits + rejects, frames, "accounting");
    assert_eq!(rejects, corrupt, "every tampered frame refused");
    assert!(rejects > 0 && admits > 0, "both classes witnessed (non-vacuous)");
    println!("tamper typing: {admits} admits, {rejects} typed refusals (injected {corrupt})");
}

// ---------------------------------------------------------------------------
// Structural court — test-KEM fallback disclosed on the session seam
// ---------------------------------------------------------------------------

/// Order-sensitive Shake256 squeeze (same construction as AF-1's corridor).
fn shake32(parts: &[&[u8]]) -> [u8; 32] {
    let mut sh = Shake256::instance();
    for p in parts {
        sh.absorb(p);
    }
    let mut out = [0u8; 32];
    sh.squeeze_into(&mut out);
    out
}

struct TestKem;

impl KemSeam for TestKem {
    fn encapsulate(&self, ek: &[u8; EK_BYTES]) -> Result<([u8; CT_BYTES], [u8; SS_BYTES],), cgap_aoq::session::SessionError> {
        let ss = shake32(&[b"rx2/testkem/ss", ek]);
        let mut ct = [0u8; CT_BYTES];
        for (i, slot) in ct.iter_mut().enumerate() {
            *slot = shake32(&[b"rx2/testkem/ct", ek, &[i as u8]])[0];
        }
        Ok((ct, ss))
    }
    fn decapsulate(&self, dk: &[u8], _ct: &[u8; CT_BYTES]) -> Result<[u8; SS_BYTES], cgap_aoq::session::SessionError> {
        let mut ek = [0u8; EK_BYTES];
        ek.copy_from_slice(dk);
        Ok(shake32(&[b"rx2/testkem/ss", &ek]))
    }
    fn kdf(&self, parts: &[&[u8]]) -> [u8; 32] {
        shake32(parts)
    }
}

#[test]
fn rfc0089_test_kem_fallback_disclosed() {
    let ek = [7u8; EK_BYTES];
    let (ct, ss_tx) = TestKem.encapsulate(&ek).expect("encapsulate");
    let ss_rx = TestKem.decapsulate(&ek, &ct).expect("decapsulate");
    assert_eq!(ss_tx, ss_rx, "test-KEM shared-secret round trip");
    assert_ne!(ct, [0u8; CT_BYTES], "ciphertext is material, not zeroed");
    println!(
        "DISCLOSED: KEM seam bound to deterministic test-KEM (dk==ek); real \
         ML-KEM-768 (AB-5) is the close condition for the UNSUPPORTED row"
    );
}

// ---------------------------------------------------------------------------
// Structural court — profile guard witness (RFC §5.2 loud-SKIP law)
// ---------------------------------------------------------------------------

#[test]
fn rfc0089_profile_guard_witness() {
    match profile_guard::assert_release() {
        Ok(()) => {
            // Release build: perf fences above are lawful; assert the inverse
            // axis is what a debug build would see (static fact, no run).
            assert!(!cfg!(debug_assertions));
            println!("profile guard: release witnessed");
        }
        Err(v) => {
            assert!(cfg!(debug_assertions), "violation must carry the debug axis");
            println!("profile guard: loud-SKIP witnessed ({v})");
        }
    }
}
