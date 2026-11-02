# A12 Finish Receipt

- **Lane**: A12 (`crates/cgap-bn3`: `src/unify.rs`, `src/project.rs`),
  finisher pass over the wave-9 tree, v26.11.1, 2026-11-01.
- **Subject**: repo `/Users/sac/cgap`, branch `main`, ONE canonical checkout,
  no worktrees/clones; no git write commands run (read-only `git log`/`git
  status`/`git show` only).
- **Lease**: `crates/cgap-bn3/src/unify.rs`,
  `crates/cgap-bn3/src/project.rs`, this receipt. No other file written;
  `git status` at receipt time: exactly these three paths modified/untracked
  by this lane.
- **Mission**: branch-free constant-time vector unification over u64 lanes
  (first-order variable binding + pattern unify, fixed-capacity arena, no
  dynamic dispatch), doe-4 courts, sandbox flip, receipt.

## Baseline (pre-edit, this session)

- `CARGO_TARGET_DIR=/tmp/cgap-lane-A12-target cargo test -p cgap-bn3` —
  first attempt hit a mid-write `arena.rs` (A11 lease; mismatched-delimiter
  compile abort, file mtime 12 s before the run). Per the compile-freeze
  SLA I polled instead of touching A11's file; A11's next write resolved it
  in ~2 min. No A12 unblock applied, nothing to disclose.
- Re-run after stabilization: **exit 0, 102 passed / 0 failed** (the wave-8
  receipt's 87 grew to 102 through lanes A11/A13/wave-9 landings).
- `arena.rs` mtime continued advancing during this pass (concurrent A11
  work); the gate count moved 102 -> 104 from concurrently-landed lane
  tests, then 104 -> 111 with my +4 courts and +3 more concurrent lane
  tests. Counts are as-of-run; attribution: A12 authored exactly the four
  courts named below.

## Work landed (all inside the lease)

1. **Constant-time comparator arm** (`unify.rs`): new `MASK_ALL` +
   `lane_eq_mask(a, b)` — branch-free u64-lane equality (XOR, neg, OR,
   shift, sub; identical instruction stream for every operand pair; no
   data-dependent branch, early exit, or lookup inside the comparison).
   Both comparator sites now AND masks into an accumulator and branch once
   on the final mask: `atom_local_match_into` (const equality + within-atom
   repetition) and the nested-loop join probe (bound-slot VALUE compare).
   Binding pushes keep first-occurrence slot order; accept/reject semantics
   are unchanged (pure comparisons, order-independent conjunction) —
   witnessed by the unchanged courts (differential, M12-6 determinism,
   zero-alloc, 1M-row stress) staying green post-rewrite.
2. **Binding-consistency court (doe-4 INV-1)**
   `semantics_court_binding_consistency_value_probe_not_alias`: one value
   per variable per match (column alignment), binding = store word at the
   first-binding atom's row, and a planted register-number trap row
   (raw 65 = R65) that a value-blind comparator would falsely join.
3. **Occurs/self-join court (doe-4 INV-2)**
   `semantics_court_occurs_self_join_is_within_row_and_matches_oracle`:
   within-atom repetition is WITHIN-ROW value equality (cross-row value
   swap rows 8/9, 9/8 do not self-join), derivations equal the naive
   oracle's (the doe-4-registered occurs-check reference), and the unbound
   head variable stays a typed compile refusal.
4. **Same-input/same-bindings determinism court**
   `determinism_court_same_input_yields_identical_bindings`: identical
   `MatchSet` across fresh-scratch repeats, warm scratch dirtied by another
   rule, and chunk size 1 — the binding-map property M12-6 pins at batch
   level.
5. **emit_into determinism court** (`project.rs`)
   `determinism_court_emit_into_reused_dirty_buffers_byte_identical`:
   byte-identical canonical bytes, digest, popcount, and rows across fresh
   buffers, reused dirty buffers (junk row + 128 junk bytes cleared), and
   chunked match sets.

## Commands and exits (this session)

| Command | Exit | Result |
|---|---|---|
| gate (pre-edit, after A11 stabilized) | 0 | 102/102 |
| same gate, post comparator rewrite, pre new courts | 0 | 104/104, 0 fail |
| same gate, post 4 new courts | 0 | 111/111 |
| sandbox baseline `cargo test -p cgap-bn3 --lib` (in `/tmp/a12-finish-sb`) | 0 | 111/111 |
| sandbox SHIM-DOE4 AFTER run | **101** | 107 pass / **4 FAILED** |
| same gate, post restore, final in-repo run | 0 | 111/111 |
| `cargo test -p cgap-bn3 -- --test-threads=1 semantics_court` | 0 | 2/2 by name |
| hygiene greps (`dyn `, sleep) in the two lease files | 1 | zero hits |

The gate command throughout is
`CARGO_TARGET_DIR=/tmp/cgap-lane-A12-target cargo test -p cgap-bn3`
run from `/Users/sac/cgap`.

## Courts run

| Court | Verdict |
|---|---|
| binding-consistency court (INV-1, new) | pass; **FLIP** under SHIM-DOE4 |
| occurs/self-join court (INV-2, new) | pass |
| same-input/same-bindings determinism court (new) | pass |
| `emit_into` reused-dirty-buffers determinism court (new) | pass |
| closure court (differential vs naive oracle) | pass; **FLIP** under SHIM-DOE4 (rc 101) |
| cross-row join kernel court | pass; FLIP under SHIM-DOE4 |
| zero-alloc scan+project court | pass; co-flip under SHIM-DOE4 |
| M12-6 determinism, M12-7 popcount, 1M stress, digest/WAL/frame | pass (111/111) |

Full function names for every row are verbatim in the SHIM section above
and in the sources.

## SHIM-DOE4 flip receipt (doe-4 Section 8 protocol, /tmp sandbox only)

- Sandbox: `/tmp/a12-finish-sb` — standalone workspace (cgap-bn3 +
  cgap-blake3 + `[workspace.package]` v26.10.29 edition 2021 + dev
  `panic = "abort"`), sources sha256-identical to repo pre-mutation:
  `unify.rs 60d10a7571ab9151ac2fae55dd7ed4f63a6ca07e824a9f4e5adaf25f52a88754`,
  `project.rs f29806cbc09ee1eb469b9c93f9a38e0216d7197cc6ffc152857116aa4bc2772a`.
- BASELINE: sha256 above; `cargo test -p cgap-bn3 --lib` exit 0, 111/111.
- SHIM: python exact-string replace, anchor count asserted == 1; the join
  probe's `ok &= lane_eq_mask(cols.col(si)[j], sc.cur_vars[vidx][m])` ->
  `ok &= lane_eq_mask((VAR_BAND_FIRST + vidx) as u64, cols.col(si)[j])`,
  marked `SHIM-COURT SHIM-DOE4` — the binding-alias comparator class (the
  binding's REGISTER NUMBER aliases the candidate lane value).
- AFTER: **exit 101**, 107 passed / 4 failed. Flip set:
  `differential_court_matches_naive_oracle_over_100_witnessed_corpora`
  (the closure court — dispatch requirement met), the new binding-
  consistency court (its named mutation), `kernel_court_cross_row_join`,
  `zero_alloc_court_scan_project_hot_path_allocates_zero_after_warmup`.
  The trap row (raw word 65) falsely joined under the alias comparator —
  the false-positive direction doe-4 designed it for.
- RESTORE: `unify.rs` back from `.pristine`, sha256 re-read
  `60d10a75…88754` == BASELINE (byte-exact, fail-closed); `SHIM-COURT`
  grep count 0 post-restore. Shared tree never touched by the flip.
- **Zero vacuous courts**: every named mutation flipped its named court.

## Standing

- `unify.rs` + `project.rs`: **ALIVE** — final in-repo gate exit 0, 111/111
  this session (command + exit above).
- Constant-time comparator arm: **ALIVE by construction** (fixed mask-
  arithmetic instruction stream per comparison, no data-dependent control
  flow inside comparisons). The branch-MISPREDICT-level claim (BM0, 0
  mispredicts, ratio fence) remains **BLOCKED / TARGET** per D-A12-2:
  macOS exposes no `perf` counter here and `benches/` is outside this
  lane's lease. No perf number in this receipt is measured.
- M12-4 (pad corruption): N/A — the authoritative A11 layout has no pad
  field (unchanged, cited from `unify.rs` divergence note 2).
- Fixed-capacity arena: `RegisterFile` 256 x u64 = 2 KiB, const-pinned
  (`unify.rs` Wave-1 const asserts); no dynamic dispatch in either lease
  file (`grep "dyn "` rc=1, zero hits).
- Lease cleanup: `/tmp/cgap-lane-A12-target` and `/tmp/a12-finish-sb` both
  deleted before return (`rm -r` rc=0; `ls` confirms absent). No unreaped
  lane leases.

## Divergences (recorded, not silent)

1. **D-F1 (concurrent-lane count drift)**: the gate count moved 102 -> 104
   -> 111 during this pass from A11/A13 landings racing the same crate. My
   authored delta is exactly +4 courts; per-court witnesses above are
   by-name runs, not count arithmetic.
2. **D-F2 (arena.rs mid-write compile abort)**: observed once (mtime 12 s
   before first gate run), resolved by A11 within the ~10 min SLA. No A12
   unblock applied; no cross-lane disclosure owed.
3. **D-F3 (mission wording vs landed kernel)**: the mission's "branch-free
   constant-time vector unification" is realized as the constant-time
   comparator arm over u64 lanes inside the existing nested-loop kernel
   (spec ADR 3.3's honest scoping). The lane doc's Arrow-batch input and
   prefix-sum compaction remain Wave-3 seams (in-file divergences 1 and
   4); the BM0 mispredict gate stays BLOCKED (D-A12-2).
4. **D-F4 (doe-4 bench-side scope)**: `crates/bench` DOE-4 Divan membrane,
   Arm-B bitset twin, and the measured ns/pair table remain
   `PENDING-LAND`/`TARGET` owned by the C14 conversion lane; this receipt
   executes only the SHIM-DOE4 flip protocol in /tmp, which doe-4 Section
   8 pre-registers.
5. **D-F5 (doc status)**: lane doc `A12-simd-unifier.md` Status still reads
   PROPOSED/TARGET; that file is not in this lane's lease. DoD-8-style
   status truth lives here and in the in-file divergence notes.
