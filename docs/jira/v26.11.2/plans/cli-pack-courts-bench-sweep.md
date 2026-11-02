# Plans — CLI wedge, cgap-pack, courts/bench/sweep docs, OST register, wave-doc gaps

Provenance: Plan wave lanes CLI/PACK/COURTS/BENCH/SWEEP-DOC/OST-REG/WAVES + Explore lane
W1-18. On-disk-grounded; re-read at use time.

## CLI verify-context wedge (W5)

Ground truth (W1-18 brief): custom_handlers has 21 implemented verbs, zero todo!/unimplemented!
(rss:17-200 + handlers_{kernel,reasoning,sovereign}); uniform flip-witness pattern (mutate →
assert typed refusal else "vacuous gate" fail — e.g. parquet_pack_check k:147-151,
crypto_receipt_verify s:161-166). Ontology: crates/cli/ontology/cgap-cli.ttl — 15 nouns
(ttl:10-13), 21 cnv:Command individuals; new-verb block shape = 5-line cnv:Command +
one-line `cnv:XBehavior a cnv:CustomBehavior .` (ttl:20-26), handler fn = snake_case of
behavior IRI, re-exported (rs:122-130).

crates/cgap-verification exports everything needed (lib.rs:23-26): ingest(&[ExtractedTriple])
-> IngestedCorpus (ingest.rs:66; ExtractedTriple{subject,predicate,object:&[u8],source}
:14-20); Rule::{domain,range,no_self_loop,no_orphan_objects,custom} (rules.rs:62-123);
run_verification(triples, rules, RuleMode) -> GroundedReceipt (run.rs:157-161);
refusals(receipt, triples) -> Vec<Refusal{rule,rule_name,triple_index,constraint}> (run.rs:192,
:36-41); seal_receipt(receipt, &[u8;32]) -> SealedContext (run.rs:216-219; verify_sealed
seal.rs:93 for the flip witness).

Plan:
1. Engine gap: add RuleKind::same_as / Rule::same_as(name, predicate) constructor (flag bit
   16 in encode_context), evaluating symmetric-pair contradictions; violations flow through
   existing Violation/Refusal machinery.
2. Ontology: bump cnv:version; add cnv:VerifyNoun + cnv:VerifyContext / cnv:VerifyRefusals
   commands as cnv:CustomBehavior individuals (exact 5-line block shape).
3. Regen: extend manifest_patch.py KERNELS with cgap-verification path dep (anchor
   `blake3 = "1.8"` unchanged); rm Cargo.toml → ggen sync run → manifest_patch.py.
4. handlers_verify.rs: verify_context (parse triples → rule set = no_self_loop,
   no_orphan_objects, same_as, domain, range → run_verification(Enabled) → on violations
   Err(execution_error("refused: rule '<name>' (<constraint>), triple #<n>")) → exit 1; on
   pass seal_receipt with derived 32-byte key → JSON {sealed, violations_total: 0});
   verify_refusals (Enabled vs Disabled comparison).
5. Anti-vacuity flips: hallucinated triple (x,knows,x) → no_self_loop typed refusal exit 1;
   orphan object → no_orphans; same-as contradiction → refusal naming same_as;
   RuleMode::Disabled seals clean — proves rules actually gate (run.rs:137-141 is the
   documented mutation target).
6. Tests in tests/zero_code_cli.rs (TestWorkspace::run pattern): pass-seals-receipt,
   hallucinated-typed-refusal, same-as-contradiction, disabled-mode-anti-vacuity,
   refusals-reports-typed-names. Update docs/command-reference.md + verb count 21 → 23.

## cgap-pack lever extension (W1-04/W3-19)

Pack /Users/sac/ggen-marketplace/packs/cgap-pack v26.10.29: ontology.ttl, gates 010-050
(.rq, refusal = row returned, FILTER NOT EXISTS per 010_subject_identity.rq), exact-stem
witness pairs, gate-court.toml (ggen.semantic-gate-witness-court/1, case_key="exact-stem"),
4 templates. Consumer wired: cgap ggen.toml [packs] path dep. Byte-identity proof: make
gate0 (Makefile:42-56 — two syncs, per-file sha256, cmp, cc -std=c99 -ffreestanding).

Full plan (as dispatched to W1-04): pack.toml → 26.11.2; p:-namespace vocabulary
(p:ParetoLever; p:leverId/leverTarget/measuredNs/groundedIn/mutated/mutationName/skipReason);
five lever individuals w/ measured + groundedIn values from 02-pareto-levers.md; gates
060_lever_identity / 070_mutation_law (mutated-true without mutationName refuses) /
080_target_measured_pair (no measuredNs without loud p:skipReason) / 090_grounding_regex
(^crates/[^ ]+:[0-9]+(-[0-9]+)?$) / 100_gate_linkage (leverGate link); exact-stem pass/fail
witness pairs (standalone TTL graphs restating vocabulary as ex:* a p:ParetoLever); templates
lever-court-rs.tmpl → generated/lever_court.rs (const table projected from SPARQL rows —
re-measurement flips p:measuredNs in the graph, never code) + lever-table-txt.tmpl →
generated/lever-table.txt. Proof: ggen sync ×2 + cmp + marketplace.py check +
check_gate_witness_courts.py. Anti-vacuity: every gate must flip its fail witness.

## 07-court-corpus / 06-bench-protocol / 08-registry-sweep doc specs

- Court corpus (W1-12 dispatch): §1 shared discipline (mtime-bump law a02-finish-receipt.md:
  170-184; exit contract 01:11-14); §2 per-lever courts grounded in real code (footer.rs
  :119-127, unify.rs join loop, sovereign.rs:706-730 + CGAP_ERR_P28_SPOLIATION = 0x5032_38FF
  const at :83, seal chain); §3 gate courts per on-disk G1-G8; §4 8-step uniform flip
  procedure; §5 falsifiable summary table, every cell filled.
- Bench protocol (W1-11 dispatch): harness law (Divan 0.1.21, harness=false,
  crates/bench/benches/, consts, ItemsCount); per-lever method incl. seed discipline
  (CHAIN_SEED 0xC001_0000_5EED_0001; CANONICAL_SEED 0xBF58_476D_1CE4_E5B9; corpus-spread
  0xA20C_0FFE_E5A7_E000 never a seed); PARETO admission rule (delta in claimed direction or
  receipt refuses; v26.11.1 receipts P3-16..19); ratio-fence law; MW4 method; unit
  discipline.
- Registry sweep (W1-13 dispatch): cell schema (crate | registry state | versions | standing
  | court pointer) from cargo metadata, never memory; coverage-compiler (crates.io API, UA
  header, network failure = loudly-MISSING never blank); publish sequencing law
  (coord-publish-record.md precedent); publish=false guarded table (bench name-collision
  squatter 1.1.0, beam guarded, court structural, verification/cli delisted); exit contract.

## OST design-around register completion (W2-7)

Patent IDs in the doc match: US 10,817,467 / 11,216,456 / 11,409,698 / 11,645,278. No ost-*
wave files exist in v26.11.1/waves. Register state: row 1 PENDING, row 2 half-landed
(bitslice yes / PLNF no), row 3 PENDING, row 4 landed (A12 unify.rs). G7 requires
docs/jira/v26.11.2/patent/claims.txt + ost_report.json (claims.txt landed by W1-19).

Evidence per row (grep receipts, command+output):
- Row 1 (parallel materialization): grep crates/cgap-storage + cgap-parquet for
  CAS|compare_exchange|AtomicU64.*snapshot — zero expected; cite per-partition delta-page
  sealing path.
- Row 2 (SIP): bitslice.rs landed + PLNF cardinality derivation receipt (pending a landed
  lever) — DISTINCT only when PLNF evidence lands, else OVERLAP-RISK with quoted element
  "runtime tuple-binding propagation".
- Row 3 (recursive SIP): grep cgap-bn3/bont for continuation|Fiber|Generator|resume — zero
  expected + landed semi-naive cascade or PENDING.
- Row 4 (topologies): unify.rs (A12) + no topology-switch dispatch surface — DISTINCT now.
Never mark DISTINCT on statement evidence alone.

## Wave-doc gap report (superseded by W1 lanes 06-16)

Planning-lane findings, now being repaired by W1: PRD §4/§6 stale (W1-07 — landed);
05-wave-plan lever table conflict (W1-08); OVERNIGHT-LOOP §1/§2 stale (W1-09);
RESTART-NOW 18/22 superseded (W1-10 — landed); 4 missing numbered docs (W1-11..14);
release_check pin (W1-16); HANDOFF truncation (W1-06). 02-pareto-levers line-range drift
(sovereign.rs:706-730 vs PRD :706-724) — reconcile to 706-730 (on-disk read).
