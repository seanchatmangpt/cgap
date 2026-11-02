//! CSCT cross-stage mutation registry (RFC-0042 Rule 2, lane CSCT-L9).
//!
//! The per-stage mutation courts live in each stage module's own tests
//! (lanes L3..L8). This module implements the META control: a registry
//! proving each of the 6 stages HAS a registered mutation control, and
//! the gate the runner invokes with `--mutate` semantics.

use crate::types::ExitCode;

/// stage name → registered mutation id (one per RFC-0042 stage).
pub const MUTATION_REGISTRY: [(&str, &str); 6] = [
    ("THE_REFRAME", "zero-tax"),
    ("RATIONAL_DROWNING", "ledger-drift"),
    ("EMOTIONAL_IMPACT", "gc-quiesced"),
    ("THE_NEW_WAY", "gc-claim"),
    ("THE_SOLUTION", "input-flip"),
    ("THE_WARMER", "shard-starved"),
];

/// RFC-0042's six canonical stage names.
const RFC_STAGE_NAMES: [&str; 6] = [
    "THE_REFRAME",
    "RATIONAL_DROWNING",
    "EMOTIONAL_IMPACT",
    "THE_NEW_WAY",
    "THE_SOLUTION",
    "THE_WARMER",
];

/// Validate a mutation registry: exactly 6 entries, all stage names
/// distinct and matching the six RFC stage names, no empty mutation
/// ids. Any violation is a VACUOUS gate (`ExitCode::GateFail`).
pub fn validate(registry: &[(&str, &str)]) -> Result<(), ExitCode> {
    if registry.len() != 6 {
        return Err(ExitCode::GateFail);
    }
    for name in RFC_STAGE_NAMES.iter() {
        let mut hits = 0usize;
        for (stage, mid) in registry.iter() {
            if stage.is_empty() || mid.is_empty() {
                return Err(ExitCode::GateFail);
            }
            if stage == name {
                hits += 1;
            }
        }
        if hits != 1 {
            return Err(ExitCode::GateFail);
        }
    }
    Ok(())
}

/// The gate the runner invokes with `--mutate` semantics: validate the
/// shipped registry; VACUOUS (GateFail) if any check fails.
pub fn run_mutation_controls() -> Result<(), ExitCode> {
    validate(&MUTATION_REGISTRY)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// (a) The shipped registry is well-formed.
    #[test]
    fn mutation_controls_pass() {
        assert_eq!(run_mutation_controls(), Ok(()));
    }

    /// ANTI-VACUITY MUTATION (self-court): a locally-built registry
    /// missing one entry (THE_WARMER dropped) must be detected as
    /// VACUOUS. The court that cannot flip is FAIL.
    #[test]
    fn missing_entry_flips_gate() {
        let mutant: [(&str, &str); 5] = [
            ("THE_REFRAME", "zero-tax"),
            ("RATIONAL_DROWNING", "ledger-drift"),
            ("EMOTIONAL_IMPACT", "gc-quiesced"),
            ("THE_NEW_WAY", "gc-claim"),
            ("THE_SOLUTION", "input-flip"),
        ];
        assert_eq!(validate(&mutant), Err(ExitCode::GateFail));
    }

    /// Second mutation: a duplicate stage name (missing THE_SOLUTION,
    /// THE_WARMER duplicated) also flips — distinctness is enforced.
    #[test]
    fn duplicate_stage_flips_gate() {
        let mutant: [(&str, &str); 6] = [
            ("THE_REFRAME", "zero-tax"),
            ("RATIONAL_DROWNING", "ledger-drift"),
            ("EMOTIONAL_IMPACT", "gc-quiesced"),
            ("THE_NEW_WAY", "gc-claim"),
            ("THE_WARMER", "shard-starved"),
            ("THE_WARMER", "shard-starved"),
        ];
        assert_eq!(validate(&mutant), Err(ExitCode::GateFail));
    }

    /// Third mutation: empty mutation id flips.
    #[test]
    fn empty_mutation_id_flips_gate() {
        let mutant: [(&str, &str); 6] = [
            ("THE_REFRAME", "zero-tax"),
            ("RATIONAL_DROWNING", "ledger-drift"),
            ("EMOTIONAL_IMPACT", "gc-quiesced"),
            ("THE_NEW_WAY", "gc-claim"),
            ("THE_SOLUTION", ""),
            ("THE_WARMER", "shard-starved"),
        ];
        assert_eq!(validate(&mutant), Err(ExitCode::GateFail));
    }
}
