//! CONFORMANCE COURT (PL16 items 2-3, OCEL-side lane): drive the REAL
//! 4x5 AoQ session machine (real collaborator, Chicago law), tap ONLY
//! the `Ok(())` arm of `step` into a [`TapSink`], then replay the
//! sidecar chain through the tap module's oracle.
//!
//! Oracle source-of-truth: `cgap-aoq/src/session.rs:453-519`
//! (`Session::step`'s match arms). The tap module re-derives the table
//! (`tap::lawful_step`) WITH a cross-check comment citing that span;
//! this court additionally pins the oracle against the LIVE machine:
//! the walk below is driven by real `step` calls, so a drifted oracle
//! cannot pass this file and a drifted machine cannot pass `lawful_step`
//! silently (the flip legs at the bottom prove it).
//!
//! LAWS ASSERTED (all on final state, no mocks, no sleeps):
//! - every replayed row is a lawful (from, kind) -> to cell,
//! - ordinal advances by EXACTLY 1 per row (purity law as replay
//!   invariant: refusals emit nothing and advance nothing),
//! - refusal frames mint NO row and NO chain link (purity at the
//!   sink boundary; witnessed live below),
//! - event_seq (the ordinal) is zero-gap across the whole walk,
//! - counted budget: AppData +1, Rekey reset, handshake/close hold,
//! - epoch + ordinal are the logical clock; no wall clock anywhere.
//!
//! ANTI-VACUITY: `impossible_row_flips_the_court` is THE FLIP — one
//! impossible row (Established + HandshakeEk) injected into a copy of
//! the lawful walk must flip the replay to a typed refusal (negative
//! control first: the pristine walk replays OK, so an always-refusing
//! court is a VACUOUS GATE, not a pass). The source-level /tmp sandbox
//! tripwire (oracle weakened to accept the impossible cell -> the
//! pristine leg itself must fail) was run this session with a
//! sha256-exact restore; see the lane receipt.

use cgap_aoq::aead::{self, Nonce96, TAG_LEN};
use cgap_aoq::codec;
use cgap_aoq::header::AoqHeader;
use cgap_aoq::session::{
    confirm_tag, derive, parse_frame, transcript_tag, CTX_HANDSHAKE, CTX_RECV, FrameKind, KemSeam,
    Session, SessionError, CT_BYTES, EK_BYTES,
};
use crate::tap::{lawful_step, TapEvent, TapRefusal, TapSink};
use cgap_p28::attestation::{
    P28LicenseAttestation, ATTESTATION_VERSION, FLAG_ROOT, LICENSE_MAGIC,
};

// ---------------------------------------------------------------------------
// Real collaborators: deterministic test KEM + real wire frames (the same
// shape cgap-aoq's own session courts drive).
// ---------------------------------------------------------------------------

fn mix32(parts: &[&[u8]]) -> [u8; 32] {
    let mut a: u64 = 0xcbf2_9ce4_8422_2325;
    let mut b: u64 = 0x9e37_79b9_7f4a_7c15;
    for part in parts {
        for &byte in *part {
            a ^= u64::from(byte);
            a = a.wrapping_mul(0x100_0000_01b3);
            b = (b ^ (u64::from(byte) | 1)).wrapping_mul(0x9e37_79b9_7f4a_7c15);
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
    fn encapsulate(
        &self,
        ek: &[u8; EK_BYTES],
    ) -> Result<([u8; CT_BYTES], [u8; 32]), SessionError> {
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
    ) -> Result<[u8; 32], SessionError> {
        if dk.len() != EK_BYTES {
            return Err(SessionError::KemRefused { op: 1 });
        }
        let mut ek = [0u8; EK_BYTES];
        ek.copy_from_slice(dk);
        let (ct_expect, ss) = self.encapsulate(&ek)?;
        if ct != &ct_expect {
            return Err(SessionError::KemRefused { op: 1 });
        }
        Ok(ss)
    }

    fn kdf(&self, parts: &[&[u8]]) -> [u8; 32] {
        mix32(parts)
    }
}

fn ek(seed: u8) -> [u8; EK_BYTES] {
    let mut k = [seed; EK_BYTES];
    k[0] = seed;
    k[1183] = seed ^ 0xa5;
    k
}

/// HandshakeEk frame: `[AOQ1 header 64B][kind 1B][ek 1184B]`.
fn handshake_frame(ek_bytes: &[u8; EK_BYTES], ordinal: u32, epoch: u16) -> Vec<u8> {
    let mut buf = vec![0u8; 64 + 1 + EK_BYTES];
    let h = AoqHeader::new(epoch, 0, ordinal, 0);
    codec::encode(&h, &mut buf).expect("header encode");
    buf[64] = FrameKind::HandshakeEk.id();
    buf[65..].copy_from_slice(ek_bytes);
    buf
}

/// HandshakeFinish frame (v3 wire law, M2a): body = ct echo of the peer's
/// pending ciphertext (1088B) PLUS the 16B confirm tag binding
/// `recv_key ‖ transcript ‖ ct_echo` — total wire width
/// `64 + 1 + 1104` (KD-COVERS: cgap-ocel2 fixture drift, v2-era 1088B
/// Finish frames refused under the v3 law).
fn finish_frame(peer_ek: &[u8; EK_BYTES], ordinal: u32, epoch: u16) -> Vec<u8> {
    let (ct, ss) = TestKem.encapsulate(peer_ek).expect("kem");
    // Recompute the direction-split recv key the session machine derived
    // at the Ek step (same deterministic TestKem, same inputs).
    let transcript = transcript_tag(&TestKem, peer_ek, &ss, epoch);
    let hs_master = derive(&TestKem, CTX_HANDSHAKE, &ss, peer_ek);
    let recv_key = TestKem.kdf(&[CTX_RECV, &hs_master]);
    let tag = confirm_tag(&TestKem, &recv_key, &transcript, &ct);
    let mut buf = vec![0u8; 64 + 1 + CT_BYTES + 16];
    let h = AoqHeader::new(epoch, 0, ordinal, 0);
    codec::encode(&h, &mut buf).expect("header encode");
    buf[64] = FrameKind::HandshakeFinish.id();
    buf[65..65 + CT_BYTES].copy_from_slice(&ct);
    buf[65 + CT_BYTES..].copy_from_slice(&tag);
    buf
}

/// AppData frame (v3 AEAD-sealed wire law): body = `[seq LE 4B][ct][tag
/// 16B]`, sealed under the session's recv_key (the fixture is a self-talk
/// walk: one session both mints and verifies, so the lawful seal key is
/// the session's own recv_key), nonce `wire_nonce(ordinal, seq)`, AAD =
/// the covered span `buf[..69]` — exactly what `verify_appdata` checks.
fn app_frame(
    seq: u32,
    ordinal: u32,
    epoch: u16,
    recv_key: &[u8; 32],
) -> Vec<u8> {
    // 8-byte plaintext payload (the old fixture's payload width).
    let payload = seq.to_le_bytes();
    let mut buf = vec![0u8; 64 + 1 + 4 + payload.len() + TAG_LEN];
    let h = AoqHeader::new(epoch, 0, ordinal, seq);
    codec::encode(&h, &mut buf).expect("header encode");
    buf[64] = FrameKind::AppData.id();
    buf[65..69].copy_from_slice(&seq.to_le_bytes());
    let nonce = Nonce96::new(u32::from(epoch), (u64::from(ordinal) << 32) | u64::from(seq))
        .expect("u16 epoch never widens to the reserved u32 epoch");
    let (covered, tail) = buf.split_at_mut(69);
    let (ct, rest) = tail.split_at_mut(payload.len());
    let (tag_out, _) = rest.split_at_mut(TAG_LEN);
    let mut tag = [0u8; TAG_LEN];
    aead::seal_detached(ct, &mut tag, &payload, covered, recv_key, &nonce);
    tag_out.copy_from_slice(&tag);
    buf
}

/// Rekey frame: fresh encapsulation key payload.
fn rekey_frame(seed: u8, ordinal: u32, epoch: u16) -> Vec<u8> {
    let mut buf = vec![0u8; 64 + 1 + EK_BYTES];
    let h = AoqHeader::new(epoch, 0, ordinal, 0);
    codec::encode(&h, &mut buf).expect("header encode");
    buf[64] = FrameKind::Rekey.id();
    buf[65..].copy_from_slice(&ek(seed));
    buf
}

/// Close frame: empty body.
fn close_frame(ordinal: u32, epoch: u16) -> Vec<u8> {
    let mut buf = vec![0u8; 64 + 1];
    let h = AoqHeader::new(epoch, 0, ordinal, 0);
    codec::encode(&h, &mut buf).expect("header encode");
    buf[64] = FrameKind::Close.id();
    buf
}

fn root_license() -> P28LicenseAttestation {
    P28LicenseAttestation {
        license_magic: LICENSE_MAGIC,
        version: ATTESTATION_VERSION,
        attestation_flags: FLAG_ROOT,
        safe_harbor_ceiling_cents: 1_000_000_000,
        origin_authority_digest: [7u8; 32],
        upstream_receipt: [0u8; 32],
    }
}

/// Tap a frame through the REAL machine: on `Ok(())` mint a TapEvent from
/// the post-step session (via X6's mint contract) and sink it; on `Err`
/// mint and sink NOTHING (the caller-side law, honored verbatim here).
fn tap_step(
    sink: &mut TapSink,
    session: &Session<'_, TestKem>,
    kind: FrameKind,
    license: &P28LicenseAttestation,
) {
    let x6 = cgap_ffi::aoq_events::AoqEvent {
        ordinal: session.ordinal(),
        state: session.state().id(),
        kind: kind.id(),
        epoch: session.epoch(),
        budget: session.budget_used(),
        fingerprint: session.fingerprint(),
    };
    let row = TapEvent::from_x6(&cgap_ffi::aoq_events::AoqEventRow {
        ordinal: x6.ordinal,
        state: x6.state,
        kind: x6.kind,
        epoch: x6.epoch,
        budget: x6.budget,
        fingerprint: x6.fingerprint,
        reserved: [0u8; 20],
    });
    sink.append(&row, license).expect("sink append");
}

/// THE LAWFUL WALK (the court fixture): HandshakeInit -> Ek -> Wait ->
/// Finish -> Established -> AppData x2 -> Rekey -> AppData -> Close.
/// Drives the REAL session machine; taps only accepted steps.
fn lawful_walk() -> TapSink {
    let kem = TestKem;
    let license = root_license();
    let mut sink = TapSink::new();
    let mut s = Session::new(&kem, 1, 0xDEAD_BEEF, 0);
    let peer = ek(1);

    // Ek: HandshakeInit -> HandshakeWait (ordinal 0)
    let wire = handshake_frame(&peer, 0, 1);
    s.step(&parse_frame(&wire).expect("parse")).expect("lawful ek step");
    tap_step(&mut sink, &s, FrameKind::HandshakeEk, &license);

    // Finish: HandshakeWait -> Established (ordinal 1)
    let wire = finish_frame(&peer, 1, 1);
    s.step(&parse_frame(&wire).expect("parse")).expect("lawful finish step");
    tap_step(&mut sink, &s, FrameKind::HandshakeFinish, &license);

    // AppData x2 (ordinals 2, 3)
    for seq in 0..2u32 {
        let wire = app_frame(seq, s.ordinal(), 1, &s.recv_key());
        s.step(&parse_frame(&wire).expect("parse")).expect("lawful app step");
        tap_step(&mut sink, &s, FrameKind::AppData, &license);
    }

    // Rekey (ordinal 4): budget resets to 0
    let wire = rekey_frame(0x42, s.ordinal(), 1);
    s.step(&parse_frame(&wire).expect("parse")).expect("lawful rekey step");
    tap_step(&mut sink, &s, FrameKind::Rekey, &license);

    // AppData (ordinal 5): budget back to 1
    let wire = app_frame(9, s.ordinal(), 1, &s.recv_key());
    s.step(&parse_frame(&wire).expect("parse")).expect("lawful app step");
    tap_step(&mut sink, &s, FrameKind::AppData, &license);

    // Close (ordinal 6): Established -> Closed
    let wire = close_frame(s.ordinal(), 1);
    s.step(&parse_frame(&wire).expect("parse")).expect("lawful close step");
    tap_step(&mut sink, &s, FrameKind::Close, &license);

    sink
}

// ---------------------------------------------------------------------------
// Court legs
// ---------------------------------------------------------------------------

/// NEGATIVE CONTROL + BASELINE: the pristine lawful walk replays OK;
/// every row is a lawful cell; the ordinal (event_seq) is zero-gap.
/// An always-refusing replay is a VACUOUS GATE, not a pass.
#[test]
fn pristine_walk_replays_lawful() {
    let sink = lawful_walk();
    assert_eq!(sink.len(), 7, "7 accepted steps in the walk");
    let license = root_license();

    // Full replay: chain verify + row binding + oracle walk. THE GATE.
    sink.replay(&license).expect("pristine walk must replay lawful");

    // Zero-gap event_seq visible on the rows themselves.
    let ords: Vec<u32> = sink.rows().iter().map(|r| r.ordinal).collect();
    // Post-advance ordinals from a fresh session: 1..=7, zero-gap.
    assert_eq!(ords, vec![1, 2, 3, 4, 5, 6, 7], "zero-gap event_seq");

    // The from-state sequence the oracle walk derives: 0 -> 1 -> 2 x4 -> 3.
    let mut from = 0u8;
    let tos: Vec<(u8, u8, u8)> = sink
        .rows()
        .iter()
        .map(|r| {
            let to = lawful_step(from, r.kind).expect("court fixture rows are lawful");
            let cell = (from, r.kind, to);
            assert_eq!(r.state, to, "row to-state matches the live machine");
            from = to;
            cell
        })
        .collect();
    assert_eq!(
        tos,
        vec![
            (0, 0, 1), // Ek: HandshakeInit -> HandshakeWait
            (1, 1, 2), // Finish -> Established
            (2, 2, 2), // AppData
            (2, 2, 2), // AppData
            (2, 3, 2), // Rekey
            (2, 2, 2), // AppData
            (2, 4, 3), // Close -> Closed
        ],
        "the 4x5 oracle walk over the live machine's rows"
    );
}

/// THE FLIP (mandatory anti-vacuity): inject ONE impossible row —
/// Established (2) + HandshakeEk (0) "accepted" — into a copy of the
/// lawful walk. The court MUST flip to a typed refusal. Negative
/// control first: the pristine walk still replays OK.
#[test]
fn impossible_row_flips_the_court() {
    let sink = lawful_walk();
    let license = root_license();
    // Negative control: pristine is lawful.
    sink.replay(&license).expect("pristine control");

    // MUTATION: splice the impossible row into a cloned walk (in-memory
    // copy only; the shared tree is untouched), keeping ordinals zero-gap
    // so the TRANSITION oracle is the gate that fires, not the ordinal law.
    let mut rows: Vec<TapEvent> = sink.rows().to_vec();
    let impossible = TapEvent {
        ordinal: rows[2].ordinal, // duplicate position 2's ordinal slot
        state: 2,                 // Established
        kind: 0,                  // HandshakeEk
        epoch: 1,
        budget: rows[2].budget,
        fingerprint: [0u8; 32],
    };
    rows[2] = impossible;
    // Replay the mutated rows through the same oracle the court core uses.
    let err = replay_rows_only(&rows).expect_err("impossible row MUST flip the court");
    assert_eq!(
        err,
        TapRefusal::IllegalTransition { from: 2, kind: 0, got: 2 },
        "Established + HandshakeEk is outside the session table"
    );
}

/// Oracle-only replay over row data (the leg-3 walk extracted for
/// mutation tests over row COPIES; the full chain gates ran in the
/// pristine leg).
fn replay_rows_only(rows: &[TapEvent]) -> Result<(), TapRefusal> {
    let mut from: u8 = 0;
    // Rows mint AFTER the accepted step (post-advance ordinal), so the
    // anchor is the FIRST row's ordinal, then +1 per accepted step.
    let mut expect_ordinal: Option<u32> = None;
    let mut expect_budget: u32 = 0;
    let mut epoch: Option<u16> = None;
    for row in rows {
        if let Some(e) = epoch {
            if row.epoch != e {
                return Err(TapRefusal::EpochLaw {
                    expected: e,
                    got: row.epoch,
                });
            }
        } else {
            epoch = Some(row.epoch);
        }
        let want_ord = expect_ordinal.unwrap_or(row.ordinal);
        if row.ordinal != want_ord {
            return Err(TapRefusal::OrdinalLaw {
                expected: want_ord,
                got: row.ordinal,
            });
        }
        expect_ordinal = Some(want_ord.wrapping_add(1));
        let to = lawful_step(from, row.kind).ok_or(TapRefusal::IllegalTransition {
            from,
            kind: row.kind,
            got: row.state,
        })?;
        if row.state != to {
            return Err(TapRefusal::IllegalTransition {
                from,
                kind: row.kind,
                got: row.state,
            });
        }
        let want = match row.kind {
            2 => expect_budget + 1,
            3 => 0,
            _ => expect_budget,
        };
        if row.budget != want {
            return Err(TapRefusal::BudgetLaw {
                expected: want,
                got: row.budget,
            });
        }
        expect_budget = want;
        from = to;
    }
    Ok(())
}

/// PURITY LAW, witnessed live: a refused frame mints NO row and NO
/// chain link, and leaves the session byte-identical (fingerprint
/// assert on the REAL machine). The refusal rows in the projector's
/// value list carry to==from BY CONSTRUCTION (nothing mutated), which
/// is exactly why no ring row can ever carry one.
#[test]
fn refusal_mints_nothing_and_preserves_purity() {
    let kem = TestKem;
    let license = root_license();
    let mut sink = TapSink::new();
    let mut s = Session::new(&kem, 1, 7, 0);
    let peer = ek(1);

    // Handshake completes (2 rows sunk).
    let wire = handshake_frame(&peer, 0, 1);
    s.step(&parse_frame(&wire).expect("parse")).expect("ek");
    tap_step(&mut sink, &s, FrameKind::HandshakeEk, &license);
    let wire = finish_frame(&peer, 1, 1);
    s.step(&parse_frame(&wire).expect("parse")).expect("finish");
    tap_step(&mut sink, &s, FrameKind::HandshakeFinish, &license);
    assert_eq!(sink.len(), 2);

    // REFUSAL 1: replayed ordinal (gap/replay attack shape). The machine
    // must refuse typed, mutate NOTHING, and the caller taps NOTHING.
    let fp_before = s.fingerprint();
    let wire = handshake_frame(&peer, 0, 1); // ordinal 0 already consumed
    let err = s
        .step(&parse_frame(&wire).expect("parse"))
        .expect_err("replayed ordinal must refuse");
    assert!(matches!(err, SessionError::OrdinalMismatch { .. }));
    assert_eq!(s.fingerprint(), fp_before, "no mutation on refusal");
    assert_eq!(sink.len(), 2, "refusal minted NO row");
    assert_eq!(
        replay_rows_only(sink.rows()),
        Ok(()),
        "row log still replays lawful"
    );

    // REFUSAL 2: impossible cell at the LIVE machine too (Established +
    // HandshakeEk) — the session table itself refuses it.
    let wire = handshake_frame(&peer, 2, 1);
    let err = s
        .step(&parse_frame(&wire).expect("parse"))
        .expect_err("Established + HandshakeEk must refuse");
    assert!(
        matches!(err, SessionError::UnexpectedFrame { state: 2, frame: 0 }),
        "live machine refuses the impossible cell, got {err:?}"
    );
    assert_eq!(s.fingerprint(), fp_before, "no mutation on refusal");
    assert_eq!(sink.len(), 2, "refusal minted NO row");

    // The sink boundary is pure too: appending is the only writer, and
    // a full replay of the two-row prefix stays lawful.
    sink.replay(&license).expect("prefix replay lawful");
}

/// Reserved-zero law: decode refuses a nonzero reserved tail, typed.
#[test]
fn reserved_zero_law_refuses_closed() {
    let mut row = TapEvent {
        ordinal: 1,
        state: 2,
        kind: 2,
        epoch: 1,
        budget: 3,
        fingerprint: [9u8; 32],
    };
    let mut wire = row.encode();
    assert!(TapEvent::decode(&wire).is_ok(), "pristine decodes");
    wire[63] = 0xFF; // reserved tail byte
    assert_eq!(
        TapEvent::decode(&wire).err(),
        Some(TapRefusal::ReservedNotZero),
        "nonzero reserved must refuse closed"
    );
    // Encode/decode round-trip is identity.
    let decoded = TapEvent::decode(&row.encode()).expect("round trip");
    assert_eq!(decoded, row);
    row.fingerprint[3] ^= 0x01;
    assert_ne!(TapEvent::decode(&row.encode()).expect("rt"), decoded);
}

/// Wire-contract tie to X6: TapEvent <-> AoqEventRow carries fields 1:1
/// and the 64-byte width pins agree.
#[test]
fn x6_row_contract_round_trip() {
    let row = TapEvent {
        ordinal: 0x0102_0304,
        state: 2,
        kind: 3,
        epoch: 0xABCD,
        budget: 4095,
        fingerprint: [0x55; 32],
    };
    let x6 = row.to_x6();
    assert_eq!(x6.ordinal, row.ordinal);
    assert_eq!(x6.state, row.state);
    assert_eq!(x6.kind, row.kind);
    assert_eq!(x6.epoch, row.epoch);
    assert_eq!(x6.budget, row.budget);
    assert_eq!(x6.fingerprint, row.fingerprint);
    assert!(x6.reserved.iter().all(|&b| b == 0));
    let back = TapEvent::from_x6(&x6);
    assert_eq!(back, row, "1:1 field contract both directions");
}

/// Chain binding flip: a row-log tamper that PASSES the transition
/// oracle (a lawful-looking budget bump) must still flip the court via
/// the row<->chain re-seal binding — rows are chain-covered, the log
/// cannot drift from the chain.
#[test]
fn row_log_drift_flips_via_chain_binding() {
    let sink = lawful_walk();
    let license = root_license();
    sink.replay(&license).expect("pristine control");
    let mut drifted: Vec<TapEvent> = sink.rows().to_vec();
    drifted[3].fingerprint[0] ^= 0x01;
    // Oracle alone would pass (the fingerprint is not cross-checked by
    // the transition walk); the chain binding must catch it.
    assert!(
        replay_rows_only(&drifted).is_ok(),
        "oracle-only replay passes the drift (binding is the gate that fires)"
    );
    let err = replay_with_binding(&sink, &drifted, &license)
        .expect_err("drifted row log must flip via chain binding");
    assert_eq!(
        err,
        TapRefusal::RowBinding { index: 3 },
        "row log and chain desync is typed"
    );
}

/// Full replay with the chain-binding gate, over a supplied row log.
fn replay_with_binding(
    sink: &TapSink,
    rows: &[TapEvent],
    license: &P28LicenseAttestation,
) -> Result<(), TapRefusal> {
    // Chain gate over the REAL committed bytes.
    sink.replay(license)?;
    // Binding gate over the SUPPLIED (possibly drifted) log.
    for (i, row) in rows.iter().enumerate() {
        let bytes = row.reseal_bytes()?;
        if bytes != sink.blocks()[i].bytes() {
            return Err(TapRefusal::RowBinding { index: i as u64 });
        }
    }
    replay_rows_only(rows)
}
