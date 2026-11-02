# repair-16 — dred swar wire probe (lane DREDP)

Date: 2026-11-02. Lane: DREDP (BR9 primitive consumption; the last
dead-library batch item). Branch: main, one canonical checkout.

## Standing: PARTIAL (KEEP on the perf fence; divergence recorded)

The dispatched swap was implemented, proven bit-identical, and then
REFUSED on the dispatch's own perf fence: the word-parallel SWAR form
measured -11% facts/s against the per-lane loop with the same bench
method. The per-lane rescue loop is KEPT; the primitive
`swar_nonzero_u64` was deleted per ERRC (delete outranks generate)
rather than exception-listed. The dead-library exception for the swar
family therefore REMAINS (gate now PASS with 17 justified exceptions;
swar_nonzero_u64 no longer exists to be flagged).

## 1. Primitive diff (built, proven, then deleted)

- `cgap-branchless/src/swar.rs`: `swar_nonzero_u64(x: u64) -> u64`
  landed with the family mask convention (0xFF per nonzero byte),
  differential court vs a naive per-byte oracle: 50k splitmix64 pairs
  (seed `0xA20C_0FFE_E5A7_E000`, multiplier
  `0xBF58_476D_1CE4_E5B9`), edge sweep, known-answer edges; plus the
  XOR identity `swar_nonzero_u64(x^y) == !eq_mask(x,y)` (the reason
  swar_eq_u64 decomposes through it). All green before deletion.
- Deletion: clean (cargo test -p cgap-branchless: 32 lib + 3 doc tests,
  exit 0, post-deletion; dead-library gate PASS, exit 0).

## 2. Swap sites

- keep_mask rescue (dred.rs:71-77 comment; loop at dred.rs:78-82):
  swapped to word-parallel SWAR in two measured variants:
  - v1: 8-lane byte-pack + swar_nonzero_u64 + magic-multiplier gather.
    The magic multiplier (0x0002_0408_1020_4081) POLLUTES across lanes
    (verified numerically; bit positions {0,7,14,21,28,35,42,49} cannot
    gather bit0-per-byte cleanly). Replaced by a shift-mask OR-fold
    gather, verified numerically vs the naive oracle (200k random u64).
  - v2: 2-lanes-per-u64 pair fold (q=a|b<<32; q|q>>16, q|q>>8 folds
    byte0<-a, byte4<-b; swar_nonzero_u64 + 2-term gather), verified
    numerically (300k random u32 pairs). Landed, bit-identical output,
    all dred courts + FA5 oracle green.
- BFS enqueue (dred.rs:366-381 seed loop; dred.rs:386-406 BFS inner
  enqueue): KEEP. Control-flow inherent: the enqueue count is
  data-dependent (per-set-bit iteration), each push at dred.rs:377 and
  dred.rs:403 writes the worklist slot that IS the enqueue, and both
  loops carry a mid-loop typed `CascadeError::WorklistCapacity` return
  (dred.rs:372-376, dred.rs:397-401). pred_inc-style counting would
  require fixed-trip predicated compaction that cannot preserve the
  mid-loop capacity refusal. Recorded KEEP, honestly.

## 3. Bench before/after (stated method)

Method: `cargo bench -p bench --bench dred_churn` (wall-clock Instant,
min-of-N rounds), CARGO_TARGET_DIR=/tmp/dredp-target, same machine,
same corpus (262144 facts, delete fractions 10/50/90). Baseline arm
run in the /tmp/dredp-sb sandbox with the pre-swap keep_mask
reconstructed byte-for-byte; post-swap arm in the shared tree; both
exit 0, BENCH-OK, PURGE_COUNTS identical.

- PRE-SWAP (per-lane): 944M / 944M / 944M facts/s (10/50/90 legs)
- SWAR v1 (8-lane pack): not benched (magic-multiplier bug found by
  unit math before benching; the fixed gather form benched at 784M on
  the 50%/90% legs)
- SWAR v2 (pair fold): 838.9M / 839.1M / 839.0M facts/s — -11% vs
  baseline. FENCE "must not regress ~943M": VIOLATED by the swap, so
  the swap is refused.
- FINAL (per-lane, kept): 943M / 814M / 879M (shared tree,
  BENCH-OK, exit 0). The 50%/90% legs are machine-noisy between runs
  (944M vs 879M for identical code); only the -11% delta (same-run
  comparison v2 vs baseline in matched environments) is load-bearing.

Run-to-run variance on the identical per-lane code spans 814-944M, so
the load-bearing comparison is the matched-run pair: baseline 944M vs
v2 839M (-11%) measured back-to-back in the same session, same target
dir pattern, same corpus, exit 0 both.

## 4. Courts run (all this session, exit 0)

- cgap-branchless: 32 lib + 3 doc tests PASS (with the primitive), and
  32+3 PASS again post-deletion.
- cgap-storage: lib 6, dred_cascade 5, dred_swar_wire 3 (NEW file:
  oracle-equality, load-bearing-rescue, mutant-divergence courts),
  court_mutations 6, court_sweep 2, court_faults 5 — all PASS.
- Anti-vacuity flip (dred_swar_wire, /tmp/dredp-sb sandbox, shared
  tree untouched): rescue-dropped mutation of dred.rs -> 0/3 (all
  three courts FAIL); restore sha256-exact verified -> 3/3 PASS
  again. Post-restore mtime bumped in sandbox; shared tree was never
  mutated.
- New courts are NOT vacuous: they flip 3/3 on the named mutation
  (drop the alt_support rescue OR).

## 5. Exception-list update note

scripts/dead_library_exceptions.json: UNCHANGED (the swar family
exceptions stand as-is). swar_nonzero_u64 was never added to the
exception list because it was deleted (ERRC asymmetry: delete outranks
generate). Dead-library gate: PASS (exit 0,
scripts/verify_dead_library.py, this session).

## 6. Divergences from the dispatch

1. The keep_mask swap did NOT land: implemented + verified
   bit-identical, but refused on the measured -11% perf fence. Per-lane
   loop KEPT; a probe note stands in dred.rs:71-77.
2. The swar family dead-library exception is NOT removed (the dispatch
   expected removal); it remains justified — the primitive could not be
   wired without regressing the churn bench. swar_nonzero_u64 itself
   was deleted rather than parked unconsumed.
3. cgap-storage/Cargo.toml: the temporary cgap-branchless dependency
   was added and then REMOVED after the swap was refused; final state
   is byte-identical to pre-lane state.
4. Pre-existing (not this lane): crates/cgap-storage/tests/
   closure_alloc_court.rs:122 fails to compile (semi_naive_closure
   tuple arity) — closure lane's file; present before DREDP work
   started; untouched.
5. Pre-existing: the dispatch's BFS line cite (dred.rs:383-397) has
   drifted to dred.rs:366-406 on disk; cited current lines.

## 7. Commands (exits witnessed this session)

- cargo test -p cgap-branchless -> exit 0 (32+3+0)
- cargo test -p cgap-storage --lib --test dred_swar_wire
  --test dred_cascade --test court_mutations --test court_sweep
  --test court_faults -> exit 0
- cargo bench -p bench --bench dred_churn -> exit 0, BENCH-OK
- python3 scripts/verify_dead_library.py -> exit 0, PASS
- /tmp sandbox flip: baseline 3/3 PASS -> mutated 0/3 FAIL -> restore
  sha256-exact -> 3/3 PASS
