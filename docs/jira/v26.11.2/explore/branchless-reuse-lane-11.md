# BR11 — Branchless reuse map: cgap-git tree/sort

Standing: READ-ONLY map lane. Anchors: GS4 (tree objects
NOT-IMPLEMENTED), FA6 (eytzinger unconsumed), GITFIX owns pack_write.rs
type-6/7 refusal. Verified this session: `grep -rln eytzinger crates/` →
only cgap-branchless (src/{lib,eytzinger}.rs, tests/branchless_census.rs);
no cgap-git dep on cgap-branchless. `ls crates/cgap-git/src/` → dag,
deflate, idx_read, inflate, ladder, lib, pack, pack_write — no tree.rs/sort.rs.

## 1. Tree canonical comparator — ABSENT (DESIGN-TARGET)

- file:line: ABSENT (no tree.rs/sort.rs in crates/cgap-git/src/);
  current: nothing — tree canonicalization unimplemented (GS4).
- class: DESIGN-TARGET — surface does not exist; a spec, not a substitution.
- spec: SWAR compare with virtual trailing-'/' byte — append virtual
  `b'/'` past each entry-name end, run a swar order-mask compare
  (cgap-branchless::swar has equality/zero/range; order-mask is missing).
- gain class: [ANALYTIC] until (a) the tree writer exists and (b) a
  git interop court compares our sort against real `git` tree ordering
  on the trailing-slash cases.
- court: git interop vs real git tree sort on corpus incl. `a`, `a.b`,
  `a/`, `a0`; mutation: swap two entries git orders differently —
  court must flip.

## 2. idx OID ordering — pack_write.rs:269 — NO-SUBSTITUTION

- file:line: crates/cgap-git/src/pack_write.rs:269
  (`entries.sort_by_key(|e| e.oid)`); current: sort_by_key on `[u8;20]`.
- verdict: NO-SUBSTITUTION. Branchless merge/Comparator gains are
  [ANALYTIC] (no hot caller); fanout at :286 already gives O(1)
  first-byte bucketing on the read side — no eytzinger needed. Radix
  (B5) is the only plausible-win class; belongs to a measured churn
  benchmark, not a blind swap.
- court: equal-OID shuffle mutation proving write_idx_v2 round-trip
  asserts real ordering.

## 3. Eytzinger for pack idx lookup — UNMEASURED-DEMAND

- file:line: crates/cgap-git/src/idx_read.rs:138 (`IdxV2::lookup`);
  current: NOT plain binary search — a 16-stage constant-time ladder
  (ladder::ladder_search / oid_lt_mask, idx_read.rs:158-176) over a
  fanout-bounded partition (LADDER_WINDOW, L6-1). FA6's proposed
  substitution is already realized by cgap-git's own ladder.rs;
  cgap-branchless::eytzinger (B4) is unconsumed.
- demand: no hot caller — lookup call sites are idx_read.rs tests
  (342..466) and lineage tests only; no production path yet.
- class: UNMEASURED-DEMAND — substituting eytzinger without a hot
  caller and a ratio fence is speculative churn. Revisit only if
  clone/fetch makes lookup hot; ladder vs eytzinger, min-of-N baseline.

## Map table

| # | surface | file:line | current | class | court |
|---|---------|-----------|---------|-------|-------|
| 1 | tree comparator | ABSENT | none | DESIGN-TARGET | git interop + swap mutation |
| 2 | idx sort | pack_write.rs:269 | sort_by_key | NO-SUBSTITUTION | equal-OID shuffle court |
| 3 | idx lookup | idx_read.rs:138 | 16-stage ladder | UNMEASURED-DEMAND | const-iter court :466 |

Net: zero substitutions admitted; one DESIGN-TARGET spec (absent surface);
eytzinger unconsumed per FA6.