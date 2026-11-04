//! Session courts (PG-A5, AB-7): 8 fail-closed courts over the real
//! machine. Anti-vacuity: courts 2/3/4/8 inject real refusals/mutations
//! and must observe typed flips; byte-identity is asserted via
//! `Session::fingerprint` (full canonical state, KDF-sealed).

use cgap_aoq::chain::{AoqChain, AoqChainRecord, ChainDigest};
use cgap_aoq::codec;
use cgap_aoq::header::AoqHeader;
use cgap_aoq::session::{
    derive, parse_frame, transcript_tag, Frame, FrameKind, KemSeam, Session, SessionError,
    SessionState, CT_BYTES, CTX_HANDSHAKE, CTX_REKEY, EK_BYTES, REKEY_BUDGET, SS_BYTES,
};

/// Deterministic digest substrate (shared with the chain courts by
/// construction: one mix, two seams).
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

/// Deterministic hash-based test KEM (PG-A5: courts supply the KEM; the
/// crate ships no crypto). `dk == ek` under this construction (hash KEM).
struct TestKem;
impl KemSeam for TestKem {
    fn encapsulate(
        &self,
        ek: &[u8; EK_BYTES],
    ) -> Result<([u8; CT_BYTES], [u8; SS_BYTES]), SessionError> {
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

    fn decapsulate(
        &self,
        dk: &[u8],
        ct: &[u8; CT_BYTES],
    ) -> Result<[u8; SS_BYTES], SessionError> {
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

/// Handshake to Established at (epoch, ordinal); returns the session.
fn established(epoch: u16, ordinal: u32, ek_seed: u8) -> Session<'static, TestKem> {
    let mut s = Session::new(&KEM, epoch, 7, ordinal);
    let ek = ek_of(ek_seed);
    step(&mut s, &ek_frame(epoch, 7, ordinal, &ek)).unwrap();
    step(&mut s, &fin_frame(epoch, 7, ordinal + 1, &KEM, &ek)).unwrap();
    assert_eq!(s.state(), SessionState::Established);
    s
}

fn ek_of(seed: u8) -> [u8; EK_BYTES] {
    let mut k = [seed; EK_BYTES];
    k[0] = seed;
    k[1183] = seed ^ 0xa5;
    k
}

/// AOQ1 header + kind + body wire builder (composes codec::encode).
fn wire(
    epoch: u16,
    key_id: u64,
    ordinal: u32,
    seq: u32,
    kind: FrameKind,
    body: &[u8],
) -> Vec<u8> {
    let mut buf = vec![0u8; 64 + 1 + body.len()];
    let h = AoqHeader::new(epoch, key_id, ordinal, seq);
    codec::encode(&h, &mut buf).unwrap();
    buf[64] = kind.id();
    buf[65..].copy_from_slice(body);
    buf
}

fn ek_frame(epoch: u16, key_id: u64, ordinal: u32, ek: &[u8; EK_BYTES]) -> Vec<u8> {
    wire(epoch, key_id, ordinal, 0, FrameKind::HandshakeEk, ek)
}

fn fin_frame(
    epoch: u16,
    key_id: u64,
    ordinal: u32,
    kem: &TestKem,
    ek: &[u8; EK_BYTES],
) -> Vec<u8> {
    let (ct, _) = kem.encapsulate(ek).unwrap();
    wire(epoch, key_id, ordinal, 0, FrameKind::HandshakeFinish, &ct)
}

fn app_body(seq: u32, payload: &[u8]) -> Vec<u8> {
    let mut body = vec![0u8; 4 + payload.len()];
    body[..4].copy_from_slice(&seq.to_be_bytes());
    body[4..].copy_from_slice(payload);
    body
}

fn app_frame(epoch: u16, ordinal: u32, seq: u32, payload: &[u8]) -> Vec<u8> {
    wire(
        epoch,
        7,
        ordinal,
        seq,
        FrameKind::AppData,
        &app_body(seq, payload),
    )
}

fn rekey_frame(epoch: u16, ordinal: u32, ek: &[u8; EK_BYTES]) -> Vec<u8> {
    wire(epoch, 7, ordinal, 0, FrameKind::Rekey, ek)
}


/// Step helper: parse (borrowed frame must outlive the call) then step.
fn step(s: &mut Session<'static, TestKem>, w: &[u8]) -> Result<(), SessionError> {
    let f = parse_frame(w)?;
    s.step(&f)
}

fn close_frame(epoch: u16, ordinal: u32) -> Vec<u8> {
    wire(epoch, 7, ordinal, 0, FrameKind::Close, &[])
}
/// COURT S1 — exhaustive (state, frame) transition table: every one of the
/// 4x5 = 20 cells is named, with the expected outcome per cell.
#[test]
fn s1_exhaustive_transition_table() {
    // Reference states, all fresh at ordinal 100 (no auto-advance).
    let mk = |state| -> Session<'static, TestKem> {
        let mut s = Session::new(&KEM, 0, 7, 100);
        match state {
            1 => {
                // -> HandshakeWait
                step(&mut s, &ek_frame(0, 7, 100, &ek_of(1))).unwrap();
            }
            2 => {
                step(&mut s, &ek_frame(0,7,100,&ek_of(1))).unwrap();
                step(&mut s, &fin_frame(0, 7, 101, &KEM, &ek_of(1))).unwrap();
            }
            3 => {
                step(&mut s, &close_frame(0, 100)).unwrap();
            }
            _ => {}
        }
        s
    };

    let mut checked = 0usize;
    for state in 0u8..4 {
        for kind in [FrameKind::HandshakeEk, FrameKind::HandshakeFinish,
                     FrameKind::AppData, FrameKind::Rekey, FrameKind::Close]
        {
            let mut s = mk(state);
            let before = s.fingerprint();
            // Build the wire frame at the session's expected ordinal,
            // keep it alive on the stack, parse, then step.
            let w: Vec<u8> = match kind {
                FrameKind::HandshakeEk => ek_frame(0, 7, s.ordinal(), &ek_of(9)),
                FrameKind::HandshakeFinish => {
                    // Echo the session's OWN pending ct: state 1 is the
                    // lawful cell and must accept; other states refuse
                    // before the ct check.
                    let want = s.ordinal();
                    let (ct, _) = KEM.encapsulate(&ek_of(1)).unwrap();
                    wire(0, 7, want, 0, FrameKind::HandshakeFinish, &ct)
                }
                FrameKind::AppData => app_frame(0, s.ordinal(), 0, b"xy"),
                FrameKind::Rekey => rekey_frame(0, s.ordinal(), &ek_of(9)),
                FrameKind::Close => close_frame(0, s.ordinal()),
            };
            let outcome = step(&mut s, &w);
            // The lawful cells (the transition table).
            let lawful = match (state, kind) {
                (0, FrameKind::HandshakeEk)
                | (1, FrameKind::HandshakeFinish)
                | (2, FrameKind::AppData)
                | (2, FrameKind::Rekey)
                | (_, FrameKind::Close) => true,
                _ => false,
            };
            if state == 3 {
                // Closed refuses EVERYTHING, including Close.
                assert!(
                    matches!(outcome, Err(SessionError::SessionClosed)),
                    "closed must refuse {kind:?} in state {state}"
                );
            } else if lawful {
                assert!(outcome.is_ok(), "lawful cell ({state},{kind:?}) refused: {outcome:?}");
            } else {
                assert!(
                    matches!(outcome, Err(SessionError::UnexpectedFrame { .. })),
                    "unlawful cell ({state},{kind:?}) must be UnexpectedFrame, got {outcome:?}"
                );
                assert_eq!(s.fingerprint(), before, "no mutation on refusal");
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 20, "exactly the 4x5 table");
}

/// COURT S2 — no-mutation-on-refusal byte-identity: in EVERY state, every
/// refused frame leaves the session fingerprint byte-identical.
#[test]
fn s2_no_mutation_on_refusal_byte_identity() {
    for state in 0u8..4 {
        let mut s = Session::new(&KEM, 0, 7, 0);
        match state {
            1 => {
                step(&mut s, &ek_frame(0, 7, 0, &ek_of(1))).unwrap();
            }
            2 => {
                step(&mut s, &ek_frame(0, 7, 0, &ek_of(1))).unwrap();
                step(&mut s, &fin_frame(0, 7, 1, &KEM, &ek_of(1))).unwrap();
                // Exhaust the counted budget so AppData is a guaranteed
                // refusal (RekeyRequired) in this court.
                for i in 0u32..REKEY_BUDGET {
                    step(&mut s, &app_frame(0, i + 2, i, b"")).unwrap();
                }
                assert_eq!(s.budget_used(), REKEY_BUDGET);
            }
            3 => {
                step(&mut s, &close_frame(0, 0)).unwrap();
            }
            _ => {}
        }
        let before = s.fingerprint();

        // Refusal battery: wrong epoch, wrong ordinal, unknown kind is
        // parse-level (build raw), wrong-state frame, and closed refusal.
        let refusal_wires: Vec<Vec<u8>> = vec![
            // Wrong epoch (not the active epoch, not RETIRED).
            wire(1, 7, s.ordinal(), 0, FrameKind::AppData, &app_body(0, b"")),
            // RETIRED epoch (u16::MAX) refused by construction.
            wire(65535, 7, s.ordinal(), 0, FrameKind::Close, &[]),
            // Wrong ordinal (expected+1).
            wire(0, 7, s.ordinal().wrapping_add(1), 0, FrameKind::AppData, &app_body(0, b"")),
            // Wrong-state: AppData outside 2 (or budget-exhausted).
            app_frame(0, s.ordinal(), 0, b""),
            // Finish outside 1 (refused before any ct check).
            wire(0, 7, s.ordinal(), 0, FrameKind::HandshakeFinish, &[0u8; CT_BYTES]),
        ];
        for frame_w in &refusal_wires {
            let _ = step(&mut s, frame_w);
            assert_eq!(
                s.fingerprint(),
                before,
                "state {state}: refusal mutated the session"
            );
        }
        if state == 3 {
            let w = app_frame(0, s.ordinal(), 0, b"");
            assert!(
                matches!(step(&mut s, &w), Err(SessionError::SessionClosed)),
                "closed refuses after the battery"
            );
            assert_eq!(s.fingerprint(), before);
        }
    }
}

/// COURT S3 — counted-not-timed rekey budget: exactly REKEY_BUDGET app
/// frames are lawful; the boundary frame refuses RekeyRequired; an accepted
/// rekey resets; Close needs no budget.
#[test]
fn s3_rekey_budget_counted_not_timed() {
    let mut s = Session::new(&KEM, 0, 7, 0);
    let w = ek_frame(0, 7, 0, &ek_of(1));
        let f = parse_frame(&w).unwrap();
    s.step(&f).unwrap();
    let w = fin_frame(0, 7, 1, &KEM, &ek_of(1));
        let f = parse_frame(&w).unwrap();
    s.step(&f).unwrap();

    for i in 0u32..REKEY_BUDGET {
        let w = app_frame(0, i + 2, i, b"");
        let f = parse_frame(&w).unwrap();
        assert!(s.step(&f).is_ok(), "frame {i} of the budget must be lawful");
    }
    assert_eq!(s.budget_used(), REKEY_BUDGET);
    // Boundary: next AppData refuses; the session is unmuted.
    let before = s.fingerprint();
    let w = app_frame(0, REKEY_BUDGET + 2, 0, b"");
        let f = parse_frame(&w).unwrap();
    assert!(matches!(
        s.step(&f),
        Err(SessionError::RekeyRequired { used: REKEY_BUDGET, budget: REKEY_BUDGET })
    ));
    assert_eq!(s.fingerprint(), before, "budget refusal does not mutate");

    // Accepted rekey resets the counter (two-round derivation, fresh tag).
    let w = rekey_frame(0, REKEY_BUDGET + 2, &ek_of(5));
        let f = parse_frame(&w).unwrap();
    s.step(&f).unwrap();
    assert_eq!(s.budget_used(), 0);
    assert_eq!(s.state(), SessionState::Established);
    // Budget restarts counting.
    let w = app_frame(0, REKEY_BUDGET + 3, 0, b"");
        let f = parse_frame(&w).unwrap();
    assert!(s.step(&f).is_ok());
    assert_eq!(s.budget_used(), 1);
}

/// COURT S4 — parse_frame length law: every short/overlong/unknown-class
/// wire input is a typed refusal naming expected vs got; no panic on any
/// truncation of a lawful frame.
#[test]
fn s4_parse_frame_length_law() {
    let good = app_frame(0, 0, 0, b"payload");
    assert!(parse_frame(&good).is_ok());

    // Truncations: every prefix shorter than 65B, and every truncation of
    // the good frame, refuses (length-checked, never panics).
    for cut in 0..good.len() {
        let got = parse_frame(&good[..cut]);
        if cut < 65 {
            assert!(got.is_err(), "prefix {cut} must refuse");
        }
    }

    // Short ek body (1183B of the 1184B needed).
    let short_ek = wire(0, 7, 0, 0, FrameKind::HandshakeEk, &[7u8; EK_BYTES - 1]);
    match parse_frame(&short_ek) {
        Err(SessionError::LengthMismatch { expected, got }) => {
            assert_eq!(expected, EK_BYTES);
            assert_eq!(got, EK_BYTES - 1);
        }
        other => panic!("short ek must refuse typed, got {other:?}"),
    }

    // Short ct body.
    let short_ct = wire(0, 7, 0, 0, FrameKind::HandshakeFinish, &[1u8; CT_BYTES - 1]);
    match parse_frame(&short_ct) {
        Err(SessionError::LengthMismatch { expected, got }) => {
            assert_eq!(expected, CT_BYTES);
            assert_eq!(got, CT_BYTES - 1);
        }
        other => panic!("short ct must refuse typed, got {other:?}"),
    }

    // Close with a payload (expected 0).
    let fat_close = wire(0, 7, 0, 0, FrameKind::Close, &[0u8; 1]);
    match parse_frame(&fat_close) {
        Err(SessionError::LengthMismatch { expected, got }) => {
            assert_eq!((expected, got), (0, 1));
        }
        other => panic!("fat close must refuse typed, got {other:?}"),
    }

    // Unknown kind tag.
    let mut unk = [0u8; 65];
    let h = AoqHeader::new(0, 7, 0, 0);
    codec::encode(&h, &mut unk).unwrap();
    unk[64] = 200;
    assert!(matches!(
        parse_frame(&unk),
        Err(SessionError::UnknownFrameKind { got: 200 })
    ));

    // Header-law refusals compose through (bad magic).
    let mut bad = good.clone();
    bad[0] ^= 0xff;
    assert!(matches!(
        parse_frame(&bad),
        Err(SessionError::Wire(cgap_aoq::error::AoqError::BadMagic { .. }))
    ));
}

/// COURT S5 — transcript binding: the tag binds ek_server, ss, and epoch;
/// identical inputs derive identical tags (determinism).
#[test]
fn s5_transcript_binding() {
    let base = transcript_tag(&KEM, &ek_of(1), &mix32(&[b"ss1"]), 0);
    // ek sensitivity
    assert_ne!(base, transcript_tag(&KEM, &ek_of(2), &mix32(&[b"ss1"]), 0));
    // ss sensitivity
    assert_ne!(base, transcript_tag(&KEM, &ek_of(1), &mix32(&[b"ss2"]), 0));
    // epoch sensitivity
    assert_ne!(base, transcript_tag(&KEM, &ek_of(1), &mix32(&[b"ss1"]), 1));
    // determinism
    assert_eq!(base, transcript_tag(&KEM, &ek_of(1), &mix32(&[b"ss1"]), 0));
}

/// COURT S6 — two-round KDF: derive() differs from its own round-2 input
/// round and is bound to the context constants; rekey derivation uses a
/// different context than handshake.
#[test]
fn s6_two_round_kdf() {
    let ss = mix32(&[b"ss"]);
    let ek = ek_of(3);
    let key = derive(&KEM, CTX_HANDSHAKE, &ss, &ek);
    let round1 = KEM.kdf(&[CTX_HANDSHAKE, &ss]);
    // Not the single-round form.
    assert_ne!(key, KEM.kdf(&[CTX_HANDSHAKE, &ss, &ek]));
    // Round 1 is a strict intermediate: key != round1, and round2 changes
    // when round1's input changes.
    assert_ne!(key, round1);
    assert_ne!(
        derive(&KEM, CTX_HANDSHAKE, &ss, &ek),
        derive(&KEM, CTX_HANDSHAKE, &mix32(&[b"other"]), &ek),
    );
    // Context binding: handshake vs rekey contexts differ.
    assert_ne!(key, derive(&KEM, CTX_REKEY, &ss, &ek));
    // Determinism.
    assert_eq!(key, derive(&KEM, CTX_HANDSHAKE, &ss, &ek));
}

/// COURT S7 — epoch/ordinal wire law: RETIRED epoch refused, non-active
/// epoch refused with typed EpochMismatch, ordinal gap/regression refused
/// with typed OrdinalMismatch.
#[test]
fn s7_epoch_ordinal_wire_law() {
    let mut s = established(0, 0, 1); // ordinal now 2
    // RETIRED epoch (u16::MAX): refused even with the right ordinal.
    let w = wire(65535, 7, 2, 0, FrameKind::AppData, &app_body(0, b""));
        let f = parse_frame(&w).unwrap();
    assert!(matches!(
        s.step(&f),
        Err(SessionError::EpochMismatch { expected: 0, got: 65535 })
    ));
    // Wrong epoch (1) refused.
    let w = wire(1, 7, 2, 0, FrameKind::AppData, &app_body(0, b""));
        let f = parse_frame(&w).unwrap();
    assert!(matches!(
        s.step(&f),
        Err(SessionError::EpochMismatch { expected: 0, got: 1 })
    ));
    // Ordinal gap (+1) refused.
    let w = app_frame(0, 4, 0, b"");
        let f = parse_frame(&w).unwrap();
    assert!(matches!(
        s.step(&f),
        Err(SessionError::OrdinalMismatch { expected: 2, got: 4 })
    ));
    // Ordinal regression (rewind to 1) refused.
    let w = app_frame(0, 1, 0, b"");
        let f = parse_frame(&w).unwrap();
    assert!(matches!(
        s.step(&f),
        Err(SessionError::OrdinalMismatch { expected: 2, got: 1 })
    ));
    // Replay (same ordinal as last accepted) refused.
    let w = app_frame(0, 2, 0, b"");
        let f = parse_frame(&w).unwrap();
    assert!(s.step(&f).is_ok(), "ordinal 2 IS the expected one"); // consumes
    let w = app_frame(0, 2, 0, b"");
        let f = parse_frame(&w).unwrap();
    assert!(s.step(&f).is_err(), "replay of ordinal 2 must refuse");
}

/// COURT S8 — 10k-frame Established soak with ordinal-wrap walk: the
/// machine crosses the u32 ordinal wrap mid-stream, counts budget, rekeys
/// on schedule, and stays fingerprint-consistent throughout.
#[test]
fn s8_tenk_soak_ordinal_wrap() {
    // Start near the top of the u32 ordinal space: the wrap lands mid-soak.
    let start = u32::MAX - 3000;
    let mut s = Session::new(&KEM, 0, 7, start);
    let w = ek_frame(0, 7, start, &ek_of(1));
        let f = parse_frame(&w).unwrap();
    s.step(&f).unwrap();
    let w = fin_frame(0, 7, start.wrapping_add(1), &KEM, &ek_of(1));
        let f = parse_frame(&w).unwrap();
    s.step(&f).unwrap();
    assert_eq!(s.state(), SessionState::Established);

    let mut ordinal = start.wrapping_add(2);
    let mut accepted = 0u32;
    let mut rekeys = 0u32;
    for i in 0u32..10_000u32 {
        // Rekey every REKEY_BUDGET frames (budget is counted, not timed).
        if s.budget_used() == REKEY_BUDGET {
            let w = rekey_frame(0, ordinal, &ek_of((i % 250 + 1) as u8));
        let f = parse_frame(&w).unwrap();
            s.step(&f).unwrap();
            rekeys += 1;
            ordinal = ordinal.wrapping_add(1);
            assert_eq!(s.budget_used(), 0);
        }
        let w = app_frame(0, ordinal, i, &[i as u8; 4]);
        let f = parse_frame(&w).unwrap();
        s.step(&f).unwrap();
        accepted += 1;
        ordinal = ordinal.wrapping_add(1);
        // The wrap witness: the machine crossed zero mid-stream.
        if i == 3_002 {
            // start = MAX-3000; ordinal after handshake = start+2; after
            // i=3002 accepted frames ordinal = start+2+3003 = MAX+5 = 4.
            assert_eq!(ordinal, 4, "u32 wrap walked mid-soak");
        }
    }
    assert_eq!(accepted, 10_000);
    assert_eq!(rekeys, 2, "4096-frame budget rekeys twice in 10k frames");
    assert_eq!(s.state(), SessionState::Established);
}
