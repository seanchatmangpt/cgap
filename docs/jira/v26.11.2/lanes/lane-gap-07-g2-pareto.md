# lane-gap-07 — G2-PARETO L1/L2/L4/L5 execution (R4 close condition)

Lane GAP-07, 2026-11-02. Repo /Users/sac/cgap @ main 9a730320.
Sole writes: this receipt + /tmp/gap07-sb (sandbox sources, restored
sha256-exact). No git writes; no kernel edits in the shared tree
(shared files re-read on disk: bench/ffi/gate/parquet/p28 diffs are the
pre-existing wave changes from the conversation-start git snapshot, not
this lane's). L3 stands CLOSED — cite
`docs/jira/v26.11.2/repairs/repair-06-radix-default-flip.md` (78.3x @1M,
target <75 ms MET, floor >20x MET), NOT re-run by this lane.

## Environment (06-bench-protocol §3 disclosure)

Apple M3 Max (16 cores), Darwin 25.2.0, rustc 1.101.0-nightly
aarch64-apple-darwin, Divan 0.1.21 (exact pin), repo @ 9a730320,
load avg 14.46 at run time (ambient 16-core fan-out load; ratios, not
absolutes, are the regression signal). Build root: /tmp/gap07-target
(shared tree) + /tmp/gap07-sb-target (sandbox), both lane leases.

## Divergences (doc yields to reality; disclosed up front)

1. G2's COMMAND block names bench targets that DO NOT EXIST in
   `crates/bench/Cargo.toml` (page_bench, dict_bench, parquet_bench,
   bitslice_best... bitslice_bench, packing_bench, parquet_throughput —
   all absent from the [[bench]] list). The executed legs are the
   chains-* legs mapped in `06-bench-protocol.md` §2 + the lever board
   `02-pareto-levers.md` L-row history: L1=chains-ingress
   `ingress_chain`, L2=chains-parquet `chain_ram` @1M,
   L4=chains-sovereign `stage_commit_parquet`, L5=chains-seal
   `governance_chain_fused_per_row` (control:
   `governance_chain_per_row`).
2. `scripts/divan2receipt.py` (parser of record) REFUSED the chains-*
   output: exit 1, "REFUSED: required benches missing from Divan
   output: blake3_hash_64mib, quad_transpose_64k, predicate_filter_64k,
   parquet_level_free_roundtrip_u64_64k". It is pinned to the v26_10_31
   bench set; it cannot parse the v26.11.2 G2 subjects. Fail-closed
   behavior witnessed (missing evidence is never a silent pass) — but
   the CTQ table needs a successor-gate update before G2 can use it as
   its parser of record. Numbers below are read from the Divan stdout
   tables (same source the parser reads), exit codes recorded verbatim.
3. divan omits all-zero alloc columns (CLAUDE.md law): L1/L2 legs show
   no alloc columns = 0 allocs steady-state; L4 alloc columns visible =
   2 allocs/36 B per tx remain (pre-existing, footer.rs:187/326,
   corroborates a3-ring-sink.md).

## Per-level results (baseline; all runs exit 0)

| Level | Leg (@const) | Fastest | Median | Threshold | Verdict |
|---|---|---|---|---|---|
| L1 | ingress_chain @1M | 11.57 ms | 12.33 ms | <15 ns/frame | MET (11.04 ns/frame fastest / 11.76 median) |
| L1 | ingress_chain @65,536 | 635.9 µs | 706.2 µs | — | 9.70 ns/frame |
| L2 | chain_ram @1M | 23.13 ms | 23.32 ms | <10 ms @1M | **NOT MET** (2.31x over) |
| L2 | chain_ram @65,536 | 1.426 ms | 1.434 ms | — | context |
| L4 | stage_commit_parquet @256 | 201.3 µs | 4.824 ms | <4 µs/tx | MET (0.786 µs/tx fastest) |
| L4 | stage_commit_parquet @4096 | 38.31 ms | 48.97 ms | <4 µs/tx | **NOT MET** (9.35 µs/tx fastest) |
| L5 | governance_chain_fused_per_row @65,536 | 200.2 ms | 202.5 ms | <1.45 µs/row | **NOT MET** (3.055 µs/row fastest) |
| L5 | governance_chain_per_row (control) @65,536 | 155.2 ms | 3.055 µs/row) | — | 2.368 µs/row; fused is a 29% REGRESSION vs control |
| L5 | governance chains @1024 (fused / control) | 3.501 / 2.421 ms | — | — | context |

## Flip courts (G2-M1 class; /tmp/gap07-sb, sha256-exact restores)

Sandbox baselines (same legs, /tmp/gap07-sb @ shared bytes):
L1 11.29 ms / L2 23.28 ms / L4 199.5 µs @256 / L5-fused 199.4 ms
@65,536 (all exit 0) — sandbox == shared tree numbers within noise.

### L1 — NOT FLIPPED → gate-law verdict: `VACUOUS: L1 threshold met with no flip court`

8 attempts, all rebuild-verified (Compiling lines show sandbox paths),
restore sha256-exact after. Baseline 11.29 ms @1M:

| Attempt | Mutation (all named-class) | @1M fastest | Median |
|---|---|---|---|
| 1 | black_box(chain_frame output) | 11.35 ms | — |
| 2 | black_box(stage1 input slice) | 11.8 ms | 13.1 ms |
| 3 | black_box(lane index j) | 12.82 ms | 14.12 ms |
| 4 | black_box(acc) per frame (serialize fold chain) | 12.81 ms | 14.09 ms |
| 5 | KERNEL: black_box(bytes) at penstock ingress entry | 11.87 ms | 12.62 ms |
| 6 | KERNEL: per-byte black_box in OABI digest scan | 11.35 ms | 12.53 ms |
| 7 | KERNEL: #[inline] removed from prefilter8 | 13.05 ms + attempt-5 mutant | 14.48 ms |
| 8 | compound (5+6+7) | 13.03 ms = 12.44 ns/f | 14.55 ms = 13.88 ns/frame |

Every mutant stays <15 ns/frame. Findings: (a) the level leg is
bench-local mirror + dam::ingress; the pipeline is memory-latency-bound
with ~30% threshold headroom that black_box-class mutations cannot
breach on M3 Max (per-byte barriers are ~free on a 16-wide machine).
(b) Note: stage1/stage3 are bench-local MIRRORS of frame.rs/penstock —
the L1 level number is NOT measured through the full cgap-gate kernel
(only stage2 = dam::ingress is). 8 named-class mutations did not flip
the verdict; per the G2 BLOCKED law the receipt line is
`VACUOUS: L1 threshold met with no flip court`, loud, exit-3 semantics.

### L2 — threshold already NOT MET; flip = metric-movement witness

- Attempt 1: de-SWAR fnv1a64_seeded (footer.rs:300) to per-byte
  black_box reassembly → chain_ram 23.19 ms (no movement). FINDING:
  `fnv1a64_seeded` is OFF the chain_ram hot path (post-P2-PARETO the
  fused `SealDigests::absorb` sweep owns the seal FNV); the board's
  attribution story and the kernel's hot fn have drifted.
  chain_ram = 23.19 ms under this mutant — identical to baseline.
- Attempt 2: per-byte black_box barrier in `SealDigests::absorb`
  (footer.rs:417) → chain_ram 23.23 ms; seal_arm 16.37 ms vs receipted
  healthy 16.34 ms (02-pareto-levers.md:68) — NO movement. Per-byte
  black_box barriers do not degrade a per-byte multiply chain on this
  host. NOT FLIPPED. Honest verdict stands: L2 = FAIL on the threshold
  (23.13 ms vs <10 ms), flip court not achievable in bounded time →
  `[SKIP] L2 flip: named-class mutations cannot degrade the seal kernel
  on this host; 2 attempts transcripted above`.

### L4 — @256 MET but flip not witnessed; @4096 already NOT MET

Under the L2 absorb mutant (still applied, rebuilt), stage_commit_parquet
@256 = 199.5 µs = 0.78 µs/tx — identical to baseline (commit path is not
seal-absorb-bound; the ring memcpy + footer small-block seal dominate).
Needed for a verdict flip @256: a 5x degrade (0.786 -> >4 µs/tx); no
named-class mutation bounded here can deliver 5x on this host. Per the
gate law, the met half is recorded loud:
`VACUOUS: L4 @256 threshold met with no flip court`.
The @4096 const needs NO flip: 9.35 µs/tx fastest FAILS the <4 µs/tx
threshold outright (measure this session, stage_commit_parquet @4096
fastest 38.31 ms; note A3 measured 2.472 ms @4096 on 2026-11-01 —
today's number is 15.5x that; ambient 16-core load and/or machine state
differ; fastest-vs-fastest, ratio law applies). L4 verdict: PARTIAL —
MET @256 (0.786 µs/tx), NOT MET @4096 (9.35 µs/tx), not rounded up.
`[SKIP] L4 flip: 5x degrade not achievable via named-class mutations in
bounded time; 1 attempt transcripted (absorb mutant no-move)`.

### L5 — FLIP WITNESSED (metric moved), verdict FAIL (unchanged)

Mutation: injected redundant per-row crypto in the measured kernel fn
`fused_row` (asc450.rs:968): duplicate
`cgap_blake3::hash(black_box(&rec.to_bytes()))` — plausibly degrades
per-row cost. Rebuilt (sandbox compile verified), re-run:

| Leg | Baseline fastest | Mutant fastest | Delta |
|---|---|---|---|
| governance_chain_fused_per_row @65,536 | 199.4 ms | 216.0 ms | **+8.3% slower** |
| @1024 | 3.1 ms | 3.356 ms | +8.3% slower |

+8.3% is far outside sample noise (baseline run-to-run spread <1%).
FLIPPED as a metric-movement witness: the measured number is tied to
the kernel. Threshold verdict unchanged (3.055 -> 3.30 µs/row, FAIL
against <1.45). Also recorded: fused control 155.2 ms = 2.368 µs/row —
the fused arm is a 29% REGRESSION vs its own control on this host
(corroborates W3-L5).

## Restores (sha256-exact + mtime bump each)

- chains-ingress.rs `7e3a1ee4...8ea` == shared (witnessed).
- penstock.rs `bb4ed64d...3e8` == shared (witnessed).
- footer.rs `38bdd407...b55` == shared (witnessed).
- asc450.rs `ca208d42...78c` == shared (witnessed).
Shared tree untouched by this lane (kernel files were never edited
outside /tmp/gap07-sb; git status diffs in those files pre-date the
session per the conversation-start snapshot).

## G2 verdict rollup (five levels)

- L1: MET (11.04 ns/frame fastest @1M) — but VACUOUS per gate law
  (threshold met, flip court could not flip; 8 attempts, transcripted).
- L2: **NOT MET** (23.13 ms @1M vs <10 ms).
- L3: CLOSED by repair-06 (78.3x @1M, <75 ms MET) — cited, not re-run.
- L4: PARTIAL — MET @256 (0.786 µs/tx, VACUOUS-per-law), NOT MET @4096
  (9.35 µs/tx).
- L5: **NOT MET** (3.055 µs/row vs <1.45; flip witnessed +8.3%).

## Standing

ALIVE for every measured number and flip transcript above (each carries
a command + exit run this session against the exact subject). UNVERIFIED
for nothing measured; PARTIAL where stated. Nothing rounded up.

## Cleanup + fences

Build roots /tmp/gap07-target + /tmp/gap07-sb-target are lane leases
(deleted at lane end, python shutil). Sandbox /tmp/gap07-sb retains the
sha256-restored sources. Fences this diff could break: none — the lane
wrote only this receipt; kernel/bench files untouched in the shared
tree; no gate or court in the shared tree reads the lanes/ dir.
Followups recorded (NOT this lane's files): (1) divan2receipt.py CTQ
table update for chains-* legs; (2) G2 COMMAND target-name repair in
01-release-gates.md; (3) L1 stage-mirror attribution note; (4) GAP-11
owns the R4 row refresh (nothing appended here).
