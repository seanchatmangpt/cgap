# Repair 20: XOR-filter probe pre-gate (BH7 adopt, exact occupancy summary)

Lane TS3, 2026-11-02. Owner file: `crates/cgap-bn3/src/unify.rs` (probe pre-gate only;
RADIXFIX/BN3NS/BFX/BN3D untouched). Court file:
`crates/cgap-bn3/tests/repair20_xorfilter_gate.rs`.

## Design

BH7's adopt (bcinr `xorf.rs:28` membership gate before chained probes; `lazy.rs:486`
precedent) applied to the L3 radix probe. cgap's probe walks the chained candidates for
every probe key; an empty bucket still pays the `bucket_head[b]` fetch + sentinel
compare. The gate packs per-bucket occupancy (bit = `bucket_head[b] != u32::MAX`) into
64 words (4096 buckets / 64; 512 B total) in `JoinProbeScratch::bucket_occupancy`,
built during `bucket_candidates` from the REAL chain build (bit OR-set when a candidate
is pushed onto bucket b), and the probe loop tests the bit before the head fetch.

### Exactness argument

Bit=0 implies the bucket is provably empty because the bit is accumulated from the
actual chain build, not a probabilistic fingerprint. The byte-fingerprint OR-fold form
(OR of inserted key bytes into a summary word) was REJECTED as unsound: OR is a byte-set
union, and two distinct keys' byte union can contain a byte tuple that no single key
has, so a fingerprint bit=1 would not imply possible membership and bit=0 would not
imply absence. Only per-bucket occupancy read off the built chains is exact.

## Courts (crates/cgap-bn3/tests/repair20_xorfilter_gate.rs)

| # | Court | Property | Result |
|---|---|---|---|
| 1 | repair20_radix_gate_vs_nested_differential | gated radix vs ungated nested, match-set identity, N in {1,64,4096} x chunk in {1,64,4096} | PASS |
| 2 | repair20_radix_gate_vs_nested_100k | wide-pool 100_003 leg, match-set identity | PASS |
| 3 | repair20_occupancy_summary_exact | final-state assert: bit set iff head != u32::MAX, all 4096 buckets; plus non-vacuity (corpus must leave empty buckets) | PASS |

Full cgap-bn3 suite post-restore: 119 unit + 2 bgp + 7 lever + 3 repair20 = 131 passed,
0 failed (exit 0).

## Anti-vacuity (sandbox mutant R20-M1)

/tmp sandbox copy of cgap-bn3 + cgap-blake3 at `/tmp/r20-mut/`; mutant drops
`occ[b >> 6] |= 1u64 << (b & 63);` (bit never set => probe skips EVERY bucket).
`cargo test --test repair20_xorfilter_gate` on the mutant: FAILED, 0 passed / 3 failed
(left: 0, right: 153230) -- the court flips; a court that cannot flip is FAIL, this one
flips.

## Bench (divan, stage_unify_join_radix, same machine/session, min/median of 100)

Baseline (gate temporarily removed via /tmp snapshot swap, sha256-verified restore
1f2e8db8...d40062, mtime bumped, courts re-run green after restore):

| N | baseline (no gate) | gated | delta |
|---|---|---|---|
| 65,536 | 1.518 ms median (1.489 fastest) | 1.577 ms median (1.462 fastest) | +3.9% median |
| 1,048,576 | 24.37 ms median (24.17 fastest) | 24.85 ms median (23.78 fastest) | +2.0% median |

Second gated run on restored tree: 65,536 median 1.589 ms; 1,048,576 median 25.14 ms,
fastest 23.79 ms. Run-to-run spread is +-3-5%; verdict: PARITY WITHIN NOISE at both N.
The 22.99 ms fence number is from a different session/configuration and was NOT
reproduced this session (both arms measured 24.2-25.3 ms median) -- the fence is
recorded UNVERIFIED against this session's method, not claimed met. The gate's win case
(probe keys landing in empty buckets) is largely ABSENT from this corpus: parents=4096
against RADIX_BUCKETS=4096 means buckets are near-fully occupied, so the gate adds a
512 B summary read with almost nothing to skip. Honest delta: parity-to-slight-overhead
on this dense corpus; the win case needs a corpus with sparse occupancy (skipped-bucket
count is observable via a counter, not added here).
