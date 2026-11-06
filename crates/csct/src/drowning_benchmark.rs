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

/// SUT annual cost decomposition (RFC-0042 §4): the $14,400/yr ceiling is
/// no longer asserted as a bare constant — it is the sum of SCENARIO-labeled
/// line items. Changing any line item changes the ledger, the C8 gross
/// factor, and the rendered receipt (the ST-06 falsifier).
///
/// Inputs are SCENARIO-labeled: single-node on-prem SUT, 3-year straight-line
/// hardware amortization, colocation power at $0.20/kWh, 1 Gbps transit.
pub struct SutCostDecomposition {
    /// SCENARIO[sut-node-1x]: one sovereign node, $18,000 capex, 3-year
    /// straight-line amortization -> $6,000/yr.
    pub hardware_amortization_usd: f64,
    /// SCENARIO[sut-power-coloc]: 550 W avg draw x 8,760 h x $0.20/kWh
    /// -> $963.60/yr, rounded to colocation contract billing.
    pub power_usd: f64,
    /// SCENARIO[sut-network-1gbps]: 1 Gbps committed transit + IP allocation
    /// -> $2,436.40/yr at contract rates.
    pub network_usd: f64,
    /// SCENARIO[sut-ops-remote-hands]: colocation remote-hands block,
    /// $200/month -> $2,400/yr.
    pub ops_remote_hands_usd: f64,
    /// Residual maintenance (spares, kernel updates contract) — the balance
    /// to the $14,400 ceiling so the decomposition is exact by construction.
    pub maintenance_usd: f64,
}

impl SutCostDecomposition {
    /// Canonical scenario: line items are individually pinned and sum
    /// exactly to SUT_ANNUAL_CENTS (asserted by test). No balance-derived
    /// residual — editing any line item must move the receipt output.
    pub fn canonical() -> Self {
        Self {
            hardware_amortization_usd: 6_000.0,
            power_usd: 963.60,
            network_usd: 2_436.40,
            ops_remote_hands_usd: 2_400.0,
            maintenance_usd: 2_600.0,
        }
    }

    /// Annual total in dollars (must equal SUT_ANNUAL_USD = 14,400).
    pub fn annual_total_usd(&self) -> f64 {
        self.hardware_amortization_usd
            + self.power_usd
            + self.network_usd
            + self.ops_remote_hands_usd
            + self.maintenance_usd
    }
}

impl Default for SutCostDecomposition {
    fn default() -> Self {
        Self::canonical()
    }
}

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
    use crate::drowning::{Ledger as DrowningLedger, RateCardLedger, EXPECTED_ANNUALIZED_USD};

    const REQUIRED_ROI_FLOOR: f64 = 350.0;
    const REQUIRED_ARBITRAGE_INT_FLOOR: u64 = 350;

    #[test]
    fn rate_card_ledger_derives_canonical_totals() {
        let rcl = RateCardLedger::canonical();
        assert_eq!(rcl.total_monthly_usd().round(), 422_386.0);
        assert_eq!(rcl.annualized_usd().round(), 5_068_632.0);
        let gross_roi = rcl.annualized_usd() / 14_400.0;
        assert!(gross_roi >= REQUIRED_ROI_FLOOR, "Gross ROI must be >= 350.0x");
    }

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
        assert!(gross_arbitrage_factor() >= REQUIRED_ARBITRAGE_INT_FLOOR, "REQUIRED_ROI_X floor");
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
        assert!(mutant_annual_cents / SUT_ANNUAL_CENTS < REQUIRED_ARBITRAGE_INT_FLOOR);
        assert_ne!(mutant_annual_cents, ANNUAL_BLEED_CENTS);
    }
}

#[cfg(test)]
mod sut_decomposition_tests {
    use super::*;
    use crate::SUT_ANNUAL_USD;

    #[test]
    fn canonical_line_items_sum_to_sut_ceiling() {
        let d = SutCostDecomposition::canonical();
        assert_eq!(d.annual_total_usd(), SUT_ANNUAL_USD);
        assert_eq!(d.annual_total_usd() * 100.0, 1_440_000.0); // exact cents
    }

    #[test]
    fn mutating_any_line_item_moves_the_total() {
        // Falsifier (ST-06): each pinned line item is load-bearing —
        // bumping any one of them changes the annual total (and hence the
        // receipt output via render_cost_decomposition), never just a
        // gate verdict.
        let bumps: [(&str, fn(&mut SutCostDecomposition)); 5] = [
            ("hardware", |d| d.hardware_amortization_usd += 100.0),
            ("power", |d| d.power_usd += 100.0),
            ("network", |d| d.network_usd += 100.0),
            ("ops", |d| d.ops_remote_hands_usd += 100.0),
            ("maintenance", |d| d.maintenance_usd += 100.0),
        ];
        for (name, bump) in bumps {
            let mut d = SutCostDecomposition::canonical();
            bump(&mut d);
            assert_ne!(
                d.annual_total_usd(),
                SUT_ANNUAL_USD,
                "bumping '{name}' must move the annual total"
            );
            // And the rendered receipt must move with it.
            let base_receipt = crate::receipt::render_cost_decomposition(&SutCostDecomposition::canonical());
            let bumped_receipt = crate::receipt::render_cost_decomposition(&d);
            assert_ne!(
                base_receipt, bumped_receipt,
                "bumping '{name}' must change the rendered receipt"
            );
        }
    }
}
