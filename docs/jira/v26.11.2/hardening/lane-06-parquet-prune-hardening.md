# Lane H6: parquet prune fail-open hardening verification

Date: 2026-11-02 · Standing: **ALIVE (lane subject) / OBSERVED (shared-tree state)**
Scope: READ-ONLY verification lane. Only file written: this receipt.
Build lease: `CARGO_TARGET_DIR=/tmp/h6-target` (deleted at lane end).

## 1. PRUNEFIX landed state — VERIFIED

Guard on disk, `crates/cgap-parquet/src/scan.rs` `read_statistics`
(scan.rs:654-673, decision arm at **scan.rs:665-669**):

```rust
if lo > hi {
    None
} else {
    Some(Bounds { min: lo, max: hi })
}
```

Present-but-inverted statistics (`min > max` bit-pattern order) are rejected
as UNTRUSTED and fail open to `None` (unpruned scan), never a wrong prune.
`scan.rs` sha256 this session:
`fb2b812116b06d9c4a9934224876bf97e0669e1354499ab1fb26c48e49b91803`
(matches repair-07's recorded pre/post hash — no drift since PRUNEFIX).

Court run this session:

```
CARGO_TARGET_DIR=/tmp/h6-target cargo test -p cgap-parquet --features std \
  --test stats_skip_court
# test result: ok. 4 passed; 0 failed  ... EXIT=0
```

4/4: `prunes_out_of_bounds_and_scans_in_bounds`,
`stripped_bounds_mutation_flips_prune`,
`inverted_bounds_fail_open_full_scan`,
`ordered_disjoint_bounds_prune_current_behavior_pinned`.

**Mutant flip (cited from repair-07, not re-run this session):** sandbox
copy at /tmp/prunefix-sandbox with the `lo > hi` arm removed =>
`inverted_bounds_fail_open_full_scan` FAILED (`left: [Some(Bounds { min:
5000, max: 10 })], right: [None]`) — trusting mutant prunes to Ok(empty).
Court flips => not vacuous. Shared-tree scan.rs sha256-identical
pre/post sandbox. Standing: cited-hearsay from
`docs/jira/v26.11.2/repairs/repair-07-prune-bounds-validation.md`
(re-run marked UNVERIFIED here, court result itself ALIVE above).

## 2. No panics on corrupt statistics — court-backed, PARTIAL

- `inverted_bounds_fail_open_full_scan` (ALIVE, exit 0 this session):
  inverted bounds parse to `None`, results byte-equal to the stripped
  control, `pages_parsed` equal — the fail-open path runs to completion
  with no panic and no wrong result.
- The corrupt-shapes courts `c1`-`c4` (mmap_writer checksum/attestation
  corruption) passed in the same session run (181 lib tests green before
  the concurrent-lane break; `c1_body_byte_flip...` etc. `ok`).
- Partial because the full `cargo test -p cgap-parquet` re-run at lane end
  could not complete: see section 5.

## 3. BR5 branchless prune-mask — REJECT recorded (with cite)

`docs/jira/v26.11.2/explore/branchless-reuse-lane-05.md` verdict stands:
the per-row-group branch is on a control-flow result inherent to
row-group granularity ("skip 65k rows vs scan them"); a mask form adds a
gather over non-contiguous `Vec<Option<Bounds>>` it does not remove a
branch worth removing; MB8 measured prune-only at 0.9 ns/RG (~230 ns/file
at 256 RGs). BR5's own court mutant (invert the `pred.0 > b.max ||
pred.1 < b.min` OR) remains the flip witness should any future lane
re-propose it. **REJECT recorded, not forced.** [ANALYTIC] — BR5's gains
are analysis, not measurements; no branchless rewrite on the stats/prune
path this wave.

## 4. OPEN defects (typed, honest scoping)

1. **Ordered-but-disjoint corrupt bounds still prune** — bounds with
   `min <= max` but disjoint from the data pass parse-time validation
   (footer-only view; no data to compare at parse time) and prune an
   in-bounds-for-the-data predicate to `Ok(empty)`. Pinned by
   `ordered_disjoint_bounds_prune_current_behavior_pinned` (ALIVE,
   exit 0). Close condition: a fix must either compare bounds against
   a data-derived digest at seal time (writer-side) or carry a
   data-overlap check at scan time; any fix must consciously flip the
   pinned court.
2. **Caller-blind degradation** — no scan-status channel reports that a
   scan degraded to unpruned full work because bounds were rejected;
   the caller sees only results. Close condition: a typed degradation
   report on the scan surface (e.g. `ScanOutcome::pruned(bool)` or a
   `stats_rejected: bool` on the row-group view), consumed by a court
   asserting both branches observable.

## 5. Session-introduced shared-tree state (disclosed, not this lane's files)

- First `cargo test -p cgap-parquet` run this session (pre-break): 181
  passed, 1 FAILED —
  `footer::courts::meta_fixed_path_is_byte_exact_vs_vec_oracle`
  panicked at footer.rs:1048 `RunOverrun { run_len: 12379570966709705216,
  budget: 9223372036854775807 }`.
- Immediately after, the crate stopped compiling: `RingSink` undeclared
  (footer.rs / mmap_writer.rs, 16 errors). A /tmp sandbox with
  `git show HEAD:footer.rs` fails identically => the break is the
  concurrent lane's in-flight uncommitted footer.rs/mmap_writer.rs pair,
  not scan.rs. `git diff --stat` at observation:
  footer.rs +341/-31, mmap_writer.rs in-flight.
- Classification: session-observed, another lane's in-flight files
  (compile-freeze SLA applies to the owning lane). The RunOverrun panic
  is UNCLASSIFIED pre-existing-vs-in-flight: observed only against
  in-flight bytes, never against a clean compile of any revision —
  it stays UNVERIFIED until the owning lane's footer lands.
- Final bounded re-polls (5 attempts mid-lane + 1 at lane end, stated
  deadline = until the crate compiles): all compile-aborted (rc 101,
  16-17 errors). **Compile-abort is not a failure count** — 0 parquet
  lib tests executed in those runs. stats_skip_court itself re-ran
  green at lane end (4 passed, exit 0) against unchanged scan.rs bytes.
- Lane subject (scan.rs prune path + stats_skip_court) was fully
  verified against the current on-disk scan.rs bytes BEFORE the break;
  those exits stand.
