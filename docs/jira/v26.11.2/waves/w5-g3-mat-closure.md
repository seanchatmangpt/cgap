# w5-g3-mat-closure receipt (lane V1, Verify)

- Date: 2026-11-02. Branch: main. Shared tree untouched by courts (all
  mutations were /tmp sandbox copies with sha256-exact restore).
- Closes: G3-MAT-BENCH machinery gap
  (`docs/jira/v26.11.2/01-release-gates.md` G3-MAT-BENCH section; dangling
  `crates/bench/Cargo.toml:102` comment terminated).

## Files written (lane ownership)

- `crates/bench/Cargo.toml` (dangling W3-R4 comment terminated +
  `[[bench]] mat_closure` target added)
- `crates/bench/benches/mat_closure.rs` (new)

## Commands + exits

- `CARGO_TARGET_DIR=/tmp/v1-target cargo bench -p bench --bench mat_closure`
  -> EXIT 0. Verbatim summary:

    bucket N=65536 chain_len=8 base_facts=57344 closure_facts=229376 rounds=7 best_secs=0.020390 facts_per_sec_core=11249252
    bucket N=262144 chain_len=16 base_facts=245760 closure_facts=1966080 rounds=15 best_secs=0.238997 facts_per_sec_core=8226371
    bucket N=1048576 chain_len=32 base_facts=1015808 closure_facts=16252928 rounds=31 best_secs=3.352891 facts_per_sec_core=4847437

- Sandbox baseline: `cargo run --manifest-path /tmp/v1-sandbox/matmut/Cargo.toml`
  (CARGO_TARGET_DIR=/tmp/v1-sandbox-target) -> `closure_facts=16252928`, EXIT 0.
- Mutant: rounds>=2 killed in the /tmp copy of closure.rs ->
  `closure_facts=1998848`, EXIT 101 (`FACT-COUNT FLIP` assert fired).

## Mutant flip (anti-vacuity)

- Subject: /tmp/v1-sandbox/cgap-storage/src/closure.rs (copy of the shared
  kernel; shared tree never mutated).
- SHA256 before = after = `0df1264101c63c175e06043f57feaa5197f71dcdc11afe94fa5747b462920262`
  (sha256-exact restore; mtimes bumped post-restore).
- Baseline 16252928 facts exit 0 -> mutant 1998848 facts exit 101. The court
  FLIPS. In-bench fence also asserts closure_facts > mutant_base_only_count
  every bucket (would panic VACUOUS otherwise).

## Standing

- G3 machinery: LANDED (bench exists, ran this session, exit 0, court flips).
- G3 threshold ("> 50M closure facts/s/core"): still TARGET — measured
  4.85M-11.2M facts/s/core on this host; threshold NOT met.
- Comparison posture: measured cgap numbers vs PUBLISHED RDFox envelope
  (2-6.1M facts/s/core) only — no ratio claims, no GB/s conversion (unit
  law, 06-bench-protocol §7). Any RDFox-arm comparison remains a TARGET
  decision, not a measured result.
- Method: harness=false Instant, 1 warmup + 3 reps, min taken, single
  thread, same splitmix64 corpus (seed 0xA20C_0FFE_E5A7_E000) and buckets
  as benches/closure.rs workload shape.
