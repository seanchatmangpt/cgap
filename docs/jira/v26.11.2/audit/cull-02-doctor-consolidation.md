# cull-02-doctor-consolidation

CULL-02 doctor-surface audit (2026-11-02). Reconciles with DXD-02's
decision: extend scripts/doctor_agent_lanes.py as the ONE canonical
doctor; crates/cli/src/bin/doctor.rs is reference-only (ggen regen
hazard). Not re-litigated.

## Inventory

S1. cli doctor.rs — 10 checks (c01 toolchain, c02 wasm-targets, c03
git, c04 b3sum, c05 pyshacl, c06 workspace-check, c07 scripts,
c08 receipts, c09 lsl-ratified, c10 dirty-tree), 0/1/2 max-exit,
--json. v26.10.31 wave-10 artifact.
S2. doctor_agent_lanes.py — 4 checks (stale-server, orphan-leases,
workspace-count, dirty-tree census) + DX1_* env overrides.
CANONICAL per DXD-02.
S3. verify_lsp_branchless.py staleness guard — NOT a doctor check;
content-hash-class witness (strings|grep ANDON_REFUSED) vs the
doctor's mtime class. Complementary. KEEP in place.
S4. flip-tests.sh — zero doctor content. Out of scope.

## Duplication analysis

Duplicated verbatim: none. Semantic overlaps:
- dirty-tree: doctor.rs c10 vs lanes (d) — SAME input (git status
  --porcelain), lanes' per-dir census strictly richer. c10 DIES.
- newest_mtime helper duplicated in both languages, different
  subjects — both survive; Python helper canonical.
Unique to doctor.rs: toolchain, wasm-targets, git, b3sum, pyshacl,
workspace-check, scripts, receipts, lsl-ratified.
Unique to lanes: orphan-leases, workspace-count, stale-server.
DXD-02 adds 6: riscv-target, profdata, go-version, ffi-dylib,
shm-rule, thermal.

## Consolidation map

MIGRATES (port to Python idiom, env-override + flip arm each):
c01/c03 trivial; c02 rustup list (DXD-02's riscv check same shape);
c04 shutil.which; c05 import-probe WARN; c06 heaviest (WARN on
warnings; runtime vs DXD-02's ~5s budget — flag: gate behind a
flag or split out); c07 10-name list is v26.10.31-frozen — migrate
as WARN-degrade or park; c08/c09 same scoping caveat.
DIES: doctor.rs c10 (subsumed). doctor.rs binary demotes to
redirect stub (5-line shim exec'ing the Python script, header
naming DXD-02 + CULL-02; caveat: confirm bin/doctor.rs survives
ggen regen — if generated, the stub dies at next sync, acceptable).
STAYS: S3, S4.

## Exit ABI

Identical already (0/1/2 max). DXD-02 adds: missing-but-expected =
WARN never RED — ported checks downgrade accordingly.

## Verdict

16 checks across 2 live surfaces today. Post-consolidation: ONE
canonical script (4 DX-1 + 6 DXD-02 + up to 9 ported; c10 culled)
= 19 live, 2 call paths -> 1.

## Open questions (parent decisions, not blockers)

1. c07/c08/c09 version-frozen paths — WARN-degrade migration vs park.
2. c06 runtime vs the ~5s budget — flag-gated or split.

## See Also

audit/dxd-02-doctor-design.md · audit/dxd-09-dual-runtime-env.md ·
scripts/doctor_agent_lanes.py
