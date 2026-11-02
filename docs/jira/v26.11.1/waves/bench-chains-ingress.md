# P3-16 BENCH-INGRESS Receipt

Lane: P3-16 (BENCH-1 crash-resume) · 2026-11-01 · repo /Users/sac/cgap

## Subject

- `crates/bench/Cargo.toml` — all five `chains-*` `[[bench]]` targets
  (harness = false) verified present (lines 79-97); no edit needed.
- `crates/bench/benches/chains-ingress.rs` — prior-crash residue was a
  complete 615-line bench (consts 65_536 / 1_048_576, seed
  0xBF58_476D_1CE4_E5B9, black_box + per-pass checksum asserts vs
  preflight fold). One fix: `stage2` needed an explicit lifetime
  `'a` on slot + `FrameView<'a>` (invariance compile error).

## Commands

- `CARGO_TARGET_DIR=/tmp/p316-target cargo bench -p bench --bench chains-ingress`
  → exit 0, full 4-bench table.
- `CARGO_TARGET_DIR=/tmp/p316-target cargo bench -p bench --bench ingress`
  → exit 0 (same-session legacy comparison).

## Results (mean, FRAMES=1_048_576; chain units)

| bench | mean | rate |
|---|---|---|
| ingress_chain | 102.7 ms | 10.2 Mframe/s |
| stage_a04_project_validate | 6.64 ms | 157.8 Mitem/s |
| stage_penstock_admission | 9.66 ms | 108.5 Mitem/s |
| stage_digest_fold | 93.38 ms | 11.22 Mitem/s |

Legacy `ingress` same session: a14_penstock_admission 1_048_576 mean
7.71 ms (136 Mitem/s); cgap_admit_frame 1_048_576 mean 5.60 ms.

## PARETO (mandatory)

Stage sums at FRAMES=1_048_576: digest 93.4 + penstock 9.7 + A04 6.6 =
109.7 ms of segment work vs 102.7 ms chain mean (stages measured over
all frames; the chain pays digest only on the admitted subset, hence
sum > chain).

**~80% segment: `stage_digest_fold` — the BLAKE3 payload digest.**
93.38 ms is ~85% of segment cost and ~91% of the chain mean. The two
admission stages combined are ~15%. A frame costs ~98 ns end-to-end,
of which ~89 ns is hashing.

**The 20% lever**: stop hashing every admitted frame at full width.
Concretely — (a) hash only on policy triggers (first-frame-per-flow or
reserved-flag mutation), or (b) fold a cheap 8-byte non-crypto
prefilter into the dam's FrameView and escalate to BLAKE3 only on
prefilter collision/suspicion. Either bounds the digest to the
suspicious subset and moves chain cost to ~9.3 ns/frame
(admission-only), a >10x chain-level lever; the admission stages it
leaves behind are already at 108-158 Mitem/s.

## Court / anti-vacuity

Every timed pass asserts its folded checksum against the untimed
preflight fold (chain, stage-1, stage-2, stage-3 separately; "degenerate
corpus: zero chain-admitted frames" guard). Mutation = corrupt any slot
byte or stage code → checksum assert fires; the bench is not vacuous.
`stage2` lifetime fix is on-disk (crates/bench/benches/chains-ingress.rs:152).

## Standing

ALIVE — both bench runs exit 0 this session with full tables;
`/tmp/p316-target` deleted post-run.
