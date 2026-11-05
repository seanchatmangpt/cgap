//! BM-9 + BR-3 — cgap-crypto primitives bench (divan membrane).
//!
//! Surfaces (tickets BM-9/BR-3, docs/jira/v26.11.4):
//! (1) BLAKE3 `hash` (Sum32 wire form) chunk-size curve: 64 B / 256 B / 1 KB /
//!     4 KB / 16 KB / 64 KB / 256 KB / 1 MB (the GB/s curve + multi-chunk knee);
//! (2) ChaCha20-Poly1305 detached seal+open size curve: 64 B / 1 KB / 16 KB /
//!     64 KB (per-byte trend, tag-overhead amortization), var-AAD 36 B;
//! (3) ML-KEM-768 keygen / encaps / decaps (decaps-vs-encaps asymmetry ratio);
//! (4) ML-DSA-65 P1 kernel ops (`ntt`, `intt_tomont`, `pointwise_montgomery`)
//!     — the full keygen/sign/verify top-level is NOT yet landed in
//!     `src/mldsa.rs` (P1 kernel only, 434 lines, poly/NTT surface); benching
//!     the landed kernel ops and recording the constant pins is the honest
//!     measure — a bench of an unlanded sign/verify would measure nothing.
//!
//! ZP-18 composition law: this file composes `profile_guard::assert_release`
//! at harness entry (loud refusal, exit 3, never a dev-profile number) and
//! `profile_guard::fence` per benched op — TIMING+guarded, census-clean, no
//! allowlist entry needed.
//!
//! Chicago: real kernels, real bytes, outputs fenced every iteration
//! (anti-DCE fence is the anti-vacuity carrier; a drifted kernel flips the
//! fence input).

use cgap_aoq::profile_guard::{assert_release, fence};
use cgap_crypto::{aead, kem, mldsa, poly1305};
use divan::{black_box, Bencher};

// ---------------------------------------------------------------------------
// Sanity pins (in-bench court, compile-time): a bench measuring a
// wrong-sized primitive is worse than none.
// ---------------------------------------------------------------------------
const _: () = assert!(kem::SS_LEN == 32, "ML-KEM-768 shared secret is 32 B");
const _: () = assert!(kem::CT_LEN == 1088, "ML-KEM-768 ciphertext is 1088 B");
const _: () = assert!(
    kem::EK_LEN == 1184,
    "ML-KEM-768 encapsulation key is 1184 B"
);
const _: () = assert!(
    mldsa::SIG_BYTES == 3309,
    "ML-DSA-65 signature is 3309 B (chatman constant)"
);
const _: () = assert!(mldsa::PK_BYTES == 1952, "ML-DSA-65 public key is 1952 B");
const _: () = assert!(mldsa::SK_BYTES == 4032, "ML-DSA-65 secret key is 4032 B");
const _: () = assert!(aead::KEY_LEN == 32 && aead::TAG_LEN == 16 && aead::NONCE_LEN == 12);

/// Canonical SplitMix64 master seed (repo-wide corpus law, verbatim).
const CORPUS_SEED: u64 = 0xBF58_476D_1CE4_E5B9;

/// SplitMix64 seeded byte corpus (canonical constants verbatim; form matches
/// crates/bench/benches/seal.rs `fill_splitmix`).
fn fill_splitmix(buf: &mut [u8], seed: u64) {
    let mut z = seed;
    let mut i = 0;
    while i < buf.len() {
        z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut w = z;
        w = (w ^ (w >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        w = (w ^ (w >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        w ^= w >> 31;
        let src = w.to_le_bytes();
        let n = (buf.len() - i).min(8);
        buf[i..i + n].copy_from_slice(&src[..n]);
        i += n;
    }
}

fn main() {
    // ZP-18: timing numbers are only lawful on a release profile — loud
    // refusal (never a fake number, never a silent skip) on a dev build.
    if let Err(v) = assert_release() {
        eprintln!("BM-9 LOUD-REFUSAL bench not run on a lawful profile: {v}");
        std::process::exit(3);
    }
    divan::Divan::default().run_benches();
}

// ---------------------------------------------------------------------------
// (1) BLAKE3 — Sum32 wire form (cgap-blake3 `hash`), BR-3 chunk-size curve:
// 64 B / 256 B / 1 KB / 4 KB / 16 KB / 64 KB / 256 KB / 1 MB. BLAKE3 chunks
// are 1024 B; the multi-chunk knee should land crossing 1 KB -> 4 KB.
// ---------------------------------------------------------------------------
#[divan::bench(args = [64, 256, 1024, 4096, 16 * 1024, 64 * 1024, 256 * 1024, 1024 * 1024])]
fn blake3_sum32(b: Bencher, len: usize) {
    let mut buf = vec![0u8; len];
    fill_splitmix(&mut buf, CORPUS_SEED);
    let expected: [u8; 32] = cgap_blake3::hash(&buf);
    b.bench(|| {
        fence(black_box(cgap_blake3::hash(black_box(&buf))));
    });
    let got = cgap_blake3::hash(&buf);
    assert_eq!(got, expected, "blake3 corpus digest drifted mid-bench");
}

// ---------------------------------------------------------------------------
// (2) ChaCha20-Poly1305 — detached seal+open round trip, BR-3 size curve
// 64 B / 1 KB / 16 KB / 64 KB, 36 B AAD, per-byte trend + tag amortization.
// ---------------------------------------------------------------------------
#[divan::bench(args = [64, 1024, 16 * 1024, 64 * 1024])]
fn aead_seal_open(b: Bencher, len: usize) {
    let mut pt = vec![0u8; len];
    fill_splitmix(&mut pt, CORPUS_SEED ^ 0xA1);
    let mut aad = [0u8; aead::AAD_LEN];
    fill_splitmix(&mut aad, CORPUS_SEED ^ 0xA2);
    let key = {
        let mut k = [0u8; aead::KEY_LEN];
        fill_splitmix(&mut k, CORPUS_SEED ^ 0xA3);
        k
    };
    let nonce = aead::Nonce96::new(0x0000_0007, 0x4746_4544_4342_4140).expect("lawful epoch");

    b.bench(|| {
        let mut ct = vec![0u8; len];
        let mut tag = [0u8; aead::TAG_LEN];
        aead::seal_detached(&mut ct, &mut tag, black_box(&pt), &aad, &key, &nonce);
        fence((black_box(&ct), black_box(&tag)));
        let mut back = vec![0u8; len];
        aead::open_detached(&mut back, &ct, &tag, &aad, &key, &nonce).expect("lawful open");
        fence(black_box(&back));
        assert_eq!(back, pt, "seal/open round trip diverged");
    });
}

// ---------------------------------------------------------------------------
// (4) Poly1305 — pure kernel (`poly1305::one_shot`) vs aead-integrated
// (`aead::mac_tag_var_aad`, var-AAD wire form: AAD || pad || ct || pad ||
// len(AAD) || len(ct)). Same 1 KB message body, same 32 B one-shot key input
// surface where the APIs allow. Bench only public surfaces: `one_shot`,
// `mac_tag_var_aad`.
// ---------------------------------------------------------------------------
#[divan::bench(args = [64, 1024, 16 * 1024, 64 * 1024])]
fn poly1305_one_shot(b: Bencher, len: usize) {
    let mut data = vec![0u8; len];
    fill_splitmix(&mut data, CORPUS_SEED ^ 0xB1);
    let mut key = [0u8; poly1305::KEY_LEN];
    fill_splitmix(&mut key, CORPUS_SEED ^ 0xB2);
    let expected: [u8; poly1305::TAG_LEN] = poly1305::one_shot(&key, &data);
    b.bench(|| {
        fence(black_box(poly1305::one_shot(
            black_box(&key),
            black_box(&data),
        )));
    });
    let got = poly1305::one_shot(&key, &data);
    assert_eq!(got, expected, "poly1305 corpus tag drifted mid-bench");
}

#[divan::bench(args = [64, 1024, 16 * 1024, 64 * 1024])]
fn aead_mac_var_aad(b: Bencher, len: usize) {
    let mut ct = vec![0u8; len];
    fill_splitmix(&mut ct, CORPUS_SEED ^ 0xB1);
    let mut aad = [0u8; aead::AAD_LEN];
    fill_splitmix(&mut aad, CORPUS_SEED ^ 0xA2);
    let key = {
        let mut k = [0u8; aead::KEY_LEN];
        fill_splitmix(&mut k, CORPUS_SEED ^ 0xA3);
        k
    };
    let nonce = aead::Nonce96::new(0x0000_0007, 0x4746_4544_4342_4140).expect("lawful epoch");
    let expected: [u8; aead::TAG_LEN] = aead::mac_tag_var_aad(&aad, &ct, &key, &nonce);
    b.bench(|| {
        fence(black_box(aead::mac_tag_var_aad(
            black_box(&ct),
            &aad,
            &key,
            &nonce,
        )));
    });
    let got = aead::mac_tag_var_aad(&aad, &ct, &key, &nonce);
    assert_eq!(got, expected, "var-AAD mac corpus tag drifted mid-bench");
}

// ---------------------------------------------------------------------------
// (3) ML-KEM-768 — keygen / encaps / decaps.
// ---------------------------------------------------------------------------
#[divan::bench]
fn mlkem768_keygen(b: Bencher) {
    let d = [7u8; 32];
    let z = [9u8; 32];
    b.bench(|| {
        let mut ek = [0u8; kem::EK_LEN];
        let mut dk = [0u8; kem::DK_LEN];
        kem::keygen(&d, &z, &mut ek, &mut dk);
        fence((black_box(&ek), black_box(&dk)));
    });
}

#[divan::bench]
fn mlkem768_encaps(b: Bencher) {
    let d = [7u8; 32];
    let z = [9u8; 32];
    let mut ek = [0u8; kem::EK_LEN];
    let mut dk = [0u8; kem::DK_LEN];
    kem::keygen(&d, &z, &mut ek, &mut dk);
    let m = [3u8; 32];
    b.bench(|| {
        let mut ct = [0u8; kem::CT_LEN];
        let mut ss = [0u8; kem::SS_LEN];
        kem::encaps(&ek, &m, &mut ct, &mut ss);
        fence((black_box(&ct), black_box(&ss)));
    });
}

#[divan::bench]
fn mlkem768_decaps(b: Bencher) {
    let d = [7u8; 32];
    let z = [9u8; 32];
    let mut ek = [0u8; kem::EK_LEN];
    let mut dk = [0u8; kem::DK_LEN];
    kem::keygen(&d, &z, &mut ek, &mut dk);
    let m = [3u8; 32];
    let mut ct = [0u8; kem::CT_LEN];
    let mut ss = [0u8; kem::SS_LEN];
    kem::encaps(&ek, &m, &mut ct, &mut ss);
    b.bench(|| {
        fence(black_box(kem::decaps(&dk, &ct)));
    });
}

// ---------------------------------------------------------------------------
// (4) ML-DSA-65 — P1 kernel ops (NTT surface). Keygen/sign/verify top-level
// is not yet landed in src/mldsa.rs; these are the landed kernel ops.
// ---------------------------------------------------------------------------
#[divan::bench]
fn mldsa65_ntt(b: Bencher) {
    let mut p = [0i32; mldsa::N];
    for (i, c) in p.iter_mut().enumerate() {
        *c = ((CORPUS_SEED ^ i as u64) % (2 * mldsa::Q as u64)) as i32 - mldsa::Q;
    }
    b.bench(|| {
        let mut g = p;
        mldsa::ntt(&mut g);
        fence(black_box(&g));
    });
}

#[divan::bench]
fn mldsa65_intt_tomont(b: Bencher) {
    let mut p = [0i32; mldsa::N];
    for (i, c) in p.iter_mut().enumerate() {
        *c = ((CORPUS_SEED ^ (i as u64 + 1)) % (2 * mldsa::Q as u64)) as i32 - mldsa::Q;
    }
    mldsa::ntt(&mut p);
    b.bench(|| {
        let mut g = p;
        mldsa::intt_tomont(&mut g);
        fence(black_box(&g));
    });
}

#[divan::bench]
fn mldsa65_pointwise_montgomery(b: Bencher) {
    let mut a = [0i32; mldsa::N];
    let mut b2 = [0i32; mldsa::N];
    for (i, c) in a.iter_mut().enumerate() {
        *c = ((CORPUS_SEED ^ i as u64) % mldsa::Q as u64) as i32;
    }
    for (i, c) in b2.iter_mut().enumerate() {
        *c = ((CORPUS_SEED ^ (i as u64 + 7)) % mldsa::Q as u64) as i32;
    }
    mldsa::ntt(&mut a);
    mldsa::ntt(&mut b2);
    b.bench(|| {
        let mut r = [0i32; mldsa::N];
        mldsa::pointwise_montgomery(&mut r, &a, &b2);
        fence(black_box(&r));
    });
}
