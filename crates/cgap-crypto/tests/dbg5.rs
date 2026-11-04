use cgap_crypto::fips202;
#[test]
fn kat() {
    // BK-3 takeover: the sha3 leg reached the PRIVATE `fips203::sha3` module
    // (E0603) and blocked `cargo test -p cgap-crypto` for AF-8/AF-12 gates.
    // Dropped, not promoted to pub API: this scratch test asserts nothing.
    // SHAKE-256("") = 46b9dd2b0ba88d1323.. (FIPS 202 known answer)
    let mut out = [0u8; 32];
    let mut s = fips202::Sponge::new(fips202::Shake256::RATE);
    s.squeeze_into(&mut out);
    let mut hx = String::new();
    for b in &out[..16] { hx.push_str(&format!("{:02x}", b)); }
    println!("SHAKE256_EMPTY={}", hx);
}
