# dod-cgap-git

EXP-07 audit lane, crates/cgap-git (2026-11-02). cargo build -p cgap-git
exit 0 (0.40s); every test target run individually this session.

## 1. Landed status

lib.rs wired (ALIVE): deflate (:19), pack_write (:20, no_std hygiene
court :745), idx_read (:136), ladder (:137), inflate (:140), dag (:143).
pack facade NOT top-level: dag.rs:120-121 mounts #[path] pack.rs as
dag::pack; promotion to crate::pack pending (dag.rs:115-119).

lib 61 passed / 0 failed. Per-target exits (all 0): fsck 2, git_interop
3, fuzz_codec 4, packfile_fuzz 2 (58.30s), large_offset 6,
ladder_mutations 11, ladder_timing_stress 0+1 ignored, perf_baseline
0+1 ignored, lineage_workflow 1, envelope_decimal 2 (100.85s).

15 new lane modules: landed + court-green via #[path], NOT in lib.rs
(correct per contract; coordinator wiring = FIX-06 lane): pack_demux 10,
pkt_line 10, ingress_ring 6, delta_resolver 8+4, object_store 6,
tree_synth 6, egress_ring 5, commit_synth 10, admission_gate 6,
patent_court 4, retract 8, octopus_merge 5, ref_store 5, mcp_driver 6,
a2a_swarm 4 — all exit 0.

## 2. Wire seam (WASM-1 Latin-1)

GAP: repo-wide grep WASM-1/Latin-1/packet-N over cgap-git + docs = ZERO
hits; invariant undocumented where pkt_line would meet a stdio harness.
No current violation (no binary framing over stdio in lib surface; the
only stdio use is cfg(test) KAT scaffolding, deflate.rs:631-649).

JSON-RPC not in no_std core: COMPLIANT — mcp_driver.rs:5-21 documents
the fixed-schema text envelope as MCP-shaped, NOT JSON-RPC; wire grammar
READ/TOOL frames (:26-31); zero-daemon law (:41-43).

## 3. soc_full_chip_court: LANDED but RED

exit 1, 0 passed / 1 failed, 9.59s. Panic at :486:
Err(Delta(TruncatedDelta)); secondary unwrap :553. Court's own emitted
exclusions (:491-494): commit_synth/admission_gate/patent_court/retract/
ref_store "landed after this pipeline froze -> NOT integrated".
Root-cause lead: court hand-encodes a delta at :357-363 against
delta_resolver.rs apply_into which reads target-varint FIRST (:152-153)
— hand frame vs landed applier disagree -> TruncatedDelta
(delta_resolver.rs:118). In-flight mismatch, not fixed (read-only lane).
Instruments present + executed before the failing assert: CountingAlloc
(:57-87) positive control FIRST (:476-478), 0-alloc assert :489; canary
[u8;8192] 0xDEADBEEFCAFEBABE (:471-474, assert :497-499, not reached).
No SoC receipt emitted (docs/csct/ has no receipt-cgap-git-soc-v1.md).

## 4. Namespace separation

SHA-1 domain: Oid = [u8; 20] (lib.rs:25), pack.rs:26 re-exports sha1.
BLAKE3 domain: DagDigest = [u8; 32] (dag.rs:131-136, "never a git OID"),
tree_synth 64-hex (:8-15, divergence documented), object_store Digest
= [u8; 32] with domain tag (:24,:29). Cross-contamination grep over
tree_synth/object_store for sha1/Oid/pack_write = ZERO hits.
Verdict: separation real via arity + docs, NOT newtypes — two [u8;32]
values (DagDigest vs Digest) are mutually assignable; newtype boundary
is the open hardening item.

## Bottom line

- ALIVE: 6 wired modules + 15 #[path] modules, all courts green.
- PENDING: lib.rs wiring (FIX-06), crate::pack promotion, WASM-1 docs.
- RED: soc_full_chip_court exit 1 (delta-frame mismatch, GIT-16 scope).
- Hardening: newtype boundary for the two 32-byte digest aliases.

## See Also

audit/README.md · lanes/lane-git-01-pack-demux.md · dag.rs:115-121
