//! C7 — certificate tamper court (real subprocess, /tmp sandbox copies,
//! never the shared tree). Emit a REAL certificate, then four mutations;
//! each MUST flip verify to exit 2 (typed refusal). A mutation that does
//! not flip is a vacuous court, not a pass.

use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_cbmqs-audit");
const RUN_SEQ: &str = "7";

struct Sandbox(std::path::PathBuf);

impl Sandbox {
    fn new(tag: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("cbmqs-c7-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("sandbox create");
        Sandbox(dir)
    }
    fn cert(&self) -> String {
        self.0.join("cert.json").to_string_lossy().into_owned()
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn emit(cert: &str) -> i32 {
    let out = Command::new(BIN)
        .args(["--emit-certificate", cert, "--run-seq", RUN_SEQ])
        .output()
        .expect("spawn cbmqs-audit emit");
    out.status.code().expect("emit exit code")
}

fn verify(cert: &str) -> (i32, String) {
    let out = Command::new(BIN)
        .args(["--verify-certificate", cert])
        .output()
        .expect("spawn cbmqs-audit verify");
    (
        out.status.code().expect("verify exit code"),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn first_replace(haystack: &str, from: &str, to: &str) -> String {
    haystack.replacen(from, to, 1)
}

#[test]
fn court_tamper_four_mutations_each_flip_verify_to_exit_2() {
    let sb = Sandbox::new("main");
    let cert = sb.cert();

    // Real emit: the honest overall today is DISQUALIFIED (G7 leg B
    // MEASURED-UNMET) — the seal binds the BODY, so this verifies fine.
    assert_eq!(emit(&cert), 0, "real emit must exit 0");
    let original = std::fs::read_to_string(&cert).expect("cert readable");
    assert!(
        std::path::Path::new(&format!("{cert}.seal")).exists(),
        "sidecar written"
    );
    assert_eq!(verify(&cert).0, 0, "baseline verify must pass");

    // Mutation 1 — flip a gates byte (verdict kind pass -> fail).
    let flipped = first_replace(&original, "\"kind\":\"pass\"", "\"kind\":\"fail\"");
    assert_ne!(flipped, original, "mutation must change bytes");
    std::fs::write(&cert, &flipped).unwrap();
    let (code, _) = verify(&cert);
    assert_eq!(code, 2, "gates-byte flip must refuse with exit 2");

    // Mutation 2 — flip one seal hex char (sidecar tamper).
    std::fs::write(&cert, &original).unwrap();
    let seal_path = format!("{cert}.seal");
    let seal = std::fs::read_to_string(&seal_path).unwrap();
    let first = seal.chars().next().expect("seal nonempty");
    let flipped_char = if first == '0' { '1' } else { '0' };
    std::fs::write(
        &seal_path,
        seal.replacen(first, &flipped_char.to_string(), 1),
    )
    .unwrap();
    let (code, _) = verify(&cert);
    assert_eq!(code, 2, "seal-hex flip must refuse with exit 2");

    // Mutation 3 — swap run_seq (identity law, typed RunSeqCollision).
    std::fs::write(&seal_path, &seal).unwrap();
    let seq_pat = format!("\"run_seq\":{RUN_SEQ}");
    let swapped = original.replace(
        &seq_pat,
        &format!("\"run_seq\":{}", RUN_SEQ.parse::<u64>().unwrap() + 1),
    );
    assert_ne!(swapped, original, "run_seq swap must change bytes");
    std::fs::write(&cert, &swapped).unwrap();
    let (code, stderr) = verify(&cert);
    assert_eq!(code, 2, "run_seq swap must refuse with exit 2");
    assert!(
        stderr.contains("RunSeqCollision"),
        "typed RunSeqCollision required, got: {stderr}"
    );

    // Mutation 4 — delete the sidecar.
    std::fs::write(&cert, &original).unwrap();
    std::fs::remove_file(&seal_path).unwrap();
    let (code, stderr) = verify(&cert);
    assert_eq!(code, 2, "missing sidecar must refuse with exit 2");
    assert!(
        stderr.contains("sidecar"),
        "missing-sidecar typing required, got: {stderr}"
    );
}

#[test]
fn court_run_seq_collision_law_typed() {
    // Same run_seq bound to a DIFFERENT input_hash -> typed RunSeqCollision.
    let sb = Sandbox::new("collision");
    let cert = sb.cert();
    assert_eq!(emit(&cert), 0);
    let original = std::fs::read_to_string(&cert).unwrap();
    // Rebind input_hash in-place (keeps the original seal -> BOTH the
    // identity law and the seal would refuse; the identity law fires
    // first, so the refusal must be TYPED RunSeqCollision).
    let idx = original
        .find("\"input_hash\":\"")
        .expect("input_hash present");
    let start = idx + "\"input_hash\":\"".len();
    let end = start + 64;
    let mut rebound = original.clone();
    // Flip the first hex char of the 64-char input_hash.
    let bytes: Vec<char> = original[start..end].chars().collect();
    let flipped: String = bytes
        .iter()
        .enumerate()
        .map(|(i, c)| {
            if i == 0 {
                if *c == '0' {
                    '1'
                } else {
                    '0'
                }
            } else {
                *c
            }
        })
        .collect();
    rebound.replace_range(start..end, &flipped);
    assert_ne!(rebound, original);
    std::fs::write(&cert, &rebound).unwrap();
    let (code, stderr) = verify(&cert);
    assert_eq!(code, 2);
    assert!(
        stderr.contains("RunSeqCollision"),
        "typed RunSeqCollision required, got: {stderr}"
    );
}

// ------------------------------------------------ DOD4 forgery courts
//
// Two attack classes D4 verified against the pre-repair binary
// (rc=0 VERIFIED for both) are now permanent regression legs:
//   A12 prepend-forgery  -> attacker bytes BEFORE the body hijacked the
//       first-occurrence string-scrape parser. New law: strict
//       fixed-order canonical parser -> any prepended/duplicated key
//       is Malformed, exit 2.
//   A13 reseal-forgery   -> the unkeyed seal lets any writer patch a
//       gate and recompute BLAKE3. New law: verify rehashes the
//       PRESENTED body's gates under the §6 law and refuses unless the
//       rehash equals the presented input_hash (BodyHashMismatch),
//       BEFORE the seal check. Exit 2.

use cgap_blake3::hash as attacker_hash;

/// Attacker-side reseal: hex BLAKE3-256 over the patched bytes — the
/// exact move D4's A13 proved sufficient against the old verify.
fn reseal(bytes: &[u8]) -> String {
    attacker_hash(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[test]
fn court_a12_prepend_forgery_refused_typed() {
    let sb = Sandbox::new("a12");
    let cert = sb.cert();
    assert_eq!(emit(&cert), 0, "real emit must exit 0");
    let original = std::fs::read_to_string(&cert).expect("cert readable");

    // Forge: attacker key-values FIRST (run_seq 2 + attacker input_hash),
    // then the lawful body — the exact A12 shape. The old first-
    // occurrence extractors parsed THESE; the fixed-order parser must
    // refuse them as a byte mismatch at `schema_version`.
    let forged = format!(
        "{{\"run_seq\":2,\"input_hash\":\"{}\",{}",
        "a".repeat(64),
        &original[1..]
    );
    assert_ne!(forged, original);
    std::fs::write(&cert, &forged).unwrap();
    std::fs::write(
        format!("{cert}.seal"),
        format!("{}\n", reseal(forged.as_bytes())),
    )
    .unwrap();
    let (code, stderr) = verify(&cert);
    assert_eq!(
        code, 2,
        "prepend forgery must refuse with exit 2, got: {stderr}"
    );
    assert!(
        stderr.contains("malformed"),
        "typed Malformed required, got: {stderr}"
    );

    // Variant: prepended DUPLICATE keys matching the real body's values
    // (hijack attempt with truthful-looking envelope keys).
    std::fs::write(&cert, &original).unwrap();
    let seal = std::fs::read_to_string(format!("{cert}.seal")).unwrap();
    let dup = original.replacen(
        '{',
        "{\"schema_version\":1,\"canonical_form\":\"cbmqs-certificate-v1\",\"seal_algorithm\":\"BLAKE3-256-hex\",",
        1,
    );
    assert_ne!(dup, original);
    std::fs::write(&cert, &dup).unwrap();
    std::fs::write(format!("{cert}.seal"), &seal).unwrap();
    let (code, stderr) = verify(&cert);
    assert_eq!(
        code, 2,
        "duplicate-key envelope must refuse with exit 2, got: {stderr}"
    );
    assert!(
        stderr.contains("malformed"),
        "typed Malformed required, got: {stderr}"
    );
}

#[test]
fn court_a13_reseal_forgery_refused_typed() {
    let sb = Sandbox::new("a13");
    let cert = sb.cert();
    assert_eq!(emit(&cert), 0, "real emit must exit 0");
    let original = std::fs::read_to_string(&cert).expect("cert readable");
    let seal_path = format!("{cert}.seal");

    // Control (non-vacuity): resealing the UNPATCHED bytes reproduces
    // the sidecar exactly — proves this court's reseal machinery is the
    // real seal law, not a strawman.
    let control = reseal(original.as_bytes());
    assert_eq!(
        control,
        std::fs::read_to_string(&seal_path).unwrap().trim(),
        "attacker reseal over unpatched bytes must equal the honest sidecar"
    );

    // Attack: patch a gate verdict pass -> fail, then reseal with the
    // SAME machinery. input_hash and run_seq stay lawful, so the
    // RunSeqCollision law passes; the seal check passes too (resealed).
    // Only the H2 body-hash law catches this — it must refuse exit 2.
    let patched = original.replacen(
        "\"kind\":\"pass\"",
        "\"kind\":\"fail\",\"why\":\"forged\"",
        1,
    );
    assert_ne!(patched, original, "gate patch must change bytes");
    std::fs::write(&cert, &patched).unwrap();
    std::fs::write(&seal_path, format!("{}\n", reseal(patched.as_bytes()))).unwrap();
    let (code, stderr) = verify(&cert);
    assert_eq!(
        code, 2,
        "reseal forgery must refuse with exit 2, got: {stderr}"
    );
    assert!(
        stderr.contains("BodyHashMismatch"),
        "typed BodyHashMismatch required, got: {stderr}"
    );

    // Restore law: the honest pair verifies green again.
    std::fs::write(&cert, &original).unwrap();
    std::fs::write(&seal_path, format!("{control}\n")).unwrap();
    assert_eq!(verify(&cert).0, 0, "restored honest pair must verify");
}
