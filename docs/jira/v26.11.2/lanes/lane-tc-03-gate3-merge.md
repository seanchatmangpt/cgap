# Lane TC-03: Gate 3 merge — cdata tombstone, schema gate subsumes

Date: 2026-11-02. Repo: /Users/sac/cgap (main). No git commands run.

## W1 Subject

`scripts/gate3_cdata.py` (replaced by tombstone) and
`scripts/gate3_schema.py` (docstring fold, config 0 = cdata assert set).
Per cull-01: VERIFIED-REDUNDANT at assert level — schema gate check 4
already re-proves bitwise buffers on the same cdylib
(`target/debug/libcgap_ffi.dylib`).

## W2 Diff

- `scripts/gate3_cdata.py`: content replaced by a 4-line tombstone
  (3 logical lines + header) pointing at gate3_schema.py config 0 and
  the re-open trigger: any C-Data export ABI change
  (include/cgap_c_data.h).
- `scripts/gate3_schema.py`: docstring only — records the TC-03 merge,
  config 0 (n=100_000, null_every=7) = cdata's exact assert set
  (rows==n, non-null export slots, type int32, null_count n//7, bitwise
  validity+values buffers via checks 1, 2, 4). Executable logic
  unchanged.

## W3 Commands / exits

| step | command | exit |
|---|---|---|
| pre-cull witness | `python3 scripts/gate3_cdata.py` | 0 |
| post-merge | `python3 scripts/gate3_schema.py` | 0 (15/15 PASS, 3 configs) |
| anti-vacuity flip | `python3 /tmp/tc03-sbox/gate3_schema.py` (mutated) | 1 (expected) |

## W4 Anti-vacuity court (SHIM-COURT)

Mutation: one buffer byte in the recomputed ground truth
(`(i if v else 0)` -> `(i if v else 1)`) in a /tmp sandbox copy
(`/tmp/tc03-sbox/`), shared tree untouched — sha256 of
`scripts/gate3_schema.py` identical before/before-flip/after:
`401d14e81819a64b7bc73d07130d2c4d4e797eed61e775203e612d2cdbd6cd80`.
Result: `FAIL [cfg0 ...] 4 buffers: ... values 400000B bitwise-NE`,
exit 1. The gate flips; not vacuous. Sandbox deleted after the run
(python shutil, no rm -rf). No restore/mtime issue: the mutated file
lived only in /tmp and never entered a cargo tree.

## W5 Tombstone content (verbatim)

    # TOMBSTONE (TC-03, cull-01 VERIFIED-REDUNDANT, 2026-11-02): this wire-parity
    # harness is subsumed by scripts/gate3_schema.py config 0 (checks 1, 2, 4
    # re-prove the bitwise buffers on the same cdylib).
    # RE-OPEN TRIGGER: any C-Data export ABI change (include/cgap_c_data.h).

## W6 Standing

ALIVE. Pre-cull cdata exit 0 witnessed this session; merged schema gate
exit 0 covering the cdata assert set as config 0; flip court exit 1.

## W7 Fences / divergences

- Fence: any caller of `python3 scripts/gate3_cdata.py` (docs, CI) now
  gets a no-op exit 0 tombstone, not the wire-parity run — the parity
  proof must be cited as gate3_schema.py config 0. grep for callers is
  coordinator scope; none swept in this lane.
- Divergence: dispatch said "3-line tombstone"; on-disk tombstone is 4
  comment lines (header line names TC-03 + date). Semantics identical.
- `git rm` is coordinator-owned; the tombstone is the deletion
  mechanism, file remains tracked.
