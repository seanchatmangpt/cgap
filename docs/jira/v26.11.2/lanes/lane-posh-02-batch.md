# lane-posh-02-batch — Receipt (2026-11-02)

Lane POSH-2 · 1,365-fact batch amortization court · branch main.

## Files written

- `crates/csct/tests/posh_batch_court.rs` (new, ~300 lines)
- `docs/jira/v26.11.2/lanes/lane-posh-02-batch.md` (this receipt)

## Surface

Real collaborator: `cgap_bn3::unify::unify_batch_probe` (the leased-buffer
batch kernel), two-atom join rule `(?x P ?y), (?y Q ?z) -> (?x P3 ?z)`,
deterministic 1,365-fact corpus (seed `0xA20C_0FFE_E5A7_E000`; the
SplitMix64 multiplier `0xBF58_476D_1CE4_E5B9` appears only as the mix
constant — never conflated). Warm un-timed pass (with a zero-match
anti-vacuity assert), then 7 runs of a black_box-ed 10,000-iteration
driver loop; per-batch ns and per-frame ns reported as min/median/max
with `runs=7` labels and the no-p99 note (population 7 too small for a
tail quantile — lane-gap-03 pattern).
`FACT_WINDOW = 1_365` provenance comment: named coprime stride
(3·5·7·13, coprime with 12 B header / 20 B trailer / 64 B SHA-1 block /
4 B BE fields), HONEST NONE (mirrors `crates/cgap-git/src/pack_demux.rs:44-53`).

## Commands + exits (this session, exact subject)

- `cd crates/csct && CARGO_TARGET_DIR=/tmp/posh2-target cargo test
  --release --test posh_batch_court -- --nocapture` → **exit 101
  (typed GATE FAIL)**. Output: batch min=23.22/med=23.35/max=23.43
  ns/frame runs=7; mutated min=24.28/med=24.35/max=24.59;
  `[MUTATION-FLIP] branchy injection lifted median per-frame
  23.35 -> 24.35 ns/frame`; then the gate assert panics with the full
  method disclosure (batch-amortized Instant, 10k-iteration loop,
  per-frame = batch_ns/1365, warm pass, no per-frame Instant calls
  citing lane-b01 divergence 3).
- `cd crates/csct && CARGO_TARGET_DIR=/tmp/posh2-target cargo test
  --test posh_batch_court -- --nocapture` (debug) → **exit 0**,
  `[LOUDSKIP] POSH-2 amortization gate is release-only; debug build
  skips timing + mutation arms (non-fatal, exit 0)`.

## Gate outcome (honest, typed)

**GATE FAIL — 23.35 ns/frame median vs the dispatched 10.0 ns AMORTIZED
limit (2.3x over). Recorded, not rounded.** Precedent: lane-b01's 12 ns
typed FAIL. Corroboration that this is the kernel's real cost, not a
court artifact: `docs/jira/v26.11.1/waves/bench-chains-bn3.md:33-34` —
`stage_unify_join` N=65,536 = 6.808 ms = ~104 ns/row; the canonical
bench is 4.5x SLOWER per row than my measured 23 ns/fact (cache-resident
1,365-row batches). 10 ns/fact is below the real two-atom unify kernel's
demonstrated cost by a wide margin.

- Anti-vacuity WITNESSED: the branchy injection (unpredictable
  parity branch + dependent rotation chain per fact inside the same
  driver loop) lifted the clean median 23.35 → 24.35 ns (+4.3%, all 7
  runs separated: clean max 23.43 < mutated min 24.28); the mutated
  median sits past the 10 ns gate, so the gate flips as designed. Two
  vacuity asserts guard the arm: mutated > 10.0 gate and
  mutated > clean median. The court is not vacuous; the gate verdict
  FAIL is a real kernel cost, not a dead court.
- Timing variants explored and discarded this session (all measured,
  all reported in this receipt for the next lane): chunk 256 vs 1,365
  (no change: 23.1 vs 23.3), single-atom rule (23.38 — no change, so
  the cost is the per-call scan+radix pipeline, not join fan-out),
  subject/object pools 1365/8191 → 16384/65571 (23.13 → 23.35, join
  fan-out was never the cost), black_box(&cols) removed (no change).
  ~31.7 µs per batched call over 1,365 rows is the kernel's per-call
  scan+radix-build cost at this size.

## Divergences from dispatch

1. **Gate verdict is FAIL, not PASS.** The dispatch's 10.0 ns AMORTIZED
   gate is not meetable by the real `unify_batch_probe` kernel on this
   host — corroborated against bench-chains-bn3 above. Per no-rounding
   law the court lands with the typed FAIL (exit 101 in the test
   harness; the cscb-bin convention maps this to exit 1), method
   disclosure carried in the assertion message as dispatched, debug
   loud-skip exit 0 as dispatched. If the campaign wants a PASSing
   batch court at this batch size, the gate must be re-scoped (MV-5
   pattern) or the workload moved to the scan primitive
   (12.49 GB/s = 1.9 ns/24B-frame per bench-chains-bn3 — that surface
   can meet 10 ns/frame; the unify kernel cannot).
   [this line cites bench-chains-bn3.md:29-30]
2. **Mutation amplification is +1.0 ns, not multi-ns.** The +4.3%
   amplification is real (non-overlapping run ranges) but small because
   the kernel's baseline already dominates. The flip criterion is met
   (mutated > gate, mutated > clean); noted for calibration of future
   mutation loads on already-expensive baselines.
3. **csct's `[workspace]` line confirmed fresh**: `crates/csct/
   Cargo.toml:10` carries `cgap-bn3 = { path = "../cgap-bn3" }` as a
   normal dependency (B02's landed line), usable from integration
   tests without any Cargo.toml edit. No Cargo.toml bytes written.

## Fences my diff could break

- `crates/csct` standalone crate: tests/ addition only; existing courts
  (`csct_coverage_court.rs`, `csct_pipeline_court.rs`) untouched and
  not rebuilt-against. csct builds clean (both profiles built exit 0;
  the only warnings are pre-existing scan.rs/breach dead-code warnings
  in sibling bins).
- `crates/cgap-bn3` (read-only collaborator): no bytes written; my court
  consumes the stable `unify_batch_probe` signature. If B02/B03 lanes
  re-signature the kernel, my test file is the registered consumer that
  will catch it (intended fence, not regression).
- Shared root workspace: never built at root target; lease
  `/tmp/posh2-target` deleted at lane end.

## Standing

**PARTIAL** — the court, both profiles, and the anti-vacuity flip all
ran this session with exits recorded above (court ALIVE as a court);
the dispatched 10.0 ns gate is honestly FAIL (23.35 ns median, 2.3x
over, kernel cost corroborated by bench-chains-bn3). Debug loud-skip
exit 0 witnessed. UNVERIFIED beyond this host/macOS/release profile.
