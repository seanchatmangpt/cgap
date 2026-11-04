//! AoQ session — 4-variant handshake/rekey state machine (PG-A5).
//!
//! Laws:
//! 1. MLKEM wire widths are `const`-pinned: encapsulation key 1184,
//!    ciphertext 1088, shared secret 32.
//! 2. `derive()` is a two-round KDF over three context constants; the
//!    transcript binds `ek_server ‖ ss ‖ epoch`.
//! 3. `REKEY_BUDGET` is COUNTED, not timed: a frame counter that refuses
//!    `AppData` at the budget with [`SessionError::RekeyRequired`] and
//!    resets only on an accepted rekey. No wall clock anywhere.
//! 4. No mutation on refusal: every typed refusal leaves the session
//!    byte-identical (courts assert via [`Session::fingerprint`]).
//! 5. Frame parsing composes the LANDED ontology field table through
//!    [`crate::codec`] / [`crate::header`] (AB-6) — offsets are never
//!    re-declared here.
//! 6. The KEM is a caller-injected seam ([`KemSeam`]); the crate ships no
//!    crypto (R-K1). Courts supply a deterministic test KEM.

use crate::codec;
use crate::error::AoqError;
use crate::header::AoqHeader;

/// ML-KEM-768 encapsulation key width (const-pinned).
pub const EK_BYTES: usize = 1184;
/// ML-KEM-768 ciphertext width (const-pinned).
pub const CT_BYTES: usize = 1088;
/// ML-KEM-768 shared secret width (const-pinned).
pub const SS_BYTES: usize = 32;
/// Rekey budget: frames per key, counted not timed.
pub const REKEY_BUDGET: u32 = 4096;

/// KDF context, handshake round 1.
pub const CTX_HANDSHAKE: &[u8] = b"cgap-aoq/hs/r1";
/// KDF context, rekey round.
pub const CTX_REKEY: &[u8] = b"cgap-aoq/rk/r1";
/// KDF context, transcript tag.
pub const CTX_TRANSCRIPT: &[u8] = b"cgap-aoq/trans/1";

/// Caller-injected KEM seam (host facade; no crypto in this crate).
pub trait KemSeam {
    /// Encapsulate to a peer encapsulation key: ciphertext + shared secret.
    ///
    /// # Errors
    /// Seam-defined; typed [`SessionError`] only.
    fn encapsulate(
        &self,
        ek: &[u8; EK_BYTES],
    ) -> Result<([u8; CT_BYTES], [u8; SS_BYTES]), SessionError>;

    /// Decapsulate a ciphertext with a secret key.
    ///
    /// # Errors
    /// Seam-defined; typed [`SessionError`] only.
    fn decapsulate(&self, dk: &[u8], ct: &[u8; CT_BYTES]) -> Result<[u8; SS_BYTES], SessionError>;

    /// Two-round-KDF substrate: order-sensitive 32-byte derivation.
    fn kdf(&self, parts: &[&[u8]]) -> [u8; 32];
}

/// Typed session refusals. Integer/fixed payloads only (alloc-free).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionError {
    /// Underlying AOQ1 header law refusal (composed, not re-declared).
    Wire(AoqError),
    /// Frame body length wrong for its kind.
    LengthMismatch { expected: usize, got: usize },
    /// Kind tag outside the closed 5-kind table.
    UnknownFrameKind { got: u8 },
    /// (state, frame) cell outside the transition table.
    UnexpectedFrame { state: u8, frame: u8 },
    /// Frame epoch is not the session's active epoch.
    EpochMismatch { expected: u16, got: u16 },
    /// Frame ordinal is not the expected next ordinal (replay/gap).
    OrdinalMismatch { expected: u32, got: u32 },
    /// ServerFinish ct-echo does not bind our pending ciphertext.
    TranscriptMismatch,
    /// Counted frame budget exhausted; a rekey is required first.
    RekeyRequired { used: u32, budget: u32 },
    /// Session is Closed; nothing is lawful.
    SessionClosed,
    /// KEM seam refused the operation (`op`: 0 encapsulate, 1 decapsulate).
    KemRefused { op: u8 },
}

impl From<AoqError> for SessionError {
    fn from(e: AoqError) -> Self {
        Self::Wire(e)
    }
}

impl core::fmt::Display for SessionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::Wire(e) => write!(f, "wire: {e}"),
            Self::LengthMismatch { expected, got } => {
                write!(f, "length mismatch: expected {expected}B, got {got}B")
            }
            Self::UnknownFrameKind { got } => write!(f, "unknown frame kind {got}"),
            Self::UnexpectedFrame { state, frame } => {
                write!(f, "unexpected frame {frame:#04x} in state {state}")
            }
            Self::EpochMismatch { expected, got } => {
                write!(f, "epoch mismatch: expected {expected}, got {got}")
            }
            Self::OrdinalMismatch { expected, got } => {
                write!(f, "ordinal mismatch: expected {expected}, got {got}")
            }
            Self::TranscriptMismatch => f.write_str("transcript mismatch"),
            Self::RekeyRequired { used, budget } => {
                write!(f, "rekey required: used {used}/{budget}")
            }
            Self::SessionClosed => f.write_str("session closed"),
            Self::KemRefused { op } => write!(f, "kem refused op {op}"),
        }
    }
}

impl core::error::Error for SessionError {}

/// The 4-variant session state machine.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Fresh session; awaiting the peer's encapsulation key frame.
    HandshakeInit = 0,
    /// Encapsulated; awaiting the server's ct-echo finish frame.
    HandshakeWait = 1,
    /// Keys derived; application data and rekeys flow.
    Established = 2,
    /// Terminal; every frame is refused.
    Closed = 3,
}

impl SessionState {
    /// Stable numeric id (courts/wire-log safe).
    #[must_use]
    pub const fn id(self) -> u8 {
        self as u8
    }
}

/// Closed frame-kind table (5 kinds).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind {
    /// Peer encapsulation key (handshake or rekey offer).
    HandshakeEk = 0,
    /// Server finish: ct-echo binding check.
    HandshakeFinish = 1,
    /// Application data.
    AppData = 2,
    /// Rekey offer (new encapsulation key).
    Rekey = 3,
    /// Terminal close; lawful from any non-Closed state.
    Close = 4,
}

impl FrameKind {
    /// Stable numeric id.
    #[must_use]
    pub const fn id(self) -> u8 {
        self as u8
    }
}

impl TryFrom<u8> for FrameKind {
    type Error = SessionError;

    fn try_from(raw: u8) -> Result<Self, SessionError> {
        match raw {
            0 => Ok(Self::HandshakeEk),
            1 => Ok(Self::HandshakeFinish),
            2 => Ok(Self::AppData),
            3 => Ok(Self::Rekey),
            4 => Ok(Self::Close),
            other => Err(SessionError::UnknownFrameKind { got: other }),
        }
    }
}

/// A parsed frame: header (AB-6 law) + kind + body slice.
#[derive(Debug)]
pub struct Frame<'a> {
    /// Decoded AOQ1 header (codec-verified: magic/version/epoch/digest).
    pub header: AoqHeader,
    /// Closed frame kind.
    pub kind: FrameKind,
    /// Body bytes after the 64-byte envelope and the 1-byte kind tag.
    pub payload: &'a [u8],
}

/// Header-envelope + kind-tag + payload wire layout, length-checked.
///
/// Wire shape: `[AOQ1 header: 64B][kind: 1B][payload]`.
///
/// # Errors
/// Typed [`SessionError`] refusals; header law is composed through
/// [`codec::decode`], never re-declared.
pub fn parse_frame(buf: &[u8]) -> Result<Frame<'_>, SessionError> {
    let header = codec::decode(buf)?;
    if buf.len() < crate::header::SIZE_BYTES + 1 {
        return Err(SessionError::LengthMismatch {
            expected: crate::header::SIZE_BYTES + 1,
            got: buf.len(),
        });
    }
    let kind = FrameKind::try_from(buf[crate::header::SIZE_BYTES])?;
    let body = &buf[crate::header::SIZE_BYTES + 1..];
    let expected = match kind {
        FrameKind::HandshakeEk | FrameKind::Rekey => EK_BYTES,
        FrameKind::HandshakeFinish => CT_BYTES,
        FrameKind::AppData => 4, // seq prefix; payload may be empty
        FrameKind::Close => 0,
    };
    if body.len() < expected {
        return Err(SessionError::LengthMismatch {
            expected,
            got: body.len(),
        });
    }
    if matches!(kind, FrameKind::Close) && body.len() != expected {
        return Err(SessionError::LengthMismatch {
            expected,
            got: body.len(),
        });
    }
    Ok(Frame {
        header,
        kind,
        payload: body,
    })
}

/// Two-round KDF: `round2 = H(ctx ‖ H(ctx ‖ ss) ‖ ek)`.
#[must_use]
pub fn derive<K: KemSeam>(kem: &K, ctx: &[u8], ss: &[u8; SS_BYTES], ek: &[u8]) -> [u8; 32] {
    let round1 = kem.kdf(&[ctx, ss]);
    kem.kdf(&[ctx, &round1, ek])
}

/// Transcript tag binding `ek_server ‖ ss ‖ epoch` (law 2).
#[must_use]
pub fn transcript_tag<K: KemSeam>(
    kem: &K,
    ek_server: &[u8],
    ss: &[u8; SS_BYTES],
    epoch: u16,
) -> [u8; 32] {
    let mut epoch_le = [0u8; 2];
    epoch_le.copy_from_slice(&epoch.to_le_bytes());
    kem.kdf(&[CTX_TRANSCRIPT, ek_server, ss, &epoch_le])
}

/// The session state machine over an injected KEM seam.
#[derive(Debug)]
pub struct Session<'k, K: KemSeam> {
    kem: &'k K,
    state: SessionState,
    epoch: u16,
    key_id: u64,
    ordinal: u32,
    budget_used: u32,
    transcript: [u8; 32],
    pending_ct: [u8; CT_BYTES],
}

impl<'k, K: KemSeam> Session<'k, K> {
    /// Construct a session in [`SessionState::HandshakeInit`] at a given
    /// epoch, key id, and starting frame ordinal (constructor state, not a
    /// transition; enables ordinal-wrap walks in courts).
    #[must_use]
    pub fn new(kem: &'k K, epoch: u16, key_id: u64, ordinal: u32) -> Self {
        Self {
            kem,
            state: SessionState::HandshakeInit,
            epoch,
            key_id,
            ordinal,
            budget_used: 0,
            transcript: [0u8; 32],
            pending_ct: [0u8; CT_BYTES],
        }
    }

    /// Current state.
    #[must_use]
    pub const fn state(&self) -> SessionState {
        self.state
    }

    /// Active epoch.
    #[must_use]
    pub const fn epoch(&self) -> u16 {
        self.epoch
    }

    /// Expected next frame ordinal (wrapping u32).
    #[must_use]
    pub const fn ordinal(&self) -> u32 {
        self.ordinal
    }

    /// Counted budget used since the last accepted rekey.
    #[must_use]
    pub const fn budget_used(&self) -> u32 {
        self.budget_used
    }

    /// Transcript tag (binds ek_server ‖ ss ‖ epoch).
    #[must_use]
    pub const fn transcript(&self) -> [u8; 32] {
        self.transcript
    }

    /// Canonical fingerprint of the full session state — the byte-identity
    /// witness for no-mutation-on-refusal courts.
    #[must_use]
    pub fn fingerprint(&self) -> [u8; 32] {
        let mut s_le = [0u8; 1];
        s_le[0] = self.state.id();
        let mut e_le = [0u8; 2];
        e_le.copy_from_slice(&self.epoch.to_le_bytes());
        let mut k_le = [0u8; 8];
        k_le.copy_from_slice(&self.key_id.to_le_bytes());
        let mut o_le = [0u8; 4];
        o_le.copy_from_slice(&self.ordinal.to_le_bytes());
        let mut b_le = [0u8; 4];
        b_le.copy_from_slice(&self.budget_used.to_le_bytes());
        self.kem.kdf(&[
            b"cgap-aoq/fp/1",
            &s_le,
            &e_le,
            &k_le,
            &o_le,
            &b_le,
            &self.transcript,
            &self.pending_ct,
        ])
    }

    /// Advance the machine by one frame. The full 4x5 (state, kind) table
    /// lives in this match; every non-table cell is a typed refusal and no
    /// refusal mutates the session.
    ///
    /// # Errors
    /// Typed [`SessionError`]; on `Err` the session is byte-identical to
    /// before the call.
    pub fn step(&mut self, frame: &Frame<'_>) -> Result<(), SessionError> {
        // Closed refuses everything (law: terminal).
        if self.state == SessionState::Closed {
            return Err(SessionError::SessionClosed);
        }
        // Epoch law on every frame: must match the active epoch. RETIRED
        // (u16::MAX) can never be an active epoch, so retired frames are
        // refused here by construction.
        if frame.header.epoch != self.epoch {
            return Err(SessionError::EpochMismatch {
                expected: self.epoch,
                got: frame.header.epoch,
            });
        }
        // Ordinal law on every frame: exactly the expected (wrapping) u32.
        if frame.header.frame_ordinal != self.ordinal {
            return Err(SessionError::OrdinalMismatch {
                expected: self.ordinal,
                got: frame.header.frame_ordinal,
            });
        }
        match (self.state, frame.kind) {
            (SessionState::HandshakeInit, FrameKind::HandshakeEk) => {
                let mut ek = [0u8; EK_BYTES];
                ek.copy_from_slice(frame.payload[..EK_BYTES].as_ref());
                let (ct, ss) = self.kem.encapsulate(&ek)?;
                self.transcript = transcript_tag(self.kem, &ek, &ss, self.epoch);
                self.pending_ct = ct;
                self.advance();
                self.state = SessionState::HandshakeWait;
                Ok(())
            }
            (SessionState::HandshakeWait, FrameKind::HandshakeFinish) => {
                let mut echo = [0u8; CT_BYTES];
                echo.copy_from_slice(frame.payload[..CT_BYTES].as_ref());
                if echo != self.pending_ct {
                    return Err(SessionError::TranscriptMismatch);
                }
                self.advance();
                self.state = SessionState::Established;
                Ok(())
            }
            (SessionState::Established, FrameKind::AppData) => {
                if self.budget_used >= REKEY_BUDGET {
                    return Err(SessionError::RekeyRequired {
                        used: self.budget_used,
                        budget: REKEY_BUDGET,
                    });
                }
                self.budget_used += 1;
                self.advance();
                Ok(())
            }
            (SessionState::Established, FrameKind::Rekey) => {
                let mut ek = [0u8; EK_BYTES];
                ek.copy_from_slice(frame.payload[..EK_BYTES].as_ref());
                let (ct, ss) = self.kem.encapsulate(&ek)?;
                // Two-round rekey derivation over the rekey context, then a
                // fresh transcript binding the new ek.
                let keys = derive(self.kem, CTX_REKEY, &ss, &ek);
                self.transcript = self.kem.kdf(&[CTX_TRANSCRIPT, &keys, &ek]);
                self.pending_ct = ct;
                self.budget_used = 0;
                self.advance();
                Ok(())
            }
            (_, FrameKind::Close) => {
                self.advance();
                self.state = SessionState::Closed;
                Ok(())
            }
            (state, kind) => Err(SessionError::UnexpectedFrame {
                state: state.id(),
                frame: kind.id(),
            }),
        }
    }

    /// Ordinal/budget bookkeeping shared by accepted transitions. The u32
    /// ordinal wraps by construction (`wrapping_add` — counted, never time).
    fn advance(&mut self) {
        self.ordinal = self.ordinal.wrapping_add(1);
    }

    /// Emit a verified application frame (Established only):
    /// `[AOQ1 header 64B][kind 2][seq BE 4B][payload][tag 4B]` where
    /// `tag == kdf(header ‖ kind ‖ seq ‖ payload)[0..4]` — the tag covers
    /// the WHOLE wire frame so the chain-side re-derivation
    /// (`append_verified` over `frame[..len-4]`) composes exactly.
    /// The trailing tag is the compose-with-`emit_verified` law surface:
    /// [`crate::chain::AoqChain::append_verified`] re-derives it with the
    /// same injected substrate before linking.
    ///
    /// Writes at most `64 + 1 + 4 + payload.len() + 4` bytes into `out` and
    /// returns the written length. Length-checked, alloc-free.
    ///
    /// # Errors
    /// [`SessionError::UnexpectedFrame`] unless Established;
    /// [`SessionError::LengthMismatch`] when `out` is too small.
    pub fn emit_verified(
        &self,
        seq: u32,
        payload: &[u8],
        out: &mut [u8],
    ) -> Result<usize, SessionError> {
        if self.state != SessionState::Established {
            return Err(SessionError::UnexpectedFrame {
                state: self.state.id(),
                frame: 0xff,
            });
        }
        let total = crate::header::SIZE_BYTES + 1 + 4 + payload.len() + 4;
        if out.len() < total {
            return Err(SessionError::LengthMismatch {
                expected: total,
                got: out.len(),
            });
        }
        let header = AoqHeader::new(self.epoch, self.key_id, self.ordinal, seq);
        codec::encode(&header, out)?;
        out[64] = FrameKind::AppData.id();
        out[65..69].copy_from_slice(&seq.to_be_bytes());
        out[69..69 + payload.len()].copy_from_slice(payload);
        let body_end = 69 + payload.len();
        let covered = &out[..body_end];
        let tag = self.kem.kdf(&[covered]);
        out[body_end..body_end + 4].copy_from_slice(&tag[0..4]);
        Ok(body_end + 4)
    }
}

#[cfg(test)]
// [AB-1 SLA DISCLOSED MARKER 2026-11-03] AB-1 added this marker as a
// minimal unblock: the inline-test module below uses alloc/vec in this
// no_std crate (heap-backed inline tests, cgap-core tracking_alloc.rs
// precedent). Disclosed in ab-1.md; owning lane may re-home the tests
// under tests/ and retire the marker.
// SOURCE-SCAN-EXEMPT: alloc:: : inline-test-only heap courts, tests-only collaborator
// SOURCE-SCAN-EXEMPT: vec! : inline-test-only heap courts, tests-only collaborator
mod tests {
    use super::*;

    /// Deterministic hash-based test KEM (courts' collaborator; real
    /// digest substrate via the same FNV-style mixing as chain's unit
    /// digest — no crypto in the crate under test).
    struct TestKem;

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
            // Deterministic inverse: the dk IS the ek (hash-based KEM).
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

    #[test]
    fn mlkem_widths_are_const_pinned() {
        assert_eq!(EK_BYTES, 1184);
        assert_eq!(CT_BYTES, 1088);
        assert_eq!(SS_BYTES, 32);
    }

    #[test]
    fn derive_is_two_round_and_context_bound() {
        let k = TestKem;
        let key = derive(&k, CTX_HANDSHAKE, &mix32(&[b"ss"]), &ek(1));
        let single = k.kdf(&[CTX_HANDSHAKE, &mix32(&[b"ss"]), &ek(1)]);
        assert_ne!(key, single, "two-round KDF must not equal one round");
        assert_ne!(
            key,
            derive(&k, CTX_REKEY, &mix32(&[b"ss"]), &ek(1)),
            "context constants must bind derivation"
        );
        assert_eq!(
            key,
            derive(&k, CTX_HANDSHAKE, &mix32(&[b"ss"]), &ek(1)),
            "deterministic"
        );
    }

    #[test]
    fn transcript_binds_ek_ss_epoch() {
        let k = TestKem;
        let base = transcript_tag(&k, &ek(1), &mix32(&[b"ss"]), 0);
        assert_ne!(base, transcript_tag(&k, &ek(2), &mix32(&[b"ss"]), 0));
        assert_ne!(base, transcript_tag(&k, &ek(1), &mix32(&[b"other"]), 0));
        assert_ne!(base, transcript_tag(&k, &ek(1), &mix32(&[b"ss"]), 1));
    }

    #[test]
    fn happy_handshake_reaches_established() {
        let k = TestKem;
        let mut s = Session::new(&k, 0, 7, 0);
        assert_eq!(s.state(), SessionState::HandshakeInit);
        let wire = handshake_frame(&ek(1), 0);
        let frame = parse_frame(&wire).unwrap();
        s.step(&frame).unwrap();
        assert_eq!(s.state(), SessionState::HandshakeWait);
        let finish_wire = finish_frame(1);
        let frame = parse_frame(&finish_wire).unwrap();
        s.step(&frame).unwrap();
        assert_eq!(s.state(), SessionState::Established);
    }

    // Wire helpers (header + kind + body, codec-encoded; stack arrays —
    // the purity gate bans alloc/vec tokens even in test code).
    const HS_WIRE: usize = crate::header::SIZE_BYTES + 1 + EK_BYTES;
    const FIN_WIRE: usize = crate::header::SIZE_BYTES + 1 + CT_BYTES;
    const APP_WIRE: usize = crate::header::SIZE_BYTES + 1 + 4 + 187;

    fn handshake_frame(ek_bytes: &[u8; EK_BYTES], ordinal: u32) -> [u8; HS_WIRE] {
        let mut buf = [0u8; HS_WIRE];
        let h = AoqHeader::new(0, 7, ordinal, 0);
        codec::encode(&h, &mut buf).unwrap();
        buf[64] = FrameKind::HandshakeEk.id();
        buf[65..].copy_from_slice(ek_bytes);
        buf
    }

    fn finish_frame(ordinal: u32) -> [u8; FIN_WIRE] {
        let mut buf = [0u8; FIN_WIRE];
        let h = AoqHeader::new(0, 7, ordinal, 0);
        codec::encode(&h, &mut buf).unwrap();
        buf[64] = FrameKind::HandshakeFinish.id();
        // Deterministic ct for ek(1): the session's pending ct.
        let (ct, _) = TestKem.encapsulate(&ek(1)).unwrap();
        buf[65..].copy_from_slice(&ct);
        buf
    }

    /// Returns (wire, total length) — payload may be shorter than the
    /// 187B capacity; callers pass `&wire[..len]`.
    fn app_frame(ordinal: u32, seq: u32, payload: &[u8]) -> ([u8; APP_WIRE], usize) {
        let mut buf = [0u8; APP_WIRE];
        let h = AoqHeader::new(0, 7, ordinal, seq);
        codec::encode(&h, &mut buf).unwrap();
        buf[64] = FrameKind::AppData.id();
        buf[65..69].copy_from_slice(&seq.to_be_bytes());
        buf[69..69 + payload.len()].copy_from_slice(payload);
        (buf, 69 + payload.len())
    }
}
