# w5-tollgate

DMADV VERIFY tollgate ladder, /Users/sac/cgap @ main, 2026-11-02. All exits
first-hand this session; ladder run on the tree as it stood (concurrent lanes
landing; OST gate rerun once, stable — not mid-flight).

## Tollgate table

| # | deliverable | gate | command (cwd /Users/sac/cgap) | exit | standing |
|---|---|---|---|---|---|
| 1 | root workspace build | build | `cargo build --workspace` | 0 | ALIVE (1 manifest warn: cgap-integration unused dep cgap-arrow — pre-existing) |
| 2 | court harness | tests | `cargo test -p cgap-court` | 0 | ALIVE (14 passed, 0 failed, 30.99s) |
| 3 | bn3 unifier | tests | `cargo test -p cgap-bn3` | 0 | ALIVE (7 passed, 0 failed, 35.65s) |
| 4 | parquet kernels | tests | `cargo test -p cgap-parquet` | 0 | ALIVE (0 failed; doc-tests 0) |
| 5a | boundary strings | gate | `python3 scripts/boundary_gate.py --strings` | 0 | ALIVE (0 hits / 15 crates; A07 register sha 7778078dfe11; cross-gate GREEN scanned=62) |
| 5b | boundary deps | gate | `python3 scripts/boundary_gate.py --deps` | 0 | ALIVE (E=0 runtime externals / 15 crates) |
| 6a | zero-string gate | gate | `python3 scripts/zero_string_gate.py` | 0 | ALIVE (scanned=62 hits=0 emitter_hits=2 exempt; gate=1.4.1) |
| 6b | gate non-vacuity | court | `python3 scripts/zero_string_gate.py --self-check` | 0 | ALIVE (poison fires hits=2, clean 0 — PASS) |
| 7a | enclave budget | gate | `python3 scripts/enclave_budget_gate.py --check` (no --artifact) | 2 | BLOCKED (subject missing: no artifact registered; gate correctly refuses a subjectless check) |
| 7b | enclave budget non-vacuity | court | `python3 scripts/enclave_budget_gate.py --self-check` | 0 | ALIVE (arm C exit 0, arm P exit 1 — non-vacuous; mutant arm [SKIP] loud, no artifact) |
| 8 | G7 patent design-around | gate | `python3 scripts/verify_ost_design_around.py` | 1 | FAIL (stable on rerun; 1/4 DISTINCT; US10,817,467 / US11,216,456 / US11,409,698 PENDING/PARTIAL — see w5-g7-patent-gate.md) |
| 9 | wave receipts metagate | audit | grep over w5-*/m*/14/15 docs | 0 | PARTIAL — see below |

## Receipts landed this wave

w5: asm-census-repair, bn3-hasher-fix, budget-gate, encode-repareto,
g3-mat-closure, g4-dred-churn, g7-patent-gate, l3-court-flips,
l3-remeasure, linker-budgets, ring-rewitness (11). m1-footprint-rerun,
m2-nostd-census, m3-lease-ceiling-audit (3). Charters: 14-dflss-charter.md,
15-ctq-tree.md. NOT landed: 16-*.md, 17-*.md, paper/ (absent on disk).

## Metagate (each receipt >=1 command+exit)

All 14 w5-/m- receipts PASS (spot-verified command tables with exit columns).
14-dflss-charter.md and 15-ctq-tree.md: ZERO command+exit evidence — metagate
FAIL for the two charters (charter/CTQ genre, but no measured anchor yet).

## Operator-gated items

- Publishes: FROZEN (one attempt per crate post-window; not attempted).
- Cpk >= 1.33: UNVERIFIED — requires multi-run distributions (>= 25-30
  within-run samples across >= 2 independent runs per CTQ); only
  single-run benches exist (w5-l3-remeasure, m1-footprint-rerun). No N
  has been collected; distribution data needed before any Cpk claim.

## Standing summary

ALIVE 9, PARTIAL 1 (metagate, charter evidence gap), BLOCKED 1 (7a
subject-missing, exit 2 = loud), FAIL 1 (#8 G7-PATENT — typed, stable),
UNVERIFIED 1 (Cpk). Tollgate verdict: NOT CLEARED until #8 resolves or
is registered as a known divergence and 7a gains artifact subjects.
