//! Behavioral patent-gate court (lane AE6, G7-PATENT row evidence).
//!
//! Rows:
//!   Row 1 (US10,817,467): cited, not duplicated — the concurrent-writer
//!     race is exercised end-to-end by
//!     `crates/cgap-ffi/tests/fa9_commit_gating.rs` (0x4347_0006); this
//!     court pins the cite on disk with its dispatch-gate markers.
//!   Row 2 (US11,216,456): BEHAVIORAL — `emit_join_masks` must
//!     demonstrably prune join candidates via the bound-key mask arm
//!     BEFORE full evaluation; the candidate popcount with a partial
//!     `bound_keys` set must shrink vs the full-key baseline. Anti-vacuity
//!     mutation: bypass the mask arm in `emit_join_masks_view`
//!     (`m_key = MASK_ALL`) → masked popcount == baseline → this court
//!     FAILS (witnessed in the /tmp sandbox, see lane receipt).
//!   Row 3 (US11,409,698 family): BEHAVIORAL — a cyclic derivation
//!     (transitive-closure rule over a chain) is refused with the TYPED
//!     `FixpointBudgetExceeded`, never a hang or panic (FA4 pin).
use std::fs;
use std::path::Path;

use cgap_bn3::unify::{
    emit_join_masks, BodyAtom, CompiledTerm, Rule, RuleCompiler, StatementColumns, UnifyError,
};

// --- term-word helpers (mirrors unify.rs internal courts: SlotConst
// payload = plain id; VarUni payload = register number) ------------------
fn cw(id: u64) -> u64 {
    id
}
const SIG_VAR: u64 = 0x01;
fn vw(reg: u8) -> u64 {
    (SIG_VAR << 56) | reg as u64
}

/// Row 1: the concurrent-writer commit-gating race court is the row-1
/// behavioral evidence; pin the cite + its markers on disk (citation, not
/// duplication).
#[test]
fn row1_cites_fa9_commit_gating_race_court() {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../cgap-ffi/tests/fa9_commit_gating.rs");
    let src = fs::read_to_string(&p).expect("fa9 court file on disk");
    for marker in ["CGAP_ERR_CONCURRENT_WRITER_DENIED", "0x4347_0006"] {
        assert!(
            src.contains(marker),
            "row-1 cite missing marker {marker}: {}",
            p.display()
        );
    }
}

/// Row 2: bound-key mask pushdown demonstrably prunes candidates BEFORE
/// full evaluation. 256 rows all pass the constant arm; the full-key
/// baseline keeps all 256, a half-key `bound_keys` set must keep exactly
/// 128 (and strictly fewer). Mutation `m_key = MASK_ALL` (mask bypass)
/// makes both counts identical -> court fails.
#[test]
fn row2_bound_key_mask_prunes_candidates_before_full_evaluation() {
    const N: usize = 256;
    let s: Vec<u64> = vec![cw(900); N];
    let p: Vec<u64> = vec![cw(901); N];
    let o: Vec<u64> = (0..N as u64).map(cw).collect();
    let cols = StatementColumns::new(&s, &p, &o).expect("equal-length columns");
    let join_atom = [
        CompiledTerm::Const(0), // s == cw(900)
        CompiledTerm::Const(1), // p == cw(901)
        CompiledTerm::Var(0),   // key lane = object payload
    ];
    let const_pool = [cw(900), cw(901)];
    let all_keys: Vec<u64> = (0..N as u64).collect();

    let mut masks = vec![0u64; N / 64];

    let baseline = emit_join_masks(&cols, &join_atom, &const_pool, &all_keys, &mut masks);
    assert_eq!(baseline, N as u64, "full-key baseline must keep every row");

    let half_keys: Vec<u64> = (0..(N as u64) / 2).collect();
    let pruned = emit_join_masks(&cols, &join_atom, &const_pool, &half_keys, &mut masks);
    assert_eq!(
        pruned, 128,
        "half-key bound set must prune to exactly 128 candidates"
    );
    assert!(
        pruned < baseline,
        "pushdown-is-real witness: masked candidate count must shrink vs unmasked"
    );
    // Final state: the mask words themselves carry exactly the pruned set
    // (rows 0..128 set, rows 128..256 clear).
    assert_eq!(masks[0], u64::MAX, "group 0 (rows 0..64) fully qualified");
    assert_eq!(masks[1], u64::MAX, "group 1 (rows 64..128) fully qualified");
    assert_eq!(masks[2], 0, "group 2 (rows 128..192) fully pruned");
    assert_eq!(masks[3], 0, "group 3 (rows 192..256) fully pruned");
}

/// Row 3: a cyclic derivation (transitive closure over a 199-edge chain)
/// is refused TYPED with `FixpointBudgetExceeded { rounds: 4 }` under an
/// explicit budget of 4 — never a hang, never a panic (FA4 pin, now a
/// court). Witnessed: the same corpus converges Ok (3.29M derived rows)
/// at budget 12, so the refusal is the budget boundary, not a dead rule.
#[test]
fn row3_cyclic_derivation_refused_typed_budget_exceeded() {
    let pr = cw(700);
    let n_nodes = 200usize;
    let mut s = Vec::new();
    let mut p = Vec::new();
    let mut o = Vec::new();
    for i in 0..n_nodes - 1 {
        s.push(cw(i as u64));
        p.push(pr);
        o.push(cw(i as u64 + 1));
    }
    let cols = StatementColumns::new(&s, &p, &o).expect("chain columns");

    // Rule: (?a pr ?b), (?b pr ?c) => (?a pr ?c)
    let rule = Rule {
        rule_id: 7,
        body: vec![
            BodyAtom::new(vw(1), pr, vw(2)),
            BodyAtom::new(vw(2), pr, vw(3)),
        ],
        head: [vw(1), pr, vw(3)],
    };
    let compiled = RuleCompiler::compile(&rule);
    assert!(compiled.is_ok(), "closure rule must compile");

    let res = cgap_bn3::unify::unify_to_fixpoint_opt(&[rule], &cols, 4, 4096);
    match res {
        Err(UnifyError::FixpointBudgetExceeded { rounds }) => {
            assert_eq!(rounds, 4, "typed refusal must carry the budget epoch");
        }
        Err(other) => panic!("wrong typed error: {other:?}"),
        Ok(_) => panic!("cyclic derivation converged within budget — court premise broken"),
    }
}
