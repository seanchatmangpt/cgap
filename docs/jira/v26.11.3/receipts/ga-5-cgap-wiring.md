# GA-5: cgap consumer wiring + first real sync (PG-A6 step 6)

Date: 2026-11-03 (night wave) | Lane: GA-5 | Repo: /Users/sac/cgap

## Standing: PARTIAL_ALIVE

Every command below was run this session against the exact subject, exit
codes recorded. PARTIAL because the pack is a STAGED fallback, not GA-4's
canonical marketplace aoq-pack.

## Dependency read (READ-FIRST)

GA-4's pack did NOT land: /Users/sac/ggen-marketplace/packs/aoq-pack
absent, no GA-4 activity in marketplace, bounded poll 140s + mtime sweep
-> NOT-LANDED. Invoked the dispatch's sanctioned fallback: staged a
minimal aoq-pack in cgap-owned space.

## Diff (owned + disclosed)

Owned:
- /Users/sac/cgap/ggen.toml - [ontology] imports line + [packs] aoq-pack
  entry. DEVIATION (disclosed): prescribed paths pointed at
  ../ggen-marketplace/packs/aoq-pack; staged paths point at
  ontology-packs/aoq (cgap-local). GA-4's landing supersedes: flip both
  lines back + re-sync + re-gate.
- /Users/sac/cgap/Makefile - GATE0_FILES += include/cgap_aoq.h (line 37).
- /Users/sac/cgap/docs/jira/v26.11.3/receipts/ - this receipt.

Staged fallback pack (cgap-owned, GA-4 supersedes):
- ontology-packs/aoq/{pack.toml,ontology.ttl,gates/010_offsets_dense.rq,
  templates/} - minimal Aligned Offset Queue FFI surface: 2 records
  (aoq_envelope_t control block, aoq_entry_t 32-byte slot), 9 fields with
  explicit graph-carried offsets; 1 violation-ASK gate.
- crates/cgap-ffi/src/lib.rs - `pub mod aoq;` registration (UNOWNED
  subtree at dispatch time: mtimes hours old, cg-1 receipt claims
  cgap-arrow only). Disclosed per fanout law 6.

## Commands + exits

1. ggen sync run (run 1)          -> EXIT 0, written: all 5 outputs
2. ggen sync run (run 2)          -> EXIT 0, written: [] (determinism)
   sha256 cmp run1 vs run2        -> DETERMINISM-PASS, 5/5 byte-identical
   graph_hash_hex 94534e536c24bffb stable across runs
3. cc -std=c99 -ffreestanding -Wall -Wextra -pedantic -c
   include/cgap_aoq.h -o /tmp/cgap_aoq_gate.o -> EXIT 0
4. cargo check -p cgap-ffi        -> Finished, EXIT 0
   (with `pub mod aoq;` registered in lib.rs)
5. Receipts dir listing           -> receipt on disk, this file

## Outputs (sha256, run-2 replay)

a07ca48d4fe4582557a274d909e30aed58d50e20084f5f61dea5c5a3c0d6b8c8  crates/cgap-ffi/src/aoq/mod.rs
207b75d587ad65d26ad98a2536a5cfed01bf8b54aae4189d31b194c83c83b1f8  crates/cgap-ffi/src/aoq/records.rs
9229bae9a49d2f0a0a0cf47d764ad839f908fe380a643dfcaea9ad4d5fecbd5f  crates/cgap-ffi/src/aoq/offsets.rs
13a20f5a6d1875d3916e03f35926ec0f6b3f9769aef055e388338ab391ccfff1  include/cgap_aoq.h
ded85e805ea0eb4f7ff5dbce9a166e2d6bd454b6017389ae95bc3eca1eec2234  docs/aoq-header-offsets.md

## Falsifier / flip condition

This receipt's PARTIAL standing flips to superseded when GA-4 lands:
flip ggen.toml paths to ../ggen-marketplace/packs/aoq-pack, rm -rf
ontology-packs/aoq, re-run gate0 (sync x2 + header compile).
