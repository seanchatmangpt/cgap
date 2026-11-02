# BR8 — Branchless reuse map: bn3 dedup/search

Lane: BR8, read-only survey + this doc. Primitives referenced:
`cgap-branchless::eytzinger::{eytzinger_search, to_eytzinger_layout}`
(FA6-flagged, UNCENSUSED reference drivers, ~5 branches in the
reference search loop only; the per-step primitives
`eytzinger_next`/`eytzinger_restore` are pure arithmetic).

## Headline

**Zero `binary_search` call sites in cgap-bn3.** The only sort sites are
`unify.rs:1340` (hot, per-probe) and a test-setup sort. The dominant
membership structure is the fixpoint `seen` BTreeSet. Verdicts below.

## Site map

| # | file:line | current | hot/cold | verdict | gain class | court mutant |
|---|---|---|---|---|---|---|
| 1 | `crates/cgap-bn3/src/unify.rs:1340-1341` | `key_set.sort_unstable()` + `dedup()` per probe | HOT (per rule x round x probe) | NO-SUBSTITUTION (for eytzinger) — the sorted `key_set` is consumed by `emit_join_masks_view` (`unify.rs:934`) as a **linear** `hit |= lane_eq_mask` scan over all keys (unify.rs:968-970), not a search; branch count is independent of layout, so Eytzinger conversion buys nothing. Gain lever here is `KEY_PREFILTER_MAX` gating + lane-NE instruction, not layout. | none (layout-insensitive) | flip `hit |=` to `hit &&=` (or drop dedup) in a /tmp copy; `seen.insert` count must change |
| 2 | `crates/cgap-bn3/tests/lever_courts.rs:397` | `keys.sort_unstable()` | COLD (test setup) | NO-SUBSTITUTION | none | n/a (test) |
| 3 | `crates/cgap-bn3/src/unify.rs:1518-1520` | `seen: BTreeSet<(u64,u64,u64)>` seeded from store, `seen.insert` per derived row (unify.rs:1562) | HOT (fixpoint termination machinery; one insert per derived fact) | STRUCTURAL-FUTURE | branch-miss-bound profile — UNMEASURABLE-here (no PMCs on this host) | mutate insert to unconditional push (kill dedup); derived-row count must inflate |
| 4 | `crates/cgap-bn3/src/noalloc.rs:323` | `seen` first-seen linear probe over fixed ≤3-entry pool | HOT but n≤3 | NO-SUBSTITUTION — 3-element linear compare beats any tree/layout at this width; already allocation-free | none | reorder `seen_idx` entries in a sandbox; lane masks must change |
| 5 | `unify.rs:1017` | `seen.iter().find` over `(usize,usize)` scratch (unify path) | HOT, small-n (body-width bounded) | NO-SUBSTITUTION — same small-n linear argument as #4 | none | n/a (same class as #4) |

## Falsifier for #3 (what would make a sorted/Eytzinger seen-set pay)

A measured branch-miss-bound profile on the fixpoint loop (insert-heavy,
log-n pointer chase per miss). Requires PMC (perf counters) this host
lacks — so standing is **UNMEASURABLE-here**, upgradeable only with a
profiled host. The redesign cost is real: Eytzinger layout over the seen
triple-set must be rebuilt each round as deltas append (layout
maintenance per round vs log-n lookup gain), and `seen` is
membership-only (unify.rs:1512 comment) — no ordered iteration is
consumed anywhere, so a BTreeSet could itself be demoted to a sorted
Vec + gallop before any Eytzinger step is justified.

## Anti-vacuity note

This is a survey lane; the named mutants (#1, #3, #4) are the court
mutants a future implementation lane must flip in /tmp sandbox copies
(sha256-exact restore, mtime bump). No code was modified; shared tree
untouched.
