//! CSCT Stage 3 — RATIONAL DROWNING (RFC-0042 §4, lane CSCT-L5).
//!
//! Billing-ledger math for status-quo bleeding. All money is carried as
//! u64 cents internally so the total assertion is float-exact; f64 only
//! appears at the StageResult assertion boundary (RFC-0042 economics:
//! $422,386/month -> $5,068,632/year).

use crate::types::{ExitCode, StageResult, StateAssertion};

/// One line of the status-quo monthly billing ledger, in USD.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Ledger {
    /// Spark compute: $318,873/month.
    pub spark_usd: f64,
    /// Shard fleet: $64,036/month.
    pub shards_usd: f64,
    /// S3 + KMS storage: $28,202/month.
    pub s3_kms_usd: f64,
    /// Cross-AZ egress/replication: $11,275/month.
    pub cross_az_usd: f64,
}

impl Default for Ledger {
    fn default() -> Self {
        Ledger {
            spark_usd: 318_873.0,
            shards_usd: 64_036.0,
            s3_kms_usd: 28_202.0,
            cross_az_usd: 11_275.0,
        }
    }
}

impl Ledger {
    /// Cents for one ledger line, rounded from the f64 USD field.
    fn line_cents(v: f64) -> u64 {
        // f64 cents of a dollars-only value is exact for these magnitudes.
        (v * 100.0).round() as u64
    }

    /// Monthly total in cents (float-exact via u64 integer sum).
    pub fn monthly_total_cents(&self) -> u64 {
        Self::line_cents(self.spark_usd)
            + Self::line_cents(self.shards_usd)
            + Self::line_cents(self.s3_kms_usd)
            + Self::line_cents(self.cross_az_usd)
    }

    /// Monthly total in USD.
    pub fn monthly_total(&self) -> f64 {
        self.monthly_total_cents() as f64 / 100.0
    }

    /// Annualize a monthly USD figure (x12).
    pub fn annualized(monthly_usd: f64) -> f64 {
        monthly_usd * 12.0
    }
}

/// Canonical ledger totals per RFC-0042 §4.
pub const EXPECTED_MONTHLY_TOTAL_CENTS: u64 = 42_238_600;
pub const EXPECTED_ANNUALIZED_USD: f64 = 5_068_632.0;

/// Evaluate the ledger against the RFC-0042 totals. `false` on any drift.
pub fn evaluate(ledger: &Ledger) -> bool {
    let monthly_ok = ledger.monthly_total_cents() == EXPECTED_MONTHLY_TOTAL_CENTS;
    let annual_ok = Ledger::annualized(ledger.monthly_total()) == EXPECTED_ANNUALIZED_USD;
    monthly_ok && annual_ok
}

/// Stage 3: RATIONAL DROWNING. Asserts the billing ledger sums to the
/// pinned RFC-0042 figures; exit Pass only when every assertion holds.
pub fn drowning() -> StageResult {
    let ledger = Ledger::default();
    let total = ledger.monthly_total();
    let annual = Ledger::annualized(total);
    let pass = evaluate(&ledger);

    let assertions = vec![
        StateAssertion {
            metric_id: "drowning_monthly_total_usd",
            observed: total,
            threshold: EXPECTED_MONTHLY_TOTAL_CENTS as f64 / 100.0,
            unit: "USD/month",
            pass: ledger.monthly_total_cents() == EXPECTED_MONTHLY_TOTAL_CENTS,
        },
        StateAssertion {
            metric_id: "drowning_annualized_usd",
            observed: annual,
            threshold: EXPECTED_ANNUALIZED_USD,
            unit: "USD/year",
            pass: annual == EXPECTED_ANNUALIZED_USD,
        },
    ];

    StageResult {
        stage_index: 3,
        stage_name: "RATIONAL_DROWNING",
        exit: if pass {
            ExitCode::Pass
        } else {
            ExitCode::GateFail
        },
        assertions,
        wall_clock_nanos: 0,
        alloc_count: 0,
        receipt: cgap_blake3::hash(b"csct-stage3-rational-drowning"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drowning_passes_with_ledger_math() {
        let result = drowning();
        assert_eq!(result.stage_index, 3);
        assert_eq!(result.stage_name, "RATIONAL_DROWNING");
        assert_eq!(result.exit, ExitCode::Pass);
        assert!(result.assertions.iter().all(|a| a.pass));
        let ledger = Ledger::default();
        assert_eq!(ledger.monthly_total_cents(), 42_238_600);
        assert_eq!(ledger.monthly_total(), 422_386.0);
        assert_eq!(Ledger::annualized(ledger.monthly_total()), 5_068_632.0);
    }

    #[test]
    fn court_flips_on_shards_perturbation() {
        // Anti-vacuity mutation: shave one zero off shards -> total drifts
        // -> evaluate() must return false and the stage must GateFail.
        let mut mutated = Ledger::default();
        mutated.shards_usd = 6_403.0;
        assert_eq!(mutated.monthly_total_cents(), 36_475_300);
        assert!(!evaluate(&mutated), "mutated ledger must fail evaluation");
        assert_ne!(mutated.monthly_total_cents(), EXPECTED_MONTHLY_TOTAL_CENTS);
    }

    #[test]
    fn court_flips_on_cross_az_perturbation() {
        let mut mutated = Ledger::default();
        mutated.cross_az_usd = 11_276.0;
        assert!(!evaluate(&mutated));
    }
}
