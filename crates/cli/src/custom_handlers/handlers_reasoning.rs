//! CLI-2 handlers: reasoning + audit surfaces (bn3, audit POWL/OCEL2).

use clap_noun_verb::{NounVerbError, Result};
use serde_json::{json, Map, Value};

fn fail<T>(what: &str, detail: String) -> Result<T> {
    Err(NounVerbError::execution_error(format!("{what}: {detail}")))
}

/// `bn3 unify` — compile one implication rule against the real A12 SIMD
/// unifier: 4-row statement set, 3 matching rows. Every match must carry
/// the projected head with the variable bound to the input subject, or the
/// handler refuses typed.
pub fn bn_3_unify(_inputs: Map<String, Value>) -> Result<Value> {
    use cgap_bn3::sigil::{encode_term, Sigil};
    use cgap_bn3::unify::{BodyAtom, Rule, RuleCompiler, StatementColumns};
    let sc = |p: u64| encode_term(Sigil::SlotConst, p).expect("payload < 2^56");
    let var0 = encode_term(Sigil::VarUni, 0).expect("reg < 256");
    // Data: (x, 1, 9) for x in 10..13, plus one non-matching row.
    let subjects: [u64; 4] = [10, 11, 12, 13];
    let s_col: Vec<u64> = subjects.iter().map(|&x| sc(x)).collect();
    let p_col = vec![sc(1), sc(1), sc(1), sc(2)];
    let o_col = vec![sc(9), sc(9), sc(9), sc(9)];
    let cols = StatementColumns::new(&s_col, &p_col, &o_col)
        .map_err(|e| NounVerbError::execution_error(format!("columns refused: {e:?}")))?;
    // Rule: ∀x (x,1,9) => (x,2,x).
    let rule = Rule {
        rule_id: 1,
        body: vec![BodyAtom::new(var0, sc(1), sc(9))],
        head: [var0, sc(2), var0],
    };
    let compiled = RuleCompiler::compile(&rule)
        .map_err(|e| NounVerbError::execution_error(format!("rule refused: {e:?}")))?;
    let matches = cgap_bn3::unify::unify_batch(&compiled, &cols, 2)
        .map_err(|e| NounVerbError::execution_error(format!("unify refused: {e:?}")))?;
    if matches.len != 3 {
        return fail("bn3 unify", format!("matches {} != 3", matches.len));
    }
    // Final state: each match binds var0 to the subject word of its row.
    for (i, expected) in subjects.iter().enumerate().take(3) {
        let bound = matches.var_values[0][i];
        if bound != s_col[i] || s_col[i] != sc(*expected) {
            return fail(
                "bn3 unify",
                format!("binding {i}: got {bound:#x}, want {:#x}", sc(*expected)),
            );
        }
    }
    Ok(json!({
        "rows": cols.len(),
        "matches": matches.len,
        "bindings_checked": 3,
        "var0_bound_to_subjects": true,
    }))
}

/// `bn3 eval` — real builtin dispatch over the register file: a math sum,
/// a trace write, and the counters latch. Final register state asserted,
/// plus eval-count monotonicity, or refuse typed.
pub fn bn_3_eval(_inputs: Map<String, Value>) -> Result<Value> {
    use cgap_bn3::builtins::{
        dispatch, BuiltinContext, BuiltinPayload, OP_LOG_COUNTERS, OP_LOG_TRACE, OP_MATH_SUM,
    };
    use cgap_bn3::sigil::REGISTER_COUNT;
    let mut regs = [0u64; REGISTER_COUNT];
    regs[1] = 5;
    regs[2] = 7;
    let slab: [u8; 8] = [0; 8];
    let mut ctx = BuiltinContext::default();
    let r0 = dispatch(
        BuiltinPayload::new(OP_MATH_SUM, 0, 1, 2),
        &mut regs,
        &slab,
        &mut ctx,
    )
    .map_err(|e| NounVerbError::execution_error(format!("sum refused: {e:?}")))?;
    if r0 != 12 || regs[0] != 12 {
        return fail("bn3 eval", format!("sum landed {r0}/R0={}", regs[0]));
    }
    let rt = dispatch(
        BuiltinPayload::new(OP_LOG_TRACE, 0, 0, 0),
        &mut regs,
        &slab,
        &mut ctx,
    )
    .map_err(|e| NounVerbError::execution_error(format!("trace refused: {e:?}")))?;
    if ctx.trace.len() != 1 || ctx.trace[0] != rt || rt != 12 {
        return fail("bn3 eval", format!("trace state {:?}", ctx.trace));
    }
    let rc = dispatch(
        BuiltinPayload::new(OP_LOG_COUNTERS, 5, 0, 0),
        &mut regs,
        &slab,
        &mut ctx,
    )
    .map_err(|e| NounVerbError::execution_error(format!("counters refused: {e:?}")))?;
    if rc != ctx.eval_count || regs[5] != 3 {
        return fail(
            "bn3 eval",
            format!("latched {rc} vs count {}", ctx.eval_count),
        );
    }
    Ok(json!({
        "sum": regs[0],
        "trace": ctx.trace,
        "eval_count": ctx.eval_count,
        "counters_latched": regs[5],
    }))
}

fn demo_log(events: usize) -> Result<cgap_audit::ocel2_slab::Ocel2ColumnarLog> {
    use cgap_audit::ocel2_slab::{Ocel2ColumnarLog, OcelEventVector, OCEL2_ATTR_WORDS};
    let mut log = Ocel2ColumnarLog::with_capacity(events, events, 1);
    for i in 0..events as u64 {
        let v = OcelEventVector {
            event_id: i + 1,
            ts_ns: 1_700_000_000_000_000_000 + i,
            activity: (i % 3 + 1) as u16,
            qualifier: 0,
            object_count: 1,
            reserved: 0,
            object_ids: [100, 0, 0, 0, 0, 0, 0, 0],
            attr_blob: [0; OCEL2_ATTR_WORDS],
        };
        log.append(&v)
            .map_err(|e| NounVerbError::execution_error(format!("append refused: {e:?}")))?;
    }
    Ok(log)
}

/// `audit powl-stats` — real POWL conformance over a Sequence(A,B,C)
/// model: conforming 6-event log; construct/binding counts asserted, plus
/// a violating log that must flip the verdict (anti-vacuity), or refuse.
pub fn audit_powl_stats(_inputs: Map<String, Value>) -> Result<Value> {
    use cgap_audit::powl::{verify, PowlModel, PowlNode};
    let log = demo_log(6)?;
    let model = PowlModel {
        root: PowlNode::Operator {
            construct: cgap_audit::powl::PowlConstruct::Sequence,
            children: vec![
                PowlNode::Activity(1),
                PowlNode::Activity(2),
                PowlNode::Activity(3),
            ],
        },
        terminals: vec![3],
    };
    let verdict = verify(&log, &model)
        .map_err(|e| NounVerbError::execution_error(format!("verify refused: {e:?}")))?;
    let conf = verdict
        .map_err(|v| NounVerbError::execution_error(format!("conforming log violated: {v:?}")))?;
    if conf.constructs_checked == 0 || conf.bindings_checked == 0 {
        return fail("audit powl-stats", format!("empty verdict {conf:?}"));
    }
    // Anti-vacuity: a log whose events run (2,1,3) breaks Sequence order
    // (activity 1 must occur before 2 in the binding).
    let bad = {
        use cgap_audit::ocel2_slab::{Ocel2ColumnarLog, OcelEventVector, OCEL2_ATTR_WORDS};
        let mut log = Ocel2ColumnarLog::with_capacity(3, 3, 1);
        for (i, activity) in [2u16, 1, 3].iter().enumerate() {
            let v = OcelEventVector {
                event_id: i as u64 + 1,
                ts_ns: 1_700_000_000_000_000_000 + i as u64,
                activity: *activity,
                qualifier: 0,
                object_count: 1,
                reserved: 0,
                object_ids: [100, 0, 0, 0, 0, 0, 0, 0],
                attr_blob: [0; OCEL2_ATTR_WORDS],
            };
            log.append(&v)
                .map_err(|e| NounVerbError::execution_error(format!("append refused: {e:?}")))?;
        }
        log
    };
    let flipped = verify(&bad, &model);
    let flipped_to_violation = matches!(flipped, Ok(Err(_)));
    if !flipped_to_violation {
        return fail(
            "audit powl-stats",
            "violating log still conformed — vacuous".to_owned(),
        );
    }
    Ok(json!({
        "events": log.event_count(),
        "constructs_checked": conf.constructs_checked,
        "bindings_checked": conf.bindings_checked,
        "conforms": true,
        "violating_log_refused": true,
    }))
}

/// `audit ocel-stats` — real append path stats plus two typed-refusal
/// witnesses (non-monotonic event id; reserved activity 0), or refuse.
pub fn audit_ocel_stats(_inputs: Map<String, Value>) -> Result<Value> {
    use cgap_audit::ocel2_slab::Ocel2Error;
    let log = demo_log(5)?;
    if log.event_count() != 5 {
        return fail("audit ocel-stats", format!("count {}", log.event_count()));
    }
    let total_object_refs: u64 = log.object_count.iter().map(|&c| u64::from(c)).sum();
    if total_object_refs != 5 {
        return fail("audit ocel-stats", format!("refs {total_object_refs} != 5"));
    }
    // Refusal witness 1: event-id regression is typed, never silent.
    // Refusal witness 1: event-id regression is typed, never silent.
    let mut one = demo_log(1)?;
    let refused_id = {
        use cgap_audit::ocel2_slab::{OcelEventVector, OCEL2_ATTR_WORDS};
        let v = OcelEventVector {
            event_id: 1, // == last id: must refuse
            ts_ns: 1,
            activity: 1,
            qualifier: 0,
            object_count: 1,
            reserved: 0,
            object_ids: [100, 0, 0, 0, 0, 0, 0, 0],
            attr_blob: [0; OCEL2_ATTR_WORDS],
        };
        matches!(one.append(&v), Err(Ocel2Error::EventIdNotMonotonic { .. }))
    };
    // Refusal witness 2: activity 0 is reserved.
    let mut one2 = demo_log(1)?;
    let refused_reserved = {
        use cgap_audit::ocel2_slab::{OcelEventVector, OCEL2_ATTR_WORDS};
        let v = OcelEventVector {
            event_id: 99,
            ts_ns: 1,
            activity: 0,
            qualifier: 0,
            object_count: 1,
            reserved: 0,
            object_ids: [100, 0, 0, 0, 0, 0, 0, 0],
            attr_blob: [0; OCEL2_ATTR_WORDS],
        };
        matches!(one2.append(&v), Err(Ocel2Error::MalformedLog(_)))
    };
    if !refused_id || !refused_reserved {
        return fail(
            "audit ocel-stats",
            format!("refusals id={refused_id} reserved={refused_reserved} — vacuous"),
        );
    }
    Ok(json!({
        "events": log.event_count(),
        "object_refs": total_object_refs,
        "monotonic_id_refused": refused_id,
        "reserved_activity_refused": refused_reserved,
    }))
}

/// `bn3 radix-join` — the L3 radix semi-join landing as a CLI verb: the
/// radix probe and the retired nested probe must agree match-for-match
/// (bindings + rows), and a tampered fact column must flip the match count
/// (else the differential is vacuous) — typed refusal on any drift.
pub fn bn_3_radix_join(_inputs: Map<String, Value>) -> Result<Value> {
    use cgap_bn3::sigil::{encode_term, Sigil};
    use cgap_bn3::unify::{
        unify_batch_probe, BodyAtom, CompiledRule, EvalScratch, JoinProbeScratch, MatchSet, Rule,
        RuleCompiler, StatementColumns,
    };
    let sc = |p: u64| encode_term(Sigil::SlotConst, p).expect("payload < 2^56");
    let var0 = encode_term(Sigil::VarUni, 0).expect("reg < 256");
    // Data: (x,1,9) for x in 10..13 (3 matches) plus one non-matching row.
    let s_col: Vec<u64> = [10u64, 11, 12, 13].iter().map(|&x| sc(x)).collect();
    let p_col = vec![sc(1), sc(1), sc(1), sc(2)];
    let o_col = vec![sc(9), sc(9), sc(9), sc(9)];
    let cols = StatementColumns::new(&s_col, &p_col, &o_col)
        .map_err(|e| NounVerbError::execution_error(format!("columns refused: {e:?}")))?;
    let rule = Rule {
        rule_id: 1,
        body: vec![BodyAtom::new(var0, sc(1), sc(9))],
        head: [var0, sc(2), var0],
    };
    let compiled: CompiledRule = RuleCompiler::compile(&rule)
        .map_err(|e| NounVerbError::execution_error(format!("rule refused: {e:?}")))?;
    let run = |cols: &StatementColumns<'_>, radix: bool| -> Result<MatchSet> {
        let compiled_ref = &compiled;
        let mut scratch = EvalScratch::default();
        let mut out = MatchSet::default();
        let mut probe = if radix {
            JoinProbeScratch::default()
        } else {
            JoinProbeScratch {
                radix: false,
                ..Default::default()
            }
        };
        unify_batch_probe(compiled_ref, cols, 2, &mut scratch, &mut out, &mut probe)
            .map_err(|e| NounVerbError::execution_error(format!("probe refused: {e:?}")))?;
        Ok(out)
    };
    let radix = run(&cols, true)?;
    let nested = run(&cols, false)?;
    // Differential court: match count, rows, and bindings must be identical.
    if radix.len != nested.len {
        return fail(
            "bn3 radix-join",
            format!("count drift radix={} nested={}", radix.len, nested.len),
        );
    }
    for m in 0..radix.len {
        if radix.atom_rows[0][m] != nested.atom_rows[0][m]
            || radix.var_values[0][m] != nested.var_values[0][m]
        {
            return fail("bn3 radix-join", format!("match {m} row/binding drift"));
        }
    }
    if radix.len != 3 || radix.var_values[0][0] != s_col[0] {
        return fail(
            "bn3 radix-join",
            format!("expected 3 subject-bound matches, got {}", radix.len),
        );
    }
    // Tamper control (flip witness): flipping one matching object MUST drop
    // the match count; an unchanged count is a vacuous differential.
    let mut o_bad = o_col.clone();
    o_bad[0] = sc(7);
    let cols_bad = StatementColumns::new(&s_col, &p_col, &o_bad)
        .map_err(|e| NounVerbError::execution_error(format!("tampered columns refused: {e:?}")))?;
    let bad = run(&cols_bad, true)?;
    if bad.len != radix.len - 1 {
        return fail(
            "bn3 radix-join",
            format!("tamper did not flip: {} -> {}", radix.len, bad.len),
        );
    }
    Ok(json!({
        "rows": cols.len(),
        "matches": radix.len,
        "nested_matches": nested.len,
        "match_for_match": true,
        "tamper_refused": true,
        "tampered_matches": bad.len,
    }))
}
