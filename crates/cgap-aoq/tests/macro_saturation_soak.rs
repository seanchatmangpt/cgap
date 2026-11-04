//! AF-32 macro benchmark: the zero-snaps saturation soak (v26.11.3).
//!
//! Continuous frame pipeline over REAL socketpair UDS, composed from
//! landed surfaces (no re-implementation):
//! - AB-8 writer/verify pipeline: `AoqWriter::frame_segments` scatter
//!   writev -> socketpair -> `verify_window` + `decrypt_in_place`
//!   (ab8_soak.rs is the direct ancestor of the hot loop).
//! - AB-10 partial-accept storm + Ok(0) injection: storm frames (every
//!   64th seq) take a 1-byte-per-accept adapter; designated storm frames
//!   additionally take one leading Ok(0) (simulated would-block; retried
//!   with no time-based coordination).
//! - AB-11 FrameTooLarge: oversize bodies refused typed over a real
//!   wire-shm ring segment.
//! - AB-17-pattern churn: per-lane worker cycles killed mid-stream and
//!   respawned; the respawn resumes from durable state (writer cursor +
//!   chain carry checkpoint) (thread-level, the macro_defense_mesh
//!   pattern; the cross-repo AB-17 real-process court is cited in cbmqs
//!   G6).
//!
//! Refusal census (typed, never drops): corrupt-tag frames (every 1024th
//! seq, tag flipped pre-write) refuse typed at verify; truncated windows
//! refuse typed offline; oversize bodies refuse `FrameTooLarge` on a
//! real ring segment.
//!
//! GATES (asserted): valid frames dropped == 0 (chain segment links ==
//! valid frames + carry links; every segment verify_all green at close);
//! unhandled worker exceptions == 0; fd count delta == 0 (lsof); RSS
//! plateaus; strict window alloc court 0 allocs / 0 bytes with positive
//! control; chain integrity post-soak.
//!
//! ESSAY-CLAIMS law: the dossier's 1-billion-transaction target is the
//! full-scale `#[ignore]` mode (`cargo test -p cgap-aoq --test
//! macro_saturation_soak -- --ignored`). The default scaled mode measures
//! what fits CI-adjacent time. Every receipt number is measured; the 1B
//! runtime is arithmetic extrapolation from the scaled rate, labeled as
//! such.

use std::io::{IoSlice, Read, Write};
use std::os::unix::net::UnixStream;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use cgap_aoq::chain::{AoqChain, AoqChainRecord, ChainDigest, GENESIS};
use cgap_aoq::error::AoqError;
use cgap_aoq::verify::{decrypt_in_place, verify_window};
use cgap_aoq::writer::{AoqWriter, TAG_BYTES};
use cgap_arrow::blake3;
use cgap_arrow::ipc;
use wire_shm::{WireError, WireRing, MAX_PAYLOAD};

// ------------------------------------------------ scale decisions --

/// Concurrent lanes: 8 socketpair UDS lanes, each with its own writer
/// and windowed chain.
const LANES: usize = 8;
/// Frames per worker cycle; cycles are the churn unit.
const BATCH: usize = 25_000;
/// Chain records per segment window. The slab is reused per cycle:
/// memory-bounded at 65536*120B/lane regardless of total frames.
const CHAIN_WIN: usize = 65_536;
/// RSS/fd sample points (waves are the join barriers between samples).
const WAVES: usize = 10;
/// Default scaled mode frames (dossier 1B scaled; CI-adjacent minutes).
const SCALED_FRAMES: usize = 10_000_000;
/// Full-scale mode frames (dossier target; `#[ignore]`).
const FULL_FRAMES: usize = 1_000_000_000;
/// Strict zero-alloc window (lane 0, wave 0, cycle 0, global seq
/// [1024, 1024+STRICT_N)).
const STRICT_N: usize = 4_096;
/// seq < WARMUP: storm/corrupt rules suppressed (clean traffic under
/// the strict window).
const WARMUP: usize = 8_192;
/// Oversize ring attempts per designated cycle (cycle % 4 == 3).
const RING_ATTEMPTS: usize = 16;
/// Injected-fault cadence (deterministic in global seq).
const STORM_EVERY: u64 = 64; // seq % 64 == 13
const OK0_EVERY: u64 = 512; // seq % 512 == 141 (storm subset)
const CORRUPT_EVERY: u64 = 1024; // seq % 1024 == 777

// ------------------------------------------------ census -----------

static VALID: AtomicUsize = AtomicUsize::new(0);
static CORRUPT_REFUSED: AtomicUsize = AtomicUsize::new(0);
static TRUNC_REFUSED: AtomicUsize = AtomicUsize::new(0);
static OVERSIZE_REFUSED: AtomicUsize = AtomicUsize::new(0);
static STORM_CALLS: AtomicUsize = AtomicUsize::new(0);
static OK0_RETRIES: AtomicUsize = AtomicUsize::new(0);
static KILLED: AtomicUsize = AtomicUsize::new(0);
static RESPAWNED: AtomicUsize = AtomicUsize::new(0);
static UNHANDLED: AtomicUsize = AtomicUsize::new(0);
static SEGMENTS_VERIFIED: AtomicUsize = AtomicUsize::new(0);
static SEGMENTS_OPENED: AtomicUsize = AtomicUsize::new(0);
static STRICT_ALLOCS: AtomicUsize = AtomicUsize::new(0);
static STRICT_BYTES: AtomicUsize = AtomicUsize::new(0);
static STRICT_POS: AtomicUsize = AtomicUsize::new(0);
static DESIGNATED: AtomicUsize = AtomicUsize::new(0);
static TRUNC_LEGS: AtomicUsize = AtomicUsize::new(0);

// ------------------------------------------------ allocator court --

/// Thread-local counting allocator (cbmqs G1 method): exact under
/// parallelism because each allocation lands on its own thread.
struct Counting;

thread_local! {
    static ALLOCS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    static BYTES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

unsafe impl std::alloc::GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: std::alloc::Layout) -> *mut u8 {
        ALLOCS.with(|c| c.set(c.get() + 1));
        BYTES.with(|c| c.set(c.get() + l.size() as u64));
        unsafe { std::alloc::System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: std::alloc::Layout) {
        unsafe { std::alloc::System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: std::alloc::Layout, n: usize) -> *mut u8 {
        ALLOCS.with(|c| c.set(c.get() + 1));
        BYTES.with(|c| c.set(c.get() + n as u64));
        unsafe { std::alloc::System.realloc(p, l, n) }
    }
}

#[global_allocator]
static GLOBAL: Counting = Counting;

// ------------------------------------------------ digest substrate -

/// Test digest substrate (macro_defense_mesh Mix32; alloc-free).
#[derive(Clone, Copy)]
struct Mix32;
impl ChainDigest for Mix32 {
    fn digest(&self, parts: &[&[u8]]) -> [u8; 32] {
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
        for (i, chunk) in out.chunks_exact_mut(8).enumerate() {
            let v = if i % 2 == 0 {
                a.wrapping_add(i as u64)
            } else {
                b.wrapping_add(i as u64)
            };
            chunk.copy_from_slice(&v.to_le_bytes());
        }
        out
    }
}

/// Keystream block (host-derived; R-K1 law, ab8_soak method).
fn keystream_block(seq: u32, block_no: u32) -> [u8; 32] {
    let mut inp = [0u8; 40];
    inp[0..7].copy_from_slice(b"af32-ks");
    inp[7..11].copy_from_slice(&seq.to_le_bytes());
    inp[11..15].copy_from_slice(&1_760_000_032u32.to_le_bytes());
    inp[15..19].copy_from_slice(&block_no.to_le_bytes());
    blake3::hash(&inp)
}

/// Deterministic frame class in global seq.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Class {
    Plain,
    Storm,
    Ok0Storm,
    Corrupt,
}

fn classify(seq: u64) -> Class {
    if seq < WARMUP as u64 {
        return Class::Plain;
    }
    if seq % CORRUPT_EVERY == 777 {
        return Class::Corrupt;
    }
    if seq % OK0_EVERY == 141 {
        return Class::Ok0Storm;
    }
    if seq % STORM_EVERY == 13 {
        return Class::Storm;
    }
    Class::Plain
}

/// Count of n in [0, max) with n % m == r (arithmetic witness for the
/// refusal-census asserts; no looping over 1e9).
fn count_pred(max: u64, m: u64, r: u64) -> u64 {
    if max <= r {
        0
    } else {
        (max - r + m - 1) / m
    }
}

/// Per-frame body (varies with seq; alloc-free fill).
fn fill_body(body: &mut [u8; 64], seq: u32) {
    let s = seq.to_le_bytes();
    for (i, b) in body.iter_mut().enumerate() {
        *b = s[i & 3] ^ i as u8 ^ 0x5A;
    }
}

static ZERO_PAD: [u8; 64] = [0u8; 64];
/// Window: 8 prefix + 120 header + 64 body + 64 pad + 8 tail.
const WIN_LEN: usize = 264;

/// Write adapter injecting partial accepts (1 byte/call) and a leading
/// Ok(0) on designated frames. Retries carry no sleep: the caller's
/// advance loop is the poll.
struct StormTx<'a> {
    inner: &'a UnixStream,
    ok0: bool,
}

impl Write for StormTx<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> std::io::Result<usize> {
        if self.ok0 {
            self.ok0 = false; // one leading Ok(0) per designated frame
            OK0_RETRIES.fetch_add(1, Ordering::Relaxed);
            return Ok(0);
        }
        STORM_CALLS.fetch_add(1, Ordering::Relaxed);
        // accept exactly one byte per call: maximal partial accept
        for b in bufs {
            if !b.is_empty() {
                return self.inner.write(&b[..1]);
            }
        }
        Ok(0)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// Advance-loop write of fixed pieces through the storm adapter (fixed
/// arrays only: allocation-free).
fn storm_write_all(tx: &mut StormTx, pieces: &[&[u8]; 6]) -> std::io::Result<usize> {
    let mut heads: [&[u8]; 6] = *pieces;
    let mut total = 0usize;
    loop {
        let mut bufs: [IoSlice; 6] = [
            IoSlice::new(&[]),
            IoSlice::new(&[]),
            IoSlice::new(&[]),
            IoSlice::new(&[]),
            IoSlice::new(&[]),
            IoSlice::new(&[]),
        ];
        let mut k = 0usize;
        for h in heads.iter_mut() {
            if !h.is_empty() {
                bufs[k] = IoSlice::new(h);
                k += 1;
            }
        }
        if k == 0 {
            break;
        }
        let n = tx.write_vectored(&bufs[..k])?;
        if n == 0 {
            // adapter already census'd this Ok(0)
            continue;
        }
        total += n;
        let mut left = n;
        for h in heads.iter_mut() {
            if left == 0 {
                break;
            }
            let take = left.min(h.len());
            *h = &h[take..];
            left -= take;
        }
    }
    Ok(total)
}

/// One frame: seal (keystream XOR + blake3 tag), scatter writev (plain
/// or storm path), commit the writer cursor.
fn seal_and_write(
    tx: &mut UnixStream,
    w: &mut AoqWriter,
    body: &[u8; 64],
    storm: bool,
    ok0: bool,
    corrupt: bool,
) -> std::io::Result<usize> {
    let seq = w.cursor().1;
    let mut ct = *body;
    let ks0 = keystream_block(seq, 0);
    let ks1 = keystream_block(seq, 1);
    for i in 0..64 {
        ct[i] ^= if i < 32 { ks0[i] } else { ks1[i - 32] };
    }
    let mut tag16: [u8; TAG_BYTES] = blake3::hash(&ct)[..TAG_BYTES].try_into().expect("16B tag");
    if corrupt {
        tag16[0] ^= 0x40; // corrupt injection: flip one tag byte pre-write
    }
    let segs = w
        .frame_segments(0, 7, &tag16, &ct)
        .expect("af32: lawful frame");
    let pieces: [&[u8]; 6] = [
        segs.prefix,
        segs.header,
        segs.pad,
        segs.body,
        &ZERO_PAD,
        segs.tail,
    ];
    let n = if storm {
        let mut stx = StormTx { inner: tx, ok0 };
        storm_write_all(&mut stx, &pieces)?
    } else {
        let bufs = [
            IoSlice::new(segs.prefix),
            IoSlice::new(segs.header),
            IoSlice::new(segs.pad),
            IoSlice::new(segs.body),
            IoSlice::new(&ZERO_PAD),
            IoSlice::new(segs.tail),
        ];
        tx.write_vectored(&bufs)?
    };
    w.commit();
    Ok(n)
}

/// Reader leg: read one window; corrupt-class frames must refuse typed;
/// valid frames verify, unseal, and return (plaintext, ct||tag payload)
/// for the chain receipt.
fn read_verify_unseal(
    rx: &mut UnixStream,
    ordinal: u32,
    seq: u32,
    win: &mut [u8; WIN_LEN],
    corrupt: bool,
) -> Result<Option<([u8; 64], [u8; 80])>, AoqError> {
    rx.read_exact(win)
        .map_err(|_| AoqError::HeaderTooShort { got: 0 })?;
    let mut ct = [0u8; 64];
    let parsed = match ipc::read_with_meta(win, None, &mut ct) {
        Ok(p) => p,
        Err(ipc::IpcError::TagMismatch { .. }) if corrupt => {
            // typed refusal at the ipc layer; census, not a drop
            return Ok(None);
        }
        Err(e) => panic!("af32: frame {seq} unexpected ipc refusal: {e:?}"),
    };
    let (n, _) = parsed;
    debug_assert_eq!(n, 64);
    // The tag law is caller-owned (verify_window checks span/cursor
    // lockstep; the caller supplies the expected tag): expected tag =
    // blake3 of the sealed body at window offset 128 (8 prefix + 120
    // header).
    let expected_tag: [u8; TAG_BYTES] = blake3::hash(&win[128..192])[..TAG_BYTES]
        .try_into()
        .expect("16B tag");
    let tag: [u8; TAG_BYTES] = win[76..92].try_into().expect("tag span");
    if corrupt {
        match verify_window(win, ordinal, seq, &expected_tag) {
            Err(AoqError::DigestMismatch { .. }) => return Ok(None),
            other => panic!("af32: corrupt frame {seq} not refused typed: {other:?}"),
        }
    }
    let vw = verify_window(win, ordinal, seq, &expected_tag)?;
    let mut ks = [0u8; 64];
    ks[..32].copy_from_slice(&keystream_block(seq, 0));
    ks[32..].copy_from_slice(&keystream_block(seq, 1));
    decrypt_in_place(&vw, &ks, win)?;
    let mut plain = [0u8; 64];
    plain.copy_from_slice(&win[vw.body_at..vw.body_at + 64]);
    let mut payload = [0u8; 80];
    payload[..64].copy_from_slice(&ct);
    payload[64..].copy_from_slice(&tag);
    Ok(Some((plain, payload)))
}

// ------------------------------------------------ churn supervisor --

/// One worker cycle: global seqs [start, start+count). Returns (frames
/// completed, segment checkpoint, links in the segment). A kill returns
/// early at the frame boundary; the partial segment is verified at close
/// and its checkpoint carried into the respawn's segment.
struct CycleSpec {
    start: u64,
    count: usize,
    kill_at: Option<usize>,
    strict: bool,
}

#[allow(clippy::too_many_arguments)]
fn run_cycle(
    spec: CycleSpec,
    tx: &mut UnixStream,
    rx: &mut UnixStream,
    w: &mut AoqWriter,
    slab: &mut [AoqChainRecord],
    carry_in: [u8; 32],
) -> (usize, [u8; 32], usize) {
    let mut win = [0u8; WIN_LEN];
    let mut body = [0u8; 64];
    let mut chain = AoqChain::new(Mix32, slab);
    // Carry-record law: every segment opens with one carry link whose
    // payload is the previous segment's checkpoint (GENESIS for the
    // first) — cross-segment hash linkage across churn respawns.
    chain.append(0, &carry_in).expect("af32: carry link");
    SEGMENTS_OPENED.fetch_add(1, Ordering::Relaxed);
    let mut done = 0usize;
    let (mut s0, mut b0): (u64, u64) = (0, 0);
    let strict_start = 1024usize;
    if spec.strict {
        assert!(
            spec.count >= strict_start + 64,
            "af32: strict cycle too short"
        );
        // Positive control: an explicit alloc proves the counter moves.
        let probe: Vec<u8> = vec![7u8; 991];
        assert_eq!(probe[990], 7);
        assert!(alloc_delta().0 > 0, "VACUOUS COUNTER: strict worker");
        STRICT_POS.store(1, Ordering::Relaxed);
    }
    for i in 0..spec.count {
        if Some(i) == spec.kill_at {
            break; // churn kill: mid-stream, at the frame boundary
        }
        let seq = spec.start + i as u64;
        let class = classify(seq);
        if spec.strict && i == strict_start {
            (s0, b0) = alloc_delta(); // window baseline
        }
        fill_body(&mut body, seq as u32);
        let storm = matches!(class, Class::Storm | Class::Ok0Storm);
        let ok0 = matches!(class, Class::Ok0Storm);
        let corrupt = class == Class::Corrupt;
        let (ord, lseq) = w.cursor();
        seal_and_write(tx, w, &body, storm, ok0, corrupt).expect("af32: seal+write");
        let res = read_verify_unseal(rx, ord, lseq, &mut win, corrupt);
        match res {
            Ok(Some((plain, payload))) => {
                let mut expect = [0u8; 64];
                fill_body(&mut expect, seq as u32);
                assert_eq!(plain, expect, "frame {seq}: round-trip identity");
                chain.append(0, &payload).expect("af32: chain append");
                VALID.fetch_add(1, Ordering::Relaxed);
            }
            Ok(None) => {
                // typed refusal census (corrupt frame), not a drop
                CORRUPT_REFUSED.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) => panic!("af32: frame {seq} unexpected refusal: {e:?}"),
        }
        if spec.strict && i == spec.count - 1 {
            // Strict delta over [strict_start, count): clean traffic
            // (seq < WARMUP), whether or not the full STRICT_N fit.
            let (sa, sb) = alloc_delta();
            STRICT_ALLOCS.store(sa.saturating_sub(s0) as usize, Ordering::Relaxed);
            STRICT_BYTES.store(sb.saturating_sub(b0) as usize, Ordering::Relaxed);
        }
        done += 1;
    }
    // Segment close: verify (partial on kill), checkpoint, carry out.
    chain.verify_all().expect("af32: segment verify_all");
    SEGMENTS_VERIFIED.fetch_add(1, Ordering::Relaxed);
    let links = chain.len();
    let chk = chain.checkpoint();
    if Some(done) == spec.kill_at {
        KILLED.fetch_add(1, Ordering::Relaxed);
    }
    (done, chk, links)
}

/// Ring refusal leg: oversize bodies refused `FrameTooLarge` typed on a
/// real wire-shm ring segment (AB-11 court surface).
fn oversize_leg(lane: usize, cycle: usize) {
    for _ in 0..RING_ATTEMPTS {
        let seg = std::env::temp_dir().join(format!(
            "af32-ovs-{}-{}-{}",
            std::process::id(),
            lane,
            cycle
        ));
        let _ = std::fs::remove_file(&seg);
        let mut ring = WireRing::create(&seg, 8).expect("af32: ring create");
        let oversized = vec![0u8; MAX_PAYLOAD + 1];
        match ring.stamp_frame(&oversized, 0) {
            Err(WireError::FrameTooLarge { .. }) => {
                OVERSIZE_REFUSED.fetch_add(1, Ordering::Relaxed);
            }
            other => panic!("af32: oversize not refused typed: {other:?}"),
        }
        drop(ring);
        let _ = std::fs::remove_file(&seg);
    }
}

/// Truncation leg: offline windows cut short refuse typed (ipc
/// Truncated / BadBodyLen / framing law).
fn truncation_leg(seq_probe: u32) {
    let mut win = [0u8; WIN_LEN];
    win[0] = 0xAB;
    let mut ct = [0u8; 64];
    let mut refused = 0usize;
    for cut in [4usize, 8, 40, 100, 200, WIN_LEN - 8] {
        match ipc::read_with_meta(&win[..cut], None, &mut ct) {
            Err(_) => refused += 1,
            Ok(_) => panic!("af32: truncated window accepted at cut {cut}"),
        }
    }
    TRUNC_REFUSED.fetch_add(refused, Ordering::Relaxed);
    let _ = seq_probe;
}

/// Per-lane supervisor: owns the durable state (socketpair, writer,
/// chain slab), drives churn cycles until the quota is met, returns
/// (frames done, designated oversize cycles, checkpoints).
fn lane_loop(
    lane: usize,
    quota: usize,
    slab: &mut [AoqChainRecord],
    start_seq: u64,
) -> (usize, usize, Vec<[u8; 32]>) {
    let batch = BATCH.min(quota / 2).max(1024);
    let (mut tx, mut rx) = UnixStream::pair().expect("af32: socketpair");
    let mut w = AoqWriter::new();
    let mut carry = GENESIS;
    let mut checkpoints: Vec<[u8; 32]> = Vec::with_capacity(quota / batch + 2);
    let mut next = start_seq;
    let mut remaining = quota;
    let mut designated = 0usize;
    let mut cycle_no = 0usize;
    while remaining > 0 {
        let count = remaining.min(batch);
        if cycle_no % 4 == 3 {
            designated += 1;
            DESIGNATED.fetch_add(1, Ordering::Relaxed);
            oversize_leg(lane, cycle_no);
        }
        if cycle_no % 4 == 1 {
            TRUNC_LEGS.fetch_add(1, Ordering::Relaxed);
            truncation_leg(cycle_no as u32);
        }
        // Every non-final cycle is killed mid-stream (deterministic kill
        // point); the last cycle runs to completion. Strict cycle 0 of
        // lane 0 runs clean (no kill) so the alloc window lands.
        let strict = lane == 0 && cycle_no == 0;
        let kill_at = if strict || count == remaining {
            None
        } else {
            Some(64 + (lane * 7_919 + cycle_no * 104_729) % (batch - 128))
        };
        let spec = CycleSpec {
            start: next,
            count,
            kill_at,
            strict,
        };
        let slab_len = CHAIN_WIN.min(slab.len());
        let out = thread::scope(|s| {
            let slab_ref: &mut [AoqChainRecord] = &mut slab[..slab_len];
            let h = s.spawn(|| run_cycle(spec, &mut tx, &mut rx, &mut w, slab_ref, carry));
            match catch_unwind(AssertUnwindSafe(|| h.join())) {
                Ok(Ok(r)) => Some(r),
                Ok(Err(_)) => {
                    UNHANDLED.fetch_add(1, Ordering::Relaxed);
                    None
                }
                Err(_) => {
                    UNHANDLED.fetch_add(1, Ordering::Relaxed);
                    None
                }
            }
        });
        let (done, chk, _links) = match out {
            Some(r) => r,
            None => panic!("af32: lane {lane} cycle {cycle_no}: unhandled worker panic"),
        };
        if kill_at.is_some() && done < count {
            RESPAWNED.fetch_add(1, Ordering::Relaxed);
        }
        carry = chk;
        checkpoints.push(chk);
        next += done as u64;
        remaining -= done;
        cycle_no += 1;
        assert!(done > 0, "af32: lane {lane}: zero-progress cycle");
    }
    (quota, designated, checkpoints)
}

// ------------------------------------------------ helpers ----------

fn alloc_delta() -> (u64, u64) {
    (ALLOCS.with(|c| c.get()), BYTES.with(|c| c.get()))
}

/// lsof fd census for this process.
fn fd_count() -> usize {
    let out = std::process::Command::new("lsof")
        .arg("-p")
        .arg(std::process::id().to_string())
        .output()
        .expect("af32: lsof");
    String::from_utf8_lossy(&out.stdout).lines().count()
}

/// Resident set size in KiB (macOS ps).
fn rss_kib() -> usize {
    let out = std::process::Command::new("ps")
        .arg("-o")
        .arg("rss=")
        .arg("-p")
        .arg(std::process::id().to_string())
        .output()
        .expect("af32: ps");
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse::<usize>()
        .expect("af32: rss parse")
}

// ------------------------------------------------ soak driver -------

/// Measured-column block printed at end of run (receipt source).
fn report(total: usize, dt: std::time::Duration, rss: &[usize], fd0: usize, fd1: usize) {
    let fps = total as f64 / dt.as_secs_f64();
    println!("af32 SOAK REPORT (measured)");
    println!("  frames_total_requested : {total}");
    println!(
        "  frames_valid           : {}",
        VALID.load(Ordering::Relaxed)
    );
    println!(
        "  refusals_corrupt_typed : {}",
        CORRUPT_REFUSED.load(Ordering::Relaxed)
    );
    println!(
        "  refusals_trunc_typed   : {}",
        TRUNC_REFUSED.load(Ordering::Relaxed)
    );
    println!(
        "  refusals_oversize_typed: {}",
        OVERSIZE_REFUSED.load(Ordering::Relaxed)
    );
    println!(
        "  storm_write_calls      : {}",
        STORM_CALLS.load(Ordering::Relaxed)
    );
    println!(
        "  ok0_retries            : {}",
        OK0_RETRIES.load(Ordering::Relaxed)
    );
    println!(
        "  churn killed           : {}",
        KILLED.load(Ordering::Relaxed)
    );
    println!(
        "  churn respawned        : {}",
        RESPAWNED.load(Ordering::Relaxed)
    );
    println!(
        "  unhandled_exceptions   : {}",
        UNHANDLED.load(Ordering::Relaxed)
    );
    println!(
        "  chain segments verified: {}",
        SEGMENTS_VERIFIED.load(Ordering::Relaxed)
    );
    println!(
        "  chain segments opened  : {}",
        SEGMENTS_OPENED.load(Ordering::Relaxed)
    );
    println!(
        "  strict alloc court     : {} allocs / {} bytes / positive-control {}",
        STRICT_ALLOCS.load(Ordering::Relaxed),
        STRICT_BYTES.load(Ordering::Relaxed),
        STRICT_POS.load(Ordering::Relaxed)
    );
    println!("  duration               : {dt:?}");
    println!("  throughput             : {fps:.0} frames/sec");
    println!("  rss samples KiB        : {rss:?}");
    println!("  fd count {fd0} -> {fd1}");
}

/// Shared soak body. Returns frames/sec (measured).
fn run_soak(total_requested: usize) -> f64 {
    let per_lw = total_requested / (WAVES * LANES);
    let total = per_lw * WAVES * LANES;
    let mut slabs: Vec<Vec<AoqChainRecord>> = Vec::with_capacity(LANES);
    for _ in 0..LANES {
        slabs.push(vec![AoqChainRecord::zeroed(); CHAIN_WIN]);
    }
    let fd0 = fd_count();
    let rss0 = rss_kib();
    let mut rss_samples: Vec<usize> = Vec::new();
    let t0 = std::time::Instant::now();
    for wave in 0..WAVES {
        thread::scope(|s| {
            for (lane, slab) in slabs.iter_mut().enumerate() {
                let slab = &mut slab[..];
                s.spawn(move || {
                    let start_seq = (wave * LANES + lane) as u64 * per_lw as u64;
                    let (_q, _d, _cp) = lane_loop(lane, per_lw, slab, start_seq);
                    let _ = (_q, _cp);
                });
            }
        });
        rss_samples.push(rss_kib());
    }
    let dt = t0.elapsed();
    let fd1 = fd_count();
    // -------- gates --------
    // Census identity: every seq classified, none dropped.
    let warm = WARMUP as u64;
    let t = total as u64;
    let exp_storm = count_pred(t, STORM_EVERY, 13) - count_pred(warm, STORM_EVERY, 13);
    let exp_ok0 = count_pred(t, OK0_EVERY, 141) - count_pred(warm, OK0_EVERY, 141);
    let exp_corrupt = count_pred(t, CORRUPT_EVERY, 777) - count_pred(warm, CORRUPT_EVERY, 777);
    assert_eq!(
        VALID.load(Ordering::Relaxed) as u64,
        t - exp_corrupt,
        "DROP LAW: valid frames must all land exactly once"
    );
    assert_eq!(
        CORRUPT_REFUSED.load(Ordering::Relaxed) as u64,
        exp_corrupt,
        "corrupt census must equal the injected count"
    );
    assert_eq!(
        OVERSIZE_REFUSED.load(Ordering::Relaxed) as u64,
        (DESIGNATED.load(Ordering::Relaxed) * RING_ATTEMPTS) as u64,
        "oversize census mismatch"
    );
    assert_eq!(
        TRUNC_REFUSED.load(Ordering::Relaxed),
        TRUNC_LEGS.load(Ordering::Relaxed) * 6,
        "truncation census mismatch (6 probes per leg)"
    );
    assert!(
        TRUNC_LEGS.load(Ordering::Relaxed) > 0,
        "trunc legs must run"
    );
    assert!(
        STORM_CALLS.load(Ordering::Relaxed) as u64 > exp_storm,
        "storm witness weak"
    );
    assert_eq!(
        OK0_RETRIES.load(Ordering::Relaxed) as u64,
        exp_ok0,
        "Ok(0) census must equal the injected count"
    );
    assert!(
        KILLED.load(Ordering::Relaxed) >= LANES * WAVES,
        "churn witness weak"
    );
    assert_eq!(
        RESPAWNED.load(Ordering::Relaxed),
        KILLED.load(Ordering::Relaxed),
        "every kill must be respawned"
    );
    assert_eq!(
        UNHANDLED.load(Ordering::Relaxed),
        0,
        "unhandled worker exceptions must be 0"
    );
    assert_eq!(
        SEGMENTS_VERIFIED.load(Ordering::Relaxed),
        SEGMENTS_OPENED.load(Ordering::Relaxed),
        "every opened chain segment must verify at close"
    );
    assert_eq!(
        STRICT_ALLOCS.load(Ordering::Relaxed),
        0,
        "strict alloc court: allocations in the steady-state window"
    );
    assert_eq!(
        STRICT_BYTES.load(Ordering::Relaxed),
        0,
        "strict alloc court: bytes in the steady-state window"
    );
    assert_eq!(
        STRICT_POS.load(Ordering::Relaxed),
        1,
        "positive control ran"
    );
    assert_eq!(fd1, fd0, "fd delta must be 0 (lsof)");
    // RSS plateau: last-half band tight, bounded total growth.
    let n = rss_samples.len();
    assert!(n >= 5, "rss sample count");
    let last = &rss_samples[n / 2..];
    let peak = *rss_samples.iter().max().expect("rss");
    let lo = *last.iter().min().expect("rss");
    let hi = *last.iter().max().expect("rss");
    let band = (peak * 5 / 100).max(16 * 1024);
    let band_used = hi - lo;
    assert!(
        band_used <= band,
        "RSS must plateau: last-half band {band_used} KiB > {band} KiB"
    );
    assert!(
        hi <= rss0 + 512 * 1024,
        "RSS growth must be bounded: {} -> {hi} KiB",
        rss0
    );
    report(total, dt, &rss_samples, fd0, fd1);
    total as f64 / dt.as_secs_f64()
}

/// (designated oversize attempts are counted by lane_loop and summed in
/// run_soak; no external arithmetic mirror to drift.)

// ------------------------------------------------ test entry points -

/// Scaled saturation soak (default; dossier 1B scaled to CI-adjacent
/// minutes). Override with CGAP_AF32_FRAMES.
#[test]
fn af32_saturation_soak_scaled() {
    let frames = std::env::var("CGAP_AF32_FRAMES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(SCALED_FRAMES);
    let fps = run_soak(frames);
    println!("af32 scaled mode: {fps:.0} frames/sec measured");
}

/// Full-scale soak: the dossier's 1-billion-transaction target.
/// `cargo test -p cgap-aoq --test macro_saturation_soak -- --ignored`
/// Runtime is extrapolated from the scaled rate (arithmetic, labeled as
/// such in the receipt).
#[test]
#[ignore = "full-scale 1B soak; run explicitly with -- --ignored"]
fn af32_saturation_soak_full_1b() {
    let fps = run_soak(FULL_FRAMES);
    println!("af32 full mode: {fps:.0} frames/sec measured");
}
