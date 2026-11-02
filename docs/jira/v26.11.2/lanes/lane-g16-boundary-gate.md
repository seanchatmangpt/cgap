# Lane G16 - boundary_gate.py sealers receipt (v26.11.2)

Date: 2026-11-02. Lane: G16. Repo: /Users/sac/cgap (main).
Files written: this receipt only. scripts/boundary_gate.py untouched
(no defect found -- see sections 3 and 4).

## 1. --deps run (command + exit, this session)

Command: python3 /Users/sac/cgap/scripts/boundary_gate.py --deps
Exit: 0

Per-crate E= table (verbatim gate stdout, 15 rows + final line):

    crates/cgap-core           E=0  runtime_deps=0
    crates/cgap-storage        E=0  runtime_deps=1
    crates/cgap-bont           E=0  runtime_deps=1
    crates/cgap-bosym          E=0  runtime_deps=0
    crates/cgap-bn3            E=0  runtime_deps=1
    crates/cgap-gate           E=0  runtime_deps=1
    crates/cgap-crypto         E=0  runtime_deps=1
    crates/cgap-p28            E=0  runtime_deps=2
    crates/cgap-court          E=0  runtime_deps=0
    crates/audit               E=0  runtime_deps=2
    crates/cgap-arrow          E=0  runtime_deps=0
    crates/cgap-blake3         E=0  runtime_deps=0
    crates/cgap-git            E=0  runtime_deps=0
    crates/cgap-parquet        E=0  runtime_deps=0
    ontologies/fibo            E=0  runtime_deps=1
    DEPS GATE: E=0 runtime externals across 15 covered crates (S6.4)

## 2. Circular deps + no_std core isolation (from the gate's own report)

- Core isolation: the row "crates/cgap-core E=0 runtime_deps=0" proves
  cgap-core declares ZERO [dependencies] of any kind (parse_runtime_deps
  counts total [dependencies] entries into runtime_deps; 0 = no edges at
  all), so cgap-core cannot participate in any dependency cycle and is
  mechanically isolated as the no_std leaf. Same-zero siblings:
  cgap-bosym (0), cgap-court (0), cgap-arrow, cgap-blake3, cgap-git,
  cgap-parquet (all 0).
- Circular deps: the gate does NOT compute a cycle check. It proves
  E=0 runtime EXTERNALS per covered crate; internal path deps between
  covered crates are counted (runtime_deps column) but never checked
  for cycles. So "zero circular deps" is witnessed here only via the
  leaf rows above (zero-edge crates cannot cycle) plus acyclicity of
  the workspace as a whole, which cargo enforces at build time. This is
  a coverage note, not a defect in this run.

## 3. crates/csct coverage question (answered mechanically)

Mechanism (read from scripts/boundary_gate.py:863-879): COVERED_CRATES
is a HARDCODED 15-entry tuple of repo-relative paths. The gate does NOT
enumerate from cargo metadata -- membership is by literal tuple entry,
fail-closed on a missing dir (covered_crate_dirs exits 2). Therefore:

- csct's exclusion from the gate is a MANUAL list decision, not a
  mechanical consequence of it being a standalone workspace. Cargo
  metadata would also exclude it (own [workspace] at
  crates/csct/Cargo.toml:11), but the gate never consults metadata.
- Adding it WOULD be a one-line change: append "crates/csct" to the
  COVERED_CRATES tuple. parse_runtime_deps reads crates/csct/Cargo.toml
  directly; its sole dep cgap-blake3 = { path = "../cgap-blake3" }
  resolves inside the repo, so it would report E=0 and pass. Not done
  by this lane: coverage-set policy is the coordinator's call (G16 is
  authorized to edit the gate only on a real defect; a coverage-set
  widening is policy, not a defect repair).

## 4. Non-vacuity status of --deps

The gate has three committed self-check arms -- --shim-inert (negative
control for the mutation campaign, main(): args.shim_inert),
--cross-gate-court (seam-neuter flip for --strings, mode_cross_gate_court),
and --interference-court (cmp-swap inversion for verify_unmutated,
mode_interference_court). NONE of them exercises --deps: there is no
poison arm that proves the E=0 gate can fail. OPEN finding (per dispatch,
no self-check added): the future arm should, on a /tmp sandbox copy of
the tree,

  mutation: append 'poison = "1.0.0"' (a registry-version dep, the
  parse_runtime_deps external class) to [dependencies] of a sandbox copy
  of crates/cgap-core/Cargo.toml; expected flip: exit 0 -> exit 1 with
  "crates/cgap-core ... EXTERNAL: poison" and the DEPS GATE FAILED line
  (mode_deps, boundary_gate.py:1209-1212).
  inert control: append a repo-internal path dep; must stay exit 0.

Note --deps refuses on a missing covered crate dir (exit 2) and an
unparsable manifest (exit 1), so some failure modes are fail-closed
already; what is unproven is the E>0 flip itself.

## 5. --strings run (command + exit, this session)

Command: python3 /Users/sac/cgap/scripts/boundary_gate.py --strings
Exit: 0
Verdict line: "STRINGS GATE: 0 hits across 15 covered crates (S6.4);
A07 register v2 consumed (sha 7778078dfe11, 6 canonical patterns, 0
allowlist entries); CROSS-GATE: canonical A07 artifact scan GREEN
(exit=0, scanned=88, hits=0, emitter_hits=2)"

## 6. Standing

ALIVE for: --deps exit 0, --strings exit 0 (both run this session,
commands above). UNVERIFIED: any mutation-flip claim for --deps (the
poison arm of section 4 is a proposal, never run). The gate's mutation
non-vacuity (--shim-court-only) was NOT run this lane; standing for the
campaign modes is unchanged from prior receipts.

## 7. Divergences / fences

- Divergence: dispatch said the csct question might be "a one-line
  manifest-in-gate change" -- precisely, it is a one-line append to the
  hardcoded COVERED_CRATES tuple in boundary_gate.py (no manifest in the
  gate; the gate reads each crate's own Cargo.toml).
- Divergence: the coordinator-witnessed 15-crate count reproduces
  exactly; no drift.
- Fence: this receipt writes no code; the must-not-regress surfaces are
  the gate's own green verdicts -- both re-run above with exit 0, so no
  regression is possible from a docs-only diff.
