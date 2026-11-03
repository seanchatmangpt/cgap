# DOD-04: unsafe soundness census + no_std SRAM gate re-run

Lane: DOD-04 · 2026-11-02 · canonical checkout /Users/sac/cgap ·
read-only census + gate re-runs; sole write = this file. No git commands
run. Build roots: /tmp/dod04-target (lease, to be deleted at lane end).

## 1. Unsafe census (crates/*/src/**.rs)

Method: python census over `crates/*/src/**/*.rs`. A site is a line
containing `unsafe` with a keyword form (`unsafe fn/impl/trait/{/[
/extern/static` or `unsafe {`). A site counts JUSTIFIED when a
`SAFETY`-family comment appears within +/-12 lines; otherwise FINDING
(unannotated at that granularity). Window method => approximate at site
granularity; per-file finding list on disk at /tmp/dod04_findings_w12.txt.

TOTALS: 601 unsafe keyword sites; 206 within-window-justified; 395
findings (no SAFETY-family comment in the +/-12-line window).

Per-crate (sites / findings):

| crate | sites | findings |
|---|---|---|
| audit | 38 | 38 |
| bench | 25 | 25 |
| cgap-arrow | 84 | 84 |
| cgap-beam | 105 | 105 |
| cgap-blake3 | 62 | 62 |
| cgap-bn3 | 8 | 8 |
| cgap-bont | 1 | 1 |
| cgap-bosym | 5 | 5 |
| cgap-branchless | 2 | 0 |
| cgap-core | 17 | 13 |
| cgap-crypto | 1 | 0 |
| cgap-ffi | 48 | 48 |
| cgap-fleet-ops | 1 | 0 |
| cgap-gate | 5 | 5 |
| cgap-git | 8 | 0 |
| cgap-gpu | 4 | 4 |
| cgap-ocel2 | 1 | 0 |
| cgap-p28 | 6 | 6 |
| cgap-parquet | 37 | 27 |
| cgap-storage | 1 | 1 |
| cgap (root) | 7 | 7 |
| cli | 3 | 3 |
| wasm-footprint | 8 | 7 |

Total raw grep hits (any `unsafe` token incl. comments): 647.

Caveat on the 395: the window census UNDERCOUNTS justification by
construction — many sites are justified by module-level `//!` contracts
or fn-level `///` docs rather than an adjacent `// SAFETY:`, e.g.
`audit_mask_words` (crates/audit/src/lib.rs:16-26, contract in the `///`
doc) and the NEON multi-chunk kernels
(crates/cgap-blake3/src/multi_chunk.rs, module-doc contract). The census
is a loud surface, not a verdict. A findings list is retained at
/tmp/dod04_findings_w12.txt (lane-tmp, not committed).

Named concentrations, corroborated on disk this session:

- Cell64 single-writer law: crates/cgap-ffi/src/stats.rs:195
  (`// SAFETY: single-writer law (module doc) ...`), `unsafe impl Sync
  for Cell64` at stats.rs:198; export cgap_stats_snapshot at stats.rs:369.
- PaddedIndex/const-assert family:
  crates/cgap-parquet/src/bitslice.rs:160 (`// SAFETY: repr(C) with
  fixed layout; CRC_AT bytes are exactly ...`).
- ref_store cfg-ladder (ADV-11): crates/cgap-git/src/ref_store.rs:232-240
  `compile_error!` gate naming both cfg families; SAFETY-grade contract
  in the module doc ref_store.rs:32-37.
- Ring indexing: cgap-beam spsc_ring/shm family (105 sites in cgap-beam,
  the largest crate concentration; justification lives in module docs).

## 2. SRAM gate legs (scripts/verify_cscb_sram.py)

Leg 1 baseline: `python3 scripts/verify_cscb_sram.py` -> EXIT 0.
Measured 36048 B (.text+.rodata+.data+.bss families) <= 131072 B hard
bound; headroom 95024 B. Script prints an empirical-baseline comparison
(report-only, NOT gated): baseline 39184 B -> measured 36048 B
(delta -3136 B), cite OVERNIGHT-LOOP empirical riscv32imc freestanding
witness. Stack estimate: max leaf frame 2048 B over 9 sp-adjusting
prologues, static analysis only, UNVERIFIED as a worst-case chain bound.
Toolchain: cargo 1.101.0-nightly (cad07cf8e 2026-10-23) / rustc
1.101.0-nightly (12433c307 2026-10-24); target riscv32imc installed.

Leg 2 refusal: `python3 scripts/verify_cscb_sram.py --subject bn3` ->
EXIT 1, typed refusal BN3_ENCLAVE_REFUSED: static noalloc state-table
charge 6520984 B (NaMatches 2162696 B + NaEngine 4358288 B, from
crates/cgap-bn3/src/noalloc.rs: NA_MAX_ATOMS=8 NA_MAX_MATCHES=2048
NA_MAX_VARS=128 NA_MAX_ROWS=8192) > 131072 B hard bound = 49.75x
ceiling; red-12 topology.

## 3. riscv32imc legs (cgap-git), invoked from repo root

Leg 3 plain: `CARGO_TARGET_DIR=/tmp/dod04-target cargo build -p cgap-git
--target riscv32imc-unknown-none-elf --no-default-features` -> EXIT 101
via scripts/run_checked.py, typed compile_error! at
crates/cgap-git/src/ref_store.rs:232 naming the missing
`implcell_critical_provided` cfg (ADV-11 loud-until-bound gate fires).
Log: /tmp/dod04_riscv_plain.log.

Leg 4 opt-in: `run_checked.py -- env RUSTFLAGS='--cfg
implcell_critical_provided' cargo build -p cgap-git --target
riscv32imc-unknown-none-elf --no-default-features` -> EXIT 0 (dev
profile Finished). Log: /tmp/dod04_riscv_cfg.log.

Toolchain-resolution lesson (from the dead prior dispatch, applied):
cargo invoked from REPO ROOT every leg; rustup resolves toolchains from
cwd. A first piped run showed `exit=0` from `tail` — the pipe masked
cargo's 101; re-run un-piped via scripts/run_checked.py
(docs/jira/v26.11.2/lanes/lane-dx-05-run-checked.md). rc-through-pipe is
a known hazard; run_checked is the method of record.

## 4. Honest scoping note

36,048 B is the gated blake3+branchless kernel SLICE only (minimum entry
surface, headroom 95,024 B inside the 131,072 B physical envelope) — it
is NOT a whole-workspace SRAM figure. bn3 is REFUSED at admission at
49.75x the ceiling (6,520,984 B vs 131,072 B) — the bn3 engine cannot
enter the SRAM enclave under any current capacity table; admission
attempt is a typed gate failure, exit 1. Both scopes stated together;
neither number may be quoted without its scope.

DIVERGENCE from dispatch: the dispatch said to read
11-measured-claims.md "row 3" for the SRAM baseline; row 3 of that table
is the root workspace build claim (exit 0, 1.84 s,
waves/w2-09-build-baseline.md:14) and the SRAM 36048 B number appears
NOWHERE in 11-measured-claims.md (grep SRAM/sram: zero hits). The SRAM
gate number is therefore UNREGISTERED in the measured-claims table and
this receipt is the only on-disk witness for this session's 36048 B run.

Standing: ALIVE for every leg above (command + exit witnessed this
session). Census classification is PARTIAL (window-based, stated
method). Per-crate findings lists: /tmp/dod04_findings_w12.txt.
