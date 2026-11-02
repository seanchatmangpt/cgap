# lane-gap-09-octopus

Lane GAP-09 — octopus merge flapping court root-cause + fix
(v26.11.2 starDF). Owns `crates/cgap-git/src/octopus_merge.rs` +
`tests/octopus_merge_court.rs`.

## Root cause (required paragraph)

The flap is an order/duplicate-dependent head misattribution in the
Phase-3 clash scan, `crates/cgap-git/src/octopus_merge.rs:309`
(pre-fix): the dedupe loop compared the current row against the
bucket's already-EMITTED output window
(`out_subject[emitted - scan_cursor[b] + j]`) but then attributed the
clash with `let (hs, _hr) = row_refs[base + j]` — indexing the INPUT
segment with an OUTPUT position `j`. Once duplicates were skipped
(dup rows advance `i` but not the emitted window), output position j
and input position j diverge, so `head_a` could be whichever head sat
at input offset j — the court's
`mutation_conflict_injection_flip` asserts `head_a: 0, head_b: 2`,
so a misattributed clash refused under the wrong names and the court
saw a typed ANDON DomainClash panic
(`octopus_merge.rs:356`, "heads N and M conflict") from an
unexpected attribution. Whether output-position j and input-position
j diverge depends entirely on the duplicate structure of the corpus
in the clashing bucket — deterministic per corpus, but
corpus-mutation-fragile, which is exactly the one-in-fleet
intermittency DX-8/GIT-16 each saw once and DF-5's 5/5 isolated
re-runs (fixed corpus, no dup divergence at that offset) could not
reproduce. Not HashMap order, not head ordering, not a parent/child
race: the re-exec witness (GIT-12 receipt:85-86) checks only
`st.code() == Some(101)`, already order-insensitive, and buckets are
walked in ascending index order, so the merge itself is
deterministic.

## Fix (diff summary)

`crates/cgap-git/src/octopus_merge.rs` Phase 3 only: the clash scan
now compares the current row against the bucket's PRIOR INPUT rows
(`row_refs[base .. base + i]`, re-reading `(s, p, o)` through the
RowRef) instead of the emitted-output window. `head_a` is now exactly
the head of the prior row holding `(s, p)` — deterministic,
dup-independent, and the emitted-output arithmetic
(`emitted - scan_cursor[b] + j`) is gone. Mutation flip intact: the
court's injected-conflict arm still refuses with a typed DomainClash
naming heads 0 and 2 (witnessed in this session's run output above
the FAILED-by-design child line). `scan_cursor` is still zeroed in
`OctopusScratch::reset` (octopus_merge.rs:202-207) but no longer read
in Phase 3; zero-alloc invariant unchanged (no new allocations).

## Commands + exits (this session)

- `CARGO_TARGET_DIR=/tmp/gap09-target cargo test -p cgap-git --test
  octopus_merge_court` → exit 0, parent 5 passed / 0 failed
  (intermediate `test result: FAILED` line is the court's by-design
  exit-101 child witness; ANDON panic printed with the CORRECT
  attribution "heads 0 and 2 conflict").
- 20 consecutive runs: `for i in $(seq 1 20); do cargo test -p
  cgap-git --test octopus_merge_court; done` → **20/20 exit 0**
  (fails=0/20), no sleeps, bounded loop.
- `cargo build -p cgap-git` → exit 0; no warnings reference
  octopus_merge.rs.
- `cargo test -p cgap-git` (whole crate) → exit 101 =
  **compile-abort in `tests/soc_full_chip_court.rs`** (E0063 missing
  field `hasher` in `ObjectStore` initializer, E0061 arity, at/near
  soc_full_chip_court.rs:167 — GIT-16's in-flight file, NOT this
  lane's diff; compile-abort aborts all targets before any test runs,
  so this is not a failure count). Not fixed here per file-ownership
  law; no SLA unblock applied (owner lane active).
- Fresh-clone-equivalent replay: the court binary recompiles the
  module via `#[path]` include; 20/20 loop re-ran the exact subject
  each time.

## Fences run (must-not-regress)

- The court's own 5 courts (oracle materialize, conflict detection,
  empty-head edge, mutation flip, zero-alloc) — all pass 20/20, zero
  alloc delta intact (court 5).
- GIT-12's sandbox SHIM mutation (neutered o-comparison) was NOT
  re-run this session; the court's built-in injected-conflict arm is
  the flip witness and it flipped (typed DomainClash, exit-101 child
  witnessed). Stated, not hidden.
