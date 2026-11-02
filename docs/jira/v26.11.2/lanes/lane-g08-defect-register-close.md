# Lane G08 receipt — defect register close

2026-11-02. Repo /Users/sac/cgap @ main 9a730320. Evidence lane: no source
edits, no git writes. Files written (exactly the two dispatched):
- docs/jira/v26.11.2/DEFECT-REGISTER.md (created, 127 lines)
- docs/jira/v26.11.2/waves/lane-g08-defect-register-close.md (this receipt)

## Verdicts

| Item | Standing | Evidence |
|---|---|---|
| WG2/G2-PARETO rollup L1/L2/L4/L5 | OPEN | HANDOFF.md:128 current; no post-dating G2 receipt; L3-only CLOSED via repair-06:58-81 |
| FA10 102,187 B maximal consumer | CLOSED (re-derived) | build exit 0 + llvm-size 102,389 B this session |
| FA1 bench-doc framing | OPEN (wording rec.) | chains-ingress.rs:24; FA1 :13,36,88-89 |
| FA2 / Cpk n=1 | OPEN (G09 pointer) | smoking-gun SS3.3; FA2 :45-50, :186 |
| RV1 per-lane semantics | OPEN (LAW RV1-PERLANE registered) | FA6 :73-92, :94-101 |

## Commands + exits (this session)

- `cd /tmp/fa10-max-consumer && CARGO_TARGET_DIR=/tmp/g08-target cargo
  +nightly build --release --target riscv32imc-unknown-none-elf` -> exit 0
  (background task byza9kizl, completed 12:55).
- `llvm-size -A /tmp/g08-target/riscv32imc-unknown-none-elf/release/
  fa10-max-consumer` -> exit 0; .text 102000 + .rodata 389 = 102,389 B
  (vs FA10's 102,187 B, +202 B nightly drift, still 22% headroom under
  131,072 B). Tool: ~/.rustup/toolchains/nightly-aarch64-apple-darwin/
  lib/rustlib/aarch64-apple-darwin/bin/llvm-size (CLAUDE.md llvm-tools
  pointer).
- `grep -n "MINIMUM ENTRY SURFACE FITS\|COVERAGE CAVEAT"
  scripts/enclave_budget_gate.py` -> exit 0, hits :667, :670 (FA10 fixes
  (a)+(c) landed witnessed by grep).
- Greps for G2 thresholds / Agent-11 / rollup rows over docs/jira/v26.11.2/
  -> exit 0 with the hits named in DEFECT-REGISTER.md D-G08-1; no
  post-dating G2 receipt found (rollup rows stay OPEN).
- Read: THE-SMOKING-GUN-REPORT.md (full), forensic-lane-06.md:60-120,
  forensic-lane-10.md:55-120, HANDOFF.md SS4/SS5, repair-06 pointers,
  lane-fa10r-verdict-rename.md (full), 01-release-gates.md:45-95,
  chains-ingress.rs:20-30.
- Write of DEFECT-REGISTER.md: initial Write tail-corrupted
  ("riscv32divergences" token); REPAIRED same session by full rewrite
  assembled via python heredoc; on-disk bytes verified (127 lines, corrupt
  token count 0, tail lines intact).

## Fences (must-not-regress surfaces this diff could break)

Doc-only diff (two new files). No gates run against source; no courts
touched; shared tree untouched outside the two dispatched files. The
register's only regression surface is claim drift: every number in it is
cited file:line and the one CLOSED row carries its re-open trigger.

## Divergences

- Dispatch named "Agent-11's 2026-11-02 re-derivation" as a register-of-
  truth input; grep for "Agent-11"/"agent-11" across docs/jira/v26.11.2/
  returns NOTHING (exit 0, zero hits). The on-disk referent used instead:
  THE-SMOKING-GUN-REPORT.md + the FA10R remediation receipt
  (countermeasures/lane-fa10r-verdict-rename.md). Recorded per
  doc-yields-to-reality law.
- The FA10 re-derivation reused FA10's surviving /tmp/fa10-max-consumer
  crate (found intact); it was NOT rebuilt from scratch this session --
  its Cargo.toml pins path deps into the shared tree read-only, which is
  the exact CLAUDE.md /tmp caller-crate method, so the derivation stands.
- Build lease /tmp/g08-target deleted at lane end (below, this session).
