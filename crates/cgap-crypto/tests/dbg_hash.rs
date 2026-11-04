use cgap_crypto::fips203 as f3;
#[test]
fn hashes() {
    let d = [7u8; 32];
    let mut buf = [0u8; 33];
    buf[..32].copy_from_slice(&d);
    buf[32] = 3;
    let mut g = [0u8; 64];
    f3::hash_g(&mut g, &buf);
    println!("rho {}", hex(&g[..32]));
    println!("sigma {}", hex(&g[32..]));
    let mut h = [0u8; 32];
    f3::hash_h(&mut h, b"");
    println!("H_empty {}", hex(&h));
    let mut s = fips202_shake256_empty();
    println!("SHAKE256_EMPTY {}", s);
}
fn hex(b: &[u8]) -> String { b.iter().map(|x| format!("{:02x}", x)).collect() }
fn fips202_shake256_empty() -> String {
    use cgap_crypto::fips202;
    let mut out = [0u8; 32];
    let mut s = fips202::Sponge::new(fips202::Shake256::RATE);
    s.squeeze_into(&mut out);
    out.iter().map(|x| format!("{:02x}", x)).collect()
}
