# W1-04 — cgap-pack Pareto-Lever Surface Extension

Standing: ALIVE (all commands below ran this session, exits shown).
Repos: /Users/sac/cgap (receipt only), /Users/sac/ggen-marketplace/packs/cgap-pack.

## Files written (pack)
- pack.toml (version 26.10.29 -> 26.11.2)
- ontology.ttl (ParetoLever/LeverGate classes; 8 lever properties incl.
  p:leverGate; 5 gate individuals; L1..L5 individuals)
- gates/{060_lever_identity,070_mutation_law,080_target_measured_pair,
  090_grounding_regex,100_gate_linkage}.rq
- witnesses/{pass,fail}/{060..100}*.ttl (10 exact-stem witness pairs)
No templates touched. No git commands run.

## Individuals (groundedIn verified on disk this session)
- L1: crates/cgap-core/src/frame.rs:86-87 (origin_authority_digest)
- L2: crates/cgap-parquet/src/footer.rs:119-127 (fnv1a64_seeded)
- L3: crates/cgap-bn3/src/unify.rs:330-342 (TriplePattern::new)
- L4: crates/cgap-ffi/src/sovereign.rs:706-730 (ring sink handoff)
- L5: crates/cgap-p28/src/asc450.rs:360-364 (ProofSealer::seal)

## Courts run (this session, exact commands + exits)
1. `python3 scripts/check_gate_witness_courts.py` (from ggen-marketplace)
   -> EXIT 0; cgap-pack case_count=10 ALIVE (structural exact-stem court;
   script has no pack-scoping flag; run scope = all 47 configured packs).
2. rdflib flip battery (ontology rows=0; fail witness >=1; pass witness =0
   per gate) -> EXIT 0, "ALL 10 GATES FLIP". First run FAILED gate 060
   (AND-joined NOT EXISTS; fail witness did not fire) — rewritten as UNION
   of missing-any; second run FAILED on ontology (L3 missing leverTarget,
   L4 missing groundedIn) — individuals repaired; third run all-flip.
3. `ggen sync run` (cgap root) -> EXIT 0. First attempt refused typed
   [FM-PACK-008] content-hash mismatch vs ggen.lock; re-locked per ggen's
   remediation (rm ggen.lock) and reran: pack cgap-pack relocked at
   blake3:1f5b9bee..., generated outputs unchanged.

## Anti-vacuity
Each lever gate carries a fail witness that fires (>=1 row) and a pass
witness that stays silent (0 rows); the battery exited 1 twice before the
060-AND and L3/L4 repairs, proving non-vacuity of the court set itself.

## Typed findings (recorded, not repaired here)
- L1 measured 102.7 ns/frame vs target <15 ns/frame: measured EXCEEDS
  target (as dispatched; recorded verbatim).
- L5 measured 2.37 us/row vs target <1.45 us/row: measured EXCEEDS target.
- L3 has no dispatch-supplied measurement; carries leverTarget
  "unbounded-until-measured" + p:skipReason "unmeasured-this-wave"
  to satisfy gates 060/080.

## Divergences from dispatch
- p:leverGate property + p:LeverGate class and p:gate-060..p:gate-100
  individuals were added (gate 100 requires a leverGate link target; the
  dispatch property list omitted it).
- Gate 080 implemented as "target without measuredNs and without
  skipReason refuses" (the literal "no measuredNs without skipReason"
  would force skipReason onto every measured lever).
- ggen.lock deleted+regenerated inside /Users/sac/cgap (documented
  remediation for FM-PACK-008 after a legitimate pack edit).
