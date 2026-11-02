//! CSCT v1 core types per RFC-0042 (lane CSCT-L2).
//!
//! No protobuf, no JSON, no serde: dogfoods cgap's own columnar/BLAKE3
//! kernels. `report_digest` hashes the report field-by-field into a
//! deterministic little-endian byte buffer via `cgap_blake3::hash`.

/// Gate exit codes (fail-closed vocabulary; a skip is loud, never a fake pass).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExitCode {
    Pass = 0,
    GateFail = 1,
    SubjectMissing = 2,
    LoudSkip = 3,
    AndonRefusal = 101,
}

impl ExitCode {
    /// Stable wire integer for this exit code.
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// One observed metric vs its threshold, asserted at a stage boundary.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct StateAssertion {
    pub metric_id: &'static str,
    pub observed: f64,
    pub threshold: f64,
    pub unit: &'static str,
    pub pass: bool,
}

/// One stage's outcome: exit code, assertions, wall clock, allocs, receipt.
#[derive(Clone, PartialEq, Debug)]
pub struct StageResult {
    pub stage_index: u8,
    pub stage_name: &'static str,
    pub exit: ExitCode,
    pub assertions: Vec<StateAssertion>,
    pub wall_clock_nanos: u64,
    pub alloc_count: u64,
    pub receipt: [u8; 32],
}

/// Full suite report with the economic block.
#[derive(Clone, Debug)]
pub struct Report {
    pub suite_id: &'static str,
    pub customer: &'static str,
    pub stages: Vec<StageResult>,
    pub total_annual_runrate_bleeding_usd: f64,
    pub total_annual_post_adoption_cost_usd: f64,
    pub net_economic_delta_usd: f64,
    pub final_commercial_verdict: bool,
}

/// Deterministic BLAKE3 digest of a [`Report`].
///
/// Field-by-field into one little-endian byte buffer (fixed order, lengths
/// prefixed, f64 written as little-endian IEEE-754 bits); no HashMap
/// iteration anywhere. Same report -> same digest, byte-for-byte.
pub fn report_digest(report: &Report) -> [u8; 32] {
    let mut buf: Vec<u8> = Vec::new();

    // String fields: u64 byte-length prefix (LE) + raw bytes.
    for s in [
        report.suite_id,
        report.customer,
        // economic scalars and flags interleave in fixed field order below
    ] {
        buf.extend_from_slice(&(s.len() as u64).to_le_bytes());
        buf.extend_from_slice(s.as_bytes());
    }

    buf.extend_from_slice(
        &report
            .total_annual_runrate_bleeding_usd
            .to_bits()
            .to_le_bytes(),
    );
    buf.extend_from_slice(
        &report
            .total_annual_post_adoption_cost_usd
            .to_bits()
            .to_le_bytes(),
    );
    buf.extend_from_slice(&report.net_economic_delta_usd.to_bits().to_le_bytes());
    buf.push(u8::from(report.final_commercial_verdict));

    // Stage count, then each stage in Vec order (never sorted).
    buf.extend_from_slice(&(report.stages.len() as u64).to_le_bytes());
    for stage in &report.stages {
        buf.push(stage.stage_index);
        buf.extend_from_slice(&(stage.stage_name.len() as u64).to_le_bytes());
        buf.extend_from_slice(stage.stage_name.as_bytes());
        buf.extend_from_slice(&stage.exit.as_u32().to_le_bytes());

        // Assertion count, then each assertion in order.
        buf.extend_from_slice(&(stage.assertions.len() as u64).to_le_bytes());
        for a in &stage.assertions {
            buf.extend_from_slice(&(a.metric_id.len() as u64).to_le_bytes());
            buf.extend_from_slice(a.metric_id.as_bytes());
            buf.extend_from_slice(&a.observed.to_bits().to_le_bytes());
            buf.extend_from_slice(&a.threshold.to_bits().to_le_bytes());
            buf.extend_from_slice(&(a.unit.len() as u64).to_le_bytes());
            buf.extend_from_slice(a.unit.as_bytes());
            buf.push(u8::from(a.pass));
        }

        buf.extend_from_slice(&stage.wall_clock_nanos.to_le_bytes());
        buf.extend_from_slice(&stage.alloc_count.to_le_bytes());
        buf.extend_from_slice(&stage.receipt);
    }

    cgap_blake3::hash(&buf)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn digest_is_deterministic_across_clones() {
        let a = report_digest(&sample_report());
        let b = report_digest(&sample_report());
        assert_eq!(a, b, "identical reports must hash identically");
    }

    #[test]
    fn digest_mutates_on_field_flip() {
        // Anti-vacuity mutation: flip one observed value -> digest MUST move.
        let mut mutated = sample_report();
        mutated.stages[0].assertions[0].observed = 1.0;
        assert_ne!(
            report_digest(&sample_report()),
            report_digest(&mutated),
            "digest collision on mutated observed value"
        );
    }

    #[test]
    fn digest_mutates_on_stage_removal() {
        let mut mutated = sample_report();
        mutated.stages.clear();
        assert_ne!(
            report_digest(&sample_report()),
            report_digest(&mutated),
            "digest collision on empty stage list"
        );
    }

    #[test]
    fn digest_mutates_on_verdict_flip() {
        let mut mutated = sample_report();
        mutated.final_commercial_verdict = false;
        assert_ne!(
            report_digest(&sample_report()),
            report_digest(&mutated),
            "digest collision on verdict flip"
        );
    }

    #[test]
    fn exit_code_wire_values() {
        assert_eq!(ExitCode::Pass.as_u32(), 0);
        assert_eq!(ExitCode::GateFail.as_u32(), 1);
        assert_eq!(ExitCode::SubjectMissing.as_u32(), 2);
        assert_eq!(ExitCode::LoudSkip.as_u32(), 3);
        assert_eq!(ExitCode::AndonRefusal.as_u32(), 101);
    }
}
