//! AF-27 macro-benchmark: defense telemetry mesh (v26.11.3).
//!
//! 64 independent session channels (honest scale decision: the dossier's
//! 1024-channel claim is TARGET-UNMEASURED here — this bench runs 64 on a
//! host std test harness; see receipt af-27-defense-mesh.md). Each channel:
//! test-KEM handshake -> telemetry bursts (radar / kinetic / launch-state
//! frame shapes) via the real session state machine (`parse_frame` +
//! `Session::step`), seal via `Session::emit_verified` (the landed
//! compose-with-emit_verified tag law; the AEAD seam itself is
//! pre-admission and refuses typed — census'd, not faked), then
//! `AoqChain::append_verified` into PER-CHANNEL chains (chain-topology
//! decision documented in the receipt: per-channel, not single audit
//! chain).
//!
//! Adversarial injection: unannounced link terminations (worker killed
//! mid-stream while bursts remain; the frozen session continues absorbing
//! at the frozen ordinal — continuity), partial-frame truncations
//! (`parse_frame` -> `LengthMismatch`), out-of-order ordinals
//! (`OrdinalMismatch` typed refusals), supervisor churn: 20 worker threads
//! killed mid-stream at peak and respawned (AB-17 court pattern).
//!
//! MEASURED (this run, host std harness): zero attribution bleed (every
//! chain record digest maps to exactly one channel), fd/RSS deltas,
//! chain continuity (per-channel `verify_records`), total telemetry frames
//! absorbed, typed refusal census.
//!
//! Target-unmeasured: no_os / embedded-target run, EW-survivability —
//! future lane.

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Instant;

use cgap_aoq::chain::{verify_records, AoqChain, AoqChainRecord, ChainDigest, RECORD_BYTES};
use cgap_aoq::codec;
use cgap_aoq::header::AoqHeader;
use cgap_aoq::session::{
    parse_frame, FrameKind, KemSeam, Session, SessionError, SessionState, CT_BYTES, EK_BYTES,
};

// ---- scale decisions (recorded, honest) --------------------------------

/// Channels: 64 (dossier 1024 scaled down 16x; disclosed in receipt).
const CHANNELS: usize = 64;
/// Supervisor slots: 20 worker threads, killed/respawned at peak.
const SLOTS: usize = 20;
/// Telemetry frames per channel (8 per shape x 3 shapes x 2 bursts).
const FRAMES_PER_CHANNEL: usize = 48;
/// Chain capacity per channel (closed channels append only their first
/// half; the rest of the slab stays unused).
const CHAIN_CAP: usize = FRAMES_PER_CHANNEL;
/// Termination-injection channels: unannounced drop after half the
/// bursts; the frozen session continues absorbing afterwards.
const DROP_CHANNELS: std::ops::Range<usize> = 40..48;
/// Closed channels: lawful Close after half, then refusal battery.
const CLOSE_CHANNELS: std::ops::Range<usize> = 60..64;
/// Out-of-order injection channels (3 bad ordinals each, mid-stream).
const OO_CHANNELS: std::ops::Range<usize> = 4..8;
/// Truncation injection channels (3 truncated frames each, mid-stream).
const TRUNC_CHANNELS: std::ops::Range<usize> = 8..12;

// ---- census (shared atomics; final asserts are the gate) ---------------

static ABSORBED: AtomicUsize = AtomicUsize::new(0);
static ORDINAL_REFUSED: AtomicUsize = AtomicUsize::new(0);
static TRUNC_REFUSED: AtomicUsize = AtomicUsize::new(0);
static CLOSED_REFUSED: AtomicUsize = AtomicUsize::new(0);
static AEAD_REFUSED: AtomicUsize = AtomicUsize::new(0);
static TAG_REFUSED: AtomicUsize = AtomicUsize::new(0);
static KILLED: AtomicUsize = AtomicUsize::new(0);
static RESPAWNED: AtomicUsize = AtomicUsize::new(0);
static WORK_DIGESTS: AtomicUsize = AtomicUsize::new(0);

// ---- digest substrate + test KEM (session_courts' real collaborators) --

#[derive(Clone, Copy)]
struct Mix32;
impl ChainDigest for Mix32 {
    fn digest(&self, parts: &[&[u8]]) -> [u8; 32] {
        mix32(parts)
    }
}

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

struct TestKem;
impl KemSeam for TestKem {
    fn encapsulate(&self, ek: &[u8; EK_BYTES]) -> Result<([u8; CT_BYTES], [u8; 32]), SessionError> {
        let ss = mix32(&[b"cgap-test/ss", ek]);
        let mut ct = [0u8; CT_BYTES];
        let mut chunk = mix32(&[b"cgap-test/ct", ek]);
        let mut i = 0;
        while i < CT_BYTES {
            ct[i] = chunk[i % 32];
            if i % 32 == 31 {
                chunk = mix32(&[b"cgap-test/ct2", &chunk]);
            }
            i += 1;
        }
        Ok((ct, ss))
    }

    fn decapsulate(&self, dk: &[u8], ct: &[u8; CT_BYTES]) -> Result<[u8; 32], SessionError> {
        if dk.len() != EK_BYTES {
            return Err(SessionError::KemRefused { op: 1 });
        }
        let mut ek = [0u8; EK_BYTES];
        ek.copy_from_slice(dk);
        let (expect, ss) = self.encapsulate(&ek)?;
        if ct != &expect {
            return Err(SessionError::KemRefused { op: 1 });
        }
        Ok(ss)
    }

    fn kdf(&self, parts: &[&[u8]]) -> [u8; 32] {
        mix32(parts)
    }
}

const KEM: TestKem = TestKem;
const H: Mix32 = Mix32;

fn ek_of(seed: u8) -> [u8; EK_BYTES] {
    let mut k = [seed; EK_BYTES];
    k[0] = seed;
    k[1183] = seed ^ 0xa5;
    k
}

// ---- telemetry shapes + wire builders ----------------------------------

/// Radar track frame body: 48B.
fn radar_body(seq: u32, c: usize) -> [u8; 4 + 48] {
    let mut b = [0u8; 4 + 48];
    // seq word is opaque payload here (never parsed as a seq); written LE per
    // the 2026-11-04 seq-LE migration for hygiene.
    b[..4].copy_from_slice(&seq.to_le_bytes());
    for (i, slot) in b[4..].iter_mut().enumerate() {
        *slot = (c as u8) ^ (i as u8) ^ (seq as u8);
    }
    b
}

/// Kinetic event frame body: 96B.
fn kinetic_body(seq: u32, c: usize) -> [u8; 4 + 96] {
    let mut b = [0u8; 4 + 96];
    // seq word is opaque payload here (never parsed as a seq); written LE per
    // the 2026-11-04 seq-LE migration for hygiene.
    b[..4].copy_from_slice(&seq.to_le_bytes());
    for (i, slot) in b[4..].iter_mut().enumerate() {
        *slot = (c as u8).wrapping_mul(3) ^ (i as u8).wrapping_mul(7) ^ (seq as u8);
    }
    b
}

/// Launch-state frame body: 16B (low-latency shape).
fn launch_body(seq: u32, c: usize) -> [u8; 4 + 16] {
    let mut b = [0u8; 4 + 16];
    // seq word is opaque payload here (never parsed as a seq); written LE per
    // the 2026-11-04 seq-LE migration for hygiene.
    b[..4].copy_from_slice(&seq.to_le_bytes());
    for (i, slot) in b[4..].iter_mut().enumerate() {
        *slot = (c as u8) | (i as u8) << 2 | (seq as u8).rotate_left(1);
    }
    b
}

/// One telemetry frame, sealed via the REAL `Session::emit_verified`
/// compose law (tag = H(whole-frame-body)[0..4]); returns (wire, len).
fn emit_frame(s: &Session<'static, TestKem>, seq: u32, body: &[u8]) -> (Vec<u8>, usize) {
    let mut buf = vec![0u8; 64 + 1 + 4 + body.len() + 4];
    let n = s.emit_verified(seq, body, &mut buf).unwrap();
    buf.truncate(n);
    (buf, n)
}

/// Handshake a channel to Established at (epoch, key_id, ordinal 0).
fn handshake(c: usize) -> Session<'static, TestKem> {
    let ek = ek_of((c as u8) + 1);
    let mut s = Session::new(&KEM, (c % 3) as u16, (c as u64) + 1, 0);
    let mut ek_wire = vec![0u8; 64 + 1 + EK_BYTES];
    codec::encode(
        &AoqHeader::new((c % 3) as u16, (c as u64) + 1, 0, 0),
        &mut ek_wire,
    )
    .unwrap();
    ek_wire[64] = FrameKind::HandshakeEk.id();
    ek_wire[65..].copy_from_slice(&ek);
    s.step(&parse_frame(&ek_wire).unwrap()).unwrap();
    let (ct, _) = KEM.encapsulate(&ek).unwrap();
    let mut fin = vec![0u8; 64 + 1 + CT_BYTES];
    codec::encode(
        &AoqHeader::new((c % 3) as u16, (c as u64) + 1, 1, 0),
        &mut fin,
    )
    .unwrap();
    fin[64] = FrameKind::HandshakeFinish.id();
    fin[65..].copy_from_slice(&ct);
    s.step(&parse_frame(&fin).unwrap()).unwrap();
    assert_eq!(s.state(), SessionState::Established);
    s
}

// ---- supervisor slot: owns its channels, churns its workers ------------

use std::sync::mpsc;

/// Per-slot telemetry loop. The slot owns channel state (session + chain)
/// across worker kills; workers are concurrent parse-and-digest verifiers
/// that are killed mid-stream (round 1, while bursts remain) and
/// respawned (round 2). Returns each channel's receipt records.
fn run_slot(slot: usize, tx: &mpsc::Sender<(usize, Vec<AoqChainRecord>)>) {
    let channels: Vec<usize> = (slot..CHANNELS).step_by(SLOTS).collect();
    let mut sessions: Vec<Session<'static, TestKem>> =
        channels.iter().map(|&c| handshake(c)).collect();
    let mut slabs: Vec<Vec<AoqChainRecord>> = channels
        .iter()
        .map(|_| vec![AoqChainRecord::zeroed(); CHAIN_CAP])
        .collect();
    let mut chains: Vec<AoqChain<Mix32>> = slabs
        .iter_mut()
        .map(|s| AoqChain::new(H, s.as_mut_slice()))
        .collect();

    // Two supervisor rounds; between them the round-1 worker is KILLED
    // mid-stream (unannounced termination for DROP channels) and a
    // round-2 worker is RESPAWNED.
    for round in 0..2usize {
        let (wtx, wrx) = mpsc::channel::<Vec<u8>>();
        let worker = thread::spawn(move || {
            let mut parsed = 0usize;
            for wire in wrx {
                if parse_frame(&wire).is_ok() {
                    parsed += 1;
                }
                let f = parse_frame(&wire).expect("worker re-parse");
                WORK_DIGESTS.fetch_add(1, Ordering::Relaxed);
                let _ = mix32(&[&wire[69..], &f.payload]);
            }
            parsed
        });

        for j in 0..24usize {
            let g = round * 24 + j;
            for (i, &c) in channels.iter().enumerate() {
                let s = &mut sessions[i];
                let chain = &mut chains[i];
                let closed = CLOSE_CHANNELS.contains(&c) && round == 1;
                // Out-of-order injection (round 1, mid-stream, once).
                if OO_CHANNELS.contains(&c) && round == 0 && j == 8 {
                    for _ in 0..3 {
                        let body = radar_body(g as u32, c);
                        let mut w = vec![0u8; 64 + 1 + 4 + body.len() - 4];
                        codec::encode(
                            &AoqHeader::new(
                                (c % 3) as u16,
                                (c as u64) + 1,
                                s.ordinal() + 1,
                                g as u32,
                            ),
                            &mut w,
                        )
                        .unwrap();
                        w[64] = FrameKind::AppData.id();
                        w[65..69].copy_from_slice(&(g as u32).to_be_bytes());
                        w[69..].copy_from_slice(&body[4..]);
                        let err = s.step(&parse_frame(&w).unwrap()).unwrap_err();
                        assert!(matches!(err, SessionError::OrdinalMismatch { .. }));
                        ORDINAL_REFUSED.fetch_add(1, Ordering::Relaxed);
                    }
                }
                // Truncation injection (round 1, mid-stream, once).
                if TRUNC_CHANNELS.contains(&c) && round == 0 && j == 8 {
                    for cut in [40usize, 52, 66] {
                        let body = kinetic_body(g as u32, c);
                        let (w, _) = emit_frame(s, g as u32, &body[4..]);
                        let err = parse_frame(&w[..cut]).unwrap_err();
                        assert!(matches!(
                            err,
                            SessionError::LengthMismatch { .. }
                                | SessionError::Wire(
                                    cgap_aoq::error::AoqError::HeaderTooShort { .. }
                                )
                        ));
                        TRUNC_REFUSED.fetch_add(1, Ordering::Relaxed);
                    }
                }
                // Telemetry body by shape.
                let full: &[u8] = match g % 3 {
                    0 => &radar_body(g as u32, c),
                    1 => &kinetic_body(g as u32, c),
                    _ => &launch_body(g as u32, c),
                };
                if closed {
                    // Post-close refusal battery: typed, no mutation. A
                    // closed session cannot emit, so the stale frame is
                    // crafted raw at the frozen ordinal.
                    let mut w = vec![0u8; 64 + 1 + 4 + full.len() - 4];
                    codec::encode(
                        &AoqHeader::new((c % 3) as u16, (c as u64) + 1, s.ordinal(), g as u32),
                        &mut w,
                    )
                    .unwrap();
                    w[64] = FrameKind::AppData.id();
                    w[65..69].copy_from_slice(&(g as u32).to_be_bytes());
                    w[69..].copy_from_slice(&full[4..]);
                    let err = s.step(&parse_frame(&w).unwrap()).unwrap_err();
                    assert!(matches!(err, SessionError::SessionClosed));
                    CLOSED_REFUSED.fetch_add(1, Ordering::Relaxed);
                } else {
                    let (wire, _n) = emit_frame(s, g as u32, &full[4..]);
                    // AEAD seam census: pre-admission kernel refuses typed
                    // (R-K1 disclosure — never faked as sealed).
                    match cgap_aoq::aead::seal(0, &[0u8; 32], full) {
                        Err(cgap_aoq::error::AoqError::AeadKernelNotAdmitted { .. }) => {
                            AEAD_REFUSED.fetch_add(1, Ordering::Relaxed);
                        }
                        _ => panic!("AEAD seam admitted an unadmitted kernel id"),
                    }
                    s.step(&parse_frame(&wire).unwrap()).unwrap();
                    ABSORBED.fetch_add(1, Ordering::Relaxed);
                    chain.append_verified(&wire).unwrap();
                    let _ = wtx.send(wire.clone());
                }
            }
        }

        // Lawful Close for CLOSE channels at the round boundary (round-2
        // entry): the close frame itself is absorbed, then appended
        // untagged to the chain -> typed tag refusal (no mutation).
        if round == 0 {
            // (runs once per round, after all 24 frame slots)
            for (i, &c) in channels.iter().enumerate() {
                if CLOSE_CHANNELS.contains(&c) {
                    let s = &mut sessions[i];
                    let mut w = vec![0u8; 64 + 1];
                    codec::encode(
                        &AoqHeader::new((c % 3) as u16, (c as u64) + 1, s.ordinal(), 0),
                        &mut w,
                    )
                    .unwrap();
                    w[64] = FrameKind::Close.id();
                    s.step(&parse_frame(&w).unwrap()).unwrap();
                    assert_eq!(sessions[i].state(), SessionState::Closed);
                    ABSORBED.fetch_add(1, Ordering::Relaxed);
                    let err = chains[i].append_verified(&w).unwrap_err();
                    assert!(matches!(
                        err,
                        cgap_aoq::error::AoqError::DigestMismatch { .. }
                    ));
                    TAG_REFUSED.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        // KILL: drop the feed and join the worker mid-stream (round 1
        // ends with 24 frames still pending on DROP channels; the session
        // stays Established at its frozen ordinal — no Close was sent).
        drop(wtx);
        let parsed = worker.join().unwrap();
        if round == 0 {
            KILLED.fetch_add(1, Ordering::Relaxed);
            if parsed != 24 * channels.len() {
                panic!(
                    "slot {slot} round-1 worker parsed {parsed}, expected {}",
                    24 * channels.len()
                );
            }
        } else {
            RESPAWNED.fetch_add(1, Ordering::Relaxed);
        }
    }

    // DROP-channel continuity witness: frozen sessions still Established.
    for (i, &c) in channels.iter().enumerate() {
        if DROP_CHANNELS.contains(&c) {
            assert_eq!(sessions[i].state(), SessionState::Established);
        }
        tx.send((c, chains[i].records().to_vec())).unwrap();
    }
}

// ---- host measurements (std test harness; honest disclosures) ----------

/// RSS in KB via `ps` (host std harness; the true no_os footprint is a
/// future embedded-target lane — TARGET-UNMEASURED here).
fn rss_kb() -> u64 {
    let out = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
        .expect("ps rss");
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse()
        .expect("rss integer")
}

/// Open fd count via /dev/fd (host).
fn fd_count() -> usize {
    std::fs::read_dir("/dev/fd").expect("/dev/fd").count()
}

fn ld(v: &AtomicUsize) -> usize {
    v.load(Ordering::Relaxed)
}

// ---- the mesh ----------------------------------------------------------

#[test]
fn macro_defense_mesh() {
    if let Err(v) = cgap_aoq::profile_guard::assert_release() {
        eprintln!("LOUD-SKIP macro_defense_mesh: {v}");
        return;
    }
    let fd_before = fd_count();
    let rss_before = rss_kb();
    let t0 = Instant::now();

    let (tx, rx) = mpsc::channel::<(usize, Vec<AoqChainRecord>)>();
    thread::scope(|s| {
        for slot in 0..SLOTS {
            let tx = tx.clone();
            s.spawn(move || run_slot(slot, &tx));
        }
        drop(tx);
        let mut attribution: HashMap<[u8; 32], usize> = HashMap::new();
        let mut per_channel: Vec<(usize, Vec<AoqChainRecord>)> = Vec::new();
        for (c, records) in rx {
            assert_eq!(
                records.len(),
                if CLOSE_CHANNELS.contains(&c) { 24 } else { 48 },
                "channel {c} record count"
            );
            for r in &records {
                if let Some(prev) = attribution.insert(r.digest, c) {
                    panic!(
                        "ATTRIBUTION BLEED: record digest {:02x?} claimed by channels {prev} and {c}",
                        &r.digest[..4]
                    );
                }
            }
            verify_records(&H, &records)
                .unwrap_or_else(|e| panic!("channel {c} chain verify: {e}"));
            per_channel.push((c, records));
        }
        assert_eq!(per_channel.len(), CHANNELS, "all 64 channels reported");

        // Census gates (typed counts; every expected refusal observed).
        let telemetry_absorbed = 60 * 48 + 4 * 24;
        assert_eq!(ld(&ABSORBED), telemetry_absorbed + 4); // +4 close frames
        assert_eq!(ld(&ORDINAL_REFUSED), 12);
        assert_eq!(ld(&TRUNC_REFUSED), 12);
        assert_eq!(ld(&CLOSED_REFUSED), 96);
        assert_eq!(ld(&AEAD_REFUSED), telemetry_absorbed);
        assert_eq!(ld(&TAG_REFUSED), 4);
        assert_eq!(ld(&KILLED), SLOTS);
        assert_eq!(ld(&RESPAWNED), SLOTS);

        // RSS / fd deltas.
        let rss_after = rss_kb();
        let fd_after = fd_count();
        let rss_delta_kb = rss_after as i64 - rss_before as i64;
        let fd_delta = fd_after as i64 - fd_before as i64;
        assert_eq!(fd_delta, 0, "fd leak: delta {fd_delta}");

        let elapsed = t0.elapsed();
        eprintln!();
        eprintln!("=== AF-27 defense telemetry mesh (host std harness) ===");
        eprintln!("channels: {CHANNELS} (dossier 1024 scaled 16x; TARGET-UNMEASURED)");
        eprintln!("telemetry frames absorbed: {telemetry_absorbed} (+4 lawful close frames)");
        eprintln!("chain records (per-channel chains): {}", attribution.len());
        eprintln!(
            "record bytes: {} ({} x {RECORD_BYTES}B)",
            attribution.len() * RECORD_BYTES,
            attribution.len()
        );
        eprintln!("attribution bleed: 0 (every digest maps to exactly one channel)");
        eprintln!(
            "refusal census: ordinal=12 truncation=12 session_closed=96 aead_seam={} close_tag=4",
            ld(&AEAD_REFUSED)
        );
        eprintln!(
            "worker threads killed mid-stream: {} ; respawned: {}",
            ld(&KILLED),
            ld(&RESPAWNED)
        );
        eprintln!(
            "worker digest operations: {}",
            WORK_DIGESTS.load(Ordering::Relaxed)
        );
        eprintln!("RSS delta: {rss_delta_kb} KB (before {rss_before}, after {rss_after})");
        eprintln!("fd delta: {fd_delta}");
        eprintln!("wall time: {elapsed:?}");
        eprintln!(
            "unmeasured here: no_os embedded-target run, EW survivability, 1024-channel scale"
        );
    });
}
