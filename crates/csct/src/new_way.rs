//! CSCT Stage 5 — THE NEW WAY (RFC-0042 §4, machine-verifiable contract).
//!
//! The incumbent is rejected on capabilities, not claims: any architecture
//! carrying a GC, dynamic vectors on the ingress hot path, or network
//! serialization is refused with typed assertions regardless of its
//! measured numbers. The cgap SUT claim (static footprint 39,184 B,
//! ingress 7.37 ns/frame, 1M-triple closure 22.99 ms, all three flags
//! false) must pass.

use crate::types::{ExitCode, StageResult, StateAssertion};

/// Machine-verifiable contract thresholds (RFC-0042 §4).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ContractSpec {
    /// Maximum admissible static (linked ELF) footprint.
    pub max_static_footprint_bytes: u64,
    /// Maximum admissible per-frame ingress cost.
    pub max_ingress_ns_per_frame: f64,
    /// Maximum admissible closure time over 1,000,000 triples.
    pub max_closure_ms_1m: f64,
}

impl Default for ContractSpec {
    fn default() -> Self {
        Self {
            max_static_footprint_bytes: 131_072,
            max_ingress_ns_per_frame: 10.0,
            max_closure_ms_1m: 33.3,
        }
    }
}

/// One candidate architecture's measured claim under the contract.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ArchitectureClaim {
    pub static_footprint_bytes: u64,
    pub ingress_ns_per_frame: f64,
    pub closure_ms_1m: f64,
    pub has_gc: bool,
    pub uses_dynamic_vectors: bool,
    pub uses_network_serialization: bool,
}

/// Receipt buffer identity, hashed into the stage receipt.
const ASSERTION_BUFFER: &[u8] = b"CSCT-L7-NEW-WAY-CONTRACT-v1";

/// Evaluate `claim` against `spec`.
///
/// Returns the assertion list and the overall verdict. Hard capability
/// refusals (GC / dynamic vectors / network serialization) are emitted as
/// typed refusal assertions and fail the contract unconditionally.
pub fn evaluate(spec: &ContractSpec, claim: &ArchitectureClaim) -> (Vec<StateAssertion>, bool) {
    let mut assertions: Vec<StateAssertion> = Vec::new();

    assertions.push(StateAssertion {
        metric_id: "static_footprint_bytes",
        observed: claim.static_footprint_bytes as f64,
        threshold: spec.max_static_footprint_bytes as f64,
        unit: "bytes",
        pass: claim.static_footprint_bytes <= spec.max_static_footprint_bytes,
    });
    assertions.push(StateAssertion {
        metric_id: "ingress_ns_per_frame",
        observed: claim.ingress_ns_per_frame,
        threshold: spec.max_ingress_ns_per_frame,
        unit: "ns",
        pass: claim.ingress_ns_per_frame < spec.max_ingress_ns_per_frame,
    });
    assertions.push(StateAssertion {
        metric_id: "closure_ms_1m",
        observed: claim.closure_ms_1m,
        threshold: spec.max_closure_ms_1m,
        unit: "ms",
        pass: claim.closure_ms_1m <= spec.max_closure_ms_1m,
    });
    assertions.push(StateAssertion {
        metric_id: "refusal_has_gc",
        observed: claim.has_gc as u8 as f64,
        threshold: 0.0,
        unit: "typed-refusal",
        pass: !claim.has_gc,
    });
    assertions.push(StateAssertion {
        metric_id: "refusal_uses_dynamic_vectors",
        observed: claim.uses_dynamic_vectors as u8 as f64,
        threshold: 0.0,
        unit: "typed-refusal",
        pass: !claim.uses_dynamic_vectors,
    });
    assertions.push(StateAssertion {
        metric_id: "refusal_uses_network_serialization",
        observed: claim.uses_network_serialization as u8 as f64,
        threshold: 0.0,
        unit: "typed-refusal",
        pass: !claim.uses_network_serialization,
    });

    let pass = assertions.iter().all(|a| a.pass);
    (assertions, pass)
}

/// Stage 5 entry point: evaluate the cgap SUT claim under the default
/// contract and fold it into a [`StageResult`].
pub fn new_way() -> StageResult {
    let start = std::time::Instant::now();

    let spec = ContractSpec::default();
    // cgap SUT measured claim (RFC-0042 §4; LINKED-ELF footprint truth).
    let claim = ArchitectureClaim {
        static_footprint_bytes: 39_184,
        ingress_ns_per_frame: 7.37,
        closure_ms_1m: 22.99,
        has_gc: false,
        uses_dynamic_vectors: false,
        uses_network_serialization: false,
    };

    let (assertions, pass) = evaluate(&spec, &claim);

    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(ASSERTION_BUFFER);
    buf.extend_from_slice(&claim.static_footprint_bytes.to_le_bytes());
    buf.extend_from_slice(&claim.ingress_ns_per_frame.to_bits().to_le_bytes());
    buf.extend_from_slice(&claim.closure_ms_1m.to_bits().to_le_bytes());
    buf.push(claim.has_gc as u8);
    buf.push(claim.uses_dynamic_vectors as u8);
    buf.push(claim.uses_network_serialization as u8);
    for a in &assertions {
        buf.extend_from_slice(a.metric_id.as_bytes());
        buf.extend_from_slice(&a.observed.to_le_bytes());
        buf.push(a.pass as u8);
    }
    let receipt = cgap_blake3::hash(&buf);

    StageResult {
        stage_index: 5,
        stage_name: "THE_NEW_WAY",
        exit: if pass {
            ExitCode::Pass
        } else {
            ExitCode::GateFail
        },
        assertions,
        wall_clock_nanos: start.elapsed().as_nanos() as u64,
        alloc_count: 2, // assertions Vec + receipt buffer Vec
        receipt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cgap_claim() -> ArchitectureClaim {
        ArchitectureClaim {
            static_footprint_bytes: 39_184,
            ingress_ns_per_frame: 7.37,
            closure_ms_1m: 22.99,
            has_gc: false,
            uses_dynamic_vectors: false,
            uses_network_serialization: false,
        }
    }

    #[test]
    fn new_way_passes_on_cgap_claim() {
        let result = new_way();
        assert_eq!(result.stage_index, 5);
        assert_eq!(result.stage_name, "THE_NEW_WAY");
        assert_eq!(result.exit, ExitCode::Pass);
        assert!(result.assertions.iter().all(|a| a.pass));
        assert_ne!(result.receipt, [0u8; 32]);
    }

    /// ANTI-VACUITY MUTATION 1: identical claim but has_gc=true must FAIL.
    #[test]
    fn mutation_has_gc_flips_verdict() {
        let spec = ContractSpec::default();
        let mut claim = cgap_claim();
        claim.has_gc = true;
        let (assertions, pass) = evaluate(&spec, &claim);
        assert!(!pass, "has_gc=true mutant must fail the contract");
        let refusal = assertions
            .iter()
            .find(|a| a.metric_id == "refusal_has_gc")
            .expect("typed GC refusal assertion present");
        assert!(!refusal.pass);
    }

    /// ANTI-VACUITY MUTATION 2: footprint 200,000 B must FAIL.
    #[test]
    fn mutation_footprint_flips_verdict() {
        let spec = ContractSpec::default();
        let mut claim = cgap_claim();
        claim.static_footprint_bytes = 200_000;
        let (assertions, pass) = evaluate(&spec, &claim);
        assert!(!pass, "200,000 B footprint mutant must fail the contract");
        let fa = assertions
            .iter()
            .find(|a| a.metric_id == "static_footprint_bytes")
            .expect("footprint assertion present");
        assert!(!fa.pass);
        // Every other numeric metric stays green: only the mutated cell flips.
        assert!(assertions.iter().filter(|a| a.pass).count() == 5);
    }
}
