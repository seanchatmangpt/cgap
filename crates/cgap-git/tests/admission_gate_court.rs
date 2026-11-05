//! Lane GIT-09 mutation court: pre-receive admission gate.
//!
//! Real collaborators only, final-state asserts, no mocks. The gate is
//! included by `#[path]` (same pattern as pkt_line_court.rs) because
//! lib.rs is outside this lane's file lease; the gate composes the real
//! GIT-02 pkt-line codec by `#[path]` from src/.
//!
//! Anti-vacuity: every refusal court names its flip twin — the mutated
//! gate behavior (always-admit / skip-closure / accept-unbound-head)
//! that would make the court FAIL. A court that cannot flip is FAIL.

#[path = "../src/admission_gate.rs"]
mod admission_gate;

use admission_gate::{
    admit_update, parse_update_line, parse_update_pkt, GateError, ProposedUpdate, RefTable,
    RefUpdate,
};

/// Old/new SHAs as hex words (the court's wire fixtures).
const OLD_HEX: &str = "1111111111111111111111111111111111111111";
const NEW_HEX: &str = "2222222222222222222222222222222222222222";

fn fixture_update() -> RefUpdate {
    parse_update_line(format!("{OLD_HEX} {NEW_HEX} refs/heads/main").as_bytes())
        .expect("fixture line parses")
}

#[test]
fn court_1_sound_update_admits_and_ref_advances() {
    let req = fixture_update();
    // Sound derivation: parent facts + a converging transitive-closure
    // rule. Termination is structural (finite term domain, monotone).
    let mut u = ProposedUpdate {
        facts: vec![(1, 900, 2), (2, 900, 3)],
        ..Default::default()
    };
    u.rules.push(admission_gate::GateRule {
        rule_id: 1,
        body: vec![(admission_gate::var(0), 900, admission_gate::var(1))],
        head: (admission_gate::var(0), 900, admission_gate::var(1)),
    });
    let mut table = RefTable::new();
    table.set(b"refs/heads/main", req.old_sha);
    let before = table.bytes();
    table
        .apply(&req, &u)
        .unwrap_or_else(|e| panic!("sound update must admit, got {e}"));
    assert_eq!(table.get(b"refs/heads/main"), Some(req.new_sha));
    assert_ne!(
        table.bytes(),
        before,
        "admitted update must mutate the table"
    );
}

#[test]
fn court_2_cycle_refuses_and_ref_table_byte_unchanged() {
    // Non-converging derivation: a 600-node chain under a transitive
    // closure rule derives ~180k facts, blowing the 4,096-fact budget —
    // the bounded-Datalog signature of an introduced derivation cycle.
    let req = fixture_update();
    let mut u = ProposedUpdate::default();
    for i in 0u64..600 {
        u.facts.push((i, 900, i + 1));
    }
    u.rules.push(admission_gate::GateRule {
        rule_id: 7,
        body: vec![
            (admission_gate::var(0), 900, admission_gate::var(1)),
            (admission_gate::var(1), 900, admission_gate::var(2)),
        ],
        head: (admission_gate::var(0), 900, admission_gate::var(2)),
    });
    let mut table = RefTable::new();
    table.set(b"refs/heads/main", req.old_sha);
    let before = table.bytes();
    let err = table
        .apply(&req, &u)
        .expect_err("cycle must be refused, not admitted");
    assert!(
        matches!(err, GateError::CyclicDerivation { rounds: 3, .. }),
        "expected CyclicDerivation at round 3, got {err:?}"
    );
    assert_eq!(
        table.bytes(),
        before,
        "refused update must leave the ref table byte-identical"
    );
}

#[test]
fn court_3_contradiction_refuses_and_ref_table_byte_unchanged() {
    // Deductive contradiction: functional predicate 0xF00D carries two
    // distinct objects for one (s, p) — unsatisfiable under the declared
    // ontology. Flip twin: a gate that skips the functional-predicate law
    // (or a proposal that drops the declaration) admits here.
    let req = fixture_update();
    let mut u = ProposedUpdate {
        facts: vec![(5, 0xF00D, 6), (5, 0xF00D, 7)],
        ..Default::default()
    };
    u.functional_predicates.insert(0xF00D);
    let mut table = RefTable::new();
    table.set(b"refs/heads/main", req.old_sha);
    let before = table.bytes();
    let err = table
        .apply(&req, &u)
        .expect_err("contradiction must be refused");
    assert_eq!(
        err,
        GateError::DeductiveContradiction {
            s: 5,
            p: 0xF00D,
            a: 6,
            b: 7
        },
        "expected the typed contradiction refusal, got {err:?}"
    );
    assert_eq!(table.bytes(), before, "table byte-unchanged after refusal");
}

#[test]
fn court_4_unbound_head_variable_is_ontology_broken() {
    // Flip twin: a gate that accepts head-only variables (bn3's
    // UnboundHeadVariable law dropped) admits here.
    let req = fixture_update();
    let mut u = ProposedUpdate {
        facts: vec![(1, 900, 2)],
        ..Default::default()
    };
    u.rules.push(admission_gate::GateRule {
        rule_id: 9,
        body: vec![(admission_gate::var(0), 900, 2)],
        head: (admission_gate::var(0), 900, admission_gate::var(5)), // var 5 unbound
    });
    let mut table = RefTable::new();
    table.set(b"refs/heads/main", req.old_sha);
    let err = table
        .apply(&req, &u)
        .expect_err("unbound head var must refuse");
    assert!(
        matches!(
            err,
            GateError::OntologyRuleBroken {
                rule_id: 9,
                reason: admission_gate::OntologyRefusal::UnboundHeadVariable
            }
        ),
        "expected OntologyRuleBroken(UnboundHeadVariable), got {err:?}"
    );
}

#[test]
fn court_5_wire_roundtrip_through_real_pkt_codec() {
    // Real collaborators end to end: build a pkt-line framed command,
    // parse it back through the gate, and admit. Flip twin: a mutated
    // parser that skips hex validation accepts a 'g' digit SHA — court 6.
    let req = fixture_update();
    let mut wire: Vec<u8> = Vec::new();
    let mut line = format!("{OLD_HEX} {NEW_HEX} refs/heads/main");
    // pkt-line framing: 4 uppercase-hex total length + payload.
    let total = 4 + line.len();
    wire.extend_from_slice(format!("{total:04X}").as_bytes());
    wire.extend_from_slice(line.as_bytes());
    line.clear();
    let parsed = parse_update_pkt(&wire).expect("framed command parses");
    assert_eq!(parsed, req);
    let u = ProposedUpdate {
        facts: vec![(1, 900, 2)],
        ..Default::default()
    };
    admit_update(&parsed, &u).expect("sound parsed update admits");
}

#[test]
fn court_6_bad_hex_sha_refused_typed() {
    let err = parse_update_line(b"zz11.. .. refs/heads/x").expect_err("non-hex sha must refuse");
    assert!(
        matches!(err, GateError::BadHexSha { .. }),
        "expected BadHexSha, got {err:?}"
    );
    // Malformed field count refuses typed too.
    let err2 = parse_update_line(b"only-one-field").expect_err("1 field must refuse");
    assert_eq!(err2, GateError::MalformedUpdateLine { fields: 1 });
}
