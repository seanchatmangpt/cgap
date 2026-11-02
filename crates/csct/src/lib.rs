//! CSCT — Challenger Sales Chicago Test (RFC-0042).
//!
//! 6-stage pipeline over real collaborators; exit codes 0/1/2/3/101.
//! Stage bodies are owned by lanes L3..L8; mutation controls by L9;
//! andon refusal by L10. This module owns sequencing + economics.

pub mod andon;
pub mod drowning;
pub mod drowning_benchmark;
pub mod emotional;
pub mod mutation;
pub mod new_way;
pub mod receipt;
pub mod reframe;
pub mod solution;
pub mod types;
pub mod warmer;

// Integration note: `pub mod court` was dropped — the court surface is
// tests/csct_pipeline_court.rs (lane L11); no src/court.rs lane ever owned it.

use types::{ExitCode, Report, StageResult};

/// Status quo bleeding: $422,386/month annualized.
pub const STATUS_QUO_ANNUAL_USD: f64 = 5_068_632.0;

/// SUT ceiling: $1,200/month annualized.
pub const SUT_ANNUAL_USD: f64 = 14_400.0;

/// Required ROI multiple per RFC-0042. Integration ruling (coordinator,
/// 2026-11-02): the RFC's "352x" holds only as the GROSS ratio
/// STATUS_QUO/SUT = 5,068,632/14,400 = 351.99; the net/gross form gives
/// 351.15. Pinned at 350.0 (gross formula) so the verdict court is
/// non-vacuous at the RFC economics. Errata: csct-lane-01-scaffold.md.
pub const REQUIRED_ROI_X: f64 = 350.0;

/// Execute the 6 stages IN ORDER. On the first failing stage, return
/// early with a Report carrying that stage's exit code (sequencing is
/// asserted by `court`). On full pass, compute economic deltas and
/// stamp `final_commercial_verdict`.
pub fn run_suite() -> Report {
    let mut stages: Vec<StageResult> = Vec::with_capacity(6);

    let seq: [(&str, fn() -> StageResult); 6] = [
        ("warmer", warmer::warmer),
        ("reframe", reframe::reframe),
        ("drowning", drowning::drowning),
        ("emotional_impact", emotional::emotional_impact),
        ("new_way", new_way::new_way),
        ("solution", solution::solution),
    ];

    for (_i, (_name, f)) in seq.iter().enumerate() {
        let result = f();
        let exit = result.exit;
        stages.push(result);
        // Sequencing contract: only hard exits abort the suite. LoudSkip
        // (exit 3) is a typed non-failure in the gate contract — the stage
        // ran, one assertion skipped loudly; aborting on it would make the
        // suite vacuous in debug builds. GateFail/SubjectMissing/AndonRefusal
        // abort with the failing stage's exit.
        if matches!(
            exit,
            ExitCode::GateFail | ExitCode::SubjectMissing | ExitCode::AndonRefusal
        ) {
            return Report {
                suite_id: "csct-rfc0042",
                customer: "challenger-sales-rfc0042",
                stages,
                total_annual_runrate_bleeding_usd: STATUS_QUO_ANNUAL_USD,
                total_annual_post_adoption_cost_usd: 0.0,
                net_economic_delta_usd: 0.0,
                final_commercial_verdict: false,
            };
        }
    }

    let net = STATUS_QUO_ANNUAL_USD - SUT_ANNUAL_USD;
    // Gross ROI per the integration ruling on REQUIRED_ROI_X above.
    let roi = STATUS_QUO_ANNUAL_USD / SUT_ANNUAL_USD;
    let verdict = roi >= REQUIRED_ROI_X;

    Report {
        suite_id: "csct-rfc0042",
        customer: "challenger-sales-rfc0042",
        stages,
        total_annual_runrate_bleeding_usd: STATUS_QUO_ANNUAL_USD,
        total_annual_post_adoption_cost_usd: SUT_ANNUAL_USD,
        net_economic_delta_usd: net,
        final_commercial_verdict: verdict,
    }
}
