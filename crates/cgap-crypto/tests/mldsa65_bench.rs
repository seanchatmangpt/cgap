//! ML-DSA-65 phase-5 bench: us/op min-of-100 on keygen / sign / verify
//! (ZP-18 release-profile semantics; loud-SKIP on dev profile).
use cgap_aoq::profile_guard;
use cgap_crypto::mldsa as m;
use std::time::Instant;

fn splitmix_bytes(state: &mut u64, out: &mut [u8]) {
    for b in out.iter_mut() {
        *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_4A0D_A71B_D881);
        *b = (z ^ (z >> 31)) as u8;
    }
}

#[test]
fn bench_keygen_sign_verify() {
    if profile_guard::assert_release().is_err() {
        eprintln!("LOUD-SKIP bench_keygen_sign_verify: dev profile");
        return;
    }
    let mut sm = 0xB3A_0000u64;
    let mut xi = [0u8; 32];
    splitmix_bytes(&mut sm, &mut xi);
    let mut ek = [0u8; m::PK_BYTES];
    let mut dk = [0u8; m::SK_BYTES];
    m::keygen(&xi, &mut ek, &mut dk);
    let tr: [u8; 64] = dk[64..128].try_into().unwrap();
    let mu = m::compute_mu(&tr, b"bench");
    let rnd = [0x5Au8; 32];
    let sig = m::sign(&dk, &mu, &rnd).expect("sign");
    let mut mk = || {
        let mut e = [0u8; m::PK_BYTES];
        let mut d = [0u8; m::SK_BYTES];
        m::keygen(&xi, &mut e, &mut d);
        profile_guard::fence((&e, &d));
    };
    let mut ms = || {
        let s = m::sign(&dk, &mu, &rnd).expect("sign");
        profile_guard::fence(s);
    };
    let mut mv = || {
        let ok = m::verify(&ek, &mu, &sig);
        profile_guard::fence(ok);
    };
    let mut best_k = f64::MAX;
    let mut best_s = f64::MAX;
    let mut best_v = f64::MAX;
    for _ in 0..100 {
        for (name, best, iters, f) in [
            ("keygen", &mut best_k, 10usize, &mut mk as &mut dyn FnMut()),
            ("sign", &mut best_s, 10, &mut ms as &mut dyn FnMut()),
            ("verify", &mut best_v, 10, &mut mv as &mut dyn FnMut()),
        ] {
            let t0 = Instant::now();
            for _ in 0..iters {
                f();
            }
            let per = t0.elapsed().as_secs_f64() * 1e6 / iters as f64;
            if per < *best {
                *best = per;
            }
            let _ = name;
        }
    }
    println!(
        "BENCH_SUMMARY mldsa65 keygen={:.2}us sign={:.2}us verify={:.2}us (min-of-100, release)",
        best_k, best_s, best_v
    );
}
