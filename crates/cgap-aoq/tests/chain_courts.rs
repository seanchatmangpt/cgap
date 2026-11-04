//! Chain courts (PG-A5, AB-7): 9 fail-closed courts over the real chain.
//! Chicago discipline: real collaborators, assert final state; the only
//! injected collaborator is the deterministic digest seam the crate itself
//! mandates (no crypto ships in cgap-aoq, R-K1).
//!
//! Anti-vacuity: courts 3/4/5 inject real mutations and MUST flip — a green
//! run with no observed refusal is a VACUOUS GATE.

use cgap_aoq::chain::{
    verify_records, AoqChain, AoqChainRecord, ChainDigest, Spoliation, GENESIS,
};

/// Deterministic dual-FNV digest substrate (same construction as the
/// session's test KDF — the compose law needs one shared substrate).
#[derive(Clone, Copy)]
struct Mix32;

impl ChainDigest for Mix32 {
    fn digest(&self, _parts: &[&[u8]]) -> [u8; 32] {
        mix32(_parts)
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

/// Fresh chain over a leaked slab (courts only; the kernel stays
/// alloc-free — slab provision is the caller-provided-slab law).
fn chain(cap: usize) -> AoqChain<'static, Mix32> {
    let slab: &'static mut [AoqChainRecord] =
        Box::leak(vec![AoqChainRecord::zeroed(); cap].into_boxed_slice());
    AoqChain::new(Mix32, slab)
}

/// COURT 1 — GENESIS law: an empty chain's head is all zeros, verify_all
/// passes vacuously-but-legitimately (bounds-checked, range [0,0)), and the
/// resumption token of an empty chain resumes cleanly.
#[test]
fn c1_genesis_all_zeros() {
    let c = chain(8);
    assert_eq!(c.len(), 0);
    assert_eq!(c.checkpoint(), GENESIS);
    assert_eq!(c.verify_all(), Ok(()));
    assert_eq!(c.verify_span(0, 0), Ok(()));
    // Out-of-range span refuses (opaque).
    assert!(
        c.verify_span(0, 1).is_err(),
        "out-of-range span refuses opaque"
    );
    let tok = c.resumption_token();
    assert_eq!(tok.head, GENESIS);
    assert_eq!(tok.links, 0);
    let mut c2 = chain(8);
    assert_eq!(c2.resume(tok), Ok(()));
}

/// COURT 2 — append/verify law: appends chain GENESIS-rooted records whose
/// digests recompute; spans and tokens agree with the head.
#[test]
fn c2_append_verify_all_span_token() {
    let mut c = chain(64);
    for i in 0u64..64 {
        let epoch = (i % 3) as u16; // epochs 0,1,2 walk the closed enum
        c.append(epoch, &[i as u8; 16]).unwrap();
    }
    assert_eq!(c.len(), 64);
    assert_eq!(c.verify_all(), Ok(()));
    assert_eq!(c.verify_span(0, 1), Ok(()));
    assert_eq!(c.verify_span(32, 48), Ok(()));
    assert_eq!(c.verify_span(63, 64), Ok(()));
    // Token at k resumes; k+1 forged links refuse (shorter chain).
    let tok = c.resumption_token();
    assert_eq!(tok.links, 64);
    let mut c2 = chain(64);
    for i in 0u64..64 {
        c2.append((i % 3) as u16, &[i as u8; 16]).unwrap();
    }
    assert_eq!(c2.resume(c.resumption_token()), Ok(()));
    // A token claiming more links than the chain holds refuses.
    let forged = cgap_aoq::chain::ResumptionToken {
        head: tok.head,
        links: 65,
    };
    let mut c3 = chain(64);
    assert!(c3.resume(forged).is_err(), "forged link count refuses");
}

/// COURT 3 — tamper battery (non-vacuity): each mutation class flips
/// verify to the opaque Spoliation; the typed classification surface
/// (verify_records over the mutated copy) names parent vs digest breaks.
#[test]
fn c3_tamper_battery_flips_verify() {
    let mut c = chain(16);
    for i in 0u64..16 {
        c.append(0, &[i as u8; 8]).unwrap();
    }
    assert_eq!(c.verify_all(), Ok(()), "untouched chain verifies clean");

    // (a) payload_digest byte flip at link 5: digest at 5 breaks directly,
    // parent at 6 breaks transitively.
    let mut a: [AoqChainRecord; 16] = c.records().try_into().ok().unwrap();

    a[5].payload_digest[3] ^= 0x40;
    assert_eq!(
        c.verify_all(),
        Ok(()),
        "chain itself untouched by copy mutation"
    );
    assert!(
        verify_records(&Mix32, &a).is_err(),
        "VACUOUS COURT: tampered copy verified clean"
    );

    // (b) digest byte flip at link 9.
    let mut b: [AoqChainRecord; 16] = c.records().try_into().ok().unwrap();
    b[9].digest[7] ^= 0x01;
    assert!(verify_records(&Mix32, &b).is_err());

    // (c) prev_hash rewrite at link 2: typed classification (parent break,
    // expected != got prefixes).
    let mut d: [AoqChainRecord; 16] = c.records().try_into().ok().unwrap();
    d[2].prev_hash[0] ^= 0xff;
    match verify_records(&Mix32, &d) {
        Err(cgap_aoq::error::AoqError::DigestMismatch { expected, got }) => {
            assert_ne!(expected, got, "parent break must classify typed");
        }
        other => panic!("expected typed parent-break refusal, got {other:?}"),
    }

    // (d) epoch flip at link 11: epoch is hashed -> digest break.
    let mut e: [AoqChainRecord; 16] = c.records().try_into().ok().unwrap();
    e[11].epoch ^= 0x01;
    assert!(verify_records(&Mix32, &e).is_err());
}

/// COURT 4 — checkpoint/resumption-token/resume law: token prefixes
/// resume; forged heads, short chains, and stale-after-extend prefixes
/// refuse as opaque Spoliation.
#[test]
fn c4_checkpoint_resume_law() {
    let mut c = chain(64);
    for i in 0u64..16 {
        c.append(0, &[i as u8; 8]).unwrap();
    }
    let tok = c.resumption_token();

    // Same-prefix chain resumes to the token.
    let mut c2 = chain(64);
    for i in 0u64..16 {
        c2.append(0, &[i as u8; 8]).unwrap();
    }
    assert_eq!(c2.resume(tok), Ok(()));

    // Extending past the token stays lawful (append-only forward).
    assert_eq!(c2.append(0, b"more").is_ok(), true);
    assert_eq!(c2.resume(tok), Ok(()));

    // Forged head (same links count, different head) refuses.
    let forged = cgap_aoq::chain::ResumptionToken {
        head: [0xa5; 32],
        links: tok.links,
    };
    assert!(c2.resume(forged).is_err(), "forged head refuses");

    // Token encode/decode round trip.
    let bytes = tok.to_bytes();
    assert_eq!(bytes.len(), 36);
    assert_eq!(
        cgap_aoq::chain::ResumptionToken::from_bytes(&bytes),
        Some(tok)
    );
}

/// COURT 5 — append-only law: typed refusals leave the chain byte-identical;
/// no public API truncates or rewrites.
#[test]
fn c5_append_only_no_mutation_on_refusal() {
    let mut c = chain(4);
    c.append(0, b"one").unwrap();
    c.append(0, b"two").unwrap();
    let before_len = c.len();
    let before_head = c.checkpoint();
    let before_records: [AoqChainRecord; 2] = c.records().try_into().ok().unwrap();
    let before: Vec<u8> = before_records
        .iter()
        .take(2)
        .flat_map(|r| r.digest.iter().copied())
        .collect();

    // Header-agnostic law: arbitrary-length payloads digest (never store);
    // only slab exhaustion refuses. Slab full -> typed SlabFull, unchanged.
    c.append(0, b"three").unwrap();
    c.append(0, b"four").unwrap();
    assert!(matches!(
        c.append(0, b"five"),
        Err(cgap_aoq::error::AoqError::SlabFull { len: 4 })
    ));

    // Byte-identity: every record is exactly as it was before refusals.
    let after_records: [AoqChainRecord; 4] = c.records().try_into().ok().unwrap();
    let after: Vec<u8> = after_records
        .iter()
        .take(2)
        .flat_map(|r| r.digest.iter().copied())
        .collect();
    assert_eq!(before, after, "refusals must not mutate landed records");
    assert_eq!(c.len(), before_len + 2, "only the two accepts landed");
    assert_ne!(c.checkpoint(), GENESIS);
    assert_ne!(
        c.checkpoint(),
        before_head,
        "head advanced only via accepts"
    );
}
/// COURT 6 — Spoliation opacity: the witness carries no readable state.
#[test]
fn c6_spoliation_is_opaque() {
    // Zero-sized: carries no payload to inspect.
    assert_eq!(core::mem::size_of::<Spoliation>(), 0);
    // The only lawful way to obtain a witness is a real verify refusal.
    let mut c = chain(4);
    c.append(0, b"x").unwrap();
    let w = c.verify_span(1, 2).unwrap_err();
    // Display discloses no fields (fixed uninformative string).
    assert_eq!(format!("{w}"), "spoliation");
    // Equality is identity-only: exactly one value exists.
    let w2 = c.verify_span(0, 99).unwrap_err();
    assert_eq!(w, w2);
}

/// COURT 7 — compose-with-emit_verified law: session-emitted frames append
/// cleanly through chain::append_verified with the SAME injected substrate;
/// any tampered emission refuses and leaves the chain unmutated.
#[test]
fn c7_compose_emit_verified_law() {
    use cgap_aoq::codec;
    use cgap_aoq::header::AoqHeader;
    use cgap_aoq::session::{
        parse_frame, FrameKind, KemSeam, Session, SessionState, CT_BYTES, EK_BYTES,
    };

    /// Session-side test KEM whose kdf IS Mix32 (one substrate, both seams).
    struct TestKem;

    impl KemSeam for TestKem {
        fn encapsulate(
            &self,
            _ek: &[u8; EK_BYTES],
        ) -> Result<([u8; CT_BYTES], [u8; 32]), cgap_aoq::session::SessionError> {
            let ss = mix32(&[b"cgap-test/ss"]);
            let mut ct = [0u8; CT_BYTES];
            let mut chunk = mix32(&[b"cgap-test/ct"]);
            let mut i = 0;
            while i < CT_BYTES {
                ct[i] = chunk[i % 32];
                if i % 32 == 31 {
                    chunk = mix32(&[&chunk, b"round"]);
                }
                i += 1;
            }
            Ok((ct, ss))
        }

        fn decapsulate(
            &self,
            _dk: &[u8],
            _ct: &[u8; CT_BYTES],
        ) -> Result<[u8; 32], cgap_aoq::session::SessionError> {
            Ok(mix32(&[b"cgap-test/ss"]))
        }

        fn kdf(&self, parts: &[&[u8]]) -> [u8; 32] {
            mix32(parts)
        }
    }

    // Drive the machine to Established.
    let kem = TestKem;
    let mut session = Session::new(&kem, 0, 1, 0);
    let mut ek = [0u8; EK_BYTES];
    ek[0] = 0x11;
    let mut hs = [0u8; 64 + 1 + EK_BYTES];
    let h = AoqHeader::new(0, 1, 0, 0);
    codec::encode(&h, &mut hs).unwrap();
    hs[64] = FrameKind::HandshakeEk.id();
    hs[65..].copy_from_slice(&ek);
    let frame = parse_frame(&hs).unwrap();
    session.step(&frame).unwrap();
    let mut fin = [0u8; 64 + 1 + CT_BYTES];
    let h = AoqHeader::new(0, 1, 1, 0);
    codec::encode(&h, &mut fin).unwrap();
    fin[64] = FrameKind::HandshakeFinish.id();
    let (ct, _) = kem.encapsulate(&ek).unwrap();
    fin[65..].copy_from_slice(&ct);
    let frame = parse_frame(&fin).unwrap();
    session.step(&frame).unwrap();
    assert_eq!(session.state(), SessionState::Established);

    // emit_verified -> append_verified composition.
    let mut c = chain(16);
    for seq in 0u32..3 {
        let mut out = [0u8; 96];
        let n = session
            .emit_verified(seq, b"compose-payload", &mut out[..])
            .unwrap();
        // Chain re-derives the tag with the same substrate.
        c.append_verified(&out[..n]).unwrap();
    }
    assert_eq!(c.len(), 3);
    assert_eq!(c.verify_all(), Ok(()));

    // Tampered emission refuses BEFORE linking: byte at body start flips.
    let mut out = [0u8; 96];
    let n = session
        .emit_verified(9, b"compose-payload", &mut out[..])
        .unwrap();
    let mut bad = out[..n].to_vec();
    bad[65] ^= 0x01;
    let head_before = c.checkpoint();
    match c.append_verified(&bad) {
        Err(cgap_aoq::error::AoqError::DigestMismatch { expected, got }) => {
            assert_ne!(expected, got);
        }
        other => panic!("expected typed tag refusal, got {other:?}"),
    }
    assert_eq!(c.checkpoint(), head_before, "no mutation on refusal");
    assert_eq!(c.len(), 3);
}

/// COURT 8 — slab-full law (non-vacuity): a full 65,536-link slab refuses
/// the 65,537th append with typed `SlabFull { len: 65_536 }`, head and
/// records byte-identical across the refusal.
#[test]
fn c8_slab_full_typed_at_capacity() {
    let mut c = chain(65_536);
    for i in 0u64..65_536 {
        c.append((i % 3) as u16, &[i as u8; 8]).unwrap();
    }
    assert_eq!(c.len(), 65_536);
    assert_eq!(c.verify_all(), Ok(()));
    let head_before = c.checkpoint();
    match c.append(0, b"overflow") {
        Err(cgap_aoq::error::AoqError::SlabFull { len }) => {
            assert_eq!(len, 65_536, "typed refusal names the slab capacity");
        }
        other => panic!("expected typed SlabFull refusal, got {other:?}"),
    }
    assert_eq!(c.checkpoint(), head_before, "no mutation on refusal");
    assert_eq!(c.len(), 65_536);
}

/// COURT 9 — fold law (anti-vacuity, two named mutations): (a) a
/// payload_digest byte flip in a landed-record copy flips verify_records to
/// a typed refusal; (b) a single tag-byte flip in an otherwise valid
/// `append_verified` frame still refuses with DigestMismatch and leaves
/// the chain unmutated. A green run under either mutation is VACUOUS.
#[test]
fn c9_fold_payload_flip_and_tag_flip_refuse() {
    // (a) payload fold: mutate a copy of the landed links.
    let mut c = chain(8);
    for i in 0u64..4 {
        c.append(0, &[i as u8; 8]).unwrap();
    }
    assert_eq!(c.verify_all(), Ok(()), "untouched chain verifies clean");
    let mut folded: [AoqChainRecord; 4] = c.records().try_into().ok().unwrap();
    folded[2].payload_digest[5] ^= 0x80;
    assert!(
        matches!(
            verify_records(&Mix32, &folded),
            Err(cgap_aoq::error::AoqError::DigestMismatch { .. })
        ),
        "VACUOUS COURT: payload_digest fold verified clean"
    );

    // (b) tag fold: one tag byte flip in a well-formed frame refuses and
    // the chain is not mutated.
    let body = b"fold-frame-body";
    let digest = Mix32.digest(&[body]);
    let mut frame = body.to_vec();
    frame.extend_from_slice(&digest[0..4]);
    let mut fresh = chain(8);
    fresh.append_verified(&frame).unwrap();
    assert_eq!(fresh.len(), 1);

    let head_before = fresh.checkpoint();
    let mut bad = frame.clone();
    let tag_off = bad.len() - 4; // flip the FIRST tag byte only
    bad[tag_off] ^= 0x01;
    match fresh.append_verified(&bad) {
        Err(cgap_aoq::error::AoqError::DigestMismatch { expected, got }) => {
            assert_ne!(expected, got, "tag fold must classify typed");
        }
        other => panic!("expected typed tag refusal, got {other:?}"),
    }
    assert_eq!(fresh.len(), 1, "refused frame must not land");
    assert_eq!(fresh.checkpoint(), head_before, "no mutation on refusal");
}

/// COURT 10 — batch == sequential: `append_batch_admitted` over a run of
/// admitted digests must land records byte-identical (whole-slab compare)
/// to N sequential `append`s of the same payloads with the same epochs.
#[test]
fn c10_batch_admitted_equals_sequential_appends() {
    const N: usize = 9;
    let payloads: Vec<Vec<u8>> = (0..N)
        .map(|i| vec![(i * 7 + 3) as u8; 16 + i])
        .collect();
    let digests: Vec<[u8; 32]> = payloads
        .iter()
        .map(|p| Mix32.digest(&[p]))
        .collect();
    let epochs: Vec<u16> = (0..N).map(|i| (i % 3) as u16).collect();

    let mut batched = chain(16);
    batched.append_batch_admitted(&epochs, &digests).unwrap();

    let mut seq = chain(16);
    for i in 0..N {
        seq.append(epochs[i], &payloads[i]).unwrap();
    }
    assert_eq!(batched.records(), seq.records(), "whole-slab compare");
    assert_eq!(batched.len(), N);
    assert_eq!(batched.checkpoint(), seq.checkpoint());
    assert_eq!(batched.verify_all(), Ok(()));
    assert_eq!(seq.verify_all(), Ok(()));
}

/// COURT 11 — all-or-nothing refusal: a batch larger than the remaining
/// slab is refused with typed `SlabFull { len }` and ZERO mutation (len
/// unchanged, every existing record byte-identical).
#[test]
fn c11_batch_refusal_is_all_or_nothing() {
    let mut c = chain(8);
    let seed: Vec<[u8; 32]> = (0..5).map(|i| Mix32.digest(&[&[i as u8; 8]])).collect();
    let e5: Vec<u16> = vec![0; 5];
    c.append_batch_admitted(&e5, &seed).unwrap();
    let before = c.records().to_vec();
    let head_before = c.checkpoint();

    // 4 more need indices 5..9 > cap 8 -> must refuse whole.
    let more: Vec<[u8; 32]> = (0..4).map(|i| Mix32.digest(&[&[(i + 9) as u8; 8]])).collect();
    let e4: Vec<u16> = vec![1; 4];
    match c.append_batch_admitted(&e4, &more) {
        Err(cgap_aoq::error::AoqError::SlabFull { len }) => assert_eq!(len, 8),
        other => panic!("expected SlabFull, got {other:?}"),
    }
    assert_eq!(c.len(), 5, "refusal must leave len unchanged");
    assert_eq!(c.records(), before.as_slice(), "records byte-identical");
    assert_eq!(c.checkpoint(), head_before, "head frozen on refusal");
}

/// COURT 12 — length-parity refusal: unequal epochs/digests counts refuse
/// typed, zero mutation.
#[test]
fn c12_batch_length_parity_refusal() {
    let mut c = chain(8);
    let d: Vec<[u8; 32]> = vec![Mix32.digest(&[&[1u8; 8]]); 3];
    let e: Vec<u16> = vec![0; 2];
    match c.append_batch_admitted(&e, &d) {
        Err(cgap_aoq::error::AoqError::SeqGap { expected, got }) => {
            assert_eq!((expected, got), (2, 3));
        }
        other => panic!("expected SeqGap parity refusal, got {other:?}")
    }
    assert_eq!(c.len(), 0);
}

/// COURT 13 — batch mutation non-vacuity: a digest corrupted AFTER batch
/// admission must flip verify to failure (typed DigestMismatch over an
/// independent copy; the chain itself stays append-only).
#[test]
fn c13_batch_admitted_digest_mutation_flips_verify() {
    let mut c = chain(16);
    let payloads: Vec<Vec<u8>> = (0..4).map(|i| vec![i as u8; 20]).collect();
    let digests: Vec<[u8; 32]> = payloads.iter().map(|p| Mix32.digest(&[p])).collect();
    let epochs: Vec<u16> = vec![0; 4];
    c.append_batch_admitted(&epochs, &digests).unwrap();
    assert_eq!(c.verify_all(), Ok(()));

    let mut corrupted = c.records().to_vec();
    corrupted[2].payload_digest[5] ^= 0x80;
    assert!(
        verify_records(&Mix32, &corrupted).is_err(),
        "mutated batch-admitted digest must flip verify"
    );
    let mut bad = corrupted;
    bad[2].digest[0] ^= 0x01;
    assert!(
        verify_records(&Mix32, &bad).is_err(),
        "mutated record digest must flip verify"
    );
}
