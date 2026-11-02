//! L2 court — parquet FNV-1a equity + committed-window byte flip
//! (07-court-corpus.md §2, L2).
//!
//! Subjects: `crates/cgap-parquet/src/footer.rs:119-127` `fnv1a64_seeded`
//! and `verify_seal` (:429). A reference byte-wise FNV-1a written in this
//! file (no kernel code in the oracle path) must be byte-exact-equal to
//! `fnv1a64_seeded` over corpora {empty, 1..16 bytes, 4097 bytes}; the W3
//! SWAR rewrite must keep this green. Plus: flip one committed-window
//! (body) byte in a sealed block and `verify_seal` must return
//! `SealError::ChecksumMismatch { region: CorruptRegion::Body }`.
//!
//! Chicago discipline: real bytes, real digests, final state asserted
//! exactly — no mocks (pure kernel).

use cgap_parquet::footer::{verify_seal, CorruptRegion, FooterSealer, SealError};

/// Reference FNV-1a-64 (FNV-1a variant, 64-bit, wrapping arithmetic) —
/// the FNV test-vector oracle, written independently of the kernel loop.
fn reference_fnv1a64_seeded(data: &[u8], seed: u64) -> u64 {
    const PRIME: u64 = 0x0000_0100_0000_01B3;
    let mut h = seed;
    for &b in data {
        h ^= u64::from(b);
        h = h.wrapping_mul(PRIME);
    }
    h
}

/// Published FNV-1a-64 test vectors (draft-ietf-tls-fnv / reference
/// standard): "" -> 0xcbf29ce484222325, "a" -> 0xaf63dc4c8601ec8c,
/// "foobar" -> 0x85944171f73967e8. Byte-exact, no tolerance.
#[test]
fn l2_fnv_published_vectors() {
    assert_eq!(cgap_parquet::footer::fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(cgap_parquet::footer::fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(
        cgap_parquet::footer::fnv1a64(b"foobar"),
        0x8594_4171_f739_67e8
    );
}

/// Equity court: reference oracle == kernel `fnv1a64_seeded` over the
/// corpus {empty, 1..=16 bytes, 4097 bytes}, byte-exact, multiple seeds.
/// The W3 SWAR rewrite must keep this green (any lane-lane divergence
/// fails here, not in the field).
#[test]
fn l2_reference_equity_over_corpus() {
    let mut corpus: Vec<Vec<u8>> = Vec::new();
    corpus.push(Vec::new()); // empty
    for n in 1..=16usize {
        corpus.push((0..n).map(|i| (i as u8) ^ 0x5A).collect());
    }
    // 4097 bytes: past the 4096 SWAR-lane boundary the rewrite targets.
    corpus.push((0..4097usize).map(|i| (i * 31 % 251) as u8).collect());
    let seeds: [u64; 5] = [
        0xcbf2_9ce4_8422_2325,
        0,
        u64::MAX,
        0xA20C_0FFE_E5A7_E000, // frozen corpus seed
        0x1234_5678_9abc_def0,
    ];
    for data in &corpus {
        for &seed in &seeds {
            let want = reference_fnv1a64_seeded(data, seed);
            let got = cgap_parquet::footer::fnv1a64_seeded(data, seed);
            assert_eq!(got, want, "len={} seed={seed:#x}", data.len());
        }
    }
}

/// A4 byte-exactness court for the interleaved `block_digest` lever
/// (v26.11.2): the one-sweep 4-lane form must equal the sequential 4-pass
/// oracle (this file, real kernel constants, no kernel loop) lane-for-lane
/// over the corpus {empty, 1..=16 bytes, 4097 bytes}. Anti-vacuity
/// mutation (witnessed in the A4 sandbox): collapse lane 2's seed onto
/// lane 1's — this assert MUST fail (a symmetric seal/verify mutation is
/// invisible to every consistency pin, so this equity pin is the only
/// flip channel).
#[test]
fn a4_block_digest_interleaved_equity() {
    const PRIME: u64 = 0x0000_0100_0000_01B3;
    const BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut corpus: Vec<Vec<u8>> = Vec::new();
    corpus.push(Vec::new()); // empty
    for n in 1..=16usize {
        corpus.push((0..n).map(|i| (i as u8) ^ 0x5A).collect());
    }
    corpus.push((0..4097usize).map(|i| (i * 31 % 251) as u8).collect());
    for data in &corpus {
        let mut want = [0u8; 32];
        for i in 0..4u64 {
            let mut h = BASIS ^ i.wrapping_mul(GOLDEN);
            for &b in data {
                h ^= u64::from(b);
                h = h.wrapping_mul(PRIME);
            }
            want[(i * 8) as usize..(i * 8 + 8) as usize].copy_from_slice(&h.to_le_bytes());
        }
        assert_eq!(
            cgap_parquet::footer::block_digest(data),
            want,
            "block_digest drifted at len={}",
            data.len()
        );
    }
}

/// Kill signal: one committed-window (body) byte flip in a sealed block
/// must fail verify_seal with ChecksumMismatch { region: Body } — the
/// corruption localizes to the body, not the footer.
#[test]
fn l2_body_byte_flip_is_checksum_mismatch_body() {
    // One row group, span = whole body.
    let body_len = 128usize;
    // Buffer must also hold the meta + 96 B trailer the sealer appends.
    let mut buf = vec![0u8; body_len + 256];
    for b in buf[..body_len].iter_mut() {
        *b = 0u8;
    }
    for (i, b) in buf[..body_len].iter_mut().enumerate() {
        *b = (i % 251) as u8;
    }

    let summary = cgap_parquet::footer::BlockSummary {
        row_groups: 1,
        rows: 4,
        spans: &[body_len as u32],
    };
    let sealed = FooterSealer::new(&mut buf, body_len, summary, None)
        .expect("seal construction")
        .seal()
        .expect("seal");
    let clean: Vec<u8> = sealed.bytes().to_vec();
    let digest: [u8; 32] = *sealed.digest();
    // Read-back: clean block verifies.
    assert!(verify_seal(&clean, &digest).is_ok());

    // Flip one committed-window byte.
    let mut flipped = clean.clone();
    let at = body_len / 2;
    flipped[at] ^= 0x01;
    assert_ne!(flipped[at], clean[at]);
    match verify_seal(&flipped, &digest) {
        Err(SealError::ChecksumMismatch { region }) => {
            assert_eq!(region, CorruptRegion::Body);
        }
        other => panic!("expected ChecksumMismatch{{Body}}, got {other:?}"),
    }
}
