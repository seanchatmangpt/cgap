//! CSCT cold-path coverage court (lane C02, v26.11.2).
//!
//! Targets demonstrated-unexercised branches, cited at 2026-11-02:
//!   - receipt.rs:91-93  push_roi "undefined" branch (zero post-adoption cost)
//!   - receipt.rs:101-103 push_assertions "(none)" branch (empty assertions)
//!   - receipt.rs:123-129 push_f64 fractional `{:?}` branch
//!   - receipt.rs:38     Debug render of SubjectMissing / LoudSkip / AndonRefusal
//!   - receipt.rs:69-70  verdict "FAIL" render branch
//!   - types.rs:63-117   report_digest over multi-stage reports, per-variant
//!                       exit encoding, stage_index / stage_name sensitivity
//!
//! Every court carries an anti-vacuity twin: the input state is mutated
//! and the court asserts the observed outcome trips. All coverage flows
//! through the PUBLIC API (no src edits needed -- see lane receipt).

use csct::receipt::render_report;
use csct::types::{report_digest, ExitCode, Report, StageResult, StateAssertion};

/// ALL exit variants in wire order; the court iterates this exhaustively.
const ALL_EXITS: [ExitCode; 5] = [
    ExitCode::Pass,
    ExitCode::GateFail,
    ExitCode::SubjectMissing,
    ExitCode::LoudSkip,
    ExitCode::AndonRefusal,
];

fn stage(
    idx: u8,
    name: &'static str,
    exit: ExitCode,
    assertions: Vec<StateAssertion>,
) -> StageResult {
    StageResult {
        stage_index: idx,
        stage_name: name,
        exit,
        assertions,
        wall_clock_nanos: 11,
        alloc_count: 3,
        receipt: [0x5A; 32],
    }
}

fn assertion(metric_id: &'static str, observed: f64, threshold: f64, pass: bool) -> StateAssertion {
    StateAssertion {
        metric_id,
        observed,
        threshold,
        unit: "pct",
        pass,
    }
}

/// A two-stage report whose second stage FAILS (GateFail) with a
/// fractional observed value, and a FAIL commercial verdict.
fn failing_report() -> Report {
    Report {
        suite_id: "csct.v1.coldpath",
        customer: "cold-path-court",
        stages: vec![
            stage(
                1,
                "warmer",
                ExitCode::Pass,
                vec![assertion("warm_intake_ratio", 0.5, 0.4, true)],
            ),
            stage(
                2,
                "reframe",
                ExitCode::GateFail,
                vec![assertion("reframe_delta_pct", 1.5, 2.0, false)],
            ),
        ],
        total_annual_runrate_bleeding_usd: 5_068_632.0,
        total_annual_post_adoption_cost_usd: 0.0,
        net_economic_delta_usd: 0.0,
        final_commercial_verdict: false,
    }
}

fn three_stage_report() -> Report {
    Report {
        suite_id: "csct.v1.coldpath",
        customer: "cold-path-court",
        stages: vec![
            stage(
                1,
                "warmer",
                ExitCode::Pass,
                vec![assertion("m1", 0.5, 0.4, true)],
            ),
            stage(2, "reframe", ExitCode::LoudSkip, vec![]),
            stage(
                3,
                "solution",
                ExitCode::Pass,
                vec![assertion("m3", 2.25, 2.0, true)],
            ),
        ],
        total_annual_runrate_bleeding_usd: 5_068_632.0,
        total_annual_post_adoption_cost_usd: 14_400.0,
        net_economic_delta_usd: 5_054_232.0,
        final_commercial_verdict: true,
    }
}

/// COURT 1: every ExitCode variant through as_u32 AND the rendered
/// receipt bytes. Pinned wire values, pairwise-distinct integers, and
/// the Debug string for each variant appears in render_report output.
#[test]
fn every_exit_variant_encodes_and_renders() {
    let wire = [0u32, 1, 2, 3, 101];
    for (v, e) in ALL_EXITS.iter().zip(wire.iter()) {
        assert_eq!(v.as_u32(), *e, "wire pin for {:?}", v);
    }
    for e in ALL_EXITS.iter() {
        let single = Report {
            suite_id: "x",
            customer: "y",
            stages: vec![stage(1, "s", *e, vec![assertion("m", 1.0, 1.0, true)])],
            total_annual_runrate_bleeding_usd: 1.0,
            total_annual_post_adoption_cost_usd: 1.0,
            net_economic_delta_usd: 0.0,
            final_commercial_verdict: true,
        };
        let rendered = render_report(&single);
        let debug_str = format!("{:?}", e);
        assert!(
            rendered.contains(&debug_str),
            "receipt must render Debug form {:?} for {:?}",
            debug_str,
            e
        );
    }
}

/// COURT 1 TWIN (anti-vacuity): swapping any variant for any OTHER
/// variant must move both the wire integer and the rendered bytes. If
/// any pair collides, this court trips.
#[test]
fn exit_variant_matrix_flips_under_swap_mutation() {
    for a in ALL_EXITS.iter() {
        for b in ALL_EXITS.iter() {
            if a == b {
                assert_eq!(a.as_u32(), b.as_u32());
                continue;
            }
            assert_ne!(
                a.as_u32(),
                b.as_u32(),
                "wire collision between {:?} and {:?}",
                a,
                b
            );
            let mk = |e: ExitCode| {
                render_report(&Report {
                    suite_id: "x",
                    customer: "y",
                    stages: vec![stage(1, "s", e, vec![])],
                    total_annual_runrate_bleeding_usd: 1.0,
                    total_annual_post_adoption_cost_usd: 1.0,
                    net_economic_delta_usd: 0.0,
                    final_commercial_verdict: true,
                })
            };
            assert_ne!(
                mk(*a),
                mk(*b),
                "rendered bytes collision between {:?} and {:?}",
                a,
                b
            );
        }
    }
}

/// COURT 2: a report with a FAILING stage renders the failure block --
/// the failing stage's Debug exit, its FAIL assertion with the
/// fractional observed value, and the FAIL verdict line.
#[test]
fn failing_stage_renders_failure_block() {
    let report = failing_report();
    let rendered = render_report(&report);
    assert!(
        rendered.contains("| 2 | reframe | GateFail | reframe_delta_pct=1.5/2 pct[FAIL] |"),
        "failing stage row must render: {}",
        rendered
    );
    assert!(
        rendered.contains("reframe_delta_pct=1.5/2 pct[FAIL]"),
        "failing assertion must render FAIL marker (fractional f64 branch): {}",
        rendered
    );
    assert!(
        rendered.contains("| final_commercial_verdict | FAIL\n"),
        "FAIL verdict branch must render: {}",
        rendered
    );
}

/// COURT 2 TWIN: flipping the failing stage to Pass (and its assertion
/// to a passing integer-valued one) must REMOVE the failure markers and
/// move the digest. If the render ignored the mutation, this trips.
#[test]
fn failing_stage_render_flips_under_pass_mutation() {
    let mut healed = failing_report();
    healed.stages[1].exit = ExitCode::Pass;
    healed.stages[1].assertions[0] = assertion("reframe_delta_pct", 3.0, 2.0, true);
    healed.final_commercial_verdict = true;
    let base = failing_report();
    let healed_s = render_report(&healed);
    assert!(
        !healed_s.contains("GateFail"),
        "GateFail must vanish after heal"
    );
    assert!(
        healed_s.contains("reframe_delta_pct=3/2 pct[PASS]"),
        "healed assertion must render integer PASS form: {}",
        healed_s
    );
    assert!(healed_s.contains("| final_commercial_verdict | PASS\n"));
    assert_ne!(
        report_digest(&base),
        report_digest(&healed),
        "digest must move under heal mutation"
    );
}

/// COURT 3: an empty-assertions stage renders the "(none)" marker
/// (receipt.rs:101-103), and the zero post-adoption cost renders ROI
/// "undefined" (receipt.rs:91-93).
#[test]
fn empty_assertions_and_zero_cost_cold_branches_render() {
    let report = failing_report(); // cost = 0.0, stage 2 has one assertion
    let mut with_empty = report.clone();
    with_empty
        .stages
        .push(stage(3, "drowning", ExitCode::LoudSkip, vec![]));
    let rendered = render_report(&with_empty);
    assert!(
        rendered.contains("| 3 | drowning | LoudSkip | (none) |"),
        "(none) branch must render for empty assertions: {}",
        rendered
    );
    assert!(
        rendered.contains("| roi | undefined |"),
        "zero-cost ROI must render undefined: {}",
        rendered
    );
}

/// COURT 3 TWIN: giving the empty stage one assertion and a nonzero
/// cost must erase BOTH cold-branch markers and move digest + bytes.
#[test]
fn empty_assertions_and_zero_cost_flip_under_mutation() {
    let mut with_empty = failing_report();
    with_empty
        .stages
        .push(stage(3, "drowning", ExitCode::LoudSkip, vec![]));
    let mut populated = with_empty.clone();
    populated.stages[2]
        .assertions
        .push(assertion("drown_depth", 4.25, 4.0, true));
    populated.total_annual_post_adoption_cost_usd = 14_400.0;
    let base_s = render_report(&with_empty);
    let pop_s = render_report(&populated);
    assert!(
        !pop_s.contains("(none)"),
        "(none) must vanish when populated"
    );
    assert!(
        !pop_s.contains("undefined"),
        "undefined ROI must vanish when cost set"
    );
    assert!(pop_s.contains("drown_depth=4.25/4 pct[PASS]"));
    assert!(pop_s.contains("352x"));
    assert_ne!(base_s, pop_s, "rendered bytes must move");
    assert_ne!(report_digest(&with_empty), report_digest(&populated));
}

/// COURT 4: multi-stage digest (types.rs:63-117) is deterministic and
/// sensitive to EVERY per-stage field: stage_index, stage_name, exit,
/// assertion list, wall clock, allocs, receipt bytes.
#[test]
fn digest_multi_stage_deterministic_and_field_sensitive() {
    let base = three_stage_report();
    let a = report_digest(&base);
    assert_eq!(a, report_digest(&base), "digest deterministic across calls");
    let mut m;
    m = base.clone();
    m.stages[1].stage_index = 9;
    assert_ne!(a, report_digest(&m), "digest must move on stage_index flip");
    m = base.clone();
    m.stages[1].stage_name = "reframed";
    assert_ne!(a, report_digest(&m), "digest must move on stage_name flip");
    m = base.clone();
    m.stages[1].exit = ExitCode::GateFail;
    assert_ne!(
        a,
        report_digest(&m),
        "digest must move on exit flip (LoudSkip -> GateFail)"
    );
    m = base.clone();
    m.stages[1].assertions.push(assertion("mx", 1.0, 1.0, true));
    assert_ne!(a, report_digest(&m), "digest must move on assertion append");
    m = base.clone();
    m.stages[1].wall_clock_nanos = 12;
    assert_ne!(a, report_digest(&m), "digest must move on wall_clock flip");
    m = base.clone();
    m.stages[1].alloc_count = 4;
    assert_ne!(a, report_digest(&m), "digest must move on alloc_count flip");
    m = base.clone();
    m.stages[1].receipt = [0x5B; 32];
    assert_ne!(
        a,
        report_digest(&m),
        "digest must move on receipt-byte flip"
    );
}

/// COURT 4 TWIN: the digest must also move on STRUCTURAL mutation --
/// removing a middle stage or reordering stages (order-sensitive, never
/// sorted per types.rs:91-92).
#[test]
fn digest_flips_on_structural_mutation() {
    let base = three_stage_report();
    let a = report_digest(&base);
    let mut removed = base.clone();
    removed.stages.remove(1);
    assert_ne!(
        a,
        report_digest(&removed),
        "digest must move on stage removal"
    );
    let mut reordered = base.clone();
    reordered.stages.swap(0, 2);
    assert_ne!(a, report_digest(&reordered), "digest must move on reorder");
}

/// COURT 5: the receipt over the multi-stage LoudSkip report is
/// byte-stable AND carries the LoudSkip Debug form for stage 2.
#[test]
fn multi_stage_receipt_byte_stable_with_loudskip() {
    let base = three_stage_report();
    let a = render_report(&base);
    let b = render_report(&base);
    assert_eq!(a, b, "byte stability on multi-stage report");
    assert!(
        a.contains("| 2 | reframe | LoudSkip | (none) | 11 | 3 |"),
        "LoudSkip empty-assertion row must render: {}",
        a
    );
    assert!(a.contains("352x"), "ROI renders on nonzero cost");
}
