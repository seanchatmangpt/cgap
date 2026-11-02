# W6-BN3NS: bn3 no_std strip + fast test gating

Lane BN3NS, repo /Users/sac/cgap, branch main, 2026-11-02.

## Diff summary (this lane only)

- `src/lib.rs`: added `#![cfg_attr(not(test), no_std)]` at crate root.
- `src/unify.rs:98-99` `use std::collections::{HashMap,HashSet}; use std::hash::{BuildHasherDefault, Hasher}`
  replaced with `extern crate alloc; use alloc::collections::{BTreeMap, BTreeSet}; use alloc::vec::Vec; use core::hash::Hasher;`.
  Production call sites renamed: `Interner.{const_index,var_index}` HashMap->BTreeMap, `binding` maps HashMap->BTreeMap,
  fixpoint `seen` HashSet<(u64,u64,u64), BuildHasherDefault<FixpointHasher>> -> BTreeSet<(u64,u64,u64)>,
  test-module `rule_ids` HashSet->BTreeSet. `std::mem::swap` x2 -> `core::mem::swap` (unify.rs:1419-1420 region;
  dispatch said :1315 — line drift, same swap region, W6-BFX driver untouched).
  `FixpointHasher` retained frozen (seed 0xBF58_476D_1CE4_E5B9) with `#[allow(dead_code)]` — no production consumer
  after the seen-set moved to BTreeSet; not deleted (seed-preservation discipline).
- `tests/bgp_hop_courts.rs`: `#[ignore = "expensive debug run - run with --release"]` on
  `bgp_two_hop_matches_oracle_at_scale` and `bgp_three_hop_matches_oracle_at_scale` (the 262,144-row legs) only;
  the N=65,536 legs still run.
- DISCLOSED MINIMAL UNBLOCK (outside listed ownership, required by the lib.rs no_std attr, disclosed per
  compile-freeze SLA): `src/project.rs` and `src/builtins.rs` relied on the std prelude's `Vec`; each gained
  `extern crate alloc; use alloc::vec::Vec;` after the doc header. No other change to those files.

## Import decision + justification

BTreeSet/BTreeMap chosen; hashbrown NOT added. The workspace carries no hashbrown dependency anywhere
(grep of root + all crate Cargo.toml = zero hits), so the dispatch condition for adding it was not met, and a
new external dep on a zero-dep kernel crate is the higher-cost path. Correctness: the seen-set is
membership-only (D5-verified per dispatch) so BTreeSet<(u64,u64,u64)> is semantically identical for the
termination machinery; the Interner/binding maps are tiny (u8/u64-keyed) so O(log n) BTreeMap is fine.
Bonus: no hasher at all in the hot loop — strictly less work than the FxHash-style hashing.

## Verification (all run this session, CARGO_TARGET_DIR=/tmp/bn3ns-target)

1. `cargo check -p cgap-bn3 --target riscv32imc-unknown-none-elf` -> exit 0, zero warnings (production lib
   no_std clean for the target; tests excluded via cfg_attr(not(test))).
2. `cargo test -p cgap-bn3` -> exit 0: lib 119 passed; bgp_hop_courts 2 passed + **2 ignored (loud, named
   reason)**; third suite 7 passed. Total wall ~95s.
3. `cargo test -p cgap-bn3 --release --no-run` -> exit 0; both ignored at_scale tests compiled into the
   bgp_hop_courts release executable (compile-check only; 10-min legs not run).
4. `cargo bench -p bench --bench chains-bn3 -- stage_unify_join` -> exit 0 (bench profile, 100 samples).

Pre-existing (not this lane): `warning: unused import: CompiledRule` in tests/lever_courts.rs.

## Bench regression vs w5-l3-remeasure baseline

| leg | w5 fastest | w6 fastest | w5 median | w6 median |
|---|---|---|---|---|
| stage_unify_join nested @65,536 | 8.435 ms | 7.801 ms | 9.967 ms | 7.898 ms |
| stage_unify_join nested @1,048,576 | 1.674 s | 1.724 s | 1.701 s | 1.741 s |
| stage_unify_join_radix @65,536 | 7.661 ms | 7.196 ms | 7.836 ms | 7.323 ms |
| stage_unify_join_radix @1,048,576 | 1.648 s | 1.600 s | 1.656 s | 1.645 s |

Verdict: no regression from BFX + strip. nested @1M is +2.9% fastest / +2.3% median (within observed build
noise; w5 itself recorded a 146 ms slowest-sample outlier on this leg); radix legs improved 4.6-6.1% fastest.
radix:nested @1M = 1.600/1.724 = 0.93x fastest (was 0.98x).

## Standing

ALIVE (all four gates above: command + exit, this session). Suite wall-time BEFORE the strip is UNVERIFIED
(strip landed in-session; no clean-before measurement taken) — the gating claim rests on the ignored count
(2 loudly ignored) and the pass counts, not a time delta.

## Divergences from dispatch

- mem::swap sites at unify.rs:1414-1415 (dispatch said 1315-1316) — same region, file shifted by the
  W6-BFX borrowed-prefix driver.
- std:: sweep found production-clean builtins/arena/sigil/noalloc; the only prelude-dependence stragglers
  (bare `Vec` in project.rs/builtins.rs) were fixed as the disclosed unblock above, not BLOCKED, because the
  breakage was created by this lane's own lib.rs edit.

Build-root lease: /tmp/bn3ns-target deleted at lane end.
