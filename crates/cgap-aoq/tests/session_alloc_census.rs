//! Session alloc census (alloc-elimination lane, 2026-11-04): the
//! Established-state AppData round trip — `emit_verified` (seal) ->
//! `parse_frame` -> `open_verified` (open/parse) — at a 1 KiB payload
//! must perform ZERO heap allocations (E1's lib-plane finding, verified
//! with the house counting allocator, not asserted).
//!
//! Anti-vacuity: the census harness itself is positively controlled by
//! `cgap-audit`'s own tests (a tracked scope counting `vec![0u8; 32]`
//! as 1); a census that cannot count is vacuous, so the harness court
//! lives with the primitive. The opened plaintext is asserted against
//! the sealed payload (final state) so the round trip cannot be folded
//! away.
//!
//! Same census harness as `crates/cgap-ffi/tests/receipt_chain_scale.rs`
//! and `tests/alloc_courts.rs` (house pattern).

#![cfg(not(target_os = "none"))]

use cgap_aoq::aead::TAG_LEN;
use cgap_aoq::codec;
use cgap_aoq::header::{AoqHeader, SIZE_BYTES};
use cgap_aoq::session::{
    confirm_tag, derive, parse_frame, transcript_tag, FrameKind, KemSeam, Session, SessionError,
    SessionState, CTX_HANDSHAKE, CTX_RECV, CT_BYTES, EK_BYTES, SS_BYTES,
};
use cgap_audit::alloc_counter::with_alloc_tracking;

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

/// Deterministic hash-based test KEM (PG-A5 shape, session_courts model).
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

    fn decapsulate(&self, dk: &[u8], ct: &[u8; CT_BYTES]) -> Result<[u8; SS_BYTES], SessionError> {
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

fn ek_of(seed: u8) -> [u8; EK_BYTES] {
    let mut k = [seed; EK_BYTES];
    k[0] = seed;
    k[1183] = seed ^ 0xa5;
    k
}

/// AOQ1 header + kind + body wire builder (composes codec::encode).
fn wire(epoch: u16, key_id: u64, ordinal: u32, seq: u32, kind: FrameKind, body: &[u8]) -> Vec<u8> {
    let mut buf = vec![0u8; SIZE_BYTES + 1 + body.len()];
    let h = AoqHeader::new(epoch, key_id, ordinal, seq);
    codec::encode(&h, &mut buf).expect("codec::encode");
    buf[SIZE_BYTES] = kind.id();
    buf[SIZE_BYTES + 1..].copy_from_slice(body);
    buf
}

/// Handshake to Established at (epoch, ordinal) — outside the census
/// scope (setup, not subject).
fn established(epoch: u16, ordinal: u32, ek_seed: u8) -> Session<'static, TestKem> {
    let mut s = Session::new(&KEM, epoch, 7, ordinal);
    let ek = ek_of(ek_seed);
    let w = wire(epoch, 7, ordinal, 0, FrameKind::HandshakeEk, &ek);
    s.step(&parse_frame(&w).expect("ek frame"))
        .expect("ek step");
    let (ct, ss) = KEM.encapsulate(&ek).expect("encapsulate");
    let transcript = transcript_tag(&KEM, &ek, &ss, epoch);
    let hs_master = derive(&KEM, CTX_HANDSHAKE, &ss, &ek);
    let recv = KEM.kdf(&[CTX_RECV, &hs_master]);
    let tag = confirm_tag(&KEM, &recv, &transcript, &ct);
    let mut body = [0u8; CT_BYTES + 16];
    body[..CT_BYTES].copy_from_slice(&ct);
    body[CT_BYTES..].copy_from_slice(&tag);
    let w = wire(epoch, 7, ordinal + 1, 0, FrameKind::HandshakeFinish, &body);
    s.step(&parse_frame(&w).expect("fin frame"))
        .expect("fin step");
    assert_eq!(s.state(), SessionState::Established);
    s
}

/// CENSUS 1 — seal direction: `emit_verified` at 1 KiB into a caller
/// buffer. Count must be 0.
#[test]
fn emit_verified_1kib_alloc_free() {
    let s = established(0, 100, 1);
    let payload = vec![0xA5u8; 1024];
    let mut out = vec![0u8; SIZE_BYTES + 1 + 4 + payload.len() + TAG_LEN];

    let (written, report) = with_alloc_tracking(|| {
        s.emit_verified(7, &payload, &mut out)
            .expect("emit_verified")
    });
    assert_eq!(written, SIZE_BYTES + 5 + payload.len() + TAG_LEN);
    assert_eq!(
        report.count, 0,
        "emit at 1 KiB must be alloc-free ({report:?})"
    );
}

/// CENSUS 2 — open/parse direction: the inbound AppData frame the wire
/// peer seals under this session's recv_key (setup, outside the scope),
/// then `parse_frame` + `open_verified` inside the scope. Count must be 0.
#[test]
fn open_parse_1kib_alloc_free() {
    let s = established(0, 100, 1);
    let ek = ek_of(1);
    let (_, ss) = KEM.encapsulate(&ek).expect("encapsulate");
    let hs_master = derive(&KEM, CTX_HANDSHAKE, &ss, &ek);
    let recv_key = KEM.kdf(&[CTX_RECV, &hs_master]);

    let payload = vec![0xA5u8; 1024];
    let mut prefix = [0u8; SIZE_BYTES + 5];
    codec::encode(&AoqHeader::new(0, 7, 102, 9), &mut prefix).expect("encode");
    prefix[SIZE_BYTES] = FrameKind::AppData.id();
    prefix[SIZE_BYTES + 1..SIZE_BYTES + 5].copy_from_slice(&9u32.to_le_bytes());
    let mut frame_in = vec![0u8; SIZE_BYTES + 1 + 4 + payload.len() + TAG_LEN];
    frame_in[..SIZE_BYTES + 5].copy_from_slice(&prefix);
    let nonce = cgap_aoq::aead::Nonce96::new(0, (u64::from(102u32) << 32) | u64::from(9u32))
        .expect("nonce");
    let mut tag = [0u8; TAG_LEN];
    cgap_aoq::aead::seal_detached(
        &mut frame_in[SIZE_BYTES + 5..SIZE_BYTES + 5 + payload.len()],
        &mut tag,
        &payload,
        &prefix,
        &recv_key,
        &nonce,
    );
    frame_in[SIZE_BYTES + 5 + payload.len()..].copy_from_slice(&tag);

    let mut plain = vec![0u8; payload.len()];
    let report = {
        let ((), r) = with_alloc_tracking(|| {
            let frame = parse_frame(&frame_in).expect("parse_frame");
            assert_eq!(frame.kind, FrameKind::AppData);
            s.open_verified(&frame_in, &mut plain)
                .expect("open_verified");
        });
        r
    };
    assert_eq!(
        report.count, 0,
        "parse+open at 1 KiB must be alloc-free ({report:?})"
    );
    assert_eq!(plain, payload, "plaintext round trip (final state)");
}
