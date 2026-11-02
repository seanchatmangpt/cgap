# g1k-04-bench-registry

Lane G1K-4: bench surface as graph individuals (cgap-pack ontology projection).

## Files written

- /Users/sac/ggen-marketplace/packs/cgap-pack/ontology.ttl (Section 7: `p:Bench`
  class + `p:benchTarget`/`p:benchFile`/`p:benchConsts`/`p:benchLever`/
  `p:benchSeedLaw` + 5 individuals `p:bench-chains-{ingress,parquet,bn3,seal,sovereign}`)
- /Users/sac/ggen-marketplace/packs/cgap-pack/templates/bench-registry-tbl.tmpl
  (to: generated/bench-registry.txt; SPARQL rows ORDER BY ?target)
- /Users/sac/ggen-marketplace/packs/cgap-pack/gates/140_bench_file_exists.rq
- /Users/sac/ggen-marketplace/packs/cgap-pack/witnesses/pass/140_bench_file_exists.ttl
- /Users/sac/ggen-marketplace/packs/cgap-pack/witnesses/fail/140_bench_file_exists.ttl
- /Users/sac/cgap/generated/bench-registry.txt (generated; ggen sync)
- /Users/sac/cgap/ggen.lock (re-locked per ggen remediation after pack change)

## Facts grounded (file:line, read this session)

- chains-ingress.rs:54 CHAIN_SEED=0xC001_0000_5EED_0001; :58 SLOT_CAP=512;
  :61 POOL=65536; :65 EC_SHORT=9; :68 EC_OK=0
- chains-parquet.rs:55 CORPUS_SEED=0xBF58_476D_1CE4_E5B9; :59
  LANE_SPREAD=0x2544_5491_4F6C_DD1D; :65 BLOCK_CAP=8*1024*1024
- chains-bn3.rs:33 CANONICAL_SEED=0xBF58_476D_1CE4_E5B9; :37 CHAIN_EPOCH=7; :77 KEY_DOM=8
- chains-seal.rs:55 CORPUS_SEED=0xBF58_476D_1CE4_E5B9; :59 RECORD_SPREAD; :63/:66/:69 domain consts
- chains-sovereign.rs:60 CORPUS_SEED=0xBF58_476D_1CE4_E5B9; :64 RECORD_SPREAD; :68 ROWS_PER_FRAME=16
- Seed law held: 0xBF58_476D_1CE4_E5B9 seed only; 0xA20C_0FFE_E5A7_E000 named as
  spread constant in p:benchSeedLaw notes, never used as a seed.

## Commands / exits (this session)

1. `python3 scripts/check_gate_witness_courts.py --packs packs --report /tmp/g1k4_court.json`
   (cwd /Users/sac/ggen-marketplace) -> exit 0; cgap-pack case_count=12,
   standing ALIVE, all 47 packs ALIVE.
2. Mutation court (rdflib, /tmp/g1k4-sandbox-pack copy of cgap-pack):
   BASELINE gate 140 rows=0 -> MUTATED (drop p:benchFile from chains-sovereign)
   rows=1 -> FLIP-OK; restore sha-exact vs the copy taken. Shared tree never mutated.
3. `make gate0` (cwd /Users/sac/cgap): first PASS at bounded-retry attempt 2
   (lane G1K-3 in-flight gate 130 status facts refused the union sync; passed
   after that lane landed LANDED- statuses). After my template front-matter
   fix, `rm ggen.lock` + `make gate0` -> "gate0: byte-stable replay proof
   PASS" + "header compile PASS" (exit 0), sync x2 cmp clean.
4. Differential assert (python, this session): projected bench names ==
   Cargo.toml [[bench]] chains-* names -> PASS (no drift).

## Standing

ALIVE - every claim above carries a command + exit run this session against
the exact subjects.

## Divergences

- ggen 26.10.11 requires template front matter to open with `---` on line 1;
  templates opening with a bare key (`force: true` first) are silently not
  discovered (no error, no render). Pre-existing latent defect also present in
  cgap_format_codes_h.tmpl / cgap_h.tmpl / layout_gen_rs.tmpl / court-table-md.tmpl
  (same first-line shape; their outputs did not render this session either).
- check_gate_witness_courts.py qualification is structural (stem correspondence
  + config schema), not SPARQL evaluation; the gate flip proof is the rdflib
  mutation court above.
- Concurrent lanes (G1K-3 leverStatus, courts section) edited the shared
  ontology.ttl during this lane; appends only, no conflicts. One gate0
  failure window (~60s) was G1K-3 in-flight state, resolved by that lane.
