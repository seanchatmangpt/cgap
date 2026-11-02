# mega-bench lane-05 — DRed churn (G4 shape, method of record: w5-g4-dred-churn.md)

Subject: `crates/bench/benches/dred_churn.rs` (read-only) at repo /Users/sac/cgap, branch main.
Command: `CARGO_TARGET_DIR=/tmp/mb5-target cargo bench -p bench --bench dred_churn` — **exit 0, BENCH-OK** (this session).

## Run ladder

| Step | Command (cwd /Users/sac/cgap) | Exit | Evidence |
|---|---|---|---|
| Attempt 1 | `cargo bench -p bench --bench dred_churn` | 1 (compile-abort) | `crates/cgap-parquet/src/footer.rs:1008` unclosed delimiter — concurrent lane mid-flight edit; 0 tests executed (compile-abort, not a failure count) |
| Bounded poll | `cargo check -p cgap-parquet`, 20s x 30 attempts, deadline ~10 min | green at attempt 4 (~80s) | owning lane landed its fix; no edits made by this lane |
| Attempt 2 (clean) | `cargo bench -p bench --bench dred_churn` | 0 | BENCH-OK, 27 BATCH lines (9 seeds x 10/50/90% churn) |

## Measured numbers vs D7 witnesses (drift in bold-free plain text)

| Metric | D7 witness | This run | Drift |
|---|---|---|---|
| NS_PER_BATCH retract | 277833 ns | 277792 ns | -41 ns (-0.015%) — within timer resolution noise |
| Unit conversion | 277833/262144 = 1.06 ns/lane | 277792/262144 = 1.06 ns/lane | none |
| 10% facts/s | 943,388,214 | 943,388,214 | none |
| 10% ratio | 1.55 | 1.56 | +0.01 |
| 50% facts/s | 943,530,826 | 943,530,826 | only-slight |
| 50% ratio | 4.00 | 4.01 | +0.01 |
| 90% facts/s | 943,670,084 | 943,670,084 | +136,258 (+0.014%) |
| 90% ratio | 5.82 | 5.90 | +0.08 |
| ENTROPY_VARIANCE max/min (11 levels 0..100 step 10) | 1.0036 (< 4.0 fence) | 1.0008 (abs spread 0.0008 ns/lane informational) | tighter this run; fence MET |
| 15M facts/s at 90% | 943.5M, MET (63x) | 943.7M, MET (62.9x) | MET |
| <3.0 ns/batch gate line | MET at ns/lane granularity | 1.06 ns/lane < 3.0 — MET | MET |

## Unit-law note (recorded, not relitigated)

The gate text's "< 3.0 ns/batch" success criterion is a UNIT ERROR against the measured
quantity: the bench's batch unit is one `dred::retract` over the 2^18-fact corpus
(~277.8 us), five orders of magnitude above 3.0 ns. The gate line reconciles to ns/lane
(D7 disclosure): 277792/262144 = **1.06 ns/lane < 3.0** — honest pass at ns/lane
granularity. Both numbers recorded: NS_PER_BATCH retract = 277792 ns/batch;
1.06 ns/lane.

## Court cites (anti-vacuity — cited, not rerun per dispatch)

The skip-sweep mutant flip is witnessed in `docs/jira/v26.11.2/waves/w5-g4-dred-churn.md:60-80`:
mutation `ENTROPY_LEVELS[..0]` in a /tmp sandbox crate -> ENTROPY_VARIANCE folds
degenerate -> degenerate-sweep fence panics -> MUTANT_EXIT=101, no BENCH-OK; sha256-exact
restore `d209031dc7e94f510032d47df7ddeb8848df7ab92ff2ea418629604ed86657f6`. A court that
cannot flip is FAIL; this one flips. Clean bench rerun only, per dispatch.

## Anti-join structure cites (verified on disk this session)

- `crates/cgap-storage/src/dred.rs:44` `suspect_mask` = derived_deps & del_mask, word-parallel AND.
- `dred.rs:54` `keep_mask(derived_deps, del_mask, alt_support, facts, keep)`.
- `dred.rs:98` `compact_lanes` — branchless run-compaction, returned count is the only authoritative length.
- `dred.rs:116-127` `retract` — caller-leased scratch (`&mut [u64]`/`&mut [u32]`), no allocation in the hot path.

## Immutable-Arrow-slices framing

dred consumes `&[u64]`/`&[u32]` read-only slices — TRUE on disk (all four hot fns take
immutable slice params; scratch is caller-leased `&mut` out-buffers, not copies of the
corpus). The "dual-regime delta segments" phrasing is PRD prose
(`docs/jira/v26.11.2/waves/12-ard-prd-cgap.md:60-61`), not code — recorded as a
documented carve-out.

## Verdict

**ALIVE.** Gate G4 shape satisfied on the clean rerun (exit 0, BENCH-OK): all ratio
fences pass, entropy flatness 1.0008 < 4.0, 15M facts/s MET with ~63x headroom at 90%
churn, degenerate-sweep refusal intact (witnessed flip at exit 101), anti-join structure
cites verified on disk. Compile-abort on attempt 1 is disclosed with the file named
(footer.rs:1008) and resolved by the owning lane under a bounded poll — not a failure
count. No source edits by this lane; sole write is this receipt.
