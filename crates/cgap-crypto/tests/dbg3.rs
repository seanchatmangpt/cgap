use cgap_crypto::kem;
#[test]
fn print_ek() {
    let d = [7u8; 32];
    let z = [9u8; 32];
    let mut ek = [0u8; kem::EK_LEN];
    let mut dk = [0u8; kem::DK_LEN];
    kem::keygen(&d, &z, &mut ek, &mut dk);
    let mut hexs = String::new();
    for b in ek.iter() { hexs.push_str(&format!("{:02x}", b)); }
    println!("EK={}", &hexs[..64]);
    let m = [3u8; 32];
    let mut ct = [0u8; kem::CT_LEN];
    let mut ss = [0u8; 32];
    kem::encaps(&ek, &m, &mut ct, &mut ss);
    let mut ch = String::new();
    for b in ct.iter() { ch.push_str(&format!("{:02x}", b)); }
    println!("CT={}", &ch[..64]);
}
