# Lane B05 — CSCB C6 retraction throughput court

Subject: `crates/csct/src/bin/cscb_c6_retraction.rs` (new),
`crates/csct/Cargo.toml` (minimal disclosed unblock, see Divergences).

## What was built

C6 court binary dogfooding the REAL DRed surface
(`cgap_storage::dred::{cascade_retract, cascade_oracle, build_csr,
splitmix64}` — `crates/cgap-storage/src/dred.rs`). No mocks.

- Corpus: 100,000 facts, seed `0xA20C_0FFE_E5A7_E000` stepped by
  `dred::splitmix64`; parent tree (99,999 edges) + every-8th chord edges
  = 112,499 edges; 9,956 deleted roots (Bernoulli 10%); 25%
  alt-supported facts; `derived_deps` bit iff the fact has a lineage
  parent.
- Timing: `std::time::Instant` around ONE full `cascade_retract`
  invocation (CSR build excluded from timing; BFS cascade + keep-mask +
  branchless compaction included), N=7 release runs, min/median/p99,
  run-to-run survivor-count determinism asserted.
- C6 gate: `min throughput >= 800,000,000 facts/s`, unit explicitly
  facts/s, counting basis stated on stdout (post-DREDLABEL law).
- Anti-phantom integrity: full-recompute oracle
  (`dred::cascade_oracle`, rounds-based least-fixpoint) set-equality vs
  the fast path, plus per-survivor keep-set membership (binary search).
- Anti-vacuity: `CSCB_MUTATE=skip-sweep` skips the set-equality sweep
  and injects 10 phantom survivors (deleted roots the cascade purged);
  the membership scan must detect them -> exit 1.

## Commands + exits (this session)

```
cd crates/csct && CARGO_TARGET_DIR=/tmp/b05-target cargo build --release --bin cscb_c6_retraction   # exit 0
/tmp/b05-target/release/cscb_c6_retraction                                                          # exit 1 (typed GATE FAIL)
CSCB_MUTATE=skip-sweep /tmp/b05-target/release/cscb_c6_retraction                                   # exit 1 (court flipped)
```

## Measured throughput (honest, units stated)

Counting basis: 100,000 facts / elapsed of one `cascade_retract` call.

- min 248,062,015.5 facts/s (403.125 µs), median 216,060,652.5 facts/s
  (462.833 µs), p99 142,104,395.6 facts/s (703.708 µs) — witness run
  2026-11-02, release, N=7.
- Run-to-run variance is material (an earlier release witness run on the
  same binary measured min 87.8M facts/s under load; p99 9.7 ms). The
  C6 threshold of 800M facts/s is 3.2x above the best observed min.

**C6 standing: typed FAIL** — measured 248M facts/s vs the 800M facts/s
gate. Per the reality fence, the claim was not tuned to the number; the
honest number is the result. (Note: `dred.rs`'s own prior measurement
cite, `crates/cgap-storage/src/dred.rs:73-76`, reports 839M-944M
facts/s for the keep-mask path ALONE on a different corpus/method;
the full cascade incl. BFS measures far lower here. The keep-mask-only
number is not the C6 number and must not be conflated — same
wrong-unit risk as DREDLABEL.)

## Integrity verdict (separate from the perf gate)

Integrity is ALIVE: oracle set-equality PASS (63,906 survivors, 36,094
purged), 0 phantoms, 0 orphan derivations. The mutant run detects
exactly the 10 injected phantoms -> exit 1. The retraction is CORRECT;
it is not fast enough for C6.

## Anti-vacuity

Mutant flip witnessed this session: normal -> `C6 GATE FAIL ...` +
integrity PASS, exit 1 (gate fail); mutant -> `C6 integrity FAIL: 10
phantom survivors detected`, exit 1. A court that cannot flip is FAIL;
this one flips on the integrity axis (the perf gate reads FAIL either
way in the current state, so the integrity flip is the load-bearing
anti-vacuity witness).

## Divergences (disclosed)

1. **Cargo.toml unblock**: dispatch named two files; reaching the real
   `cgap_storage::dred` API required one dep line in
   `crates/csct/Cargo.toml` (`cgap-storage = { path = "../cgap-storage" }`).
   Disclosed per compile-freeze SLA / disclose-in-both-directions law;
   coordinator should ratify or supersede.
2. Mutant-mode oracle is built even under skip-sweep so the membership
   scan flags exactly the 10 injected phantoms (first draft flagged all
   63,916 survivors via an empty oracle — detection was real but coarse;
   tightened before landing).

## Must-not-regress fences

- `cd crates/cgap-storage && CARGO_TARGET_DIR=/tmp/b05-target cargo test
  dred --release` — dred module courts 4/4 PASS (exit 0, this session).
- `cd crates/csct && CARGO_TARGET_DIR=/tmp/b05-target cargo test
  --release` — exit 0 (note: two other lanes' cscb bins
  `cscb_c1_c2_ingress`, `cscb_c4_alloc` were present and compiled with
  warnings, not mine; my bin built clean).
- Root workspace build (`cargo build --workspace`) — csct is standalone,
  outside the root workspace; no shared-surface edit other than the
  disclosed Cargo.toml line in the standalone csct crate.
- Root workspace build (`cargo build --workspace`) — csct is standalone,
  outside the root workspace; no shared-surface edit other than the
  disclosed Cargo.toml line in the standalone csct crate.


---

## ERRATA (2026-11-02, TC-08 / cull-08)

Lines 21 and 47 present the 800M facts/s figure as the operative C6
gate. Post-re-scope, 800M is ASPIRATIONAL_FPS, reported loudly, never
gating; the operative gate is DECIDED_GATE_FPS = 150,000,000 facts/s
min-of-7 full-cascade (100k facts, single `cascade_retract`). Gate
re-scope receipt of record: lane-mv-05-sla-reconcile.md:44; register
append: audit/release-blockers.md:233. This errata supersedes the two
lines; they are retained unedited.
