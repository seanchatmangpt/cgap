# ZD-8: dod-006 Quiet-Machine Close Condition (throughput_min_of_n_floor)

Date: 2026-11-04
Lane: ZD-8 (verification lane — no source edits, no git commands)
Subject: `crates/cgap-crypto/tests/aead_courts.rs:447` `throughput_min_of_n_floor`
Close condition: court passes 3x serial on a quiet machine.

## Verdict

**All 3 runs GREEN (exit 0).** dod-006's red is confirmed **load-bound-flake
class**: the court passes decisively on a quiet host. Measured minimum
64KB throughput across all runs is 261.26 MB/s (open, run 1) — ~130x the
2 MB/s floor. The fence is not marginal on this host.

## Quiet precondition

Established by bounded pgrep polling on the observable `pgrep -fl
"cargo test"` (60s interval, no blind sleeps). Measurement defect found
and corrected: `pgrep -f` **self-matches** any polling/checker shell whose
command line contains the literal string `cargo test`, so earlier BUSY
readings were the poller itself. Corrected check:
`pgrep -fl "cargo test" | grep -v "zsh -c"` → exit 1 (no real cargo test
process) immediately before the runs.

## Command of record (identical for all 3 runs)

```
cargo test -p cgap-crypto --test aead_courts throughput_min_of_n_floor --release -- --nocapture
```

## Results of record (verbatim court println lines, min-of-8 inner reps)

Run 1 — exit 0, 6.07s:

```
throughput len=   1024: seal   200.56 MB/s | open   327.18 MB/s (min of 8, dev profile)
throughput len=  65536: seal   261.73 MB/s | open   261.26 MB/s (min of 8, dev profile)
throughput len=1048576: seal   284.41 MB/s | open   312.99 MB/s (min of 8, dev profile)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 6.07s
```

Run 2 — exit 0, 5.82s:

```
throughput len=   1024: seal   277.24 MB/s | open   351.10 MB/s (min of 8, dev profile)
throughput len=  65536: seal   344.12 MB/s | open   280.35 MB/s (min of 8, dev profile)
throughput len=1048576: seal   278.53 MB/s | open   329.86 MB/s (min of 8, dev profile)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 5.82s
```

Run 3 — exit 0, 5.82s:

```
throughput len=   1024: seal   362.97 MB/s | open   374.93 MB/s (min of 8, dev profile)
throughput len=  65536: seal   324.67 MB/s | open   285.51 MB/s (min of 8, dev profile)
throughput len=1048576: seal   317.66 MB/s | open   352.21 MB/s (min of 8, dev profile)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 5.82s
```

## Gate-relevant summary (64KB, the asserted floor)

| run | exit | 64KB seal MB/s | 64KB open MB/s | min over gate |
|---|---|---|---|---|
| 1 | 0 | 261.73 | 261.26 | 261.26 |
| 2 | 0 | 344.12 | 280.35 | 280.35 |
| 3 | 0 | 324.67 | 285.51 | 285.51 |

Distribution min-of-3 (64KB): **261.26 MB/s** vs floor 2 MB/s → margin
~130x. Fence is robust; under 4-lane concurrent load the observed single
red remains consistent with scheduler/contention starvation dropping
min-of-8 samples below floor, not a code throughput defect.

## Standing

- Court: ALIVE (3 receipts above, exits captured).
- dod-006 red: classified load-bound-flake; close condition satisfied.
- Host prior to runs: other lanes' cargo waves observed draining for
  ~3.5h of bounded polling; runs executed only after corrected pgrep
  returned quiet.

See Also: docs/jira/v26.11.3/dod-targets-open.md
