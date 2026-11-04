//! kem - ML-KEM-768 key encapsulation (FIPS 203 modules 5-7).
//!
//! # PROVENANCE
//!
//! FO transform derived from the pqcrystals/kyber reference lineage
//! (PQClean `crypto_kem_*_derand`); FIPS 203 final semantics
//! (eta1=eta2=2, K = G(m || H(ek)), rejection key J(z || ct)).
//!
//! Deterministic cores: NO RNG in-crate. `keygen(d, z)`, `encaps(ek, m)`,
//! `decaps(dk, ct)`. Decaps is INFALLIBLE by type: both candidate keys are
//! computed unconditionally and selected branchlessly (byte cmov).
//! `#[inline(always)]` banned on the three entry symbols (named symbols
//! are the OQ-4 rekey-cost bench targets).
//!
//! Feature `decaps-only`: keygen/encaps cfg'd out when enabled.

use crate::fips203 as f3;

/// Encapsulation key bytes (1184).
pub const EK_LEN: usize = f3::EK_BYTES;
/// Decapsulation key bytes (2400).
pub const DK_LEN: usize = f3::DK_BYTES;
/// Ciphertext bytes (1088).
pub const CT_LEN: usize = f3::CT_BYTES;
/// Shared secret bytes (32).
pub const SS_LEN: usize = 32;

const _: () = {
    assert!(EK_LEN == 1184 && DK_LEN == 2400 && CT_LEN == 1088 && SS_LEN == 32);
    assert!(DK_LEN == 3 * 384 + EK_LEN + 64);
    assert!(CT_LEN == 3 * 320 + 128);
};

/// Branchless byte select (cmov class).
fn cmov(r: &mut [u8], x: &[u8], b: u8) {
    let m = (b as u8).wrapping_neg();
    for i in 0..r.len() {
        r[i] ^= m & (r[i] ^ x[i]);
    }
}

// ---------------------------------------------------------------------------
// KeyGen(d, z) -> (ek, dk), deterministic, no RNG
// ---------------------------------------------------------------------------

#[cfg_attr(feature = "decaps-only", allow(dead_code))]
pub fn keygen(d: &[u8; 32], z: &[u8; 32], ek: &mut [u8; EK_LEN], dk: &mut [u8; DK_LEN]) {
    let mut buf = [0u8; 33];
    buf[..32].copy_from_slice(d);
    buf[32] = f3::K as u8;
    let mut g = [0u8; 64];
    f3::hash_g(&mut g, &buf);
    let rho: [u8; 32] = g[..32].try_into().unwrap();
    let sigma: [u8; 32] = g[32..].try_into().unwrap();

    let mut a = [[[0i16; f3::N]; f3::K]; f3::K];
    f3::gen_matrix(&mut a, &rho, false);
    let mut s_hat = [[0i16; f3::N]; f3::K];
    let mut e_hat = [[0i16; f3::N]; f3::K];
    for i in 0..f3::K {
        s_hat[i] = f3::sample_poly_cbd_prf(&sigma, i as u8);
    }
    for i in 0..f3::K {
        e_hat[i] = f3::sample_poly_cbd_prf(&sigma, (f3::K + i) as u8);
    }
    for p in s_hat.iter_mut() {
        f3::ntt(p);
    }
    for p in e_hat.iter_mut() {
        f3::ntt(p);
    }

    let mut t_hat = [[0i16; f3::N]; f3::K];
    for i in 0..f3::K {
        let mut acc = [0i16; f3::N];
        let mut prod = [0i16; f3::N];
        for j in 0..f3::K {
            f3::poly_basemul_montgomery(&mut prod, &a[i][j], &s_hat[j]);
            f3::poly_add_assign(&mut acc, &prod);
        }
        f3::poly_tomont(&mut acc);
        f3::poly_add_assign(&mut acc, &e_hat[i]);
        f3::poly_reduce(&mut acc);
        t_hat[i] = acc;
    }

    // Serialize ek = t_hat || rho ; dk = s_hat || ek || H(ek) || z.
    let mut ek_full = [0u8; EK_LEN];
    for i in 0..f3::K {
        let mut pb = [0u8; 384];
        f3::poly_tobytes(&mut pb, &t_hat[i]);
        ek_full[384 * i..384 * (i + 1)].copy_from_slice(&pb);
    }
    ek_full[1152..].copy_from_slice(&rho);
    let mut h_ek = [0u8; 32];
    f3::hash_h(&mut h_ek, &ek_full);
    for i in 0..f3::K {
        let mut pb = [0u8; 384];
        let mut si = s_hat[i];
        // FIPS 203: dk stores canonical ByteEncode12(shat). Kyber-R3 code
        // packed shat unreduced (self-consistent with its own frombytes,
        // but NOT conformant); liboqs/FIPS 203 reduces first. Reduce here.
        f3::poly_reduce(&mut si);
        f3::poly_tobytes(&mut pb, &si);
        dk[384 * i..384 * (i + 1)].copy_from_slice(&pb);
    }
    dk[1152..1152 + EK_LEN].copy_from_slice(&ek_full);
    dk[1152 + EK_LEN..1152 + EK_LEN + 32].copy_from_slice(&h_ek);
    dk[1152 + EK_LEN + 32..].copy_from_slice(z);
    *ek = ek_full;
}

// ---------------------------------------------------------------------------
// Encaps(ek, m) -> (ct, ss), deterministic, no RNG
// ---------------------------------------------------------------------------

#[cfg_attr(feature = "decaps-only", allow(dead_code))]
pub fn encaps(ek: &[u8; EK_LEN], m: &[u8; 32], ct: &mut [u8; CT_LEN], ss: &mut [u8; SS_LEN]) {
    let mut h_ek = [0u8; 32];
    f3::hash_h(&mut h_ek, ek);
    let mut kr = [0u8; 64];
    f3::hash_g_parts(&mut kr, &[m, &h_ek]);
    let mut r = [0u8; 32];
    r.copy_from_slice(&kr[32..]);
    pke_encrypt(ek, m, &r, ct);
    ss.copy_from_slice(&kr[..32]);
}

// ---------------------------------------------------------------------------
// K-PKE encrypt
// ---------------------------------------------------------------------------

#[cfg_attr(feature = "decaps-only", allow(dead_code))]
fn pke_encrypt(ek: &[u8; EK_LEN], m: &[u8; 32], r: &[u8; 32], ct: &mut [u8; CT_LEN]) {
    let mut t_hat = [[0i16; f3::N]; f3::K];
    let mut rho = [0u8; 32];
    for i in 0..f3::K {
        let mut pb = [0u8; 384];
        pb.copy_from_slice(&ek[384 * i..384 * (i + 1)]);
        f3::poly_frombytes(&mut t_hat[i], &pb);
    }
    rho.copy_from_slice(&ek[1152..]);
    let mut at = [[[0i16; f3::N]; f3::K]; f3::K];
    f3::gen_matrix(&mut at, &rho, true);
    let mut y_hat = [[0i16; f3::N]; f3::K];
    for i in 0..f3::K {
        y_hat[i] = f3::sample_poly_cbd_prf(r, i as u8);
        f3::ntt(&mut y_hat[i]);
    }
    let mut e1 = [[0i16; f3::N]; f3::K];
    for i in 0..f3::K {
        e1[i] = f3::sample_poly_cbd_prf(r, (f3::K + i) as u8);
    }
    let mut e2 = f3::sample_poly_cbd_prf(r, (2 * f3::K) as u8);

    // u_j = sum_i A^T[j][i] . y_i  + e1_j   (invntt_tomont then add)
    let mut u = [[0i16; f3::N]; f3::K];
    for j in 0..f3::K {
        let mut acc = [0i16; f3::N];
        let mut prod = [0i16; f3::N];
        for i in 0..f3::K {
            f3::poly_basemul_montgomery(&mut prod, &at[j][i], &y_hat[i]);
            f3::poly_add_assign(&mut acc, &prod);
        }
        f3::intt_tomont(&mut acc);
        f3::poly_add_assign(&mut acc, &e1[j]);
        f3::poly_reduce(&mut acc);
        u[j] = acc;
    }

    // v = sum_i t_hat[i] . y_hat[i]  + e2 + m
    let mut v = [0i16; f3::N];
    {
        let mut acc = [0i16; f3::N];
        let mut prod = [0i16; f3::N];
        for i in 0..f3::K {
            f3::poly_basemul_montgomery(&mut prod, &t_hat[i], &y_hat[i]);
            f3::poly_add_assign(&mut acc, &prod);
        }
        f3::intt_tomont(&mut acc);
        f3::poly_add_assign(&mut acc, &e2);
        let mut mp = [0i16; f3::N];
        f3::poly_frommsg(&mut mp, m);
        f3::poly_add_assign(&mut acc, &mp);
        f3::poly_reduce(&mut acc);
        v = acc;
    }
    // ct = compress10(u) || compress4(v)
    let mut cb = [0u8; 320];
    for j in 0..f3::K {
        let mut pj = u[j];
        f3::poly_compress10(&mut cb, &pj);
        ct[320 * j..320 * (j + 1)].copy_from_slice(&cb);
    }
    let mut cb128 = [0u8; 128];
    f3::poly_compress4(&mut cb128, &v);
    ct[960..1088].copy_from_slice(&cb128);
}

// ---------------------------------------------------------------------------
// K-PKE decrypt
// ---------------------------------------------------------------------------

fn pke_decrypt(dk: &[u8; DK_LEN], ct: &[u8; CT_LEN]) -> [u8; 32] {
    let mut s_hat = [[0i16; f3::N]; f3::K];
    for i in 0..f3::K {
        let mut pb = [0u8; 384];
        pb.copy_from_slice(&dk[384 * i..384 * (i + 1)]);
        f3::poly_frombytes(&mut s_hat[i], &pb);
    }
    let mut u = [[0i16; f3::N]; f3::K];
    let mut ub = [0u8; 320];
    for j in 0..f3::K {
        ub.copy_from_slice(&ct[320 * j..320 * (j + 1)]);
        f3::poly_decompress10(&mut u[j], &ub);
    }
    let mut vb = [0u8; 128];
    vb.copy_from_slice(&ct[960..1088]);
    let mut v = [0i16; f3::N];
    f3::poly_decompress4(&mut v, &vb);

    // w = v - s_hat^T . u_hat   (start from v, subtract the product)
    let mut w = v;
    {
        let mut acc = [0i16; f3::N];
        let mut prod = [0i16; f3::N];
        for i in 0..f3::K {
            let mut ui = u[i];
            f3::ntt(&mut ui);
            f3::poly_basemul_montgomery(&mut prod, &s_hat[i], &ui);
            f3::poly_add_assign(&mut acc, &prod);
        }
        f3::intt_tomont(&mut acc);
        f3::poly_sub_assign(&mut w, &acc);
        f3::poly_reduce(&mut w);
    }
    let mut m = [0u8; 32];
    f3::poly_tomsg(&mut m, &w);
    m
}

// ---------------------------------------------------------------------------
// Decaps(dk, ct) -> ss, INFALLIBLE (FO ct-select)
// ---------------------------------------------------------------------------

/// ML-KEM-768 Decaps. Infallible by type: both candidate keys are computed
/// unconditionally and selected branchlessly (byte cmov). No early return,
/// no branch on secret data; `#[inline(never)]` (OQ-4 bench target).
#[inline(never)]
pub fn decaps(dk: &[u8; DK_LEN], ct: &[u8; CT_LEN]) -> [u8; SS_LEN] {
    let m = pke_decrypt(dk, ct);
    let h_ek: [u8; 32] = dk[1152 + EK_LEN..1152 + EK_LEN + 32].try_into().unwrap();
    let mut kr = [0u8; 64];
    f3::hash_g_parts(&mut kr, &[&m, &h_ek]);
    let k_bar: [u8; 32] = kr[..32].try_into().unwrap();
    let r: [u8; 32] = kr[32..].try_into().unwrap();
    let ek: [u8; EK_LEN] = dk[1152..1152 + EK_LEN].try_into().unwrap();
    let mut ct2 = [0u8; CT_LEN];
    pke_encrypt(&ek, &m, &r, &mut ct2);
    // fail = 1 if ct != ct2 (OR-fold, branchless).
    let mut fold: u8 = 0;
    for i in 0..CT_LEN {
        fold |= ct[i] ^ ct2[i];
    }
    let fail = ((fold as u64).wrapping_neg() >> 63) as u8;
    // Rejection key J(z || ct), computed unconditionally.
    let z: [u8; 32] = dk[1152 + EK_LEN + 32..].try_into().unwrap();
    let mut rej = [0u8; 32];
    f3::hash_j(&mut rej, &z, ct);
    let mut ss = k_bar;
    cmov(&mut ss, &rej, fail);
    ss
}
