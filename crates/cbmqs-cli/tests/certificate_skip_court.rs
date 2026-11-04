//! C8 — certificate skip court (§6 disqualification law, loud skip).
//! A sandbox copy of the AB-14 receipt with its `min-of-3` disclosure
//! stripped makes the §6 G7 source unmeasurable → typed
//! `CertificateIncomplete` → emit exits 3 and NO artifact is written.
//! The shared tree (/Users/sac/ggen receipt) is never mutated.

use std::path::PathBuf;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_cbmqs-audit");
const AB14_RECEIPT: &str = "/Users/sac/ggen/docs/jira/v26.11.3/receipts/ab-14-chain-scale.md";

#[test]
fn court_min_of_3_strip_fails_closed_no_artifact() {
    let dir = std::env::temp_dir().join(format!("cbmqs-c8-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("sandbox create");

    let source = std::fs::read_to_string(AB14_RECEIPT)
        .expect("AB-14 receipt readable (single §6 source of truth)");
    assert!(
        source.contains("min-of-3"),
        "precondition: landed AB-14 receipt carries the min-of-3 disclosure"
    );

    // Leg 1: unmutated sandbox copy -> emit exits 0, artifact written.
    let intact = dir.join("ab-14-intact.md");
    std::fs::write(&intact, &source).unwrap();
    let cert_ok: PathBuf = dir.join("cert-intact.json");
    let out = Command::new(BIN)
        .args([
            "--emit-certificate",
            cert_ok.to_str().unwrap(),
            "--run-seq",
            "7",
            "--g7-receipt",
            intact.to_str().unwrap(),
        ])
        .output()
        .expect("spawn emit (intact)");
    assert_eq!(out.status.code(), Some(0), "intact receipt emits");
    assert!(cert_ok.exists(), "intact emit wrote the certificate");

    // Leg 2: strip EVERY min-of-3 disclosure from the sandbox copy.
    let stripped: String = source
        .lines()
        .filter(|l| !l.contains("min-of-3"))
        .collect::<Vec<_>>()
        .join("\n");
    assert_ne!(stripped, source, "strip must change the receipt copy");
    let mutated = dir.join("ab-14-stripped.md");
    std::fs::write(&mutated, &stripped).unwrap();

    // The kernel refuses the now-unmeasurable §6 source typed. If the
    // gate somehow passes on a stripped receipt, this court is vacuous.
    let cert_bad: PathBuf = dir.join("cert-stripped.json");
    let out = Command::new(BIN)
        .args([
            "--emit-certificate",
            cert_bad.to_str().unwrap(),
            "--run-seq",
            "7",
            "--g7-receipt",
            mutated.to_str().unwrap(),
        ])
        .output()
        .expect("spawn emit (stripped)");
    let code = out.status.code().expect("emit exit code");
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert_eq!(code, 3, "stripped receipt must exit 3 (loud skip), got {code}: {stderr}");
    assert!(
        stderr.contains("CertificateIncomplete"),
        "typed CertificateIncomplete required, got: {stderr}"
    );
    assert!(!cert_bad.exists(), "NO certificate artifact on loud skip");
    assert!(
        !PathBuf::from(format!("{}.seal", cert_bad.display())).exists(),
        "NO seal sidecar on loud skip"
    );

    // Restore law: the shared AB-14 receipt is untouched (byte-identical).
    let after = std::fs::read_to_string(AB14_RECEIPT).expect("receipt still readable");
    assert_eq!(after, source, "shared tree must be byte-identical after the court");

    let _ = std::fs::remove_dir_all(&dir);
}
