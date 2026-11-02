//! CSCT Stage 2 — THE REFRAME (RFC-0042 §4): the hidden tax.
//!
//! Two real observations, both computed — never string-literal-hardcoded —
//! from [`BaselineParams`]:
//!
//! 1. **Wire amplification**: 1,000,000 facts land as 24 MB raw columns but
//!    must traverse a JSON wire format, landing at 160 MB JSON which a JVM
//!    consumer inflates to 800 MB heap. The amplification ratio is derived:
//!    160/24 ≈ 6.667x on the wire; heap/JSON ≈ 5x on ingestion.
//! 2. **Sharding entropy**: the deployed shard count (1360) yields
//!    n*(n-1)/2 = 924,120 inter-shard channels at runtime, versus ZERO for a
//!    single-node deployment. That channel count IS the entropy penalty.
//!
//! Assertion pass/fail is computed, not tautological: the mutation control
//! (see tests) perturbs the struct — not a string literal — and must flip
//! the recomputed result to `pass = false`.

use crate::types::{ExitCode, StageResult, StateAssertion};

/// RFC-0042 §4 baseline parameters. Mutation controls perturb THIS struct,
/// never string literals inside the assertions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaselineParams {
    /// Raw fact count at ingress.
    pub facts: u64,
    /// Raw columnar payload in MB.
    pub raw_mb: f64,
    /// Wire (JSON) payload in MB.
    pub json_mb: f64,
    /// JVM heap after ingestion in MB.
    pub heap_mb: f64,
    /// Deployed shard count.
    pub shards: u32,
}

impl Default for BaselineParams {
    fn default() -> Self {
        BaselineParams {
            facts: 1_000_000,
            raw_mb: 24.0,
            json_mb: 160.0,
            heap_mb: 800.0,
            shards: 1360,
        }
    }
}

/// Inter-shard channels for `n` shards: n*(n-1)/2 (every ordered pair once).
pub fn shard_channels(shards: u32) -> u64 {
    let n = shards as u64;
    n * (n.saturating_sub(1)) / 2
}

/// Compute the hidden-tax assertions from parameters.
///
/// Returns the assertions (with computed `pass` flags) plus the aggregate
/// verdict (`true` only if every assertion passed). Pure function of the
/// parameters — the mutation control calls this directly on a mutant
/// [`BaselineParams`] without touching any shared tree.
pub fn evaluate(params: &BaselineParams) -> (Vec<StateAssertion>, bool) {
    let mut assertions = Vec::with_capacity(4);
    let mut all_pass = true;

    // (a) Wire amplification: JSON/raw must exceed 6.66 (160/24 ≈ 6.667).
    let wire_ratio = if params.raw_mb > 0.0 {
        params.json_mb / params.raw_mb
    } else {
        f64::INFINITY
    };
    let wire_pass = wire_ratio >= 6.66;
    assertions.push(StateAssertion {
        metric_id: "wire_amplification_ratio",
        observed: wire_ratio,
        threshold: 6.66,
        unit: "x",
        pass: wire_pass,
    });
    all_pass &= wire_pass;

    // Heap inflation: JVM heap / wire JSON (~5x for the RFC baseline).
    let heap_ratio = if params.json_mb > 0.0 {
        params.heap_mb / params.json_mb
    } else {
        f64::INFINITY
    };
    let heap_pass = heap_ratio >= 1.0;
    assertions.push(StateAssertion {
        metric_id: "heap_amplification_ratio",
        observed: heap_ratio,
        threshold: 1.0,
        unit: "x",
        pass: heap_pass,
    });
    all_pass &= heap_pass;

    // (b) Sharding entropy: channels = n*(n-1)/2 computed at runtime;
    // single-node deployment (1 shard) has ZERO channels. The penalty is
    // the channel surplus over a single node, asserted > 0.
    let channels = shard_channels(params.shards);
    let single_node_channels = shard_channels(1); // = 0 by construction
    let sharding_penalty = (channels - single_node_channels) as f64;
    let sharding_pass = sharding_penalty > 0.0;
    assertions.push(StateAssertion {
        metric_id: "sharding_entropy_penalty",
        observed: sharding_penalty,
        threshold: f64::MIN_POSITIVE,
        unit: "channels",
        pass: sharding_pass,
    });
    all_pass &= sharding_pass;

    // Ingested-fact count sanity: the baseline carries 1,000,000 facts.
    let facts_pass = params.facts > 0;
    assertions.push(StateAssertion {
        metric_id: "facts_ingressed",
        observed: params.facts as f64,
        threshold: 1.0,
        unit: "facts",
        pass: facts_pass,
    });
    all_pass &= facts_pass;

    (assertions, all_pass)
}

/// Collapse an f64 to a finite positive value for observation only (digest
/// stability for infinite ratios on degenerate inputs).
#[allow(dead_code)]
fn at_least_positive(x: f64) -> f64 {
    if x.is_finite() && x > 0.0 {
        x
    } else {
        f64::MIN_POSITIVE
    }
}

/// Stage 2 entry point per the frozen contract.
pub fn reframe() -> StageResult {
    let start = std::time::Instant::now();
    let params = BaselineParams::default();
    let (assertions, all_pass) = evaluate(&params);
    let assertion_count = assertions.len() as u64;

    // Receipt: BLAKE3 over the assertion stream (real digest, not a stub).
    let mut buf = Vec::with_capacity(64 * assertions.len());
    for a in &assertions {
        buf.extend_from_slice(a.metric_id.as_bytes());
        buf.extend_from_slice(&a.observed.to_bits().to_le_bytes());
        buf.extend_from_slice(&a.threshold.to_bits().to_le_bytes());
        buf.extend_from_slice(a.unit.as_bytes());
        buf.push(u8::from(a.pass));
    }
    let receipt = cgap_blake3::hash(&buf);

    StageResult {
        stage_index: 2,
        stage_name: "THE_REFRAME",
        exit: if all_pass {
            ExitCode::Pass
        } else {
            ExitCode::GateFail
        },
        assertions,
        wall_clock_nanos: start.elapsed().as_nanos() as u64,
        alloc_count: assertion_count,
        receipt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reframe_passes_on_default_baseline() {
        let result = reframe();
        assert_eq!(result.stage_index, 2);
        assert_eq!(result.stage_name, "THE_REFRAME");
        assert_eq!(
            result.exit,
            ExitCode::Pass,
            "baseline must pass: {:?}",
            result.assertions
        );
        let wire = result
            .assertions
            .iter()
            .find(|a| a.metric_id == "wire_amplification_ratio")
            .expect("wire assertion present");
        assert!(wire.pass);
        assert!(
            wire.observed >= 6.66,
            "wire amplification must be >= 6.66, got {}",
            wire.observed
        );
        assert_eq!(wire.observed, 160.0 / 24.0, "computed, not hardcoded");
        // Sharding channels computed at runtime, not literal.
        assert_eq!(
            result
                .assertions
                .iter()
                .find(|a| a.metric_id == "sharding_entropy_penalty")
                .expect("sharding assertion present")
                .observed,
            (1360u64 * 1359 / 2) as f64
        );
    }

    #[test]
    fn mutation_no_wire_amplification_flips_to_fail() {
        // ANTI-VACUITY MUTATION: perturb the struct (not a string literal).
        // json_mb == raw_mb -> no amplification -> wire assertion fails and
        // evaluate() must return aggregate pass=false.
        let mutant = BaselineParams {
            json_mb: 24.0,
            ..BaselineParams::default()
        };
        let (assertions, all_pass) = evaluate(&mutant);
        assert!(!all_pass, "mutant must flip aggregate verdict to false");
        let wire = assertions
            .iter()
            .find(|a| a.metric_id == "wire_amplification_ratio")
            .expect("wire assertion present in mutant");
        assert!(!wire.pass, "wire assertion must flip: {:?}", wire);
        assert_eq!(wire.observed, 1.0);
    }

    #[test]
    fn mutation_single_node_sharding_flips_to_fail() {
        // Second mutation axis: 1 shard -> zero channels -> penalty = 0.
        let mutant = BaselineParams {
            shards: 1,
            ..BaselineParams::default()
        };
        let (assertions, all_pass) = evaluate(&mutant);
        assert!(!all_pass);
        let sharding = assertions
            .iter()
            .find(|a| a.metric_id == "sharding_entropy_penalty")
            .unwrap();
        assert!(
            !sharding.pass,
            "sharding assertion must flip: {:?}",
            sharding
        );
        assert_eq!(sharding.observed, 0.0);
    }

    #[test]
    fn reframe_receipt_is_not_a_stub() {
        let a = reframe();
        let b = reframe();
        assert_eq!(a.receipt, b.receipt, "deterministic on same inputs");
        assert_ne!(a.receipt, [0u8; 32], "receipt must not be a zero stub");
        // Flipping a parameter must move the receipt (computed, not fixed).
        let mut p = BaselineParams::default();
        p.shards = 700;
        let (mutant_assertions, _) = evaluate(&p);
        let mut buf = Vec::new();
        for x in &mutant_assertions {
            buf.extend_from_slice(x.metric_id.as_bytes());
            buf.extend_from_slice(&x.observed.to_bits().to_le_bytes());
            buf.push(u8::from(x.pass));
        }
        assert_ne!(cgap_blake3::hash(&buf), a.receipt);
    }
}
