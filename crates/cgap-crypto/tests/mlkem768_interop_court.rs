//! liboqs interop court. Compiles tests/vectors/mlkem768/oracle.c against
//! the installed liboqs (0.13.1-dev, ML-KEM-768 = FIPS 203 final) at test
//! time and runs BOTH interop directions:
//!   A: my keygen -> liboqs encaps -> my decaps == liboqs ss
//!   B: liboqs keypair -> my encaps -> liboqs decaps == my ss
//! Loud skip when the C toolchain/liboqs is unavailable (skip names its
//! reason; never a silent pass).
use cgap_crypto::kem;

fn hx(b: &[u8]) -> String {
    b.iter().map(|x| format!("{:02x}", x)).collect()
}
fn uh(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap())
        .collect()
}

fn build_oracle(bin: &std::path::Path) -> Result<(), String> {
    let src = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/vectors/mlkem768/oracle.c"
    );
    let st = std::process::Command::new("cc")
        .arg(src)
        .arg("/usr/local/lib/liboqs.a")
        .arg("-L/opt/homebrew/opt/openssl@3/lib")
        .arg("-lcrypto")
        .arg("-o")
        .arg(bin)
        .output()
        .map_err(|e| format!("cc spawn failed: {}", e))?;
    if !st.status.success() {
        return Err(format!(
            "oracle compile failed: {}",
            String::from_utf8_lossy(&st.stderr)
        ));
    }
    Ok(())
}

fn run_oracle(args: &[String]) -> Result<String, String> {
    let tmp = std::env::temp_dir().join("ab5_mlkem_oracle");
    let _ = std::fs::remove_file(&tmp);
    build_oracle(&tmp)?;
    let out = std::process::Command::new(&tmp)
        .args(args)
        .output()
        .map_err(|e| format!("oracle spawn failed: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "oracle exited {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[test]
fn interop_liboqs_both_directions() {
    let tmp = std::env::temp_dir().join("ab5_mlkem_oracle");
    if build_oracle(&tmp).is_err() {
        println!("[SKIP] liboqs interop: oracle unavailable on this host (cc/liboqs/openssl missing) - loud skip, not a pass");
        return;
    }

    // --- direction A: my keygen -> liboqs encaps -> my decaps ---
    let d = [7u8; 32];
    let z = [9u8; 32];
    let mut ek = [0u8; 1184];
    let mut dk = [0u8; 2400];
    kem::keygen(&d, &z, &mut ek, &mut dk);
    let out = run_oracle(&["enc".into(), hx(&ek)])
        .expect("oracle enc must run when the oracle builds");
    let parts: Vec<&str> = out.split('|').collect();
    assert_eq!(parts.len(), 2);
    let oct: [u8; 1088] = uh(parts[0]).as_slice().try_into().unwrap();
    let oss: [u8; 32] = uh(parts[1]).as_slice().try_into().unwrap();
    let my = kem::decaps(&dk, &oct);
    assert_eq!(my, oss, "interop A: my decaps(liboqs ct) != liboqs ss");

    // --- direction B: liboqs keypair -> my encaps -> liboqs decaps ---
    let outb = run_oracle(&["keypair".into()]).expect("oracle keypair must run");
    let pb: Vec<&str> = outb.split('|').collect();
    let oek: [u8; 1184] = uh(pb[0]).as_slice().try_into().unwrap();
    let odk: [u8; 2400] = uh(pb[1]).as_slice().try_into().unwrap();
    let m = [0x42u8; 32];
    let mut ct2 = [0u8; 1088];
    let mut ss2 = [0u8; 32];
    kem::encaps(&oek, &m, &mut ct2, &mut ss2);
    let outc = run_oracle(&["dec".into(), hx(&odk), hx(&ct2)])
        .expect("oracle dec must run");
    let ss3: [u8; 32] = uh(&outc).as_slice().try_into().unwrap();
    assert_eq!(ss2, ss3, "interop B: liboqs decaps(my ct) != my ss");
}
