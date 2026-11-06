# Wave-closing receipt — cgap v26.11.6 (lane R-CGCLOSE)

Standing: ALIVE

- Subject: `/Users/sac/cgap` HEAD `027138ce` + in-tree untracked v26.11.6 artifacts.
- Lane: R-CGCLOSE (closure census + gate rerun + receipt; read-only git, no commits).
- Date: 2026-11-06.

## What cgap contributed this milestone

1. **Conservation-identity asserts in the adversarial Little's Law court**
   (commit `027138ce`, `crates/bench/benches/adversarial_littles_law_court.rs`).
   Regime 2 (burst fill) now asserts
   `pushed == popped + resident + refused` (new `len()`/`is_empty()` on
   `SovereignSpscRing`); regime 3 (destructive saturation) asserts
   `pushed_ok + refusals == SATURATION_PROBES`. These close the S12
   DIFFERENT-IDENTITY finding on the cgap side: the court previously measured
   only timing; it now carries arcus-E10-parity conservation identities over a
   different concrete ring identity, so the Little's Law law is pinned on both
   subsystems rather than one.
2. **Divan harness registration fix** (same commit,
   `crates/bench/Cargo.toml`): `adversarial_littles_law_court` was registered
   with `harness = false` plus an explicit comment. Before this, the bench was
   NOT in the bench-target inventory — auto-discovery would build it as a
   libtest harness, `divan::main` never runs, and none of the conservation
   asserts could ever execute. The bench had never actually run as a divan
   bench until this registration landed.
3. **Same-commit gate battery** (census context for `027138ce`):
   `include/cgap.h` (FFI surface prototype gap closure),
   `ontology-packs/fibo/tests/wave4_courts.rs`,
   `scripts/boundary_gate.py`.

## Gate evidence (run this session)

Command: `cargo bench -p bench --bench adversarial_littles_law_court`
Exit: **0**. Timer precision 41 ns, 100 samples/100 iters per regime.
Conservation asserts executed (an assert failure would abort the bench with a
nonzero exit; all regimes completed):

| regime | fastest | median | throughput (median) |
|---|---|---|---|
| regime1_in_register_steady_state | 3.082 µs | 3.249 µs | 307.7 Mitem/s |
| regime2_spsc_ring_burst_fill | 361.9 µs | 391.4 µs | 167.4 Mitem/s |
| regime3_destructive_saturation_refusal | 10.2 µs | 10.49 µs | 952.4 Mitem/s |

Regime 2 conservation witness: `pushed == popped + resident + refused` held
across 100 samples at 65,536 injected items/run. Regime 3 conservation
witness: `pushed_ok + refusals == 10_000` held across 100 samples.

## Untracked in-tree work (censused, NOT committed, NOT modified by this lane)

- `crates/bench/src/bin/cgap_e2e_consumer.rs` — cross-subsystem E2E SIMD
  consumer: mmaps `/tmp/sovereign_e2e_arrow.arena` (64 MiB SHM ring, 128B-
  aligned `ShmRingHeader`), consumes 64B frames in 32 KiB L2 tiles, applies
  in-register BN3 anomaly pruning (nominal 42.0 vs anomaly > 50.0), advances
  the SPSC tail atomically, writes exact `total_pruned`/`total_cleared` back
  to the SHM header.
- `reports/e2e_cross_subsystem_receipt.json` — receipt produced by
  `scripts/courts/court_e2e_arcus_cgap_smalt.sh` (arcus -> cgap -> smalt
  pipeline, 1,000,000 frames): `arcus_sent=1,000,000`,
  `arcus_admitted=355,824`, `arcus_refused=644,176`,
  `cgap_bn3_pruned=355,468`, `smalt_state_cleared=356`,
  blake3 root `fdc7d400…d2823`, `"standing": "ALIVE"`, 61 verified cleared
  slots. Both conservation invariants asserted by the court script
  (`sent == admitted + refused`; `admitted == pruned + cleared`) and passed.
- `scripts/courts/court_e2e_arcus_cgap_smalt.sh` — the court orchestrator
  itself: pass 1 nominal conservation verification + pass 2 anti-vacuity
  mutation (corrupts the arena header's cleared counter at offset 264, expects
  smalt to fail-closed; exits 1 if smalt accepts the corrupted arena).

Disposition: left untracked per lane contract; recorded here as census
evidence that the cross-subsystem E2E court ran ALIVE this milestone. Landing
these is a coordinator git transition, not a lane action.

## Standing

ALIVE — gate command + exit run this session against the exact subject
(`027138ce` + in-tree untracked artifacts); conservation witness numbers cited
from the real run above; untracked files read, not assumed.
