# Repair 07: rg_prunes corrupt-bounds validation (FA8)

Lane: PRUNEFIX · Date: 2026-11-02 · Standing: **ALIVE**

## Defect (FA8, CONFIRMED-IN-PART)

Present-but-corrupt bounds (inverted `min > max`) trusted at parse time made
`rg_prunes` fire for every predicate range below `min`, silently pruning
matching rows into `Ok(empty)` — a wrong-result path contradicting the
fail-open docstring at `crates/cgap-parquet/src/scan.rs:317-319` (pre-fix
numbering).

## Fix

`crates/cgap-parquet/src/scan.rs`,
`read_statistics` (post-fix ~:620-666): after decoding both 8-byte PLAIN
bit patterns, reject inverted bounds as UNTRUSTED — `lo > hi => None`
(fail open, identical to the missing-stats path). Docstrings updated to
state the full trust policy (absent → no prune; present+valid → prune;
present+invalid → no prune) and the bit-pattern-Ord safety argument
(`PhysType::from_id` refuses FLOAT/DOUBLE, so no float trap).

## Courts (stats_skip_court.rs, 4 tests, all green)

| Court | Assertion | Status |
|---|---|---|
| prunes_out_of_bounds_and_scans_in_bounds (existing) | valid bounds prune; `pages_parsed == 0` on prune | PASS |
| stripped_bounds_mutation_flips_prune (existing) | no bounds → no prune, pages parsed | PASS |
| inverted_bounds_fail_open_full_scan (NEW) | inverted `min=5000,max=10` → parse yields `None`; results identical to stripped control; pages parsed (no pruning); `rg_prunes` never fires for any pred | PASS |
| ordered_disjoint_bounds_prune_current_behavior_pinned (NEW) | ordered-but-disjoint corrupt bounds (min≤max, disjoint from data) still prune — **pinned OPEN defect**, undetectable at parse time (no data to compare against); valid-bounds prune regression guard | PASS (pins defect) |

## Anti-vacuity flip witness (sandbox, shared tree untouched)

Sandbox: full `crates/` copy to `/tmp/prunefix-sandbox` with a workspace
manifest; mutation removes the `lo > hi` validation (restores the trusting
arm). Command run in sandbox:

```
cd /tmp/prunefix-sandbox && CARGO_TARGET_DIR=/tmp/prunefix-target \
  cargo test -p cgap-parquet --features std --test stats_skip_court
```

Result: **FAILED** — `inverted_bounds_fail_open_full_scan` panics with
`left: [Some(Bounds { min: 5000, max: 10 })], right: [None]`; the trusting
mutant would prune to `Ok(empty)`. 3 others passed. Court flips → not
vacuous. Shared-tree `scan.rs` sha256 before and after the sandbox work:
`fb2b812116b06d9c4a9934224876bf97e0669e1354499ab1fb26c48e49b91803`
(identical — mutation lived only in /tmp; sandbox + target leases deleted
via `shutil.rmtree`).

## Verification (this session)

```
cargo test -p cgap-parquet                  # default features: exit 0
cargo test -p cgap-parquet --features std   # exit 0 (stats_skip_court: 4 passed)
```

Pre-existing warnings in bitslice.rs / unpack_filter.rs unchanged (not this
lane's files).

## OPEN defects (typed, not built in this lane)

1. **Caller-blindness (FA8)**: no scan-status channel reports that a scan
   degraded to unpruned work because bounds were rejected; the caller sees
   only results. Fix needs a typed degradation report on the scan surface —
   out of scope here.
2. **Ordered-but-disjoint corrupt bounds still prune** (pinned by
   `ordered_disjoint_bounds_prune_current_behavior_pinned`): parse-time
   validation sees only the footer, not the data; a bounds struct with
   `min <= max` whose range misses the data cannot be rejected at parse
   time. Any future fix must consciously flip the pinned court.

## Files written

- `crates/cgap-parquet/src/scan.rs` (validation + docstrings)
- `crates/cgap-parquet/tests/stats_skip_court.rs` (2 new courts)
- this receipt
