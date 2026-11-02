# W2-14: just doctor W2 gate-chain wiring

| Field | Value |
|---|---|
| Standing | PARTIAL |
| Falsifier | doctor chain exits 2 on gate rc=1 and goes YELLOW-[SKIP] on rc=2/3; flip witnessed in /tmp court |
| Commands | `just doctor` exit 1 (8G/2Y bin + d11/d12/d13 GREEN); /tmp flip court rc1->2, rc3->YELLOW |
| Scope | justfile doctor recipe; doctor.rs c07 SCRIPTS 7->10; receipt (this file) |

## What landed

1. `justfile` doctor recipe: chains doctor bin -> zero_datafusion_gate.py ->
   receipt_lint.py --root . -> registry_sweep.py. Mapping: gate 0=GREEN,
   1=RED exit 2 (chain aborts), 2/3=YELLOW loud `[SKIP] rc=N` (rc_total=1).
   Chain exit: 0 all-GREEN / 1 any-YELLOW / 2 any-RED (doctor.rs model).
   Receipt: waves/w2-14-doctor-wiring.md.
2. `crates/cli/src/bin/doctor.rs` c07: SCRIPTS `[&str; 7]` -> `[&str; 10]`,
   added `zero_datafusion_gate.py`, `receipt_lint.py`, `registry_sweep.py`;
   green detail now `10/10 gate scripts executable`. Closes the W1-05
   divergence (c07 never observing the new gates).

## Witnessed runs (this session)

- `just doctor` exit 1: c07 GREEN "10/10 gate scripts executable"
  (receipt: waves/w2-14-doctor-wiring.md);
  d11 zero_datafusion_gate GREEN, d12 receipt_lint GREEN (findings
  YELLOW-class, exit 0 per 4.2 default mode), d13 registry_sweep GREEN
  ("REGISTRY-SWEEP: PASS 0 missing"). Bin: 8 GREEN, 2 YELLOW (c06 5
  warnings; c08 receipt staleness), 0 RED.
  Receipt: waves/w2-14-doctor-wiring.md.
- Flip court (/tmp/w2-14-court, exact recipe logic, stub gates):
  stub rc=1 -> "RED (gate refuse)", chain exit 2; stub rc=3 ->
  "YELLOW [SKIP] rc=3", chain exit 1. Court lease deleted after.
  Receipt: waves/w2-14-doctor-wiring.md.
- Bounded poll (30s interval, 20-min deadline) waited out the three
  sibling gate scripts; all three landed before deadline (receipt:
  waves/w2-14-doctor-wiring.md).

## Standing

- PARTIAL, not ALIVE: the wiring ran green end to end this session, but
  the anti-vacuity flips were witnessed on the recipe logic in a /tmp
  sandbox with stub gates, not by mutating the landed gate scripts
  themselves (those flips are owned by lanes W2-01/W2-03/W2-05 per their
  own receipts). c06/c08 YELLOW are pre-existing, not session-introduced.
  Receipt: waves/w2-14-doctor-wiring.md.

## Divergences

- `chmod +x` applied to the three scripts/*.py (sibling-lane files, not
  in this lane's write set): doctor.rs c07 classifies exists-but-not-exec
  as RED, and all three landed non-executable. Metadata-only change,
  disclosed here and owed to the owning lanes' receipts
  (receipt: waves/w2-14-doctor-wiring.md).
- W1-05 spec section 3 sample aborts on gate rc=1 with exit 1; this
  recipe exits 2 (RED class) per spec section 5 and the dispatch.
- justfile `$$` passes through verbatim (shell expands `$$` as PID) —
  the recipe uses single `$`; first two run attempts hit this.
