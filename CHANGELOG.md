# Changelog

All notable changes to cgap are documented in this file. Format based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
the project's calendar scheme (`26.10.29` = 2026-10-29). Every number below is
transcribed from the v26.10.29 receipt docs (`docs/RECEIPTS.md`,
`docs/BEAM_LATENCY.md`, `docs/BRANCH_REGISTER.md`, `docs/wasm-footprint.txt`,
`docs/BENCH_FLOORS.json`); conflicts between receipt docs are listed in the
release notes' Discrepancies section, not reconciled here.

## [26.11.3] - 2026-11-03

AoQ Wave-1 campaign release. Every number below is transcribed from the
session's landed receipts (`docs/jira/v26.11.3/receipts/`); UNVERIFIED
never rounds up and fail-honest gate results are recorded as measured,
not relaxed.

### Added

- `crates/cgap-aoq` — AOQ1 wire-law kernel (64-byte `AoqHeader`,
  `#[repr(C)]`, closed `Epoch` enum, compile-time layout court with
  ontology drift court + mutation flip witness: ab-6; typed
  `AoqError` codec decode: ab-6/ab-8; hash-chained receipt chain
  `AoqChain` with 4-state session machine and `verify_records`
  (BLAKE3 digest substrate, cgap-arrow): ab-7; wire-tier IPC/SHM
  stress and 0-alloc counting-allocator courts: ab-9/ab-10/ab-11;
  zero-dep writer stress: ab-10; slab rate floor 12.2M pushes/s:
  ab-19; FFI stress: ab-15).
- `crates/cgap-crypto` kernels — `fips202` KECCAK-p[1600,24] +
  SHAKE-128/256 with OpenSSL-oracle KAT courts (two from-memory KAT
  constants caught wrong and corrected: ab-2); ChaCha20 block
  (RFC 8439) and Poly1305 (ab-3); AEAD with typed
  `AeadKernelNotAdmitted` pre-admission seam (ab-4); ML-KEM/fips203
  NTT court tables (ab-5, disclosed round-trip defect). Negative
  mutation demonstration court `kat_flip_demo.rs`: 324/324 flips
  detected, baselines against landed pub APIs, sandbox-only mutation
  with byte-exact restore (af-12).
- CBMQS qualification machinery — executable seven-gate evaluator
  `cbmqs_gates.rs` (8 gate functions incl. G1b slab floor; G2 wire
  timing recorded FAIL-honest at ~717 ns vs the 100 ns fence, fence
  not relaxed; G6 typed SKIP: af-10); CBMQS-7G/1 bake-off runner
  `cbmqs_runner.rs` (`harness = false`, exit-status verdict, BLAKE3-
  sealed JSON report, `--court` non-vacuity mode, foreign-subject
  subprocess mode, self-test verdict PASS: af-14); RFC-0089 §8.4
  qualification certificate machinery, fail-closed on SKIP gates
  (rx-5); scorecard (rx-3) and profile guard (rx-4).
- Macro-benchmarks over real UDS transport (all in `cgap-aoq/tests/`,
  no mocks): flash-crash clearinghouse, scaled 1M x2 green + full
  10M-frame documented run, 128 B wire units, margin-ledger risk
  gate, per-frame BLAKE3 receipt chain (af-26); defense telemetry
  mesh, 64 session channels through the real 4-state session
  machine, per-channel `AoqChain` verification, AEAD attempts
  honestly census'd as typed refusals (af-27); healthcare records
  federation, 500 UDS endpoints / 1000 threads, consent-preimage
  law bound into the chain hash, real RFC 8439 detached AEAD seals,
  adversarial probe census incl. canonically-encoded epoch-0 probe
  (af-29); cage-match Contender A runner, 0.5630 µs/frame release,
  0 allocs (af-30; full cage-match receipt in arcus); saturation and
  density follow-ups (af-32, af-33).
- Corridor join kernel (af-1) and per-link/per-call latency profiles
  (af-21, af-22), transit bench (af-35), G2-convergence (af-25),
  SIMD profile (af-3).
- cgap-arrow wire-tier gates (CG-1): `BadVersion`/`BadFlags` ipc
  gates in shared order, `parse_meta` seam (`ParsedMeta`,
  `read_with_meta`), truncation classified `Truncated` (killing the
  wrong-class `BodyDigestNoRoom`), seq wrap-safe successor law,
  witness-tier doc section, 6 new G1 courts (145 lib tests green),
  frame-ABI mirror court in cgap-gate, wire-shm zero-"blake3" scan
  court (cg-1).
- Publish-wave fixes: gofmt formatting, `bench_gates` bash
  compatibility, workspace version drift, CHANGELOG (this section)
  and threat-model/CLAUDE.md updates.

## [26.11.2] - 2026-11-02

starDF campaign release. Every number below cites its registry row or
receipt in `docs/jira/v26.11.2/`; UNVERIFIED never rounds up and TARGET
figures are labeled as such.

### Summary

- Sovereign columnar/deductive substrate: 23-member root workspace at
  26.11.2, 18-dossier corrosion pass + 19-lane red-team hardening,
  95-court
  conformance kit, publish wave halted by operator order after 8
  crates landed live.
- Numeric registry of record: `11-measured-claims.md` (rows 1-37,
  including the consolidation addendum rows 30-32 and the
  optimization-discovery addendum rows 33-37).

### Added

- FFI surface: 12 exported `cgap_*` symbols == 12 header prototypes
  (1:1), including the new `cgap_stats_snapshot`
  (lanes/lane-xc01-ffi-surface-truth.md:26;
  lanes/lane-r05-gate-battery-recount.md:24).
- `CgasStatsBlock` - fixed-layout `repr(C)` runtime stats snapshot,
  120 B with FNV-1a-32 CRC over bytes [0,112) and typed poll verdicts
  (crates/cgap-ffi/src/stats.rs:30-33,111-113;
  lanes/lane-rc05-stats-block.md:12-15).
- Header mirror: `typedef struct CgasStatsBlock` + prototype added to
  include/cgap.h:223-245 (lanes/lane-xc01-ffi-surface-truth.md:9-12).
- M-CTQ consolidation stations, first receipted facts/s figure of the
  campaign (rows 30-32, consolidation-addendum class; cross-instrument
  wire legs remain UNVERIFIED, 11-measured-claims.md:126-127).
- breach_regression court: 5 legs over the four hard breach repairs +
  the SRAM gate, 5/5 mutation flips witnessed in /tmp sandboxes with
  sha256-exact restores (lanes/lane-r14-breach-regression-court.md
  W2-W4, final run exit 0, 5 passed).
- Cargo lints: `[lints.rust]` `unexpected_cfgs` allowlist
  (`check-cfg = ['cfg(implcell_critical_provided)']`) added to
  `crates/cgap-git/Cargo.toml:13-18` per rustc's printed suggestion
  (lane WARN-01; receipt audit/gap-04-final-verification.md:111-120,
  post-fix build witnessed: warning gone, cgap-git lib warning delta
  12 -> 11, width-arms court 8/8 exit 0).
- CLI license-file via the lawful graph path: `crates/cli/
  manifest_patch.py` step 1.5 inserts
  `license-file = "../../LICENSE.md"` on ggen regen; byte-identical
  replay witnessed, root metadata still 23 members
  (lanes/lane-cli01-license-via-graph.md W3-W4).
- Optimization-discovery register rows 33-37 (TARGET,
  v26.11.3-candidate): see the v26.11.2 release notes roadmap.

### Hardening summary

- Red-team wave: 19 lanes (RED-01..RED-19) synthesized by RED-20 into
  18 breach rows, 13 UNSUPPORTED narrated targets, 10 survivor surfaces
  (redteam/red-20-hardened-survivor-synthesis.md:38-56,66-83,87-98).
- Corrosion wave: 18 ADV-NN dossiers on disk under
  redteam/corrosion/; repairs landed by the lane-r-adv* receipts
  (adv05fix gate-battery-utf8, adv09fix contention-doc-wiring, adv10fix
  spill-gate-rename-toctou, adv11fix implcell-slide, adv12fix
  quota-repair) plus FIX-01 (fa9 alias rekey), FIX-02 (bn3 SRAM gate),
  FIX-04 (andon quota), FIX-05 (pack_demux poison); 15
  REPAIRED-RECEIPTED rows in DEFECT-REGISTER.md.
- breach_regression court: 5/5 legs green (exit 0, 5 passed) with 5/5
  witnessed mutation flips and fail-closed restores
  (lanes/lane-r14-breach-regression-court.md W3-W4).
- Conformance kit: live court count 95 (85 cargo-test + 5 corpus-doc +
  5 gate-mode) per lanes/lane-x-05fix-court-count.md:8; count is
  hearsay-cited there, not independently re-run (X-05FIX disclosure,
  carried verbatim).
- License: P28-SIL v2.1.1 revision rider landed (REVISION RIDER,
  LICENSE-P28-SIL.md) - Controlled Group aggregation (SS2.2.1-2.2.3)
  and the $50M SS2.4 numeric safe-harbor floor, per
  redteam/fix-10-license-aggregation.md; URN unchanged (coordinator
  item). ERRATA rider (TC-08) qualifies the (mod Sigma_PQC) restatement
  as invented lore with zero code referent.

### Changed

- Publish freeze (operator order, 2026-11-02): 8 crates landed live at
  26.11.2 before the halt (cgap-blake3, cgap-arrow, cgap-git,
  cgap-parquet, cgap umbrella, cgap-core, cgap-storage, cgap-bont);
  remaining set FROZEN-BY-OPERATOR
  (waves/publish-halt.md:5-24). Package-mode dry-run matrix: 16/20
  publish-intended crates DRY-RUN-OK, 4 BLOCKED-DEP-ORDER; flag census
  23/23 explicit (20 true / 3 false)
  (audit/pub-01-dry-run-matrix.md:36-40,70-76).
- riscv32imc atomic ladder: legacy cfg ladder no longer silently
  selects implcell; `implcell::critical` is TYPED behind
  `#[cfg(implcell_critical_provided)]` - without the RUSTFLAGS opt-in,
  a typed refusal
  (lanes/lane-r-adv11fix-implcell-slide.md:36-51,72-73).
- Version unification: all 23 members at 26.11.2 via
  `version.workspace = true` (dod/dod-01-cargo-hygiene.md, finding
  class 1); csct standalone remains 0.1.0 (recorded divergence F1).

### Honest known bounds (never quote these as claims)

- Commit-leg receipts: minting ZERO receipts at commit is the landed
  state; commit-leg non-repudiation is UNSUPPORTED until P-07 wires
  it. Plan of record LANDED on disk (PLAN status: transcribed, NOT
  yet executed):
  `lanes/plan-p07-commit-leg-wiring.md` (seam
  sovereign.rs:811-819; per-thread chain inside CommitLease), also
  dod/v26.11.3-forward-backlog.md §1; register row RW-09-O4
  (DEFECT-REGISTER.md:322-327).
- impl16/implcell behavioral execution legs: compile-only coverage
  (lanes/lane-rc02-ref-store-width-arms.md:25-29); behavioral RMW
  coverage remains OPEN (RW-09-O3, DEFECT-REGISTER.md:316-321).
- Cross-instrument wire legs (cgo admit, Go-harness mmap read,
  1M-window seal) are UNVERIFIED; consolidation-addendum figures are
  never promoted to wire-MEASURED until re-measured on the wire
  instruments (11-measured-claims.md:126-127; RW-09-O6).
- Case-variant portability residual: alias protection is PROTECTED
  ON THIS HOST only (APFS case-insensitive); case-sensitive mounts
  are unprotected (RW-09-O1, DEFECT-REGISTER.md:304-310).
- O_NOFOLLOW|O_EXCL intra-syscall swap residual: verify->create
  window is kernel-serialization-bound, kept loud
  (RW-09-O2, DEFECT-REGISTER.md:311-315).
- P28 R1-R10 execution legs still owed (R4 measured-honestly-failing,
  R10 OPEN-NARROWED, NEW-5 gate decision pending) - register OPEN
  (RW-09-O5, DEFECT-REGISTER.md:328-333; audit/tc-10-open-recensus.md).
- License URN: `urn:avec:license:p28-sil:v2.1.0-sovereign` retained;
  a URN bump after the v2.1.1 rider is a coordinator-owned decision,
  OPEN (LICENSE-P28-SIL.md REVISION RIDER).
- "Constant-time" ingress validation: UNSUPPORTED on disk, PRD prose
  only (row 29; red-10:15-26). Never quote as a security property.
- riscv32imc default-features build fails E0463; the no_std claim
  holds only with `--no-default-features`
  (redteam/red-11-riscv-reentrancy-corruption.md:34-37; red-20 §0.2).

### Measured performance (rows 39-40, consolidation-addendum class)

Append-only addendum over the numeric registry of record
(`11-measured-claims.md` §6.1, lane PROMO-01, 2026-11-02). Both rows are
MEASURED, hot-ambient scoped, doubly witnessed (F-06 + FS-02), exact
subject stated; NOT chains-bn3 legs (FS-02 correction :198). Ambient
per §6.1: neither run held a quiet-host marker (F-06 hot ambient with
loadavg snapshots per leg; FS-02 drift check loadavg1 5.78-8.91).

| # | Claim | Number | Unit | Method | Receipt cite |
|---|---|---|---|---|---|
| 39 | BN3 `statement_scan_24b` @65536 (v26.11.1 dossier 82.24 GB/s): CONFIRMED-IN-BAND twice independently — F-06 80.49 GB/s (-2.1%); FS-02 drift check 19.12 µs fastest = exact 82.24 reproduction; @1M companion 78.3 GB/s vs dossier 78.94 (-0.8%) | 82.24 (dossier) / 80.49 (F-06) / 82.24 (FS-02) | GB/s | `cargo bench -p bench --bench bn3 -- statement_scan_24b`, divan fastest @65536, exit 0 both runs; NOT a chains-bn3 stage leg | `bench-validate/bench-02-seal-bn3.md:83-85,103,118-119,147` (F-06); `bench-validate/bench-03-drift-check.md:107,129-135,159` (FS-02) |
| 40 | P28 attestation verify @1024 (v26.11.1 dossier ~460 ns = 459.96): CONFIRMED-IN-BAND twice independently — F-06 462.3 ns (+0.51%); FS-02 458.0 ns (468.8 µs/1024, -0.4%) | ~460 (dossier 459.96) / 462.3 (F-06) / 458.0 (FS-02) | ns/attestation | `cargo bench -p bench --bench seal -- p28_attestation_verify`, divan fastest @1024, exit 0 both runs; NOT a chains-bn3 leg | `bench-validate/bench-02-seal-bn3.md:90-92,105,121-123,148` (F-06); `bench-validate/bench-03-drift-check.md:109,138-141,160` (FS-02) |

## [Unreleased]

### Added

- `docs/CONSUMER_REQUIREMENTS.md` — PurRDF dependency pass: publication-fit
  facts, the T1-T4 requirement tickets (batch/table layer, typed zero-copy
  ingest, timestamp `Scalar`, wasm32-unknown-unknown receipt), and the
  standing table.
- Ticket T1: `batch.rs` — `Column`/`Batch`/`BatchBuilder` named column
  layer with the finish-time length law; `VarlenBuilder::len`/`is_empty`
  and `ViewVecBuilder::len`/`is_empty`.
- Ticket T2: `foreign.rs` — typed zero-copy ingest of foreign C Data
  Interface pairs (`format_of`, `primitive_view`, `is_valid`,
  `ForeignError`), bounds- and alignment-checked against caller-supplied
  buffer lengths.
- Ticket T3: `timestamps.rs` — `TimestampSecond/Milli/Micro/Nano` `Scalar`
  newtypes over i64 (format codes `tss:/tsm:/tsu:/tsn:UTC`).
- Ticket T4: `scripts/footprint.sh` accepts `WASM_TARGET`; the full
  footprint gate passes for `wasm32-unknown-unknown` (3,435 B stripped,
  CTQ-4 PASS).

### Changed

- Rename: avec→cgap across crates, C ABI, BEAM NIFs, headers, docs; version
  frozen at 26.10.29.

## [26.10.29] - 2026-10-29

Initial entry: first release ledger for the cgap columnar substrate.

### Added

- cgap-arrow v26.10.29 — freestanding, branchless in-memory columnar
  substrate; `no_std` + `alloc`, zero external dependencies. Publish-enabled
  (`publish = true`). Kernels present on disk: `bitmap`, `primitive`,
  `varlen`, `views`, `c_abi` (+ `c_abi_nested`), `silicon`, `buffer`,
  `runend`, `slice`.
- cgap-parquet v26.10.29 — branchless Parquet persistence kernels;
  `no_std`, zero deps. Publish-enabled (`publish = true`). Kernels present on
  disk: `rle`, `packing`, `unpack`, `encode_fast`, `dictionary`, `thrift`,
  `page`.
- Internal crates (`publish = false`): cgap-ffi (C-ABI membrane, Gate 3
  surface), cgap-beam (erl_nif membrane), cgap-audit (branch-audit cdylib),
  bench (Gate 2 harness), integration (integration tests), wasm-footprint
  (standalone footprint / symbol-gate artifact crate).
- Gate surface (`Makefile`): `test gate0 gate1 gate2 gate3 gates lint
  publish-check`, plus auxiliary `footprint`, `branch-audit`, `symbol-gate`,
  `gate-parquet`, `bench-all`, `bench-page`, `bench-gate`.
- Ontology-driven generation: `ggen.toml` pins `cgap-pack` from
  ggen-marketplace; four generated surfaces (2 headers under `include/`,
  2 Rust modules under `crates/cgap-arrow/src/`); Gate 0 proves byte-stable
  replay.
- Docs of record: DFLSS charter, receipts ledger, BEAM latency receipt,
  branch register, wasm footprint doc, bench floors, mutation-court backlog.
- Measurement harnesses: `scripts/branch_audit.py`, `scripts/gate3_cdata.py`,
  `scripts/footprint.sh`, `scripts/symbol_gate.py`,
  `test/beam_latency.escript`.

### Changed

- Branch-audit claim rescoped to compiled reality
  (`docs/BRANCH_REGISTER.md`): the earlier "9/9 kernels, 0 conditional
  branches" result was vacuous-by-wrapper (Class B audit defect); the
  callee-extent rewrite lands the honest claim — **inner-loop bodies are
  branch-free; latches, dispatch, and bounds paths are itemized**. Source
  and doc headers corrected (`bitmap.rs`, `lib.rs`, `silicon.rs` scoping).
- Gate 1 (`make gate1`) = CC=1 machine-code branch audit over
  `target/audit/libcgap_audit.dylib`; the mutation-court receipt lives
  behind `make gate-parquet` (requires `docs/mutation-court-backlog.txt`).
- Wording discipline: no repo claim survives as "zero conditional branches"
  without the body/latch/dispatch scoping (register R4).

### Verified

Every row was executed 2026-10-29/30 in `/Users/sac/cgap` with exit codes
captured in the receipt docs.

- `cargo test --workspace` — exit 0; 26 passed / 0 failed (arrow 15,
  parquet 11). Receipt R1.
- gate0 — `ggen sync run` x2 + freestanding C99 header compile; exit 0,
  byte-identical replay, zero warnings. Receipt R5.
- gate1 — `scripts/branch_audit.py` (extent-scoped); exit 0, 9 kernels
  located, bodies branch-free. Receipt R3 + branch register.
- gate2 — `cargo run --release -p bench`; exit 0, 58.903 GiB/s single-core,
  0 allocs, 16-lane 74.8 GiB/s. Receipt R2. Machine-local.
- gate3 — pyarrow `pa.Array._import_from_c` parity; exit 0, bitwise,
  100000 rows, 14285 nulls. Receipt R4.
- ggen-marketplace check + gate-witness courts — exit 0, cgap-pack
  qualify=ok, 5 cases, ALIVE standing. Receipt R6.
- footprint — `scripts/footprint.sh`; exit 101, BLOCKED, no artifact.
  Conflicting 10123 B PASS in `docs/wasm-footprint.txt` (Discrepancy 1).
- symbol-gate — zero-panic Jidoka, wasm-primary; no run receipt in docs,
  UNVERIFIED.
- gate-parquet — mutation-court receipt; ledger present, 114 survivor
  entries; court-run totals not receipted.
- BEAM scheduler-impact CTQ (<= 50 us) — FAIL as measured; impact
  7985.791 us, loaded max 11128.208 us; instrument floor disclosed.
  Receipt: BEAM_LATENCY.

Open items carried forward: 114 mutation-court survivors, BEAM latency FAIL,
symbol-gate scope, CTQ-4 reconciliation, Parquet kernels + minimal page
framing only (not the full file format). See `RELEASE_NOTES.md` (Known Open,
Next Milestone, Discrepancies).
