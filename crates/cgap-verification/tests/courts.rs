
//! Chicago courts for the Deterministic Verification Surface.
//!
//! Real collaborators only: the real cgap-bont `slot_id` derivation, the
//! real cgap-crypto chatman keyed sealer, real byte assertions. No mocks.

use cgap_verification::ingest::ExtractedTriple;
use cgap_verification::rules::{Rule, RuleKind, RuleMode};
use cgap_verification::run::{refusals, run_verification, GROUNDED_MAGIC};
use cgap_verification::seal::{seal_grounded, verify_sealed, SealError};

fn t(s: &'static [u8], p: &'static [u8], o: &'static [u8], src: u32) -> ExtractedTriple<'static> {
    ExtractedTriple { subject: s, predicate: p, object: o, source: src }
}

fn base_rules() -> Vec<Rule> {
    vec![
        RuleKind::no_self_loop(b"rule/no-self-loop", None),
        RuleKind::no_orphan_objects(b"rule/no-orphans", None, &[b"entity"]),
        RuleKind::domain(b"rule/actor-domain", b"actedIn", &[b"film", b"person", b"review"]),
        RuleKind::range(b"rule/actedIn-range", b"actedIn", &[b"film", b"review"]),
    ]
}

fn clean_corpus() -> Vec<ExtractedTriple<'static>> {
    vec![
        t(b"film", b"type", b"entity", 0),
        t(b"person", b"type", b"entity", 0),
        t(b"review", b"type", b"entity", 0),
        t(b"person", b"actedIn", b"film", 1),
        t(b"film", b"directed", b"review", 2),
    ]
}

const KEY: [u8; 32] = [42u8; 32];

/// COURT 1 - hallucinated-triple refusal: a triple violating a rule yields a
/// typed refusal naming the rule (lexeme + slot) and constraint; a clean
/// corpus yields none; the hallucination moves the seal root.
#[test]
fn court_hallucinated_triple_refusal_names_the_rule() {
    let rules = base_rules();
    let clean = run_verification(&clean_corpus(), &rules, RuleMode::Enabled).expect("ingest");
    assert_eq!(clean.violations_total, 0, "clean corpus must pass every rule");

    // Hallucinated: "phone" (outside the actor domain) actedIn "film".
    let mut hallucinated = clean_corpus();
    hallucinated.push(t(b"phone", b"actedIn", b"film", 3));
    let receipt = run_verification(&hallucinated, &rules, RuleMode::Enabled).expect("ingest");
    assert_eq!(receipt.violations_total, 1);
    let fired = &receipt.results[2];
    assert!(!fired.passed, "domain rule must fail on the hallucinated triple");
    let refs = refusals(&receipt, &hallucinated);
    assert_eq!(refs.len(), 1);
    let r = &refs[0];
    assert_eq!(r.rule_name, b"rule/actor-domain".to_vec());
    assert_eq!(r.constraint, "domain");
    let msg = format!("{r}");
    assert!(msg.contains("rule/actor-domain"), "refusal must name the rule: {msg}");
    assert!(msg.contains("0x"), "refusal must name the rule slot: {msg}");

    // The hallucination must move the seal root (grounding is load-bearing).
    let clean_root = seal_grounded(&KEY, &clean.grounded_context).unwrap().root;
    let bad_root = seal_grounded(&KEY, &receipt.grounded_context).unwrap().root;
    assert_ne!(clean_root, bad_root, "hallucination must move the seal root");
}

/// COURT 2 - grounded-context seal round trip: seal -> verify identical; a
/// wrong key, any single-byte flip, or a length change is typed-loud.
#[test]
fn court_seal_round_trip_and_tamper_flip() {
    let ctx = &run_verification(&clean_corpus(), &base_rules(), RuleMode::Enabled)
        .expect("ingest")
        .grounded_context;
    assert_eq!(&ctx[0..4], &GROUNDED_MAGIC);
    let sealed = seal_grounded(&KEY, ctx).unwrap();
    verify_sealed(&KEY, ctx, &sealed).expect("round trip must verify identical");
    verify_sealed(&[43u8; 32], ctx, &sealed).unwrap_err();
    for i in [0usize, 17, ctx.len() / 2, ctx.len() - 1] {
        let mut tampered = ctx.clone();
        tampered[i] ^= 0x40;
        match verify_sealed(&KEY, &tampered, &sealed) {
            Err(SealError::Tampered { .. }) => {}
            other => panic!("byte {i} flip must be typed Tampered, got {other:?}"),
        }
    }
    let truncated = &ctx[..ctx.len() - 1];
    match verify_sealed(&KEY, truncated, &sealed) {
        Err(SealError::LengthMismatch { .. }) => {}
        other => panic!("truncation must be typed LengthMismatch, got {other:?}"),
    }
}

/// COURT 2b - the sealer handles multi-chunk (> 1024 B) grounded contexts.
#[test]
fn court_seal_multi_chunk() {
    let mut big = clean_corpus();
    for i in 0..200u32 {
        big.push(ExtractedTriple {
            subject: b"bulk",
            predicate: b"idx",
            object: Box::leak(format!("item{i}").into_boxed_str()).as_bytes(),
            source: i,
        });
    }
    let ctx = &run_verification(&big, &base_rules(), RuleMode::Enabled)
        .expect("ingest")
        .grounded_context;
    assert!(ctx.len() > 1024, "corpus must produce a multi-chunk context");
    let sealed = seal_grounded(&KEY, ctx).unwrap();
    assert!(sealed.leaves >= 2);
    verify_sealed(&KEY, ctx, &sealed).expect("multi-chunk round trip");
    let mut mid = ctx.clone();
    let i = ctx.len() / 2;
    mid[i] ^= 0x20;
    assert!(matches!(verify_sealed(&KEY, &mid, &sealed), Err(SealError::Tampered { .. })));
}

/// COURT 3 - anti-vacuity (in-process half): `RuleMode::Disabled` seals the
/// hallucinated corpus clean; `Enabled` fires. Rule evaluation is
/// load-bearing, not decorative.
#[test]
fn court_rule_evaluation_is_load_bearing() {
    let rules = base_rules();
    let mut hallucinated = clean_corpus();
    hallucinated.push(t(b"phone", b"actedIn", b"film", 3));
    let disabled = run_verification(&hallucinated, &rules, RuleMode::Disabled).expect("ingest");
    assert_eq!(disabled.violations_total, 0, "Disabled mode seals hallucinations clean");
    assert!(seal_grounded(&KEY, &disabled.grounded_context).is_ok());
    let enabled = run_verification(&hallucinated, &rules, RuleMode::Enabled).expect("ingest");
    assert_eq!(enabled.violations_total, 1, "Enabled mode must fire on the hallucination");
    assert_ne!(
        enabled.grounded_context, disabled.grounded_context,
        "enabled mode must change the grounded context (evaluation is real)"
    );
}

/// COURT 4 - grounded-context determinism: identical corpus+rules -> byte-
/// identical context; reordered ingestion -> identical bytes; one changed
/// lexeme -> different bytes (the encoder is not vacuous).
#[test]
fn court_grounded_context_determinism() {
    let rules = base_rules();
    let a = run_verification(&clean_corpus(), &rules, RuleMode::Enabled).unwrap();
    let b = run_verification(&clean_corpus(), &rules, RuleMode::Enabled).unwrap();
    assert_eq!(a.grounded_context, b.grounded_context);

    let mut reordered = clean_corpus();
    reordered.reverse();
    let c = run_verification(&reordered, &rules, RuleMode::Enabled).unwrap();
    assert_eq!(a.grounded_context, c.grounded_context, "context must be ingestion-order independent");

    let mut swapped = clean_corpus();
    swapped.push(t(b"person", b"actedIn", b"review", 4));
    let d = run_verification(&swapped, &rules, RuleMode::Enabled).unwrap();
    assert_ne!(a.grounded_context, d.grounded_context);
}

/// COURT 5 - ingestion SlotId law: terms derive through the real cgap-bont
/// `slot_id`; the registry names slots; distinct lexemes never share slots.
#[test]
fn court_ingestion_slot_law() {
    let corpus = cgap_verification::ingest::ingest(&clean_corpus()).unwrap();
    assert_eq!(corpus.triples.len(), clean_corpus().len());
    let slot = cgap_verification::ingest::ingest(&[t(b"phone", b"actedIn", b"film", 0)])
        .unwrap()
        .lexeme(cgap_bont::primitives::slot_id(b"phone"))
        .expect("registry must name the phone slot")
        .to_vec();
    assert_eq!(slot, b"phone".to_vec());
    // Distinct lexemes must not collide across this corpus (exercising the
    // CollisionDetector on every term).
    // Distinct lexemes must not share slots: bucket every term lexeme by its
    // derived slot and assert one lexeme per slot.
    let mut by_slot: std::collections::BTreeMap<u64, std::collections::BTreeSet<Vec<u8>>> = Default::default();
    for triple in &clean_corpus() {
        for lexeme in [triple.subject, triple.predicate, triple.object] {
            by_slot.entry(cgap_bont::primitives::slot_id(lexeme).0)
                .or_default()
                .insert(lexeme.to_vec());
        }
    }
    for (slot, lexemes) in &by_slot {
        assert_eq!(lexemes.len(), 1, "slot 0x{slot:016x} claimed by {} distinct lexemes", lexemes.len());
    }
}

/// COURT 6 - custom rules as data: a Custom rule (no predicate filter +
/// forbid_self_loop + exemptions) behaves exactly like the built-ins, and
/// the orphan rule's exemption list is honored.
#[test]
fn court_custom_rule_as_data_and_orphan_exemption() {
    let custom = vec![Rule {
        name: b"rule/custom-all-loops".to_vec(),
        kind: RuleKind {
            predicate: None,
            domain: None,
            range: None,
            forbid_self_loop: true,
            no_orphans: false,
            orphan_exemptions: vec![],
        },
    }];
    let mut corpus = clean_corpus();
    corpus.push(t(b"loop", b"knows", b"loop", 9));
    let r = run_verification(&corpus, &custom, RuleMode::Enabled).unwrap();
    assert_eq!(r.violations_total, 1, "custom rule must catch the self-loop");

    // Orphan exemption honored: "exempt-term" as object of a matching triple.
    let orphan_rules = vec![RuleKind::no_orphan_objects(b"rule/no-orphans", None, &[b"entity"])];
    let r2 = run_verification(&clean_corpus(), &orphan_rules, RuleMode::Enabled).unwrap();
    assert_eq!(r2.violations_total, 0, "entity objects are exempt; film/review occur as subjects");
    let mut orphaned = clean_corpus();
    orphaned.push(t(b"person", b"mentions", b"ghost", 5));
    let r3 = run_verification(&orphaned, &orphan_rules, RuleMode::Enabled).unwrap();
    assert_eq!(r3.violations_total, 1, "dangling object must be an orphan violation");
}
