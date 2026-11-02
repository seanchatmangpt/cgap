# Lane MV-5 — SLA reconcile: C3/C6 gate re-scope (receipt)

Date: 2026-11-02. Repo /Users/sac/cgap (main). NO git commands run.
Files written: `crates/csct/src/bin/cscb_c3_closure.rs` (gate re-scope),
`crates/csct/src/bin/cscb_c6_retraction.rs` (gate re-scope),
`docs/jira/v26.11.2/audit/release-blockers.md` (R1/R10 append-only),
this receipt.

## Dispatch-divergence note

The dispatch cited C3 ~796 ms and C6 ~242-248M facts/s. My session
measurements: C3 min 36,918 ms (N=7, self-gen 1M) and C6 min
194,253,212.9 facts/s (N=7). The ~796 ms figure did not reproduce;
budgets are set on MY measurements per dispatch ("pick the numbers
YOUR measurements support"). The 25 ms (EXP-03) and 800M (EXP-05)
figures are recorded as aspirational, never gates.

## Commands + exits (this session, this host)

| command | exit | note |
|---|---|---|
| `cargo build --release --bin cscb_c3_closure --bin cscb_c6_retraction` (in crates/csct) | 0 | after re-scope |
| `cscb_c6_retraction` | 0 | PASS min 211,174,510.4 facts/s; aspirational 3.79x shortfall printed |
| `cscb_c3_closure --self-gen` (N=7) | 0 | PASS min 36,918.401 ms <= 60,000 ms; aspirational 1476x shortfall printed |
| `CSCB_MUTATE=skip-sweep cscb_c6_retraction` | 1 | 10 phantoms detected — court flips |
| `CSCB_MUTATE=zero-tax cscb_c6_retraction` | 1 | alias — court flips |
| `CSCB_MUTATE=radix-clamp cscb_c3_closure --self-gen` (N=7) | 1 | closure 4,837,284 != 1,500,000 — COURT-FLIPPED |
| `CSCB_MUTATE=zero-tax CSCB_RUNS=1 cscb_c3_closure --self-gen` | 1 | alias — COURT-FLIPPED |
| `cargo build --release` (crates/csct) | 0 | full crate |
| `cscb_runner --receipt-out /tmp/mv5-cscb-certified-run.md` | 0 | all 8 coordinates exit 0 |

## Decided budgets + provenance

- C3: DECIDED_BUDGET_MS = 60,000 ms min-of-7 on the self-gen 1M-edge
  corpus (seed 0xA20C_0FFE_E5A7_E000, SplitMix64 multiplier
  0xBF58_476D_1CE4_E5B9 as PRNG internals only). Session N=7:
  min 36,918.401 ms, median 40,324.179 ms, max 42,674.974 ms.
  Corroborating: B02 receipt (lanes/lane-b02-cscb-c3.md) min 42.394 s /
  median 48.478 s / p99 62.498 s — same order. ASPIRATIONAL_MS = 25.0
  (EXP-03: never a gate anywhere; repair-20's 24-25 ms was a different
  corpus), reported loudly, never gates.
- C6: DECIDED_GATE_FPS = 150,000,000 facts/s min-of-7 full-cascade
  (100k facts, single `cascade_retract`). Session N=7: min 194,253,212.9
  / median 159,130,004.4 / max 129,554,655.9; confirm run min
  211,174,510.4. ASPIRATIONAL_FPS = 800,000,000 (EXP-05: mask-only legs
  dred.rs:72-77), reported loudly (3.79x), never gates. Register: R1.

## Full-ladder per-coordinate exit table (cscb_runner, exit 0)

blake3 transcript root 9ab685137bbdbd47b42898bee24c1c34e0307a78a59bc91c46285fa989b1bd2e
(certified run: /tmp/mv5-cscb-certified-run.md; /tmp — not committed).

| coord | subject | exit | ms |
|---|---|---|---|
| C1 | cscb_c1_c2_ingress | 0 | 267 |
| C2 | cscb_c1_c2_ingress | 0 | 9 |
| C3 | cscb_c3_closure | 0 | 6301 |
| C4 | cscb_c4_alloc | 0 | 371 |
| C5 | scripts/verify_cscb_sram.py | 0 | 826 |
| C6 | cscb_c6_retraction | 0 | 245 |
| C7 | cscb_c7_receipt_sealing | 0 | 2254 |
| C8 | cargo test -p csct drowning_benchmark | 0 | 1110 |

Corpus-shape note: ladder C3 consumed the C1-written fixture at
/tmp/cscb_corpus.bin (6.3 s wall) — a different shape than the
standalone self-gen 1M run (36.9 s min). Both exit 0 under the decided
budget; divergence recorded, not reconciled.

## Fences (must-not-regress) checked

- cscb_runner aggregate semantics unchanged (only bins' internal gates
  edited; runner read-only).
- B02's anti-vacuity contract preserved: radix-clamp still flips at
  full scale; debug-profile still exits 3 loud skip (untouched arm).
- B05's C6 court preserved: oracle-equality sweep still runs in normal
  mode; phantom court still flips in both mutant tokens.
- Seeds untouched (corpus 0xA20C_0FFE_E5A7_E000; SM64 multiplier only
  as PRNG internals). No shared lib.rs/Cargo.toml edits. Root
  workspace not built; csct is a standalone workspace.

## Standing: ALIVE (all commands above run + exited this session)

- C3 gate: PASS (decided 60,000 ms), aspirational shortfall loud.
- C6 gate: PASS (decided 150M facts/s), aspirational shortfall loud.
- Courts: radix-clamp, skip-sweep, zero-tax all flip (exit 1).
- Ladder: 8/8 coordinates exit 0.
- Register: R1 DECIDED (close condition satisfied via the row's own
  gate-change clause); R10 PARTIAL-FLIP (C3+C6 flip; other children
  still ignore zero-tax — remains OPEN for full close).
