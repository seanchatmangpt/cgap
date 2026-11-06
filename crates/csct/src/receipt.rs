//! Deterministic markdown receipt rendering for CSCT v1 (lane CSCT-L2).
//!
//! W7 standard: fixed field order, byte-for-byte stable across calls.
//! No serde, no JSON, no HashMap iteration.

use crate::types::{report_digest, Report};

/// Render the SUT annual cost decomposition ([`crate::drowning_benchmark::
/// SutCostDecomposition`]) as a deterministic markdown block. Byte-for-byte
/// stable on identical input; renders each SCENARIO-labeled line item and
/// the annual total.
pub fn render_cost_decomposition(d: &crate::drowning_benchmark::SutCostDecomposition) -> String {
    let mut out = String::new();
    out.push_str("## SUT Annual Cost Decomposition\n\n");
    out.push_str("| line item | annual usd |\n");
    out.push_str("|---|---|\n");
    let rows: [(&str, f64); 5] = [
        (
            "hardware_amortization [SCENARIO=sut-node-1x, 3yr straight-line]",
            d.hardware_amortization_usd,
        ),
        (
            "power [SCENARIO=sut-power-coloc, $0.20/kWh]",
            d.power_usd,
        ),
        (
            "network [SCENARIO=sut-network-1gbps]",
            d.network_usd,
        ),
        (
            "ops_remote_hands [SCENARIO=sut-ops-remote-hands]",
            d.ops_remote_hands_usd,
        ),
        ("maintenance [SCENARIO=sut-spares-contract]", d.maintenance_usd),
    ];
    for (label, v) in rows {
        out.push_str("| ");
        out.push_str(label);
        out.push_str(" | $");
        push_usd(&mut out, v);
        out.push_str(" |\n");
    }
    out.push_str("| **annual_total** | $");
    push_usd(&mut out, d.annual_total_usd());
    out.push_str(" |\n");
    out
}

/// Render a [`Report`] as a deterministic markdown receipt string.
///
/// Byte-for-byte stable across calls on identical input: fixed field
/// order, no sorting, no map iteration. Includes the hex BLAKE3 report
/// digest from [`report_digest`].
pub fn render_report(report: &Report) -> String {
    let mut out = String::new();

    out.push_str("# CSCT Receipt\n\n");
    out.push_str("- suite_id: ");
    out.push_str(report.suite_id);
    out.push('\n');
    out.push_str("- customer: ");
    out.push_str(report.customer);
    out.push('\n');
    out.push_str("- blake3: ");
    push_hex(&mut out, &report_digest(report));
    out.push_str("\n\n");

    out.push_str("## Stages\n\n");
    out.push_str(
        "| stage | name | exit | assertions | wall_clock_nanos | alloc_count | receipt |\n",
    );
    out.push_str("|---|---|---|---|---|---|---|\n");
    for stage in &report.stages {
        out.push_str("| ");
        out.push_str(&stage.stage_index.to_string());
        out.push_str(" | ");
        out.push_str(stage.stage_name);
        out.push_str(" | ");
        out.push_str(&format!("{:?}", stage.exit));
        out.push_str(" | ");
        push_assertions(&mut out, &stage.assertions);
        out.push_str(" | ");
        out.push_str(&stage.wall_clock_nanos.to_string());
        out.push_str(" | ");
        out.push_str(&stage.alloc_count.to_string());
        out.push_str(" | ");
        push_hex(&mut out, &stage.receipt);
        out.push_str(" |\n");
    }

    out.push_str("\n## Economic Block\n\n");
    out.push_str("| field | value |\n");
    out.push_str("|---|---|\n");
    out.push_str("| annual_runrate_bleeding_usd | $");
    push_usd(&mut out, report.total_annual_runrate_bleeding_usd);
    out.push_str(" |\n");
    out.push_str("| annual_post_adoption_cost_usd | $");
    push_usd(&mut out, report.total_annual_post_adoption_cost_usd);
    out.push_str(" |\n");
    out.push_str("| net_economic_delta_usd | $");
    push_usd(&mut out, report.net_economic_delta_usd);
    out.push_str(" |\n");
    out.push_str("| roi | ");
    push_roi(&mut out, report);
    out.push_str(" |\n");
    out.push_str("| final_commercial_verdict | ");
    out.push_str(if report.final_commercial_verdict {
        "PASS"
    } else {
        "FAIL"
    });
    out.push('\n');

    out
}

/// Comma-formatted integer dollar rendering (e.g. `$5,068,632`).
fn push_usd(out: &mut String, amount: f64) {
    if amount.fract() != 0.0 {
        push_usd_cents(out, amount);
        return;
    }
    let whole = amount as u64;
    let digits = whole.to_string();
    let bytes = digits.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*b as char);
    }
}

/// Comma-formatted dollars with exact cents (e.g. `$963.60`), used when a
/// line item carries a fractional dollar amount.
fn push_usd_cents(out: &mut String, amount: f64) {
    let cents = (amount * 100.0).round() as u64;
    let whole = cents / 100;
    let frac = cents % 100;
    push_usd(out, whole as f64);
    out.push('.');
    if frac < 10 {
        out.push('0');
    }
    out.push_str(&frac.to_string());
}

/// ROI as `<whole>x` from post-adoption cost, e.g. `352x` for 5,068,632 / 14,400.
fn push_roi(out: &mut String, report: &Report) {
    if report.total_annual_post_adoption_cost_usd == 0.0 {
        out.push_str("undefined");
        return;
    }
    let roi = report.total_annual_runrate_bleeding_usd / report.total_annual_post_adoption_cost_usd;
    out.push_str(&format!("{}x", roi.round() as u64));
}

/// `id=observed/threshold unit [PASS|FAIL]` joined by `; `, sorted never.
fn push_assertions(out: &mut String, assertions: &[crate::types::StateAssertion]) {
    if assertions.is_empty() {
        out.push_str("(none)");
        return;
    }
    for (i, a) in assertions.iter().enumerate() {
        if i > 0 {
            out.push_str("; ");
        }
        out.push_str(a.metric_id);
        out.push('=');
        push_f64(out, a.observed);
        out.push('/');
        push_f64(out, a.threshold);
        out.push(' ');
        out.push_str(a.unit);
        out.push('[');
        out.push_str(if a.pass { "PASS" } else { "FAIL" });
        out.push(']');
    }
}

/// Full f64 rendering without format-allocation churn.
fn push_f64(out: &mut String, v: f64) {
    if v == v.trunc() && v.abs() < 1e15 {
        out.push_str(&(v as i64).to_string());
    } else {
        out.push_str(&format!("{:?}", v));
    }
}

/// Lowercase hex, fixed order.
fn push_hex(out: &mut String, bytes: &[u8]) {
    for b in bytes {
        out.push_str(&format!("{:02x}", b));
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{report_digest, ExitCode, Report, StageResult, StateAssertion};

    fn sample_report() -> Report {
        Report {
            suite_id: "csct.v1.sample",
            customer: "acme",
            stages: vec![StageResult {
                stage_index: 0,
                stage_name: "ingress",
                exit: ExitCode::Pass,
                assertions: vec![StateAssertion {
                    metric_id: "rows_per_sec",
                    observed: 1_000_000.0,
                    threshold: 900_000.0,
                    unit: "rows/s",
                    pass: true,
                }],
                wall_clock_nanos: 42,
                alloc_count: 7,
                receipt: [0xAB; 32],
            }],
            total_annual_runrate_bleeding_usd: 5_068_632.0,
            total_annual_post_adoption_cost_usd: 14_400.0,
            net_economic_delta_usd: 5_054_232.0,
            final_commercial_verdict: true,
        }
    }

    #[test]
    fn render_is_byte_stable_across_calls() {
        let a = crate::receipt::render_report(&sample_report());
        let b = crate::receipt::render_report(&sample_report());
        assert_eq!(a, b, "receipt must be byte-for-byte stable across calls");
        assert!(a.contains("blake3: "), "receipt must carry hex digest");
        assert!(a.contains("$5,068,632"), "economic block must render bleed");
        assert!(
            a.contains("$14,400"),
            "economic block must render post-adoption"
        );
        assert!(a.contains("352x"), "economic block must render ROI");
        assert!(a.contains("PASS"), "verdict line must render");
    }

    #[test]
    fn render_mutates_on_exit_code_change() {
        // Anti-vacuity mutation: GateFail must change the rendered bytes.
        let mut mutated = sample_report();
        mutated.stages[0].exit = ExitCode::GateFail;
        assert_ne!(
            crate::receipt::render_report(&sample_report()),
            crate::receipt::render_report(&mutated),
        );
    }

    #[test]
    fn render_mutates_on_assertion_flip() {
        let mut mutated = sample_report();
        mutated.stages[0].assertions[0].pass = false;
        assert_ne!(
            crate::receipt::render_report(&sample_report()),
            crate::receipt::render_report(&mutated),
        );
    }

    #[test]
    fn digest_and_render_agree_on_mutation() {
        // The digest and the receipt must both move on the same mutation.
        let mut mutated = sample_report();
        mutated.total_annual_runrate_bleeding_usd = 1.0;
        assert_ne!(report_digest(&sample_report()), report_digest(&mutated));
        assert_ne!(
            crate::receipt::render_report(&sample_report()),
            crate::receipt::render_report(&mutated),
        );
    }
}
