# W-COMMITPLAN-V2 — refreshed integration commit plan (plan-only, lane F11)

Date: 2026-11-02. Repo: /Users/sac/cgap @ main 9a730320. PLAN ONLY: zero
git writes performed or authorized for this lane; the coordinator owns
every commit. Supersedes waves/w-commitplan.md (2026-11-01) as the
integration plan of record. Style: <=100-char lines. Never `git add -A`
— explicit paths per group, manifests-first / pack-last ordering.
No Process.sleep anywhere in this plan.

## Inputs

- W1 census (221 dirty/untracked files at census time; 10 proposed
  groups; `libae8_dbg.rmeta` EXCLUDED as build debris — never commit;
  coordinator deletes the lease at integration). Census receipt not yet
  on disk in waves/ as of this session (UNVERIFIED pointer; the
  221-file/10-group facts are taken from dispatch, not re-derived).
- Post-census additions: W16's enterprise generator
  `scripts/generate_enterprise_cgap.py` (untracked on disk, verified
  via `git status --porcelain` this session) and the F-lane outputs
  (crates/cgap-branchless/ tree, cgap-core tracking_alloc, cgap-arrow
  SWAR validate/bitmask courts, bench mat_closure).
- On-disk re-count this session: `git status --porcelain` -> 138
  entries (83 `??`, 55 `M`), exit 0. The delta 221 -> 138 is recorded
  as a [Registered divergence]: files landed, pruned, or
  never-written since the census ran; groups below are re-grounded
  against the live 138-entry status, not the census snapshot.

## Commit groups (execute in numeric order)

### 1. MANIFESTS FIRST (workspace must be green at this commit)

- Cargo.toml
- Cargo.lock
- crates/cgap-branchless/Cargo.toml (new crate manifest)
- crates/bench/Cargo.toml
- crates/cgap-arrow/Cargo.toml
- crates/cgap-gate/Cargo.toml
- crates/cgap/Cargo.toml
- ontologies/{cloud,fibo}/Cargo.toml (if dirty at commit time)

### 2. NEW CRATE + F-LANE KERNEL (cgap-branchless)

- crates/cgap-branchless/ (entire tree: src/, tests/, benches/ as
  scaffolded by the F-lane; explicit `git add crates/cgap-branchless`)
- crates/cgap-core/src/tracking_alloc.rs
- crates/cgap-arrow/src/validate.rs
- crates/cgap-arrow/src/bitmask.rs
- crates/cgap-arrow/tests/bitmask_swar.rs
- crates/cgap-arrow/src/{ipc,lib}.rs

### 3. KERNEL LEVERS (modified sources, per-w6/w5 receipts)

- crates/cgap-bn3/src/{arena,builtins,lib,noalloc,project,sigil,unify}.rs
- crates/cgap-bn3/tests/bgp_hop_courts.rs
- crates/cgap-git/src/{dag,deflate,inflate,ladder,lib,pack,pack_write}.rs
- crates/cgap-git/tests/{fuzz_codec,delta_refusal_court,envelope_decimal_court}.rs
- crates/cgap-parquet/src/{footer,mmap_writer,scan}.rs
- crates/cgap-parquet/tests/{seal_rows_bound_court,stats_skip_court}.rs
- crates/cgap-storage/src/dred.rs
- crates/cgap-storage/tests/dred_cascade.rs
- crates/cgap-storage/tests/court_bitplane_seen.rs
- crates/cgap-crypto/src/lease.rs
- crates/cgap-gate/src/penstock.rs
- crates/cgap-ffi/src/sovereign.rs
- crates/cgap-ffi/{xip.ld,enclave_128k.ld}
- crates/cgap-ffi/tests/fa9_commit_gating.rs
- crates/cgap/src/{lib,pipeline}.rs crates/cgap/src/bin/ crates/cgap/tests/
- crates/cgap/benches/

### 4. BENCHES

- crates/bench/benches/{chains-bn3,dred_churn}.rs
- crates/bench/benches/mat_closure.rs (new)

### 5. SCRIPTS + GATES (incl. W16 post-census addition)

- scripts/boundary_gate.py
- scripts/zero_string_gate.py
- scripts/projection_coverage.py
- scripts/registry_sweep.py
- scripts/release_check.py
- scripts/generate_enterprise_cgap.py   (W16; untracked, verified)
- scripts/{dead_library_exceptions.json,enclave_budget_gate.py,
  gate_cargo_test.py,verify_dead_library.py,
  verify_disassembly_invariants.py,verify_ost_design_around.py}
- .gitignore

### 6. CLI (standalone ggen crate; only if dirty at commit time)

- crates/cli/Cargo.toml crates/cli/Cargo.lock
- crates/cli/ontology/cgap-cli.ttl
- crates/cli/.clap-noun-verb/ocel.json
- crates/cli/src/bin/doctor.rs

### 7. GENERATED (regenerate, never hand-edit)

- generated/{bench-registry.txt,campaign-board.md,court-table.md,
  manifest-facts.txt}
- ggen.lock (CL1 re-lock: pins blake3:22ba9f10..., sha256
  a1356da1...f36 per closure/lane-01-lockfile-sync.md:13-15)
- ontology/cgap.ttl projections (repo-root `ggen sync run`)

### 8. DOCS + RECEIPTS (pack class, near-last)

- docs/jira/v26.11.2/** (all untracked dirs: adversarial/, closure/,
  convergence/, countermeasures/, exhaustive/, explore/, hardening/,
  mega-bench/, repairs/, research/, paper/, patent/ost_report.json,
  plans/, waves/*, 14..17-*.md) and modified docs/ files
- HANDOFF.md (SS5 post-CL1 addendum, lane F11) and this file

### 9. PACK LAST (only on explicit operator publish lift)

- Publishes remain FROZEN (HANDOFF SS4/SS5.4). No version bump, no
  `cargo publish` in any commit of this integration; G1 stays BLOCKED
  on the operator lift + release_check.py REQUIRED_VERSION repair
  (HANDOFF SS3 D-4/D-5).

## Cleanup at integration (coordinator)

- Delete lane build-root leases under /tmp (w-commitplan.md:70-76 list
  still applies; verify absence, never `rm -rf` by hand through the
  permission layer — shutil.rmtree via python).
- `libae8_dbg.rmeta` (repo root, untracked): build debris — delete,
  never commit (W1 census exclusion, restated).
- After restoring any sandboxed file into a rebuilt tree: bump mtime
  or `cargo clean -p <crate>` (stale-mtime law).

## Verification ladder before each commit boundary (coordinator)

1. `cargo build --workspace` exit 0 (after group 1; again before 7).
2. `cargo test -p cgap-court` exit 0 (14 passed baseline,
   w-commitplan.md:11).
3. `python3 scripts/boundary_gate.py --strings` and `--deps` exit 0.
4. `python3 scripts/zero_string_gate.py` + `--self-check` exit 0.
5. `make gate0` exit 0 (post-CL1; closure/lane-01:18-20).
6. Final: `git status --porcelain` empty except lease debris and the
   frozen publish set.

<!-- json-footer
{"lane":"F11","receipt":"docs/jira/v26.11.2/waves/w-commitplan-v2.md",
"files_written":["docs/jira/v26.11.2/HANDOFF.md",
"docs/jira/v26.11.2/waves/w-commitplan-v2.md"],
"commands":["git status --porcelain (exit 0, 138 entries: 83 ??, 55 M)",
"ls waves/ dirs (exit 0; 84-receipt hardening-family count)",
"ggen.lock sha256 re-read from closure/lane-01 (a1356da1...f36)"],
"courts_run":[],
"standing":"ALIVE (reads+exits this session; plan-only lane, no builds)",
"git_writes":0,
"divergences":["W1 census receipt not on disk (221/10-group facts from dispatch)",
"live status 138 entries vs census 221 [registered]",
"hardening-family receipt count 84 on disk vs dispatch '~100' [registered]",
"W8 deferred-register full sweep not yet executed on disk (loud)"]}
json-footer -->
## Refresh (2026-11-02, lane MV-8, re-derived from disk)

Fresh re-census: `git status --porcelain` -> 252 entries (100 `M`,
152 `??`), exit 0. Prior session-verbatim counts above (138 entries;
W1 census 221) are stale-as-of-now per the receipt-cited-counts law.
Debris exclusion (delete, NEVER commit): `libae8_dbg.rmeta` (repo
root, 2,377 B, mtime Nov 2 11:12, `ls -la` exit 0) and
`crates/cgap-branchless/wrapper_riscv32imc.o` (6,648 B, mtime Nov 2
12:08) — R8 class.

### R.1 MANIFESTS FIRST (workspace green at this commit)

- `Cargo.toml`, `Cargo.lock`, `ggen.lock`, `.gitignore`, `justfile`,
  `CLAUDE.md`, `include/cgap.h`, `generated/capability-table.txt`,
  `ontologies/cloud/tests/courts.rs` (9 modified, exit 0 read).

### R.2 .claude/ harness (gate + hooks + agents)

- 18 entries: 17 modified (agents x4, hooks x9, rules x3,
  settings.json) + untracked `.claude/hooks/branchless-court.sh`.

### R.3 csct crate + docs

- `crates/csct/` + `docs/csct/` (2 untracked dir entries).

### R.4 cgap-git lanes (pack-last, biggest group)

- 49 entries: 16 modified (Cargo.toml, src/{dag,deflate,inflate,
  ladder,lib,pack,pack_write}, tests x8) + 33 untracked (src x15:
  a2a_swarm admission_gate commit_synth delta_resolver egress_ring
  ingress_ring mcp_driver object_store octopus_merge pack_demux
  patent_court pkt_line ref_store retract tree_synth; tests x18).
  cargo metadata confirms cgap-git is a root member.

### R.5 bench fixes

- 8 entries: 7 modified (Cargo.toml + benches/{bgp,chains-bn3,
  chains-parquet,chains-seal,dred_churn}.rs + bin/cpk_study.rs) +
  untracked `crates/bench/benches/mat_closure.rs`.

### R.6 docs family (jira + audit)

- 92 entries: 91 under docs/jira (10 modified: 02/03 registers,
  HANDOFF.md, patent/claims.txt, waves/{bp-4-rules,p2-asm-0blr-census,
  pub-w3-witness,pub-w5-witness,registry-sweep-table.json,sm-4-hooks};
  81 untracked: lanes/, adversarial/, audit/, closure/,
  countermeasures/, convergence/, exhaustive/, explore/, hardening/,
  mega-bench/, paper/, repairs/, research/, waves/ additions,
  14-17 DFLSS docs, DEFECT-REGISTER.md, patent/ost_report.json)
  + untracked `docs/audit/` (release-blockers register home).

### R.7 scripts + tests/gates

- 19 entries: 10 modified scripts + 8 untracked scripts (verify_*
  gates, generate_* generators, dead_library_exceptions.json,
  run_coverage.sh) + untracked `tests/gates/`.

### R.8 remaining kernel crates (54 entries incl. branchless)

- arrow 8 (5M+3?), bn3 11 (9M+2?), parquet 7 (5M+2?), storage 7
  (4M+3?), ffi 6 (2M+4?), umbrella cgap 6 (2M+4?), core 3 (2M+1?),
  gate 2 M, p28 1 M, crypto 1 M, integration 1 M,
  cgap-branchless/ whole-dir untracked (1 entry; its
  wrapper_riscv32imc.o debris lives INSIDE this dir entry — R.9's
  file-level exclusion applies without double-counting).

### R.9 Debris to DELETE at integration (never commit)

- `libae8_dbg.rmeta`, `crates/cgap-branchless/wrapper_riscv32imc.o`.
  Verify absence post-integration; `rm -rf` is denied by the
  permission layer — python shutil.rmtree for dirs, unlink for files.
