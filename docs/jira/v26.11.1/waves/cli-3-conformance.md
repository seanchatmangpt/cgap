# CLI-3: marketplace-pack conformance audit of the extended CLI ontology

## Lane
CLI-3 (court-auditor), repo /Users/sac/cgap, 2026-11-01/02. Read-only on
crates/cli/ (CLI-1 owns the writes); audited CLI-1's landed state after a
bounded poll (270 s) confirmed regen landed (cli-1-ontology.md present,
generated_cli.rs mtime 20:14 > ontology 20:11).

## Verdict: CONFORMANT (with 2 named observations, no violations)

## Laws audited + verdicts
1. Pack schema conformance (clap-noun-verb-schema-pack 1.1.0, the pack pinned
   in crates/cli/ggen.lock; main clap-noun-verb-pack is DEPRECATED and
   correctly NOT consumed): all 16 SPARQL gates
   (~/ggen-marketplace/packs/clap-noun-verb-schema-pack/gates/010..160.rq)
   run with rdflib 7.6.0 against crates/cli/ontology/cgap-cli.ttl (214
   triples): 0 violation rows total. PASS.
2. Anti-vacuity mutation court (in /tmp sandbox copy only; original never
   touched, sha256 4617dc82... unchanged before/after): deleting one
   `cnv:belongsToNoun` triple (cnv:Bn3Eval) flips the gate suite 0 -> 2
   violation rows (010_required.rq: 1, 050_semantic_closure.rq: 1). The
   courts FLIP; not vacuous. PASS.
3. Naming/shape laws: 15 nouns x 21 commands, noun-verb shape
   (`#[verb("<verb>", "<noun>")]` x21), every command exactly one behavior,
   all cnv:CustomBehavior; 21/21 handler call sites in generated_cli.rs
   resolve to defined `pub fn` in custom_handlers.rs. PASS.
4. clap-noun-verb routing contract (~/clap-noun-verb 26.7.62, real binary
   target/debug/cgap): known route `self version` -> "26.11.1" exit 0;
   unknown noun `bogus list` -> clap refusal exit 1; unknown verb
   `self bogus` -> refusal exit 1; routed-but-unimplemented surfaces
   (`beam info`, `court verdict-run`) fail LOUD and typed
   (UNIMPLEMENTED: handler lands in CLI-2, exit 1) — routed, not broken,
   exactly as declared in cli-1-ontology.md. cargo test: 25 passed, 0
   failed, exit 0; cargo build exit 0. zero_code_cli.rs generated
   real-binary tests are among the passing suite. PASS.
5. On-disk byte integrity (long-file corruption sweep): no corruption
   tokens (conflict markers, truncation) in generated_cli.rs (437 lines),
   cgap-cli.ttl (256), command-reference.md (132, 21/21 sections),
   zero_code_cli.rs; sha256 of all audited files recorded in this receipt's
   command log. PASS.

## Observations (not violations)
- O1: the compiler digit-splits noun "bn3"/"p28" into handler names
  `bn_3_unify`/`p_28_attest_verify` — consistent on both generator and
  consumer sides (compiles, tests green), but the schema-pack naming law
  "underscores-to-hyphens" comment applies to flags, not nouns; watch if a
  future regen changes the split.
- O2: mid-audit the ontology was observed in-flight (mtime 20:10) missing
  definitions for 5 nouns; landed state is complete. No action.

## Commands (auditor's own runs, all exit 0 unless noted)
- rdflib gate suite x16 on landed ontology: 0 violations
- mutation court in /tmp sandbox: 0 -> 2 rows, restore sha256-exact
- cargo build (crates/cli): 0; cargo test (crates/cli): 0 (25 passed)
- binary probes: self version 0; bogus list 1; self bogus 1; beam info 1
  (typed UNIMPLEMENTED); court verdict-run 1 (typed UNIMPLEMENTED)

## Standing
CONFORMANT. No skip taken. No unreproducible claim. crates/cli untouched by
this lane (git status shows only CLI-1's diff).
