//! StreamWriter stress suite (lane AB-10, STRESS).
//!
//! Four courts over the REAL collaborators (`StreamWriter` + `VectoredSink`
//! over live UDS, raw `ipc::emit` as the byte-identity reference):
//!
//! 1. SOAK: 50k frames seal-scatter -> writev -> peer read_exact, each
//!    frame byte-identical to a raw `ipc::emit` reference built with the
//!    same seq; throughput is RECORDED, never fenced (no wall-clock assert).
//! 2. PARTIAL-ACCEPT STORM: `FirstSegSink` (one segment per call) over
//!    10k frames — reassembly byte-identical to the single-shot reference;
//!    zero `ZeroProgress` (a FirstSeg accept is always > 0).
//! 3. STALLED-SINK RECOVERY: sink returns `Ok(0)` every Nth accept for 1k
//!    frames — writer refuses with `ZeroProgress`, caller retries the same
//!    frame at the same frame-start offset, and the walk preserves
//!    seq/ordinal lockstep (no skips, no doubles) INCLUDING the u32 wrap
//!    (started at u32::MAX - 2).
//! 4. FD-HYGIENE WITNESS: 1k sequential UDS pairs created and torn down;
//!    lsof-count delta ~0 for the test process (lock-fd law).
//!
//! Chicago discipline: real UDS, real kernel writev, real bytes asserted
//! on the wire. No mocks; the stall/partial sinks are real hand-written
//! `VectoredSink` implementations (fault-injection fixtures, not stand-ins
//! for a collaborator). Timing tolerance: no absolute-ms fences anywhere;
//! throughput is printed as a receipt, ratio-free.
//!
//! Anti-vacuity: every identity court compares against an INDEPENDENT
//! reference path (raw `ipc::emit`), so a StreamWriter framing regression
//! flips all of soak/storm/stall; the stall court additionally asserts the
//! sink actually witnessed Ok(0) injects (count > 0), and the fd court
//! asserts the pre-count is nonzero (lsof actually listed the process).

use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use cgap_arrow::ipc::{emit as raw_emit, seal_meta, window_len, MetaHeader, LAYOUT_VERSION, MAGIC};
use cgap_arrow::stream_writer::{StreamError, StreamWriter, VectoredSink};

// ---- deterministic bodies (SplitMix64, frozen kernel multiplier) ----

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_4A9B_B4E6_3200);
    z ^ (z >> 31)
}

fn make_body(frame: u64, len: usize) -> Vec<u8> {
    let mut st: u64 = 0xA20C_0FFE_E5A7_E000 ^ frame.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let mut b = Vec::with_capacity(len);
    while b.len() < len {
        b.extend_from_slice(&splitmix64(&mut st).to_le_bytes());
    }
    b.truncate(len);
    b
}

fn meta_for(seq: u32, body: &[u8]) -> MetaHeader {
    seal_meta(MetaHeader {
        magic: MAGIC,
        version: LAYOUT_VERSION,
        flags: 0,
        schema_digest: [0; 32],
        body_len: body.len() as u32,
        alignment: 64,
        seq,
        reserved0: 0,
        header_digest: 0,
        pad_tail: [0; 4],
    })
}

/// Independent reference: raw ipc::emit at the same seq — the StreamWriter
/// path must land byte-identical to this on the wire.
fn reference(seq: u32, body: &[u8]) -> Vec<u8> {
    let mut buf = vec![0u8; window_len(body.len(), 64)];
    let n = raw_emit(&mut buf, &meta_for(seq, body), body).expect("raw emit ok");
    buf.truncate(n);
    buf
}

/// seq lives at meta offset 0x30, meta starts at PREFIX_SIZE=8.
const SEQ_AT: usize = 8 + 0x30;

/// Test-only kernel writev adapter over a live UnixStream.
struct Uds<'a> {
    stream: &'a mut UnixStream,
}

impl<'a> VectoredSink for Uds<'a> {
    type Error = std::io::Error;
    fn write_vectored(&mut self, bufs: &[&[u8]]) -> Result<usize, Self::Error> {
        let mut slices = [std::io::IoSlice::new(&[]); 4];
        let k = bufs.len().min(slices.len());
        for (dst, src) in slices.iter_mut().zip(bufs) {
            *dst = std::io::IoSlice::new(src);
        }
        self.stream.write_vectored(&slices[..k])
    }
}

/// Accepts ONLY the first segment per call (partial-accept storm fixture).
struct FirstSegSink<'a> {
    out: &'a mut Vec<u8>,
    calls: usize,
}

impl<'a> VectoredSink for FirstSegSink<'a> {
    type Error = std::io::Error;
    fn write_vectored(&mut self, bufs: &[&[u8]]) -> Result<usize, Self::Error> {
        self.calls += 1;
        let first = bufs.first().copied().unwrap_or(&[]);
        if first.is_empty() {
            // The writer never passes an empty segment list with k >= 1 and
            // all-empty segments; accepting 0 here would flip the court.
            return Ok(0);
        }
        self.out.extend_from_slice(first);
        Ok(first.len())
    }
}

/// FULL-ACCEPT sink that returns Ok(0) every Nth call (stall fixture).
///
/// TERMINATION LAW (fixture bug found by this suite's first real run): a
/// periodic zero inject against a one-segment-per-call sink aligns
/// adversarially — every retry attempt makes exactly 3 accepts, then dies
/// on the injected zero, forever. That spin was witnessed live (stray test
/// binaries burned 15+ CPU-minutes before being killed). This fixture is
/// FULL-ACCEPT so each attempt either completes in one call or hits one
/// inject and completes on retry. The writer's advance loop is unchanged —
/// it correctly refused ZeroProgress on every injected zero; only the
/// fixture shapes WHEN zeros arrive so the caller's retry terminates.
struct StalledSink<'a> {
    out: &'a mut Vec<u8>,
    calls: usize,
    zero_injects: usize,
    every: usize,
}

impl<'a> VectoredSink for StalledSink<'a> {
    type Error = std::io::Error;
    fn write_vectored(&mut self, bufs: &[&[u8]]) -> Result<usize, Self::Error> {
        self.calls += 1;
        if self.calls % self.every == 0 {
            self.zero_injects += 1;
            return Ok(0);
        }
        for seg in bufs {
            self.out.extend_from_slice(seg);
        }
        Ok(bufs.iter().map(|s| s.len()).sum())
    }
}

fn socket_path(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock before epoch")
        .subsec_nanos();
    let mut p: PathBuf = std::env::temp_dir();
    p.push(format!(
        "cgap-stress-{}-{}-{}.sock",
        std::process::id(),
        nanos,
        tag
    ));
    p
}

/// One live UDS pair (listener accepted), both ends owned by the caller.
fn uds_pair(tag: &str) -> (UnixListener, UnixStream, UnixStream, PathBuf) {
    let path = socket_path(tag);
    let listener = UnixListener::bind(&path).expect("bind live UDS");
    let conn = UnixStream::connect(&path).expect("connect live UDS");
    let (peer, _addr) = listener.accept().expect("accept");
    (listener, conn, peer, path)
}

/// Frame ordinal -> deterministic body length (varies segment scatter).
fn body_len_for(frame: u64) -> usize {
    128 + ((frame % 7) as usize) * 16
}

/// SOAK: 50k frames over one live UDS pair, per-frame byte identity vs the
/// raw ipc::emit reference, peer read_exact per frame (kernel sync, no
/// sleeps). Throughput recorded as a receipt, never fenced.
#[test]
fn soak_50k_frames_byte_identity_and_throughput() {
    const FRAMES: u64 = 50_000;
    let (_listener, mut stream, mut peer, path) = uds_pair("soak");
    let mut writer = StreamWriter::with_seq(
        Uds {
            stream: &mut stream,
        },
        0,
    );
    let mut scratch: Vec<u8> = Vec::new();
    let total_bytes = core::cell::Cell::new(0usize);

    let t0 = Instant::now();
    for i in 0..FRAMES {
        let body = make_body(i, body_len_for(i));
        let want = reference(i as u32, &body);
        scratch.resize(want.len(), 0);
        let n = writer
            .emit(&mut scratch, meta_for(i as u32, &body), &body)
            .expect("soak emit ok");
        assert_eq!(n, want.len(), "window len matches reference at frame {i}");
        let mut got = vec![0u8; n];
        peer.read_exact(&mut got).expect("peer read_exact");
        assert_eq!(
            &got[..],
            &want[..],
            "frame {i} byte-identity vs raw emit reference"
        );
        total_bytes.set(total_bytes.get() + n);
    }
    let elapsed = t0.elapsed();

    let frames_sec = FRAMES as f64 / elapsed.as_secs_f64();
    let mb_sec = total_bytes.get() as f64 / (1024.0 * 1024.0) / elapsed.as_secs_f64();
    println!(
        "soak receipt: {FRAMES} frames, {} bytes, {:.3?}, {:.0} frames/sec, {:.1} MB/s",
        total_bytes.get(),
        elapsed,
        frames_sec,
        mb_sec
    );
    assert_eq!(writer.seq(), (FRAMES as u32), "final seq == frame count");
    drop(writer);
    drop(stream);
    drop(peer);
    let _ = std::fs::remove_file(&path);
}

/// PARTIAL-ACCEPT STORM: FirstSegSink over 10k frames; every window needs
/// multiple vectored calls; reassembly is byte-identical to the single-shot
/// reference concatenation; ZeroProgress never fires (accepts always > 0).
#[test]
fn partial_accept_storm_firstseg_10k() {
    const FRAMES: u64 = 10_000;
    let mut reassembled: Vec<u8> = Vec::new();
    let mut reference_all: Vec<u8> = Vec::new();
    let calls;
    {
        let mut sink = FirstSegSink {
            out: &mut reassembled,
            calls: 0,
        };
        let mut writer = StreamWriter::with_seq(&mut sink, 0);

        for i in 0..FRAMES {
            let body = make_body(i, body_len_for(i));
            let want = reference(i as u32, &body);
            reference_all.extend_from_slice(&want);
            let mut scratch = vec![0u8; want.len()];
            writer
                .emit(&mut scratch, meta_for(i as u32, &body), &body)
                .expect("storm emit ok");
        }
        assert!(writer.seq() == FRAMES as u32, "seq lockstep after storm");
        calls = sink.calls;
    }
    assert!(
        calls > FRAMES as usize,
        "storm exercised: {calls} calls > {FRAMES} frames (multiple accepts per window)"
    );
    assert_eq!(
        &reassembled, &reference_all,
        "FirstSegSink reassembly byte-identical to single-shot reference"
    );
}

/// STALLED-SINK RECOVERY: Ok(0) every 4th accept for 1k frames, started at
/// u32::MAX - 2 so the seq walks THROUGH the u32 wrap. The writer refuses
/// ZeroProgress on each injected stall; the caller retries the SAME frame
/// at the same frame-start offset. Asserts: every stall was witnessed, each
/// successful frame lands at the exact expected ordinal with the exact
/// expected seq (no skips, no doubles), and the full reassembly (rollback
/// + retry included) is byte-identical to the reference concatenation.
#[test]
fn stalled_sink_recovery_seq_lockstep_through_u32_wrap() {
    const FRAMES: u64 = 1_000;
    const START_SEQ: u32 = u32::MAX - 2;
    const EVERY: usize = 4;
    let mut reassembled: Vec<u8> = Vec::new();
    let mut reference_all: Vec<u8> = Vec::new();
    let mut sink = StalledSink {
        out: &mut reassembled,
        calls: 0,
        zero_injects: 0,
        every: EVERY,
    };

    for i in 0..FRAMES {
        let expected_seq = START_SEQ.wrapping_add(i as u32);
        let body = make_body(i, body_len_for(i));
        let want = reference(expected_seq, &body);
        reference_all.extend_from_slice(&want);
        let frame_start = sink.out.len();
        let mut scratch = vec![0u8; want.len()];
        // Each attempt is a fresh StreamWriter at the SAME seq: seq
        // advances only on a fully accepted window, so a retry after a
        // ZeroProgress refusal re-attempts the SAME ordinal — never a
        // skip, never a double.
        loop {
            let mut writer = StreamWriter::with_seq(&mut sink, expected_seq);
            match writer.emit(&mut scratch, meta_for(expected_seq, &body), &body) {
                Ok(_) => break,
                Err(StreamError::ZeroProgress) => {
                    // The inject appended nothing (Ok(0)), so the
                    // reassembly needs no rollback: the retry re-attempts
                    // the SAME ordinal at the same frame_start — never a
                    // skip, never a double. `into_inner` proves the sink is
                    // lawfully recoverable after a refusal.
                    let _recovered = writer.into_inner();
                }
                Err(e) => panic!("unexpected stream error: {e:?}"),
            }
        }
        assert_eq!(
            u32::from_le_bytes(
                sink.out[frame_start + SEQ_AT..frame_start + SEQ_AT + 4]
                    .try_into()
                    .unwrap()
            ),
            expected_seq,
            "wire seq stamped == expected ordinal seq at {i} (wrap-aware)"
        );
    }
    assert!(
        sink.zero_injects > 0,
        "vacuous court: no stalls actually injected"
    );
    // Exact arithmetic court: with a full-accept sink every completed
    // frame costs exactly one call and every injected zero costs exactly
    // one refused call + one retry — so calls == frames + injects. Any
    // writer-side double-emit or skipped window breaks this identity.
    assert_eq!(
        sink.calls,
        FRAMES as usize + sink.zero_injects,
        "stall walk arithmetic: {} calls vs {} frames + {} injects (skip/double detector)",
        sink.calls,
        FRAMES,
        sink.zero_injects
    );
    assert_eq!(
        &*sink.out, &reference_all,
        "stalled walk reassembly byte-identical (rollback+retry lossless)"
    );
}

/// FD-HYGIENE WITNESS: 1k sequential UDS pairs created and fully torn
/// down; lsof fd-count delta ~0 for the test process. Strict lefts:
/// zero sockets left at our path namespace; total fd delta recorded and
/// bounded (transient fds from concurrently running sibling tests are
/// noise the court names explicitly; run with --test-threads=1 for the
/// exact-zero reading).
#[test]
fn fd_hygiene_1k_uds_pairs() {
    fn fd_count() -> (usize, usize) {
        let out = Command::new("/usr/sbin/lsof")
            .args(["-a", "-p", &std::process::id().to_string(), "-F", "n"])
            .output()
            .expect("lsof ran");
        assert!(out.status.success(), "lsof exited nonzero");
        let text = String::from_utf8_lossy(&out.stdout);
        let total = text.lines().filter(|l| l.starts_with('n')).count();
        // Only THIS test's sockets (sibling tests hold live cgap-stress-*
        // pairs concurrently; their paths never carry the -fd tag).
        let ours = text
            .lines()
            .filter(|l| l.contains("cgap-stress-") && l.contains("-fd"))
            .count();
        (total, ours)
    }

    let (before_total, before_ours) = fd_count();
    assert!(
        before_total > 0,
        "vacuous court: lsof listed no fds for this process"
    );

    const PAIRS: usize = 1_000;
    for i in 0..PAIRS {
        let (_l, conn, peer, path) = uds_pair(&format!("fd{i}"));
        drop(conn);
        drop(peer);
        drop(_l);
        let removed = std::fs::remove_file(&path).is_ok();
        assert!(removed, "socket file {path:?} (pair {i}) not cleaned up");
    }

    let (after_total, after_ours) = fd_count();
    let delta = after_total as isize - before_total as isize;
    println!(
        "fd receipt: {PAIRS} pairs, lsof total {before_total} -> {after_total} (delta {delta}), cgap-stress sockets {before_ours} -> {after_ours}"
    );
    assert_eq!(
        after_ours, 0,
        "no cgap-stress fd sockets left open after teardown"
    );
    assert!(
        delta.abs() <= 4,
        "fd leak: lsof delta {delta} exceeds transient tolerance"
    );
}
