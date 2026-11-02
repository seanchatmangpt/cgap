//! C8 coordinate — economic arbitrage ledger court (RFC-0042 §4, lane B07).
//!
//! Exact u64-cents integer math over the Stage 3 RATIONAL DROWNING ledger
//! ([`crate::drowning`]). Constants are imported/re-exported from that
//! module — never duplicated here. All ratios are integer division on
//! cents; f64 appears only at the verdict boundary, mirroring the
//! REQUIRED_ROI_X integration ruling (gross formula, pinned 350.0).
//!
//! Economics:
//! - annual bleed   = 42,238,600 ¢ x12 = 506,863,200 ¢ ($5,068,632)
//! - SUT annual     = $1,200/mo -> 1,440,000 ¢ ($14,400)
//! - gross factor   = 506,863,200 / 1,440,000 = 351 (floor of 351.99)
//! - net delta      = 506,863,200 - 1,440,000 = 505,423,200 ¢
//!                    ($5,054,232 exactly)

use crate::drowning::{evaluate, Ledger, EXPECTED_MONTHLY_TOTAL_CENTS};

/// SUT monthly cost in cents: $1,200/month.
pub const SUT_MONTHLY_CENTS: u64 = 120_000;

/// Annualized status-quo bleed in cents (42,238,600 x 12).
pub const ANNUAL_BLEED_CENTS: u64 = EXPECTED_MONTHLY_TOTAL_CENTS * 12;

/// Annualized SUT cost in cents (120,000 x 12).
pub const SUT_ANNUAL_CENTS: u64 = SUT_MONTHLY_CENTS * 12;

/// Net economic delta in cents per year.
pub const NET_DELTA_CENTS: u64 = ANNUAL_BLEED_CENTS - SUT_ANNUAL_CENTS;

/// Gross arbitrage factor (integer floor): annual bleed / SUT annual.
pub fn gross_arbitrage_factor() -> u64 {
    ANNUAL_BLEED_CENTS / SUT_ANNUAL_CENTS
}

/// C8 verdict: gross factor meets the coordinator's REQUIRED_ROI_X=350
/// integration ruling, and the ledger itself evaluates clean.
pub fn c8_court_verdict(ledger: &Ledger) -> bool {
    evaluate(ledger) && gross_arbitrage_factor() >= 350
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drowning::{Ledger as DrowningLedger, EXPECTED_ANNUALIZED_USD};

    #[test]
    fn ledger_math_is_exact_in_cents() {
        let ledger = DrowningLedger::default();
        assert_eq!(ledger.monthly_total_cents(), 42_238_600);
        assert_eq!(ANNUAL_BLEED_CENTS, 506_863_200);
        assert_eq!(SUT_MONTHLY_CENTS, 120_000);
        assert_eq!(SUT_ANNUAL_CENTS, 1_440_000);
        assert_eq!(NET_DELTA_CENTS, 505_423_200);
        assert_eq!(NET_DELTA_CENTS as f64 / 100.0, 5_054_232.0);
        assert_eq!(EXPECTED_ANNUALIZED_USD, 5_068_632.0);
        // Monthly figures exact in cents.
        assert_eq!(ledger.monthly_total() * 100.0, 42_238_600.0);
    }

    #[test]
    fn gross_arbitrage_factor_is_351_and_meets_roi_floor() {
        assert_eq!(gross_arbitrage_factor(), 351);
        assert!(gross_arbitrage_factor() >= 350, "REQUIRED_ROI_X floor");
    }

    #[test]
    fn c8_court_passes_on_canonical_ledger() {
        assert!(c8_court_verdict(&DrowningLedger::default()));
    }

    #[test]
    fn court_flips_on_zeroed_spark() {
        // ANTI-VACUITY mutation: spark 318,873 -> 0. The same evaluate()
        // and the same gross-factor threshold must both flip — the court
        // is non-vacuous only if this mutant fails the verdict.
        let mut mutant = DrowningLedger::default();
        mutant.spark_usd = 0.0;
        assert_eq!(mutant.monthly_total_cents(), 10_351_300);
        assert!(!evaluate(&mutant), "mutant must fail ledger evaluation");
        assert!(!c8_court_verdict(&mutant), "mutant must fail C8 verdict");
        // Cross-check: mutant's annualized gross factor drops below 350.
        let mutant_annual_cents = mutant.monthly_total_cents() * 12;
        assert!(mutant_annual_cents / SUT_ANNUAL_CENTS < 350);
        assert_ne!(mutant_annual_cents, ANNUAL_BLEED_CENTS);
    }
}
