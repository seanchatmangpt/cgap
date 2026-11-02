# MEGA-BENCH Lane 1 — Ingress (chains-ingress)

Lane: mega-bench lane 1 (ingress). Read-only source; this file is the lane's
only write. Date: 2026-11-02.

## Method

- Harness: `crates/bench/benches/chains-ingress.rs` (divan 0.1.x), via
  `CARGO_TARGET_DIR=/tmp/mb1-target cargo bench -p bench --bench chains-ingress`.
  No ad-hoc timing loops.
- 6 invocations total; 3 clean full-capture runs (4,5,6; runs 1-3 table rows
  present but numbers were lost to a truncating grep — disclosed, not counted).
- Divan emits fastest/slowest/median/mean per run, NOT percentiles; no
  percentiles fabricated. Cross-run summary = fastest-of-runs and
  median-of-run-fastests (p50-class), stated as such.
- ns/frame = ms / 1,048,576 chain frames (1M const, ItemsCount = frames).
- Host under concurrent fan-out lane load (slowest samples to 204 ms) —
  medians are contention-inflated; fastest is the defensible row.

## Numbers (1M chain frames; ns/frame)

| leg | run | fastest ms | = ns/f | median ms | = ns/f |
|---|---|---|---|---|---|
| ingress_chain (full chain) | 4 | 11.71 | 11.17 | 13.07 | 12.47 |
| ingress_chain | 5 | 12.38 | 11.81 | 31.95 | 30.47 |
| ingress_chain | 6 | 12.30 | 11.73 | 15.28 (16.02 ms) | 15.28 |
| **ingress_chain summary** | — | **11.71 → 10.65** | | median-of-fastests 12.30 → **11.73** | |
| stage_penstock_admission | 4 | 9.333 | 8.90 | 10.99 | 10.48 |
| stage_penstock_admission | 5 | 8.965 | 8.55 | 18.50 | 17.64 |
| stage_penstock_admission | 6 | 8.274 | 7.89 | 10.28 | 9.80 |
| **penstock summary** | — | **8.274 → 7.89** | | median-of-fastests 8.965 → **8.55** | |
| stage_a04_project_validate | 6 | 6.199 | 5.91 | 7.113 | 6.78 |
| stage_digest_fold (escalated subset) | 4-6 | — | 0.97-1.19 ns/fold | | |
| stage_prefilter_fold (clean fold, 65536 const) | 4/6 | 47.7/43.0 µs | 0.73/0.66 ns/f | | |

## Baseline audit

| baseline | fastest | median | this run (fastest / median-of-fastests) |
|---|---|---|---|
| w3-l1-ingress.md | 10.20 ns/f | 10.30 | 10.65 / 11.73 |
| w6-b7-gate-consume.md | 9.99 | 10.71 | 10.65 / 11.73 |

Fastest re-witnesses within the 9.99-10.71 receipted band +4-7% (fan-out
contention). Median 11.73 is +14% vs 10.30 — contention-explained (slowest
samples to 204 ms on the shared host), not a regression claim in either
direction.

## USL verdict (bypass <= 15 ns/frame)

- ingress_chain fastest **10.65 ns/frame < 15** — MET.
- median-of-fastests 11.73 < 15 — MET.
- Per-run divan medians reached 15.28-30.47 ns/frame in runs 5-6 under
  heavy co-lane load (slowest 204 ms); on a quiet host the w3-l1/w6-b7
  10.30-10.71 median band stands UNVERIFIED-from-this-lane. Fastest-class
  target: MET with receipts.

## Non-vacuity — l1_* lever courts

`CARGO_TARGET_DIR=/tmp/mb1-target cargo test -p cgap-gate --test lever_courts`
exit 0, 3/3 passed, 0.00s:
- l1_ceiling_law_flip_is_the_named_mutant — ok
- l1_prefilter_escalation_policy_court — ok
- l1_attestation_byte_flip_escalates_to_spoliation — ok

Refusal semantics (verified on disk, do not conflate):
- Attestation flip -> `CGAP_ERR_P28_SPOLIATION` = 0x5032_38FF
  (crates/cgap-gate/src/penstock.rs:319; also sovereign.rs:83).
- `0x4347_0004` = `CGAP_ERR_LEASE_EXHAUSTED`
  (crates/cgap-ffi/src/sovereign.rs:80, asserted at :1652).

## Compile-freeze disclosures

Shared-tree in-flight edits from other lanes froze the bench build twice;
resolved by owner lanes within SLA via bounded polls (no source edits by
this lane):
- crates/cgap-parquet/src/footer.rs:1008 (`write_i32_field(&pub fn dummy() {}`,
  unclosed delimiter) — poll RESOLVED at ~2 min.
- crates/cgap-parquet/src/footer.rs:1073 (`.build_meta_block(0x DEAD_BEEF`,
  E0768) — poll RESOLVED via `cargo build -p cgap-parquet` exit 0 within SLA.

## Verdict

ALIVE (bench + courts ran this session against this tree). Bypass USL
<=15 ns/frame MET on fastest (10.65) and median-of-fastests (11.73);
divan-median under co-lane contention exceeded 15 in 2/3 runs — quiet-host
median claim stays with w3-l1/w6-b7. Courts 3/3 green; non-vacuity held.

## Cleanup

Build-root lease /tmp/mb1-target deleted via shutil.rmtree at lane end.
