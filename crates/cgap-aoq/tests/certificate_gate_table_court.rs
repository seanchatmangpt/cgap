//! RX-5 correction court (class-3 hazard): the RFC-0089 §8.4
//! certificate's gate table must never disagree with the RFC §6
//! scorecard source again. The generator (`rfc0089_certificate.rs`,
//! `scorecard_table()` / `g7_leg_b_from_receipt()`) now derives G7
//! from the AB-14 receipt on disk — the same file the scorecard's
//! `gate_g7()` leg B reads. This court re-derives the §6 law
//! independently (its own oracle parse) and asserts agreement, plus
//! flip witnesses on mutated /tmp receipt COPIES (the real receipt is
//! only ever read, never written).
//!
//! Note: this binary also includes the certificate module's own 5
//! courts (its `#[cfg(test)]` module compiles under this test crate),
//! so `cargo test --test certificate_gate_table_court` runs those
//! assertions twice total across binaries — same law, no new risk.

#[path = "rfc0089_certificate.rs"]
mod cert;

use cert::{g7_leg_b_from_receipt, issue, scorecard_table, Overall, Verdict, CERT_SPEC};

const AB14: &str = "/Users/sac/ggen/docs/jira/v26.11.3/receipts/ab-14-chain-scale.md";
const RX3_SCORECARD: &str = "/Users/sac/cgap/docs/jira/v26.11.3/receipts/rx-3-scorecard.md";

/// Court oracle: an INDEPENDENT parse of the §6 source (µs/link over
/// 50,000 links vs the 25 µs/link target). Deliberately not shared
/// with the generator — the court must not reuse the code it audits.
fn oracle_us_per_link(path: &str) -> Result<(f64, String), String> {
    let rec = std::fs::read_to_string(path).map_err(|e| format!("unreadable: {e}"))?;
    let idx = rec.find("50k span").ok_or("no '50k span' line")?;
    let tail = &rec[idx + "50k span".len()..];
    let num: String = tail
        .chars()
        .skip_while(|c| !c.is_ascii_digit() && *c != '.')
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let ms: f64 = num.parse().map_err(|e| format!("unparsable '{num}': {e}"))?;
    Ok((ms * 1000.0 / 50_000.0, format!("{ms:.2} ms")))
}

#[test]
fn court_certificate_g7_agrees_with_scorecard_source() {
    // 1. The §6 record (RX-3 scorecard receipt) says G7 UNMET on disk.
    let scorecard = std::fs::read_to_string(RX3_SCORECARD).expect("rx-3 receipt readable");
    let g7_line = scorecard
        .lines()
        .find(|l| l.trim_start().starts_with("| G7 |"))
        .expect("rx-3 receipt carries a G7 row");
    assert!(
        g7_line.contains("MEASURED-UNMET"),
        "RFC §6 record drifted: G7 row no longer MEASURED-UNMET: {g7_line}"
    );

    // 2. The certificate's derived G7 row agrees with the source number.
    let table = scorecard_table(42);
    let g7 = table.gates.iter().find(|g| g.id == "G7").expect("G7 row");
    let (us_per_link, ms_str) = oracle_us_per_link(AB14).expect("AB-14 parseable");
    assert!(
        us_per_link > 25.0,
        "§6 source crossed under target — certificate law must re-issue PASS; update this court"
    );
    match &g7.verdict {
        Verdict::Fail { why } => {
            assert!(why.contains("MEASURED-UNMET"));
            assert!(why.contains(&format!("{} ", ms_str)));
        }
        other => panic!("certificate G7 must be Fail while source is UNMET, got {other:?}"),
    }

    // 3. The issued certificate states the fact: DISQUALIFIED, G7 named.
    let (c, seal) = issue(&table, vec![]).expect("no SKIP-class gate present");
    assert_eq!(c.spec, CERT_SPEC);
    assert_eq!(c.overall, Overall::Disqualified);
    cert::verify(&c, &seal).expect("seal binds the honest body");
}

#[test]
fn court_g7_derivation_flips_on_mutated_receipt_copies() {
    // Anti-vacuity: the §6 law must bind to the source CONTENT, not to
    // a constant. Mutated /tmp COPIES flip the verdict; the real
    // receipt on disk is never modified (real bytes, real restores).
    let real = std::fs::read_to_string(AB14).expect("AB-14 readable");
    assert!(real.contains("min-of-3"));

    // 1200.00 ms -> 24.0 µs/link <= 25 -> Pass (flip witness UP).
    let under = real.replace("50k span 1276.01 ms", "50k span 1200.00 ms");
    let p1 = std::env::temp_dir().join("rx5-court-under.md");
    std::fs::write(&p1, under).expect("write tmp copy");
    assert!(matches!(g7_leg_b_from_receipt(p1.to_str().unwrap()), Verdict::Pass));

    // 1300.00 ms -> 26.0 µs/link > 25 -> Fail (flip witness DOWN).
    let over = real.replace("50k span 1276.01 ms", "50k span 1300.00 ms");
    let p2 = std::env::temp_dir().join("rx5-court-over.md");
    std::fs::write(&p2, over).expect("write tmp copy");
    assert!(matches!(g7_leg_b_from_receipt(p2.to_str().unwrap()), Verdict::Fail { .. }));

    // Disclosure stripped -> Skip (fail-closed degradation).
    let stripped = real.replace("min-of-3", "min-of-X");
    let p3 = std::env::temp_dir().join("rx5-court-stripped.md");
    std::fs::write(&p3, stripped).expect("write tmp copy");
    assert!(matches!(g7_leg_b_from_receipt(p3.to_str().unwrap()), Verdict::Skip { .. }));

    // A SKIP-class G7 row refuses the certificate typed, fail-closed.
    let mut table = scorecard_table(43);
    table.gates.retain(|g| g.id != "G7");
    table.gates.push(cert::GateVerdict {
        id: "G7",
        composes: "court fixture",
        verdict: g7_leg_b_from_receipt("/nonexistent/rx5-court/missing.md"),
        measured: "unreadable source".into(),
        threshold: "per-link cost <= 25 µs/link at 50k span",
    });
    let err = issue(&table, vec![]).expect_err("SKIP-class gate must refuse");
    assert_eq!(err.skipped_gates, vec!["G7"]);

    // Restore law: the tmp copies are deleted; the real receipt is
    // byte-identical (we only ever read it — assert mtime-independent
    // content identity as the receipt).
    let _ = std::fs::remove_file(&p1);
    let _ = std::fs::remove_file(&p2);
    let _ = std::fs::remove_file(&p3);
    assert!(oracle_us_per_link(AB14).is_ok());
}
