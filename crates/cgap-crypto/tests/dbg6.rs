use cgap_crypto::fips203 as f3;
#[test]
fn mirror_cmp() {
    let d = [7u8; 32];
    let mut buf = [0u8; 33];
    buf[..32].copy_from_slice(&d);
    buf[32] = 3;
    let mut g = [0u8; 64];
    f3::hash_g(&mut g, &buf);
    let rho: [u8; 32] = g[..32].try_into().unwrap();
    let sigma: [u8; 32] = g[32..].try_into().unwrap();
    let s0 = f3::sample_poly_cbd_prf(&sigma, 0);
    let mut digest = [0u8; 32];
    let s0_str = format!("{:?}", s0);
    // cheap fingerprint: sum + xor
    let sum: i64 = s0.iter().map(|&x| x as i64).sum();
    let xsum: i64 = s0.iter().map(|&x| x as i64).product::<i64>().signum();
    println!("S0SUM={}", sum);
    println!("S016={:?}", &s0[..16]);
    let mut sh0 = s0;
    f3::ntt(&mut sh0);
    println!("SHFULL={:?}", &sh0[..128]);
    println!("FQLAW={}", f3::fqmul(-1044, -1044));
    let mut probe = [0i16; 256];
    probe[1] = 1;
    f3::ntt(&mut probe);
    println!("NTTX={:?}", &probe[..4]);
    // manual stage-1 trace: zeta=-758, butterflies (j, j+128), j=0..127
    let mut st1 = s0;
    let mut outs = [0i32; 4];
    let zeta = -758i16;
    for j in 0..4 {
        let t = f3::fqmul(zeta, st1[j + 128]);
        outs[j] = ((st1[j] - t) as i32) * 1000 + ((st1[j] + t) as i32);
        st1[j + 128] = st1[j] - t;
        st1[j] = st1[j] + t;
    }
    println!("ST1R={}", outs.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
    let st1b = s0;
    let mut outs2 = [0i32; 4];
    for j in 0..4 {
        let t = f3::fqmul(zeta, st1b[j + 128]);
        outs2[j] = ((st1b[j] - t) as i32) * 1000 + ((st1b[j] + t) as i32);
    }
    println!("ST1R2={}", outs2.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
}
