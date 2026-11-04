use cgap_crypto::fips203 as f3;
use cgap_crypto::kem;

#[test]
#[allow(dead_code)]
fn unsafe_vec_i16(arr: &[i16; 256]) -> Vec<u8> {
    let mut v = Vec::new();
    for x in arr.iter() { v.extend_from_slice(&x.to_le_bytes()); }
    v
}

#[test]
fn dec_full() {
    let d = [7u8; 32]; let z = [9u8; 32];
    let mut ek = [0u8; 1184]; let mut dk = [0u8; 2400];
    kem::keygen(&d, &z, &mut ek, &mut dk);
    let m = [3u8; 32];
    let mut ct = [0u8; 1088]; let mut ss = [0u8; 32];
    kem::encaps(&ek, &m, &mut ct, &mut ss);
    let mut s_hat = [[0i16; 256]; 3];
    for i in 0..3 {
        let mut pb = [0u8; 384];
        pb.copy_from_slice(&dk[384*i..384*(i+1)]);
        f3::poly_frombytes(&mut s_hat[i], &pb);
    }
    let mut u = [[0i16; 256]; 3];
    let mut ub = [0u8; 320];
    for j in 0..3 {
        ub.copy_from_slice(&ct[320*j..320*(j+1)]);
        f3::poly_decompress10(&mut u[j], &ub);
    }
    let mut vb = [0u8; 128];
    vb.copy_from_slice(&ct[960..1088]);
    let mut v = [0i16; 256];
    f3::poly_decompress4(&mut v, &vb);
    println!("U_RS={:?}", &u[0][..8]);
    let mut w = v;
    let mut acc = [0i16; 256];
    let mut prod = [0i16; 256];
    for i in 0..3 {
        let mut ui = u[i];
        f3::ntt(&mut ui);
        f3::poly_basemul_montgomery(&mut prod, &s_hat[i], &ui);
        let hx: String = prod.iter().map(|v| format!("{:04x}", (*v as i32).rem_euclid(3329) as u16)).collect();
        std::fs::write(format!("/tmp/rust_sh{}.bin", i), unsafe_vec_i16(&s_hat[i])).unwrap();
        std::fs::write(format!("/tmp/rust_u{}.bin", i), unsafe_vec_i16(&ui)).unwrap();
        println!("PRDF{}={}", i, hx);
        f3::poly_add_assign(&mut acc, &prod);
    }
    println!("ACC_RS={:?}", &acc[..8]);
    f3::intt_tomont(&mut acc);
    println!("ACCPOST={:?}", &acc[..8]);
    f3::poly_sub_assign(&mut w, &acc);
    f3::poly_reduce(&mut w);
    println!("W={:?}", &w[..8]);
    let mut m2 = [0u8; 32];
    f3::poly_tomsg(&mut m2, &w);
    println!("M2={:?}", &m2[..8]);
}
