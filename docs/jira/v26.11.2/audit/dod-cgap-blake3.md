# DoD audit — cgap-blake3 (EXP-04, 2026-11-02)

Auditor: EXP-04 read-only lane. Repo /Users/sac/cgap, branch main.
Receipt of record: docs/jira/v26.11.2/audit/dod-cgap-blake3.md (this file).
No writes other than this receipt. /tmp/exp04-target left on disk (rm was
denied by permission system; cleanup owed by operator).

## T1 — B06 4.35M receipts/s methodology (verdict: HONEST, with 2 weak points)

Court source: crates/csct/src/bin/cscb_c7_receipt_sealing.rs
Lane receipt: docs/jira/v27.11.2/lanes/lane-b06-cscb-c7.md is
docs/jira/v26.11.2/lanes/lane-b06-cscb-c7.md (no v27 dir exists).

What the code does:
- N=7 timed runs, each sealing 1,000,000 receipts
  (cscb_c7_receipt_sealing.rs:32-33); min/median/p99 all reported in one
  typed line (lines 130-136); median is the gated statistic (line 139).
- Full chain-validity court over all 1M receipts, OUTSIDE the timed region
  (lines 77-96, 151-162): nonces strict from 1, parent==prev digest
  (genesis parent = zero digest), every digest re-derives via the real
  `cgap_blake3::hash`.
- Load-sensitivity DISCLOSED, not hidden: the contended run failed the
  1M/s floor at 727,755/s and the lane receipt records it typed
  (lane-b06-cscb-c7.md:25-27, 50-54). Debug build also typed-failed at
  204,511/s (lane-b06-cscb-c7.md:31-32). The gate is honest about being
  a release-profile, uncontended-box gate.

Weak points:
1. "p99" over N=7 is a mislabel: percentile() at p=99 with len 7 gives
   ceil(0.99*7)=7 -> the 7th sorted sample = the MAX, not a tail
   quantile. cscb_c7_receipt_sealing.rs:98-101,131. Honest samples, wrong
   name — should be reported as max.
2. The timed region includes splitmix64 payload gen + Vec push + parent
   chaining, not pure hash time (lines 113-124). Conservative (the
   reported rate UNDERSTATES hashing throughput), so it cannot inflate
   the claim — but it means 4.35M/s is a sealing-pipeline rate, not a
   BLAKE3 rate. Also single box, single process, no CI repro spec.

Verdict rationale: min/median/max reported, full-state chain court,
typed failures under contention and debug, conservative timing scope.
The figure cannot be vacuous given the mutation court and the witnessed
contended exit-1. HONEST. Two labeled weak points above.

## T2 — no_std + purity (verdict: PASS, exit 0)

- CARGO_TARGET_DIR=/tmp/exp04-target cargo check -p cgap-blake3
  --no-default-features -> exit 0 ("Finished dev profile").
  #![no_std] at src/lib.rs:8; features: default=[], optional std only
  (crates/cgap-blake3/Cargo.toml [features]; zero [dependencies]).
- Float arithmetic: grep f32/f64/float across src/ -> ZERO arithmetic
  hits. All matches are u32 NEON intrinsics (vget_low_u32 etc,
  multi_chunk.rs:169-172,401-408) or comment text (lib.rs:827,1058).
- Syscalls / std I/O: grep libc|std::fs|File|mmap|syscall|thread|
  Instant|SystemTime across src/ -> ZERO hits.
- Verdict: no floats, no syscalls in src/. PASS.

## T3 — anti-vacuity courts (verdict: PASS externally, GAP in-crate)

Crate's own tests/ (all 5 files: official_vectors, multi_chunk_vectors,
small_batch_vectors, thread_differential, zero_alloc): NO bitflip
rejection test and NO parent-digest-chain mutation test exist. The
closest in-crate coverage is structural only:
- official_vectors.rs:100 chunk_boundary_2048_two_chunk_parent_root
- official_vectors.rs:76 differential_hash_wide_simd_matches_hash_on_full_corpus
- thread_differential.rs:309 lane5_thread_ladder_8mib_differential
None flips a bit and asserts rejection. GAP: the mutation court lives
only in the csct bin, not cargo-testable in-crate.

External court (B06): crates/csct/src/bin/cscb_c7_receipt_sealing.rs
- chain rejection: verify_chain lines 77-96 (parent==prev digest,
  digest re-derivation), exit 1 typed.
- bitflip rejection, two layers: identical-digest typed FAIL
  (lines 186-190) and ACCEPTED-by-court typed FAIL (lines 191-196),
  plus land-check self-fail (174-182). Witnessed PASS this session
  (lane-b06-cscbC7 line 28-29; path lane-b06-cscb-c7.md:28-29).

## T4 — packaging (verdict: PASS mechanically; 26.11.3 republish OWED)

- CARGO_TARGET_DIR=/tmp/exp04-target cargo package -p cgap-blake3
  --allow-dirty (full verify) -> exit 0, "Packaged 14 files, 105.0KiB
  (26.3KiB compressed)", verify compile Finished, exit 0.
  (`--dry-run` is not a cargo package flag — exit 2 on that exact
  spelling; -l used to enumerate, full no-verify-free run used for
  verdict.)
- Metadata vs house style: name cgap-blake3, version 26.11.2
  (workspace), publish=true, license-file ../../LICENSE.md, zero
  dependencies, README.md + readme-archive/ shipped in tarball
  (14 files listed). Consistent with publish-halt.md:33-34 READY
  format.
- Law: docs/jira/v26.11.2/waves/publish-halt.md:39-42 — cgap-blake3
  26.11.3 republish is OWED on freeze-lift (NEON opt>=1 divergence
  D1). Corroborated: waves/p4-terminus.md:39 (D1 FIXED in tree,
  published 26.11.2 tarball still carries it, repair = 26.11.3 on
  lift); waves/w14-lift-register.md:31 item (d).
- F3 ledger: docs/jira/v26.11.2/waves/f3-publish-ledger.md DOES NOT
  EXIST in v26.11.2 (f3-* receipts live under v26.11.1/waves/).
  Closest v26.11.2 receipts: waves/pub-w1-witness.md:28 (exactly_once
  PASS, 26.11.2, crates.io) and waves/w14-lift-register.md:31. The
  task's cited ledger path is stale; standing owed is unchanged.

## Exit log (this session)

- cargo check -p cgap-blake3 --no-default-features -> exit 0
- cargo package -p cgap-blake3 --allow-dirty -> exit 0 (verify pass)
- cargo package --dry-run spelling -> exit 2 (flag does not exist)
- No crate source or doc files modified besides this receipt.
