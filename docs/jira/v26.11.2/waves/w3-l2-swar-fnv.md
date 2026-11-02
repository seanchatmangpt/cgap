# W3-L2 — word-wise SWAR FNV-1a in cgap-parquet footer

Date: 2026-11-01. Repo: /Users/sac/cgap (branch main, no git commands run).
Lane: W3-L2. Plan: `docs/jira/v26.11.2/plans/levers-L1-L5.md` §L2.

## Subject

`crates/cgap-parquet/src/footer.rs` — `fnv1a64_seeded` (was :119-127)
rewritten word-wise: prologue is folded into the bulk loop via
`split_at(len/8*8)` + byte-wise tail; bulk body consumes one 8-byte LE
word per iteration, FNV step unrolled per lane (`h ^= (w>>8k) & 0xFF;
h *= P`, k=0..8); epilogue byte-wise tail. Frozen constants untouched
(basis `0xcbf2_9ce4_8422_2325`, prime `0x0000_0100_0000_01B3`) — loop
structure only, byte-exact FNV-1a semantics.

## Commands run this session (exit codes)

- `CARGO_TARGET_DIR=/tmp/w3-l2-target cargo test -p cgap-parquet`
  pre-lever: all suites green (lever_courts 3/3), exit 0.
- Same command post-SWAR: 222 passed / 0 failed across 13 suites
  (incl. lever_courts 3/3, mutation_kills 17/17), exit 0 — W2-11
  (receipt: waves/w3-l2-swar-fnv.md)
  equity court byte-exact green
  (receipt: waves/w3-l2-swar-fnv.md).
- Re-run after concurrent-lane unblock edits: same 222/0, exit 0.
- `CARGO_TARGET_DIR=/tmp/w3-l2-target cargo bench -p bench --bench
  chains-parquet` exit 0. Full output: /tmp/w3-l2-bench.txt.
  - `chain_file` @1,048,576: fastest 42.82 ms / mean 43.1 ms.
  - `chain_ram` @1,048,576 @1M: fastest 42.58 ms / mean 42.74 ms.
  - Baseline 41.54 ms — chain did NOT move; **<10 ms target NOT met**
    (chain is dominated by encode/decode stages; the FNV share of the
    chain is much smaller than the plan assumed). Honest standing for
    the perf claim: TARGET NOT MET, measured, not rounding up.
- Sandbox (/tmp/w3-l2-sandbox, isolated workspace): baseline equity
  court green (3/3). M-B: lane-3 shift corrupted `(w >> 24)` ->
  `(w >> 25)`, equity court `l2_reference_equity_over_corpus`
  FAILED (1 failed / 2 passed, cargo exit 101). Restore: sha256
  (receipt: waves/w3-l2-swar-fnv.md).
  3e0c59736ee0d0f8ff357939c5b5439ac9aa34eadbbdfef207351c7c1c2c5a3a
  byte-exact + mtime bump; court green again (3/3). M-B FLIPS
  (receipt: waves/w3-l2-swar-fnv.md).
- M-A (committed-window byte flip -> `ChecksumMismatch{Body}`): the
  `l2_body_byte_flip_is_checksum_mismatch_body` court in
  tests/lever_courts.rs performs the flip on real sealed bytes and
  asserts the typed error; ran green on the shared tree this session
  (post-SWAR run above). M-A FLIPS (as witnessed pre-change too;
  receipt: waves/w3-l2-swar-fnv.md).

## Compile-freeze SLA disclosures

- mmap_writer.rs (another lane's in-flight RingSink): two corrupt
  lines broke the shared parquet compile; I applied a minimal unblock
  (`wraps()` returning `self.wraps`, `commit(self, len: usize)`
  signature). Disclosed for the owning lane's receipt.
- cgap-ffi/src/sovereign.rs FrameView init: I attempted a fix, but the
  owning lane (W3-R2) landed theirs first; the shared tree now carries
  the owning lane's fix. No change from me on that file
  (receipt: waves/w3-l2-swar-fnv.md).
- cgap-bn3 unify.rs: third lane mid-edit (`JoinProbeScratch`
  undefined); I did not touch it — owner landed within the SLA window
  (bounded poll, 7x30s, then green; receipt:
  waves/w3-l2-swar-fnv.md).

## Standing

- SWAR lever: ALIVE (landed, byte-exact equity green, both flips
  witnessed this session).
- <10 ms @1M chain target: NOT MET (measured 42.7-43.1 ms; see 02
  -pareto-levers.md re-measure row). The bottleneck attribution in
  the L2 plan (FNV as dominant chain share) is refuted by this
  receipt; the chain bottleneck is elsewhere (encode/decode stages).
