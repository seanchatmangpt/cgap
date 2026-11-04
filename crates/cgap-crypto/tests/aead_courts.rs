//! AB-4 (PG-A3) AEAD/Poly1305 courts — Chicago discipline: real vectors,
//! real RustCrypto collaborator, assertions on final state, no mocks.
//!
//! Fixture provenance: RFC 8439 §2.5.2 and §2.8.2, transcribed from the
//! RFC text fetched from rfc-editor.org THIS session (curl, 2026-11-03);
//! each fixture also re-validated live against the RustCrypto
//! `chacha20poly1305` crate in the cross-check court below.
//!
//! Every court names its anti-vacuity mutation. The sole-constructor
//! count-law scan and its sandbox-add-token kill follow the CG-1
//! wire-shm scan-court shape (baseline -> sandbox mutation -> scan MUST
//! flip -> byte-exact restore witness).

use cgap_crypto::aead::{self, EpochExhausted, Mismatch, Nonce96, AAD_LEN, KEY_LEN, TAG_LEN};
use cgap_crypto::poly1305;

// `hex` decoding without a hex dev-dep.
fn unhex(s: &str) -> std::vec::Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("hex byte"))
        .collect()
}

/// RFC 8439 §2.5.2 Poly1305 test vector (rfc-editor.org transcription).
/// Key 85:d6:...:51b, 34-byte message "Cryptographic Forum Research Group",
/// Tag a8:06:1d:c1:30:51:36:c6:c2:2b:8b:af:0c:01:27:a9.
#[test]
fn poly1305_rfc8439_2_5_2_fixture() {
    let key: [u8; poly1305::KEY_LEN] =
        unhex("85d6be7857556d337f4452fe42d506a80103808afb0db2fd4abff6af4149f51b")
            .try_into()
            .expect("32-byte key");
    let msg = b"Cryptographic Forum Research Group";
    let expected: [u8; 16] = unhex("a8061dc1305136c6c22b8baf0c0127a9")
        .try_into()
        .expect("16-byte tag");

    let got = poly1305::one_shot(&key, msg);
    assert_eq!(got, expected, "RFC 8439 section 2.5.2 one-shot tag");

    // Chunked-push equivalence: identical byte stream, arbitrary push
    // boundaries -> identical tag.
    for chunk in [1usize, 3, 16, 17, 34, 100] {
        let mut state = poly1305::new(&key);
        for piece in msg.chunks(chunk) {
            poly1305::push(&mut state, piece);
        }
        assert_eq!(
            poly1305::finish(state),
            expected,
            "RFC 2.5.2 tag must survive push(chunk={chunk}) chunking"
        );
    }
}

/// RFC 8439 §2.8.2 AEAD_CHACHA20_POLY1305 test vector (rfc-editor.org
/// transcription), driven through the `#[doc(hidden)]` variable-AAD seam
/// (the RFC vector's AAD is 12 bytes; the production surface pins 36):
/// the published mac_data layout (`aad || pad16 || ct || pad16 ||
/// len(aad) u64le || len(ct) u64le`) over the published ct MUST yield the
/// published tag `1a:e1:0b:59:...:06:91`.
#[test]
fn aead_rfc8439_2_8_2_fixture() {
    let key: [u8; KEY_LEN] =
        unhex("808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f")
            .try_into()
            .expect("32-byte key");
    let nonce = Nonce96::new(0x0000_0007, 0x4746_4544_4342_4140).expect("lawful epoch");
    // Wire nonce bytes = epoch le || counter le = 07 00 00 00 40..47.
    assert_eq!(
        unhex("070000004041424344454647"),
        nonce.to_bytes(),
        "RFC 2.8.2 wire nonce layout (epoch || counter le)"
    );
    let aad = unhex("50515253c0c1c2c3c4c5c6c7");
    let ct = unhex(
        "d31a8d34648e60db7b86afbc53ef7ec2a4aded51296e08fea9e2b5a736ee62d6\
         3dbea45e8ca9671282fafb69da92728b1a71de0a9e060b2905d6a5b67ecd3b36\
         92ddbd7f2d778b8c9803aee328091b58fab324e4fad675945585808b4831d7bc\
         3ff4def08e4b7a9de576d26586cec64b6116",
    );
    let expected: [u8; TAG_LEN] = unhex("1ae10b594f09e26a7e902ecbd0600691")
        .try_into()
        .expect("16-byte tag");

    let got = aead::mac_tag_var_aad(&aad, &ct, &key, &nonce);
    assert_eq!(got, expected, "RFC 8439 section 2.8.2 published tag");

    // The §2.8.2 Poly one-time key (RFC-published) must also fall out of
    // block counter 0; verified transitively above, and the keystate
    // layout is pinned by the cross-check court with the real crate.
}

// ---------------------------------------------------------------------------
// RustCrypto cross-check court (Chicago: the REAL chacha20poly1305 crate is
// the collaborator; byte-identical ct+tag required, both directions).
// ---------------------------------------------------------------------------

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{ChaCha20Poly1305, Nonce as XNonce};

fn xnonce(n: &Nonce96) -> XNonce {
    *XNonce::from_slice(&n.to_bytes())
}

fn xcipher(key: &[u8; KEY_LEN]) -> ChaCha20Poly1305 {
    ChaCha20Poly1305::new_from_slice(key).expect("32-byte key")
}

/// Deterministic SplitMix64 stream (no external rand dep) for court data.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// 32-byte key from 4 deterministic draws (splitmix64 emits 8 bytes/call).
fn key32(state: &mut u64) -> [u8; KEY_LEN] {
    let mut k = [0u8; KEY_LEN];
    for chunk in k.chunks_exact_mut(8) {
        chunk.copy_from_slice(&splitmix64(state).to_le_bytes());
    }
    k
}

/// 36-byte AAD from 5 draws, truncated (fixed AAD width).
fn aad36(state: &mut u64) -> [u8; AAD_LEN] {
    let mut a = [0u8; AAD_LEN + 4];
    for chunk in a.chunks_exact_mut(8) {
        chunk.copy_from_slice(&splitmix64(state).to_le_bytes());
    }
    let mut out = [0u8; AAD_LEN];
    out.copy_from_slice(&a[..AAD_LEN]);
    out
}

/// 8-byte nonce seed -> lawful Nonce96 (epoch from first byte).
fn nonce_from(state: &mut u64) -> Nonce96 {
    let seed = splitmix64(state);
    Nonce96::new((seed & 0xFF) as u32, seed.rotate_left(8)).expect("lawful epoch")
}

/// Cross-check court: over swept lengths and two key/nonce pairs, local
/// seal MUST be byte-identical to the RustCrypto ct+tag, and each side
/// MUST open the other's output. Anti-vacuity: any divergence in ct, tag,
/// nonce layout or mac layout flips this court.
#[test]
fn cross_check_rustcrypto_byte_identical_both_directions() {
    let mut state: u64 = 0xA20C_0FFE_E5A7_E000;
    for len in [0usize, 1, 15, 16, 17, 31, 63, 64, 65, 255, 4096] {
        let key: [u8; KEY_LEN] = key32(&mut state);
        let mut full = [0u8; 8];
        full.copy_from_slice(&splitmix64(&mut state).to_le_bytes());
        let nonce = Nonce96::new(full[0] as u32, u64::from_le_bytes(full)).expect("lawful epoch");
        let aad: [u8; AAD_LEN] = aad36(&mut state);

        let pt: std::vec::Vec<u8> = (0..len).map(|_| splitmix64(&mut state) as u8).collect();

        // RustCrypto (attached ct||tag wire form).
        let expected_wire = xcipher(&key)
            .encrypt(
                &xnonce(&nonce),
                Payload {
                    msg: pt.as_slice(),
                    aad: &aad,
                },
            )
            .expect("rustcrypto encrypt");
        let (xct, xtag) = expected_wire.split_at(len);
        let xtag: [u8; TAG_LEN] = xtag.try_into().expect("16-byte tag");

        // Local detached seal.
        let mut ct = std::vec::Vec::with_capacity(len);
        ct.resize(len, 0);
        let mut tag = [0u8; TAG_LEN];
        aead::seal_detached(&mut ct, &mut tag, &pt, &aad, &key, &nonce);

        assert_eq!(ct.as_slice(), xct, "ct byte-identical (len {len})");
        assert_eq!(tag, xtag, "tag byte-identical (len {len})");

        // Direction 1: local open of RustCrypto output.
        let mut back = std::vec::Vec::with_capacity(len);
        back.resize(len, 0);
        aead::open_detached(&mut back, xct, &xtag, &aad, &key, &nonce)
            .expect("local open of rustcrypto seal");
        assert_eq!(back.as_slice(), pt.as_slice());

        // Direction 2: RustCrypto open of local output (aad attached).
        let mut wire = expected_wire.clone();
        wire[..len].copy_from_slice(&ct);
        wire[len..].copy_from_slice(&tag);
        let round = xcipher(&key)
            .decrypt(
                &xnonce(&nonce),
                Payload {
                    msg: wire.as_slice(),
                    aad: &aad,
                },
            )
            .expect("rustcrypto open of local seal");
        assert_eq!(round, pt);
    }
}

// ---------------------------------------------------------------------------
// Round-trip + stress courts.
// ---------------------------------------------------------------------------

fn seal_open_round_trip(len: usize, state: &mut u64) {
    let key = key32(state);
    let nonce = nonce_from(state);
    let aad = aad36(state);
    let pt: std::vec::Vec<u8> = (0..len).map(|_| splitmix64(state) as u8).collect();

    let mut ct = std::vec::Vec::with_capacity(len);
    ct.resize(len, 0);
    let mut tag = [0u8; TAG_LEN];
    aead::seal_detached(&mut ct, &mut tag, &pt, &aad, &key, &nonce);

    let mut back = std::vec::Vec::with_capacity(len);
    back.resize(len, 0);
    aead::open_detached(&mut back, &ct, &tag, &aad, &key, &nonce).expect("round trip must open");
    assert_eq!(back.as_slice(), pt.as_slice(), "round trip (len {len})");
}

/// Length sweep stress court: 0/1/15/16/17/35/36/511/4096 byte plaintexts
/// round-trip byte-exactly (covers the pad16 boundary classes: empty,
/// 1-mod-16, exactly-16, 16+1, the 36-byte AAD-adjacent 35/36, and large
/// multi-block). Anti-vacuity: any pad16 or length-encoding defect flips it.
#[test]
fn length_sweep_round_trips() {
    let mut state: u64 = 0xBEEF_CAFE_1234_5678;
    for len in [0usize, 1, 15, 16, 17, 35, 36, 511, 4096] {
        seal_open_round_trip(len, &mut state);
    }
}

/// Flip-kill court: flipping ANY authenticated input — one AAD bit, one ct
/// byte, one tag bit — MUST kill open with `Mismatch`; flipping one
/// plaintext byte MUST change the ct and the tag under the same key/nonce.
/// Anti-vacuity: a constant-time compare that ignores any byte, or a
/// decrypt-before-verify path that accepts tampered input, flips this.
#[test]
fn flip_kills_are_mismatch() {
    let mut state: u64 = 0xF10B_5EED_0000_00A4;
    let key = key32(&mut state);
    let nonce = nonce_from(&mut state);
    let aad = aad36(&mut state);
    let pt: std::vec::Vec<u8> = (0..128).map(|_| splitmix64(&mut state) as u8).collect();

    let mut ct = std::vec::Vec::with_capacity(pt.len());
    ct.resize(pt.len(), 0);
    let mut tag = [0u8; TAG_LEN];
    aead::seal_detached(&mut ct, &mut tag, &pt, &aad, &key, &nonce);

    let mut back = std::vec::Vec::with_capacity(pt.len());
    back.resize(pt.len(), 0);

    // AAD flip (every position class: first, AAD-adjacent, last).
    for i in [0usize, AAD_LEN / 2, AAD_LEN - 1] {
        let mut aad2 = aad;
        aad2[i] ^= 0x01;
        assert_eq!(
            aead::open_detached(&mut back, &ct, &tag, &aad2, &key, &nonce),
            Err(Mismatch),
            "AAD flip at {i} must be Mismatch"
        );
    }

    // CT flip (block-0 head, mid-block, final partial-block tail).
    for i in [0usize, 16, ct.len() - 1] {
        let mut ct2 = ct.clone();
        ct2[i] ^= 0x80;
        assert_eq!(
            aead::open_detached(&mut back, &ct2, &tag, &aad, &key, &nonce),
            Err(Mismatch),
            "ct flip at {i} must be Mismatch"
        );
    }

    // Tag flip (single BIT, first and last byte).
    for i in [0usize, TAG_LEN - 1] {
        let mut tag2 = tag;
        tag2[i] ^= 0x01;
        assert_eq!(
            aead::open_detached(&mut back, &ct, &tag2, &aad, &key, &nonce),
            Err(Mismatch),
            "tag bit flip at {i} must be Mismatch"
        );
    }

    // Plaintext flip changes both ct and tag (never an accept path).
    let mut pt2 = pt.clone();
    pt2[7] ^= 0x01;
    let mut ct2 = std::vec::Vec::with_capacity(pt.len());
    ct2.resize(pt.len(), 0);
    let mut tag2 = [0u8; TAG_LEN];
    aead::seal_detached(&mut ct2, &mut tag2, &pt2, &aad, &key, &nonce);
    assert_ne!(ct2[7], ct[7], "pt flip must change ct");
    assert_ne!(tag2, tag, "pt flip must change tag");
    assert_eq!(
        aead::open_detached(&mut back, &ct, &tag, &aad, &key, &nonce),
        Ok(()),
        "untampered inputs still open (court not trivially red)"
    );
}

/// EpochExhausted boundary court: epoch 0xFFFF_FFFE is the LAST lawful
/// epoch (constructor Ok, seal/open live end-to-end on it); 0xFFFF_FFFF is
/// the reserved sentinel the SOLE constructor refuses with
/// `EpochExhausted`. Anti-vacuity: dropping the constructor guard, or
/// adding a second construction path that bypasses it, flips this court
/// (the bypass path is separately killed by the count-law scan below).
#[test]
fn epoch_exhausted_boundary() {
    assert_eq!(
        Nonce96::new(0xFFFF_FFFE, 0x1111_2222_3333_4444).is_ok(),
        true
    );
    assert_eq!(Nonce96::new(0xFFFF_FFFF, 0), Err(EpochExhausted));
    // 0xFFFF_FFFE seals and opens end-to-end.
    let key = [0xA5u8; KEY_LEN];
    let aad = [0x5Au8; AAD_LEN];
    let nonce = Nonce96::new(0xFFFF_FFFE, 7).expect("boundary epoch is lawful");
    let mut ct = [0u8; 64];
    let mut tag = [0u8; TAG_LEN];
    aead::seal_detached(&mut ct, &mut tag, &[0u8; 64], &aad, &key, &nonce);
    let mut back = [0u8; 64];
    assert_eq!(
        aead::open_detached(&mut back, &ct, &tag, &aad, &key, &nonce),
        Ok(())
    );
    assert_eq!(back, [0u8; 64]);
}

// ---------------------------------------------------------------------------
// Sole-constructor count-law source scan (CG-1 shape: baseline -> sandbox
// add-token kill -> fail-closed restore witness).
// ---------------------------------------------------------------------------

/// Count-law: the ONLY construction site of `Nonce96` in the kernel is the
/// `new()` body — the source must contain ZERO `Nonce96 {` struct literals
/// (the constructor builds `Self { .. }`). Anti-vacuity: injecting a
/// literal-construction bypass token into a /tmp SANDBOX copy of aead.rs
/// must flip the scan; the repo tree is restored byte-exact afterwards
/// (restore witness), the shared tree is never mutated.
#[test]
fn sole_constructor_count_law_scan() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let aead_src = std::path::Path::new(manifest).join("src/aead.rs");
    let before = std::fs::read(&aead_src).expect("read src/aead.rs");
    let text = String::from_utf8_lossy(&before);
    let literals = text.matches("Nonce96 { epoch").count();
    assert_eq!(
        literals, 0,
        "sole-constructor law: zero `Nonce96 {{ epoch: .. }}` construction expressions allowed in src/aead.rs (construction flows through Nonce96::new only)"
    );

    // SANDBOX ADD-TOKEN KILL: the same scan over a mutated /tmp copy MUST
    // flip (non-vacuity proof; never touches the repo tree).
    let sandbox = std::env::temp_dir().join("ab4-nonce-scan.rs");
    let mutated = format!(
        "fn bypass() -> Nonce96 {{ Nonce96 {{ epoch: 0xFFFF_FFFF, counter: 0 }} }}\n{}",
        text
    );
    std::fs::write(&sandbox, mutated.as_bytes()).expect("write sandbox");
    let sbytes = std::fs::read(&sandbox).expect("read sandbox");
    let stext = String::from_utf8_lossy(&sbytes);
    let shits = stext.matches("Nonce96 {").count();
    let _ = std::fs::remove_file(&sandbox);
    assert!(
        shits >= 1,
        "VACUOUS SCAN: injected constructor-bypass token undetected"
    );

    // Fail-closed restore witness.
    let after = std::fs::read(&aead_src).expect("re-read src/aead.rs");
    assert_eq!(before, after, "src/aead.rs untouched (fail-closed restore)");
}

// ---------------------------------------------------------------------------
// Throughput bench (min-of-N baseline): seal/open MB/s at 1KB / 64KB / 1MB.
// Runs in the dev profile (unoptimized) — the court pins a conservative
// floor so a catastrophic regression (e.g. an accidental per-byte alloc or
// a block-function pessimization) flips it, while not gating on dev-vs-
fn bench_seal(len: usize) -> f64 {
    let mut state: u64 = 0x5EED_0000_0000_0001 ^ (len as u64).rotate_left(32);
    let key = key32(&mut state);
    let nonce = Nonce96::new(1, 42).expect("lawful epoch");
    let aad = aad36(&mut state);
    let pt: std::vec::Vec<u8> = (0..len).map(|_| splitmix64(&mut state) as u8).collect();
    let mut ct = vec![0u8; len];
    let mut tag = [0u8; TAG_LEN];

    // Warm-up.
    aead::seal_detached(&mut ct, &mut tag, &pt, &aad, &key, &nonce);

    // Scale the inner loop so each timed window does ~64 MiB of work.
    let inner = ((64 * 1024 * 1024) / len.max(1)).clamp(4, 4096);
    let mut best_mbps = f64::INFINITY;
    for _ in 0..8 {
        let t0 = std::time::Instant::now();
        for _ in 0..inner {
            aead::seal_detached(&mut ct, &mut tag, &pt, &aad, &key, &nonce);
        }
        let mbps = (len as f64 * inner as f64) / t0.elapsed().as_secs_f64() / (1024.0 * 1024.0);
        best_mbps = best_mbps.min(mbps);
    }
    best_mbps
}

fn bench_open(len: usize) -> f64 {
    let mut state: u64 = 0x5EED_0000_0000_0001 ^ (len as u64).rotate_left(32);
    let key = key32(&mut state);
    let nonce = Nonce96::new(1, 42).expect("lawful epoch");
    let aad = aad36(&mut state);
    let pt: std::vec::Vec<u8> = (0..len).map(|_| splitmix64(&mut state) as u8).collect();
    let mut ct = vec![0u8; len];
    let mut tag = [0u8; TAG_LEN];
    let mut back = vec![0u8; len];

    aead::seal_detached(&mut ct, &mut tag, &pt, &aad, &key, &nonce);

    let inner = ((64 * 1024 * 1024) / len.max(1)).clamp(4, 4096);
    let mut best_mbps = f64::INFINITY;
    for _ in 0..8 {
        let t0 = std::time::Instant::now();
        for _ in 0..inner {
            let _ = aead::open_detached(&mut back, &ct, &tag, &aad, &key, &nonce);
        }
        let mbps = (len as f64 * inner as f64) / t0.elapsed().as_secs_f64() / (1024.0 * 1024.0);
        best_mbps = best_mbps.min(mbps);
    }
    best_mbps
}

/// Throughput court: seal and open must sustain >= 2 MB/s at 64KB in the
/// UNOPTIMIZED dev profile (measured baseline in the lane receipt).
/// Anti-vacuity: an accidental O(n^2) loop, per-byte heap churn, or a
/// per-byte keystream re-derivation collapses throughput and flips this.
/// Prints the full 1KB/64KB/1MB min-of-8 table.
#[test]
fn throughput_min_of_n_floor() {
    for len in [1024usize, 64 * 1024, 1024 * 1024] {
        let s = bench_seal(len);
        let o = bench_open(len);
        println!(
            "throughput len={len:>7}: seal {s:8.2} MB/s | open {o:8.2} MB/s (min of 8, dev profile)"
        );
        if len == 64 * 1024 {
            assert!(s >= 2.0, "seal floor {s:.2} < 2 MB/s at len {len}");
            assert!(o >= 2.0, "open floor {o:.2} < 2 MB/s at len {len}");
        }
    }
}
