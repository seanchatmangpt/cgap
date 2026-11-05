//! C9 — subprocess parity court: in-test emit -> bin --verify-certificate
//! exit 0 roundtrip, then four NAMED mutations each flip the exit code.
//! If any named mutation does NOT flip, the court is VACUOUS and fails.

use std::path::PathBuf;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_cbmqs-audit");
const AB14_RECEIPT: &str = "/Users/sac/ggen/docs/jira/v26.11.3/receipts/ab-14-chain-scale.md";

fn run(args: &[&str]) -> (i32, String) {
    let out = Command::new(BIN)
        .args(args)
        .output()
        .expect("spawn cbmqs-audit");
    (
        out.status.code().expect("exit code"),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

fn sandbox(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("cbmqs-c9-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("sandbox create");
    dir
}

#[test]
fn court_parity_roundtrip_and_four_mutations_flip() {
    let dir = sandbox("main");
    let source = std::fs::read_to_string(AB14_RECEIPT).expect("AB-14 receipt readable");
    let receipt = dir.join("ab-14.md");
    std::fs::write(&receipt, &source).unwrap();
    let receipt_s = receipt.to_str().unwrap();

    // Roundtrip: emit -> verify exit 0.
    let cert = dir.join("cert.json");
    let cert_s = cert.to_str().unwrap();
    let (code, out) = run(&[
        "--emit-certificate",
        cert_s,
        "--run-seq",
        "7",
        "--g7-receipt",
        receipt_s,
    ]);
    assert_eq!(code, 0, "emit must pass: {out}");
    let (code, out) = run(&["--verify-certificate", cert_s]);
    assert_eq!(code, 0, "verify roundtrip must pass: {out}");

    let original = std::fs::read_to_string(&cert).unwrap();
    let seal_path = format!("{cert_s}.seal");
    let original_seal = std::fs::read_to_string(&seal_path).unwrap();
    let mut flips = 0;

    // Mutation 1 (gate-byte): flip a verdict kind in the body.
    let m1 = original.replacen("\"kind\":\"pass\"", "\"kind\":\"fail\"", 1);
    assert_ne!(m1, original, "gate-byte mutation must change bytes");
    std::fs::write(&cert, &m1).unwrap();
    let (code, _) = run(&["--verify-certificate", cert_s]);
    assert_eq!(code, 2, "gate-byte mutation must flip to exit 2");
    flips += 1;

    // Mutation 2 (run_seq swap): rebinds the (run_seq, input_hash) identity.
    std::fs::write(&cert, &original).unwrap();
    let m2 = original.replace("\"run_seq\":7", "\"run_seq\":8");
    assert_ne!(m2, original);
    std::fs::write(&cert, &m2).unwrap();
    let (code, _) = run(&["--verify-certificate", cert_s]);
    assert_eq!(code, 2, "run_seq swap must flip to exit 2");
    flips += 1;

    // Mutation 3 (min-of-3 strip): emit against the stripped §6 source
    // must exit 3 and write NO artifact.
    let stripped: String = source
        .lines()
        .filter(|l| !l.contains("min-of-3"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_ne!(stripped, source, "min-of-3 strip must change the receipt");
    let mutated = dir.join("ab-14-stripped.md");
    std::fs::write(&mutated, &stripped).unwrap();
    let cert3 = dir.join("cert-stripped.json");
    let (code, out) = run(&[
        "--emit-certificate",
        cert3.to_str().unwrap(),
        "--run-seq",
        "7",
        "--g7-receipt",
        mutated.to_str().unwrap(),
    ]);
    assert_eq!(code, 3, "min-of-3 strip must flip emit to exit 3: {out}");
    assert!(!cert3.exists(), "no artifact on loud skip");
    flips += 1;

    // Mutation 4 (seal truncation): 32-char half-seal must refuse.
    std::fs::write(&cert, &original).unwrap();
    std::fs::write(&seal_path, &original_seal[..32]).unwrap();
    let (code, _) = run(&["--verify-certificate", cert_s]);
    assert_eq!(code, 2, "seal truncation must flip to exit 2");
    flips += 1;

    // Non-vacuity: all four named mutations flipped. A green run where
    // none flips would be a VACUOUS court.
    assert_eq!(flips, 4, "all four named mutations must flip (got {flips})");

    // Restore sanity: original cert + seal verify green again.
    std::fs::write(&seal_path, &original_seal).unwrap();
    let (code, out) = run(&["--verify-certificate", cert_s]);
    assert_eq!(code, 0, "restored pair verifies green: {out}");

    let _ = std::fs::remove_dir_all(&dir);
}
