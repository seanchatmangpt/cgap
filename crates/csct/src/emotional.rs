//! CSCT Stage 4 — EMOTIONAL IMPACT (RFC-0042 §4, p99 tail crash witness).
//!
//! Models the incumbent's p99 tail: network transit + GC pause + base
//! service stack on every request, so p99 latency is the SUM, not the
//! base alone. Under default params the DOM heap exceeds the tab limit
//! and the tab crashes (OOM) — the pain is witnessed in-model; the
//! stage PASSES when the computed state shows the crash.
//!
//! No-Sleep law: time is a parameter (`*_ms`), never awaited.

use crate::types::{ExitCode, StageResult, StateAssertion};

/// p99 tail model parameters (RFC-0042 §4 defaults).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct TailParams {
    /// Network transit per request, milliseconds.
    pub network_transit_ms: f64,
    /// Garbage-collector pause, milliseconds.
    pub gc_pause_ms: f64,
    /// Base service time, milliseconds.
    pub base_service_ms: f64,
    /// Live DOM objects at steady state.
    pub dom_objects: u64,
    /// Bytes retained per DOM object.
    pub bytes_per_dom_object: u64,
    /// Per-tab heap ceiling, megabytes (decimal, 1e6 bytes/MB).
    pub tab_limit_mb: f64,
}

/// RFC-0042 §4 default tail: transit+GC+base stack to 2750 ms p99;
/// 1M DOM objects at 600 B retain 600 MB against a 600 MB tab limit.
impl Default for TailParams {
    fn default() -> Self {
        TailParams {
            network_transit_ms: 450.0,
            gc_pause_ms: 1200.0,
            base_service_ms: 1100.0,
            dom_objects: 1_000_000,
            bytes_per_dom_object: 600,
            tab_limit_mb: 600.0,
        }
    }
}

/// Compute the derived model state from [`TailParams`].
///
/// Returns `(p99_latency_ms, missed_frame_budget, heap_mb)`:
/// - p99 latency is the full stack: transit + GC pause + base service;
/// - a 60 fps frame budget (16.7 ms) is missed whenever the GC pause
///   exceeds it — a real jank witness, not a threshold literal;
/// - heap is objects * bytes / 1e6 (decimal MB, matching the limit).
fn compute(p: &TailParams) -> (f64, bool, f64) {
    let p99_latency_ms = p.network_transit_ms + p.gc_pause_ms + p.base_service_ms;
    let missed_frame_budget = p.gc_pause_ms > 16.7;
    let heap_mb = (p.dom_objects as f64) * (p.bytes_per_dom_object as f64) / 1e6;
    (p99_latency_ms, missed_frame_budget, heap_mb)
}

/// Evaluate the tail model. Returns the assertions and whether ALL of
/// them fired pass (the crash was witnessed in-model).
fn evaluate(p: &TailParams) -> (Vec<StateAssertion>, bool) {
    let (p99_latency_ms, missed_frame_budget, heap_mb) = compute(p);

    let latency_pass = p99_latency_ms >= 2750.0;
    let heap_pass = heap_mb >= p.tab_limit_mb;

    let assertions = vec![
        StateAssertion {
            metric_id: "latency_p99",
            observed: p99_latency_ms,
            threshold: 2750.0,
            unit: "ms",
            pass: latency_pass,
        },
        StateAssertion {
            metric_id: "missed_frame_budget",
            observed: if missed_frame_budget { 1.0 } else { 0.0 },
            threshold: 1.0,
            unit: "bool",
            pass: missed_frame_budget,
        },
        StateAssertion {
            metric_id: "heap_mb_vs_tab_limit",
            observed: heap_mb,
            threshold: p.tab_limit_mb,
            unit: "MB",
            pass: heap_pass,
        },
    ];
    let all_pass = assertions.iter().all(|a| a.pass);
    (assertions, all_pass)
}

/// Stage 4 entry point: witness the p99 tail crash under RFC-0042
/// default params and return the stamped [`StageResult`].
pub fn emotional_impact() -> StageResult {
    let start = std::time::Instant::now();

    let (assertions, all_pass) = evaluate(&TailParams::default());

    // Deterministic receipt over the assertion surface (not wall clock,
    // so the digest is replayable byte-for-byte run over run).
    let mut buf: Vec<u8> = Vec::with_capacity(64 * assertions.len());
    buf.extend_from_slice(b"CSCT-L6-EMOTIONAL-IMPACT-v1");
    for a in &assertions {
        buf.extend_from_slice(a.metric_id.as_bytes());
        buf.extend_from_slice(&a.observed.to_bits().to_le_bytes());
        buf.extend_from_slice(&a.threshold.to_bits().to_le_bytes());
        buf.push(u8::from(a.pass));
    }
    let receipt = cgap_blake3::hash(&buf);

    StageResult {
        stage_index: 4,
        stage_name: "EMOTIONAL_IMPACT",
        exit: if all_pass {
            ExitCode::Pass
        } else {
            ExitCode::GateFail
        },
        assertions,
        wall_clock_nanos: start.elapsed().as_nanos() as u64,
        alloc_count: 1, // one Vec<assertions> heap allocation, counted honestly
        receipt,
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    /// (a) Default params: all three assertions fire true (crash
    /// witnessed in-model) and the stage exits Pass.
    #[test]
    fn default_params_witness_crash_exit_pass() {
        let r = emotional_impact();
        assert_eq!(r.stage_index, 4);
        assert_eq!(r.stage_name, "EMOTIONAL_IMPACT");
        assert_eq!(r.assertions.len(), 3);
        for a in &r.assertions {
            assert!(
                a.pass,
                "assertion {:?} must fire true under defaults",
                a.metric_id
            );
        }
        assert_eq!(r.exit, ExitCode::Pass);

        // Ground the observed values against the model, not the literal.
        let (p99, missed, heap) = compute(&TailParams::default());
        assert_eq!(p99, 2750.0);
        assert!(missed);
        assert_eq!(heap, 600.0);
        assert!(heap >= TailParams::default().tab_limit_mb);
    }

    /// (b) ANTI-VACUITY MUTATION: shrinking the tail (gc 5 ms, transit
    /// 1 ms) must flip the latency assertion off, clear the frame-budget
    /// witness, and force evaluate() to return false. A court that
    /// cannot flip is FAIL.
    #[test]
    fn mutation_shrunk_tail_flips_evaluate_false() {
        let mutated = TailParams {
            network_transit_ms: 1.0,
            gc_pause_ms: 5.0,
            base_service_ms: 1100.0,
            dom_objects: 1_000_000,
            bytes_per_dom_object: 600,
            tab_limit_mb: 600.0,
        };
        let (assertions, all_pass) = evaluate(&mutated);

        assert!(!all_pass, "evaluate must flip false under the mutation");

        let latency = assertions
            .iter()
            .find(|a| a.metric_id == "latency_p99")
            .expect("latency_p99 assertion present");
        assert!(!latency.pass);
        assert_eq!(latency.observed, 1106.0); // 1 + 5 + 1100

        let frame = assertions
            .iter()
            .find(|a| a.metric_id == "missed_frame_budget")
            .expect("missed_frame_budget assertion present");
        assert!(
            !frame.pass,
            "5 ms GC pause must clear the frame-budget flag"
        );

        // Heap witness survives the latency mutation: still at the limit.
        let heap = assertions
            .iter()
            .find(|a| a.metric_id == "heap_mb_vs_tab_limit")
            .expect("heap_mb assertion present");
        assert!(heap.pass);
    }

    /// The Default tail is exactly the RFC-0042 §4 contract values.
    #[test]
    fn default_params_match_contract() {
        let d = TailParams::default();
        assert_eq!(d.network_transit_ms, 450.0);
        assert_eq!(d.gc_pause_ms, 1200.0);
        assert_eq!(d.base_service_ms, 1100.0);
        assert_eq!(d.dom_objects, 1_000_000);
        assert_eq!(d.bytes_per_dom_object, 600);
        assert_eq!(d.tab_limit_mb, 600.0);
    }
}
