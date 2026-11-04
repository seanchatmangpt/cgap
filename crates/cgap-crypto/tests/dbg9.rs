use cgap_crypto::fips203 as f3;
#[test]
fn enc() {
    let mut p = [0i16; 256];
    p[0] = -623; p[1] = 629; p[2] = -1; p[3] = 106;
    let mut b = [0u8; 384];
    f3::poly_tobytes(&mut b, &p);
    let mut hx = String::new();
    for byte in &b[..6] { hx.push_str(&format!("{:02x}", byte)); }
    println!("ENC={}", hx);
    println!("CAQ={}", f3::cond_add_q(-623));
}
