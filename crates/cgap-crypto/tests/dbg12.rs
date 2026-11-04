use cgap_crypto::fips203 as f3;
#[test]
fn dec_chain() {
    let d = [7u8; 32];
    let mut buf = [0u8; 33];
    buf[..32].copy_from_slice(&d);
    buf[32] = 3;
    let mut g = [0u8; 64];
    f3::hash_g(&mut g, &buf);
    let sigma: [u8; 32] = g[32..].try_into().unwrap();
    let mut sh = [[0i16; 256]; 3];
    for i in 0..3 {
        sh[i] = f3::sample_poly_cbd_prf(&sigma, i as u8);
        f3::ntt(&mut sh[i]);
    }
    let mut dk_head = [0u8; 1152];
    for i in 0..3 {
        let mut pb = [0u8; 384];
        f3::poly_tobytes(&mut pb, &sh[i]);
        dk_head[384*i..384*(i+1)].copy_from_slice(&pb);
    }
    let ct: [u8; 1088] = {
        let mut c = [0u8; 1088];
        let hexs = "459f33fd2319498f268ce825ba9e26718a05e2bfb91d398068f1a1b782a040f9";
        // full ct not needed; only structurally exercise decompress
        c
    };
    let _ = ct;
    // decompress probe: compress then decompress u poly directly
    let mut u = [0i16; 256];
    for i in 0..256 { u[i] = ((i * 7) % 3329) as i16; }
    let mut packed = [0u8; 320];
    f3::poly_compress10(&mut packed, &u);
    let mut back = [0i16; 256];
    f3::poly_decompress10(&mut back, &packed);
    println!("U0={:?}", &back[..8]);
    println!("UE={:?}", &[((0*7)%3329*3329+512)>>10, ((1*7)%3329*3329+512)>>10, 23, 23]);
}
