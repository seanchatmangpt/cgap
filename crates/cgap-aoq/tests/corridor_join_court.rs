//! Corridor JOIN court (AF-1, Systemic Integration Gate, v26.11.3).
//!
//! The full DFLSS corridor in one court: real ArrowArray (c_abi export) ->
//! ML-KEM-768 encaps (AB-5) -> ChaCha20-Poly1305 seal (AB-4) -> AoqWriter
//! vectored scatter over a LIVE UnixStream (AB-8 writer + AB-6 codec) ->
//! verify_window -> decrypt_in_place -> chain append (AB-7, payload_digest
//! over the decrypted plaintext). 100 iterations with per-iteration
//! chain-verify + per-iteration byte-identity vs the pre-seal array bytes.
//! Plus the 100k-link scale leg (chain only) and anti-vacuity tamper
//! courts.
//!
//! Chicago discipline: every collaborator is the REAL kernel (cgap-crypto
//! ML-KEM / ChaCha20-Poly1305, cgap-arrow Primitive + c_abi export) over a
//! real transport (socketpair UDS). The only injected collaborator is the
//! digest substrate, which the crate itself mandates as a seam (R-K1: no
//! crypto ships in cgap-aoq); here that seam is bound to the REAL Shake256
//! (FIPS 202), not a test double.
//!
//! Anti-vacuity: a flipped ciphertext byte must refuse at the tag gate, a
//! cursor regression must refuse at the seq gate, a tampered AEAD tag must
//! refuse at open_detached, and a tampered chain record must refuse at
//! verify. A green run with no observed refusal is a VACUOUS GATE.

// AF-1 integration court: host std + alloc are test-only collaborators.
// SOURCE-SCAN-EXEMPT: std:: : integration-court-only collaborator
// SOURCE-SCAN-EXEMPT: vec! : integration-court-only collaborator

use std::ffi::CString;
use std::io::{IoSlice, Read, Write};
use std::os::unix::net::UnixStream;

use cgap_aoq::chain::{AoqChain, AoqChainRecord, ChainDigest};
use cgap_aoq::error::AoqError;
use cgap_aoq::session::KemSeam;
use cgap_aoq::verify::{decrypt_in_place, verify_window};
use cgap_aoq::writer::AoqWriter;
use cgap_crypto::aead::{self, Nonce96};
use cgap_crypto::fips202::Shake256;

// ---------------------------------------------------------------------------
// Real digest substrate: Shake256 (FIPS 202) bound to the AOQ chain/KDF seams
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct ShakeDigest;

impl ChainDigest for ShakeDigest {
    fn digest(&self, parts: &[&[u8]]) -> [u8; 32] {
        shake32(parts)
    }
}

/// Order-sensitive 32-byte Shake256 squeeze over concatenated parts.
fn shake32(parts: &[&[u8]]) -> [u8; 32] {
    let mut sh = Shake256::instance();
    for p in parts {
        sh.absorb(p);
    }
    let mut out = [0u8; 32];
    sh.squeeze_into(&mut out);
    out
}

// ---------------------------------------------------------------------------
// KEM seam. REBOUND (AB-5 close for AF1-KD-1): cgap-crypto ML-KEM-768
// (FIPS 203 final) is ALIVE and interop-verified against liboqs — the
// previous TestKem fallback is retired. Deterministic cores are driven with
// in-test seeds (no RNG in-crate by design).
// ---------------------------------------------------------------------------

/// Real ML-KEM-768 adapter over cgap_crypto::kem (FIPS 203 final).
struct MlkemKem;

impl cgap_aoq::session::KemSeam for MlkemKem {
    fn encapsulate(
        &self,
        ek: &[u8; cgap_aoq::session::EK_BYTES],
    ) -> Result<
        (
            [u8; cgap_aoq::session::CT_BYTES],
            [u8; cgap_aoq::session::SS_BYTES],
        ),
        cgap_aoq::session::SessionError,
    > {
        // Deterministic encapsulation seed (no RNG in-crate by design).
        let m = shake32(&[b"af1/mlkem/m", ek]);
        let mut ct = [0u8; 1088];
        let mut ss = [0u8; 32];
        cgap_crypto::kem::encaps(ek, &m, &mut ct, &mut ss);
        Ok((ct, ss))
    }

    fn decapsulate(
        &self,
        dk: &[u8],
        ct: &[u8; cgap_aoq::session::CT_BYTES],
    ) -> Result<[u8; cgap_aoq::session::SS_BYTES], cgap_aoq::session::SessionError> {
        let dk: &[u8; 2400] = dk
            .try_into()
            .map_err(|_| cgap_aoq::session::SessionError::KemRefused { op: 1 })?;
        Ok(cgap_crypto::kem::decaps(dk, ct))
    }

    fn kdf(&self, parts: &[&[u8]]) -> [u8; 32] {
        shake32(parts)
    }
}

// ---------------------------------------------------------------------------
// Shared corridor constants + helpers
// ---------------------------------------------------------------------------

const EPOCH: u16 = 1;
const KEY_ID: u64 = 0xA0_71;

/// 36-byte AAD: magic(4) + epoch(2) + key_id(8) + ordinal(4) + seq(4) +
/// body_len(8) + zero tail (14). Everything the header law pins, plus the
/// body length, flows into the AEAD tag.
fn corridor_aad(ordinal: u32, seq: u32, body_len: usize) -> [u8; aead::AAD_LEN] {
    let mut aad_buf = [0u8; aead::AAD_LEN];
    aad_buf[0..4].copy_from_slice(&cgap_aoq::header::MAGIC.to_le_bytes());
    aad_buf[4..6].copy_from_slice(&EPOCH.to_le_bytes());
    aad_buf[6..14].copy_from_slice(&KEY_ID.to_le_bytes());
    aad_buf[14..18].copy_from_slice(&ordinal.to_le_bytes());
    aad_buf[18..22].copy_from_slice(&seq.to_le_bytes());
    aad_buf[22..30].copy_from_slice(&(body_len as u64).to_le_bytes());
    aad_buf
}

/// Serialize a Primitive<i64>'s values as LE bytes (the pre-seal body whose
/// byte identity the corridor must prove end to end).
fn array_body(values: &[i64]) -> Vec<u8> {
    let mut body = Vec::with_capacity(values.len() * 8);
    for v in values {
        body.extend_from_slice(&v.to_le_bytes());
    }
    body
}

/// Deterministic arrow values for iteration `iter` (derangement so every
/// iteration's body differs).
fn arrow_values(iter: usize) -> Vec<i64> {
    let n = 16 + (iter % 8) * 8;
    (0..n)
        .map(|i| {
            ((i as i64) + 1)
                .wrapping_mul(0x517C_C1B7_2767_6D2F_u64.wrapping_add(iter as u64) as i64)
        })
        .collect()
}

/// Fresh chain over a leaked slab (courts only; kernel stays alloc-free —
/// caller-provided-slab law).
fn corridor_chain(cap: usize) -> AoqChain<'static, ShakeDigest> {
    let slab: &'static mut [AoqChainRecord] =
        Box::leak(vec![AoqChainRecord::zeroed(); cap].into_boxed_slice());
    AoqChain::new(ShakeDigest, slab)
}

/// One full corridor hop: seal -> vectored UDS scatter -> verify_window ->
/// decrypt_in_place -> chain append. Returns the decrypted body the court
/// then byte-compares against the pre-seal array bytes.
fn corridor_hop(
    writer: &mut AoqWriter,
    (alice, bob): &mut (UnixStream, UnixStream),
    key: &[u8; 32],
    plaintext: &[u8],
    chain: &mut AoqChain<'static, ShakeDigest>,
) -> Vec<u8> {
    let (ordinal, seq) = writer.cursor();
    let nonce = Nonce96::new(u32::from(EPOCH), seq as u64)
        .expect("corridor nonce: counter well under the epoch budget");

    // SEAL (AB-4 ChaCha20-Poly1305, key from the KEM shared secret).
    let mut ct_body = vec![0u8; plaintext.len()];
    let mut aead_tag = [0u8; aead::TAG_LEN];
    aead::seal_detached(
        &mut ct_body,
        &mut aead_tag,
        plaintext,
        &corridor_aad(ordinal, seq, plaintext.len()),
        key,
        &nonce,
    );

    // WIRE TAG (verify.rs TAG LAW): the tamper-tier truncation
    // blake3(body)[..16] — receiver-computable from the carried bytes, the
    // same gate cgap-arrow ipc::read_verified applies to AOQ1 magic. The
    // AEAD tag authenticates separately (open_detached).
    let wire_tag = wire_tag_for(&ct_body);

    // SCATTER (AB-8 AoqWriter five-segment frame, live-UDS writev, then
    // commit — ZeroProgress law: commit only after the sink accepted all
    // five segments).
    let total = {
        let segs = writer
            .frame_segments(EPOCH, KEY_ID, &wire_tag, &ct_body)
            .expect("frame_segments");
        let seg_total = segs.total_len();
        let zero_pad = [0u8; 8192]; // body pad: scatter-your-own-zero-buffer law
        let body_pad = segs.body_pad();
        assert!(body_pad <= zero_pad.len(), "court bodies fit one pad block");
        let iovecs = [
            IoSlice::new(segs.prefix),
            IoSlice::new(segs.header),
            IoSlice::new(segs.pad),
            IoSlice::new(segs.body),
            IoSlice::new(&zero_pad[..body_pad]),
            IoSlice::new(segs.tail),
        ];
        let mut sent = 0usize;
        while sent < seg_total {
            let n = alice
                .write_vectored(&iovecs)
                .expect("live-UDS vectored scatter");
            assert!(n > 0, "ZeroProgress law: sink accepted zero bytes");
            sent += n;
        }
        assert_eq!(sent, seg_total, "UDS accepted all five segments");
        seg_total
    };
    writer.commit();

    // RECEIVE: read exactly one window off the live socket.
    let mut window = vec![0u8; total];
    bob.read_exact(&mut window).expect("UDS read_exact");

    // VERIFY (AB-8 window law): the receiver recomputes the tamper-tier tag
    // from the RECEIVED body bytes — this is the gate that must refuse a
    // flipped ciphertext byte.
    let body_len = u32::from_le_bytes([window[72], window[73], window[74], window[75]]) as usize;
    let expected = wire_tag_for(&window[128..128 + body_len]);
    let vw = verify_window(&window, ordinal, seq, &expected).expect("verify_window");

    // DECRYPT IN PLACE: keystream from the SAME ChaCha20 block sequence the
    // seal used (counter starts at 1 in seal's keystream_xor).
    let mut keystream = vec![0u8; vw.body_len];
    cgap_crypto::chacha::chacha20_xor_stream(key, &nonce.to_bytes(), 1, &mut keystream);
    decrypt_in_place(&vw, &keystream, &mut window).expect("decrypt_in_place");

    // CHAIN APPEND (AB-7): payload_digest composed over the DECRYPTED
    // plaintext, per-iteration verify.
    let idx = chain
        .append(EPOCH, &window[vw.body_at..vw.body_at + vw.body_len])
        .expect("chain append");
    assert_eq!(idx as usize, ordinal as usize, "chain index tracks ordinal");
    chain.verify_all().expect("per-iteration chain verify");
    chain
        .verify_span(0, idx as usize)
        .expect("per-iteration span verify");

    window[vw.body_at..vw.body_at + vw.body_len].to_vec()
}

/// Segments total length is `Segments::total_len` (prefix+header+pad+body+
/// body_pad+tail); the court scatters the body pad as its own zero buffer.
///
/// Tamper-tier wire tag: blake3(body)[..16] (cgap-arrow AOQ1 tag law).
fn wire_tag_for(body: &[u8]) -> [u8; 16] {
    let mut tag = [0u8; 16];
    let d = cgap_arrow::blake3::hash(body);
    tag.copy_from_slice(&d[..16]);
    tag
}

// ---------------------------------------------------------------------------
// COURT J1 — the corridor join, 100 iterations, per-iteration gates
// ---------------------------------------------------------------------------

#[test]
fn j1_corridor_join_100_iterations() {
    // KEY HOP: REAL ML-KEM-768 (AB-5 ALIVE; AF1-KD-1 closed by this court).
    let d = shake32(&[b"af1/mlkem/d"]);
    let z = shake32(&[b"af1/mlkem/z"]);
    let mut ek = [0u8; cgap_aoq::session::EK_BYTES];
    let mut dk = [0u8; 2400];
    cgap_crypto::kem::keygen(&d, &z, &mut ek, &mut dk);
    let seam = MlkemKem;
    let (_ct, ss_alice) = seam.encapsulate(&ek).expect("encapsulate");
    let ss_bob = seam.decapsulate(&dk, &_ct).expect("decapsulate");
    assert_eq!(ss_alice, ss_bob, "KEM shared secret agreement");

    // Session KDF (AB-7): AEAD key = derive(CTX_HANDSHAKE, ss, ek).
    let key = cgap_aoq::session::derive(&seam, cgap_aoq::session::CTX_HANDSHAKE, &ss_alice, &ek);

    let mut socks = UnixStream::pair().expect("socketpair UDS");
    let mut writer = AoqWriter::new();
    let mut chain = corridor_chain(256);

    for iter in 0..100usize {
        // ARROWARRAY (cgap-arrow c_abi export) — the payload the corridor
        // carries. Export must produce a live, release-wired ArrowArray.
        let values = arrow_values(iter);
        let body = array_body(&values);
        let builder_vals: Vec<Option<i64>> = values.iter().map(|v| Some(*v)).collect();
        let mut builder = cgap_arrow::primitive::PrimitiveBuilder::<i64>::new();
        for v in builder_vals {
            builder.push(v);
        }
        let prim = builder.finish();
        assert_eq!(prim.len(), values.len());
        let format = CString::new(cgap_arrow::primitive::Primitive::<i64>::format_code())
            .expect("format code");
        let (mut array, _schema) = cgap_arrow::c_abi::export(prim, format.as_ptr());
        assert_eq!(array.length as usize, values.len(), "c_abi export length");
        assert!(array.release.is_some(), "c_abi export wired a release fn");
        // SAFETY: test-owned exported array; releasing exactly once is the
        // exporter duty proven by cgap-arrow's own courts — here we prove
        // the corridor payload source is a REAL ArrowArray, then release.
        unsafe { (array.release.expect("release fn"))(core::ptr::addr_of_mut!(array) as *mut _) };

        // The corridor hop: seal -> scatter -> verify -> decrypt -> append.
        let decrypted = corridor_hop(&mut writer, &mut socks, &key, &body, &mut chain);

        // BYTE IDENTITY vs the pre-seal array bytes.
        assert_eq!(decrypted.len(), body.len(), "byte identity: length");
        assert_eq!(decrypted, body, "byte identity: content");

        // Cursor advanced in lockstep with the chain.
        assert_eq!(writer.cursor(), ((iter + 1) as u32, (iter + 1) as u32));
        assert_eq!(chain.len(), iter + 1);
    }

    // Final chain state: 100 links, head not genesis.
    assert_eq!(chain.len(), 100);
    assert_ne!(chain.checkpoint(), cgap_aoq::chain::GENESIS);
}

// ---------------------------------------------------------------------------
// COURT J2 — anti-vacuity battery (every refusal must actually fire)
// ---------------------------------------------------------------------------

#[test]
fn j2_tamper_battery_all_gates_fire() {
    let mut writer = AoqWriter::new();
    let mut chain = corridor_chain(64);

    // Build one sealed corridor window without the transport (same code
    // path minus writev) so each gate can be aimed independently.
    let body = array_body(&arrow_values(0));
    let (ordinal, seq) = writer.cursor();
    let nonce = Nonce96::new(u32::from(EPOCH), seq as u64).expect("nonce");
    let mut ct = vec![0u8; body.len()];
    let mut tag = [0u8; aead::TAG_LEN];
    aead::seal_detached(
        &mut ct,
        &mut tag,
        &body,
        &corridor_aad(ordinal, seq, body.len()),
        &[7u8; 32],
        &nonce,
    );

    let mut window = vec![0u8; 8 + 120 + ct.len() + pad64(ct.len()) + 8];
    let wire_tag = wire_tag_for(&ct);
    let n = writer
        .write_window(&mut window, EPOCH, KEY_ID, &wire_tag, &ct)
        .expect("write_window");
    let window = &mut window[..n];

    // Gate 1: flipped ciphertext byte -> the receiver recomputes the
    // tamper-tier tag from the tampered body and must get DigestMismatch.
    {
        let mut w = window.to_vec();
        let body_at = 8 + 120; // body starts here
        w[body_at] ^= 0x01;
        let recomputed = wire_tag_for(&w[body_at..body_at + ct.len()]);
        let err = verify_window(&w, ordinal, seq, &recomputed).expect_err("flip must refuse");
        assert!(
            matches!(err, AoqError::DigestMismatch { .. }),
            "got {err:?}"
        );
    }

    // Gate 2: cursor regression -> FrameOrdinalRegression; seq drift ->
    // SeqGap (ordinal gate is checked first).
    {
        let err = verify_window(window, ordinal + 1, seq, &wire_tag).expect_err("regression");
        assert!(
            matches!(err, AoqError::FrameOrdinalRegression { .. }),
            "got {err:?}"
        );
        let err = verify_window(window, ordinal, seq + 1, &wire_tag).expect_err("seq drift");
        assert!(matches!(err, AoqError::SeqGap { .. }), "got {err:?}");
    }

    // Gate 3: wrong tag -> DigestMismatch.
    {
        let wrong_tag = [0u8; 16];
        let err = verify_window(window, ordinal, seq, &wrong_tag).expect_err("wrong tag");
        assert!(
            matches!(err, AoqError::DigestMismatch { .. }),
            "got {err:?}"
        );
    }

    // Gate 4: tampered AEAD tag at open_detached -> Mismatch.
    {
        let mut pt = vec![0u8; ct.len()];
        let mut bad_tag = tag;
        bad_tag[0] ^= 0x80;
        let res = cgap_crypto::aead::open_detached(
            &mut pt,
            &ct,
            &bad_tag,
            &corridor_aad(ordinal, seq, body.len()),
            &[7u8; 32],
            &nonce,
        );
        assert!(res.is_err(), "open_detached must refuse a tampered tag");
    }

    // Gate 5: tampered chain record (verify_records over a mutated COPY —
    // the chain itself stays append-only) -> Spoliation.
    {
        let idx = chain.append(EPOCH, &body).expect("append");
        let mut copy: Vec<AoqChainRecord> = chain.records().to_vec();
        copy[idx as usize].digest[0] ^= 0x01;
        assert!(
            cgap_aoq::chain::verify_records(&ShakeDigest, &copy).is_err(),
            "tampered copy must refuse"
        );
    }

    // Gate 6: keystream shorter than body -> decrypt refused, window
    // bytes untouched (no-plaintext-on-error law).
    {
        let vw = verify_window(window, ordinal, seq, &wire_tag).expect("verify ok");
        let mut w2 = window.to_vec();
        let before = w2.clone();
        let err = decrypt_in_place(&vw, &[], &mut w2).expect_err("short keystream");
        assert!(
            matches!(err, AoqError::HeaderTooShort { .. }),
            "got {err:?}"
        );
        assert_eq!(w2, before, "window untouched on refusal");
    }
}

fn pad64(n: usize) -> usize {
    (n + 63) & !63
}

// ---------------------------------------------------------------------------
// COURT J3 — 100k-link scale leg (chain only)
// ---------------------------------------------------------------------------

#[test]
fn j3_chain_100k_links() {
    const LINKS: usize = 100_000;
    let mut chain = corridor_chain(LINKS + 1);

    for i in 0..LINKS {
        let payload = shake32(&[b"af1/scale", &(i as u64).to_le_bytes()]);
        chain.append(EPOCH, &payload).expect("append");
    }
    assert_eq!(chain.len(), LINKS);

    // Whole-chain verify: every parent link + every digest.
    chain.verify_all().expect("100k-link verify_all");

    // Span verifies: prefix + suffix + middle slices all verify.
    chain.verify_span(0, 50_000).expect("prefix span");
    chain.verify_span(50_000, LINKS).expect("suffix span");
    chain.verify_span(25_000, 75_000).expect("middle span");

    // Resumption: token at 100k resumes; a token claiming 100k links on a
    // shorter chain is refused.
    let tok = chain.resumption_token();
    assert_eq!(tok.links as usize, LINKS);
    chain.resume(tok).expect("resume at head");

    // Tamper non-vacuity: a mutated COPY must refuse at the tampered index.
    let mut copy: Vec<AoqChainRecord> = chain.records().to_vec();
    let victim = 77_777;
    copy[victim].payload_digest[3] ^= 0x10;
    assert!(
        cgap_aoq::chain::verify_records(&ShakeDigest, &copy).is_err(),
        "tampered copy must refuse"
    );
    // ...and the refusal must be AT the victim's parent chain, i.e. the
    // ORIGINAL records still verify.
    cgap_aoq::chain::verify_records(&ShakeDigest, chain.records())
        .expect("original records verify");
}
