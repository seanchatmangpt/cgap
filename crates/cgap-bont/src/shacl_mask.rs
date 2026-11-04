//! Columnar SHACL kernels over slot columns (operator doctrine §1.C
//! reductions, lane p1-shacl, v26.11.2). No SPARQL strings: a shape's
//! cardinality and allowed-values constraints compile to a mask plan
//! evaluated word-wise over `slot_s`/`slot_p`/`slot_o` columns.
//!
//! Constraints (§1.C):
//! - `sh:minCount`/`sh:maxCount` -> group-by `slot_s` + count filter on
//!   statements matching the predicate slot.
//! - `sh:datatype`/`sh:in` -> vectorized membership bitmask on `slot_o`.
//! - `sh:hasValue` -> word-wise equality filter on `(slot_p, slot_o)` pairs.
//!
//! Identity is [`SlotId`] from `primitives::slot_id` (A06); the corpus
//! constant `CORPUS_SEED` seeds the court/bench corpora (frozen seed law —
//! the corpus seed never substitutes for the SplitMix64 algorithm
//! multiplier `0xBF58_476D_1CE4_E5B9`, and vice versa; swapping them
//! breaks replay — bitslice.rs:687-699 precedent).
#![forbid(unsafe_code)]

use crate::primitives::{slot_id, SlotId};
use std::collections::{HashMap, HashSet};

/// Corpus seed for court/bench corpora (frozen pin: corpus seed).
pub const CORPUS_SEED: u64 = 0xA20C_0FFE_E5A7_E000;

/// SplitMix64 finalizer multiplier — the ALGORITHM constant (frozen pin;
/// never conflated with `CORPUS_SEED`).
const SPLITMIX_MULT: u64 = 0xBF58_476D_1CE4_E5B9;

/// Deterministic corpus RNG: SplitMix64 seeded from [`CORPUS_SEED`].
pub struct CorpusRng(u64);

impl CorpusRng {
    /// New generator from a seed (courts use `CORPUS_SEED`).
    pub fn new(seed: u64) -> Self {
        CorpusRng(seed)
    }

    /// Next raw word.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(SPLITMIX_MULT);
        z = (z ^ (z >> 27)).wrapping_mul(SPLITMIX_MULT);
        z ^ (z >> 31)
    }

    /// Uniform-ish value in `0..n` (`n > 0`).
    pub fn below(&mut self, n: u64) -> u64 {
        self.next_u64() % n
    }
}

/// `slot_id` of the rdf:type IRI (computed, not a const: `slot_id` hashes).
fn rdf_type_slot() -> SlotId {
    slot_id(b"http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
}

/// One compiled constraint in the mask plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constraint {
    /// `sh:minCount`/`sh:maxCount`: statements with `pred` per subject must
    /// be `>= min` and `<= max`.
    Cardinality {
        /// Predicate slot.
        pred: SlotId,
        /// Minimum count (0 = no floor).
        min: u64,
        /// Maximum count (`None` = no ceiling).
        max: Option<u64>,
    },
    /// `sh:datatype`/`sh:in`: every statement with `pred` (on an in-scope
    /// subject) must have `slot_o` in `allowed`.
    Membership {
        /// Predicate slot.
        pred: SlotId,
        /// Allowed object slots.
        allowed: Vec<SlotId>,
    },
    /// `sh:hasValue`: every in-scope subject must carry at least one
    /// `(pred, value)` pair.
    HasValue {
        /// Predicate slot.
        pred: SlotId,
        /// Required object slot.
        value: SlotId,
    },
}

/// One compiled shape: target class + constraint list. Subjects are in
/// scope iff they carry `(rdf:type, target_class)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapePlan {
    /// Target class slot (scope selector).
    pub target_class: SlotId,
    /// Compiled constraints, evaluated in order.
    pub constraints: Vec<Constraint>,
}

/// One (shape, subject, constraint) verdict triple.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Violation {
    /// Index into [`MaskPlan::shapes`].
    pub shape_idx: usize,
    /// Violating subject slot.
    pub subject: SlotId,
    /// Index into [`ShapePlan::constraints`].
    pub constraint_idx: usize,
}

/// A compiled mask plan: the whole shape set, evaluated over three aligned
/// slot columns `(slot_s, slot_p, slot_o)` in one pass per shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaskPlan {
    /// Compiled shapes.
    pub shapes: Vec<ShapePlan>,
}

impl MaskPlan {
    /// Compile a mask plan from shape plans (identity: the plan is exactly
    /// its input shape list; determinism: violations are sorted).
    pub fn new(shapes: Vec<ShapePlan>) -> Self {
        MaskPlan { shapes }
    }

    /// Vectorized evaluation over aligned columns. Unequal column lengths
    /// are a caller-contract breach and panic loudly (never a silent
    /// truncation) — the court corpus pins this.
    pub fn evaluate(&self, s: &[SlotId], p: &[SlotId], o: &[SlotId]) -> Vec<Violation> {
        assert_eq!(
            s.len(),
            p.len(),
            "slot columns must be aligned (s.len == p.len)"
        );
        assert_eq!(
            p.len(),
            o.len(),
            "slot columns must be aligned (p.len == o.len)"
        );
        let type_slot = rdf_type_slot();
        let mut out: Vec<Violation> = Vec::new();
        for (shape_idx, shape) in self.shapes.iter().enumerate() {
            // Scope kernel: subjects carrying (rdf:type, target_class).
            let mut scope: HashSet<SlotId> = HashSet::new();
            for i in 0..s.len() {
                if p[i] == type_slot && o[i] == shape.target_class {
                    scope.insert(s[i]);
                }
            }
            if scope.is_empty() {
                continue;
            }
            for (ci, c) in shape.constraints.iter().enumerate() {
                match c {
                    Constraint::Cardinality { pred, min, max } => {
                        // Group-by slot_s + count filter (one column pass).
                        let mut counts: HashMap<SlotId, u64> = HashMap::new();
                        for i in 0..s.len() {
                            if p[i] == *pred {
                                *counts.entry(s[i]).or_insert(0) += 1;
                            }
                        }
                        for subject in &scope {
                            let n = counts.get(subject).copied().unwrap_or(0);
                            let bad = n < *min || matches!(max, Some(m) if n > *m);
                            if bad {
                                out.push(Violation {
                                    shape_idx,
                                    subject: *subject,
                                    constraint_idx: ci,
                                });
                            }
                        }
                    }
                    Constraint::Membership { pred, allowed } => {
                        // Membership bitmask: linear scan over the (small)
                        // allowed set, vectorized over the column.
                        for i in 0..s.len() {
                            if p[i] == *pred && scope.contains(&s[i]) && !allowed.contains(&o[i]) {
                                out.push(Violation {
                                    shape_idx,
                                    subject: s[i],
                                    constraint_idx: ci,
                                });
                            }
                        }
                    }
                    Constraint::HasValue { pred, value } => {
                        // Word equality filter on (slot_p, slot_o) pairs.
                        let mut satisfied: HashSet<SlotId> = HashSet::new();
                        for i in 0..s.len() {
                            if p[i] == *pred && o[i] == *value {
                                satisfied.insert(s[i]);
                            }
                        }
                        for subject in &scope {
                            if !satisfied.contains(subject) {
                                out.push(Violation {
                                    shape_idx,
                                    subject: *subject,
                                    constraint_idx: ci,
                                });
                            }
                        }
                    }
                }
            }
        }
        out.sort_unstable_by_key(|v| (v.shape_idx, v.constraint_idx, v.subject.0));
        out
    }
}

/// Oracle court kernel: brute-force per-subject counting. O(subjects x
/// statements) nested scans, no group-by maps — an independent path to the
/// same accept/reject sets. `Violation` ordering is identical to
/// [`MaskPlan::evaluate`] (sorted) so equality is exact.
pub fn brute_force(plan: &MaskPlan, s: &[SlotId], p: &[SlotId], o: &[SlotId]) -> Vec<Violation> {
    assert_eq!(
        s.len(),
        p.len(),
        "slot columns must be aligned (s.len == p.len)"
    );
    assert_eq!(
        p.len(),
        o.len(),
        "slot columns must be aligned (p.len == o.len)"
    );
    let type_slot = rdf_type_slot();
    // Subject universe: sorted dedup of slot_s (deterministic order).
    let mut subjects: Vec<SlotId> = s.to_vec();
    subjects.sort_unstable_by_key(|x| x.0);
    subjects.dedup();
    let mut out: Vec<Violation> = Vec::new();
    for (shape_idx, shape) in plan.shapes.iter().enumerate() {
        for (si, &subject) in subjects.iter().enumerate() {
            // Naive scope test: full-column linear scan per subject.
            let in_scope = s
                .iter()
                .zip(p.iter())
                .zip(o.iter())
                .any(|((&ss, &pp), &oo)| {
                    ss == subject && pp == type_slot && oo == shape.target_class
                });
            if !in_scope {
                continue;
            }
            for (ci, c) in shape.constraints.iter().enumerate() {
                let count = |pred: SlotId| -> u64 {
                    s.iter()
                        .zip(p.iter())
                        .filter(|&(&ss, &pp)| ss == subject && pp == pred)
                        .count() as u64
                };
                let pair_exists = |pred: SlotId, val: SlotId| -> bool {
                    s.iter()
                        .zip(p.iter())
                        .zip(o.iter())
                        .any(|((&ss, &pp), &oo)| ss == subject && pp == pred && oo == val)
                };
                match c {
                    Constraint::Cardinality { pred, min, max } => {
                        let n = count(*pred);
                        if n < *min || matches!(max, Some(m) if n > *m) {
                            out.push(Violation {
                                shape_idx,
                                subject,
                                constraint_idx: ci,
                            });
                        }
                    }
                    Constraint::Membership { pred, allowed } => {
                        // Each matching statement checked independently.
                        for i in 0..s.len() {
                            if s[i] == subject && p[i] == *pred && !allowed.contains(&o[i]) {
                                out.push(Violation {
                                    shape_idx,
                                    subject,
                                    constraint_idx: ci,
                                });
                            }
                        }
                    }
                    Constraint::HasValue { pred, value } => {
                        if !pair_exists(*pred, *value) {
                            out.push(Violation {
                                shape_idx,
                                subject,
                                constraint_idx: ci,
                            });
                        }
                    }
                }
                let _ = si;
            }
        }
    }
    out.sort_unstable_by_key(|v| (v.shape_idx, v.constraint_idx, v.subject.0));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shacl_compiler::mask_plan_from_ttl;

    fn sid(b: &[u8]) -> SlotId {
        slot_id(b)
    }

    fn person_class() -> SlotId {
        sid(b"urn:ex#Person")
    }

    fn pred_name() -> SlotId {
        sid(b"urn:ex#name")
    }

    fn pred_age() -> SlotId {
        sid(b"urn:ex#age")
    }

    fn pred_flag() -> SlotId {
        sid(b"urn:ex#premium")
    }

    /// Court corpus: 4 typed-Person subjects. Violations under
    /// [`court_plan`]: S3+S4 fail the name floor, S3 fails age membership,
    /// S1+S2+S3 miss the premium hasValue. Deterministic, no RNG.
    fn court_corpus() -> (Vec<SlotId>, Vec<SlotId>, Vec<SlotId>) {
        let ty = rdf_type_slot();
        let cls = person_class();
        let mut s: Vec<SlotId> = Vec::new();
        let mut p: Vec<SlotId> = Vec::new();
        let mut o: Vec<SlotId> = Vec::new();
        let mut push = |ss: SlotId, pp: SlotId, oo: SlotId| {
            s.push(ss);
            p.push(pp);
            o.push(oo);
        };
        let s1 = sid(b"urn:ex#s1");
        let s2 = sid(b"urn:ex#s2");
        let s3 = sid(b"urn:ex#s3");
        let s4 = sid(b"urn:ex#s4");
        for &subj in &[s1, s2, s3, s4] {
            push(subj, ty, cls);
        }
        push(s1, pred_name(), sid(b"urn:ex#n1"));
        push(s2, pred_name(), sid(b"urn:ex#n2"));
        push(s3, pred_age(), sid(b"urn:ex#badAge"));
        push(s4, pred_flag(), sid(b"urn:ex#ok"));
        (s, p, o)
    }

    /// Plan: name exactly-once, age slot membership, premium hasValue.
    fn court_plan() -> MaskPlan {
        MaskPlan::new(vec![ShapePlan {
            target_class: person_class(),
            constraints: vec![
                Constraint::Cardinality {
                    pred: pred_name(),
                    min: 1,
                    max: Some(1),
                },
                Constraint::Membership {
                    pred: pred_age(),
                    allowed: vec![sid(b"urn:ex#ageSlot")],
                },
                Constraint::HasValue {
                    pred: pred_flag(),
                    value: sid(b"urn:ex#ok"),
                },
            ],
        }])
    }

    /// Fuzz columns from a small vocabulary, SplitMix64-seeded
    /// (CORPUS_SEED; frozen seed law).
    fn fuzz_cols(n: usize, trial: u64) -> (Vec<SlotId>, Vec<SlotId>, Vec<SlotId>) {
        let mut rng = CorpusRng::new(CORPUS_SEED ^ trial);
        let vocab: Vec<SlotId> = (0..16u64)
            .map(|i| sid(format!("urn:v#{}", i).as_bytes()))
            .collect();
        let ty = rdf_type_slot();
        let mut s = Vec::new();
        let mut p = Vec::new();
        let mut o = Vec::new();
        for _ in 0..n {
            s.push(SlotId(rng.next_u64() | 1));
            let pick = rng.below(3);
            if pick == 0 {
                p.push(ty);
                let v = vocab[rng.below(vocab.len() as u64) as usize];
                o.push(v);
            } else {
                p.push(vocab[rng.below(vocab.len() as u64) as usize]);
                o.push(vocab[rng.below(vocab.len() as u64) as usize]);
            }
        }
        (s, p, o)
    }

    fn fuzz_plan() -> MaskPlan {
        let shapes = (0..8usize)
            .map(|i| ShapePlan {
                target_class: sid(format!("urn:v#{}", i).as_bytes()),
                constraints: vec![
                    Constraint::Cardinality {
                        pred: sid(format!("urn:v#{}", (i + 1) % 16).as_bytes()),
                        min: 1,
                        max: Some(3),
                    },
                    Constraint::Membership {
                        pred: sid(format!("urn:v#{}", (i + 2) % 16).as_bytes()),
                        allowed: vec![sid(b"urn:v#0"), sid(b"urn:v#1"), sid(b"urn:v#2")],
                    },
                    Constraint::HasValue {
                        pred: sid(format!("urn:v#{}", (i + 3) % 16).as_bytes()),
                        value: sid(b"urn:v#5"),
                    },
                ],
            })
            .collect();
        MaskPlan::new(shapes)
    }

    #[test]
    fn t1_oracle_parity_court_corpus() {
        let (s, p, o) = court_corpus();
        let plan = court_plan();
        let got = plan.evaluate(&s, &p, &o);
        let want = brute_force(&plan, &s, &p, &o);
        assert_eq!(got, want, "kernel vs brute-force oracle must agree exactly");
        assert_eq!(
            got.len(),
            6,
            "S3+S4 floor, S3 membership, S1+S2+S3 hasValue: {:?}",
            got
        );
    }

    #[test]
    fn t2_oracle_parity_fuzz() {
        for trial in 0..5u64 {
            let (s, p, o) = fuzz_cols(512, trial);
            let plan = fuzz_plan();
            let got = plan.evaluate(&s, &p, &o);
            let want = brute_force(&plan, &s, &p, &o);
            assert_eq!(got, want, "fuzz trial {}", trial);
        }
    }

    #[test]
    fn t3_min_count_mutation_flips_verdict() {
        // Per-shape mutation flip: raising minCount 1 -> 2 must change the
        // accept/reject set (S1 had exactly one name: becomes a violation).
        let (s, p, o) = court_corpus();
        let mut mutated = court_plan();
        if let Constraint::Cardinality { min, .. } = &mut mutated.shapes[0].constraints[0] {
            *min = 2;
        }
        let before = court_plan().evaluate(&s, &p, &o);
        let after = mutated.evaluate(&s, &p, &o);
        assert_ne!(before, after, "minCount 1->2 must flip the verdict set");
    }

    #[test]
    fn t4_mask_corruption_flips_verdict() {
        // Corrupt the membership mask: allowed set widened to accept the
        // bad age slot -> verdict set must shrink (flip).
        let (s, p, o) = court_corpus();
        let mut corrupted = court_plan();
        if let Constraint::Membership { allowed, .. } = &mut corrupted.shapes[0].constraints[1] {
            allowed.push(sid(b"urn:ex#badAge"));
        }
        let before = court_plan().evaluate(&s, &p, &o);
        let after = corrupted.evaluate(&s, &p, &o);
        assert_ne!(before, after, "mask corruption must flip the verdict set");
    }

    #[test]
    fn t5_ttl_bridge_end_to_end() {
        let ttl = "@prefix sh: <http://www.w3.org/ns/shacl#> .\n\
@prefix ex: <urn:ex#> .\n\
ex:PersonShape a sh:NodeShape ;\n  sh:closed true ;\n\
  sh:targetClass ex:Person ;\n  sh:property [\n    \
sh:path ex:name ; sh:minCount 1 ; sh:maxCount 2 ;\n  ] .\n";
        let plan = mask_plan_from_ttl(ttl).expect("ttl bridge must compile");
        assert_eq!(plan.shapes.len(), 1);
        assert_eq!(plan.shapes[0].constraints.len(), 1);
        assert_eq!(
            plan.shapes[0].constraints[0],
            Constraint::Cardinality {
                pred: sid(b"urn:ex#name"),
                min: 1,
                max: Some(2),
            }
        );
        let (s, p, o) = court_corpus();
        let got = plan.evaluate(&s, &p, &o);
        let want = brute_force(&plan, &s, &p, &o);
        assert_eq!(got, want);
        assert_eq!(got.len(), 2, "S3+S4 (zero names) violate: {:?}", got);
    }

    /// Bench court (TARGET-v26.11.2 until Divan-measured): shape-evals/s
    /// over 65536 statements x 8 shapes. Timing is reported, never asserted
    /// (a time bound would be flake-bait; correctness is asserted).
    #[test]
    fn t6_bench_shape_evals_per_s() {
        let (s, p, o) = fuzz_cols(65536, 0);
        let plan = fuzz_plan();
        // Warm pass + correctness pin: identical to the oracle on the
        // first 256 rows is not claimed here; the oracle is O(n^2) over
        // 65536 rows, so parity at scale is pinned by t2 (512-row fuzz).
        let warm = plan.evaluate(&s, &p, &o);
        let warm2 = plan.evaluate(&s, &p, &o);
        assert_eq!(warm, warm2, "determinism across repeated evaluation");
        let rounds = 10;
        let t0 = std::time::Instant::now();
        let mut sink = 0usize;
        for _ in 0..rounds {
            sink += plan.evaluate(&s, &p, &o).len();
        }
        let dt = t0.elapsed();
        let evals = (rounds * plan.shapes.len()) as f64;
        let per_s = evals / dt.as_secs_f64();
        println!(
            "p1-shacl bench: {} shape-evals/s over 65536 statements x 8 shapes \
             (total {:?}, sink {})",
            per_s as u64, dt, sink
        );
        assert!(sink > 0 || warm.is_empty());
    }
}
