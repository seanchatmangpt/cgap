//! mldsa_ntt_courts - ML-DSA-65 kernel courts.
//!
//! External ground truth (fetched this session, see src/mldsa.rs PROVENANCE):
//! - pq-crystals/dilithium master ref/ntt.c zetas literal (255 entries,
//!   [0] = 0 padding not carried here).
//! - GMUCERG/Dilithium dilithium-256/consts.cpp zetas_barrett literal
//!   (round-2-lineage plain-domain twiddles, 255 entries + 0 padding).
//! - Scalar (schoolbook / direct modular arithmetic) references computed
//!   in-test; no mocks.
use cgap_crypto::mldsa;

// pq-crystals/dilithium master ref/ntt.c zetas[1..=255].
#[rustfmt::skip]
const PQCRYSTALS_ZETAS_255: [i32; 255] = [
       25847, -2608894,  -518909,   237124,  -777960,  -876248,   466468,  1826347,
     2353451,  -359251, -2091905,  3119733, -2884855,  3111497,  2680103,  2725464,
     1024112, -1079900,  3585928,  -549488, -1119584,  2619752, -2108549, -2118186,
    -3859737, -1399561, -3277672,  1757237,   -19422,  4010497,   280005,  2706023,
       95776,  3077325,  3530437, -1661693, -3592148, -2537516,  3915439, -3861115,
    -3043716,  3574422, -2867647,  3539968,  -300467,  2348700,  -539299, -1699267,
    -1643818,  3505694, -3821735,  3507263, -2140649, -1600420,  3699596,   811944,
      531354,   954230,  3881043,  3900724, -2556880,  2071892, -2797779, -3930395,
    -1528703, -3677745, -3041255, -1452451,  3475950,  2176455, -1585221, -1257611,
     1939314, -4083598, -1000202, -3190144, -3157330, -3632928,   126922,  3412210,
     -983419,  2147896,  2715295, -2967645, -3693493,  -411027, -2477047,  -671102,
    -1228525,   -22981, -1308169,  -381987,  1349076,  1852771, -1430430, -3343383,
      264944,   508951,  3097992,    44288, -1100098,   904516,  3958618, -3724342,
       -8578,  1653064, -3249728,  2389356,  -210977,   759969, -1316856,   189548,
    -3553272,  3159746, -1851402, -2409325,  -177440,  1315589,  1341330,  1285669,
    -1584928,  -812732, -1439742, -3019102, -3881060, -3628969,  3839961,  2091667,
     3407706,  2316500,  3817976, -3342478,  2244091, -2446433, -3562462,   266997,
     2434439, -1235728,  3513181, -3520352, -3759364, -1197226, -3193378,   900702,
     1859098,   909542,   819034,   495491, -1613174,   -43260,  -522500,  -655327,
    -3122442,  2031748,  3207046, -3556995,  -525098,  -768622, -3595838,   342297,
      286988, -2437823,  4108315,  3437287, -3342277,  1735879,   203044,  2842341,
     2691481, -2590150,  1265009,  4055324,  1247620,  2486353,  1595974, -3767016,
     1250494,  2635921, -3548272, -2994039,  1869119,  1903435, -1050970, -1333058,
     1237275, -3318210, -1430225,  -451100,  1312455,  3306115, -1962642, -1279661,
     1917081, -2546312, -1374803,  1500165,   777191,  2235880,  3406031,  -542412,
    -2831860, -1671176, -1846953, -2584293, -3724270,   594136, -3776993, -2013608,
     2432395,  2454455,  -164721,  1957272,  3369112,   185531, -1207385, -3183426,
      162844,  1616392,  3014001,   810149,  1652634, -3694233, -1799107, -3038916,
     3523897,  3866901,   269760,  2213111,  -975884,  1717735,   472078,  -426683,
     1723600, -1803090,  1910376, -1667432, -1104333,  -260646, -3833893, -2939036,
    -2235985,  -420899, -2286327,   183443,  -976891,  1612842, -3545687,  -554416,
     3919660,   -48306, -1362209,  3937738,  1400424,  -846154,  1976782,
];

// GMUCERG/Dilithium dilithium-256/consts.cpp zetas_barrett[1..=255].
#[rustfmt::skip]
const GMUCERG_ZETAS_BARRETT_255: [i32; 255] = [
    -3572223,  3765607,  3761513, -3201494, -2883726, -3145678, -3201430,  -601683,
     3542485,  2682288,  2129892,  3764867, -1005239,   557458, -1221177, -3370349,
    -4063053,  2663378, -1674615, -3524442,  -434125,   676590, -1335936, -3227876,
     1714295,  2453983,  1460718,  -642628, -3585098,  2815639,  2283733,  3602218,
     3182878,  2740543, -3586446, -3110818,  2101410,  3704823,  1159875,   394148,
      928749,  1095468, -3506380,  2071829, -4018989,  3241972,  2156050,  3415069,
     1759347,  -817536, -3574466,  3756790, -1935799, -1716988, -3950053, -2897314,
     3192354,   556856,  3870317,  2917338,  1853806,  3345963,  1858416,  3073009,
     1277625, -2635473,  3852015,  4183372, -3222807, -3121440,  -274060,  2508980,
     2028118,  1937570, -3815725,  2811291, -2983781, -1109516,  4158088,  1528066,
      482649,  1148858, -2962264,  -565603,   169688,  2462444, -3334383, -4166425,
    -3488383,  1987814, -3197248,  1736313,   235407, -3250154,  3258457, -2579253,
     1787943, -2391089, -2254727,  3482206, -4182915, -1300016, -2362063, -1317678,
     2461387,  3035980,   621164,  3901472, -1226661,  2925816,  3374250,  1356448,
    -2775755,  2683270, -2778788, -3467665,  2312838,  -653275,  -459163,   348812,
     -327848,  1011223, -2354215, -3818627, -1922253, -2236726,  1744507,     1753,
    -1935420, -2659525, -1455890,  2660408, -1780227,   -59148,  2772600,  1182243,
       87208,   636927, -3965306, -3956745, -2296397, -3284915, -3716946,   -27812,
      822541,  1009365, -2454145, -1979497,  1596822, -3956944, -3759465, -1685153,
    -3410568,  2678278, -3768948, -3551006,   635956,  -250446, -2455377, -4146264,
    -1772588,  2192938, -1727088,  2387513, -3611750,  -268456, -3180456,  3747250,
     2296099,  1239911, -3838479,  3195676,  2642980,  1254190,   -12417,  2998219,
      141835,   -89301,  2513018, -1354892,   613238, -1310261, -2218467,  -458740,
    -1921994,  4040196, -3472069,  2039144, -1879878,  -818761, -2178965, -1623354,
     2105286, -2374402, -2033807,   586241, -1179613,   527981, -2743411, -1476985,
     1994046,  2491325, -1393159,   507927, -1187885,  -724804, -1834526, -3033742,
     -338420,  2647994,  3009748, -2612853,  4148469,   749577, -4022750,  3980599,
     2569011, -1615530,  1723229,  1665318,  2028038,  1163598, -3369273,  3994671,
      -11879, -1370517,  3020393,  3363542,   214880,   545376,  -770441,  3105558,
    -1103344,   508145,  -553718,   860144,  3430436,   140244, -1514152, -2185084,
     3123762,  2358373, -2193087, -3014420, -1716814,  2926054,  -392707,  -303005,
     3531229, -3974485, -3773731,  1900052,  -781875,  1054478,  -731434,
];

/// Deterministic LCG (no external RNG dep; fixed seed for replay).
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
    fn poly(&mut self) -> [i32; mldsa::N] {
        let mut p = [0i32; mldsa::N];
        for c in p.iter_mut() {
            // centered in (-q, q)
            *c = (self.next() % (2 * mldsa::Q as u64)) as i32 - mldsa::Q;
        }
        p
    }
}

// ---------------------------------------------------------------------------
// External literal pins
// ---------------------------------------------------------------------------

#[test]
fn zetas_entry_check_vs_pqcrystals() {
    assert_eq!(mldsa::ZETAS[0], 0, "ZETAS[0] is padding");
    assert_eq!(&mldsa::ZETAS[1..256], &PQCRYSTALS_ZETAS_255,
        "ZETAS[1..] must equal the fetched pq-crystals ref/ntt.c literal");
}

#[test]
fn zetas_barrett_entry_check_vs_gmucerg() {
    assert_eq!(mldsa::ZETAS_BARRETT[0], 0, "ZETAS_BARRETT[0] is padding");
    assert_eq!(&mldsa::ZETAS_BARRETT[1..256], &GMUCERG_ZETAS_BARRETT_255,
        "ZETAS_BARRETT[1..] must equal the fetched GMUCERG zetas_barrett literal");
}

/// blake3 digest pins over the full 256-entry tables (LE i32 bytes).
/// Mutation guard: any value edit flips this court AND the entry checks.
#[test]
fn zetas_digest_pins() {
    let mut bytes = Vec::new();
    for v in mldsa::ZETAS.iter() {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    let d = cgap_blake3::hash(&bytes);
    let mut hexs = String::new();
    for b in d.iter() {
        hexs.push_str(&format!("{:02x}", b));
    }
    assert_eq!(hexs, "2772b1df66a71a02db41a967550dcde1bbe3d29413716cac69d61e1cf0b7e71b");
    let mut bytes = Vec::new();
    for v in mldsa::ZETAS_BARRETT.iter() {
        bytes.extend_from_slice(&v.to_le_bytes());
    }
    let d = cgap_blake3::hash(&bytes);
    let mut hexs = String::new();
    for b in d.iter() {
        hexs.push_str(&format!("{:02x}", b));
    }
    assert_eq!(hexs, "ba458d8d259c4e3c6318ba27d1291fe075ef2df35bbc9dab13d44b8db947dfcd");
}

// ---------------------------------------------------------------------------
// NTT roundtrip + convolution vs scalar references
// ---------------------------------------------------------------------------

/// intt_tomont(ntt(p)) == p * 2^32 mod q (centered), scalar-referenced.
#[test]
fn ntt_intt_tomont_vs_scalar() {
    let mut rng = Lcg(0xA20C_0FFE_E5A7_E001);
    for _ in 0..8 {
        let p = rng.poly();
        let mut g = p;
        mldsa::ntt(&mut g);
        // ref contract: intt inputs must be smaller than Q in abs value
        for c in g.iter_mut() { *c = mldsa::reduce32(*c); }
        mldsa::intt_tomont(&mut g);
        for i in 0..mldsa::N {
            let expect = ((p[i] as i64).rem_euclid(mldsa::Q as i64) * (1u64 << 32) as i64
                % mldsa::Q as i64) as i32;
            let got = g[i].rem_euclid(mldsa::Q);
            assert_eq!(got, expect, "coeff {}", i);
        }
    }
}

/// intt_plain(ntt(p)) == p exactly (plain-domain identity, scalar-referenced
/// by construction: the scalar expected value is p itself).
#[test]
fn ntt_intt_plain_identity_vs_scalar() {
    let mut rng = Lcg(0xA20C_0FFE_E5A7_E002);
    for _ in 0..8 {
        let p = rng.poly();
        let mut g = p;
        mldsa::ntt(&mut g);
        // ref contract: intt inputs must be smaller than Q in abs value
        for c in g.iter_mut() { *c = mldsa::reduce32(*c); }
        mldsa::intt_plain(&mut g);
        for i in 0..mldsa::N {
            assert_eq!(
                g[i].rem_euclid(mldsa::Q),
                mldsa::reduce32(p[i]).rem_euclid(mldsa::Q),
                "coeff {}",
                i
            );
        }
    }
}

/// ntt + pointwise_montgomery + intt_plain == schoolbook negacyclic
/// convolution in i64 (the scalar reference, computed directly).
#[test]
fn ntt_pointwise_convolution_vs_scalar() {
    let mut rng = Lcg(0xA20C_0FFE_E5A7_E003);
    let n = mldsa::N as usize;
    for _ in 0..4 {
        let a = rng.poly();
        let b = rng.poly();
        // schoolbook negacyclic convolution mod q
        let mut expect = [0i64; mldsa::N];
        for i in 0..n {
            for j in 0..n {
                let v = a[i] as i64 * b[j] as i64;
                if i + j < n {
                    expect[i + j] += v;
                } else {
                    expect[i + j - n] -= v;
                }
            }
        }
        let mut ahat = a;
        let mut bhat = b;
        mldsa::ntt(&mut ahat);
        mldsa::ntt(&mut bhat);
        // ref contract: pointwise inputs in Montgomery domain, |coeff| < q
        for c in ahat.iter_mut() { *c = mldsa::reduce32(*c); }
        for c in bhat.iter_mut() { *c = mldsa::reduce32(*c); }
        let mut ch = [0i32; mldsa::N];
        mldsa::pointwise_montgomery(&mut ch, &ahat, &bhat);
        mldsa::intt_tomont(&mut ch);
        for i in 0..n {
            let got = ch[i].rem_euclid(mldsa::Q);
            let want = (expect[i].rem_euclid(mldsa::Q as i64)) as i32;
            assert_eq!(got, want, "coeff {}", i);
        }
    }
}

// ---------------------------------------------------------------------------
// Rounding / hint courts (spec law, exhaustively bounded)
// ---------------------------------------------------------------------------

/// Scalar decompose reference: ceil-bucket with the collapse at bucket 16.
fn decompose_ref(a: i32) -> (i32, i32) {
    let alpha = mldsa::ALPHA;
    let mut r1 = (a + mldsa::GAMMA2 - 1) / alpha;
    if r1 == mldsa::M_BUCKETS {
        r1 = 0;
    }
    let mut r0 = a - r1 * alpha;
    if r0 > (mldsa::Q - 1) / 2 {
        r0 -= mldsa::Q;
    }
    (r1, r0)
}

/// Exhaustive tiling: for every a in [0, q), crate decompose must equal the
/// scalar reference, reconstruct a mod q, and stay in the bucket ranges
/// (r0 in [-gamma2, gamma2]; the single specified edge bucket carries
/// r0 = -gamma2 exactly).
#[test]
fn decompose_tiling_exhaustive_vs_scalar() {
    let mut q_ = mldsa::Q;
    for a in 0..q_ {
        let (r1, r0) = mldsa::decompose(a);
        let (e1, e0) = decompose_ref(a);
        assert_eq!((r1, r0), (e1, e0), "a = {}", a);
        assert_eq!((r1 as i64 * mldsa::ALPHA as i64 + r0 as i64).rem_euclid(mldsa::Q as i64),
            (a as i64).rem_euclid(mldsa::Q as i64), "a = {}", a);
        assert!(r1 >= 0 && r1 < mldsa::M_BUCKETS, "a = {}", a);
        assert!(r0 >= -mldsa::GAMMA2 && r0 <= mldsa::GAMMA2, "a = {}", a);
    }
    q_ = 0; // silence unused-mut warning if any
    let _ = q_;
}

/// Power2Round exhaustive tiling: a1*2^13 + a0 == a, a0 in (-2^12, 2^12].
#[test]
fn power2round_tiling_exhaustive() {
    for a in 0..mldsa::Q {
        let (r1, r0) = mldsa::power2round(a);
        assert_eq!(r1 * (1 << mldsa::D) + r0, a, "a = {}", a);
        assert!(r0 > -(1 << (mldsa::D - 1)) && r0 <= (1 << (mldsa::D - 1)), "a = {}", a);
    }
}

/// Hint property (reference sign.c compound convention): for w in [0,q) and
/// verifier perturbation z with |z| <= 2048 (< gamma2 - beta = 261692, well
/// inside the correctness region), UseHint(h, w + z) == HighBits(w) where
/// h = make_hint(LowBits(w) + z, HighBits(w)). Exhaustive z sweep over a
/// deterministic random w sample.
#[test]
fn hint_usehint_property_court() {
    let mut rng = Lcg(0xA20C_0FFE_E5A7_E004);
    for _ in 0..48 {
        let w = (rng.next() % mldsa::Q as u64) as i32;
        let (a1w, a0w) = mldsa::decompose(w);
        for z in -2048i32..=2048 {
            let h = mldsa::make_hint(a0w + z, a1w);
            let wz = (w + z).rem_euclid(mldsa::Q);
            assert_eq!(
                mldsa::use_hint(wz, h),
                a1w,
                "w = {}, z = {}",
                w,
                z
            );
        }
    }
}
