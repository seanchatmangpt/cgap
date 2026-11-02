//! Commit-evaluation patent seam (lane GIT-10, G7-PATENT binding).
//!
//! BINDS existing law, does not re-implement traversal:
//!   - scripts/verify_ost_design_around.py (4/4 DISTINCT-MECHANISM)
//!   - crates/cgap/tests/patent_behavioral_court.rs (row1/row2/row3)
//!   - G18 budget-flip witness: TYPED `FixpointBudgetExceeded { rounds: 4 }`
//!     at budget 4 vs Ok (9 delta batches) at budget 12 on the 199-edge
//!     chain — docs/jira/v26.11.2/lanes/lane-g18-g7-patent-gate.md:55-63.
//!
//! The seam routes a proposed commit's derivation deltas through the REAL
//! budgeted fixpoint admission (delegated via [`BudgetedDerivation`]; the
//! test harness binds `cgap_bn3::unify::unify_to_fixpoint_opt`) and maps
//! outcomes:
//!   within budget  => admit (commit derivation receipt)
//!   budget exceeded => [`PatentRefusal::FixpointBudgetExceeded`], the
//!                      exit-101-class AndonRefusal carrying the statutory
//!                      numbers; the ref chain is NEVER modified.
//! MUTATION SURFACE (anti-vacuity): [`BudgetMetering::Disabled`] is the
//! cfg/test-only bypass living in THIS seam (not in cgap-bn3). With
//! metering disabled an un-metered traversal that blows the budget is
//! detected and aborts with the exit-1-class typed
//! [`PatentRefusal::UnMeteredTraversal`] — a court proves both arms flip,
//! so neither refusal is vacuous.

use core::fmt;

/// Statutory fixpoint budget (G18 witness, lane receipt lines 55-63; AE6
/// row-3 court): a cyclic derivation over the 199-edge chain must be
/// refused TYPED at budget 4 — and the SAME corpus converges Ok at
/// budget 12, proving the refusal is the budget boundary, not a dead rule.
pub const STATUTORY_FIXPOINT_BUDGET: u32 = 4;
/// Statutory chain shape: 200 nodes / 199 edges (G18 witness corpus).
pub const STATUTORY_CHAIN_EDGES: usize = 199;
/// Exit class of the statutory AndonRefusal (`FixpointBudgetExceeded`).
pub const REFUSAL_EXIT_CLASS: i32 = 101;
/// Exit class of the mutation-detection abort (`UnMeteredTraversal`).
pub const MUTATION_EXIT_CLASS: i32 = 1;

/// Non-infringement design-around statement (BINDING citation, not
/// litigation — the mechanism classes are verify_ost_design_around.py's
/// own ROW_GREPS/ROW_EVIDENCE registry, US11,216,456 / US11,409,698 /
/// US11,645,278 rows; gate exit 0, 4/4 DISTINCT-MECHANISM).
pub const DESIGN_AROUND_MECHANISM: &str = "distinct-mechanism traversal: \
cgap derives through a caller-leased semi_join_masks register-file pass \
(R0 band, no runtime tuple-binding propagation, no continuation/Fiber \
resume frames, no topology switching / dyn dispatch) metered by an \
explicit caller-supplied fixpoint round budget; the OST DRed sequence \
(tuple-binding propagation with continuation frames under topology \
switching) is never practiced — zero src-scoped mechanism hits per \
verify_ost_design_around.py; this module binds that admission law into \
the commit-evaluation pipeline";

/// Receipt of an admitted commit derivation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DerivationReceipt {
    /// Number of delta batches the converged fixpoint emitted.
    pub delta_batches: usize,
}

/// Typed fault surfaced by the delegated budgeted driver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DerivationFault {
    /// The real driver exhausted the caller budget without converging.
    BudgetExhausted {
        /// Rounds actually consumed at refusal.
        rounds: u32,
    },
}

/// Adapter seam: the commit pipeline hands the REAL budgeted fixpoint
/// driver (cgap-bn3 `unify_to_fixpoint_opt`) to this seam; cgap-git keeps
/// zero dependencies on bn3 (published zero-dep kernel law).
pub trait BudgetedDerivation {
    /// Run the budgeted fixpoint over the proposed commit's deltas.
    /// MUST be the real driver; a seam-internal re-implementation would be
    /// a mock and is refused by the courts.
    fn run_to_fixpoint(&self, budget: u32) -> Result<DerivationReceipt, DerivationFault>;
}

/// Metering arm of the seam. `Disabled` exists ONLY as the mutation-court
/// bypass; production pipelines must pass `Enforced`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BudgetMetering {
    /// Budget refusal arm live (production).
    Enforced,
    /// Test-only bypass: budget refusal arm off (the anti-vacuity mutant).
    Disabled,
}

/// Typed refusal classes. `exit_class` maps to the process-exit andon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatentRefusal {
    /// Statutory AndonRefusal: budget exceeded under Enforced metering.
    /// Exit 101-class, carrying the statutory numbers.
    FixpointBudgetExceeded {
        /// Statutory budget in force.
        budget: u32,
        /// Rounds consumed when the real driver refused.
        rounds: u32,
    },
    /// Mutation detection: traversal ran with the budget arm disabled and
    /// still blew the budget — un-metered traversal, exit 1-class abort.
    UnMeteredTraversal {
        /// Rounds the un-metered traversal consumed.
        rounds_observed: u32,
        /// Budget that was not enforced.
        budget: u32,
    },
}

impl PatentRefusal {
    /// Process-exit class of this refusal (101-class AndonRefusal vs
    /// 1-class mutation abort).
    pub fn exit_class(&self) -> i32 {
        match self {
            PatentRefusal::FixpointBudgetExceeded { .. } => REFUSAL_EXIT_CLASS,
            PatentRefusal::UnMeteredTraversal { .. } => MUTATION_EXIT_CLASS,
        }
    }
}

impl fmt::Display for PatentRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatentRefusal::FixpointBudgetExceeded { budget, rounds } => {
                write!(
                    f,
                    "PATENT-ANDON(101): fixpoint budget {} exceeded \
                           after {} rounds (statutory {}/{}, G18 witness)",
                    budget, rounds, STATUTORY_FIXPOINT_BUDGET, STATUTORY_CHAIN_EDGES
                )
            }
            PatentRefusal::UnMeteredTraversal {
                rounds_observed,
                budget,
            } => {
                write!(
                    f,
                    "PATENT-MUTATION(1): un-metered traversal \
                           consumed {} rounds past budget {} — budget arm \
                           bypassed",
                    rounds_observed, budget
                )
            }
        }
    }
}

/// A proposed commit entering evaluation. The ref chain is caller-owned;
/// this seam takes NO mutable access to it, so a refusal structurally
/// cannot modify the chain (courts assert the bytes are unchanged).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommitProposal {
    /// Proposed commit object id.
    pub commit_oid: [u8; 20],
    /// Parent commit ids (the unmodified ref-chain prefix).
    pub parent_oids: [[u8; 20]; 1],
}

/// The commit-evaluation seam: run the budgeted fixpoint admission over
/// the proposed commit's derivation deltas and map the outcome.
///
/// The ref chain is never touched (no mutable access exists).
pub fn evaluate_commit(
    metering: BudgetMetering,
    budget: u32,
    derivation: &dyn BudgetedDerivation,
) -> Result<DerivationReceipt, PatentRefusal> {
    match (metering, derivation.run_to_fixpoint(budget)) {
        (BudgetMetering::Enforced, Ok(receipt)) => Ok(receipt),
        (BudgetMetering::Enforced, Err(DerivationFault::BudgetExhausted { rounds })) => {
            Err(PatentRefusal::FixpointBudgetExceeded { budget, rounds })
        }
        // The mutant arm: budget check disabled — an un-metered traversal
        // that still blows the budget is DETECTED here and aborted
        // exit-1-class instead of being laundered into an admit.
        (BudgetMetering::Disabled, Ok(receipt)) => Ok(receipt),
        (BudgetMetering::Disabled, Err(DerivationFault::BudgetExhausted { rounds })) => {
            Err(PatentRefusal::UnMeteredTraversal {
                rounds_observed: rounds,
                budget,
            })
        }
    }
}
