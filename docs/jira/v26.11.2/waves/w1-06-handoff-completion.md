# W1-06 HANDOFF-COMPLETION receipt

Lane: W1-06 (lane-runner). Repo /Users/sac/cgap, branch main.
Sole writes: docs/jira/v26.11.2/HANDOFF.md (completed) and this
receipt. No git commands run. /Users/sac/stardf untouched.

## What landed

- HANDOFF.md SS2 gate-status table G1-G8 (on-disk names from
  01-release-gates.md:20,52,89,121,150,185,218,250).
- HANDOFF.md SS3 divergence register D-1..D-5.
- HANDOFF.md SS4 receipt index (26 pre-existing waves/ files +
  this receipt; re-listed via ls this session).

## Standing per claim (evidence re-read from disk this session)

- G1-PUBLISH BLOCKED: Cargo.toml:39 `version = "26.11.1"`;
  scripts/release_check.py:24 `REQUIRED_VERSION = "26.10.29"`.
- G2-PARETO UNVERIFIED: thresholds at 01-release-gates.md:67-71.
- G3/G4 BLOCKED: no mat/dred bench targets
  (01-release-gates.md:115-117, 145-146).
- G5 UNVERIFIED: riscv path unwired (01-release-gates.md:180-181).
- G6 PARTIAL: Cargo.toml:33 stale comment; 01-release-gates.md:200-202.
- G7/G8 BLOCKED: scripts absent (01-release-gates.md:242-244, 277-280).

## Divergences registered (HANDOFF SS3)

- D-1 PRD SS4 (00-ard-prd.md:47-60) gate names vs 01-release-gates
  on-disk G1-G8 — the latter govern runner dispatch.
- D-2 18/22 (RESTART-NOW) vs 19/22 (01-release-gates.md:27,32;
  HANDOFF SS1 F3) — 19/22 current.
- D-3 02-pareto-levers.md:92 `sovereign.rs:706-730` vs PRD
  00-ard-prd.md:43 `:706-724` — wider span is current (call site 722).
- D-4 release_check.py:24 stale 26.10.29 pin, must read 26.11.2
  (01-release-gates.md:25-26).
- D-5 workspace 26.11.1 (Cargo.toml:39) vs campaign 26.11.2 — bump is
  a G1 precondition, outside this lane's write set.

## Verification

- awk line-length sweep of HANDOFF.md: zero lines > 100 chars (exit 0).
- On-disk re-read after write: sections 2-4 present, SS1 preserved
  byte-intact above the SS2 header.

Standing: ALIVE (doc lane; artifacts written and re-read this
session; no executable court applies — mutation non-vacuity is
inherited from the cited gate specs' own FLIP blocks).
