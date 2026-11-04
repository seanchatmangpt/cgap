//! AB-5 bench: keygen / encaps / decaps microseconds-per-op, min-of-100,
//! release-profile semantics (run under `cargo test --release`). These are
//! the OQ-4 rekey-cost baseline numbers.
use cgap_crypto::kem;
use std::time::Instant;

fn bench<F: FnMut()>(name: &str, iters: usize, mut f: F) -> f64 {
    let mut best = f64::MAX;
    for _ in 0..100 {
        let t0 = Instant::now();
        for _ in 0..iters {
            f();
        }
        let el = t0.elapsed().as_secs_f64() * 1e6;
        let per = el / iters as f64;
        if per < best {
            best = per;
        }
    }
    println!("BENCH {} min-of-100: {:.2} us/op", name, best);
    best
}

#[test]
fn bench_keygen_encaps_decaps() {
    let d = [7u8; 32];
    let z = [9u8; 32];
    let mut ek = [0u8; 1184];
    let mut dk = [0u8; 2400];
    kem::keygen(&d, &z, &mut ek, &mut dk);
    let m = [3u8; 32];
    let mut ct = [0u8; 1088];
    let mut ss = [0u8; 32];
    kem::encaps(&ek, &m, &mut ct, &mut ss);

    let k = bench("keygen", 10, || {
        let mut ek2 = [0u8; 1184];
        let mut dk2 = [0u8; 2400];
        kem::keygen(&d, &z, &mut ek2, &mut dk2);
    });
    let e = bench("encaps", 10, || {
        let mut c2 = [0u8; 1088];
        let mut s2 = [0u8; 32];
        kem::encaps(&ek, &m, &mut c2, &mut s2);
    });
    let dec = bench("decaps", 10, || {
        let _ = kem::decaps(&dk, &ct);
    });
    println!(
        "BENCH_SUMMARY keygen={:.2}us encaps={:.2}us decaps={:.2}us",
        k, e, dec
    );
}
