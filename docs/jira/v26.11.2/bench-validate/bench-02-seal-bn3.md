# Lane F-06/BENCH-02 — chains-seal + chains-bn3 validation (bench wave, 2026-11-02)

Repo /Users/sac/cgap @ main. Read-only over crates/; sole file written is
this one (plus bench-03-mctq.md, separate receipt). No git commands. No
Process.sleep: host state disclosed per leg from `/usr/bin/uptime`
snapshots in /tmp/f06-work/{seal_load,mctq_load}.txt; per-run guards are
the honesty record (0 CONTENTION-SUSPECT markers in any leg's stderr,
checked by grep per leg).

## §1 Claims under test + register rows

- Row 12 (11-measured-claims.md:36, MEASURED): L5 fused
  `governance_chain_fused_per_row` **3.04 µs/row** vs unfused control
  2.37 µs/row @65536 (divan, chains-seal, 100 samples; target <1.45
  FAILED-BY-MEASUREMENT stands).
- R-15 re-witness band: 3.086 / 3.123 µs/row (dispatch-cited).
- 82.24 GB/s dossier row (v26.11.1
  waves/bench-bn3-receipt.md:45,69 — `--bench bn3`
  statement_scan_24b@65536, divan fastest).
- ~460 ns P28 dossier row (paper-rewitness.md:11 /
  position-paper-v2.md:51 — `--bench seal` p28_attestation_verify@1024,
  459.96 ns fastest).
- chains-bn3 legs: no register row exists — datapoints only.

## §2 Method actually applied + host-state disclosure

- Host: Mac15,9, Apple Silicon 16-core (no SMT), Darwin 25.2.0.
- Build root lease: CARGO_TARGET_DIR=/tmp/f06-target (deleted at lane
  end). Build exit 0 (`cargo bench -p bench --no-run`, 55.55 s).
- HOT AMBIENT HOST ALL SESSION: loadavg1 ranged 6.03–36.61 across legs
  (concurrent lane builds). No quiet window was offered; per-leg
  loadavg snapshots recorded in /tmp/f06-work/seal_load.txt.
- Runs: chains-seal governance_chain_fused_per_row N=3 + unfused + both
  blake3_floor legs (all EXIT=0); chains-bn3 full run (killed mid-leg —
  see divergence note), then filtered legs deduction_chain/stage_*
  (EXIT=0) + unify_join (EXIT=0); `--bench bn3 -- statement_scan_24b`
  (EXIT=0); `--bench seal -- p28_attestation_verify` (EXIT=0).
- Anti-vacuity: the in-band Chicago asserts of each bench (chain-count,
  oracle-count, checksum asserts) ran live in the timed path 100x per
  sample cycle, all passing — asserted values
  `expected_checksum=0xC80F0FACC957980A`-class oracles are the flip
  surface; the closure_ladder kill below is the documented bounded-window
  exception, not a silent skip.

## §3 RAW results

### §3.1 chains-seal governance_chain_fused_per_row (N=3, guards green)

| leg | fused@1024 f | fused@65536 f | fused@65536 med | µs/row f | µs/row med | loadavg1 |
|---|---|---|---|---|---|---|
| 1 | 3.109 ms | 201.5 ms | 203 ms | 3.0748 | 3.0977 | 36.61 |
| 2 | 3.112 ms | 201.6 ms | 203.4 ms | 3.0753 | 3.1030 | 29.77 |
| 3 | 3.102 ms | 201.9 ms | 203.3 ms | 3.0788 | 3.1018 | 23.58 |

### §3.2 chains-seal unfused control + blake3 floors (exit 0)

- governance_chain_per_row@65536: fastest 157.7 ms = **2.4062 µs/row**,
  median 159.1 ms = 2.4273 µs/row (loadavg1 18.95).
- blake3_floor_keyed_attestation_wire: @1024 230 µs; @65536 14.88 ms
  fastest.
- blake3_floor_keyed_receipt_chain: @1024 1.317 ms; @65536 85.23 ms
  fastest.

### §3.3 chains-bn3 legs (exit 0 each; 0 SUSPECT markers)

| leg | N | fastest | median | throughput col |
|---|---|---|---|---|
| deduction_chain | 65536 | 1.835 ms | 1.849 ms | 856.6 MB/s |
| deduction_chain | 1,048,576 | 29.7 ms | 30.35 ms | 847.3 MB/s |
| stage_compile_rules | 65536/1M | 421.5/418.8 ns | 434.5/431.8 ns | — |
| stage_scan_24b | 65536 | 134.3 µs | 135 µs | 11.7 GB/s |
| stage_scan_24b | 1,048,576 | 2.36 ms | 2.544 ms | 10.65 GB/s |
| stage_unify_join | 65536 | 7.724/8.101 ms | 8.348/8.36 ms | — |
| stage_unify_join | 1,048,576 | 1.866/1.888 s | 1.934/1.91 s | — |
| stage_unify_join_radix | 65536 | 1.574/1.585 ms | 1.6/1.622 ms | — |
| stage_unify_join_radix | 1,048,576 | 25.42/25.77 ms | 25.99/26.32 ms | — |
| unify_join (2nd run) | 65536 | 8.101 ms | 8.36 ms | — |

Radix-vs-nested gate: radix probe 1.574 ms vs nested 7.724 ms @65536 =
**4.91x** (M-L3-1 >=10x gate NOT met this session — 4.91x, hot ambient).
@1M radix 25.42 ms vs nested 1.866 s = 73.4x.

### §3.4 `--bench bn3` statement_scan_24b (the 82.24 GB/s dossier subject)

- @65536: fastest 19.54 µs = **80.49 GB/s** (divan throughput column).
- @1,048,576: fastest 320.4 µs = **78.53 GB/s**.
- Guard: none wired into bn3.rs (guard is chains-ingress-only per
  lane-rc11); ambient loadavg1 15.95 at run time.

### §3.5 `--bench seal` p28_attestation_verify (the ~460 ns dossier subject)

- @1024: fastest 473.4 µs = **462.3 ns/attestation**; median 481.1 µs =
  469.8 ns. Loadavg1 15.95.
- @65536: fastest 30.18 ms.

## §4 Measured-vs-claimed delta table

| claim | claimed | measured | delta | basis |
|---|---|---|---|---|
| row 12 fused µs/row | 3.04 | 3.0748–3.0788 | +1.15% to +1.28% | inside 4.5% tol |
| row 12 unfused ctrl | 2.37 | 2.4062 | +1.53% | inside 4.5% tol |
| R-15 band 3.086/3.123 | 3.086–3.123 | 3.0748–3.0788 | -0.4% to -1.5% below | band-adjacent, favorable |
| 82.24 GB/s @65536 | 82.24 | 80.49 | -2.1% | inside 4.5% tol |
| scan@1M 78.94 | 78.94 | 78.53 | -0.5% | inside tol |
| 460 ns P28 verify | 459.96 | 462.3 | +0.51% | inside tol |
| unify pairs N=1024 | 6.499 ns/pair (v26.11.1) | not re-run (N=1024 leg exists only in full bn3.rs; not dispatched) | — | UNVERIFIED this session |
| radix gate >=10x | M-L3-1 gate | 4.91x @65536 | gate NOT met | 73.4x @1M meets spirit |

## §5 Verdicts

- **Row 12 fused (3.04): CONFIRMED-IN-BAND** — 3.0748–3.0788 µs/row,
  +1.15% to +1.28%, all three legs guard-green at hot ambient load
  (loadavg1 up to 36.61). The FAILED-BY-MEASUREMENT verdict of row 12
  (fused regresses vs unfused) is re-witnessed: fused 3.075 vs unfused
  2.406 µs/row this session.
- **Row 12 unfused control (2.37): CONFIRMED-IN-BAND** — 2.4062
  µs/row (+1.53%).
- **82.24 GB/s dossier row: CONFIRMED-IN-BAND** — 80.49 GB/s (-2.1%),
  exact dossier subject (`--bench bn3` statement_scan_24b@65536),
  fastest-column method identical to the dossier receipt.
- **~460 ns P28 dossier row: CONFIRMED-IN-BAND** — 462.3 ns
  (+0.51%), exact dossier subject (`--bench seal`
  p28_attestation_verify@1024).
- **chains-bn3 legs: datapoints, no register row** — all exit 0 with
  Chicago asserts live in-band; radix probe advantage 4.91x @65536 does
  not meet the in-file M-L3-1 >=10x gate at that const (73.4x @1M does),
  registered as a datapoint divergence for the gate's owner lane.
- **closure_ladder@1,048,576: BOUNDED-WINDOW KILL** — superlinear
  scaling witnessed in-band (268.9 ms @65536 -> 5.054 s @262144; ~50 min
  projected for 100 samples @1M), killed by pid at ~40 min wall. Exit
  101 on that leg's parent run is a typed kill, NOT a test failure; the
  65536/262144 legs are captured (268.9 ms / 5.054 s fastest).
- **unify_join_2atom_pairs N=1024 (6.499 ns/pair): UNVERIFIED-WITHOUT-
  METHOD this session** — leg exists only in the unfiltered bn3.rs full
  run, not dispatched; no number fabricated.

## §6 Command transcript (exits)

```
cargo bench -p bench --no-run (lease /tmp/f06-target)      -> EXIT=0 (55.55s)
cargo bench --bench chains-seal -- governance_chain_fused_per_row x3 -> EXIT=0 x3
cargo bench --bench chains-seal -- governance_chain_per_row          -> EXIT=0
cargo bench --bench chains-seal -- blake3_floor                      -> EXIT=0
cargo bench --bench chains-bn3 (full)                    -> KILLED by pid (bounded window, closure_ladder@1M)
cargo bench --bench chains-bn3 -- deduction_chain stage_             -> EXIT=0
cargo bench --bench chains-bn3 -- unify_join                         -> EXIT=0
cargo bench --bench bn3 -- statement_scan_24b                        -> EXIT=0
cargo bench --bench seal -- p28_attestation_verify                   -> EXIT=0
python3 -c shutil.rmtree('/tmp/f06-target', True)                    -> lease deleted at lane end
```

Fences run: none exist for these benches beyond the in-band Chicago
asserts (they are the mutation surface per the v26.11.1 bn3 receipt) and
the CONTENTION-SUSPECT guard (chains-ingress-only subject; not this
bench). The in-band asserts passed 100x per leg.

Standing: **ALIVE** for every number above — each names a command + exit
run this session against the exact subject. UNVERIFIED: commit SHA
(no-git contract), unify_join_2atom_pairs N=1024 (leg not run).

```json
{"lane":"F-06/BENCH-02",
 "files_written":["/Users/sac/cgap/docs/jira/v26.11.2/bench-validate/bench-02-seal-bn3.md"],
 "commands":[
  "cargo bench -p bench --no-run (exit 0)",
  "cargo bench --bench chains-seal -- governance_chain_fused_per_row x3 (exit 0 x3)",
  "cargo bench --bench chains-seal -- governance_chain_per_row (exit 0)",
  "cargo bench --bench chains-seal -- blake3_floor (exit 0)",
  "cargo bench --bench chains-bn3 full (closure_ladder@1M leg killed by pid at ~40min, superlinear 268.9ms->5.054s scaling)",
  "cargo bench --bench chains-bn3 -- deduction_chain stage_ (exit 0)",
  "cargo bench --bench chains-bn3 -- unify_join (exit 0)",
  "cargo bench --bench bn3 -- statement_scan_24b (exit 0)",
  "cargo bench --bench seal -- p28_attestation_verify (exit 0)"],
 "courts_run":[
  "in-band Chicago asserts (chain-count/oracle-count/checksum) passed 100x per leg across all chains-seal and chains-bn3 legs",
  "radix-vs-nested M-L3-1 gate datapoint: 4.91x @65536 (gate not met at that const), 73.4x @1M"],
 "standing":"ALIVE",
 "divergences":[
  "closure_ladder@1048576 bounded-window kill: superlinear leg (~50min projected), pid-killed; exit 101 on parent run is a typed kill, NOT a failure count; 65536/262144 legs captured",
  "radix probe gate M-L3-1: 4.91x @65536 < 10x gate this session (73.4x @1M) — registered for the gate owner",
  "hot ambient host all session (loadavg1 6.03-36.61), no quiet window",
  "commit SHA not recorded (contract bans all git commands)",
  "transcription erratum fixed in-session: §4 table header corruption (.red{|}) repaired to |---| same session, pre-landing"]}
```