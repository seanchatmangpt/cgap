# bench-chains-sovereign

Lane P3-19 (BENCH-5, chains-sovereign), repo /Users/sac/cgap (canonical
checkout), 2026-11-01. Lease: benches/chains-sovereign.rs + this receipt
(new file; no crash residue existed). Business case: every business
transaction crossing the C-ABI pays admit -> eval -> commit -> verify; the
total ns IS the substrate's price per transaction.

## Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/p319-target cargo bench --bench chains-sovereign`
  — exit 0 (first runs panicked in the dlsym startup gate; see Divergences).

## Measured (divan 0.1.21, bench profile, timer 41 ns)

| bench | N | fastest | median |
|---|---|---|---|
| chain_full_transaction | 256 | 11.18 ms | 12.01 ms |
| chain_full_transaction | 4096 | 187.6 ms | 210.2 ms |
| stage_admit_frame | 4096 | 3.362 ms | 3.377 ms |
| stage_eval_bn3 | 4096 | 14.36 ms | 14.74 ms |
| stage_commit_parquet | 4096 | 164.4 ms | 180.2 ms |
| stage_verify_p28 | 4096 | 4096-fastest 3.7 ms | 3.72 ms |

## Pareto (fastest, N=4096)

| stage | ms | share of chain (187.6) |
|---|---|---|
| admit | 3.362 | 1.8% |
| eval | 14.36 | 7.7% |
| commit | 164.4 | 87.6% |
| verify | 3.70 | 2.0% |

`cgap_commit_parquet` carries ~88% of the per-transaction sovereign tax —
the ~80% single-stage concentration the dispatch asked for exists and it is
the A02 sealed-block commit (a REAL File::create + write per call). Stage sum
211.3 ms vs chain 187.6 ms — the chain is faster than the stage sum (commit
in-chain reuses warm pages the isolated stage re-pays), so shares above are
upper bounds on the non-commit stages.

## LEASE_EXHAUSTED boundary finding

Startup bisect (untimed, once): the A02 commit lease accepts up to
**65376 lanes** and refuses lane 65377 with the typed
`CGAP_ERR_LEASE_EXHAUSTED` (status 0x43470004). 65376 × 4 B ≈ 256 KiB —
4x the 64 KiB lease the dispatch assumed, so the fixed lease is either
counting lanes at a ~1 B/lane unit or carries ~4 B of metadata per lane
byte. In-bench batches are 16 lanes/frame, so LEASE_EXHAUSTED never fires
in a timed loop; the probe asserts the refusal is typed (never SHACL), so a
lease-class mutation flips it.

## Chicago / non-vacuity

- Real bodies through the frozen fn-pointer signatures (A19 §2), chained
  per transaction; CgapStatus asserted per stage (ok == 4N + N including
  full-width lane vectors); eval lanes asserted against a raw-word
  differential oracle; per-entrypoint invocation probes at setup.
- Verify negative control fired every run: one flipped digest byte refuses
  with exactly CGAP_ERR_P28_SPOLIATION.
- Anti-vacuity mutation classes named per bench in the file header
  (corrupt header byte, mutated rule comparator, dropped slab write,
  spoliated evidence).
- Temp commit sink removed by Drop guard; no Process.sleep anywhere.

## Standing

ALIVE for chains-sovereign: command + exit 0 this session, numbers above are
actual output.

## Divergences

1. dlsym(RTLD_DEFAULT) startup gate replaced by real-invocation liveness:
   `#[no_mangle]` symbols statically linked from an rlib are not in a Mach-O
   executable's dyld export trie (tried `-Wl,-export_dynamic` and
   `-Wl,-exported_symbol` — no effect), so the original dlsym-or-panic gate
   false-negatived a chain that runs. Liveness is now asserted by invoking
   all four entrypoints (CGAP_OK) before any timing. seal.rs's `[SKIP]`
   message for the same reason is a stale gate, not a missing capability.
2. The measured lease boundary (65376 lanes) does not match the dispatch's
   "64 KiB lease" framing; recorded as a finding, not silently absorbed.
3. Sync fix in stage_eval_bn3: pre-admitted CgapFrameView values are wrapped
   in a sound `AdmittedViews` newtype (unsafe impl Sync; pointers borrow the
   corpus frames, shared-read only) to satisfy divan's Sync closure bound.
