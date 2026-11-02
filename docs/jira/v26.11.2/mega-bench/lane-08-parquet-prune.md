# Lane 08: Parquet Stats-Prune Mega-Bench (rg_prunes)

Campaign: v26.11.2 starDF mega-bench, lane 8. Subject:
`crates/cgap-parquet/src/scan.rs` pruning surface (`rg_prunes`,
`scan_u64_pred`, `pages_parsed`). READ-ONLY lane: repo untouched; only
writes are this receipt and /tmp scratch.

## Gate ladder

| Gate | Command (cwd /Users/sac/cgap) | Exit | Evidence |
|---|---|---|---|
| D8 court (regression) | `CARGO_TARGET_DIR=/tmp/mb8-target cargo test -p cgap-parquet --features
  std --test stats_skip_court` | 0 | 2 passed; 0 failed |
| Full parquet suite | `CARGO_TARGET_DIR=/tmp/mb8-target cargo test -p cgap-parquet --features std`
  | 1 | 192 passed / 1 failed / 1 ignored; failure named below |
| Probe build | `CARGO_TARGET_DIR=/tmp/mb8-target cargo build --release --manifest-path
  /tmp/mb8-probe/Cargo.toml` | 0 | release build |
| Probe run 1 | `/tmp/mb8-target/release/mb8-probe` | 0 | tables below |
| Probe run 2 (loaded) | `/tmp/mb8-target/release/mb8-probe` | 0 | stability rerun |

**Pre-existing (other-lane) failure, named, not folded into lane-08
tally:** `footer::courts::max_row_groups_over_capacity_refused_with_exact_payload`
panics at `crates/cgap-parquet/src/footer.rs:1151` ("lawful arm must
fail later at the buffer check"); footer.rs carries an in-flight
working-tree diff (+327/-30) owned by another lane. stats_skip_court
(the pruning gate this lane exercises) is 2/2 green.

## Method

- Probe crate `/tmp/mb8-probe` (path dep `cgap-parquet` features=["std"],
  CARGO_TARGET_DIR=/tmp/mb8-target). Fixtures built through the D8
  writer seam `write_fixture_file_with_bounds`; a probe-local assembler
  (mirroring `assemble_file` wire shape via the crate's pub
  `thrift` encoders) joins 256 seam-produced single-RG pages into one
  file with 256 row groups (the seam emits one RG per file).
- Corpus: 256 RGs x 64 u64 rows, RG i holds [64i, 64i+63]; domain
  [0, 16384). Predicate [0, 64k-1] fully covers k groups.
- Timing: warmup scan (fills resolve cache; pages_parsed stops
  moving), then 2000 timed scans; ns/scan = elapsed/iters. Prune-only
  arm: 20000 iters on a disjoint predicate. pages_parsed recorded
  cold (one scan from fresh open) and warm delta over the timed loop.
- Results asserted per tier: hit rows equal the derived expectation,
  and pruned-file rows identical to the no-bounds control file.

## Tier table (run 1, quiet machine)

tier | pred | pruned ns/scan | cold pages / warm delta | control ns/scan | delta ns | hit rows
---|---|---|---|---|---|---
~1% (3/256 RGs) | (0,191) | 1496 | 3 / 0 | 55610 | +54114 | 192
~10% (26/256) | (0,1663) | 9418 | 26 / 0 | 58225 | +48807 | 1664
~50% (128/256) | (0,8191) | 37705 | 128 / 0 | 64906 | +27201 | 8192
100% | (0,16383) | 72866 | 256 / 0 | 72018 | -847 | 16384
pruned-all (0%) | 2^40-range | 241 | 0 / 0 | 54674 | +54433 | 0

- `pages_parsed`: cold count equals hit-group count exactly
  (3/26/128/256/0); warm delta 0 in every tier — zero page I/O on
  pruned groups (D8 counter observable).
- 100% tier delta is noise-level (-847 ns over 2000 iters): the prune
  check costs ~0.9 ns/RG, invisible at 100% hit rate.
- Run 2 (loaded machine): same shape, absolute numbers up to ~2.2x
  (prune-only 527 ns/scan = 2.1 ns/RG; deltas hold relative). Min
  of runs: 0.9 ns/RG; max 2.1 ns/RG.

## Criterion verdict: <=500 ns per skipped row group

Prune-only: 241 ns/scan / 256 RGs = **0.9 ns per pruned row group**
(run 2, loaded: 2.1 ns). PASS — 2-3 orders of magnitude inside the
criterion. The skip is a two-compare range check
(`scan.rs:320-325`): no page header parse, no bytes touched.

## Fail-open witnesses (corrupted statistics, /tmp fixtures)

Reader semantics (scan.rs:594-647): bounds admit only the fixed-width
8/8-byte min/max case; anything else leaves None -> full scan.

| Mutation | Byte flip (footer) | Open | rg0 bounds | Scan vs control | Pages |
|---|---|---|---|---|---|
| A: field-id flip | 0x58 -> 0x78 (max_value id 5 -> unknown) | OK | [None] | identical rows,
  assert-eq | >0 (scanned) |
| B: length truncation | 0x08 -> 0x03 (max_value 8 -> 3 bytes) | Err(MalformedFooter) | n/a | typed
  refusal, no panic | n/a |

- A is the fail-open path: open succeeds, bounds slot None, scan
  returns rows identical to the no-bounds control (assert_eq), pages
  parsed — cost regresses to full scan, results unchanged.
- B is a typed refusal at open, recorded honestly: the truncated
  stats struct desyncs the compact-protocol field walk, the footer
  parse refuses the whole file. Typed error, never a panic or a
  wrong answer. Both mutations are real byte flips in the statistics
  region of a /tmp fixture copy.

## Anti-vacuity

D8's court is already proven non-vacuous: the bounds-stripped writer
mutation flips `stripped_bounds_mutation_flips_prune` (panic
"MUTATION FLIP FAILED", cargo exit 101) —
`docs/jira/v26.11.2/waves/w5-stats-skip.md:46-47`, cited from disk
this session. This lane adds the byte-flip witnesses above; the probe
asserts pruned-file rows == control rows on every tier, so a prune
that changed results would fail the probe loudly.

## Verdict

- stats_skip_court 2/2: PASS (exit 0).
- Prune cost criterion: PASS (0.9-2.1 ns/RG vs <=500 ns/RG).
- Zero page I/O on pruned groups: PASS (pages_parsed 0 cold / warm
  delta 0 on pruned tiers).
- Prune-vs-full-scan delta: +54.1 us/scan saved at 1% hit rate, +48.8
  us at 10%, +27.2 us at 50%; breakeven at ~100% (noise).
- Fail-open: PASS via field-id flip (results identical to control);
  truncation of a stats value length is a typed MalformedFooter
  refusal at open, recorded as-is.
- Full suite: PARTIAL — 1 pre-existing other-lane failure named above
  (footer.rs:1151, in-flight +327-line diff, not lane-08's subject).
- Standing: ALIVE for the measured pruning surface, commands + exits
  this session. Probe + target root under /tmp; target lease deleted
  at cleanup.

## Cleanup

/tmp/mb8-target deleted via
`python3 -c "import shutil; shutil.rmtree(p, True)"` after the final
run (lease law). Probe crate left in /tmp/mb8-probe.
