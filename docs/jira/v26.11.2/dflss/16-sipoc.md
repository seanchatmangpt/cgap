# 16-sipoc

SIPOC for the v26.11.2 starDF campaign, DMEDI Define/Measure stage
(`14-dflss-charter.md`). Canonical home is now
`docs/jira/v26.11.2/dflss/16-sipoc.md`; the former orphan
`docs/jira/v26.11.2/16-sipoc.md` redirects here (lane DF-2).
Metric classes follow `11-measured-claims.md`; the measurement method
of record is `06-bench-protocol.md`.

## Section 1 - Suppliers

| Supplier | Provides | Court / anchor |
|---|---|---|
| DMA ring producers (`crates/cgap-core/src/dma_ring.rs`) | Frame transport into admission | `cgap-core/tests/unit_dma_ring.rs` |
| Git pack streams (`crates/cgap-git/src/pack.rs`, `pack_write.rs`) | Pack/inflate transport, ref-store leases | `cgap-git/tests/git_interop.rs` |
| A2A agents (`crates/cgap-git/src/a2a_swarm.rs`) | Agent lease envelopes, commit proposals | `cgap-git/tests/a2a_swarm_court.rs` |
| ggen CLI pack (`crates/cli`, standalone) | Generated command surface, byte-identical replay | `make gate0` |
| Kernel crates (`cgap-{arrow,blake3,git,parquet}`) | Zero-dep columnar/hash kernels | `python3 scripts/boundary_gate.py` (15-crate table) |
| Moonshot lanes (`cgap-{core,storage,bont,bosym,bn3,gate,crypto,p28}`) | Deductive/storage substrate, P28 attestation | `python3 scripts/boundary_gate.py` |
| riscv32imc toolchain | Cross-compiled audit-profile objects | `cgap-core/tests/rv32imc_court.rs` |
| Court harness (`cgap-court`, `scripts/*.py`) | Fail-closed gates, flip batteries | `boundary_gate.py`, `zero_string_gate.py --self-check` |

## Section 2 - Inputs

| Input | Source spec | Anchor |
|---|---|---|
| Frames over `CgapFrameHeader` (128 B) | A04 | `cgap-core/src/frame.rs:178` |
| `P28LicenseAttestation` (80 B), URN `urn:avec:license:p28-sil:v2.1.0-sovereign` | A17 | `cgap-p28/src/attestation.rs:69,135` |
| Coordinate triples / key words (u64) | A11 BN3 layout | `cgap-bn3/src/unify.rs:748-840` |
| Corpus seed `0xA20C_0FFE_E5A7_E000` | Frozen seed pair (never conflate with the SplitMix64 multiplier `0xBF58_476D_1CE4_E5B9`) | `cgap-parquet/src/bitslice.rs:687-699` |
| WAL framing | A05 | `cgap-storage` WAL surface |
| Bench protocol + unit law | Sections 6-7 | `06-bench-protocol.md` |

## Section 3 - Process (5 stages)

| Stage | Name | Real module | Court name | Gate |
|---|---|---|---|---|
| P1 | Ingress (SWAR scan) | `cgap-core/src/dma_ring.rs` + `cgap-branchless/src/scan.rs` | `cgap-core/tests/unit_dma_ring.rs`, `cgap-branchless/tests/scan_courts.rs` | <15 ns/frame [TARGET]; chain MEASURED 10.20 ns/frame quiet host (`waves/w3-l1-ingress.md:63`; claims rows 5/28) |
| P2 | Radix fixpoint unify | `cgap-bn3/src/unify.rs:840` (`RADIX_BUCKETS=4096`) | `cgap-bn3/tests/bgp_hop_courts.rs`, `lever_courts.rs` | >=1M facts/s [TARGET-v26.11.2] |
| P3 | DRed retract | `cgap-storage/src/dred.rs` | `cgap-storage/tests/dred_cascade.rs`, `dred_swar_wire.rs` | closure fixed-point identity, typed retracts |
| P4 | Receipt seal | `csct/src/receipt.rs` + `cgap-blake3` digest | `csct/tests/csct_pipeline_court.rs`, `csct_coverage_court.rs` | byte-stable receipts; blake3 digest in every receipt |
| P5 | Commit | `cgap-parquet/src/mmap_writer.rs:721` (`RingSink`) | `cgap-parquet/tests/seal_rows_bound_court.rs`, `mutation_kills.rs` | <4 us/tx [TARGET]; MEASURED 1.05-1.07 us/tx |

Stage-boundary detail: P1 passes admitted frames only; P2 passes
derived facts with the BLAKE3 fold (85-91% fold observed,
`00-ard-prd.md:41`); P3 passes surviving facts after typed retracts;
P4 passes sealed blocks with 0x5032_38FF tamper refusal
(`cgap-gate/src/penstock.rs:317-319`); P5 commits under the commit
lease (`cgap-ffi/src/sovereign.rs:729-794`). Budget fence: Flash
<= 131,072 B linked-ELF ceiling, <115 KiB [TARGET-v26.11.2];
riscv32imc audit profile <= 32,768 B (object MEASURED 28,679 B,
`waves/w3-t3-silicon.md:36,50`).

## Section 4 - Outputs

| Output | Accepting stage / customer | Class |
|---|---|---|
| Admitted frame stream | P2 | MEASURED (10.20 ns/frame chain, quiet host; divergence row 28) |
| Derived closure facts | P3 | [TARGET-v26.11.2] facts/s |
| Typed retracts (DRed cascade) | P4 | court class (`dred_cascade.rs`) |
| Sealed blocks + BLAKE3 receipts | P5 | court class (`csct_pipeline_court.rs`) |
| Committed parquet via `RingSink` | customers | MEASURED (1.05-1.07 us/tx) |
| Fail-closed refusal 0x5032_38FF (`P28_SPOLIATION`) | customers | MEASURED court, both ways |
| Receipts + wave ledgers (`docs/jira/v26.11.2/waves/`) | coordinator | MEASURED |
| 101 typed refusals | customers | court corpus class |

## Section 5 - Customers

| Customer | Consumes |
|---|---|
| Enclave OEMs | 131,072 B ELF ceiling, 0 heap, fail-closed |
| Real-time integrators | <15 ns/frame, <4 us/tx, 0 blr / 0 brk |
| Agentic AI runtimes | >=1M facts/s, WAL restart replay, typed refusals |
| High-assurance auditors | 100% receipted claims, non-vacuous flip courts, frozen pins |

## Honest-notes register (mandatory)

- The seal register `0x0001_FF80` is an ABSTRACT 128-B slot, not a
  hardware map: provenance comment at
  `crates/cgap-git/src/ref_store.rs:1-13` (GIT-13), constant at
  `ref_store.rs:32`. No in-tree hardware referent exists.
- Swarm leases carry no SMP claim: single-threaded court,
  single-writer-per-agent; atomics deliberately NOT used
  (`crates/cgap-git/src/a2a_swarm.rs:1-13`).
- `cgap-git/src/mcp_driver.rs` is a fixed-schema TEXT envelope that is
  MCP-shaped, NOT JSON-RPC 2.0 (header divergence note,
  `mcp_driver.rs:1-14`).
- No hardware lore anywhere in this document: every register-style
  constant is traced to a named in-tree referent or marked abstract.
