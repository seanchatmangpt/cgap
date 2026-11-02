# BENCH-BN3 Receipt (F2-13)

- **Lane**: F2-13 BENCH-BN3 (`crates/bench/benches/bn3.rs`, Divan membrane
  target `bn3`), v26.11.1 bench campaign, 2026-11-01.
- **Subject**: repo `/Users/sac/cgap`, branch `main`, ONE canonical
  checkout, no worktrees; no git write commands run.
- **Lease**: `crates/bench/benches/bn3.rs` (NEW) + this receipt. No other
  file written; `crates/bench/Cargo.toml` (the `[[bench]] name = "bn3"`
  declaration) is F2-10's, consumed read-only via a bounded poll.
- **Charter**: `docs/jira/v26.11.1/bench-campaign-charter.md` bench map row
  "A11-A13"; unify metric per
  `docs/jira/v26.11.1/dflss/doe-4-unifier-data-structure.md` (one PAIR =
  one (parent match, candidate row) probe of the hot loop, unify.rs:850-864;
  pair count = sum over join atoms of cur_len x cand_len; ns/pair =
  unify_batch_into wall time / pair count).

## Command + exit

```bash
CARGO_TARGET_DIR=/tmp/lane-f2-13-target cargo bench -p bench --bench bn3
# exit 0 (full log: /tmp/lane-f2-13-bench.log, table verbatim below)
```

Pre-run gates, this session: bounded poll for the F2-10 `[[bench]]
name = "bn3"` declaration — POLL-OK after 1 iteration, exit 0; compile
`cargo bench -p bench --bench bn3 --no-run` — exit 0 in 12.69s.
Two full runs exited 0: run 1, then a re-run after a source-only line-wrap
(`eprintln!` string continuation) so the table below is MEASURED from the
exact on-disk bench bytes. Asserted values were identical across both runs
(same `expected_checksum=0xC80F0FACC957980A`, same oracle_matches
71/1031) — SplitMix corpora are deterministic. Lane target
`/tmp/lane-f2-13-target` deleted post-run via `python3 shutil.rmtree`,
verified `exists_after: False` (re-created for the re-run, deleted again).

## Full Divan table (verbatim)

```text
Timer precision: 41 ns
bn3                               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ builtin_dispatch_hot_arms_65k  [bn3-bench] dispatch evals=65536 ops=gt,sum,min,assert expected_checksum=0xC80F0FACC957980A
204.2 µs      │ 228.3 µs      │ 210.1 µs      │ 209.8 µs      │ 100     │ 100
├─ statement_scan_24b                           │               │               │               │         │
│  ├─ 65536                       [bn3-bench] scan N=65536 bytes=1572864
19.12 µs      │ 26.04 µs      │ 19.56 µs      │ 19.76 µs      │ 100     │ 100
│  │                              82.24 GB/s    │ 60.39 GB/s    │ 80.4 GB/s     │ 79.57 GB/s    │         │
│  ╰─ 1048576                     [bn3-bench] scan N=1048576 bytes=25165824
318.7 µs      │ 519.3 µs      │ 330.4 µs      │ 352.7 µs      │ 100     │ 100
│                                 78.94 GB/s    │ 48.45 GB/s    │ 76.14 GB/s    │ 71.33 GB/s    │         │
╰─ unify_join_2atom_pairs                       │               │               │               │         │
   ├─ 1024                        [bn3-bench] unify N=1024 parents=16 cands=256 pairs=4096 oracle_matches=71
26.62 µs      │ 40.29 µs      │ 26.95 µs      │ 27.1 µs       │ 100     │ 100
   ╰─ 65536                       [bn3-bench] unify N=65536 parents=256 cands=256 pairs=65536 oracle_matches=1031
6.184 ms      │ 6.574 ms      │ 6.255 ms      │ 6.255 ms      │ 100     │ 100
```

Interleave note: the `[bn3-bench] ...` lines are the benches' own setup
`eprintln!` witnesses (stderr) interleaved by the shell into the Divan
stdout table; the asserted `pairs`/`oracle_matches`/`expected_checksum`
values are visible in-band, per run.

## TARGET -> MEASURED (never rounded)

Method: Divan `fastest` column (min of 100 samples, charter "Min-of-N");
GB/s implied from bytes/time cross-checks Divan's own throughput column
(`bytes` values are the in-band `[bn3-bench]` witnesses).

| Bench | N | TARGET | MEASURED (fastest) | Verdict |
|---|---|---|---|---|
| `statement_scan_24b` | 65536 | > 1.0 GB/s | 82.24 GB/s (Divan fastest) | **MET** |
| `statement_scan_24b` | 1048576 | > 1.0 GB/s | 78.94 GB/s (Divan fastest) | **MET** |
| `unify_join_2atom_pairs` | 1024 | sub-µs/pair | 6.499023 ns/pair | **MET** |
| `unify_join_2atom_pairs` | 65536 | sub-µs/pair | 94.360352 ns/pair | **MET** |
| `builtin_dispatch_hot_arms_65k` | 65536 | (no stated target) | 3.115845 ns/eval | datapoint |

Implied-GB/s arithmetic (bytes / printed fastest time): 1,572,864 B /
19.12 µs = 82.262762 GB/s; 25,165,824 B / 318.7 µs = 78.963991 GB/s.
ns/pair arithmetic: 26,620 ns / 4,096 pairs = 6.499023; 6,184,000 ns /
65,536 pairs = 94.360352; 204,200 ns / 65,536 evals = 3.115845.

Mean-based secondary readings: scan 79.57 / 71.33 GB/s; unify 6.616211
ns/pair (N=1024) and 95.443726 ns/pair (N=65536); dispatch 3.201294
ns/eval.

## Chicago asserts (ran live in the timed path, 100x each, all passing)

- **Scan** — closure count `assert_eq!(count, N)`; mutation class killed:
  a skipped-row / dropped-tail-chunk scan.
- **Unify** — binding count `assert_eq!(out.len, oracle)` plus both
  var-column alignments, where `oracle` is an O(N) raw-word histogram
  over the corpus (NO unify code in the oracle path — differential check,
  not a tautology); mutation classes killed: the binding-alias comparator
  class (doe-4 SHIM-DOE4) and wrong binding order. The doe-4 pair count
  (`sum over join atoms of cur_len x cand_len` against
  `DEFAULT_CHUNK_ROWS` boundaries) is setup-computed and asserted via the
  in-band witness; `oracle_matches` = 71 (N=1024) / 1031 (N=65536).
- **Dispatch** — production `eval_count == 65536` closure count plus a
  checksum vs an integer oracle computed from the initial register file
  without any dispatch call; mutation classes killed: arm swap (min->max),
  inverted gt comparator, always-failing assert arm, skipped evaluation.
- Setup asserts: compiled join shape (2 atoms, `var_count == 2`),
  `oracle > 0` and `pairs > 0` (loud vacuity guard), slab
  fixed-capacity law, payload table size.

## Named anti-vacuity mutations (charter) — for the court-auditor pass

Per bench, the mutation that must move its number (sandbox flip is the
court-auditor's lease, not run by this lane):

1. Scan: drop the tail chunk / skip rows -> closure-count assert flips.
2. Unify: substitute the binding-alias comparator (SHIM-DOE4 class) ->
   oracle assert flips; also ns/pair moves (comparator is the hot path).
3. Dispatch: swap `OP_MATH_MIN` arms / invert `OP_MATH_GREATER_THAN` ->
   checksum assert flips and ns/eval moves.

## Divergences / notes

- First bounded poll used a wrong predicate (checked only the LAST
  `[[bench]]` section = `seal`, never true) and looped to a manual kill;
  the corrected poll (all `[[bench]]` sections) passed on iteration 1.
  The bench-observed state was never wrong — the first predicate was.
- Divan 0.1.21 default invocation emits no alloc column, so doe-4's
  secondary "alloc column = 0 B" reading stays owned by the A12 zero-alloc
  court, not this table. Warm-scratch shape (`unify_batch_into` + reused
  `EvalScratch`/`MatchSet`) is the production hot path regardless.
- `log:trace` is excluded from the dispatch hot-arm cycle: its trace-sink
  Vec grows unboundedly across 100 samples x 65,536 evals (instrument
  hazard, not a kernel limitation); `log:assert` payloads self-compare
  R[201] (always-passing shape of the hot arm, recorded here).
- Divan numbers are host-machine numbers (charter caveat); Gate 5 silicon
  fit remains an `llvm-size` measurement, unaffected by this receipt.

## Standing

**ALIVE** — `cargo bench -p bench --bench bn3` ran this session, exit 0,
100/100 samples per bench, all Chicago asserts passing, full table
captured verbatim, TARGETs graduated to MEASURED (4/4 cells MET).
Court-auditor sandbox flips and the gate-runner aggregate are downstream
lanes, not run here.

## Identity (GAP-6 backfill)

Backfill lane GAP-6, 2026-11-01, closing the F2 schema-drift finding
witnessed by F2-15: this receipt originally cited no subject sha256 —
identity rode on lease statements alone (R_missing_identity-lite).
Backfilled identity for the exact on-disk subject:

- **Subject**: `crates/bench/benches/bn3.rs`, sha256
  `5bc657174eaff206dac209dca2088f7728b7e0410a2fcdc66296caf2d5014745`,
  14,646 bytes, as on disk 2026-11-01.
- **F2-15 flip verdict (this bench)**: mutation class "substitute the
  binding-alias comparator (SHIM-DOE4 class)" — alias wrapper
  118.9 -> 232.2 ns/pair, **1.95x, FLIPPED**; sandbox restore sha-exact.
  Executed by the F2-15 court-auditor; GAP-6 transcribes the audit's
  verdict into `f2-15-bench-flips.md`, it does not re-measure.
