// Bench court (AB-6): encode/decode ns-per-header, TWO methodologies
// (G2 ruling, recorded side by side, never conflated):
//   AMORTIZED: min-of-7x100k loop; the per-op cost inside a tight loop
//     (scheduler noise reduced by min-of-N). The G2 100ns ceiling is
//     bound to THIS method only.
//   PER-CALL: median per-call wall over n=1000 individually-timed calls
//     (AF-10's method; includes call/setup overhead — larger number,
//     not comparable to the amortized figure).

use cgap_aoq::codec::{decode, encode};
use cgap_aoq::header::{AoqHeader, Epoch};

fn timed<F: FnMut()>(mut f: F) -> f64 {
    let start = std::time::Instant::now();
    f();
    start.elapsed().as_secs_f64() * 1e9
}

#[test]
fn bench_encode_decode_ns_per_header() {
    if let Err(v) = cgap_aoq::profile_guard::assert_release() {
        eprintln!("LOUD-SKIP bench_encode_decode_ns_per_header: {v}");
        return;
    }
    const ROUNDS: usize = 7;
    const ITERS: usize = 100_000;
    let h = AoqHeader::new(Epoch::Genesis as u16, 0xDEAD_BEEF, 42, 7);
    let mut wire = [0u8; 64];
    encode(&h, &mut wire).expect("encode");
    let mut sink = 0u64;

    let mut min_enc = f64::INFINITY;
    let mut min_dec = f64::INFINITY;
    for _ in 0..ROUNDS {
        let enc = timed(|| {
            for _ in 0..ITERS {
                // Loop-carried dependency: this iteration's sink feeds the
                // next iteration's encode INPUT (key_id), and each sink
                // update routes through the house anti-DCE fence — LLVM
                // cannot constant-fold across iterations (BR-5 fix).
                let mut hi = h;
                hi.key_id = h.key_id ^ sink;
                let mut w = [0u8; 64];
                encode(&hi, &mut w).ok();
                // w[60] is the low byte of header_digest — the digest covers
                // bytes [0,0x3C) including the sink-chained key_id, so the
                // FULL encode work (digest loop included) is required to
                // produce it. Fencing a digest-independent byte (w[0] magic,
                // w[8] key_id) lets DSE kill the digest as a dead store.
                sink = sink.wrapping_add(cgap_aoq::profile_guard::fence(w[60] as u64));
            }
        }) / ITERS as f64;
        let dec = timed(|| {
            for _ in 0..ITERS {
                // decode a fresh copy so the cache state matches encode's;
                // byte 61 is chained from the previous iteration's sink so
                // the decode INPUT is loop-carried (not hoistable).
                let mut w = wire;
                w[61] = (sink & 0xFF) as u8;
                let r = decode(&w).ok();
                // House fence on the folded result keeps sink opaque to the
                // optimizer — a constant success-bit cannot collapse the
                // chain into a closed form.
                sink = sink.wrapping_add(cgap_aoq::profile_guard::fence(
                    r.map(|hdr| (hdr.header_digest as u64) & 1).unwrap_or(0),
                ));
            }
        }) / ITERS as f64;
        min_enc = min_enc.min(enc);
        min_dec = min_dec.min(dec);
    }
    assert!(min_enc > 0.0 && min_dec > 0.0);
    println!(
        "bench[amortized]: encode {min_enc:.1} ns/header, decode {min_dec:.1} ns/header (min of {ROUNDS}x{ITERS})"
    );

    // PER-CALL methodology (AF-10's method): median per-call wall over
    // n=1000 individually-timed calls, black_box'd inputs/outputs so the
    // call is not folded away.
    const CALLS: usize = 1_000;
    let mut enc_samples: Vec<f64> = Vec::with_capacity(CALLS);
    let mut dec_samples: Vec<f64> = Vec::with_capacity(CALLS);
    for k in 0..CALLS {
        let mut w = [0u8; 64];
        let start = std::time::Instant::now();
        encode(std::hint::black_box(&h), std::hint::black_box(&mut w)).ok();
        enc_samples.push(start.elapsed().as_secs_f64() * 1e9);
        std::hint::black_box(&w);

        let mut r = wire;
        r[61] = (k as u64) as u8;
        let start = std::time::Instant::now();
        let res = decode(std::hint::black_box(&r));
        dec_samples.push(start.elapsed().as_secs_f64() * 1e9);
        std::hint::black_box(&res);
        std::hint::black_box(&r);
    }
    enc_samples.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    dec_samples.sort_by(|a, b| a.partial_cmp(b).expect("no NaN"));
    let med_enc = enc_samples[CALLS / 2];
    let med_dec = dec_samples[CALLS / 2];
    assert!(med_enc > 0.0 && med_dec > 0.0);
    println!(
        "bench[per-call]: encode {med_enc:.1} ns/call, decode {med_dec:.1} ns/call (median of {CALLS} individually-timed calls, AF-10 method)"
    );
    let _ = sink;
}
