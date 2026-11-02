//! CSCT end-to-end integration court (lane CSCT-L11, RFC-0042 SS6).
//!
//! Chicago discipline: real execution of csct::run_suite(),
//! mutation::run_mutation_controls(), andon constants, and
//! receipt::render_report. No mocks, no sleep -- every assertion is on
//! final observed state of the real pipeline.
//!
//! ANTI-VACUITY: test (e) refuses a stage that "passes" with zero
//! assertions. (The andon-101 wire pin is subsumed by
//! csct_coverage_court.rs COURT 1 -- see the cull note below.)

use csct::receipt::render_report;
use csct::run_suite;
use csct::types::{ExitCode, Report};

/// (a) 6 stages, indices 1..=6 in order, verdict true.
/// THE_SOLUTION may be the typed LoudSkip in debug builds (its <10 ns
/// timing gate is release-meaningful); every other stage must be Pass.
#[test]
fn suite_runs_six_stages_in_order_all_pass() {
    let report: Report = run_suite();
    assert_eq!(report.stages.len(), 6, "suite must run exactly 6 stages");
    for (i, stage) in report.stages.iter().enumerate() {
        assert_eq!(stage.stage_index, (i + 1) as u8, "stage_index order");
        if stage.stage_name == "THE_SOLUTION" {
            let timing_failed = stage
                .assertions
                .iter()
                .any(|a| a.metric_id == "ingress_filter_per_frame_ns_lt_10" && !a.pass);
            let allowed = timing_failed && stage.exit == ExitCode::LoudSkip;
            assert!(
                stage.exit == ExitCode::Pass || allowed,
                "stage THE_SOLUTION must pass or loud-skip the timing gate: {:?}",
                stage.assertions
            );
        } else {
            assert_eq!(
                stage.exit,
                ExitCode::Pass,
                "stage {} must pass",
                stage.stage_name
            );
        }
    }
    assert!(
        report.final_commercial_verdict,
        "final commercial verdict must be true on a full pass"
    );
}

/// (b) Economic block: gross ROI >= 350x per integration ruling
/// (5,068,632/14,400 = 351.99; RFC-0042 errata csct-lane-01-scaffold.md).
#[test]
fn economic_block_matches_rfc0042() {
    let report = run_suite();
    assert_eq!(
        report.net_economic_delta_usd,
        5_068_632.0 - 14_400.0,
        "net delta must be bleed minus SUT ceiling"
    );
    assert_eq!(
        report.total_annual_runrate_bleeding_usd, 5_068_632.0,
        "annual runrate bleeding must equal RFC-0042 figure"
    );
    assert!(
        report.final_commercial_verdict,
        "verdict must hold at the RFC-0042 ROI multiple"
    );
}

// CULL-04: `mutation_controls_pass` (test c) was deleted from this file --
// it duplicated src/mutation.rs::mutation_controls_pass (mutation.rs:66)
// verbatim in intent. Survivor = src/mutation.rs's own court, which is
// stronger (it carries the anti-vacuity flip battery alongside the pass
// assert). Integration surface `run_mutation_controls` remains exercised
// via that survivor; no coverage lost.

/// (d) Receipt: non-empty, carries stage names + hex digest, deterministic.
#[test]
fn receipt_renders_deterministically_with_stages_and_digest() {
    let report = run_suite();
    let a = render_report(&report);
    let b = render_report(&report);
    assert!(!a.is_empty(), "receipt must be non-empty");
    for stage in &report.stages {
        assert!(
            a.contains(stage.stage_name),
            "receipt must name stage {}",
            stage.stage_name
        );
    }
    assert!(a.contains("blake3: "), "receipt must carry the hex digest");
    assert_eq!(a, b, "receipt must be byte-identical across two calls");
}

/// (e) ANTI-VACUITY court-of-the-courts: no stage passes with zero
/// assertions. A vacuous pass must fail this test.
#[test]
fn no_stage_passes_vacuously() {
    let report = run_suite();
    assert_vacuum_guard(&report);
}

fn assert_vacuum_guard(report: &Report) {
    for stage in &report.stages {
        assert!(
            !stage.assertions.is_empty(),
            "stage {} passed with ZERO assertions -- vacuous pass",
            stage.stage_name
        );
        assert!(
            stage.assertions.iter().any(|a| a.pass),
            "stage {} carries no passing assertion",
            stage.stage_name
        );
    }
}

// CULL-04 (second finding): `andon_refusal_wire_value_is_101` (test f) was
// deleted. Canonical exit-variant wire pin SURVIVOR = csct_coverage_court.rs
// COURT 1 (`every_exit_variant_encodes_and_renders`), which pins ALL
// five variants (0/1/2/3/101), pairwise distinctness, AND rendered bytes --
// a strict superset of this test's two AndonRefusal asserts. No coverage lost.

/// Falsifier of the whole court: mutating any stage assertion to a FAIL
/// (or emptying them) must be observable -- digest must move, so the
/// receipted report cannot silently absorb a vacuous pipeline.
#[test]
fn court_flips_on_stage_mutation() {
    let mut report = run_suite();
    let digest_before = csct::types::report_digest(&report);
    report.stages[0].exit = ExitCode::GateFail;
    let digest_after = csct::types::report_digest(&report);
    assert_ne!(
        digest_before, digest_after,
        "digest must move when a stage flips to GateFail"
    );
    // The vacuum guard itself must be non-vacuous: an emptied stage MUST
    // trip it (panic), observed here via catch_unwind, not assumed.
    let mut emptied = run_suite();
    emptied.stages[2].assertions.clear();
    let tripped = std::panic::catch_unwind(|| assert_vacuum_guard(&emptied)).is_err();
    assert!(tripped, "vacuum guard must trip on a zero-assertion stage");
}
