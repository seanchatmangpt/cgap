# Lane G20 — make gate0 replay + release build + lock full-replay + commit inventory

Date: 2026-11-02. Repo: /Users/sac/cgap, branch main. Read-only lane; only this
receipt written (no Makefile/ggen.lock edits needed — no replay failure observed).

## 1. Fresh `make gate0`

- Command: `make gate0` (repo root). **Exit 0.**
- Tail: `gate0: byte-stable replay proof PASS` + `gate0: header compile PASS`.
- cmp: `/tmp/cgap_gate0_run1.sha` vs `/tmp/cgap_gate0_run2.sha` — byte-identical
  (`cmp` silent, target continued to header compile, which only runs after cmp).
- Four surfaces witnessed:
  - `01d828b391ca739b5d482dcf76df8b841b4b356412ac8b36d49c697dfbef4f88  include/cgap_c_data.h`
  - `bf7f5b54215c81d30f11ec9cbdb1f60bb72a162b978eed506a10a6da49bf2bc1  include/cgap_format_codes.h`
  - `96c67c577582d4193513aab686b7a3f7f7a638abe7e12db69004e68f7f5c09c3  crates/cgap-arrow/src/generated_layout.rs`
  - `e962af1843a673550397982ba7d426355cd7025536479596bf84b9ae797a0a0e  crates/cgap-arrow/src/generated_version.rs`

## 2. Workspace release build (member count audited below)

- Command: `cargo build --workspace --release`. **Exit 0**, `Finished release
  profile [optimized] target(s) in 13.06s`.
- Warnings: 6 lines matching `^warning` in the full output (includes
  `warning: bench (manifest) generated 3 warnings` — dep-removal hints for
  cgap-storage). Workspace green, warnings tolerated per Makefile lint policy.
- Member count: `cargo metadata --no-deps` = **23 packages**, NOT 22.
  Members: cgap, cgap-arrow, cgap-core, cgap-blake3, cgap-gate,
  **cgap-branchless** (new this campaign), cgap-storage, cgap-git,
  cgap-parquet, cgap-bn3, cgap-p28, cgap-crypto, cgap-ffi, cgap-audit,
  cgap-beam, cgap-gpu, bench, cgap-integration, cgap-bont, cgap-bosym,
  cgap-court, cgap-fibo, cgap-cloud.
- csct: correctly ABSENT from `cargo metadata` (standalone confirmed). csct
  exists at `crates/csct/` with its own Cargo.lock/target.
- Divergence recorded: CLAUDE.md says "22-member workspace"; reality is 23
  members (cgap-branchless joined the workspace this campaign). Doc yields to
  reality; CLAUDE.md line is stale, not corrected by this lane (out of scope).

## 3. Full-replay leg (ggen.lock)

- Procedure note: `make gate0` itself re-locks via plain `ggen sync run` at the
  ROOT (Makefile:45-55); it never touches `crates/cli/ggen.lock`. My first
  full-replay attempt wrongly `rm`'d `crates/cli/ggen.lock`; restored via the
  CLI's own procedure (`rm Cargo.toml && ggen sync run && python3
  manifest_patch.py`, both exit 0). Post-restore `git status` shows
  crates/cli/ggen.lock + Cargo.toml CLEAN vs HEAD — zero drift.
- Root lock replay, per gate0's own procedure:
  - `rm -f ggen.lock && ggen sync run` — exit 0, twice.
  - Lock format: `[packs.cgap-pack] content_hash = "blake3:22ba9f10057187b369b4b3637a1e2e87a0b652681aedf028de1c36ebd1f07761"`
  - **This lock format carries `content_hash`, not `graph_hash_hex`** — the
    dispatch's expected field does not exist in this ggen version's lock.
    Divergence recorded. Byte-identity proven by full-file sha256 of the lock
    instead: `/tmp/g20_lock1.sha` vs `/tmp/g20_lock2.sha` → `REPLAY-BYTE-IDENTICAL`.
  - Re-lock content_hash matches the known state: blake3:22ba9f10… (vs HEAD
    e085b8d1… — confirmed `git diff --stat -- ggen.lock` = 1 line change).
- Final `make gate0` after the rm leg: **exit 0** (lock re-locked correctly).

## 4. Commit inventory (193 paths total: 83 modified, 111 untracked; read via git status --porcelain, READ-ONLY)

Grouped list for the coordinator — counts are per git status snapshot 2026-11-02.

### (a) csct crate + docs + lane receipts (17 paths)
- `crates/csct/` (whole dir untracked: Cargo.toml, Cargo.lock, src/, tests/, target/ —
  target/ is lease debris, gitignore backstop)
- `docs/csct/`, `docs/audit/` (2 dirs)
- waves: csct-lane-01..14 receipts + csct-wave-register (15 files under docs/jira/v26.11.2/waves/)

### (b) cgap-branchless + gates (4 paths)
- `crates/cgap-branchless/` (untracked dir; NOTE: contains wrapper_riscv32imc.o —
  delete before staging, see group (f))
- `scripts/verify_disassembly_invariants.py`, `scripts/verify_dead_library.py`,
  `scripts/dead_library_exceptions.json`

### (c) .claude/ hooks+agents+rules+settings (17 paths)
- agents: court-auditor, gate-runner, lane-runner, reviewer (4 M)
- hooks: ban_generic_agents.py, flip-tests.sh, guard-generated.sh, guard-secrets.sh,
  mtime-restore-guard.sh, publish-guard.sh (6 M)
- rules: bench.md, kernels-no-std.md, publish.md (3 M)
- skills: bench-chains, release, run-gates SKILL.md (3 M)
- settings.json (1 M)

### (d) root manifests + ggen.lock + generated + scripts (33 paths)
- Cargo.toml, Cargo.lock, ggen.lock, CLAUDE.md, .gitignore (5 M)
- include/cgap.h, generated/capability-table.txt (2 M)
- scripts (M): boundary_gate.py, projection_coverage.py, registry_sweep.py,
  release_check.py, shacl_negative_corpus.py, shacl_negative_corpus_b.py, zero_string_gate.py (7)
- scripts (??): enclave_budget_gate.py, gate_cargo_test.py,
  generate_enterprise_cgap.py, verify_ost_design_around.py (4)
- crates/cgap/ (M: Cargo.toml, lib.rs; ??: src/pipeline.rs, src/bin/, benches/, tests/) (7)
- crates/integration/Cargo.toml (1)
- crate sources not in other groups (M): bench (4: Cargo.toml, benches/chains-bn3.rs,
  dred_churn.rs, src/bin/cpk_study.rs), cgap-arrow (4 M), cgap-bn3 (9), cgap-core (2),
  cgap-crypto/lease.rs, cgap-ffi (3: sovereign.rs, xip.ld), cgap-gate (2),
  cgap-git (9), cgap-parquet (3 M), cgap-storage (4 M+2 tests M) — see raw list
- untracked tests/benches across crates: bitmask_swar.rs, validate.rs,
  validate_minimal.rs, mat_closure.rs, repair20_xorfilter_gate.rs,
  tracking_alloc.rs, enclave_128k.ld, c_header_abi_court.rs, fa9_commit_gating.rs,
  delta_refusal_court.rs, envelope_decimal_court.rs, seal_rows_bound_court.rs,
  stats_skip_court.rs, closure_alloc_court.rs, dred_cascade.rs, dred_swar_wire.rs (16)

### (e) docs/jira receipts + registers (89 paths)
- Modified (11): 02-pareto-levers.md, 03-ost-design-around-register.md, HANDOFF.md,
  patent/claims.txt, waves/{bp-4-rules, p2-asm-0blr-census, pub-w3-witness,
  pub-w5-witness, registry-sweep-table.json, sm-4-hooks}
- Untracked: 14-dflss-charter.md, 15-ctq-tree.md, 16-sipoc.md, 17-c4-architecture.md,
  DEFECT-REGISTER.md, adversarial/, closure/, convergence/, countermeasures/,
  exhaustive/, explore/, hardening/, lanes/ (this receipt included),
  mega-bench/, paper/, repairs/, research/ (7 files), waves/ (~45 receipt files),
  patent/ost_report.json

### (f) debris-to-DELETE-never-commit (2 paths, both located this session)
- `/Users/sac/cgap/libae8_dbg.rmeta` — 2377 bytes, 2026-11-02 11:12
- `/Users/sac/cgap/crates/cgap-branchless/wrapper_riscv32imc.o`

## 5. Known pre-commit blockers (re-verified this session, not hearsay)

1. `python3 scripts/verify_disassembly_invariants.py` — **exit 2** (run this
   session): `[FAIL] emit-asm build failed (exit 101): could not compile
   cgap-branchless (lib) due to 2 previous errors`. G01/G02/G03 are in flight
   on cgap-branchless; the gate aborts at the emit-asm build step before
   reaching symbol checks. Coordinator: re-run after those lanes drain.
2. repair-09 stale warning at `crates/cgap-branchless/src/scan.rs:122` —
   confirmed the site (`segmented_prefix_sum_u32x8`, `#[inline(always)]`,
   unrolled reset-mask body at lines 118-126). G01 is fixing; not touched by
   this lane.

## Standing

- make gate0: ALIVE (exit 0, twice this session, byte-identical replays).
- release build: ALIVE (exit 0, 6 warning lines).
- ggen.lock full-replay: ALIVE (2x exit 0, sha256-identical locks;
  content_hash blake3:22ba9f10… re-confirmed post-replay).
- Commit inventory: complete per snapshot; coordinator owns staging.
- Blockers: recorded with live evidence; standing BLOCKED (owned by G01/G02/G03).
