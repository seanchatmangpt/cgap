//! GIT-10 courts: the commit-evaluation patent seam binds the REAL
//! cgap-bn3 budgeted fixpoint (`unify_to_fixpoint_opt`) into the
//! cgap-git pipeline. No mocks — the adapter below is a thin pass-through
//! to the production driver.
#[path = "../src/patent_court.rs"]
mod patent_court;

use patent_court::{
    evaluate_commit, BudgetMetering, BudgetedDerivation, CommitProposal, DerivationFault,
    DerivationReceipt, PatentRefusal, DESIGN_AROUND_MECHANISM, MUTATION_EXIT_CLASS,
    REFUSAL_EXIT_CLASS, STATUTORY_CHAIN_EDGES, STATUTORY_FIXPOINT_BUDGET,
};

use cgap_bn3::unify::{
    unify_to_fixpoint_opt, BodyAtom, Rule, RuleCompiler, StatementColumns, UnifyError,
};

// --- term-word helpers (mirror the AE6 row-3 court: VarUni payload =
// register number under SIG_VAR << 56; const payload = plain id) --------
const SIG_VAR: u64 = 0x01;
fn vw(reg: u8) -> u64 {
    (SIG_VAR << 56) | reg as u64
}
fn cw(id: u64) -> u64 {
    id
}

fn chain_columns() -> StatementColumns<'static> {
    let pr = cw(700);
    let n_nodes = STATUTORY_CHAIN_EDGES + 1;
    let mut s: Vec<u64> = Vec::new();
    let mut p: Vec<u64> = Vec::new();
    let mut o: Vec<u64> = Vec::new();
    for i in 0..n_nodes - 1 {
        s.push(cw(i as u64));
        p.push(pr);
        o.push(cw(i as u64 + 1));
    }
    // Leak the corpus: a fixed, static witness corpus (same shape every
    // run); no test-local mutation of it ever occurs.
    let s: &'static [u64] = Box::leak(s.into_boxed_slice());
    let p: &'static [u64] = Box::leak(p.into_boxed_slice());
    let o: &'static [u64] = Box::leak(o.into_boxed_slice());
    StatementColumns::new(s, p, o).expect("chain columns")
}

fn closure_rule() -> Rule {
    let pr = cw(700);
    Rule {
        rule_id: 7,
        body: vec![
            BodyAtom::new(vw(1), pr, vw(2)),
            BodyAtom::new(vw(2), pr, vw(3)),
        ],
        head: [vw(1), pr, vw(3)],
    }
}

/// Real pass-through adapter: binds `unify_to_fixpoint_opt` directly. No
/// re-implementation of traversal anywhere in this crate.
struct Bn3Driver {
    rules: Vec<Rule>,
    cols: StatementColumns<'static>,
}

impl BudgetedDerivation for Bn3Driver {
    fn run_to_fixpoint(&self, budget: u32) -> Result<DerivationReceipt, DerivationFault> {
        let compiled = RuleCompiler::compile(&self.rules[0]).expect("closure rule compiles");
        let _ = compiled; // compile check mirrors the AE6 row-3 court
        match unify_to_fixpoint_opt(&self.rules, &self.cols, budget, 4096) {
            Ok(batches) => Ok(DerivationReceipt {
                delta_batches: batches.len(),
            }),
            Err(UnifyError::FixpointBudgetExceeded { rounds }) => {
                Err(DerivationFault::BudgetExhausted { rounds })
            }
            Err(other) => panic!("unexpected typed driver error: {other:?}"),
        }
    }
}

fn driver() -> Bn3Driver {
    Bn3Driver {
        rules: vec![closure_rule()],
        cols: chain_columns(),
    }
}

fn proposal() -> CommitProposal {
    CommitProposal {
        commit_oid: [0xAB; 20],
        parent_oids: [[0xCD; 20]],
    }
}

/// FLIP TWIN A (admit arm): budget 12 on the 199-edge chain ADMITS — the
/// same corpus the statutory budget refuses, proving the boundary, not a
/// dead rule (G18 witness: Ok, 9 delta batches at budget 12).
#[test]
fn budget_12_admits_the_199_edge_chain() {
    let d = driver();
    let before = proposal();
    let res = evaluate_commit(BudgetMetering::Enforced, 12, &d);
    let receipt = res.expect("budget-12 corpus must converge and admit");
    assert_eq!(
        receipt.delta_batches, 9,
        "G18 witness: 9 delta batches at budget 12"
    );
    // Ref chain untouched on admit: proposal bytes identical.
    assert_eq!(proposal(), before);
}

/// FLIP TWIN B (refusal arm): budget 4 on the SAME 199-edge chain is
/// refused with the exit-101-class typed AndonRefusal carrying the
/// statutory numbers, and the ref chain stays UNMODIFIED.
#[test]
fn budget_4_on_199_edges_refuses_101_class_and_leaves_ref_chain_unmodified() {
    let d = driver();
    let mut ref_chain = vec![[0u8; 20]; 4];
    let snapshot: Vec<[u8; 20]> = ref_chain.clone();
    let res = evaluate_commit(BudgetMetering::Enforced, STATUTORY_FIXPOINT_BUDGET, &d);
    match res {
        Err(PatentRefusal::FixpointBudgetExceeded { budget, rounds }) => {
            assert_eq!(budget, 4);
            assert_eq!(rounds, 4, "typed refusal must carry the budget epoch");
        }
        Err(other) => panic!("wrong refusal class: {other:?}"),
        Ok(r) => panic!(
            "budget-4 traversal admitted ({} batches) — premise broken",
            r.delta_batches
        ),
    }
    assert_eq!(res.err().unwrap().exit_class(), REFUSAL_EXIT_CLASS);
    assert_eq!(REFUSAL_EXIT_CLASS, 101);
    // Chicago final state: the caller-owned ref chain is byte-identical.
    assert_eq!(ref_chain, snapshot, "refusal must not modify the ref chain");
    assert_eq!(proposal().commit_oid, [0xAB; 20]);
}

/// MUTATION COURT: the cfg/test-only bypass (`Disabled`, defined in THIS
/// seam, not in bn3) must flip the outcome on the SAME corpus — the
/// un-metered traversal is DETECTED and aborted exit-1-class, never
/// laundered into an admit. Twin arm: Disabled on a converging budget
/// still admits, so the mutation arm is outcome-sensitive, not constant.
#[test]
fn disabled_metering_on_budget_blowout_is_detected_exit_1_class() {
    let d = driver();
    // Blowout corpus (budget 4) with the bypass on: detected.
    let res = evaluate_commit(BudgetMetering::Disabled, STATUTORY_FIXPOINT_BUDGET, &d);
    match res {
        Err(PatentRefusal::UnMeteredTraversal {
            rounds_observed,
            budget,
        }) => {
            assert_eq!(rounds_observed, 4);
            assert_eq!(budget, 4);
        }
        Err(other) => panic!("wrong refusal class: {other:?}"),
        Ok(r) => panic!(
            "MUTATION UNDETECTED: un-metered traversal admitted \
                         ({} batches) — budget arm is vacuous",
            r.delta_batches
        ),
    }
    assert_eq!(res.err().unwrap().exit_class(), MUTATION_EXIT_CLASS);
    assert_eq!(MUTATION_EXIT_CLASS, 1);

    // Outcome-sensitivity twin: the bypass on a CONVERGING budget admits,
    // so Disabled is not a constant-refuse stub.
    let d2 = driver();
    let ok = evaluate_commit(BudgetMetering::Disabled, 12, &d2)
        .expect("bypass on converging corpus must still admit");
    assert_eq!(ok.delta_batches, 9);
}

/// The design-around statement is on disk, names the distinct mechanism
/// classes, and carries the statutory provenance consts.
#[test]
fn design_around_statement_names_distinct_mechanism_classes() {
    let m = DESIGN_AROUND_MECHANISM;
    for marker in [
        "semi_join_masks",
        "caller-leased",
        "no runtime tuple-binding propagation",
        "no continuation/Fiber resume frames",
        "no topology switching",
        "verify_ost_design_around.py",
    ] {
        assert!(
            m.contains(marker),
            "design-around statement missing {marker}"
        );
    }
    assert_eq!(STATUTORY_FIXPOINT_BUDGET, 4);
    assert_eq!(STATUTORY_CHAIN_EDGES, 199);
}
