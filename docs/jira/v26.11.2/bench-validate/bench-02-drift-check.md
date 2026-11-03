# Lane FS-01 — chains-seal drift check vs F-06 bench-02 (2026-11-02)

Repo /Users/sac/cgap @ main. Sole file written: this one
(`bench-validate/bench-02-drift-check.md`). No git commands. Contract
originally named `bench-02-seal-bn3.md`; the original F-06 lane landed
that file first — per coordinator collision notice, this lane converted
to an INDEPENDENT DRIFT CHECK and landed under the new filename.
F-06's landed receipt (`bench-02-seal-bn3.md`) is cited, not overwritten.

## §1 Claims under test + register rows

- Row 12 (11-measured-claims.md row 12, MEASURED / FAILED-BY-MEASUREMENT
  verdict): fused ASC450-into-P28 row fold 3.04 µs/row @65536 (199.1 ms)
  vs unfused control 2.37 µs/row (155.5 ms); target <1.45 µs/row.
- R-15 re-witness band (redteam/fix-18-measured-claims-sweep.md W5):
  3.086 / 3.123 µs/row fastest/median.
- F-06 landed band (bench-02-seal-bn3.md §3.1): 3.0748–3.0788 fastest,
  medians 3.0977–3.1030 µs/row (loadavg1 23.58–36.61).

## §2 Method actually applied

- Host: Mac15,9, Apple Silicon 16-core (no SMT), Darwin 25.2.0.
- Build root lease: CARGO_TARGET_DIR=/tmp/fs01-target (deleted at lane
  end). Build exit 0 (`cargo bench -p bench --bench chains-seal
  --no-run`, 18.15 s).
- Ambient disclosure (mandatory): host ran HOT all session — loadavg1
  per leg below; never below 5.45, peaked 26.00 during leg 1. NO
  CONTENTION-SUSPECT guard exists on chains-seal (guard is
  chains-ingress-only per lane-rc11; same disclosure as R-15 W5 and
  F-06 §2) — verdicts rest on cross-lane agreement, not a quiet-host
  marker.
- Runs: 3 fused legs + 1 unfused-control leg, all
  `cargo bench -p bench --bench chains-seal`, all EXIT=0, 100 samples
  each. stderr+stdout logs /tmp/fs01-work/seal{1,2,3}.{out,err},
  unfused2.{out,err}.
- Divergence disclosed: first unfused attempt with filter
  `governance_chain_per_row/` (trailing slash) ran ZERO benches,
  EXIT=0 (divan substring filter with `/` matched nothing) — a silent
  skip by divan's default; caught by inspecting the empty output and
  re-run loudly. Recorded as a method note, not a bench result.

## §3 RAW results

### §3.1 governance_chain_fused_per_row, N=3 legs (no guard on this bench)

| leg | fused@1024 f/slow/med | fused@65536 f | med | µs/row f | µs/row med | loadavg1 pre/post |
|---|---|---|---|---|---|---|
| 1 | 3.367 / 58.05 / 3.633 ms | 200.2 ms | 202.8 ms | 3.0548 | 3.0945 | 22.20 / 26.00 |
| 2 | 3.112 / 4.977 / 3.167 ms | 200.3 ms | 202.5 ms | 3.0563 | 3.0899 | 24.32 / 19.16 |
| 3 | 3.108 / 3.369 / 3.169 ms | 200.4 ms | 202.2 ms | 3.0579 | 3.0853 | 17.11 / 14.28 |

Leg 1 carried one extreme slowest-sample (58.05 ms @1024, 659.5 ms
@65536) at the peak ambient (26.00) — a contention spike, retained raw
per the raw-table law, not excluded.

### §3.2 governance_chain_per_row (unfused control), 1 leg

@65536: fastest 155.9 ms = **2.3788 µs/row** / median 156.8 ms =
2.3926 µs/row (loadavg1 10.61 pre / 8.40 post). @1024: fastest
2.422 ms.

## §4 Measured-vs-claimed delta table

| claim | claimed | measured (this lane) | delta | basis |
|---|---|---|---|---|
| row 12 fused fastest | 3.04 µs/row | 3.0548–3.0579 (3 legs) | +0.49% to +0.59% | inside red-10 4.5% tol |
| row 12 fused median | — | 3.0853–3.0945 | n/a | vs R-15 median 3.123: -0.9% to -1.2% |
| R-15 band | 3.086/3.123 | 3.0548–3.0579 / 3.0853–3.0945 | fastest -0.9% to -1.0% below band; median straddles | typed, band-adjacent |
| row 12 unfused control | 2.37 µs/row | 2.3788 fastest | +0.37% | inside tol |
| F-06 landed band | 3.0748–3.0788 f | 3.0548–3.0579 f | -0.55% to -0.65% vs F-06 | same-day, same bench, same host |

## §5 Verdicts per number

- **Row 12 fused (3.04): CONFIRMED-IN-BAND** — 3.0548–3.0579 µs/row
  fastest (+0.49% to +0.59%), median 3.0853–3.0945; the
  FAILED-BY-MEASUREMENT verdict re-witnessed: fused/unfused ratio
  1.2848 this lane vs F-06's 1.278 — fused still regresses ~28% over
  the unfused control.
- **Row 12 unfused control (2.37): CONFIRMED-IN-BAND** — 2.3788 µs/row
  (+0.37%), reproducing the P3-19 baseline side.
- **R-15 band (3.086/3.123): CONFIRMED-IN-BAND (median), fastest
  band-adjacent below** — medians 3.0853–3.0945 sit on the 3.086 band
  floor; fastest values 3.0548–3.0579 run 0.9–1.0% below, inside the
  4.5% reproduction tolerance.
- **F-06 drift check: SAME-DAY AGREEMENT** — my fastest band
  3.0548–3.0579 vs F-06's 3.0748–3.0788: -0.55% to -0.65% (max gap
  0.031 µs/row, well inside tolerance); medians overlap
  (3.0853–3.0945 vs 3.0977–3.1030, gap 0.012–0.018 µs/row). No drift
  registered.
- **UNVERIFIED remains**: quiet-host scope (no guard installed on
  chains-seal, all legs at loadavg1 5.45–26.00); commit SHA (no-git
  contract).

## §6 Full transcript

```
uptime (start): loadavg 5.45 7.05 8.29
mkdir -p /tmp/fs01-work
CARGO_TARGET_DIR=/tmp/fs01-target cargo bench -p bench --bench chains-seal --no-run -> EXIT=0 (18.15s)
uptime -> 22.20 10.81 9.59
cargo bench -p bench --bench chains-seal -- governance_chain_fused_per_row -> EXIT=0 (leg 1)
uptime -> 26.00 12.76 10.34
cargo bench ... -- governance_chain_fused_per_row -> EXIT=0 (leg 2)
uptime -> 24.32 12.63 10.31 pre / 19.16 12.22 10.21 post
cargo bench ... -- governance_chain_fused_per_row -> EXIT=0 (leg 3)
uptime -> 17.11 12.01 10.16 pre / 14.28 11.74 10.12 post
cargo bench ... -- "governance_chain_per_row/" -> EXIT=0, ZERO BENCHES RAN (divan filter matched nothing; loud re-run below)
cargo bench ... -- governance_chain_per_row -> EXIT=0 (unfused control + filter matched only unfused; loadavg 10.61 pre / 8.40 post)
```

## §7 Standing

ALIVE for every cell: each names a command + exit run this session
against the exact bench target. Target cleanup at lane end:
`python3 -c "import shutil; shutil.rmtree('/tmp/fs01-target', True)"`.

```json
{"lane":"FS-01 (drift check, bench-02 successor)",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/bench-validate/bench-02-drift-check.md"],
 "commands":[
  "cargo bench -p bench --bench chains-seal --no-run (exit 0)",
  "cargo bench -p bench --bench chains-seal -- governance_chain_fused_per_row x3 (exit 0 x3)",
  "cargo bench -p bench --bench chains-seal -- governance_chain_per_row (exit 0)"],
 "courts_run":["cross-lane drift reconciliation vs bench-02-seal-bn3.md + R-15 W5 + register row 12 (agreement, all deltas <=1.5%)"],
 "standing":"ALIVE",
 "divergences":[
  "F-06 landed bench-02-seal-bn3.md first; lane converted to drift check per coordinator notice",
  "first unfused leg with trailing-slash divan filter ran 0 benches at exit 0 (silent skip class); caught and re-run loudly",
  "no CONTENTION-SUSPECT guard on chains-seal (chains-ingress-only); hot ambient 5.45-26.00 disclosed per leg"]}
```
